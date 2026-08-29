//! Routing across the config/adapter boundary: which of native, fallback,
//! and unsupported a dispatch picks, and what each adapter puts on the wire.

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use keylex::adapters::{SocketAdapter, WebSocketAdapter};
use keylex::config::{AdapterKind, KeyCombo, Registry, Target};
use keylex::dispatch::{Adapter, Adapters, FallbackSender, Notifier, Outcome, Router};

fn write_config(name: &str, targets: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("keylex-test-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("targets.toml"), targets).unwrap();
    dir
}

fn load(name: &str, targets: &str) -> Registry {
    let dir = write_config(name, targets);
    let registry = Registry::load(&dir).expect("fixture config should load");
    let _ = std::fs::remove_dir_all(&dir);
    registry
}

type Log<T> = Rc<RefCell<Vec<T>>>;

/// What a dispatch reached, recorded instead of actually sent.
#[derive(Default)]
struct Recorder {
    commands: Log<(String, String)>,
    notifications: Log<String>,
    keycodes: Log<String>,
}

impl Recorder {
    fn router<'a>(&self, registry: &'a Registry, adapter: Option<AdapterKind>) -> Router<'a> {
        let mut adapters = Adapters::new();
        if let Some(kind) = adapter {
            adapters.insert(kind, Box::new(RecordingAdapter(Rc::clone(&self.commands))));
        }
        Router::new(
            registry,
            adapters,
            Box::new(RecordingNotifier(Rc::clone(&self.notifications))),
            Box::new(RecordingFallback(Rc::clone(&self.keycodes))),
        )
    }
}

struct RecordingAdapter(Log<(String, String)>);
impl Adapter for RecordingAdapter {
    fn send(&self, target: &Target, native_command: &str) {
        self.0
            .borrow_mut()
            .push((target.program.clone(), native_command.to_string()));
    }
}

struct RecordingNotifier(Log<String>);
impl Notifier for RecordingNotifier {
    fn show(&self, message: &str) {
        self.0.borrow_mut().push(message.to_string());
    }
}

struct RecordingFallback(Log<String>);
impl FallbackSender for RecordingFallback {
    fn send(&self, combo: &KeyCombo) {
        self.0.borrow_mut().push(combo.to_string());
    }
}

const VSCODE: &str = r#"[[target]]
program = "vscode"
match_process = ["Code.exe"]
adapter = "socket"
"#;

/// There's no static per-target `supports` map any more (see CLAUDE.md's
/// "Known gaps") -- `Router::send_native` is the actual native-dispatch
/// primitive now, the same one `spotlight::Entry::dispatch` calls once it
/// already has a live-reported command. `Router::dispatch`'s own
/// action-id-based native path is untested here because it's currently
/// unreachable dead weight: nothing populates `Registry`'s action table,
/// so it always falls straight through to the fallback tests below.
#[test]
fn send_native_delivers_straight_to_the_named_target() {
    let registry = load("native", VSCODE);
    let recorder = Recorder::default();
    let router = recorder.router(&registry, Some(AdapterKind::Socket));

    let outcome = router.send_native(&registry.targets()[0], "workbench.action.closeActiveEditor");

    assert_eq!(
        outcome,
        Outcome::Native("workbench.action.closeActiveEditor".to_string())
    );
    assert_eq!(
        recorder.commands.borrow().as_slice(),
        [(
            "vscode".to_string(),
            "workbench.action.closeActiveEditor".to_string()
        )]
    );
}

#[test]
fn a_missing_adapter_is_reported_rather_than_silently_dropped() {
    let registry = load("adapter-missing", VSCODE);
    let recorder = Recorder::default();

    let outcome = recorder
        .router(&registry, None)
        .send_native(&registry.targets()[0], "workbench.action.closeActiveEditor");

    assert!(matches!(outcome, Outcome::Unsupported(_)));
}

/// With no action populated in `Registry` (see CLAUDE.md's "Known gaps"),
/// `Router::dispatch` always falls through to this path -- covered here
/// since the config-file-driven fallback-tier tests that used to exercise
/// it were retired along with `actions.toml` (see `src/dispatch.rs`'s own
/// unit tests for the fallback-tier behavior itself).
#[test]
fn dispatch_with_no_configured_action_is_unsupported() {
    let registry = load("no-action", VSCODE);
    let recorder = Recorder::default();

    let outcome = recorder
        .router(&registry, Some(AdapterKind::Socket))
        .dispatch("close.tab", Some("Code.exe"));

    assert!(matches!(outcome, Outcome::Unsupported(_)));
}

#[test]
fn the_socket_adapter_asks_a_target_for_its_action_catalog() {
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpListener;

    let listener = TcpListener::bind("127.0.0.1:0").expect("fake target should bind");
    let port = listener.local_addr().unwrap().port();

    let target = std::thread::spawn(move || {
        let (stream, _) = listener.accept().expect("the daemon should connect");
        let mut line = String::new();
        BufReader::new(&stream)
            .read_line(&mut line)
            .expect("a request line");
        let request: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
        assert_eq!(request["type"], "list_actions");

        let response = serde_json::json!({
            "actions": [{
                "id": "close.tab",
                "native_command": "workbench.action.closeActiveEditor",
                "title": "Close Editor",
            }]
        });
        writeln!(&stream, "{response}").unwrap();
    });

    let registry = load(
        "handshake",
        &format!("[[target]]\nprogram = \"vscode\"\nadapter = \"socket\"\naddress = \"127.0.0.1:{port}\"\n"),
    );
    let actions = SocketAdapter::new()
        .fetch_actions(&registry.targets()[0])
        .expect("handshake should succeed");

    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].id, "close.tab");
    assert_eq!(actions[0].title, "Close Editor");
    target.join().unwrap();
}

#[test]
fn a_silent_target_leaves_the_catalog_alone() {
    let registry = load(
        "unreachable",
        // Nothing is listening on this loopback port.
        "[[target]]\nprogram = \"vscode\"\nadapter = \"socket\"\naddress = \"127.0.0.1:1\"\n",
    );

    assert!(SocketAdapter::new()
        .fetch_actions(&registry.targets()[0])
        .is_none());
}

type Client = tungstenite::WebSocket<tungstenite::stream::MaybeTlsStream<std::net::TcpStream>>;

/// Connects a fake client and returns it once the adapter is listening.
fn connect(port: u16) -> Client {
    loop {
        if let Ok((client, _response)) = tungstenite::connect(format!("ws://127.0.0.1:{port}")) {
            return client;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

#[test]
fn the_websocket_adapter_delivers_to_a_connected_client() {
    const PORT: u16 = 47778;

    let registry = load(
        "websocket",
        &format!(
            "[[target]]\nprogram = \"chrome\"\nmatch_process = [\"chrome.exe\"]\nadapter = \"websocket\"\nport = {PORT}\n"
        ),
    );
    let adapter = WebSocketAdapter::spawn(PORT, None).expect("should bind");

    let mut client = connect(PORT);
    // Let the accept thread promote the connection before dispatching.
    std::thread::sleep(std::time::Duration::from_millis(50));

    let mut adapters = Adapters::new();
    adapters.insert(AdapterKind::WebSocket, Box::new(adapter));
    let router = Router::new(
        &registry,
        adapters,
        Box::new(RecordingNotifier(Log::default())),
        Box::new(RecordingFallback(Log::default())),
    );

    assert!(matches!(
        router.send_native(&registry.targets()[0], "chrome.tab.close"),
        Outcome::Native(_)
    ));

    let message = client
        .read()
        .expect("the client should receive the command");
    let parsed: serde_json::Value =
        serde_json::from_str(&message.into_text().unwrap()).expect("JSON");
    assert_eq!(parsed["command"], "chrome.tab.close");
}

#[test]
fn the_websocket_adapter_rejects_a_handshake_from_another_origin() {
    const PORT: u16 = 47781;
    let _adapter = WebSocketAdapter::spawn(
        PORT,
        Some("chrome-extension://expected-id".to_string()),
    )
    .expect("should bind");

    while std::net::TcpStream::connect(("127.0.0.1", PORT)).is_err() {
        std::thread::sleep(std::time::Duration::from_millis(20));
    }

    let request = tungstenite::client::ClientRequestBuilder::new(
        format!("ws://127.0.0.1:{PORT}").parse().unwrap(),
    )
    .with_header("Origin", "https://evil.example");

    assert!(tungstenite::connect(request).is_err());
}

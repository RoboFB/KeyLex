//! Routing across the config/adapter boundary: which of native, fallback,
//! and unsupported a dispatch picks, and what each adapter puts on the wire.

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use keylex::adapters::{SocketAdapter, WebSocketAdapter};
use keylex::config::{AdapterKind, KeyCombo, Registry, Target};
use keylex::dispatch::{Adapter, Adapters, FallbackSender, Notifier, Outcome, Router};

/// Covers every word the fixtures below build an action id from.
const VOCABULARY: &str = r#"
modifiers = ["close", "save", "duplicate", "go_to", "move"]
locations = ["tab", "line", "definition", "left"]
"#;

fn write_config(name: &str, actions: &str, targets: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("keylex-test-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("vocabulary.toml"), VOCABULARY).unwrap();
    std::fs::write(dir.join("actions.toml"), actions).unwrap();
    std::fs::write(dir.join("targets.toml"), targets).unwrap();
    dir
}

fn load(name: &str, actions: &str, targets: &str) -> Registry {
    let dir = write_config(name, actions, targets);
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

const CLOSE_TAB: &str = "[[action]]\nmodifier = \"close\"\nlocation = \"tab\"\n";
const VSCODE: &str = r#"[[target]]
program = "vscode"
match_process = ["Code.exe"]
adapter = "socket"
exempt_command_grammar = true

  [target.supports]
  "close.tab" = "workbench.action.closeActiveEditor"
"#;

#[test]
fn a_focused_target_that_supports_the_action_gets_it_natively() {
    let registry = load("native", CLOSE_TAB, VSCODE);
    let recorder = Recorder::default();
    let router = recorder.router(&registry, Some(AdapterKind::Socket));

    let outcome = router.dispatch("close.tab", Some("Code.exe"));

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
    let registry = load("adapter-missing", CLOSE_TAB, VSCODE);
    let recorder = Recorder::default();

    let outcome = recorder
        .router(&registry, None)
        .dispatch("close.tab", Some("Code.exe"));

    assert!(matches!(outcome, Outcome::Unsupported(_)));
}

#[test]
fn the_silent_tier_sends_a_keycode_without_notifying() {
    let registry = load(
        "silent",
        "[[action]]\nmodifier = \"save\"\nfallback_tier = \"silent\"\nfallback_keycode = \"ctrl+s\"\n",
        "",
    );
    let recorder = Recorder::default();

    let outcome = recorder
        .router(&registry, None)
        .dispatch("save", Some("Code.exe"));

    assert_eq!(outcome.to_string(), "fallback: ctrl+s");
    assert_eq!(
        recorder.keycodes.borrow().as_slice(),
        ["ctrl+s".to_string()]
    );
    assert!(recorder.notifications.borrow().is_empty());
}

#[test]
fn the_notify_attempt_tier_sends_a_keycode_and_notifies() {
    let registry = load(
        "notify-attempt",
        "[[action]]\nmodifier = \"duplicate\"\nlocation = \"line\"\nfallback_keycode = \"ctrl+shift+d\"\n",
        "",
    );
    let recorder = Recorder::default();

    let outcome = recorder
        .router(&registry, None)
        .dispatch("duplicate.line", None);

    assert!(matches!(outcome, Outcome::Fallback(_)));
    assert_eq!(
        recorder.keycodes.borrow().as_slice(),
        ["ctrl+shift+d".to_string()]
    );
    assert_eq!(recorder.notifications.borrow().len(), 1);
}

#[test]
fn an_action_with_nothing_safe_to_send_only_notifies() {
    let registry = load(
        "notify-only",
        "[[action]]\nmodifier = \"go_to\"\nlocation = \"definition\"\nfallback_tier = \"notify_only\"\n",
        "",
    );
    let recorder = Recorder::default();

    let outcome = recorder
        .router(&registry, None)
        .dispatch("go_to.definition", Some("chrome.exe"));

    assert!(matches!(outcome, Outcome::Unsupported(_)));
    assert!(recorder.keycodes.borrow().is_empty());
    assert_eq!(recorder.notifications.borrow().len(), 1);
}

/// The OS-wide listener catches actions no focused app claims -- but never
/// ahead of an app that does claim them.
#[test]
fn the_system_target_is_the_fallback_before_the_keycode_not_before_the_app() {
    let system = format!(
        r#"[[target]]
program = "system-{os}"
os = "{os}"
adapter = "socket"

  [target.supports]
  "move.left" = "os.window.move_left"
  "close.tab" = "os.window.unused"
"#,
        os = std::env::consts::OS
    );
    let registry = load(
        "system-target",
        &format!("{CLOSE_TAB}[[action]]\nmodifier = \"move\"\nlocation = \"left\"\nfallback_tier = \"notify_only\"\n"),
        &format!("{VSCODE}\n{system}"),
    );
    let recorder = Recorder::default();
    let router = recorder.router(&registry, Some(AdapterKind::Socket));

    assert_eq!(
        router.dispatch("move.left", Some("unknown.exe")),
        Outcome::Native("os.window.move_left".to_string())
    );
    assert_eq!(
        router.dispatch("close.tab", Some("Code.exe")),
        Outcome::Native("workbench.action.closeActiveEditor".to_string()),
        "the focused app should win over the system listener"
    );
}

#[test]
fn the_socket_adapter_asks_a_target_for_its_action_catalog() {
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpListener;

    const TOKEN: &str = "handshake-token";
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
        assert_eq!(request["token"], TOKEN);

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
        CLOSE_TAB,
        &format!("[[target]]\nprogram = \"vscode\"\nadapter = \"socket\"\naddress = \"127.0.0.1:{port}\"\n"),
    );
    let actions = SocketAdapter::new(TOKEN.to_string())
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
        CLOSE_TAB,
        // Nothing is listening on this loopback port.
        "[[target]]\nprogram = \"vscode\"\nadapter = \"socket\"\naddress = \"127.0.0.1:1\"\n",
    );

    assert!(SocketAdapter::new("token".to_string())
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

fn send_token(client: &mut Client, token: &str) {
    let frame = serde_json::json!({ "token": token }).to_string();
    client
        .send(tungstenite::Message::Text(frame.into()))
        .expect("auth frame should send");
}

#[test]
fn the_websocket_adapter_delivers_to_an_authenticated_client() {
    const PORT: u16 = 47778;
    const TOKEN: &str = "test-token";

    let registry = load(
        "websocket",
        CLOSE_TAB,
        &format!(
            "[[target]]\nprogram = \"chrome\"\nmatch_process = [\"chrome.exe\"]\nadapter = \"websocket\"\nport = {PORT}\n\n[target.supports]\n\"close.tab\" = \"chrome.tab.close\"\n"
        ),
    );
    let adapter = WebSocketAdapter::spawn(PORT, TOKEN.to_string(), None).expect("should bind");

    let mut client = connect(PORT);
    send_token(&mut client, TOKEN);
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
        router.dispatch("close.tab", Some("chrome.exe")),
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
fn the_websocket_adapter_drops_a_client_with_the_wrong_token() {
    const PORT: u16 = 47780;
    let _adapter = WebSocketAdapter::spawn(PORT, "correct".to_string(), None).expect("should bind");

    let mut client = connect(PORT);
    send_token(&mut client, "wrong");

    // Read on another thread, so a regression that leaves the connection
    // open fails the test instead of hanging it.
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || tx.send(client.read()));
    let read = rx
        .recv_timeout(std::time::Duration::from_secs(5))
        .expect("the server should close the connection, not leave it open");

    assert!(read.is_err());
}

#[test]
fn the_websocket_adapter_rejects_a_handshake_from_another_origin() {
    const PORT: u16 = 47781;
    let _adapter = WebSocketAdapter::spawn(
        PORT,
        "correct".to_string(),
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

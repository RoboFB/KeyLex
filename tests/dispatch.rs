//! Port of the previous tests/test_router.py cases: native dispatch,
//! fallback tiers, and the two "unsupported" paths.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use keylex::adapters::WebSocketAdapter;
use keylex::config::{Registry, Target};
use keylex::dispatch::{Adapter, DispatchStatus, FallbackSender, Notifier, Router};

fn temp_config_dir(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("keylex-test-{name}-{}-{:?}", std::process::id(), std::thread::current().id()))
}

/// Covers every modifier/location these fixture tests bind an action to.
const TEST_VOCABULARY: &str = "application,group,command_id,default_hotkey,comment,modifier,location,condition\n\
    Test,Test,x,x,x,close,tab,\n\
    Test,Test,x,x,x,save,line,\n\
    Test,Test,x,x,x,duplicate,definition,\n\
    Test,Test,x,x,x,go_to,left,\n\
    Test,Test,x,x,x,move,,\n";

fn write_config(actions_toml: &str, targets_toml: &str, name: &str) -> PathBuf {
    let dir = temp_config_dir(name);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("hotkeys-reference.csv"), TEST_VOCABULARY).unwrap();
    std::fs::write(dir.join("actions.toml"), actions_toml).unwrap();
    std::fs::write(dir.join("targets.toml"), targets_toml).unwrap();
    dir
}

fn load(dir: &Path) -> Registry {
    Registry::load(dir).expect("fixture config should load")
}

#[derive(Default)]
struct Recording {
    notifier_messages: Rc<RefCell<Vec<String>>>,
    fallback_sent: Rc<RefCell<Vec<String>>>,
}

struct RecordingAdapter(Rc<RefCell<Vec<(String, String)>>>);
impl Adapter for RecordingAdapter {
    fn send(&self, target: &Target, native_command: &str) {
        self.0.borrow_mut().push((target.program.clone(), native_command.to_string()));
    }
}

struct RecordingNotifier(Rc<RefCell<Vec<String>>>);
impl Notifier for RecordingNotifier {
    fn show(&self, message: &str) {
        self.0.borrow_mut().push(message.to_string());
    }
}

struct RecordingFallbackSender(Rc<RefCell<Vec<String>>>);
impl FallbackSender for RecordingFallbackSender {
    fn send(&self, keycode: &str) {
        self.0.borrow_mut().push(keycode.to_string());
    }
}

fn router_with<'a>(
    registry: &'a Registry,
    adapters: HashMap<String, Box<dyn Adapter>>,
) -> (Router<'a>, Recording) {
    let recording = Recording::default();
    let router = Router {
        registry,
        adapters,
        notifier: Box::new(RecordingNotifier(Rc::clone(&recording.notifier_messages))),
        fallback_sender: Box::new(RecordingFallbackSender(Rc::clone(&recording.fallback_sent))),
    };
    (router, recording)
}

#[test]
fn dispatch_uses_native_adapter_when_target_supports_action() {
    let dir = write_config(
        "[[action]]\nmodifier = \"close\"\nlocation = \"tab\"\n",
        r#"[[target]]
program = "vscode"
match_process = ["Code.exe"]
adapter = "socket"
exempt_command_grammar = true

  [target.supports]
  "close.tab" = "workbench.action.closeActiveEditor"
"#,
        "native",
    );
    let registry = load(&dir);

    let calls = Rc::new(RefCell::new(Vec::new()));
    let mut adapters: HashMap<String, Box<dyn Adapter>> = HashMap::new();
    adapters.insert("socket".to_string(), Box::new(RecordingAdapter(Rc::clone(&calls))));
    let (router, _recording) = router_with(&registry, adapters);

    let result = router.dispatch("close.tab", "Code.exe");

    assert_eq!(result.status, DispatchStatus::Native);
    assert_eq!(result.detail, "workbench.action.closeActiveEditor");
    assert_eq!(
        calls.borrow().as_slice(),
        [("vscode".to_string(), "workbench.action.closeActiveEditor".to_string())]
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn dispatch_falls_back_when_target_does_not_support_action() {
    let dir = write_config(
        r#"[[action]]
modifier = "save"
fallback_tier = "silent"
fallback_keycode = "ctrl+s"
"#,
        r#"[[target]]
program = "vscode"
match_process = ["Code.exe"]
adapter = "socket"
"#,
        "fallback-silent",
    );
    let registry = load(&dir);

    let (router, recording) = router_with(&registry, HashMap::new());
    let result = router.dispatch("save", "Code.exe");

    assert_eq!(result.status, DispatchStatus::Fallback);
    assert_eq!(result.detail, "ctrl+s");
    assert_eq!(recording.fallback_sent.borrow().as_slice(), ["ctrl+s".to_string()]);
    assert!(recording.notifier_messages.borrow().is_empty(), "silent tier: no popup");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn dispatch_notifies_on_fallback_attempt_tier() {
    let dir = write_config(
        r#"[[action]]
modifier = "duplicate"
location = "line"
fallback_tier = "notify_attempt"
fallback_keycode = "ctrl+shift+d"
"#,
        "",
        "fallback-notify-attempt",
    );
    let registry = load(&dir);

    let (router, recording) = router_with(&registry, HashMap::new());
    let result = router.dispatch("duplicate.line", "unknown.exe");

    assert_eq!(result.status, DispatchStatus::Fallback);
    assert_eq!(recording.fallback_sent.borrow().as_slice(), ["ctrl+shift+d".to_string()]);
    assert_eq!(recording.notifier_messages.borrow().len(), 1);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn dispatch_reports_unsupported_when_notify_only_and_no_target() {
    let dir = write_config(
        r#"[[action]]
modifier = "go_to"
location = "definition"
fallback_tier = "notify_only"
"#,
        "",
        "notify-only",
    );
    let registry = load(&dir);

    let (router, recording) = router_with(&registry, HashMap::new());
    let result = router.dispatch("go_to.definition", "chrome.exe");

    assert_eq!(result.status, DispatchStatus::Unsupported);
    assert_eq!(recording.notifier_messages.borrow().len(), 1);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn dispatch_reports_unsupported_when_native_adapter_missing() {
    let dir = write_config(
        "[[action]]\nmodifier = \"close\"\nlocation = \"tab\"\n",
        r#"[[target]]
program = "vscode"
match_process = ["Code.exe"]
adapter = "socket"
exempt_command_grammar = true

  [target.supports]
  "close.tab" = "workbench.action.closeActiveEditor"
"#,
        "adapter-missing",
    );
    let registry = load(&dir);

    let (router, _recording) = router_with(&registry, HashMap::new()); // no "socket" adapter registered
    let result = router.dispatch("close.tab", "Code.exe");

    assert_eq!(result.status, DispatchStatus::Unsupported);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn dispatch_uses_system_target_when_no_focused_target_supports_action() {
    let dir = write_config(
        "[[action]]\nmodifier = \"move\"\nlocation = \"left\"\n",
        &format!(
            r#"[[target]]
program = "system-{os}"
os = "{os}"
adapter = "socket"

  [target.supports]
  "move.left" = "os.window.move_left"
"#,
            os = std::env::consts::OS
        ),
        "system-target",
    );
    let registry = load(&dir);

    let calls = Rc::new(RefCell::new(Vec::new()));
    let mut adapters: HashMap<String, Box<dyn Adapter>> = HashMap::new();
    adapters.insert("socket".to_string(), Box::new(RecordingAdapter(Rc::clone(&calls))));
    let (router, _recording) = router_with(&registry, adapters);

    // No app is focused with a matching target at all -- "unknown.exe" -- so
    // this should only succeed via the OS-wide system target.
    let result = router.dispatch("move.left", "unknown.exe");

    assert_eq!(result.status, DispatchStatus::Native);
    assert_eq!(result.detail, "os.window.move_left");
    assert_eq!(
        calls.borrow().as_slice(),
        [(format!("system-{}", std::env::consts::OS), "os.window.move_left".to_string())]
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn dispatch_prefers_focused_app_target_over_system_target() {
    let dir = write_config(
        "[[action]]\nmodifier = \"close\"\nlocation = \"tab\"\n",
        &format!(
            r#"[[target]]
program = "vscode"
match_process = ["Code.exe"]
adapter = "socket"
exempt_command_grammar = true

  [target.supports]
  "close.tab" = "workbench.action.closeActiveEditor"

[[target]]
program = "system-{os}"
os = "{os}"
adapter = "socket"

  [target.supports]
  "close.tab" = "os.window.unused"
"#,
            os = std::env::consts::OS
        ),
        "system-target-precedence",
    );
    let registry = load(&dir);

    let calls = Rc::new(RefCell::new(Vec::new()));
    let mut adapters: HashMap<String, Box<dyn Adapter>> = HashMap::new();
    adapters.insert("socket".to_string(), Box::new(RecordingAdapter(Rc::clone(&calls))));
    let (router, _recording) = router_with(&registry, adapters);

    let result = router.dispatch("close.tab", "Code.exe");

    assert_eq!(result.status, DispatchStatus::Native);
    assert_eq!(result.detail, "workbench.action.closeActiveEditor");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn websocket_adapter_delivers_command_to_connected_client() {
    const PORT: u16 = 47778;

    let dir = write_config(
        "[[action]]\nmodifier = \"close\"\nlocation = \"tab\"\n",
        r#"[[target]]
program = "chrome"
match_process = ["chrome.exe"]
adapter = "websocket"
port = 47778

  [target.supports]
  "close.tab" = "chrome.tab.close"
"#,
        "websocket",
    );
    let registry = load(&dir);

    const TOKEN: &str = "test-token";
    let ws_adapter = WebSocketAdapter::spawn(PORT, TOKEN.to_string(), None)
        .expect("websocket adapter should bind");

    // Give the accept-loop thread a moment to start listening before the
    // fake client tries to connect.
    let mut client = loop {
        match tungstenite::connect(format!("ws://127.0.0.1:{PORT}")) {
            Ok((client, _response)) => break client,
            Err(_) => std::thread::sleep(std::time::Duration::from_millis(20)),
        }
    };

    // The adapter won't promote this connection into its live slot until it
    // sees a matching auth frame (docs/protocol.md#trust-model--authentication).
    client
        .send(tungstenite::Message::Text(
            serde_json::json!({ "token": TOKEN }).to_string().into(),
        ))
        .expect("fake client should send the auth frame");

    // Let the adapter's accept thread register the connection before
    // dispatching, so `send` doesn't race the handshake completing.
    std::thread::sleep(std::time::Duration::from_millis(50));

    let mut adapters: HashMap<String, Box<dyn Adapter>> = HashMap::new();
    adapters.insert("websocket".to_string(), Box::new(ws_adapter));
    let (router, _recording) = router_with(&registry, adapters);

    let result = router.dispatch("close.tab", "chrome.exe");
    assert_eq!(result.status, DispatchStatus::Native);

    let message = client
        .read()
        .expect("fake client should receive the dispatched command");
    let text = message.into_text().expect("message should be text");
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("message should be JSON");
    assert_eq!(parsed["command"], "chrome.tab.close");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn websocket_adapter_closes_connection_on_wrong_token() {
    const PORT: u16 = 47780;

    let _ws_adapter = WebSocketAdapter::spawn(PORT, "correct-token".to_string(), None)
        .expect("websocket adapter should bind");

    let mut client = loop {
        match tungstenite::connect(format!("ws://127.0.0.1:{PORT}")) {
            Ok((client, _response)) => break client,
            Err(_) => std::thread::sleep(std::time::Duration::from_millis(20)),
        }
    };

    client
        .send(tungstenite::Message::Text(
            serde_json::json!({ "token": "wrong-token" }).to_string().into(),
        ))
        .expect("fake client should send the (wrong) auth frame");

    // The adapter should reject and close the connection rather than
    // promoting it -- read on a separate thread so a would-be regression
    // (connection left open, no data ever arrives) fails the test instead
    // of hanging it forever.
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(client.read());
    });
    let result = rx
        .recv_timeout(std::time::Duration::from_secs(5))
        .expect("server should close the connection after rejecting the wrong token");
    assert!(result.is_err(), "connection should be closed, not left open");
}

#[test]
fn websocket_adapter_rejects_handshake_from_disallowed_origin() {
    const PORT: u16 = 47781;

    let _ws_adapter = WebSocketAdapter::spawn(
        PORT,
        "correct-token".to_string(),
        Some("chrome-extension://expected-id".to_string()),
    )
    .expect("websocket adapter should bind");

    // Wait for the listener to actually be up before attempting the (single,
    // expected-to-fail) handshake.
    loop {
        match std::net::TcpStream::connect(("127.0.0.1", PORT)) {
            Ok(_) => break,
            Err(_) => std::thread::sleep(std::time::Duration::from_millis(20)),
        }
    }

    let request = tungstenite::client::ClientRequestBuilder::new(
        format!("ws://127.0.0.1:{PORT}").parse().unwrap(),
    )
    .with_header("Origin", "https://evil.example");

    let result = tungstenite::connect(request);
    assert!(
        result.is_err(),
        "handshake from a disallowed Origin should have been rejected"
    );
}

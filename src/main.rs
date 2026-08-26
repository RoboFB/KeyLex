//! Keylex daemon: loads config, wires the router to adapters and the
//! platform-specific keyboard capture backend.
//!
//! `keylex` starts the real, blocking capture loop (Linux: evdev/uinput,
//! Windows: WH_KEYBOARD_LL hook). `--demo` skips capture entirely and
//! dispatches two example actions once -- useful for a quick smoke test
//! with no real hardware/permissions needed.

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::ExitCode;

use keylex::adapters::{SocketAdapter, WebSocketAdapter};
use keylex::config::Registry;
use keylex::dispatch::{Adapter, FallbackSender, Notifier, Router};

struct LogNotifier;

impl Notifier for LogNotifier {
    fn show(&self, message: &str) {
        println!("[notify] {message}");
    }
}

struct LogFallbackSender;

impl FallbackSender for LogFallbackSender {
    fn send(&self, keycode: &str) {
        println!("[fallback keycode] {keycode}");
    }
}

/// Any `[[target]]` in `targets.toml` configured with `adapter = "websocket"`
/// needs a live server spawned for it (see `keylex::adapters::WebSocketAdapter`
/// -- the daemon is the WS *server*, the target's client, e.g. the Chrome
/// extension, connects in). v1 only expects one such target; if more than
/// one is configured they'd need distinct ports, but only the first is
/// spawned today. Also reads the target's optional `allowed_origin`
/// (docs/protocol.md#trust-model--authentication).
fn websocket_target_config(registry: &Registry) -> Option<(u16, Option<String>)> {
    let target = registry.targets.iter().find(|t| t.adapter == "websocket")?;
    let port = target
        .extra
        .get("port")
        .and_then(|v| v.as_integer())
        .and_then(|p| u16::try_from(p).ok())?;
    Some((port, target.allowed_origin().map(str::to_string)))
}

fn build_adapters(token: &str, websocket_adapter: Option<WebSocketAdapter>) -> HashMap<String, Box<dyn Adapter>> {
    let mut adapters: HashMap<String, Box<dyn Adapter>> = HashMap::new();
    adapters.insert("socket".to_string(), Box::new(SocketAdapter::new(token.to_string())));
    if let Some(ws) = websocket_adapter {
        adapters.insert("websocket".to_string(), Box::new(ws));
    }
    // "rpc" (Neovim) adapter follows later.
    adapters
}

fn run_demo(registry: &Registry, token: &str) {
    let router = Router {
        registry,
        adapters: build_adapters(token, None),
        notifier: Box::new(LogNotifier),
        fallback_sender: Box::new(LogFallbackSender),
    };

    println!("close.tab -> {}", router.dispatch("close.tab", "code"));
    println!("go_to.definition -> {}", router.dispatch("go_to.definition", "chrome.exe"));
}

fn main() -> ExitCode {
    let mut demo = false;
    let mut config_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config");

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--demo" => demo = true,
            "--config-dir" => {
                let Some(dir) = args.next() else {
                    eprintln!("keylex: --config-dir requires a path");
                    return ExitCode::FAILURE;
                };
                config_dir = PathBuf::from(dir);
            }
            other => {
                eprintln!("keylex: unknown argument {other:?}");
                return ExitCode::FAILURE;
            }
        }
    }

    let registry = match Registry::load(&config_dir) {
        Ok(registry) => registry,
        Err(e) => {
            eprintln!("keylex: failed to load config from {}: {e}", config_dir.display());
            return ExitCode::FAILURE;
        }
    };

    let token = match keylex::auth::load_or_create_token(&config_dir) {
        Ok(token) => token,
        Err(e) => {
            eprintln!("keylex: failed to load or create auth token in {}: {e}", config_dir.display());
            return ExitCode::FAILURE;
        }
    };

    if demo {
        run_demo(&registry, &token);
        return ExitCode::SUCCESS;
    }

    let websocket_adapter = match websocket_target_config(&registry) {
        Some((port, allowed_origin)) => match WebSocketAdapter::spawn(port, token.clone(), allowed_origin) {
            Ok(adapter) => Some(adapter),
            Err(e) => {
                eprintln!("keylex: failed to start websocket adapter on port {port}: {e}");
                None
            }
        },
        None => None,
    };

    match keylex::capture::run(&registry, build_adapters(&token, websocket_adapter), Box::new(LogNotifier)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("keylex: capture loop stopped: {e}");
            ExitCode::FAILURE
        }
    }
}

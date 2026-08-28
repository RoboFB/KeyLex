//! Keylex daemon: loads config, wires the router to adapters and the
//! platform-specific keyboard capture backend.
//!
//! `keylex` starts the real, blocking capture loop (Linux: evdev/uinput,
//! Windows: WH_KEYBOARD_LL hook). `--demo` skips capture entirely and
//! dispatches two example actions once -- useful for a quick smoke test
//! with no real hardware/permissions needed.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use keylex::adapters::{SocketAdapter, WebSocketAdapter};
use keylex::config::Registry;
use keylex::dispatch::{Adapter, FallbackSender, Notifier, Router};
use keylex::spotlight;

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

/// Interactive `keylex --spotlight` fuzzy launcher: see
/// `keylex::spotlight::run_interactive` for the actual search/render/dispatch
/// loop -- this just does the same registry/token/adapter wiring `run_demo`
/// does, plus the `list_actions` handshake bootstrap
/// (docs/protocol.md#action-catalog-handshake-list_actions).
fn run_spotlight_interactive(registry: &Registry, config_dir: &Path, token: &str) -> ExitCode {
    let handshake_adapter = SocketAdapter::new(token.to_string());
    let mut index = spotlight::bootstrap(registry, config_dir, &handshake_adapter);

    let router = Router {
        registry,
        adapters: build_adapters(token, None),
        notifier: Box::new(LogNotifier),
        fallback_sender: Box::new(LogFallbackSender),
    };

    match spotlight::run_interactive(&mut index, &router) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("keylex: spotlight UI failed: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Non-interactive `keylex --spotlight-query <text>`: prints the ranked
/// match list for `<text>` as JSON to stdout and exits, no dispatch, no
/// frecency mutation. Meant for another front-end to embed (e.g. a GNOME
/// Shell search provider shelling out to reuse this same fuzzy engine, see
/// extensions/linux-extension/search-provider.js).
fn run_spotlight_query(registry: &Registry, config_dir: &Path, token: &str, query: &str) -> ExitCode {
    let handshake_adapter = SocketAdapter::new(token.to_string());
    let index = spotlight::bootstrap(registry, config_dir, &handshake_adapter);

    match serde_json::to_string(&index.search(query)) {
        Ok(json) => {
            println!("{json}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("keylex: failed to serialize spotlight results: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Non-interactive `keylex --spotlight-run <action-id>`: dispatches one
/// spotlight entry (`spotlight::dispatch_entry` -- a real Keylex action id
/// goes through the normal focus-aware `Router::dispatch`, a raw
/// target-native command id goes straight to its source target) and records
/// the frecency hit, then exits. The activation half of the same
/// external-front-end use case as `run_spotlight_query`. Falls back to a
/// plain `Router::dispatch` if `action_id` isn't in the freshly-bootstrapped
/// index at all (shouldn't happen for anything `--spotlight-query` handed
/// back, but keeps this usable for a bare Keylex action id too).
fn run_spotlight_action(registry: &Registry, config_dir: &Path, token: &str, action_id: &str) -> ExitCode {
    let handshake_adapter = SocketAdapter::new(token.to_string());
    let mut index = spotlight::bootstrap(registry, config_dir, &handshake_adapter);

    let router = Router {
        registry,
        adapters: build_adapters(token, None),
        notifier: Box::new(LogNotifier),
        fallback_sender: Box::new(LogFallbackSender),
    };

    let focused = keylex::focus::focused_process_name();
    let result = match index.entries().iter().find(|e| e.action_id == action_id) {
        Some(entry) => spotlight::dispatch_entry(entry, &focused, &router),
        None => router.dispatch(action_id, &focused),
    };
    index.record_use(action_id);
    println!("{action_id} -> {result}");
    ExitCode::SUCCESS
}

fn main() -> ExitCode {
    let mut demo = false;
    let mut spotlight = false;
    let mut spotlight_query: Option<String> = None;
    let mut spotlight_run: Option<String> = None;
    let mut config_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config");

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--demo" => demo = true,
            "--spotlight" => spotlight = true,
            "--spotlight-query" => {
                let Some(query) = args.next() else {
                    eprintln!("keylex: --spotlight-query requires a search string");
                    return ExitCode::FAILURE;
                };
                spotlight_query = Some(query);
            }
            "--spotlight-run" => {
                let Some(action_id) = args.next() else {
                    eprintln!("keylex: --spotlight-run requires an action id");
                    return ExitCode::FAILURE;
                };
                spotlight_run = Some(action_id);
            }
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

    if let Some(action_id) = spotlight_run {
        return run_spotlight_action(&registry, &config_dir, &token, &action_id);
    }
    if let Some(query) = spotlight_query {
        return run_spotlight_query(&registry, &config_dir, &token, &query);
    }
    if spotlight {
        return run_spotlight_interactive(&registry, &config_dir, &token);
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

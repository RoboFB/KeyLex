//! The `keylex` command line: what to run, and the wiring each mode needs.

use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::adapters::{SocketAdapter, WebSocketAdapter};
use crate::config::{AdapterKind, KeyCombo, Registry};
use crate::dispatch::{Adapters, FallbackSender, Notifier, Router};
use crate::{auth, capture, focus, spotlight};

const USAGE: &str = "\
keylex -- resolve keystrokes into actions and dispatch them natively

Usage: keylex [options]

With no options, runs the real capture loop (needs evdev/uinput permissions
on Linux). Options:

  --demo                    dispatch two example actions and exit; no
                            capture, hardware or permissions needed
  --spotlight               interactive fuzzy action search
  --spotlight-query <text>  print ranked matches as JSON and exit
  --spotlight-run <id>      dispatch one action id and exit
  --config-dir <path>       load config from somewhere else
  -h, --help                show this message
";

#[derive(Debug)]
enum Command {
    Capture,
    Demo,
    Spotlight,
    Query(String),
    Run(String),
}

#[derive(Debug)]
struct Options {
    config_dir: PathBuf,
    command: Command,
}

pub fn run() -> ExitCode {
    match execute(std::env::args().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("keylex: {e}");
            ExitCode::FAILURE
        }
    }
}

fn execute(args: impl Iterator<Item = String>) -> Result<(), Box<dyn Error>> {
    let Some(Options {
        config_dir,
        command,
    }) = parse(args)?
    else {
        print!("{USAGE}");
        return Ok(());
    };

    let registry = Registry::load(&config_dir)
        .map_err(|e| format!("failed to load config from {}: {e}", config_dir.display()))?;
    let token = auth::load_or_create_token(&config_dir).map_err(|e| {
        format!(
            "failed to read the auth token in {}: {e}",
            config_dir.display()
        )
    })?;

    match command {
        Command::Demo => demo(&router(&registry, &token)),
        Command::Spotlight => {
            let mut index = catalog(&registry, &config_dir, &token);
            spotlight::run_interactive(&mut index, &router(&registry, &token))?;
        }
        Command::Query(query) => {
            let index = catalog(&registry, &config_dir, &token);
            println!("{}", serde_json::to_string(&index.search(&query))?);
        }
        Command::Run(action_id) => {
            let mut index = catalog(&registry, &config_dir, &token);
            run_action(&mut index, &router(&registry, &token), &action_id);
        }
        Command::Capture => {
            let websocket = websocket_adapter(&registry, &token);
            capture::run(
                &registry,
                adapters(&token, websocket),
                Box::new(LogNotifier),
            )?;
        }
    }
    Ok(())
}

/// `Ok(None)` means usage was requested and nothing else should run.
fn parse(mut args: impl Iterator<Item = String>) -> Result<Option<Options>, String> {
    let mut options = Options {
        config_dir: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config"),
        command: Command::Capture,
    };

    while let Some(arg) = args.next() {
        let mut value = || args.next().ok_or_else(|| format!("{arg} requires a value"));
        match arg.as_str() {
            "-h" | "--help" => return Ok(None),
            "--demo" => options.command = Command::Demo,
            "--spotlight" => options.command = Command::Spotlight,
            "--spotlight-query" => options.command = Command::Query(value()?),
            "--spotlight-run" => options.command = Command::Run(value()?),
            "--config-dir" => options.config_dir = PathBuf::from(value()?),
            other => return Err(format!("unknown argument {other:?} (try --help)")),
        }
    }
    Ok(Some(options))
}

/// Two hardcoded dispatches, so the config/adapter wiring can be smoke
/// tested without a keyboard grab.
fn demo(router: &Router) {
    for (action_id, focused) in [("close.tab", "code"), ("go_to.definition", "chrome.exe")] {
        println!(
            "{action_id} -> {}",
            router.dispatch(action_id, Some(focused))
        );
    }
}

/// Dispatches one catalog entry by id and records the use, for a front end
/// that drives search from outside (see
/// `extensions/linux-extension/search-provider.js`). An id the catalog
/// doesn't know is still tried as a plain action.
fn run_action(index: &mut spotlight::Index, router: &Router, action_id: &str) {
    let focused = focus::focused_process_name();
    let outcome = match index
        .entries()
        .iter()
        .find(|entry| entry.action_id == action_id)
    {
        Some(entry) => entry.dispatch(focused.as_deref(), router),
        None => router.dispatch(action_id, focused.as_deref()),
    };
    index.record_use(action_id);
    println!("{action_id} -> {outcome}");
}

fn catalog<'a>(registry: &'a Registry, config_dir: &Path, token: &str) -> spotlight::Index<'a> {
    spotlight::bootstrap(registry, config_dir, &SocketAdapter::new(token.to_string()))
}

fn adapters(token: &str, websocket: Option<WebSocketAdapter>) -> Adapters {
    let mut adapters = Adapters::new();
    adapters.insert(
        AdapterKind::Socket,
        Box::new(SocketAdapter::new(token.to_string())),
    );
    if let Some(websocket) = websocket {
        adapters.insert(AdapterKind::WebSocket, Box::new(websocket));
    }
    adapters
}

/// The router the CLI modes share. Only the capture loop needs a
/// websocket adapter, and it builds its own `Adapters` for that.
fn router<'a>(registry: &'a Registry, token: &str) -> Router<'a> {
    Router::new(
        registry,
        adapters(token, None),
        Box::new(LogNotifier),
        Box::new(LogFallbackSender),
    )
}

/// Starts the WebSocket server the daemon needs for any target that
/// connects *in* rather than being connected to. Only the first such target
/// is served; a second one would need its own port and server.
fn websocket_adapter(registry: &Registry, token: &str) -> Option<WebSocketAdapter> {
    let target = registry
        .targets()
        .iter()
        .find(|target| target.adapter == AdapterKind::WebSocket)?;
    let Some(port) = target.port else {
        eprintln!(
            "keylex: target {:?} has no 'port' configured",
            target.program
        );
        return None;
    };

    WebSocketAdapter::spawn(port, token.to_string(), target.allowed_origin.clone())
        .map_err(|e| eprintln!("keylex: could not start the websocket adapter on port {port}: {e}"))
        .ok()
}

/// Real OS notifications aren't implemented yet on either platform; this
/// stands in for them, deliberately, until they are.
struct LogNotifier;

impl Notifier for LogNotifier {
    fn show(&self, message: &str) {
        println!("[notify] {message}");
    }
}

/// Outside the capture loop there is no virtual device to inject through,
/// so the CLI modes report the keycode they would have sent instead.
struct LogFallbackSender;

impl FallbackSender for LogFallbackSender {
    fn send(&self, combo: &KeyCombo) {
        println!("[fallback keycode] {combo}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_args(args: &[&str]) -> Result<Option<Options>, String> {
        parse(args.iter().map(|a| a.to_string()))
    }

    #[test]
    fn no_arguments_runs_the_capture_loop() {
        let options = parse_args(&[])
            .unwrap()
            .expect("should not be a help request");
        assert!(matches!(options.command, Command::Capture));
    }

    #[test]
    fn flags_taking_a_value_report_a_missing_one() {
        assert!(matches!(
            parse_args(&["--spotlight-run", "save"]),
            Ok(Some(_))
        ));
        assert!(parse_args(&["--spotlight-run"]).is_err());
        assert!(parse_args(&["--nope"]).is_err());
        assert!(parse_args(&["--help"]).unwrap().is_none());
    }
}

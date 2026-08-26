use std::fs;
use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

static WARNED_NO_XDOTOOL: AtomicBool = AtomicBool::new(false);

const COMMAND_TIMEOUT: Duration = Duration::from_millis(500);

pub fn focused_process_name() -> String {
    if which("xdotool").is_none() {
        if !WARNED_NO_XDOTOOL.swap(true, Ordering::Relaxed) {
            eprintln!(
                "keylex: xdotool not found -- can't resolve the focused process under X11, \
                 actions will run through the keycode fallback. (Wayland isn't supported by \
                 this prototype yet.)"
            );
        }
        return String::new();
    }

    let Some(pid) = run_with_timeout(
        Command::new("xdotool").args(["getactivewindow", "getwindowpid"]),
        COMMAND_TIMEOUT,
    ) else {
        return String::new();
    };
    let pid = pid.trim();
    if pid.is_empty() {
        return String::new();
    }

    fs::read_to_string(format!("/proc/{pid}/comm"))
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

fn which(program: &str) -> Option<()> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .any(|dir| dir.join(program).is_file())
        .then_some(())
}

/// `std::process::Command` has no built-in timeout; this polls
/// `try_wait()` and kills the child if it runs longer than `timeout`, to
/// keep the same "never block dispatch on a hung subprocess" guarantee
/// the previous `subprocess.run(..., timeout=0.5)` gave.
fn run_with_timeout(cmd: &mut Command, timeout: Duration) -> Option<String> {
    let mut child = cmd.stdout(Stdio::piped()).stderr(Stdio::null()).spawn().ok()?;
    let start = Instant::now();

    let status = loop {
        match child.try_wait().ok()? {
            Some(status) => break status,
            None => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return None;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    };

    if !status.success() {
        return None;
    }
    let mut out = String::new();
    child.stdout.take()?.read_to_string(&mut out).ok()?;
    Some(out)
}

//! X11 focused-window lookup, via `xdotool` and `/proc/<pid>/comm`.
//! Wayland is not supported: there is no equivalent portable query, so this
//! warns once and reports nothing, leaving dispatch on the keycode path.

use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// Bounds how long a dispatch can wait on the subprocess. Dispatch sits in
/// the keyboard path, so a hung `xdotool` must never stall it.
const COMMAND_TIMEOUT: Duration = Duration::from_millis(500);

static WARNED: AtomicBool = AtomicBool::new(false);

pub fn focused_process_name() -> Option<String> {
    if !is_on_path("xdotool") {
        if !WARNED.swap(true, Ordering::Relaxed) {
            eprintln!(
                "keylex: xdotool not found -- can't resolve the focused process under X11, \
                 so actions will run through the keycode fallback. (Wayland isn't supported \
                 by this prototype yet.)"
            );
        }
        return None;
    }

    let pid = run(Command::new("xdotool").args(["getactivewindow", "getwindowpid"]))?;
    let comm = std::fs::read_to_string(format!("/proc/{}/comm", pid.trim())).ok()?;
    Some(comm.trim().to_string()).filter(|name| !name.is_empty())
}

fn is_on_path(program: &str) -> bool {
    std::env::var_os("PATH")
        .is_some_and(|path| std::env::split_paths(&path).any(|dir| dir.join(program).is_file()))
}

/// `std::process::Command` has no timeout of its own, so this polls
/// `try_wait` and kills the child once `COMMAND_TIMEOUT` is up.
fn run(command: &mut Command) -> Option<String> {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let started = Instant::now();

    let status = loop {
        match child.try_wait().ok()? {
            Some(status) => break status,
            None if started.elapsed() > COMMAND_TIMEOUT => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
            None => std::thread::sleep(Duration::from_millis(10)),
        }
    };

    let output = child.wait_with_output().ok()?;
    status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}

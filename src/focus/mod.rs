//! Resolves the process name of the currently focused window, needed on
//! every dispatch to match against `targets.toml`.
//!
//! `None` is a normal answer, not an error: on an unsupported platform, a
//! Wayland session, or a window that reports nothing, `Router::dispatch`
//! simply falls through to the keycode path.

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::focused_process_name;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::focused_process_name;

/// Same "compiles everywhere, unsupported at runtime" shape as
/// `capture::run`, so callers outside `src/capture/` (the spotlight CLI
/// modes) can call this unconditionally.
#[cfg(not(any(target_os = "linux", windows)))]
pub fn focused_process_name() -> Option<String> {
    None
}

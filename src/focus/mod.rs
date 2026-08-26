//! Resolves the process name of the currently focused window. Needed on
//! every dispatch to match against targets.toml. An empty/unknown result
//! is not an error -- Router::dispatch falls back to the keycode path.

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::focused_process_name;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::focused_process_name;

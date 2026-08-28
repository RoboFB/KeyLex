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

/// Same "compiles everywhere, backend-unsupported at runtime" pattern as
/// `capture::run`'s fallback -- lets callers outside `src/capture/` (e.g.
/// the `--spotlight` CLI mode in `main.rs`) call this unconditionally
/// without breaking the build on a platform with no focus backend yet
/// (macOS).
#[cfg(not(any(target_os = "linux", windows)))]
pub fn focused_process_name() -> String {
    String::new()
}

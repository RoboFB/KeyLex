//! A single global hotkey listener for `keylex --spotlight-daemon`
//! (`src/spotlight/gui.rs`) -- distinct from `capture`, which grabs the
//! whole keyboard and resolves every bound key into an action. This only
//! ever watches for one configured combo and calls back when it fires;
//! every other key is left completely alone, so it needs none of
//! `capture`'s consume/re-emit machinery.

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::listen;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::listen;

/// macOS and everything else: compiles, but there is no backend to run,
/// same stance as `capture::run` for an unsupported platform.
#[cfg(not(any(target_os = "linux", windows)))]
pub fn listen(_combo: &crate::config::KeyCombo, _on_trigger: impl FnMut()) -> std::io::Result<()> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "keylex has no global hotkey backend for this platform",
    ))
}

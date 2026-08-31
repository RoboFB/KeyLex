//! Lowest-level keyboard interception: grab the physical keyboard as deeply
//! as the OS allows, match each key against the actions registry, and
//! either consume and dispatch it or re-emit it unchanged so ordinary
//! typing stays untouched.

#[cfg(any(target_os = "linux", windows))]
mod chord;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::run;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::run;
#[cfg(windows)]
pub(crate) use windows::vk_for_token;

/// macOS and everything else: compiles, but there is no backend to run.
/// The planned approach is a `CGEventTap` behind an Accessibility grant.
#[cfg(not(any(target_os = "linux", windows)))]
pub fn run(
    _registry: &crate::config::Registry,
    _adapters: crate::dispatch::Adapters,
    _notifier: Box<dyn crate::dispatch::Notifier>,
) -> std::io::Result<()> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "keylex has no keyboard capture backend for this platform",
    ))
}

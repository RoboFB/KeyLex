//! Lowest-level keyboard interception: grabs the physical keyboard as
//! deeply as the OS allows, matches each key combo against the actions
//! registry, and either consumes+dispatches a match or re-emits the
//! event unchanged so normal typing stays untouched.

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::run;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::run;

#[cfg(not(any(target_os = "linux", windows)))]
pub fn run(
    _registry: &crate::config::Registry,
    _adapters: std::collections::HashMap<String, Box<dyn crate::dispatch::Adapter>>,
    _notifier: Box<dyn crate::dispatch::Notifier>,
) -> std::io::Result<()> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "keylex has no keyboard capture backend for this platform",
    ))
}

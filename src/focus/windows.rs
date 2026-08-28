//! Windows focused-window lookup. Like `src/capture/windows.rs`, this is a
//! port that has never run on an actual Windows machine.

use windows::core::PWSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

pub fn focused_process_name() -> Option<String> {
    // SAFETY: takes no arguments and returns a window handle or null.
    let window = unsafe { GetForegroundWindow() };
    if window.0.is_null() {
        return None;
    }

    let mut pid = 0u32;
    // SAFETY: `window` is the handle just returned, and the call only
    // writes the process id into the live local it is given.
    unsafe { GetWindowThreadProcessId(window, Some(&mut pid)) };
    if pid == 0 {
        return None;
    }

    // SAFETY: opening a process by id is safe to attempt for any id; a
    // dead or forbidden one fails rather than misbehaving.
    let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }.ok()?;
    let name = image_name(process);
    // SAFETY: `process` came from the `OpenProcess` above, is still open,
    // and is closed exactly once here.
    unsafe {
        let _ = CloseHandle(process);
    }
    name
}

fn image_name(process: HANDLE) -> Option<String> {
    let mut buffer = [0u16; 260];
    let mut length = buffer.len() as u32;
    // SAFETY: `process` is a live handle with QUERY_LIMITED_INFORMATION
    // rights, and `length` is genuinely the capacity of the buffer being
    // written into -- the one invariant this call relies on.
    unsafe {
        QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &mut length,
        )
    }
    .ok()?;

    let path = String::from_utf16_lossy(&buffer[..length as usize]);
    path.rsplit(['\\', '/'])
        .next()
        .map(str::to_string)
        .filter(|name| !name.is_empty())
}

//! Windows focused-process lookup, ported from the previous ctypes
//! implementation. Untestable on this (Linux) dev machine -- same caveat
//! the project already carried for the Python listener.

use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};
use windows::core::PWSTR;

pub fn focused_process_name() -> String {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return String::new();
        }

        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return String::new();
        }

        let Ok(handle) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else {
            return String::new();
        };
        let name = query_image_name(handle).unwrap_or_default();
        let _ = CloseHandle(handle);
        name
    }
}

unsafe fn query_image_name(handle: HANDLE) -> Option<String> {
    let mut buf = [0u16; 260];
    let mut size: u32 = buf.len() as u32;
    QueryFullProcessImageNameW(handle, windows::Win32::System::Threading::PROCESS_NAME_WIN32, PWSTR(buf.as_mut_ptr()), &mut size).ok()?;

    let path = String::from_utf16_lossy(&buf[..size as usize]);
    path.rsplit(['\\', '/']).next().map(str::to_string)
}

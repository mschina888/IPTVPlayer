use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::ffi::c_void;
use windows_sys::Win32::Foundation::{HWND, LPARAM, POINT, RECT};
use windows_sys::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, MonitorFromWindow, HDC, HMONITOR, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

const SS_BLACKRECT: u32 = 0x4;

pub fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Win32 HWND of a Slint window (winit backend). Available after the window is shown.
pub fn hwnd_of(window: &slint::Window) -> Option<isize> {
    let wh = window.window_handle();
    match wh.window_handle().ok()?.as_raw() {
        RawWindowHandle::Win32(h) => Some(h.hwnd.get()),
        _ => None,
    }
}

/// Black child window that hosts the mpv video (libmpv `wid`).
pub fn create_video_child(parent: isize, x: i32, y: i32, w: i32, h: i32) -> isize {
    unsafe {
        let class = wide("STATIC");
        let hinst = GetModuleHandleW(std::ptr::null());
        let hwnd = CreateWindowExW(
            0,
            class.as_ptr(),
            std::ptr::null(),
            WS_CHILD | WS_VISIBLE | WS_CLIPSIBLINGS | WS_CLIPCHILDREN | SS_BLACKRECT,
            x,
            y,
            w.max(1),
            h.max(1),
            parent as HWND,
            std::ptr::null_mut(),
            hinst,
            std::ptr::null::<c_void>(),
        );
        hwnd as isize
    }
}

pub fn move_child(child: isize, x: i32, y: i32, w: i32, h: i32) {
    unsafe {
        SetWindowPos(child as HWND, std::ptr::null_mut(), x, y, w.max(1), h.max(1), SWP_NOACTIVATE | SWP_NOZORDER);
    }
}

pub struct MenuItemSpec {
    pub id: usize,
    pub label: String,
    pub checked: bool,
    pub separator: bool,
}

/// Native Windows popup menu at the cursor. Returns the chosen item id (0 = none).
pub fn popup_menu(owner: isize, items: &[MenuItemSpec]) -> usize {
    unsafe {
        let menu = CreatePopupMenu();
        for it in items {
            if it.separator {
                AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());
            } else {
                let w = wide(&it.label);
                let flags = MF_STRING | if it.checked { MF_CHECKED } else { 0 };
                AppendMenuW(menu, flags, it.id, w.as_ptr());
            }
        }
        let mut pt = POINT { x: 0, y: 0 };
        GetCursorPos(&mut pt);
        SetForegroundWindow(owner as HWND);
        let cmd = TrackPopupMenu(menu, TPM_RETURNCMD | TPM_RIGHTBUTTON, pt.x, pt.y, 0, owner as HWND, std::ptr::null());
        PostMessageW(owner as HWND, WM_NULL, 0, 0);
        DestroyMenu(menu);
        cmd as usize
    }
}

unsafe extern "system" fn mon_proc(_m: HMONITOR, _dc: HDC, rc: *mut RECT, data: LPARAM) -> i32 {
    let v = &mut *(data as *mut Vec<RECT>);
    v.push(*rc);
    1
}

/// Work-area origin of the next monitor after the one containing `hwnd`.
pub fn next_monitor_origin(hwnd: isize) -> Option<(i32, i32)> {
    unsafe {
        let mut rects: Vec<RECT> = Vec::new();
        EnumDisplayMonitors(std::ptr::null_mut(), std::ptr::null(), Some(mon_proc), &mut rects as *mut Vec<RECT> as LPARAM);
        if rects.len() < 2 {
            return None;
        }
        let cur = MonitorFromWindow(hwnd as HWND, MONITOR_DEFAULTTONEAREST);
        let mut mi: MONITORINFO = std::mem::zeroed();
        mi.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
        GetMonitorInfoW(cur, &mut mi);
        let pos = rects
            .iter()
            .position(|r| r.left == mi.rcMonitor.left && r.top == mi.rcMonitor.top)
            .unwrap_or(0);
        let n = rects[(pos + 1) % rects.len()];
        Some((n.left + 60, n.top + 60))
    }
}

pub fn set_autostart(enable: bool) {
    use std::os::windows::process::CommandExt;
    let key = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
    let mut cmd = std::process::Command::new("reg");
    if enable {
        if let Ok(exe) = std::env::current_exe() {
            cmd.args(["add", key, "/v", "IPTVPlayer", "/t", "REG_SZ", "/d", &format!("\"{}\"", exe.display()), "/f"]);
        } else {
            return;
        }
    } else {
        cmd.args(["delete", key, "/v", "IPTVPlayer", "/f"]);
    }
    let _ = cmd.creation_flags(0x0800_0000).output();
}

pub fn show_window(hwnd: isize, show: bool) {
    unsafe {
        if show {
            ShowWindow(hwnd as HWND, SW_SHOW);
            ShowWindow(hwnd as HWND, SW_RESTORE);
            SetForegroundWindow(hwnd as HWND);
        } else {
            ShowWindow(hwnd as HWND, SW_HIDE);
        }
    }
}

#[allow(dead_code)]
pub fn open_url(url: &str) {
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    unsafe {
        let op = wide("open");
        let u = wide(url);
        ShellExecuteW(std::ptr::null_mut(), op.as_ptr(), u.as_ptr(), std::ptr::null(), std::ptr::null(), SW_SHOWNORMAL);
    }
}

/// Native "Open file" dialog (Explorer style). `filters`: pairs (name, patterns like "*.m3u;*.m3u8").
pub fn pick_file(title: &str, filters: &[(&str, &str)]) -> Option<String> {
    use windows_sys::Win32::UI::Controls::Dialogs::{GetOpenFileNameW, OFN_EXPLORER, OFN_FILEMUSTEXIST, OFN_HIDEREADONLY, OFN_PATHMUSTEXIST, OPENFILENAMEW};
    let w = |s: &str| -> Vec<u16> { s.encode_utf16().collect() };
    let mut flt: Vec<u16> = Vec::new();
    for (n, pat) in filters {
        flt.extend(w(n));
        flt.push(0);
        flt.extend(w(pat));
        flt.push(0);
    }
    flt.push(0);
    let mut ttl = w(title);
    ttl.push(0);
    let mut buf = vec![0u16; 2048];
    unsafe {
        let mut ofn: OPENFILENAMEW = std::mem::zeroed();
        ofn.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
        ofn.lpstrFilter = flt.as_ptr();
        ofn.lpstrFile = buf.as_mut_ptr();
        ofn.nMaxFile = buf.len() as u32;
        ofn.lpstrTitle = ttl.as_ptr();
        ofn.Flags = OFN_EXPLORER | OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST | OFN_HIDEREADONLY;
        if GetOpenFileNameW(&mut ofn) != 0 {
            let n = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
            Some(String::from_utf16_lossy(&buf[..n]))
        } else {
            None
        }
    }
}

/// Returns a handle guard if this is the first instance; None if another copy is already running.
pub fn single_instance(name: &str) -> Option<isize> {
    use windows_sys::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
    use windows_sys::Win32::System::Threading::CreateMutexW;
    let mut w: Vec<u16> = name.encode_utf16().collect();
    w.push(0);
    unsafe {
        let h = CreateMutexW(std::ptr::null(), 0, w.as_ptr());
        if h.is_null() || GetLastError() == ERROR_ALREADY_EXISTS {
            None
        } else {
            Some(h as isize)
        }
    }
}

// ---- mouse over the video: low-level hook (mpv's own window does not deliver mouse events when embedded)
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::OnceLock;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetKeyState, VK_CONTROL};

pub static VIDEO_HWND: AtomicIsize = AtomicIsize::new(0);
/// kind: 0 left down, 1 right down, 2 middle down, 3 wheel; delta (wheel), ctrl pressed
static MOUSE_CB: OnceLock<fn(u32, i32, bool)> = OnceLock::new();

unsafe extern "system" fn ll_mouse_proc(code: i32, w: usize, l: isize) -> isize {
    if code >= 0 {
        let msg = w as u32;
        let kind = match msg {
            WM_LBUTTONDOWN => Some(0),
            WM_RBUTTONDOWN => Some(1),
            WM_MBUTTONDOWN => Some(2),
            WM_MOUSEWHEEL => Some(3),
            _ => None,
        };
        if let Some(kind) = kind {
            let vid = VIDEO_HWND.load(Ordering::Relaxed);
            if vid != 0 {
                let ms = &*(l as *const MSLLHOOKSTRUCT);
                // WindowFromPoint skips the embedded mpv window, so test: top-level window under the cursor is ours AND the point is inside the video rectangle
                let hit = WindowFromPoint(ms.pt);
                let mut rc: RECT = std::mem::zeroed();
                GetWindowRect(vid as HWND, &mut rc);
                let inside = ms.pt.x >= rc.left && ms.pt.x < rc.right && ms.pt.y >= rc.top && ms.pt.y < rc.bottom;
                let same_root = !hit.is_null() && GetAncestor(hit, GA_ROOT) == GetAncestor(vid as HWND, GA_ROOT);
                if inside && same_root {                    let delta = ((ms.mouseData >> 16) as i16) as i32;
                    let ctrl = GetKeyState(VK_CONTROL as i32) < 0;
                    if let Some(cb) = MOUSE_CB.get() {
                        cb(kind, delta, ctrl);
                    }
                }
            }
        }
    }
    CallNextHookEx(std::ptr::null_mut(), code, w, l)
}

pub fn install_mouse_hook(cb: fn(u32, i32, bool)) {
    let _ = MOUSE_CB.set(cb);
    unsafe {
        let h = SetWindowsHookExW(WH_MOUSE_LL, Some(ll_mouse_proc), GetModuleHandleW(std::ptr::null()), 0);
        crate::dlog(&format!("mouse hook installed: {}", !h.is_null()));
    }
}



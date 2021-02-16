#[cfg(windows)]
extern crate winapi;
use std::io::Error;
use std::mem;
use std::ptr;
use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::shared::winerror::*;
use winapi::um::dwmapi::*;
use winapi::um::winnt::*;
use winapi::um::winuser::*;

macro_rules! msgbox {
    ($title:tt, $($arg:tt)*) => ({
        let res = format!($($arg)*);
        _msgbox($title, &res).unwrap()
    })
}

fn _msgbox(title: &str, msg: &str) -> Result<i32, Error> {
    use std::ffi::OsStr;
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    let title: Vec<u16> = OsStr::new(title).encode_wide().chain(once(0)).collect();
    let msg: Vec<u16> = OsStr::new(msg).encode_wide().chain(once(0)).collect();
    let ret = unsafe { MessageBoxW(ptr::null_mut(), msg.as_ptr(), title.as_ptr(), MB_OK) };

    if ret == 0 {
        Err(Error::last_os_error())
    } else {
        Ok(ret)
    }
}

struct State {
    hwnd: HWND,
    rect: RECT,
}

extern "system" fn enum_windows_callback(hwnd: HWND, state: LPARAM) -> BOOL
{
    println!("Inside addr: {:?}", state);
    let state: &State = unsafe { mem::transmute(state) };
    println!("{:?}", hwnd); //hwnd of enumerated window.
    let rect = state.rect;
    let moved_hwnd = state.hwnd;
    println!("(x,y)\nhandle: {:?}\nStart: ({}, {})\nStop: ({}, {})", moved_hwnd, rect.left, rect.top, rect.right, rect.bottom);
    1 // Return true to continue enumerating
}

extern "system" fn set_win_event_hook_callback(_: HWINEVENTHOOK, event: DWORD, hwnd: HWND, _: LONG, id_child: LONG, _: DWORD, _: DWORD) {
    if event == EVENT_SYSTEM_MOVESIZEEND && id_child == 0 {
        unsafe {
            let rect: RECT = mem::zeroed();
            let ptr: LPVOID = mem::transmute(&rect);
            if DwmGetWindowAttribute(hwnd, DWMWA_EXTENDED_FRAME_BOUNDS, ptr, mem::size_of_val(&rect) as u32) != S_OK {
                msgbox!("Err", "Failed to get rect!");
            } else {
                let state = State{hwnd, rect};
                let state: LPARAM = mem::transmute(&state);
                let callback:WNDENUMPROC = Some(enum_windows_callback);
                println!("Outside addr: {:?}", state);
                EnumWindows(callback, state);
            }
        };
    }
}

fn main() {
    let callback: WINEVENTPROC = Some(set_win_event_hook_callback);
    let hook = unsafe {
        SetWinEventHook(
            EVENT_SYSTEM_MOVESIZEEND,
            EVENT_SYSTEM_MOVESIZEEND,
            ptr::null_mut(),
            callback,
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        )
    };
    if hook.is_null() {
        msgbox!("Err", "Failed to set move/resize event hook!");
        return;
    }

    loop {
        unsafe {
            let mut message: MSG = mem::zeroed();
            if GetMessageW(&mut message as *mut MSG, ptr::null_mut(), 0, 0) > 0 {
                TranslateMessage(&message as *const MSG);
                DispatchMessageW(&message as *const MSG);
            } else {
                break;
            }
        }
    }
}

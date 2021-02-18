#![macro_use]
use std::io::Error;
use std::mem;
use std::ptr;
use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::shared::winerror::*;
use winapi::um::dwmapi::*;
use winapi::um::winnt::*;
use winapi::um::winuser::*;

pub fn get_message() -> Option<MSG> {
    let mut msg: MSG = unsafe { mem::MaybeUninit::zeroed().assume_init() };
    let ret = unsafe { GetMessageW(&mut msg as *mut MSG, ptr::null_mut(), 0, 0) };
    match ret {
        | 0 => {
            println!("Error in get_message() return.");
            None
        }
        | -1 => None,
        | _ => Some(msg),
    }
}

pub fn translate_message(msg: *const MSG) {
    unsafe { TranslateMessage(msg) };
}

pub fn dispatch_message(msg: *const MSG) {
    unsafe { DispatchMessageW(msg) };
}

macro_rules! msgbox {
    ($title:tt, $($arg:tt)*) => ({
        let res = format!($($arg)*);
        _msgbox($title, &res).unwrap()
    })
}

pub fn _msgbox(title: &str, msg: &str) -> Result<i32, Error> {
    use std::ffi::OsStr;
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    let title: Vec<u16> = OsStr::new(title).encode_wide().chain(once(0)).collect();
    let msg: Vec<u16> = OsStr::new(msg).encode_wide().chain(once(0)).collect();
    let ret = unsafe { MessageBoxW(ptr::null_mut(), msg.as_ptr(), title.as_ptr(), MB_OK) };

    match ret {
        | 0 => Err(Error::last_os_error()),
        | _ => Ok(ret),
    }
}

pub fn is_window_visible(hwnd: HWND) -> bool {
    return unsafe { IsWindowVisible(hwnd) } != 0;
}

pub fn get_window_bounds(hwnd: HWND) -> Option<RECT> {
    let (ret, rect) = unsafe {
        let mut _rect = mem::MaybeUninit::<RECT>::zeroed().assume_init();
        let ptr = &_rect as *const RECT as LPVOID;
        let ret = DwmGetWindowAttribute(
            hwnd,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            ptr,
            mem::size_of_val(&_rect) as u32,
        );
        (ret, _rect)
    };
    match ret {
        | S_OK => Some(rect),
        | _ => {
            println!("Failed to get rect! for handle {:?}", hwnd);
            None
        }
    }
}

pub fn enum_windows<F>(mut func: F)
where
    F: FnMut(HWND) -> i32,
{
    // Implement trait to pass closure as lparam.
    // See: https://stackoverflow.com/questions/38995701/how-do-i-pass-a-closure-through-raw-pointers-as-an-argument-to-a-c-function
    let mut trait_obj: &mut dyn FnMut(HWND) -> i32 = &mut func;
    let lparam = &mut trait_obj as *mut _ as LPARAM;

    // Wrapper function to call closure.
    extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let trait_obj_ref: &mut &mut dyn FnMut(HWND) -> i32 =
            unsafe { &mut *(lparam as *mut LPARAM as *mut _) };
        return trait_obj_ref(hwnd);
    }

    // Run Callback API.
    let callback: WNDENUMPROC = Some(enum_windows_callback);
    unsafe {
        EnumWindows(callback, lparam);
    }
}

type EventHookFunc = extern "system" fn(
    _: HWINEVENTHOOK,
    event: DWORD,
    hwnd: HWND,
    _: LONG,
    id_child: LONG,
    _: DWORD,
    _: DWORD,
);

pub fn set_win_event_hook(cb: EventHookFunc) -> bool {
    let cb: WINEVENTPROC = Some(cb);
    let hook = unsafe {
        SetWinEventHook(
            EVENT_SYSTEM_MOVESIZEEND,
            EVENT_SYSTEM_MOVESIZEEND,
            ptr::null_mut(),
            cb,
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        )
    };
    !hook.is_null()
}

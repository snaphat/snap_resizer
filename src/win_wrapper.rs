#![macro_use]
extern crate winapi;
use std::io::Error;
use std::mem;
use std::ptr;
use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::shared::*;
use winapi::um::winnt::*;
use winapi::um::winuser;
use winapi::um::winuser::*;

pub type HWND = windef::HWND;
pub type RECT = windef::RECT;
//pub const EVENT_SYSTEM_MOVESIZESTART: UINT = winuser:: EVENT_SYSTEM_MOVESIZESTART;
pub const EVENT_SYSTEM_MOVESIZEEND: UINT = winuser::EVENT_SYSTEM_MOVESIZEEND;

// Safe API to retrieve Windows Messages.
pub fn get_message() -> Option<MSG> {
    // Initialize memory.
    let mut msg: MSG = unsafe { mem::MaybeUninit::zeroed().assume_init() };

    // Get Message.
    let ret = unsafe { GetMessageW(&mut msg as *mut MSG, ptr::null_mut(), 0, 0) };

    // Check for errors before returning message.
    match ret {
        | 0 => {
            println!("Error in get_message() return.");
            None
        }
        | -1 => None,
        | _ => Some(msg),
    }
}

// Safe API to translate Windows Messages.
pub fn translate_message(msg: *const MSG) {
    unsafe { TranslateMessage(msg) };
}

// Safe API to dispatch Windows Messages.
pub fn dispatch_message(msg: *const MSG) {
    unsafe { DispatchMessageW(msg) };
}

// println like macro for message boxes.
macro_rules! msgbox {
    ($title:tt, $($arg:tt)*) => ({
        let res = format!($($arg)*);
        _msgbox($title, &res).unwrap()
    })
}

// API for msgbox macro.
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

// Safe API to handle is window visible.
pub fn is_window_visible(hwnd: HWND) -> bool {
    return unsafe { IsWindowVisible(hwnd) } != 0;
}

// Safe API to get window bounds.
pub fn get_window_rect(hwnd: HWND) -> Result<RECT, Error> {
    // Run windows API and return the return value and rect.
    /*
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
    */

    // Run windows API and return the return value and rect.
    let (ret, rect) = unsafe {
        let mut _rect = mem::MaybeUninit::<RECT>::zeroed().assume_init();
        let ptr = &_rect as *const RECT as LPRECT;
        let ret = GetWindowRect(hwnd, ptr);
        (ret, _rect)
    };

    // Make sure return value was valid before returning rect.
    match ret {
        | 0 => Err(Error::last_os_error()),
        | _ => Ok(rect),
    }
}

pub fn set_window_pos(hwnd: HWND, x: i32, y: i32, cx: i32, cy: i32) -> Result<i32, Error> {
    let ret = unsafe {
        SetWindowPos(
            hwnd,
            ptr::null_mut(),
            x,
            y,
            cx,
            cy,
            SWP_NOACTIVATE | SWP_NOZORDER,
        )
    };
    match ret {
        | 0 => Err(Error::last_os_error()),
        | _ => Ok(ret),
    }
}

// Safe API to enumerate windows.
// Takes a closure.
pub fn enum_windows<F>(mut func: F)
where
    F: FnMut(HWND) -> i32,
{
    // Implement trait to pass closure as lparam (unsafely).
    // See: https://stackoverflow.com/questions/38995701/how-do-i-pass-a-closure-through-raw-pointers-as-an-argument-to-a-c-function
    let mut trait_obj: &mut dyn FnMut(HWND) -> i32 = &mut func;
    let lparam = &mut trait_obj as *mut _ as LPARAM;

    // C-compatible EnumWindows callback to call closure.
    extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let func: &mut &mut dyn FnMut(HWND) -> i32 =
            unsafe { &mut *(lparam as *mut LPARAM as *mut _) };

        return func(hwnd); // Call closure
    }

    // Run Callback API.
    let callback: WNDENUMPROC = Some(enum_windows_callback);
    unsafe {
        EnumWindows(callback, lparam);
    }
}

// Static variable to hold our closure event hook handler.
static mut HANDLER: Option<Box<dyn FnMut(DWORD, HWND, LONG)>> = None;

// Safe API to setup callbacks to listen for events.
// Takes a closure.
pub fn set_win_event_hook<F>(func: F, event_min: UINT, event_max: UINT) -> bool
where
    F: FnMut(DWORD, HWND, LONG) + 'static,
{
    // C-compatible SetWinEventHook callback to call closure.
    extern "system" fn set_win_event_hook_callback(
        _: HWINEVENTHOOK,
        event: DWORD,
        hwnd: HWND,
        _: LONG,
        id_child: LONG,
        _: DWORD,
        _: DWORD,
    ) {
        // Unwrap actual closure handler and pass in arguments.
        unsafe {
            if let Some(func) = &mut HANDLER {
                // Call closure.
                func(event as u32, hwnd, id_child as i32);
            }
        };
    }

    let hook = unsafe {
        // This callback hook passes no user arguments so we cannot pass the closure to it.
        // Therefore we setup the closure as a handler via global memory (unsafely).
        HANDLER = Some(Box::new(func));
        SetWinEventHook(
            event_min,
            event_max,
            ptr::null_mut(),
            Some(set_win_event_hook_callback), // Setup static C-compatible ABI handler.
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        )
    };
    !hook.is_null()
}

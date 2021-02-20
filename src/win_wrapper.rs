#![macro_use]
extern crate winapi;
use std::io::Error;
use std::mem;
use std::mem::size_of;
use std::ptr;
use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::shared::*;
use winapi::um::dwmapi::*;
use winapi::um::winnt::*;
use winapi::um::winuser;
use winapi::um::winuser::*;

pub type HWND = windef::HWND;
pub type RECT = windef::RECT;
//pub const EVENT_SYSTEM_MOVESIZESTART: UINT = winuser:: EVENT_SYSTEM_MOVESIZESTART;
pub const EVENT_SYSTEM_MOVESIZEEND: UINT = winuser::EVENT_SYSTEM_MOVESIZEEND;
pub const DPI_AWARENESS_CONTEXT_SYSTEM_AWARE: DPI_AWARENESS_CONTEXT =
    windef::DPI_AWARENESS_CONTEXT_SYSTEM_AWARE;

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

// Safe API to check if a window shows on the taskbar.
pub fn is_taskbar_window(hwnd: HWND) -> bool {
    // Invisible windows do not show on the taskbar..
    if !is_window_visible(hwnd) {
        return false;
    }

    // Cloaked windows do not show on the taskbar.
    if let Ok(ret) = is_window_cloaked(hwnd) {
        if ret == true {
            return false;
        }
    } else {
        return false; // Drop-out if API call failed.
    }

    // Check Window Extended Style
    if let Ok(ret) = get_window_long(hwnd, GWL_EXSTYLE) {
        // App Windows always show on the taskbar when visible.
        if ret & WS_EX_APPWINDOW != 0 {
            return true;
        }
        // Tool windows and no-activate windows do not show on the taskbar.
        // Note: no-active windows can show in the taskbar if WS_EX_APPWINDOW is set.
        if ret & (WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE) != 0 {
            return false;
        }
    } else {
        return false; // Drop-out if API call failed.
    }

    // Check Window Style
    if let Ok(ret) = get_window_long(hwnd, GWL_STYLE) {
        // Child windows do not show on the taskbar.
        if ret & WS_CHILD != 0 {
            return false;
        }
    } else {
        return false; // Drop-out if API call failed.
    }

    // Windows with owner's do not appear on the taskbar.
    if get_window(hwnd, GW_OWNER) as u32 != 0 {
        return false;
    }

    // Windows with invisible titlebars do not appear on the taskbar.
    if let Ok(ti) = get_titlebar_info(hwnd) {
        if ti.rgstate[0] & STATE_SYSTEM_INVISIBLE != 0 {
            return false;
        }
    } else {
        return false; // Drop-out if API call failed.
    }

    // See: https://web.archive.org/web/20190217140538/https://blogs.msdn.microsoft.com/oldnewthing/20071008-00/?p=24863
    // Quote from Raymond Cen (MSFT):
    // For each visible window, walk up its owner chain until you find the root owner.
    // Then walk back down the visible last active popup chain until you find a visible window.
    // If you're back to where you're started, then put the window in the Alt+Tab list:
    {
        let hwnd_walk;
        let mut hwnd_try = get_window_ancestor(hwnd, GA_ROOTOWNER);
        loop {
            hwnd_walk = hwnd_try;
            hwnd_try = get_last_active_popup(hwnd_walk);
            if is_window_visible(hwnd_try) || hwnd_walk == hwnd_try {
                break;
            }
            break;
        }
        if hwnd_walk != hwnd {
            return false; // Window isn't a root window.
        }
    }

    return true; // Appears to be an taskbar window.
}

// Safe API to retrieve whether a window is iconic (minimized).
// Doesn't detect invalid handles.
pub fn is_window_iconic(hwnd: HWND) -> bool {
    // Returns 0 if the HWND isn't valid.
    return unsafe { IsIconic(hwnd) } != 0;
}

// Safe API to retrieve whether a window is minimized.
pub fn is_window_minimized(hwnd: HWND) -> Result<bool, Error> {
    // Call API.
    let ret = get_window_placement(hwnd);

    // Check for errors.
    match ret {
        | Ok(ret) => Ok(ret.showCmd == SW_SHOWMINIMIZED as u32), // Return wrapped bool.
        | Err(ret) => Err(ret),                                  // Most likely an invalid handle.
    }
}

// Safe API to retrieve whether a window is maximized.
pub fn is_window_maximized(hwnd: HWND) -> Result<bool, Error> {
    // Call API.
    let ret = get_window_placement(hwnd);

    // Check for errors.
    match ret {
        | Ok(ret) => Ok(ret.showCmd == SW_SHOWMAXIMIZED as u32), // Return wrapped bool.
        | Err(ret) => Err(ret),                                  // Most likely an invalid handle.
    }
}

// Safe API to retrieve window visibility.
// Doesn't detect invalid handles.
pub fn is_window_visible(hwnd: HWND) -> bool {
    // Returns 0 if the HWND isn't valid.
    return unsafe { IsWindowVisible(hwnd) } != 0;
}

// Safe API to retrieve window cloaked state.
pub fn is_window_cloaked(hwnd: HWND) -> Result<bool, String> {
    // Fill type.
    let mut is_cloaked = 0;

    // Call API.
    let ret = unsafe {
        DwmGetWindowAttribute(
            hwnd,
            DWMWA_CLOAKED,
            &mut is_cloaked as *const _ as LPVOID, // coerce pointer.
            size_of::<i32>() as u32,
        )
    };

    // Check for errors. HRESULT of non-zero is an error. Good luck decoding this.
    match ret {
        | winerror::S_OK => Ok(is_cloaked != 0), // Return wrapped bool.
        | _ => Err(format!("Returned HRESULT: 0x{:x}", ret)), // Most likely an invalid handle.
    }
}

// Safe API to retrieve a handle to a window that has the specified relationship to the specified window.
// Doesn't detect invalid handles.
pub fn get_window(hwnd: HWND, cmd: UINT) -> HWND {
    // Returns 0 if the HWND isn't valid or the relationship doesn't exist.
    unsafe { GetWindow(hwnd, cmd) }
}

//Safe API to retrieve information for the specified window.
pub fn get_window_long(hwnd: HWND, n_index: i32) -> Result<DWORD, Error> {
    // Call API.
    let ret = unsafe { GetWindowLongW(hwnd, n_index) };

    // Check for Errors.
    match ret {
        | 0 => Err(Error::last_os_error()), // Most likely an invalid handle.
        | _ => Ok(ret as u32),              // Return wrapped LONG as a DWORD (bitfield).
    }
}

// Safe API to retrieve the ancestor of the specified window.
// Doesn't detect invalid handles.
pub fn get_window_ancestor(hwnd: HWND, flags: UINT) -> HWND {
    // Returns 0 if the HWND isn't valid.
    unsafe { GetAncestor(hwnd, flags) }
}

pub fn get_window_placement(hwnd: HWND) -> Result<WINDOWPLACEMENT, Error> {
    // Fill structure.
    let mut wp = WINDOWPLACEMENT {
        flags: 0,
        length: size_of::<WINDOWPLACEMENT>() as u32,
        ptMaxPosition: POINT { x: 0, y: 0 },
        ptMinPosition: POINT { x: 0, y: 0 },
        rcNormalPosition: RECT { left: 0, top: 0, right: 0, bottom: 0 },
        showCmd: 0,
    };

    // Call API.
    let ret = unsafe { GetWindowPlacement(hwnd, &mut wp as *mut _ as *mut WINDOWPLACEMENT) };

    // Check for Errors.
    match ret {
        | 0 => Err(Error::last_os_error()), // Most likely an invalid handle.
        | _ => Ok(wp),                      // Return wrapped WINDOWPLACEMENT information.
    }
}

// Safe API to retrieve window attributes.
// Unsafe for unexpected types.
pub fn get_window_attribute<T>(hwnd: HWND, dw_attribute: DWORD) -> Result<T, Error> {
    // Initialize unknown type to zero.
    let mut pv_attribute = unsafe { mem::MaybeUninit::<T>::zeroed().assume_init() };

    // Call API.
    let (ret, pv_attribute) = unsafe {
        // Call API.
        let ret = DwmGetWindowAttribute(
            hwnd,
            dw_attribute,
            &mut pv_attribute as *mut _ as LPVOID,
            size_of::<T>() as u32,
        );
        (ret, pv_attribute)
    };

    // Check for Errors.
    match ret {
        | winerror::S_OK => Ok(pv_attribute), // Wrapped attribute.
        | _ => Err(Error::last_os_error()), // an invalid handle, or type size for the given attribute?
    }
}

// Safe API to get window position. Returns screen relative coordinates.
pub fn get_window_rect(hwnd: HWND) -> Result<RECT, Error> {
    // Call API.
    let ret = get_window_attribute::<RECT>(hwnd, DWMWA_EXTENDED_FRAME_BOUNDS);

    // Check for Errors.
    match ret {
        | Ok(rect) => Ok(rect),             // Return wrapped RECT.
        | _ => Err(Error::last_os_error()), // Most likely an invalid handle.
    }
}

// Safe API to set the current process as DPI aware.
pub fn set_process_dpi_aware_context(ctx: DPI_AWARENESS_CONTEXT) -> Result<bool, Error> {
    // Call API.
    let ret = unsafe { SetProcessDpiAwarenessContext(ctx) };

    // Check for Errors.
    match ret != 0 {
        | false => Err(Error::last_os_error()), // Most likely an invalid handle.
        | true => Ok(true),                     // Return wrapped true on success.
    }
}

// Safe API to retrieve the last active popup of the specified window.
// Doesn't detect invalid handles.
pub fn get_last_active_popup(hwnd: HWND) -> HWND {
    // Returns 0 if the HWND isn't valid.
    unsafe { GetLastActivePopup(hwnd) }
}

// Safe API to retrieve title bar information for the specified titlebar.
pub fn get_titlebar_info(hwnd: HWND) -> Result<TITLEBARINFO, Error> {
    // Fill structure.
    let mut ti: TITLEBARINFO = TITLEBARINFO {
        cbSize: size_of::<TITLEBARINFO>() as u32,
        rcTitleBar: RECT { left: 0, top: 0, right: 0, bottom: 0 },
        rgstate: [0; 6],
    };

    // Call API.
    let ret = unsafe { GetTitleBarInfo(hwnd, &mut ti as *mut _ as PTITLEBARINFO) };

    // Check for Errors.
    match ret {
        | 0 => Err(Error::last_os_error()), // Most likely an invalid handle.
        | _ => Ok(ti),                      // Return wrapped TITLEBARINFO.
    }
}

// Safe API to set window position. Takes in screen relative coordinates.
pub fn set_window_pos(hwnd: HWND, rect: RECT) -> Result<i32, Error> {
    // Run windows API to get client (inner frame) coordinates and return client RECT.
    let client = match get_window_rect(hwnd) {
        | Ok(ret) => ret,              // Unwrap frame.
        | Err(err) => return Err(err), // Most likely an invalid handle.
    };

    // Run windows API to get frame and return frame RECT.
    let frame = match get_window_attribute::<RECT>(hwnd, DWMWA_EXTENDED_FRAME_BOUNDS) {
        | Ok(ret) => ret,              // Unwrap frame.
        | Err(err) => return Err(err), // Most likely an invalid handle.
    };

    // Compute borders from frame and client.
    let mut border = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    border.left = frame.left - client.left;
    border.top = frame.top - client.top;
    border.right = client.right - frame.right;
    border.bottom = client.bottom - frame.bottom;

    // Adjust  because the windows API for setting position is stupid and not screen relative.
    let ret = unsafe {
        SetWindowPos(
            hwnd,
            ptr::null_mut(),
            rect.left - border.left,
            rect.top - border.top,
            rect.right - rect.left + border.left + border.right,
            rect.bottom - rect.top + border.top + border.bottom,
            SWP_NOACTIVATE | SWP_NOZORDER,
        )
    };

    // Make sure the return value was valid before returning.
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
    let lparam = &mut trait_obj as *mut _ as LPARAM; // coerce pointer.

    // C-compatible EnumWindows callback to call closure.
    extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let func: &mut &mut dyn FnMut(HWND) -> i32 =
            unsafe { &mut *(lparam as *mut LPARAM as *mut _) }; // coerce pointer inverse.

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

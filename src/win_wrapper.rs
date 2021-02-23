#![macro_use]
extern crate winapi;
use std::io::Error;
use std::lazy::SyncLazy;
use std::mem;
use std::mem::size_of;
use std::ptr;
use std::{collections::hash_map::HashMap, sync::RwLock};
use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::shared::*;
use winapi::um::dwmapi::*;
use winapi::um::winnt::*;
use winapi::um::winuser;
use winapi::um::winuser::*;

pub type HWND = windef::HWND;
pub type RECT = windef::RECT;
pub type HWINEVENTHOOK = windef::HWINEVENTHOOK;

//pub const EVENT_SYSTEM_MOVESIZESTART: UINT = winuser:: EVENT_SYSTEM_MOVESIZESTART;
pub const EVENT_SYSTEM_MOVESIZEEND: UINT = winuser::EVENT_SYSTEM_MOVESIZEEND;

pub const DPI_AWARENESS_CONTEXT_UNAWARE: DPI_AWARENESS_CONTEXT =
    windef::DPI_AWARENESS_CONTEXT_UNAWARE;
pub const DPI_AWARENESS_CONTEXT_SYSTEM_AWARE: DPI_AWARENESS_CONTEXT =
    windef::DPI_AWARENESS_CONTEXT_SYSTEM_AWARE;
pub const DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE: DPI_AWARENESS_CONTEXT =
    windef::DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE;

pub const DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2: DPI_AWARENESS_CONTEXT =
    windef::DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2;

pub const DPI_AWARENESS_CONTEXT_UNAWARE_GDISCALED: DPI_AWARENESS_CONTEXT =
    windef::DPI_AWARENESS_CONTEXT_UNAWARE_GDISCALED;

// Safe API to retrieve Windows Messages. Returns None on WM_QUIT result.
pub fn get_message() -> Result<Option<MSG>, String> {
    // Initialize memory.
    let mut msg: MSG = unsafe { mem::MaybeUninit::zeroed().assume_init() };

    // Get Message.
    let ret = unsafe { GetMessageW(&mut msg as *mut MSG, ptr::null_mut(), 0, 0) };

    // Check for errors before returning message.
    match ret {
        | 0 => Ok(None),
        | -1 => Err(Error::last_os_error().to_string()),
        | _ => Ok(Some(msg)),
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
pub fn _msgbox(title: &str, msg: &str) -> Result<i32, String> {
    use std::ffi::OsStr;
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    let title: Vec<u16> = OsStr::new(title).encode_wide().chain(once(0)).collect();
    let msg: Vec<u16> = OsStr::new(msg).encode_wide().chain(once(0)).collect();
    let ret = unsafe { MessageBoxW(ptr::null_mut(), msg.as_ptr(), title.as_ptr(), MB_OK) };

    match ret {
        | 0 => Err(Error::last_os_error().to_string()),
        | _ => Ok(ret),
    }
}

// Safe API to check if a window shows on the taskbar.
pub fn is_taskbar_window(hwnd: HWND) -> bool {
    // Invisible windows do not show on the taskbar.
    match is_window_visible(hwnd) {
        | false => return false, // Invisible.
        | true => (),            // Visible.
    };

    // Cloaked windows do not show on the taskbar.
    match is_window_cloaked(hwnd) {
        | Ok(ret) if ret != false => return false, // Cloaked.
        | Ok(_) => (),                             // Not Cloaked.
        | Err(_) => return false,                  // Drop-out if API call failed.
    };

    // Check Windows Extended style: App windows alway show on the task bar and normal and no-active windows do not.
    // Note: no-active windows can show in the taskbar if WS_EX_APPWINDOW is set, hence the order.
    match get_window_long(hwnd, GWL_EXSTYLE) {
        | Ok(ret) if ret & WS_EX_APPWINDOW != 0 => return true, // App Window
        | Ok(ret) if ret & (WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE) != 0 => return false, // Tool or no-active window.
        | Ok(_) => (), // Need more checks to determine if the window should be displayed.
        | Err(_) => return false, // Drop-out if API call failed.
    }

    // Check Window Style: Child windows do not show on the task bar.
    match get_window_long(hwnd, GWL_STYLE) {
        | Ok(ret) if ret & WS_CHILD != 0 => return false, // Child window.
        | Ok(_) => (),                                    // Not child window.
        | Err(_) => return false,                         // Drop-out if API call failed.
    }

    // Windows with owner's do not appear on the taskbar.
    match get_window(hwnd, GW_OWNER) as u32 {
        | 0 => (),           // No owner.
        | _ => return false, // An owner.
    };

    // Windows with invisible titlebars do not appear on the taskbar.
    match get_titlebar_info(hwnd) {
        | Ok(ti) if ti.rgstate[0] & STATE_SYSTEM_INVISIBLE != 0 => return false, // Invisible titlebar.
        | Ok(_) => (),            // Visible titlebar.
        | Err(_) => return false, // Drop-out if API call failed.
    };

    return true; // Appears to be an taskbar window.
}

// Safe API to retrieve whether a window is iconic (minimized).
// Doesn't detect invalid handles.
pub fn is_window_iconic(hwnd: HWND) -> bool {
    // Returns 0 if the HWND isn't valid.
    return unsafe { IsIconic(hwnd) } != 0;
}

// Safe API to retrieve whether a window is minimized.
pub fn is_window_minimized(hwnd: HWND) -> Result<bool, String> {
    // Call API or return error.
    let ret = get_window_placement(hwnd)?; // Failure most likely an invalid handle.

    Ok(ret.showCmd == SW_SHOWMINIMIZED as u32) // Return wrapped bool.
}

// Safe API to retrieve whether a window is maximized.
pub fn is_window_maximized(hwnd: HWND) -> Result<bool, String> {
    // Call API or return error.
    let ret = get_window_placement(hwnd)?; // Failure most likely an invalid handle.

    Ok(ret.showCmd == SW_SHOWMAXIMIZED as u32) // Return wrapped bool.
}

// Safe API to retrieve window visibility.
// Doesn't detect invalid handles.
pub fn is_window_visible(hwnd: HWND) -> bool {
    // Returns 0 if the HWND isn't valid.
    return unsafe { IsWindowVisible(hwnd) } != 0;
}

// Safe API to retrieve window cloaked state.
pub fn is_window_cloaked(hwnd: HWND) -> Result<bool, String> {
    // Call API or return error.
    let ret = get_window_attribute::<i32>(hwnd, DWMWA_CLOAKED)?;

    return Ok(ret != 0); // Return wrapped bool.
}

// Safe API to retrieve a handle to a window that has the specified relationship to the specified window.
// Doesn't detect invalid handles.
pub fn get_window(hwnd: HWND, cmd: UINT) -> HWND {
    // Returns 0 if the HWND isn't valid or the relationship doesn't exist.
    unsafe { GetWindow(hwnd, cmd) }
}

//Safe API to retrieve information for the specified window.
pub fn get_window_long(hwnd: HWND, n_index: i32) -> Result<DWORD, String> {
    // Call API.
    let ret = unsafe { GetWindowLongW(hwnd, n_index) };

    // Check for Errors.
    match ret {
        | 0 => Err(Error::last_os_error().to_string()), // Most likely an invalid handle.
        | _ => Ok(ret as u32), // Return wrapped LONG as a DWORD (bitfield).
    }
}

// Safe API to retrieve the ancestor of the specified window.
// Doesn't detect invalid handles.
pub fn get_window_ancestor(hwnd: HWND, flags: UINT) -> HWND {
    // Returns 0 if the HWND isn't valid.
    unsafe { GetAncestor(hwnd, flags) }
}

// Safe API to retrieve window placement.
pub fn get_window_placement(hwnd: HWND) -> Result<WINDOWPLACEMENT, String> {
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
        | 0 => Err(Error::last_os_error().to_string()), // Most likely an invalid handle.
        | _ => Ok(wp), // Return wrapped WINDOWPLACEMENT information.
    }
}

// Safe API to retrieve window attributes.
// Unsafe for unexpected types.
pub fn get_window_attribute<T>(hwnd: HWND, dw_attribute: DWORD) -> Result<T, String> {
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

    // Check for Errors. HRESULT of non-zero is an error. Good luck decoding this.
    match ret {
        | winerror::S_OK => Ok(pv_attribute), // Wrapped attribute.
        | _ => Err(format!("Returned HRESULT: 0x{:x}", ret)), // an invalid handle, or type size for the given attribute?
    }
}

// Safe API to get window frame position (client area). Returns screen relative coordinates.
pub fn get_window_rect(hwnd: HWND) -> Result<RECT, String> {
    // Initialize unknown type to zero.
    let mut rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };

    // Call API.
    let (ret, client) = unsafe {
        let ptr = &mut rect as *const RECT as LPRECT;
        let ret = GetWindowRect(hwnd, ptr);
        (ret, rect)
    };

    // Check for Errors.
    match ret {
        | 0 => Err(Error::last_os_error().to_string()), // Most likely an invalid handle.
        | _ => Ok(client),                              // Return wrapped RECT.
    }
}

// Safe API to get window outer frame position. Returns screen relative coordinates.
pub fn get_window_frame_rect(hwnd: HWND) -> Result<RECT, String> {
    // Call API or return Error.
    let rect = get_window_attribute::<RECT>(hwnd, DWMWA_EXTENDED_FRAME_BOUNDS)?; // Most likely an invalid handle.

    Ok(rect) // Return wrapped RECT.
}

// Safe API to set the current process as DPI aware.
pub fn set_process_dpi_aware_context(ctx: DPI_AWARENESS_CONTEXT) -> Result<bool, String> {
    // Call API.
    let ret = unsafe { SetProcessDpiAwarenessContext(ctx) };

    // Check for Errors.
    match ret != 0 {
        | false => Err(Error::last_os_error().to_string()), // Most likely an invalid handle.
        | true => Ok(true),                                 // Return wrapped true on success.
    }
}

// Safe API to retrieve the last active popup of the specified window.
// Doesn't detect invalid handles.
pub fn get_last_active_popup(hwnd: HWND) -> HWND {
    // Returns 0 if the HWND isn't valid.
    unsafe { GetLastActivePopup(hwnd) }
}

// Safe API to retrieve title bar information for the specified titlebar.
pub fn get_titlebar_info(hwnd: HWND) -> Result<TITLEBARINFO, String> {
    // Fill structure.
    let mut ti: TITLEBARINFO = TITLEBARINFO {
        cbSize: size_of::<TITLEBARINFO>() as u32, // size must be set before calling GetTitleBarInfo.
        rcTitleBar: RECT { left: 0, top: 0, right: 0, bottom: 0 },
        rgstate: [0; 6],
    };

    // Call API.
    let ret = unsafe { GetTitleBarInfo(hwnd, &mut ti as *mut _ as PTITLEBARINFO) };

    // Check for Errors.
    match ret {
        | 0 => Err(Error::last_os_error().to_string()), // Most likely an invalid handle.
        | _ => Ok(ti),                                  // Return wrapped TITLEBARINFO.
    }
}

// Safe API to set window position. Takes in screen relative coordinates.
pub fn set_window_pos(hwnd: HWND, rect: RECT) -> Result<i32, String> {
    // Run windows API to get client (inner frame) coordinates and return client RECT.
    // An error result is most likely an invalid handle.
    let client = get_window_rect(hwnd)?;

    // Run windows API to get frame and return frame RECT.
    // An error result is most likely an invalid handle.
    let frame = get_window_frame_rect(hwnd)?;

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
        | 0 => Err(Error::last_os_error().to_string()),
        | _ => Ok(ret),
    }
}

// Required trait for closures passed to enum_windows.
pub trait FnEnum = Fn(HWND) -> i32;

// Safe API to enumerate windows.
// Takes a closure.
pub fn enum_windows<F>(func: F)
where
    F: FnEnum,
{
    // C-compatible EnumWindows callback to call closure.
    extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let func: &&dyn FnEnum = unsafe { &*(lparam as *const _) }; // coerce pointer inverse.

        return func(hwnd); // Call closure
    }

    // Implement trait to pass closure as lparam.
    let trait_obj: &dyn FnEnum = &func;
    let lparam = &trait_obj as *const _ as LPARAM; // coerce pointer.

    // Run Callback API.
    let callback: WNDENUMPROC = Some(enum_windows_callback);
    unsafe {
        EnumWindows(callback, lparam);
    }
}

// The following code is a kludge to safely pass data via closures to the SetWinEventHook() API.
// This is necessary because the API does not allow passing of user provided data (such as
// a closure environment, and utilizes an ABI that Rust closures cannot be directly called from.
//
// Implemented is a static lookup table operating as a trampoline for Fn handlers.
// These are looked up by unique HWINEVENTHOOK id and called from within a stub a
// WINEVENTPROC handler function (implemented below).

// Required trait for closures passed to set_win_event_hook_callback.
pub trait FnHook = Fn(usize, DWORD, HWND, LONG, LONG, DWORD, DWORD) + 'static + Send + Sync;

// Multiple reader, single writer lock implementation.
// Maps: Unique HWINEVENTHOOK ID -> Fn(...)
static EVENT_HOOK_MAP: SyncLazy<RwLock<HashMap<usize, Box<dyn FnHook>>>> =
    SyncLazy::new(|| RwLock::new(HashMap::new()));

// Safe API to setup callbacks to listen for events.
// Takes a closure and windows event constants.
pub fn set_win_event_hook<F>(
    func: F,
    event_min: UINT,
    event_max: UINT,
) -> Result<HWINEVENTHOOK, String>
where
    F: FnHook,
{
    // C-compatible SetWinEventHook callback stub to lookup and call user provided closures.
    extern "system" fn set_win_event_hook_callback(
        h_win_event_hook: HWINEVENTHOOK,
        event: DWORD,
        hwnd: HWND,
        id_object: LONG,
        id_child: LONG,
        id_event_thread: DWORD,
        dwms_event_time: DWORD,
    ) {
        //Guard actual closure handler for multiple read access and pass in arguments.
        if let Ok(guard) = EVENT_HOOK_MAP.read() {
            let map = &*guard; // Get a reference to the map which is guaranteed to be initialized.
            if let Some(func) = map.get(&(h_win_event_hook as usize)) {
                // Call closure.
                func(
                    h_win_event_hook as usize,
                    event,
                    hwnd,
                    id_object,
                    id_child,
                    id_event_thread,
                    dwms_event_time,
                );
            }
        }
    }

    // Call API.
    let hook = unsafe {
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

    match hook {
        | hook if hook.is_null() => Err("Failed to setup event hook!".to_string()), // Failed to setup a hook.
        | hook => {
            // Acquire single-writer lock for safely adding a new entry to our closure lookup table.
            // We lock after setting up the stub callback because we can safely miss intransit events.
            // The lookup will simply fail and teh callback will return.
            EVENT_HOOK_MAP.write().unwrap().insert(hook as usize, Box::new(func));
            Ok(hook)
        } // Return wrapped hook.
    }
}

// Safe API to unhook event listeners.
pub fn unhook_win_event(h_win_even_hook: HWINEVENTHOOK) -> Result<bool, String> {
    // Call API.
    let ret = unsafe { UnhookWinEvent(h_win_even_hook) };

    // Remove entry from lookup table.
    EVENT_HOOK_MAP.write().unwrap().remove(&(h_win_even_hook as usize));

    // Check for Errors.
    match ret != 0 {
        | false => Err(Error::last_os_error().to_string()), // Most likely an invalid handle.
        | true => Ok(true),                                 // Return wrapped true on success.
    }
}

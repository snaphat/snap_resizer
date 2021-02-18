#[cfg(windows)]
extern crate winapi;
mod win_wrapper;
use win_wrapper::*;
use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::um::winnt::*;
use winapi::um::winuser::*;

macro_rules! unwrap_or_return {
    ( $e:expr ) => {
        match $e {
            | Some(x) => x,
            | _ => return,
        }
    };
}

extern "system" fn set_win_event_hook_callback(
    _: HWINEVENTHOOK,
    event: DWORD,
    hwnd: HWND,
    _: LONG,
    id_child: LONG,
    _: DWORD,
    _: DWORD,
) {
    if event != EVENT_SYSTEM_MOVESIZEEND || id_child != 0 {
        return;
    }
    println!("Move event...");
    let rect = unwrap_or_return!(get_window_bounds(hwnd));

    // Enum handler
    let enum_closure = |o_hwnd: HWND| -> i32 {
        if !is_window_visible(o_hwnd) {
            return 1;
        }

        if let Some(o_rect) = get_window_bounds(o_hwnd) {
            println!(
                "(x,y)\nhandle: {:?}\nStart: ({}, {})\nStop: ({}, {})",
                hwnd, rect.left, rect.top, rect.right, rect.bottom
            );
            println!(
                "\n(x,y)\nhandle: {:?}\nStart: ({}, {})\nStop: ({}, {})",
                o_hwnd, o_rect.left, o_rect.top, o_rect.right, o_rect.bottom
            );
            println!("==========================");
        }

        return 1;
    };
    enum_windows(enum_closure);
}

fn main() {
    if set_win_event_hook(set_win_event_hook_callback) == false {
        msgbox!("Err", "Failed to set move/resize event hook!");
        return;
    }

    loop {
        let msg = get_message();
        match msg {
            | Some(msg) => {
                translate_message(&msg);
                dispatch_message(&msg);
            }
            | _ => break,
        };
    }
}

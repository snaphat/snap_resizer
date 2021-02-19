#[cfg(windows)]
mod win_wrapper;
use win_wrapper::*;

/*
macro_rules! unwrap_or_return {
    ( $e:expr ) => {
        match $e {
            | Some(x) => x,
            | _ => return,
        }
    };
}
*/

macro_rules! unwrap_or_return {
    ( $e:expr ) => {
        match $e {
            | Ok(x) => x,
            | _ => return,
        }
    };
}

// Enumerate Windows Handler
fn enum_handler(m_hwnd: HWND, o_hwnd: HWND, m_rect: RECT) -> i32 {
    // Ignore invisible windows.
    if !is_window_visible(o_hwnd) {
        return 1; // Return 1 to continue enumerating.
    }

    // Get bounds of enumerated window.
    if let Ok(o_rect) = get_window_rect(o_hwnd) {
        // Print moved/resized Window handle and coordinates.
        println!(
            "(x,y)\nhandle: {:?}\nStart: ({}, {})\nStop: ({}, {})",
            m_hwnd, m_rect.left, m_rect.top, m_rect.right, m_rect.bottom
        );

        // Print other window Window handle and coordinates.
        println!(
            "\n(x,y)\nhandle: {:?}\nStart: ({}, {})\nStop: ({}, {})",
            o_hwnd, o_rect.left, o_rect.top, o_rect.right, o_rect.bottom
        );
        println!("==========================");
    }

    return 1; // Return 1 to continue enumerating.
}

// System Event Handler
fn event_handler(event: u32, m_hwnd: HWND, id_child: i32) {
    // Return if the event isn't for us.
    if event != EVENT_SYSTEM_MOVESIZEEND || id_child != 0 {
        return;
    }

    // Retrieve bounds for the moved window or return if failed.
    let m_rect = unwrap_or_return!(get_window_rect(m_hwnd));

    // Setup closure for EnumWindow callback. Done this way for readability.
    let enum_closure = |o_hwnd| -> i32 { enum_handler(m_hwnd, o_hwnd, m_rect) };

    // Enumerate windows.
    enum_windows(enum_closure);
}

fn main() {
    // Setup closure for event hook. Done this way for readability.
    let func = |event, m_hwnd, id_child| {
        event_handler(event, m_hwnd, id_child);
    };

    // Setup hook.
    if set_win_event_hook(func, EVENT_SYSTEM_MOVESIZEEND, EVENT_SYSTEM_MOVESIZEEND) == false {
        msgbox!("Err", "Failed to set move/resize event hook!");
        return;
    }

    // Run safe windows message pump.
    loop {
        // Wait for message (blocking).
        let msg = get_message();

        // Handle message.
        match msg {
            | Some(msg) => {
                translate_message(&msg);
                dispatch_message(&msg);
            }
            | _ => break, // Quit if WM_Quit or error.
        };
    }
}

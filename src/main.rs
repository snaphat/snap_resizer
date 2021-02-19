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

const MOD:i32 = 12;

// Enumerate Windows Handler
fn enum_handler(m_hwnd: HWND, o_hwnd: HWND, mut m_rect: RECT) -> i32 {
    // Ignore invisible windows.
    if !is_window_visible(o_hwnd) {
        return 1; // Return 1 to continue enumerating.
    }

    // Get bounds of enumerated window.
    if let Ok(o_rect) = get_window_rect(o_hwnd) {
        let mut reposition = false;
        //println!("{} = {} - {}",  i32::abs(m_rect.right - o_rect.left), m_rect.right, o_rect.left);
        if i32::abs(-MOD + m_rect.right - o_rect.left) < MOD*2 {
            println!("Window on left");
            println!("{} {}", m_rect.right, o_rect.left);
            m_rect.right = o_rect.left + MOD;
            reposition = true;
        }
        else if m_rect.left > 1 && i32::abs(MOD + m_rect.left - o_rect.right) < MOD*2 {
            println!("Window on right");
            println!("{} {}", m_rect.left, o_rect.right);
            m_rect.left = o_rect.right;
            reposition = true;
        }
        else if m_rect.top > 1 && i32::abs(-MOD + m_rect.bottom - o_rect.top) < MOD*2 {
            println!("Window on top");
            println!("{} {}", m_rect.bottom, o_rect.top);
            m_rect.bottom = o_rect.top + MOD/2;
            reposition = true;
        }
        else if m_rect.top > 1 && i32::abs(MOD + m_rect.top - o_rect.bottom) < MOD*2 {
            println!("Window on bottom");
            println!("{} {}", m_rect.top, o_rect.bottom);
            m_rect.top = o_rect.bottom - MOD/2;
            reposition = true;
        }

        if reposition {
            if let Err(a) = set_window_pos(m_hwnd, m_rect) {
                println!("{}", a);
            } else {
                println!("success");
            }
        }

        // Print moved/resized Window handle and coordinates.
        //println!(
        //    "(x,y)\nhandle: {:?}\nStart: ({}, {})\nStop: ({}, {})",
        //    m_hwnd, m_rect.left, m_rect.top, m_rect.right, m_rect.bottom
        //);
        //
        //// Print other window Window handle and coordinates.
        //println!(
        //    "\n(x,y)\nhandle: {:?}\nStart: ({}, {})\nStop: ({}, {})",
        //    o_hwnd, o_rect.left, o_rect.top, o_rect.right, o_rect.bottom
        //);
        //println!("==========================");
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

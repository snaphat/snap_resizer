use winapi_safe::constants::*;
use winapi_safe::*;

const THRESH: i32 = 40;

// Enumerate Windows Handler
fn enum_handler(m_hwnd: HWND, o_hwnd: HWND, mut m_rect: RECT) -> i32 {
    // Ignore minimized windows.
    if let Ok(ret) = is_window_minimized(o_hwnd) {
        if ret == true {
            return 1;
        }
    } else {
        return 1; // Drop-out if API call failed.
    }

    // Ignore maximized windows.
    if let Ok(ret) = is_window_maximized(o_hwnd) {
        if ret == true {
            return 1;
        }
    } else {
        return 1; // Drop-out if API call failed.
    }

    // Ignore non-taskbar windows.
    if !is_taskbar_window(o_hwnd) {
        return 1; // Return 1 to continue enumerating.
    }

    // Get bounds of enumerated window.
    if let Ok(o_rect) = get_window_frame_rect(o_hwnd) {
        //
        let (thread_id, process_id) = get_window_thread_process_id(1 as HWND);
        println!(
            "{:?} {} {} {} {} id t:{} p:{}",
            o_hwnd, o_rect.left, o_rect.top, o_rect.right, o_rect.bottom, thread_id, process_id
        );

        // Compare positions and snap windows that are close by.
        let mut reposition = false;
        if i32::abs(m_rect.right - o_rect.left) < THRESH {
            println!("Window on left");
            m_rect.right = o_rect.left;
            reposition = true;
        } else if i32::abs(m_rect.left - o_rect.right) < THRESH {
            println!("Window on right");
            m_rect.left = o_rect.right;
            reposition = true;
        } else if i32::abs(m_rect.bottom - o_rect.top) < THRESH {
            println!("Window on top");
            m_rect.bottom = o_rect.top;
            reposition = true;
        } else if i32::abs(m_rect.top - o_rect.bottom) < THRESH {
            println!("Window on bottom");
            m_rect.top = o_rect.bottom;
            reposition = true;
        }

        // Apply new position.
        if reposition {
            if let Err(err) = set_window_pos(m_hwnd, m_rect) {
                println!("{}", err);
            } else {
                return 0; // Stop enumerating.
            }
        }
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
    let m_rect = get_window_frame_rect(m_hwnd);
    match m_rect {
        | Ok(m_rect) => {
            // Setup closure for EnumWindow callback. Done this way for readability.
            let enum_closure = |o_hwnd| -> i32 { enum_handler(m_hwnd, o_hwnd, m_rect) };

            println!("\n========\n");

            // Enumerate windows.
            enum_windows(enum_closure);
        }
        | Err(err) => println!("{}", err),
    };
}

fn main() {
    // Set the process as DPI aware.
    if let Err(err) = set_process_dpi_aware_context(DPI_AWARENESS_CONTEXT_SYSTEM_AWARE) {
        msgbox!("Error", "{}", err);
    }

    // Setup closure for event hook. Done this way for readability.
    let func = |_, event, hwnd, _, id_child, _, _| {
        event_handler(event, hwnd, id_child);
    };

    // Setup hook.
    if let Err(err) = set_win_event_hook(func, EVENT_SYSTEM_MOVESIZEEND, EVENT_SYSTEM_MOVESIZEEND) {
        msgbox!("Error", "{}", err);
        return;
    }

    // Run safe windows message pump.
    loop {
        // Wait for message (blocking).
        if let Ok(msg) = get_message() {
            if let Some(msg) = msg {
                // Handle message.
                translate_message(&msg);
                dispatch_message(&msg);
            } else {
                return; // Return on WM_QUIT.
            }
        } else {
            return; // Return on error.
        }
    }
}

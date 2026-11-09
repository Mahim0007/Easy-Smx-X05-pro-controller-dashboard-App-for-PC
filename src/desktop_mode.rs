use crate::config::AppConfig;
use crate::controller::ControllerState;

#[cfg(windows)]
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
    MOUSEEVENTF_MOVE, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_WHEEL, MOUSEINPUT,
};

static mut LAST_A_STATE: bool = false;
static mut LAST_B_STATE: bool = false;

pub fn handle_desktop_navigation(controller: &ControllerState, config: &AppConfig) {
    #[cfg(windows)]
    unsafe {
        // Move mouse via Left Thumbstick
        let deadzone = 0.15; // slightly higher deadzone for desktop mouse stability
        let mag = (controller.left_stick_x.powi(2) + controller.left_stick_y.powi(2)).sqrt();

        if mag > deadzone {
            let speed = config.mouse_sensitivity * (mag - deadzone) / (1.0 - deadzone);
            let dx = (controller.left_stick_x * speed) as i32;
            let dy = (-controller.left_stick_y * speed) as i32; // Invert Y for screen coordinates

            send_mouse_move(dx, dy);
        }

        // Left Click via Controller A button
        let a_pressed = controller.buttons.a;
        if a_pressed && !LAST_A_STATE {
            send_mouse_button(MOUSEEVENTF_LEFTDOWN);
        } else if !a_pressed && LAST_A_STATE {
            send_mouse_button(MOUSEEVENTF_LEFTUP);
        }
        LAST_A_STATE = a_pressed;

        // Right Click via Controller B button
        let b_pressed = controller.buttons.b;
        if b_pressed && !LAST_B_STATE {
            send_mouse_button(MOUSEEVENTF_RIGHTDOWN);
        } else if !b_pressed && LAST_B_STATE {
            send_mouse_button(MOUSEEVENTF_RIGHTUP);
        }
        LAST_B_STATE = b_pressed;

        // Scroll Wheel via D-Pad Up / Down
        if controller.buttons.dpad_up {
            send_mouse_wheel(120);
        } else if controller.buttons.dpad_down {
            send_mouse_wheel(-120);
        }
    }
}

#[cfg(windows)]
unsafe fn send_mouse_move(dx: i32, dy: i32) {
    let input = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx,
                dy,
                mouseData: 0,
                dwFlags: MOUSEEVENTF_MOVE,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    let _ = SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
}

#[cfg(windows)]
unsafe fn send_mouse_button(flag: windows::Win32::UI::Input::KeyboardAndMouse::MOUSE_EVENT_FLAGS) {
    let input = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: 0,
                dwFlags: flag,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    let _ = SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
}

#[cfg(windows)]
unsafe fn send_mouse_wheel(delta: i32) {
    let input = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: delta as u32,
                dwFlags: MOUSEEVENTF_WHEEL,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    let _ = SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
}


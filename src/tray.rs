use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread;

static TRAY_SHOW_REQUEST: AtomicBool = AtomicBool::new(false);
static TRAY_EXIT_REQUEST: AtomicBool = AtomicBool::new(false);

#[derive(Clone)]
enum TrayCommand {
    UpdateTooltip(String),
    ShowBalloon { title: String, text: String, is_warning: bool },
}

static CMD_SENDER: Mutex<Option<std::sync::mpsc::Sender<TrayCommand>>> = Mutex::new(None);

pub fn take_show_request() -> bool {
    TRAY_SHOW_REQUEST.swap(false, Ordering::SeqCst)
}

pub fn take_exit_request() -> bool {
    TRAY_EXIT_REQUEST.swap(false, Ordering::SeqCst)
}

pub fn update_tooltip(text: impl Into<String>) {
    let text = text.into();
    if let Ok(guard) = CMD_SENDER.lock() {
        if let Some(tx) = guard.as_ref() {
            let _ = tx.send(TrayCommand::UpdateTooltip(text));
        }
    }
}

pub fn show_balloon(title: impl Into<String>, text: impl Into<String>, is_warning: bool) {
    let title = title.into();
    let text = text.into();
    if let Ok(guard) = CMD_SENDER.lock() {
        if let Some(tx) = guard.as_ref() {
            let _ = tx.send(TrayCommand::ShowBalloon { title, text, is_warning });
        }
    }
}

#[cfg(windows)]
extern "system" {
    fn MessageBeep(uType: u32) -> i32;
}

#[cfg(windows)]
pub fn play_alert_sound(is_warning: bool) {
    unsafe {
        if is_warning {
            // MB_ICONWARNING / Exclamation chime (0x00000030)
            MessageBeep(0x00000030);
        } else {
            // MB_ICONASTERISK / Information chime (0x00000040)
            MessageBeep(0x00000040);
        }
    }
}

#[cfg(not(windows))]
pub fn play_alert_sound(_is_warning: bool) {}

#[cfg(windows)]
pub fn init_tray() {
    let (tx, rx) = std::sync::mpsc::channel();
    if let Ok(mut guard) = CMD_SENDER.lock() {
        *guard = Some(tx);
    }

    thread::Builder::new()
        .name("system-tray".to_string())
        .spawn(move || {
            unsafe {
                run_tray_loop(rx);
            }
        })
        .expect("Failed to start tray thread");
}

#[cfg(not(windows))]
pub fn init_tray() {}

#[cfg(windows)]
use windows::Win32::Foundation::{HANDLE, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::HICON;

#[cfg(windows)]
#[repr(C)]
struct CustomWndClassW {
    style: u32,
    lpfn_wnd_proc: Option<unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT>,
    cb_cls_extra: i32,
    cb_wnd_extra: i32,
    h_instance: HINSTANCE,
    h_icon: HICON,
    h_cursor: HICON,
    hbr_background: HANDLE,
    lpsz_menu_name: *const u16,
    lpsz_class_name: *const u16,
}

#[cfg(windows)]
extern "system" {
    fn RegisterClassW(lpWndClass: *const CustomWndClassW) -> u16;
}

#[cfg(windows)]
pub unsafe fn restore_main_window() {
    use windows::core::PCWSTR;
    use windows::Win32::UI::WindowsAndMessaging::{FindWindowW, SetForegroundWindow, ShowWindow, SW_RESTORE, SW_SHOW};

    let title: Vec<u16> = "EasySMX X05 Pro - Gamepad Suite & Optimizer\0".encode_utf16().collect();
    let hwnd = FindWindowW(PCWSTR::null(), PCWSTR(title.as_ptr())).unwrap_or_default();
    if !hwnd.is_invalid() {
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = ShowWindow(hwnd, SW_RESTORE);
        let _ = SetForegroundWindow(hwnd);
    }
}

#[cfg(windows)]
unsafe fn delete_tray_icon(hwnd: HWND) {
    use windows::Win32::UI::Shell::{Shell_NotifyIconW, NIM_DELETE, NOTIFYICONDATAW};
    let nid = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: 1001,
        ..Default::default()
    };
    let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
}

#[cfg(windows)]
unsafe fn run_tray_loop(rx: std::sync::mpsc::Receiver<TrayCommand>) {
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::*;
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::Shell::*;
    use windows::Win32::UI::WindowsAndMessaging::*;

    const WM_TRAY_CALLBACK: u32 = WM_APP + 101;
    const TRAY_ID: u32 = 1001;

    let class_name: Vec<u16> = "EasySMX_X05_Tray_Class\0".encode_utf16().collect();
    let window_title: Vec<u16> = "EasySMX_X05_Tray_Window\0".encode_utf16().collect();
    let hinstance = GetModuleHandleW(None).unwrap_or_default();

    let wc = CustomWndClassW {
        style: 0,
        lpfn_wnd_proc: Some(tray_wnd_proc),
        cb_cls_extra: 0,
        cb_wnd_extra: 0,
        h_instance: hinstance.into(),
        h_icon: HICON::default(),
        h_cursor: HICON::default(),
        hbr_background: HANDLE::default(),
        lpsz_menu_name: std::ptr::null(),
        lpsz_class_name: class_name.as_ptr(),
    };
    RegisterClassW(&wc);

    // Create a hidden top-level popup window to receive tray & menu messages
    let hwnd_res = CreateWindowExW(
        WINDOW_EX_STYLE::default(),
        PCWSTR(class_name.as_ptr()),
        PCWSTR(window_title.as_ptr()),
        WS_POPUP,
        0, 0, 0, 0,
        HWND::default(),
        HMENU::default(),
        hinstance,
        None,
    );

    let hwnd = match hwnd_res {
        Ok(h) if !h.is_invalid() => h,
        _ => return,
    };

    // Try loading embedded gamepad icon (Resource ID 1) from PE resources, or fallback to application icon
    let mut hicon = LoadIconW(hinstance, PCWSTR(1 as _)).unwrap_or_default();
    if hicon.is_invalid() {
        hicon = LoadIconW(HMODULE::default(), IDI_APPLICATION).unwrap_or_default();
    }

    let mut nid = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: TRAY_ID,
        uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
        uCallbackMessage: WM_TRAY_CALLBACK,
        hIcon: hicon,
        ..Default::default()
    };

    copy_to_wide_buf("EasySMX X05 Pro Suite - Active", &mut nid.szTip);
    let _ = Shell_NotifyIconW(NIM_ADD, &nid);

    // Set tray version for modern behavior (NIN_BALLOONUSERCLICK etc.)
    nid.Anonymous.uVersion = NOTIFYICON_VERSION_4;
    let _ = Shell_NotifyIconW(NIM_SETVERSION, &nid);

    // Message loop with non-blocking channel polling
    let mut msg = MSG::default();
    loop {
        // Drain pending commands from main app
        while let Ok(cmd) = rx.try_recv() {
            match cmd {
                TrayCommand::UpdateTooltip(text) => {
                    copy_to_wide_buf(&text, &mut nid.szTip);
                    nid.uFlags = NIF_TIP;
                    let _ = Shell_NotifyIconW(NIM_MODIFY, &nid);
                }
                TrayCommand::ShowBalloon { title, text, is_warning } => {
                    copy_to_wide_buf(&title, &mut nid.szInfoTitle);
                    copy_to_wide_buf(&text, &mut nid.szInfo);
                    nid.uFlags = NIF_INFO;
                    nid.dwInfoFlags = if is_warning { NIIF_WARNING } else { NIIF_INFO };
                    let _ = Shell_NotifyIconW(NIM_MODIFY, &nid);
                }
            }
        }

        // Process Win32 messages
        while PeekMessageW(&mut msg, HWND::default(), 0, 0, PM_REMOVE).as_bool() {
            if msg.message == WM_QUIT {
                let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
                return;
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        thread::sleep(std::time::Duration::from_millis(50));
    }
}

#[cfg(windows)]
unsafe extern "system" fn tray_wnd_proc(
    hwnd: windows::Win32::Foundation::HWND,
    msg: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::*;
    use windows::Win32::UI::WindowsAndMessaging::*;

    const WM_TRAY_CALLBACK: u32 = WM_APP + 101;
    const IDM_OPEN: usize = 2001;
    const IDM_EXIT: usize = 2002;
    const NIN_SELECT: u32 = 0x0400;
    const NIN_KEYSELECT: u32 = 0x0401;
    const WM_CONTEXTMENU: u32 = 0x007B;

    match msg {
        WM_TRAY_CALLBACK => {
            let event = (lparam.0 & 0xFFFF) as u32;
            match event {
                WM_LBUTTONUP | WM_LBUTTONDBLCLK | NIN_SELECT | NIN_KEYSELECT => {
                    restore_main_window();
                    TRAY_SHOW_REQUEST.store(true, Ordering::SeqCst);
                }
                WM_RBUTTONUP | WM_CONTEXTMENU => {
                    let mut pt = POINT::default();
                    let _ = GetCursorPos(&mut pt);
                    let _ = SetForegroundWindow(hwnd);

                    let menu = CreatePopupMenu().unwrap_or_default();
                    if !menu.is_invalid() {
                        let title: Vec<u16> = "EasySMX X05 Pro Suite\0".encode_utf16().collect();
                        let open_str: Vec<u16> = "Open Suite Window\0".encode_utf16().collect();
                        let exit_str: Vec<u16> = "Exit EasySMX Suite\0".encode_utf16().collect();

                        let _ = AppendMenuW(menu, MF_STRING | MF_DISABLED, 0, PCWSTR(title.as_ptr()));
                        let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
                        let _ = AppendMenuW(menu, MF_STRING, IDM_OPEN, PCWSTR(open_str.as_ptr()));
                        let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
                        let _ = AppendMenuW(menu, MF_STRING, IDM_EXIT, PCWSTR(exit_str.as_ptr()));

                        // TPM_RETURNCMD makes TrackPopupMenuEx return the chosen item ID synchronously!
                        let uflags = (TPM_LEFTALIGN | TPM_BOTTOMALIGN | TPM_RIGHTBUTTON | TPM_RETURNCMD).0;
                        let cmd_res = TrackPopupMenuEx(
                            menu,
                            uflags,
                            pt.x,
                            pt.y,
                            hwnd,
                            None,
                        );
                        let _ = DestroyMenu(menu);

                        let chosen = cmd_res.0 as usize;
                        if chosen == IDM_OPEN {
                            restore_main_window();
                            TRAY_SHOW_REQUEST.store(true, Ordering::SeqCst);
                        } else if chosen == IDM_EXIT {
                            delete_tray_icon(hwnd);
                            std::process::exit(0);
                        }
                    }
                }
                _ => {}
            }
            LRESULT(0)
        }
        WM_COMMAND => {
            let id = (wparam.0 & 0xFFFF) as usize;
            if id == IDM_OPEN {
                restore_main_window();
                TRAY_SHOW_REQUEST.store(true, Ordering::SeqCst);
            } else if id == IDM_EXIT {
                delete_tray_icon(hwnd);
                std::process::exit(0);
            }
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn copy_to_wide_buf(src: &str, dest: &mut [u16]) {
    let mut i = 0;
    for c in src.encode_utf16() {
        if i + 1 < dest.len() {
            dest[i] = c;
            i += 1;
        }
    }
    dest[i] = 0;
}

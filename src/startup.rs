#[cfg(windows)]
use windows::core::PCWSTR;
#[cfg(windows)]
use windows::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW,
    HKEY, HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_SZ,
};

const RUN_KEY_PATH: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const APP_NAME: &str = "EasySMXX05Suite";

/// Check if the app is currently configured to start automatically with Windows
pub fn is_startup_enabled() -> Result<bool, String> {
    #[cfg(windows)]
    unsafe {
        let mut h_key = HKEY::default();
        let subkey = to_wide_chars(RUN_KEY_PATH);

        let status = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey.as_ptr()),
            0,
            KEY_READ,
            &mut h_key,
        );

        if status.is_err() {
            return Ok(false);
        }

        let val_name = to_wide_chars(APP_NAME);
        let mut buffer = [0u16; 512];
        let mut buffer_size = (buffer.len() * 2) as u32;

        let query_status = RegQueryValueExW(
            h_key,
            PCWSTR(val_name.as_ptr()),
            None,
            None,
            Some(buffer.as_mut_ptr() as *mut u8),
            Some(&mut buffer_size),
        );

        let _ = RegCloseKey(h_key);
        Ok(query_status.is_ok())
    }

    #[cfg(not(windows))]
    Ok(false)
}

/// Enable or disable starting with Windows
pub fn set_startup_enabled(enable: bool) -> Result<(), String> {
    #[cfg(windows)]
    unsafe {
        let mut h_key = HKEY::default();
        let subkey = to_wide_chars(RUN_KEY_PATH);

        let status = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey.as_ptr()),
            0,
            KEY_WRITE | KEY_READ,
            &mut h_key,
        );

        if status.is_err() {
            return Err("Failed to open Windows Registry Run key.".to_string());
        }

        let val_name = to_wide_chars(APP_NAME);

        let result = if enable {
            // Get current executable path
            let current_exe = std::env::current_exe()
                .map_err(|e| format!("Could not get current executable path: {}", e))?;
            let exe_str = current_exe.to_string_lossy().to_string();
            let wide_exe = to_wide_chars(&exe_str);

            let reg_set = RegSetValueExW(
                h_key,
                PCWSTR(val_name.as_ptr()),
                0,
                REG_SZ,
                Some(std::slice::from_raw_parts(
                    wide_exe.as_ptr() as *const u8,
                    wide_exe.len() * 2,
                )),
            );

            if reg_set.is_ok() {
                Ok(())
            } else {
                Err("Failed to set registry value for Windows startup.".to_string())
            }
        } else {
            // Delete the value
            let _ = RegDeleteValueW(h_key, PCWSTR(val_name.as_ptr()));
            Ok(())
        };

        let _ = RegCloseKey(h_key);
        result
    }

    #[cfg(not(windows))]
    Ok(())
}

#[cfg(windows)]
fn to_wide_chars(s: &str) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}


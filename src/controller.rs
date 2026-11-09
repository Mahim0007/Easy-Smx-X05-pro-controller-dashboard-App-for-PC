use std::time::Instant;
use crate::config::AppConfig;

#[cfg(windows)]
use windows::Win32::System::Registry::{
    RegOpenKeyExW, RegEnumKeyExW, RegQueryValueExW, RegCloseKey, HKEY_LOCAL_MACHINE, KEY_READ, REG_VALUE_TYPE,
};
#[cfg(windows)]
use windows::core::w;

#[cfg(windows)]
use windows::Win32::UI::Input::XboxController::{
    XInputGetBatteryInformation, XInputGetState, XInputSetState, BATTERY_DEVTYPE_GAMEPAD,
    BATTERY_LEVEL_EMPTY, BATTERY_LEVEL_FULL, BATTERY_LEVEL_LOW, BATTERY_LEVEL_MEDIUM,
    XINPUT_BATTERY_INFORMATION, XINPUT_GAMEPAD_A, XINPUT_GAMEPAD_B, XINPUT_GAMEPAD_BACK,
    XINPUT_GAMEPAD_DPAD_DOWN, XINPUT_GAMEPAD_DPAD_LEFT, XINPUT_GAMEPAD_DPAD_RIGHT,
    XINPUT_GAMEPAD_DPAD_UP, XINPUT_GAMEPAD_LEFT_SHOULDER, XINPUT_GAMEPAD_LEFT_THUMB,
    XINPUT_GAMEPAD_RIGHT_SHOULDER, XINPUT_GAMEPAD_RIGHT_THUMB, XINPUT_GAMEPAD_START,
    XINPUT_GAMEPAD_X, XINPUT_GAMEPAD_Y, XINPUT_STATE, XINPUT_VIBRATION,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ConnectionMode {
    Disconnected,
    Wired,
    Wireless24G,
    Bluetooth,
}

impl ConnectionMode {
    pub fn display_label(&self) -> &'static str {
        match self {
            ConnectionMode::Wired => "Connected (Wired)",
            ConnectionMode::Wireless24G => "Connected (2.4GHz)",
            ConnectionMode::Bluetooth => "Connected (Bluetooth)",
            ConnectionMode::Disconnected => "Controller Disconnected",
        }
    }

    pub fn short_badge(&self) -> &'static str {
        match self {
            ConnectionMode::Wired => "Wired",
            ConnectionMode::Wireless24G => "2.4GHz",
            ConnectionMode::Bluetooth => "Bluetooth",
            ConnectionMode::Disconnected => "Disconnected",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatteryStatus {
    Disconnected,
    Full,
    Medium,
    Low,
    Empty,
    Unknown,
}

impl BatteryStatus {
    pub fn estimated_percentage(&self) -> u8 {
        match self {
            BatteryStatus::Full => 100,
            BatteryStatus::Medium => 65,
            BatteryStatus::Low => 20,
            BatteryStatus::Empty => 5,
            _ => 100,
        }
    }

    pub fn label(&self, is_charging: bool, mode: ConnectionMode) -> String {
        match mode {
            ConnectionMode::Disconnected => "Controller Disconnected".to_string(),
            ConnectionMode::Wired => {
                if is_charging {
                    format!("Charging ({}%)", self.estimated_percentage())
                } else {
                    match self {
                        BatteryStatus::Full => "100% (Pass-Through)".to_string(),
                        BatteryStatus::Medium => "~65% (Fully Powered)".to_string(),
                        _ => format!("{}%", self.estimated_percentage()),
                    }
                }
            }
            ConnectionMode::Wireless24G | ConnectionMode::Bluetooth => {
                match self {
                    BatteryStatus::Full => "100% (Full)".to_string(),
                    BatteryStatus::Medium => "~65% (Medium)".to_string(),
                    BatteryStatus::Low => "~20% (Low Battery)".to_string(),
                    BatteryStatus::Empty => "< 10% (Critical)".to_string(),
                    _ => format!("{}%", self.estimated_percentage()),
                }
            }
        }
    }

    pub fn short_badge(&self, is_charging: bool, mode: ConnectionMode) -> String {
        match mode {
            ConnectionMode::Disconnected => "Off".to_string(),
            ConnectionMode::Wired => {
                if is_charging {
                    format!("Charging ({}%)", self.estimated_percentage())
                } else {
                    match self {
                        BatteryStatus::Full => "100% (Pass-Through)".to_string(),
                        BatteryStatus::Medium => "65%".to_string(),
                        _ => format!("{}%", self.estimated_percentage()),
                    }
                }
            }
            ConnectionMode::Wireless24G | ConnectionMode::Bluetooth => {
                match self {
                    BatteryStatus::Full => "100%".to_string(),
                    BatteryStatus::Medium => "65%".to_string(),
                    BatteryStatus::Low => "20% (Low)".to_string(),
                    BatteryStatus::Empty => "<10% (Crit)".to_string(),
                    _ => format!("{}%", self.estimated_percentage()),
                }
            }
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct ControllerButtons {
    pub a: bool,
    pub b: bool,
    pub x: bool,
    pub y: bool,
    pub dpad_up: bool,
    pub dpad_down: bool,
    pub dpad_left: bool,
    pub dpad_right: bool,
    pub lb: bool,
    pub rb: bool,
    pub left_thumb_click: bool,
    pub right_thumb_click: bool,
    pub start: bool,
    pub back: bool,
}

#[derive(Clone, Debug)]
pub struct ControllerState {
    pub user_index: u32,
    pub connected: bool,
    pub is_dongle_present: bool,
    pub is_controller_awake: bool,
    pub last_input_time: Instant,
    pub battery: BatteryStatus,
    pub buttons: ControllerButtons,
    // Normalized sticks: -1.0 to 1.0
    pub left_stick_x: f32,
    pub left_stick_y: f32,
    pub right_stick_x: f32,
    pub right_stick_y: f32,
    // Raw stick magnitude for circularity calculation
    pub left_magnitude: f32,
    pub right_magnitude: f32,
    pub left_circ_error: f32,
    pub right_circ_error: f32,
    // Triggers: 0.0 to 1.0
    pub left_trigger: f32,
    pub right_trigger: f32,
    pub last_battery_check: Instant,
    pub connection_mode: ConnectionMode,
    pub is_charging: bool,
    pub vib_left: u16,
    pub vib_right: u16,
}

impl ControllerState {
    pub fn new(user_index: u32) -> Self {
        Self {
            user_index,
            connected: false,
            is_dongle_present: false,
            is_controller_awake: false,
            last_input_time: Instant::now() - std::time::Duration::from_secs(120),
            battery: BatteryStatus::Unknown,
            buttons: ControllerButtons::default(),
            left_stick_x: 0.0,
            left_stick_y: 0.0,
            right_stick_x: 0.0,
            right_stick_y: 0.0,
            left_magnitude: 0.0,
            right_magnitude: 0.0,
            left_circ_error: 0.0,
            right_circ_error: 0.0,
            left_trigger: 0.0,
            right_trigger: 0.0,
            last_battery_check: Instant::now(),
            connection_mode: ConnectionMode::Disconnected,
            is_charging: false,
            vib_left: 0,
            vib_right: 0,
        }
    }

    #[cfg(windows)]
    pub fn poll(&mut self, config: &AppConfig) {
        unsafe {
            let prev_slot = self.user_index;
            let prev_connected = self.connected;

            // Multi-slot dynamic scan across all 4 XInput slots
            let mut best_slot = self.user_index;
            let mut best_score: u8 = 0; // 0=none, 1=idle receiver (all 0s), 2=live sensor, 3=active input
            let mut chosen_state = XINPUT_STATE::default();
            let mut any_dongle_present = false;

            for slot in 0..4 {
                let mut st = XINPUT_STATE::default();
                if XInputGetState(slot, &mut st) == 0 {
                    let gp = st.Gamepad;
                    let l_thresh = (config.left_deadzone * 32767.0).max(1000.0) as i32;
                    let r_thresh = (config.right_deadzone * 32767.0).max(1000.0) as i32;
                    let trigger_thresh = if config.hair_triggers {
                        (config.hair_trigger_threshold * 255.0).max(12.0) as u8
                    } else {
                        30 // Normal deadzone avoids slight trigger rest activating gaming power draw
                    };
                    let has_input = gp.wButtons.0 != 0
                        || gp.bLeftTrigger > trigger_thresh
                        || gp.bRightTrigger > trigger_thresh
                        || (gp.sThumbLX as i32).abs() > l_thresh
                        || (gp.sThumbLY as i32).abs() > l_thresh
                        || (gp.sThumbRX as i32).abs() > r_thresh
                        || (gp.sThumbRY as i32).abs() > r_thresh;

                    let has_sensor = gp.sThumbLX != 0 || gp.sThumbLY != 0 || gp.sThumbRX != 0 || gp.sThumbRY != 0;

                    let score = if has_input {
                        3
                    } else if has_sensor {
                        2
                    } else {
                        any_dongle_present = true;
                        1 // Receiver dongle present but controller is off / idle
                    };

                    // Priority: active user input (3) > live sensor data (2) > idle dongle (1)
                    if score > best_score || (score == best_score && slot == self.user_index) {
                        best_score = score;
                        best_slot = slot;
                        chosen_state = st;
                    }
                }
            }

            self.user_index = best_slot;
            self.is_dongle_present = any_dongle_present || best_score == 1;

            if best_score >= 1 {
                let gp = chosen_state.Gamepad;

                // Buttons
                let w_buttons = gp.wButtons.0;
                self.buttons.a = (w_buttons & XINPUT_GAMEPAD_A.0) != 0;
                self.buttons.b = (w_buttons & XINPUT_GAMEPAD_B.0) != 0;
                self.buttons.x = (w_buttons & XINPUT_GAMEPAD_X.0) != 0;
                self.buttons.y = (w_buttons & XINPUT_GAMEPAD_Y.0) != 0;
                self.buttons.dpad_up = (w_buttons & XINPUT_GAMEPAD_DPAD_UP.0) != 0;
                self.buttons.dpad_down = (w_buttons & XINPUT_GAMEPAD_DPAD_DOWN.0) != 0;
                self.buttons.dpad_left = (w_buttons & XINPUT_GAMEPAD_DPAD_LEFT.0) != 0;
                self.buttons.dpad_right = (w_buttons & XINPUT_GAMEPAD_DPAD_RIGHT.0) != 0;
                self.buttons.lb = (w_buttons & XINPUT_GAMEPAD_LEFT_SHOULDER.0) != 0;
                self.buttons.rb = (w_buttons & XINPUT_GAMEPAD_RIGHT_SHOULDER.0) != 0;
                self.buttons.left_thumb_click = (w_buttons & XINPUT_GAMEPAD_LEFT_THUMB.0) != 0;
                self.buttons.right_thumb_click = (w_buttons & XINPUT_GAMEPAD_RIGHT_THUMB.0) != 0;
                self.buttons.start = (w_buttons & XINPUT_GAMEPAD_START.0) != 0;
                self.buttons.back = (w_buttons & XINPUT_GAMEPAD_BACK.0) != 0;

                // Process Thumbsticks with Deadzones
                let (lx, ly, lmag) = process_stick(
                    gp.sThumbLX,
                    gp.sThumbLY,
                    config.left_deadzone,
                    config.stick_curve,
                );
                let (rx, ry, rmag) = process_stick(
                    gp.sThumbRX,
                    gp.sThumbRY,
                    config.right_deadzone,
                    config.stick_curve,
                );
                self.left_stick_x = lx;
                self.left_stick_y = ly;
                self.left_magnitude = lmag;
                self.right_stick_x = rx;
                self.right_stick_y = ry;
                self.right_magnitude = rmag;

                // Circularity error computation when sticks are near outer gate
                if lmag > 0.80 {
                    let err = ((lmag - 1.0).abs() * 100.0).min(20.0);
                    self.left_circ_error = self.left_circ_error * 0.95 + err * 0.05;
                }
                if rmag > 0.80 {
                    let err = ((rmag - 1.0).abs() * 100.0).min(20.0);
                    self.right_circ_error = self.right_circ_error * 0.95 + err * 0.05;
                }

                // Process Triggers with Hair-Trigger Option
                let raw_lt = gp.bLeftTrigger as f32 / 255.0;
                let raw_rt = gp.bRightTrigger as f32 / 255.0;

                if config.hair_triggers {
                    self.left_trigger = if raw_lt >= config.hair_trigger_threshold {
                        1.0
                    } else {
                        0.0
                    };
                    self.right_trigger = if raw_rt >= config.hair_trigger_threshold {
                        1.0
                    } else {
                        0.0
                    };
                } else {
                    self.left_trigger = raw_lt;
                    self.right_trigger = raw_rt;
                }

                if best_score == 3 {
                    // Active player input (button pressed, trigger pulled, stick pushed past deadzone)
                    self.last_input_time = Instant::now();
                    self.is_controller_awake = true;
                } else {
                    // Idle on desk (sensor resting center or idle dongle)
                    if self.last_input_time.elapsed() > std::time::Duration::from_millis(800) {
                        self.is_controller_awake = false;
                    }
                }

                // Connection state: If score >= 2, physical controller is active!
                if best_score >= 2 {
                    self.connected = true;
                } else {
                    self.connected = false;
                }

                let slot_changed = prev_slot != self.user_index;
                let conn_changed = prev_connected != self.connected;

                // Check battery and connection mode immediately on slot change, connection change, or every 2 seconds
                if slot_changed
                    || conn_changed
                    || self.battery == BatteryStatus::Unknown
                    || self.last_battery_check.elapsed().as_secs() >= 2
                {
                    self.poll_battery();
                    self.last_battery_check = Instant::now();
                }
            } else {
                self.is_dongle_present = false;
                self.is_controller_awake = false;
                self.connected = false;
                self.battery = BatteryStatus::Disconnected;
                self.connection_mode = ConnectionMode::Disconnected;
                self.is_charging = false;
            }
        }
    }

    #[cfg(windows)]
    pub fn set_vibration(&mut self, left_motor: u16, right_motor: u16) -> bool {
        self.vib_left = left_motor;
        self.vib_right = right_motor;
        let mut vib = XINPUT_VIBRATION {
            wLeftMotorSpeed: left_motor,
            wRightMotorSpeed: right_motor,
        };
        unsafe {
            XInputSetState(self.user_index, &mut vib) == 0
        }
    }

    #[cfg(windows)]
    pub fn stop_vibration(&mut self) {
        let _ = self.set_vibration(0, 0);
    }

    #[cfg(not(windows))]
    pub fn set_vibration(&mut self, _left_motor: u16, _right_motor: u16) -> bool {
        false
    }

    #[cfg(not(windows))]
    pub fn stop_vibration(&mut self) {}

    #[cfg(not(windows))]
    pub fn poll(&mut self, _config: &AppConfig) {
        self.connected = false;
        self.is_dongle_present = false;
        self.is_controller_awake = false;
        self.connection_mode = ConnectionMode::Disconnected;
        self.is_charging = false;
    }

    #[cfg(windows)]
    fn poll_battery(&mut self) {
        if !self.connected && !self.is_dongle_present {
            self.battery = BatteryStatus::Disconnected;
            self.connection_mode = ConnectionMode::Disconnected;
            self.is_charging = false;
            return;
        }

        if self.connected {
            unsafe {
                let mut bat_info = XINPUT_BATTERY_INFORMATION::default();
                let result =
                    XInputGetBatteryInformation(self.user_index, BATTERY_DEVTYPE_GAMEPAD, &mut bat_info);

                if result == 0 {
                    self.battery = match bat_info.BatteryLevel {
                        BATTERY_LEVEL_FULL => BatteryStatus::Full,
                        BATTERY_LEVEL_MEDIUM => BatteryStatus::Medium,
                        BATTERY_LEVEL_LOW => BatteryStatus::Low,
                        BATTERY_LEVEL_EMPTY => BatteryStatus::Empty,
                        _ => BatteryStatus::Full,
                    };
                }
            }
        } else {
            // Dongle present but gamepad sleeping
            self.battery = BatteryStatus::Disconnected;
        }

        // Automatic connection mode detection natively (Zero PowerShell, 0.0% CPU)
        self.connection_mode = detect_hardware_connection_mode(self.connected, self.is_dongle_present, self.user_index);

        // Automatic Charging State Determination:
        if self.connection_mode == ConnectionMode::Wired {
            self.is_charging = self.battery != BatteryStatus::Full;
        } else {
            self.is_charging = false;
        }
    }
}

pub fn detect_hardware_connection_mode(connected: bool, is_dongle: bool, active_slot: u32) -> ConnectionMode {
    if !connected && !is_dongle {
        return ConnectionMode::Disconnected;
    }

    #[cfg(windows)]
    {
        unsafe {
            // 1. Check if Bluetooth is present
            let mut bth_key = windows::Win32::System::Registry::HKEY::default();
            if RegOpenKeyExW(
                HKEY_LOCAL_MACHINE,
                w!("SYSTEM\\CurrentControlSet\\Enum\\BTHENUM"),
                0,
                KEY_READ,
                &mut bth_key,
            ).is_ok() {
                let mut sub_idx = 0;
                let mut name_buf = [0u16; 256];
                while {
                    let mut name_len = name_buf.len() as u32;
                    RegEnumKeyExW(
                        bth_key,
                        sub_idx,
                        windows::core::PWSTR(name_buf.as_mut_ptr()),
                        &mut name_len,
                        None,
                        windows::core::PWSTR::null(),
                        None,
                        None,
                    ).is_ok()
                } {
                    let name = String::from_utf16_lossy(&name_buf[..name_buf.iter().position(|&c| c == 0).unwrap_or(256)]);
                    if (name.contains("DEV_") && name.contains("045E")) || name.to_lowercase().contains("xbox") {
                        let _ = RegCloseKey(bth_key);
                        return ConnectionMode::Bluetooth;
                    }
                    sub_idx += 1;
                }
                let _ = RegCloseKey(bth_key);
            }

            // 2. Check USB devices under VID_045E&PID_028E (Zero subprocesses, microsecond native registry lookup)
            let mut usb_key = windows::Win32::System::Registry::HKEY::default();
            if RegOpenKeyExW(
                HKEY_LOCAL_MACHINE,
                w!("SYSTEM\\CurrentControlSet\\Enum\\USB\\VID_045E&PID_028E"),
                0,
                KEY_READ,
                &mut usb_key,
            ).is_ok() {
                let mut sub_idx = 0;
                let mut name_buf = [0u16; 256];
                let target_suffix = format!("\\{:04}", active_slot);

                while {
                    let mut name_len = name_buf.len() as u32;
                    RegEnumKeyExW(
                        usb_key,
                        sub_idx,
                        windows::core::PWSTR(name_buf.as_mut_ptr()),
                        &mut name_len,
                        None,
                        windows::core::PWSTR::null(),
                        None,
                        None,
                    ).is_ok()
                } {
                    let inst_id = String::from_utf16_lossy(&name_buf[..name_buf.iter().position(|&c| c == 0).unwrap_or(256)]);
                    
                    let mut inst_key = windows::Win32::System::Registry::HKEY::default();
                    let wide_inst: Vec<u16> = inst_id.encode_utf16().chain(std::iter::once(0)).collect();
                    if RegOpenKeyExW(usb_key, windows::core::PCWSTR(wide_inst.as_ptr()), 0, KEY_READ, &mut inst_key).is_ok() {
                        let mut driver_buf = [0u16; 256];
                        let mut size = (driver_buf.len() * 2) as u32;
                        let mut val_type = REG_VALUE_TYPE::default();
                        if RegQueryValueExW(
                            inst_key,
                            w!("Driver"),
                            None,
                            Some(&mut val_type),
                            Some(driver_buf.as_mut_ptr() as *mut u8),
                            Some(&mut size),
                        ).is_ok() {
                            let drv = String::from_utf16_lossy(&driver_buf[..(size as usize / 2).saturating_sub(1)]);
                            if drv.ends_with(&target_suffix) {
                                let _ = RegCloseKey(inst_key);
                                let _ = RegCloseKey(usb_key);
                                if inst_id.contains("5F101F5E") {
                                    return ConnectionMode::Wireless24G;
                                } else {
                                    return ConnectionMode::Wired;
                                }
                            }
                        }
                        let _ = RegCloseKey(inst_key);
                    }
                    sub_idx += 1;
                }
                let _ = RegCloseKey(usb_key);
            }
        }

        if is_dongle {
            ConnectionMode::Wireless24G
        } else {
            ConnectionMode::Wired
        }
    }

    #[cfg(not(windows))]
    {
        ConnectionMode::Wired
    }
}

// Helper to normalize and apply deadzone & curves
fn process_stick(
    raw_x: i16,
    raw_y: i16,
    deadzone: f32,
    curve: crate::config::CurveType,
) -> (f32, f32, f32) {
    let nx = raw_x as f32 / 32767.0;
    let ny = raw_y as f32 / 32767.0;

    let magnitude = (nx * nx + ny * ny).sqrt();

    if magnitude < deadzone {
        (0.0, 0.0, 0.0)
    } else {
        // Rescale magnitude from deadzone..1.0
        let norm_mag = ((magnitude - deadzone) / (1.0 - deadzone)).min(1.0);
        let curved_mag = match curve {
            crate::config::CurveType::Linear => norm_mag,
            crate::config::CurveType::Smooth => norm_mag * norm_mag,
            crate::config::CurveType::Aggressive => norm_mag.sqrt(),
        };

        let factor = curved_mag / magnitude;
        ((nx * factor).clamp(-1.0, 1.0), (ny * factor).clamp(-1.0, 1.0), norm_mag)
    }
}


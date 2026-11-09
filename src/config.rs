use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CurveType {
    Linear,
    Smooth,
    Aggressive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VibrationMode {
    Off,
    Low,
    Medium,
    High,
}

impl VibrationMode {
    pub fn speeds(&self) -> (u16, u16) {
        match self {
            VibrationMode::Off => (0, 0),
            VibrationMode::Low => (20000, 18000),      // ~30% strength (gentle)
            VibrationMode::Medium => (42000, 38000),   // ~65% strength (balanced esports)
            VibrationMode::High => (65535, 65535),     // 100% full rumble
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            VibrationMode::Off => "Off (0%)",
            VibrationMode::Low => "Low (30%)",
            VibrationMode::Medium => "Medium (65%)",
            VibrationMode::High => "High (100%)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RgbMode {
    Rainbow,
    Breathing,
    Static,
    Reactive,
    Off,
}

impl RgbMode {
    pub fn label(&self) -> &'static str {
        match self {
            RgbMode::Rainbow => "Rainbow Wave",
            RgbMode::Breathing => "Breathing Pulse",
            RgbMode::Static => "Static Color",
            RgbMode::Reactive => "Input Reactive",
            RgbMode::Off => "Lights Off",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RgbConfig {
    pub mode: RgbMode,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub brightness: f32, // 0.0 to 1.0
    pub speed: f32,      // 0.5 to 2.0
}

impl Default for RgbConfig {
    fn default() -> Self {
        Self {
            mode: RgbMode::Rainbow,
            red: 0,
            green: 210,
            blue: 255, // Cyan default
            brightness: 0.85,
            speed: 1.0,
        }
    }
}

fn default_vibration_mode() -> VibrationMode {
    VibrationMode::Medium
}

fn default_rgb_config() -> RgbConfig {
    RgbConfig::default()
}

fn default_github_repo() -> String {
    "mahim/x05-controller-suite".to_string()
}

fn default_true() -> bool {
    true
}

fn default_polling_rate() -> u32 {
    1000
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub autostart: bool,
    pub minimize_to_tray: bool,
    pub hair_triggers: bool,
    pub hair_trigger_threshold: f32, // 0.05 = 5% actuation
    pub left_deadzone: f32,          // e.g. 0.08
    pub right_deadzone: f32,         // e.g. 0.08
    pub stick_curve: CurveType,
    pub desktop_mouse_mode: bool,
    pub mouse_sensitivity: f32,
    pub turbo_enabled: bool,
    pub turbo_speed_hz: u32,
    pub low_battery_alert: bool,
    pub active_profile: String,
    #[serde(default = "default_github_repo")]
    pub github_repo: String,
    #[serde(default = "default_true")]
    pub auto_check_updates: bool,
    #[serde(default = "default_polling_rate")]
    pub polling_rate_hz: u32,
    #[serde(default = "default_vibration_mode")]
    pub vibration_mode: VibrationMode,
    #[serde(default = "default_rgb_config")]
    pub rgb: RgbConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            autostart: true,
            minimize_to_tray: true,
            hair_triggers: false,
            hair_trigger_threshold: 0.06,
            left_deadzone: 0.06,
            right_deadzone: 0.06,
            stick_curve: CurveType::Linear,
            desktop_mouse_mode: false,
            mouse_sensitivity: 15.0,
            turbo_enabled: false,
            turbo_speed_hz: 12,
            low_battery_alert: true,
            active_profile: "Balanced Default".to_string(),
            github_repo: default_github_repo(),
            auto_check_updates: true,
            polling_rate_hz: 1000,
            vibration_mode: VibrationMode::Medium,
            rgb: RgbConfig::default(),
        }
    }
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            let mut path = PathBuf::from(appdata);
            path.push("EasySMX_X05_Suite");
            let _ = fs::create_dir_all(&path);
            path.push("config.json");
            path
        } else {
            PathBuf::from("config.json")
        }
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(cfg) = serde_json::from_str(&content) {
                    return cfg;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        fs::write(path, json).map_err(|e| format!("Failed to write config file: {}", e))
    }

    pub fn apply_profile(&mut self, profile_name: &str) {
        self.active_profile = profile_name.to_string();
        if profile_name.contains("FIFA") || profile_name.contains("FC") || profile_name.contains("FOOTBALL") {
            // Competitive FIFA / EA Sports FC Esports Profile
            self.hair_triggers = false;          // Progressive triggers for controlled sprint & jockey pacing
            self.hair_trigger_threshold = 0.08;
            self.left_deadzone = 0.025;          // 2.5% ultra-agile Left Stick Dribbling & rapid directional pivots
            self.right_deadzone = 0.03;          // 3.0% snappy Right Stick skill moves & RS player switching
            self.stick_curve = CurveType::Linear;// 1:1 true direct pass & shot aim
            self.polling_rate_hz = 1000;         // 1000 Hz zero-input-delay for online FUT Champions & Rivals
            self.vibration_mode = VibrationMode::Medium;
            self.desktop_mouse_mode = false;
        } else if profile_name.contains("Zelda") {
            // Optimized for Zelda: smooth camera, generous deadzone for bow aiming
            self.hair_triggers = true;
            self.hair_trigger_threshold = 0.05; // Instant bow firing & shield parry
            self.left_deadzone = 0.05;
            self.right_deadzone = 0.04;
            self.stick_curve = CurveType::Smooth;
            self.desktop_mouse_mode = false;
        } else if profile_name.contains("Shooter") || profile_name.contains("FPS") {
            // Ultra instant triggers, minimal deadzones, aggressive stick response
            self.hair_triggers = true;
            self.hair_trigger_threshold = 0.02;
            self.left_deadzone = 0.03;
            self.right_deadzone = 0.03;
            self.stick_curve = CurveType::Aggressive;
            self.desktop_mouse_mode = false;
        } else if profile_name.contains("Racing") {
            // Linear trigger travel for precise acceleration/braking
            self.hair_triggers = false;
            self.left_deadzone = 0.08;
            self.right_deadzone = 0.08;
            self.stick_curve = CurveType::Linear;
            self.desktop_mouse_mode = false;
        } else {
            // Default Balanced Console
            self.hair_triggers = false;
            self.hair_trigger_threshold = 0.06;
            self.left_deadzone = 0.06;
            self.right_deadzone = 0.06;
            self.stick_curve = CurveType::Linear;
        }
    }

    pub fn deactivate_profile(&mut self) {
        self.active_profile = "Default (No Profile)".to_string();
        self.hair_triggers = false;
        self.hair_trigger_threshold = 0.06;
        self.left_deadzone = 0.06;
        self.right_deadzone = 0.06;
        self.stick_curve = CurveType::Linear;
    }
}


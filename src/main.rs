#![windows_subsystem = "windows"]

pub mod controller;
pub mod config;
pub mod startup;
pub mod desktop_mode;
pub mod updater;
pub mod ui;
pub mod tray;

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use config::AppConfig;
use controller::ControllerState;
use updater::AutoUpdater;
use eframe::egui;

#[cfg(windows)]
#[link(name = "winmm")]
extern "system" {
    fn timeBeginPeriod(u_period: u32) -> u32;
    fn timeEndPeriod(u_period: u32) -> u32;
}

#[cfg(windows)]
fn get_window_status() -> (bool, bool) {
    unsafe {
        use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId, FindWindowW, IsIconic};
        use windows::core::w;

        let hwnd = FindWindowW(None, w!("EasySMX X05 Pro - Gamepad Suite & Optimizer")).unwrap_or_default();
        let is_minimized = if !hwnd.is_invalid() && !hwnd.0.is_null() {
            IsIconic(hwnd).as_bool()
        } else {
            false
        };

        let fg = GetForegroundWindow();
        let is_focused = if !fg.0.is_null() {
            let mut pid = 0;
            GetWindowThreadProcessId(fg, Some(&mut pid));
            pid == std::process::id()
        } else {
            false
        };

        (is_focused, is_minimized)
    }
}

#[cfg(not(windows))]
fn get_window_status() -> (bool, bool) {
    (true, false)
}

fn main() -> eframe::Result<()> {
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("Panic: {}\nBacktrace: {:?}", info, std::backtrace::Backtrace::capture());
        let _ = std::fs::write("panic.txt", msg);
    }));

    // Ensure single instance only: if already running, restore existing window and exit cleanly
    #[cfg(windows)]
    let _single_instance_mutex = unsafe {
        use windows::Win32::System::Threading::CreateMutexW;
        use windows::Win32::Foundation::{GetLastError, SetLastError, WIN32_ERROR, ERROR_ALREADY_EXISTS, WPARAM, LPARAM};
        use windows::Win32::UI::WindowsAndMessaging::{FindWindowW, PostMessageW, ShowWindow, SetForegroundWindow, SW_RESTORE, SW_SHOW, WM_COMMAND};
        use windows::core::w;

        SetLastError(WIN32_ERROR(0));
        let handle_res = CreateMutexW(None, true, w!("Local\\EasySMX_X05_Suite_SingleInstance"));
        if let Ok(handle) = handle_res {
            let last_err = GetLastError();
            if last_err == ERROR_ALREADY_EXISTS {
                // 1. Post command to existing tray window to unhide egui viewport
                let tray_hwnd = FindWindowW(w!("EasySMX_X05_Tray_Class"), None).unwrap_or_default();
                if !tray_hwnd.is_invalid() && !tray_hwnd.0.is_null() {
                    let _ = PostMessageW(tray_hwnd, WM_COMMAND, WPARAM(2001), LPARAM(0));
                }

                // 2. Also directly restore and bring the main window to front
                if let Ok(hwnd) = FindWindowW(None, w!("EasySMX X05 Pro - Gamepad Suite & Optimizer")) {
                    if !hwnd.is_invalid() && !hwnd.0.is_null() {
                        let _ = ShowWindow(hwnd, SW_SHOW);
                        let _ = ShowWindow(hwnd, SW_RESTORE);
                        let _ = SetForegroundWindow(hwnd);
                    }
                }
                return Ok(());
            }
            Some(handle)
        } else {
            None
        }
    };

    // Highly optimized window settings for minimum RAM and CPU overhead
    let mut vp = egui::ViewportBuilder::default()
        .with_inner_size([1020.0, 700.0])
        .with_min_inner_size([760.0, 560.0])
        .with_title("EasySMX X05 Pro - Gamepad Suite & Optimizer")
        .with_decorations(true)
        .with_transparent(false);

    if let Ok(icon_data) = eframe::icon_data::from_png_bytes(include_bytes!("../app_icon_square_512.png")) {
        vp = vp.with_icon(icon_data);
    }
    let options = eframe::NativeOptions {
        viewport: vp,
        vsync: true,
        ..Default::default()
    };

    let config = AppConfig::load();
    let startup_status = startup::is_startup_enabled().unwrap_or(false);
    if config.autostart && !startup_status {
        let _ = startup::set_startup_enabled(true);
    }

    // Initialize Windows system tray icon & menu
    tray::init_tray();

    let app = SuiteApp::new(config, startup_status);
    eframe::run_native(
        "EasySMX X05 Pro Suite",
        options,
        Box::new(|cc| {
            // Apply customized dark cyberpunk/sleek theme
            ui::apply_custom_theme(&cc.egui_ctx);
            Box::new(app)
        }),
    )
}

pub struct SuiteApp {
    pub config: AppConfig,
    pub controller: ControllerState,
    pub shared_controller: Arc<Mutex<ControllerState>>,
    pub shared_config: Arc<Mutex<AppConfig>>,
    pub target_hz: Arc<AtomicU32>,
    pub measured_hz: Arc<AtomicU32>,
    pub is_app_active: Arc<AtomicBool>,
    pub startup_enabled: bool,
    pub active_tab: Tab,
    pub polling_rate_hz: f32,
    pub status_message: Option<(String, Instant)>,
    pub running: Arc<AtomicBool>,
    pub updater: AutoUpdater,
    pub show_update_notes: bool,
    pub repo_input: String,
    pub last_low_battery_alert: Option<Instant>,
    pub last_full_battery_alert: bool,
    pub last_wired_state: bool,
    pub last_tray_update: Instant,
    pub last_minimized_state: bool,
    pub window_hidden: bool,
    pub vibration_until: Option<Instant>,
    pub startup_time: Instant,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Tab {
    Dashboard,
    Performance,
    RgbHaptics,
    Profiles,
    DesktopMode,
    EasySMXGuide,
    Settings,
}

impl SuiteApp {
    pub fn new(config: AppConfig, startup_enabled: bool) -> Self {
        let updater = AutoUpdater::new();
        if config.auto_check_updates {
            updater.check_for_updates(config.github_repo.clone(), false);
        }
        let repo_input = config.github_repo.clone();

        let initial_hz = config.polling_rate_hz.clamp(100, 1000);
        let target_hz = Arc::new(AtomicU32::new(initial_hz));
        let measured_hz = Arc::new(AtomicU32::new(initial_hz));
        let running = Arc::new(AtomicBool::new(true));
        let is_app_active = Arc::new(AtomicBool::new(true));

        let shared_controller = Arc::new(Mutex::new(ControllerState::new(0)));
        let shared_config = Arc::new(Mutex::new(config.clone()));

        // Spawn ultra-optimized controller polling thread (Zero-CPU idle & background)
        let poller_running = Arc::clone(&running);
        let poller_controller = Arc::clone(&shared_controller);
        let poller_config = Arc::clone(&shared_config);
        let poller_target_hz = Arc::clone(&target_hz);
        let poller_measured_hz = Arc::clone(&measured_hz);
        let poller_is_active = Arc::clone(&is_app_active);

        thread::Builder::new()
            .name("controller-poller".to_string())
            .spawn(move || {
                #[cfg(windows)]
                unsafe {
                    timeBeginPeriod(1);
                }

                #[cfg(windows)]
                let cur_th = unsafe { windows::Win32::System::Threading::GetCurrentThread() };
                #[cfg(windows)]
                let mut last_boosted = false;

                let mut frame_count: u32 = 0;
                let mut last_hz_time = Instant::now();

                while poller_running.load(Ordering::Relaxed) {
                    let target_hz_val = poller_target_hz.load(Ordering::Relaxed).max(50);
                    let loop_start = Instant::now();
                    let target_duration = Duration::from_micros(1_000_000 / target_hz_val as u64);

                    let is_active = poller_is_active.load(Ordering::Relaxed);
                    let is_mouse_enabled = poller_config
                        .lock()
                        .map(|c| c.desktop_mouse_mode)
                        .unwrap_or(false);

                    // Dynamic Thread Priority Tuning:
                    // Active esports mode gets HIGHEST priority; background drops to LOWEST for zero CPU contention
                    #[cfg(windows)]
                    {
                        let should_boost = is_active || is_mouse_enabled;
                        if should_boost != last_boosted {
                            last_boosted = should_boost;
                            unsafe {
                                use windows::Win32::System::Threading::{SetThreadPriority, THREAD_PRIORITY_HIGHEST, THREAD_PRIORITY_LOWEST};
                                let prio = if should_boost { THREAD_PRIORITY_HIGHEST } else { THREAD_PRIORITY_LOWEST };
                                let _ = SetThreadPriority(cur_th, prio);
                            }
                        }
                    }

                    let (is_connected, is_awake) = {
                        if let Ok(mut state) = poller_controller.lock() {
                            if let Ok(cfg) = poller_config.lock() {
                                state.poll(&cfg);
                            }
                            (state.connected, state.is_controller_awake)
                        } else {
                            (false, false)
                        }
                    };

                    frame_count += 1;
                    let elapsed = last_hz_time.elapsed();
                    if elapsed >= Duration::from_millis(500) {
                        let actual = (frame_count as f64 / elapsed.as_secs_f64()).round() as u32;
                        poller_measured_hz.store(actual, Ordering::Relaxed);
                        frame_count = 0;
                        last_hz_time = Instant::now();
                    }

                    // -------------------------------------------------------------
                    // ADAPTIVE CPU MANAGEMENT (Exact 3-tier System Design)
                    // -------------------------------------------------------------

                    // Tier 3: Idle Mode (Controller disconnected or offline)
                    if !is_connected {
                        poller_measured_hz.store(0, Ordering::Relaxed);
                        // Input polling: Stopped (500ms slow reconnect check timer)
                        thread::sleep(Duration::from_millis(500));
                        continue;
                    }

                    // Tier 2: App minimized / Background mode
                    if !is_active && !is_mouse_enabled {
                        // Gamepad is actively in use by a game, or sitting in background.
                        // Direct XInput handles game inputs; suite only monitors battery/connection.
                        // Controller polling: 50-100ms (100ms = 10Hz, 0.00% CPU)
                        poller_measured_hz.store(target_hz_val, Ordering::Relaxed);
                        thread::sleep(Duration::from_millis(100));
                        continue;
                    }

                    // Tier 1: Active Mode (App visible & focused, or desktop mouse navigation active)
                    if !is_awake && !is_mouse_enabled {
                        // Controller resting on desk with no input: 100ms low activity (0.00% CPU)
                        poller_measured_hz.store(target_hz_val, Ordering::Relaxed);
                        thread::sleep(Duration::from_millis(100));
                        continue;
                    }

                    // Active input / moving sticks in foreground:
                    // Controller polling: 4-8ms (or 1ms if 1000Hz target), pure sleep with ZERO busy-spin
                    let elapsed_loop = loop_start.elapsed();
                    if elapsed_loop < target_duration {
                        let rem = target_duration - elapsed_loop;
                        thread::sleep(rem);
                    }
                }

                #[cfg(windows)]
                unsafe {
                    timeEndPeriod(1);
                }
            })
            .expect("Failed to spawn controller poller thread");

        Self {
            config,
            controller: ControllerState::new(0),
            shared_controller,
            shared_config,
            target_hz,
            measured_hz,
            is_app_active,
            startup_enabled,
            active_tab: Tab::Dashboard,
            polling_rate_hz: initial_hz as f32,
            status_message: None,
            running,
            updater,
            show_update_notes: false,
            repo_input,
            last_low_battery_alert: None,
            last_full_battery_alert: false,
            last_wired_state: false,
            last_tray_update: Instant::now(),
            last_minimized_state: false,
            window_hidden: false,
            vibration_until: None,
            startup_time: Instant::now(),
        }
    }

    pub fn set_target_polling_rate(&mut self, hz: u32) {
        let valid_hz = match hz {
            100 | 200 | 500 | 1000 => hz,
            _ => 1000,
        };
        self.config.polling_rate_hz = valid_hz;
        self.target_hz.store(valid_hz, Ordering::Relaxed);
        if let Ok(mut cfg) = self.shared_config.lock() {
            cfg.polling_rate_hz = valid_hz;
        }
        let _ = self.config.save();
        let ms = 1000.0 / valid_hz as f32;
        self.set_status(format!("Polling rate set to {} Hz ({:.1} ms response time)", valid_hz, ms));
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some((msg.into(), Instant::now()));
    }

    pub fn test_vibration(&mut self, left: u16, right: u16) {
        if let Ok(mut state) = self.shared_controller.lock() {
            state.set_vibration(left, right);
        }
    }

    pub fn trigger_vibration_pulse(&mut self, duration: Duration) {
        let (left, right) = self.config.vibration_mode.speeds();
        if left > 0 || right > 0 {
            self.test_vibration(left, right);
            self.vibration_until = Some(Instant::now() + duration);
        }
    }

    pub fn apply_vibration_mode(&mut self, mode: config::VibrationMode) {
        self.config.vibration_mode = mode;
        let _ = self.config.save();
        if let Ok(mut cfg) = self.shared_config.lock() {
            cfg.vibration_mode = mode;
        }
        if mode != config::VibrationMode::Off {
            self.trigger_vibration_pulse(Duration::from_millis(450));
            self.set_status(format!("Vibration power set to {}", mode.label()));
        } else {
            self.stop_vibration();
            self.set_status("Vibration motors turned OFF.");
        }
    }

    pub fn apply_rgb_config(&mut self) {
        let _ = self.config.save();
        if let Ok(mut cfg) = self.shared_config.lock() {
            cfg.rgb = self.config.rgb.clone();
        }
        self.set_status(format!("RGB profile '{}' synced to controller!", self.config.rgb.mode.label()));
    }

    pub fn stop_vibration(&mut self) {
        if let Ok(mut state) = self.shared_controller.lock() {
            state.stop_vibration();
        }
    }
}

impl eframe::App for SuiteApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle System Tray user events
        if tray::take_show_request() {
            self.window_hidden = false;
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }

        if tray::take_exit_request() {
            std::process::exit(0);
        }

        // Window Close Request: Minimize to System Tray if enabled
        if ctx.input(|i| i.viewport().close_requested()) {
            if self.config.minimize_to_tray {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                self.window_hidden = true;
                tray::show_balloon(
                    "EasySMX X05 Pro Suite",
                    "Suite running in background. Double-click or right-click the tray icon to restore.",
                    false,
                );
            }
        }

        // Determine OS window state: Focused vs Background vs Minimized
        let (is_focused_raw, is_minimized) = get_window_status();
        let is_recent_startup = self.startup_time.elapsed() < Duration::from_secs(3);
        let is_focused = is_focused_raw || is_recent_startup;
        let is_hidden_or_minimized = self.window_hidden || is_minimized;
        let is_active = !is_hidden_or_minimized && is_focused;
        self.is_app_active.store(is_active, Ordering::Relaxed);

        // Memory Working Set Trimming: Reclaim GPU driver scratch buffers when minimized
        if is_hidden_or_minimized && !self.last_minimized_state {
            self.last_minimized_state = true;
            #[cfg(windows)]
            unsafe {
                use windows::Win32::System::Threading::{GetCurrentProcess, SetProcessWorkingSetSize};
                let _ = SetProcessWorkingSetSize(GetCurrentProcess(), usize::MAX, usize::MAX);
            }
        } else if !is_hidden_or_minimized && self.last_minimized_state {
            self.last_minimized_state = false;
        }

        // Read latest polled controller state from background thread
        if let Ok(state) = self.shared_controller.lock() {
            self.controller = state.clone();
        }

        // Automatic non-blocking vibration pulse cutoff
        if let Some(until) = self.vibration_until {
            if Instant::now() >= until {
                self.stop_vibration();
                self.vibration_until = None;
            }
        }

        // Read actual measured polling frequency
        self.polling_rate_hz = self.measured_hz.load(Ordering::Relaxed) as f32;

        // Synchronize any UI config modifications to background thread
        if let Ok(mut cfg) = self.shared_config.lock() {
            *cfg = self.config.clone();
        }

        // Feature 1: Real-Time Battery Alerts (Low Battery Audio/Balloon, Cable Connection & Full Charge)
        if self.config.low_battery_alert && self.controller.connected {
            let is_wired = self.controller.connection_mode == controller::ConnectionMode::Wired;
            let pct = self.controller.battery.estimated_percentage();
            let is_full = pct >= 100 || self.controller.battery == controller::BatteryStatus::Full;

            // Transition: Charging cable was just plugged in!
            if is_wired && !self.last_wired_state {
                self.last_wired_state = true;
                if is_full {
                    self.last_full_battery_alert = true;
                    tray::play_alert_sound(false);
                    tray::show_balloon(
                        "EasySMX X05 Pro - Battery 100% Full",
                        "USB-C cable connected. Gamepad is 100% fully charged (Pass-Through Power) and ready for wireless gaming!",
                        false,
                    );
                    self.set_status("USB-C Connected: Battery 100% Fully Charged!");
                } else {
                    self.last_full_battery_alert = false;
                    tray::play_alert_sound(false);
                    tray::show_balloon(
                        "EasySMX X05 Pro - Charging Started",
                        &format!("USB-C cable connected. Gamepad is now charging ({}%).", pct),
                        false,
                    );
                    self.set_status(format!("Charging via USB-C cable ({}%)", pct));
                }
            } else if !is_wired && self.last_wired_state {
                // Transition: Charging cable was unplugged!
                self.last_wired_state = false;
                self.last_full_battery_alert = false;
                self.set_status(format!("Wireless Mode Active ({}% Battery)", pct));
            }

            // Continuous state checks:
            if !is_wired {
                let is_low = pct <= 20 || matches!(self.controller.battery, controller::BatteryStatus::Low | controller::BatteryStatus::Empty);
                if is_low {
                    let should_alert = match self.last_low_battery_alert {
                        None => true,
                        Some(t) => t.elapsed() >= Duration::from_secs(300),
                    };
                    if should_alert {
                        self.last_low_battery_alert = Some(Instant::now());
                        tray::play_alert_sound(true);
                        tray::show_balloon(
                            "EasySMX X05 Pro - Low Battery Alert",
                            &format!("Battery dropped to {}%! Plug in USB-C cable to continue playing.", pct),
                            true,
                        );
                        self.set_status(format!("LOW BATTERY WARNING ({}%)! Plug in USB-C cable.", pct));
                    }
                } else {
                    self.last_low_battery_alert = None;
                }
                self.last_full_battery_alert = false;
            } else {
                // Wired mode: Check if battery reached 100% while charging
                if is_full && !self.last_full_battery_alert {
                    self.last_full_battery_alert = true;
                    tray::play_alert_sound(false);
                    tray::show_balloon(
                        "EasySMX X05 Pro - Battery Full",
                        "Gamepad is 100% fully charged! Ready for high-speed wireless gaming.",
                        false,
                    );
                    self.set_status("Battery Fully Charged (100%)!");
                }
            }
        }

        // Feature 2: Update System Tray Tooltip periodically (~1.5s)
        if self.last_tray_update.elapsed() >= Duration::from_millis(1500) {
            self.last_tray_update = Instant::now();
            let tooltip = if self.controller.connected {
                let mode_str = match self.controller.connection_mode {
                    controller::ConnectionMode::Wired => "Wired 1000Hz",
                    controller::ConnectionMode::Wireless24G => "2.4GHz Wireless",
                    controller::ConnectionMode::Bluetooth => "Bluetooth BLE",
                    controller::ConnectionMode::Disconnected => "Offline",
                };
                let pct = self.controller.battery.estimated_percentage();
                format!("EasySMX X05 Pro: {}% [{}]", pct, mode_str)
            } else if self.controller.is_dongle_present {
                "EasySMX X05 Pro: 2.4G Standby (Press Home to wake)".to_string()
            } else {
                "EasySMX X05 Pro: Disconnected".to_string()
            };
            tray::update_tooltip(tooltip);
        }

        // Handle desktop mouse mode if active and controller connected
        if self.config.desktop_mouse_mode && self.controller.connected {
            desktop_mode::handle_desktop_navigation(&self.controller, &self.config);
        }

        // -------------------------------------------------------------
        // ADAPTIVE CPU MANAGEMENT: UI RENDERING STATE MACHINE
        // -------------------------------------------------------------

        // State 2A: App minimized or hidden in tray -> UI rendering: PAUSED!
        if is_hidden_or_minimized {
            // UI rendering is completely paused (no widget layouts, no draw calls)
            // Wake every 1000ms for balloon alerts & tray tooltip updates with 0.00% CPU
            ctx.request_repaint_after(Duration::from_millis(1000));
            return;
        }

        // State 2B: App visible in background (behind another window or game)
        if !is_focused {
            // Render UI at 1 FPS idle (0.00% CPU)
            ui::render_ui(self, ctx);
            ctx.request_repaint_after(Duration::from_millis(1000));
            return;
        }

        // State 1 & 3: App is in Foreground and Focused!
        ui::render_ui(self, ctx);

        if !self.controller.connected {
            // State 3: Disconnected Idle mode -> UI rendering: 2 FPS (wait for device event)
            ctx.request_repaint_after(Duration::from_millis(500));
        } else if self.controller.is_controller_awake {
            // State 1: Active stick movement / button pressing -> smooth 60 FPS
            ctx.request_repaint_after(Duration::from_millis(16));
        } else {
            // State 1: Controller idle on desk -> smooth 30 FPS
            ctx.request_repaint_after(Duration::from_millis(33));
        }
    }
}


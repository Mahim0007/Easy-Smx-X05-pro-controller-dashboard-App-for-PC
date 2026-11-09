use crate::config::{CurveType, VibrationMode};
use crate::controller::{BatteryStatus, ConnectionMode};
use crate::startup;
use crate::updater::{AutoUpdater, UpdateStatus};
use crate::{SuiteApp, Tab};
use eframe::egui::{self, Color32, Pos2, Stroke, Vec2};

#[derive(Clone, Copy)]
pub enum NavIcon {
    Dashboard,
    Performance,
    Vibration,
    Profiles,
    DesktopMouse,
    Settings,
    Guide,
}

pub fn apply_custom_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.window_fill = Color32::from_rgb(11, 14, 20);       // Deep matte obsidian
    visuals.panel_fill = Color32::from_rgb(14, 18, 26);        // Sleek dark graphite
    visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(18, 24, 34);
    visuals.widgets.noninteractive.rounding = egui::Rounding::same(8.0);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(22, 28, 40);
    visuals.widgets.inactive.rounding = egui::Rounding::same(8.0);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(32, 42, 62);
    visuals.widgets.hovered.rounding = egui::Rounding::same(8.0);
    visuals.widgets.active.bg_fill = Color32::from_rgb(0, 180, 240);
    visuals.widgets.active.rounding = egui::Rounding::same(8.0);
    visuals.selection.bg_fill = Color32::from_rgb(0, 150, 230);
    visuals.selection.stroke = Stroke::new(1.0_f32, Color32::from_rgb(0, 210, 255));
    ctx.set_visuals(visuals);

    // Global Font Scaling: Make all text easily readable at a glance!
    let mut style = (*ctx.style()).clone();
    use egui::{FontFamily, FontId, TextStyle};
    style.text_styles = [
        (TextStyle::Heading, FontId::new(20.0, FontFamily::Proportional)),
        (TextStyle::Name("Subheading".into()), FontId::new(16.0, FontFamily::Proportional)),
        (TextStyle::Body, FontId::new(14.0, FontFamily::Proportional)),
        (TextStyle::Monospace, FontId::new(13.0, FontFamily::Monospace)),
        (TextStyle::Button, FontId::new(13.5, FontFamily::Proportional)),
        (TextStyle::Small, FontId::new(12.5, FontFamily::Proportional)),
    ].into();
    ctx.set_style(style);
}

pub fn render_ui(app: &mut SuiteApp, ctx: &egui::Context) {
    let current_time = ctx.input(|i| i.time);

    // ----------------------------------------------------
    // TOP HEADER BAR (Modern Gaming Brand & Status Cluster)
    // ----------------------------------------------------
    egui::TopBottomPanel::top("top_panel")
        .frame(
            egui::Frame::none()
                .fill(Color32::from_rgb(12, 16, 24))
                .inner_margin(egui::Margin::symmetric(16.0, 10.0))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(24, 32, 48))),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Brand Title
                ui.label(
                    egui::RichText::new("EASYSMX")
                        .strong()
                        .size(16.0)
                        .color(Color32::WHITE),
                );
                ui.label(
                    egui::RichText::new("X05 PRO")
                        .strong()
                        .size(16.0)
                        .color(Color32::from_rgb(0, 220, 255)),
                );

                // Ultimate Edition Tag Pill
                let tag_frame = egui::Frame::none()
                    .fill(Color32::from_rgb(16, 28, 44))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(0, 160, 230)))
                    .rounding(egui::Rounding::same(10.0))
                    .inner_margin(egui::Margin::symmetric(8.0, 2.0));
                tag_frame.show(ui, |ui| {
                    ui.label(
                        egui::RichText::new("PRO SUITE")
                            .size(10.0)
                            .strong()
                            .color(Color32::from_rgb(0, 210, 255)),
                    );
                });

                ui.add_space(14.0);

                // Animated Hardware Status Pill (Vector dot + crisp text)
                render_connection_status_pill(app, ui, current_time);

                // Right-aligned Telemetry & Hardware Badges
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let avail_top = ui.available_width();
                    if avail_top > 460.0 {
                        // Version Chip
                        ui.label(
                            egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                                .small()
                                .color(Color32::from_rgb(80, 95, 120)),
                        );
                        ui.add_space(6.0);
                        ui.separator();
                        ui.add_space(6.0);
                    }

                    // Polling Rate Badge Pill
                    let hz_pill = egui::Frame::none()
                        .fill(Color32::from_rgb(14, 26, 40))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(0, 170, 240)))
                        .rounding(egui::Rounding::same(12.0))
                        .inner_margin(egui::Margin::symmetric(8.0, 3.0));
                    hz_pill.show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(format!("{:.0} Hz  (1.0 ms)", app.polling_rate_hz))
                                .monospace()
                                .strong()
                                .size(11.0)
                                .color(Color32::from_rgb(0, 225, 255)),
                        );
                    });

                    ui.add_space(6.0);

                    // Battery Top Status Pill
                    render_battery_top_pill(app, ui, current_time);
                });
            });
        });

    // ----------------------------------------------------
    // LEFT NAVIGATION SIDEBAR (Commercial Gaming Style)
    // ----------------------------------------------------
    egui::SidePanel::left("nav_panel")
        .resizable(false)
        .exact_width(185.0)
        .frame(
            egui::Frame::none()
                .fill(Color32::from_rgb(13, 17, 25))
                .inner_margin(egui::Margin::symmetric(8.0, 10.0))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(24, 32, 48))),
        )
        .show(ctx, |ui| {
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("PERIPHERAL MENU")
                    .size(10.0)
                    .strong()
                    .color(Color32::from_rgb(85, 100, 125)),
            );
            ui.add_space(8.0);

            nav_button(ui, &mut app.active_tab, Tab::Dashboard, NavIcon::Dashboard, "Dashboard");
            nav_button(ui, &mut app.active_tab, Tab::Performance, NavIcon::Performance, "Performance");
            nav_button(ui, &mut app.active_tab, Tab::Vibration, NavIcon::Vibration, "Vibration & Haptics");
            nav_button(ui, &mut app.active_tab, Tab::Profiles, NavIcon::Profiles, "Game Profiles");
            nav_button(ui, &mut app.active_tab, Tab::DesktopMode, NavIcon::DesktopMouse, "Desktop Mouse");
            nav_button(ui, &mut app.active_tab, Tab::Settings, NavIcon::Settings, "Settings & Sys");
            nav_button(ui, &mut app.active_tab, Tab::EasySMXGuide, NavIcon::Guide, "Hardware Guide");

            ui.add_space(16.0);

            // Bottom Device Info Card
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.add_space(4.0);
                egui::Frame::none()
                    .fill(Color32::from_rgb(16, 22, 32))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(28, 38, 56)))
                    .rounding(egui::Rounding::same(8.0))
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new("EasySMX X05 Pro")
                                .strong()
                                .size(12.0)
                                .color(Color32::WHITE),
                        );
                        ui.add_space(2.0);
                        ui.label(
                            egui::RichText::new("[+] Hall-Effect Sensors")
                                .small()
                                .color(Color32::from_rgb(0, 210, 255)),
                        );
                        ui.label(
                            egui::RichText::new("[+] 1000Hz Ultra Polling")
                                .small()
                                .color(Color32::from_rgb(0, 245, 140)),
                        );
                        ui.label(
                            egui::RichText::new("[+] RAM: ~18 MB (Lean)")
                                .small()
                                .color(Color32::from_rgb(85, 100, 120)),
                        );
                    });
            });
        });

    // ----------------------------------------------------
    // MAIN CONTENT PANEL (Responsive & Smooth Scroll)
    // ----------------------------------------------------
    egui::CentralPanel::default()
        .frame(
            egui::Frame::none()
                .fill(Color32::from_rgb(11, 14, 20))
                .inner_margin(16.0),
        )
        .show(ctx, |ui| {
            // Auto-updater notification banner if available
            render_updater_banner(app, ui);

            // Transient status message banner
            if let Some((msg, created)) = &app.status_message {
                if created.elapsed().as_secs() < 3 {
                    egui::Frame::none()
                        .fill(Color32::from_rgb(14, 32, 22))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(0, 220, 110)))
                        .rounding(egui::Rounding::same(8.0))
                        .inner_margin(egui::Margin::symmetric(12.0, 6.0))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(msg)
                                    .color(Color32::from_rgb(0, 255, 140))
                                    .strong(),
                            );
                        });
                    ui.add_space(8.0);
                }
            }

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    match app.active_tab {
                        Tab::Dashboard => render_dashboard(app, ui, current_time),
                        Tab::Performance => render_performance(app, ui),
                        Tab::Vibration => render_vibration(app, ui),
                        Tab::Profiles => render_profiles(app, ui),
                        Tab::DesktopMode => render_desktop_mode(app, ui),
                        Tab::Settings => render_settings(app, ui),
                        Tab::EasySMXGuide => render_hardware_guide(ui),
                    }
                });
        });
}

// ----------------------------------------------------
// VECTOR ICON PAINTER (100% Crisp, ZERO Tofu Boxes)
// ----------------------------------------------------
fn draw_nav_icon(painter: &egui::Painter, center: Pos2, icon: NavIcon, color: Color32) {
    let stroke = Stroke::new(1.4_f32, color);
    match icon {
        NavIcon::Dashboard => {
            // 4 mini square tiles (2x2 grid)
            let s = 4.0_f32;
            let gap = 2.0_f32;
            painter.rect_filled(egui::Rect::from_min_size(Pos2::new(center.x - s - gap / 2.0, center.y - s - gap / 2.0), Vec2::splat(s)), egui::Rounding::same(1.0), color);
            painter.rect_filled(egui::Rect::from_min_size(Pos2::new(center.x + gap / 2.0, center.y - s - gap / 2.0), Vec2::splat(s)), egui::Rounding::same(1.0), color);
            painter.rect_filled(egui::Rect::from_min_size(Pos2::new(center.x - s - gap / 2.0, center.y + gap / 2.0), Vec2::splat(s)), egui::Rounding::same(1.0), color);
            painter.rect_filled(egui::Rect::from_min_size(Pos2::new(center.x + gap / 2.0, center.y + gap / 2.0), Vec2::splat(s)), egui::Rounding::same(1.0), color);
        }
        NavIcon::Performance => {
            // Lightning bolt symbol
            let p1 = Pos2::new(center.x + 1.0, center.y - 6.5);
            let p2 = Pos2::new(center.x - 3.5, center.y);
            let p3 = Pos2::new(center.x - 0.5, center.y);
            let p4 = Pos2::new(center.x - 1.0, center.y + 6.5);
            let p5 = Pos2::new(center.x + 3.5, center.y - 1.0);
            let p6 = Pos2::new(center.x + 0.5, center.y - 1.0);
            painter.line_segment([p1, p2], stroke);
            painter.line_segment([p2, p3], stroke);
            painter.line_segment([p3, p4], stroke);
            painter.line_segment([p4, p5], stroke);
            painter.line_segment([p5, p6], stroke);
            painter.line_segment([p6, p1], stroke);
        }
        NavIcon::Vibration => {
            // Dual vibration / rumble wave arcs
            painter.circle_filled(center, 2.5, color);
            // Left waves
            painter.line_segment([Pos2::new(center.x - 4.5, center.y - 4.0), Pos2::new(center.x - 6.0, center.y)], stroke);
            painter.line_segment([Pos2::new(center.x - 6.0, center.y), Pos2::new(center.x - 4.5, center.y + 4.0)], stroke);
            // Right waves
            painter.line_segment([Pos2::new(center.x + 4.5, center.y - 4.0), Pos2::new(center.x + 6.0, center.y)], stroke);
            painter.line_segment([Pos2::new(center.x + 6.0, center.y), Pos2::new(center.x + 4.5, center.y + 4.0)], stroke);
        }
        NavIcon::Profiles => {
            // Controller D-Pad cross
            let arm = 5.5_f32;
            let thick = 2.0_f32;
            painter.line_segment([Pos2::new(center.x - arm, center.y), Pos2::new(center.x + arm, center.y)], Stroke::new(thick, color));
            painter.line_segment([Pos2::new(center.x, center.y - arm), Pos2::new(center.x, center.y + arm)], Stroke::new(thick, color));
            painter.circle_filled(center, 1.2, Color32::from_rgb(13, 17, 25));
        }
        NavIcon::DesktopMouse => {
            // Sleek Mouse silhouette
            let r = egui::Rect::from_center_size(center, Vec2::new(7.5, 12.0));
            painter.rect_stroke(r, egui::Rounding::same(3.5), stroke);
            painter.line_segment([Pos2::new(center.x, center.y - 6.0), Pos2::new(center.x, center.y - 1.5)], Stroke::new(1.0_f32, color));
        }
        NavIcon::Settings => {
            // Tuning sliders / Equalizer (Modern settings icon)
            let w = 5.5_f32;
            painter.line_segment([Pos2::new(center.x - w, center.y - 3.5), Pos2::new(center.x + w, center.y - 3.5)], Stroke::new(1.2_f32, color));
            painter.circle_filled(Pos2::new(center.x - 1.5, center.y - 3.5), 1.8, color);

            painter.line_segment([Pos2::new(center.x - w, center.y), Pos2::new(center.x + w, center.y)], Stroke::new(1.2_f32, color));
            painter.circle_filled(Pos2::new(center.x + 2.0, center.y), 1.8, color);

            painter.line_segment([Pos2::new(center.x - w, center.y + 3.5), Pos2::new(center.x + w, center.y + 3.5)], Stroke::new(1.2_f32, color));
            painter.circle_filled(Pos2::new(center.x - 2.5, center.y + 3.5), 1.8, color);
        }
        NavIcon::Guide => {
            // Info circle with 'i'
            painter.circle_stroke(center, 6.0, stroke);
            painter.circle_filled(Pos2::new(center.x, center.y - 2.2), 1.0, color);
            painter.line_segment([Pos2::new(center.x, center.y - 0.5), Pos2::new(center.x, center.y + 3.0)], Stroke::new(1.2_f32, color));
        }
    }
}

// ----------------------------------------------------
// HEADER PILL HELPERS (Vector dot + Clean Typography)
// ----------------------------------------------------
fn render_connection_status_pill(app: &SuiteApp, ui: &mut egui::Ui, current_time: f64) {
    if app.controller.connected {
        let (bg, stroke, text_color, dot_color, label) = match app.controller.connection_mode {
            ConnectionMode::Wired => (
                Color32::from_rgb(10, 32, 18),
                Color32::from_rgb(0, 220, 110),
                Color32::from_rgb(0, 245, 130),
                Color32::from_rgb(0, 255, 128),
                "WIRED USB",
            ),
            ConnectionMode::Wireless24G => (
                Color32::from_rgb(10, 28, 44),
                Color32::from_rgb(0, 190, 245),
                Color32::from_rgb(0, 220, 255),
                Color32::from_rgb(0, 220, 255),
                "2.4GHz WIRELESS",
            ),
            ConnectionMode::Bluetooth => (
                Color32::from_rgb(28, 16, 44),
                Color32::from_rgb(170, 110, 245),
                Color32::from_rgb(195, 150, 255),
                Color32::from_rgb(210, 160, 255),
                "BLUETOOTH",
            ),
            ConnectionMode::Disconnected => (
                Color32::from_rgb(34, 14, 18),
                Color32::from_rgb(240, 70, 70),
                Color32::from_rgb(255, 90, 90),
                Color32::from_rgb(255, 80, 80),
                "DISCONNECTED",
            ),
        };

        egui::Frame::none()
            .fill(bg)
            .stroke(Stroke::new(1.0_f32, stroke))
            .rounding(egui::Rounding::same(12.0))
            .inner_margin(egui::Margin::symmetric(10.0, 3.5))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let (dot_rect, _) = ui.allocate_exact_size(Vec2::splat(8.0), egui::Sense::hover());
                    ui.painter().circle_filled(dot_rect.center(), 3.5, dot_color);
                    ui.add_space(2.0);
                    ui.label(
                        egui::RichText::new(label)
                            .strong()
                            .size(10.5)
                            .color(text_color),
                    );
                });
            });
    } else if app.controller.is_dongle_present {
        let pulse = (current_time * 2.8).sin() as f32 * 0.25 + 0.75;
        let amber_color = Color32::from_rgb(
            (255.0 * pulse) as u8,
            (170.0 * pulse) as u8,
            0,
        );

        egui::Frame::none()
            .fill(Color32::from_rgb(32, 22, 8))
            .stroke(Stroke::new(1.0_f32, amber_color))
            .rounding(egui::Rounding::same(12.0))
            .inner_margin(egui::Margin::symmetric(10.0, 3.5))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let (dot_rect, _) = ui.allocate_exact_size(Vec2::splat(8.0), egui::Sense::hover());
                    ui.painter().circle_filled(dot_rect.center(), 3.5, amber_color);
                    ui.add_space(2.0);
                    ui.label(
                        egui::RichText::new("2.4GHz STANDBY")
                            .strong()
                            .size(10.5)
                            .color(Color32::from_rgb(255, 185, 30)),
                    );
                });
            });
    } else {
        egui::Frame::none()
            .fill(Color32::from_rgb(32, 12, 16))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(230, 60, 60)))
            .rounding(egui::Rounding::same(12.0))
            .inner_margin(egui::Margin::symmetric(10.0, 3.5))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let (dot_rect, _) = ui.allocate_exact_size(Vec2::splat(8.0), egui::Sense::hover());
                    ui.painter().circle_filled(dot_rect.center(), 3.5, Color32::from_rgb(255, 80, 80));
                    ui.add_space(2.0);
                    ui.label(
                        egui::RichText::new("DISCONNECTED")
                            .strong()
                            .size(10.5)
                            .color(Color32::from_rgb(255, 90, 90)),
                    );
                });
            });
    }
}

fn render_battery_top_pill(app: &SuiteApp, ui: &mut egui::Ui, current_time: f64) {
    if !app.controller.connected {
        return;
    }

    let is_chg = app.controller.is_charging;
    let mode = app.controller.connection_mode;
    let pct = app.controller.battery.estimated_percentage();

    let (bg, stroke, text_color, label) = if is_chg && mode == ConnectionMode::Wired {
        let pulse = (current_time * 3.2).sin() as f32 * 0.25 + 0.75;
        let cyan_stroke = Color32::from_rgb(0, (220.0 * pulse) as u8, (255.0 * pulse) as u8);
        (
            Color32::from_rgb(8, 28, 36),
            cyan_stroke,
            Color32::from_rgb(0, 235, 255),
            format!("Charging ({}%)", pct),
        )
    } else {
        match app.controller.battery {
            BatteryStatus::Full => (
                Color32::from_rgb(10, 32, 18),
                Color32::from_rgb(0, 220, 110),
                Color32::from_rgb(0, 245, 130),
                "100% Full".to_string(),
            ),
            BatteryStatus::Medium => (
                Color32::from_rgb(32, 28, 8),
                Color32::from_rgb(240, 200, 0),
                Color32::from_rgb(255, 215, 30),
                "~65% Power".to_string(),
            ),
            BatteryStatus::Low | BatteryStatus::Empty => (
                Color32::from_rgb(34, 14, 18),
                Color32::from_rgb(240, 70, 70),
                Color32::from_rgb(255, 90, 90),
                format!("{}% Low Battery", pct),
            ),
            _ => (
                Color32::from_rgb(18, 22, 30),
                Color32::from_rgb(55, 70, 90),
                Color32::GRAY,
                "100%".to_string(),
            ),
        }
    };

    egui::Frame::none()
        .fill(bg)
        .stroke(Stroke::new(1.0_f32, stroke))
        .rounding(egui::Rounding::same(12.0))
        .inner_margin(egui::Margin::symmetric(10.0, 3.5))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(label)
                    .strong()
                    .size(11.0)
                    .color(text_color),
            );
        });
}

// ----------------------------------------------------
// SIDEBAR NAVIGATION WITH 100% ISOLATED HOVER ANIMATION
// ----------------------------------------------------
fn nav_button(ui: &mut egui::Ui, current_tab: &mut Tab, target_tab: Tab, icon: NavIcon, label: &str) {
    let is_selected = *current_tab == target_tab;
    let btn_id = ui.id().with(label);

    let desired_size = Vec2::new(ui.available_width(), 36.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    if response.clicked() {
        *current_tab = target_tab;
    }

    // Isolated Hover: EXACTLY true only for the button hovered!
    let is_hovered = response.hovered();
    if is_hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let hover_t = ui.ctx().animate_bool(btn_id.with("nav_hover"), is_hovered);

    let painter = ui.painter();

    if is_selected {
        // Active selected button: Solid navy obsidian fill + Cyan border + Left accent bar
        painter.rect_filled(rect, egui::Rounding::same(8.0), Color32::from_rgb(18, 30, 48));
        painter.rect_stroke(rect, egui::Rounding::same(8.0), Stroke::new(1.4_f32, Color32::from_rgb(0, 190, 245)));
        painter.rect_filled(
            egui::Rect::from_min_size(Pos2::new(rect.min.x + 2.0, rect.min.y + 8.0), Vec2::new(3.5, 20.0)),
            egui::Rounding::same(1.5),
            Color32::from_rgb(0, 220, 255),
        );
    } else if hover_t > 0.01 {
        // Hovered button: Animated smooth fade-in of background and cyan border
        let bg = Color32::from_rgba_unmultiplied(26, 36, 54, (hover_t * 180.0) as u8);
        let border = Color32::from_rgba_unmultiplied(0, 190, 245, (hover_t * 200.0) as u8);
        painter.rect_filled(rect, egui::Rounding::same(8.0), bg);
        painter.rect_stroke(rect, egui::Rounding::same(8.0), Stroke::new(1.0_f32, border));
    }
    // Unhovered and unselected: Purely transparent, NO border, NO background!

    let text_color = if is_selected {
        Color32::WHITE
    } else {
        Color32::from_rgb(
            (145.0 + hover_t * 85.0) as u8,
            (160.0 + hover_t * 85.0) as u8,
            (180.0 + hover_t * 75.0) as u8,
        )
    };

    let icon_color = if is_selected {
        Color32::from_rgb(0, 220, 255)
    } else {
        Color32::from_rgb(
            (110.0 + hover_t * 110.0) as u8,
            (130.0 + hover_t * 110.0) as u8,
            (155.0 + hover_t * 90.0) as u8,
        )
    };

    // Draw vector icon
    let icon_center = Pos2::new(rect.min.x + 18.0, rect.center().y);
    draw_nav_icon(painter, icon_center, icon, icon_color);

    // Draw text label
    painter.text(
        Pos2::new(rect.min.x + 36.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(13.0),
        text_color,
    );

    ui.add_space(2.0);
}

// ----------------------------------------------------
// DASHBOARD (Elite Commercial Peripheral Suite Look)
// ----------------------------------------------------
fn render_dashboard(app: &mut SuiteApp, ui: &mut egui::Ui, current_time: f64) {
    let radar_color = Color32::from_rgb(0, 210, 255);
    // TOP HERO METRIC CARDS (Using 3 isolated columns)
    ui.columns(3, |cols| {
        // Column 0: Connection Protocol & Link
        render_hero_card(&mut cols[0], "LINK & CONNECTION", |ui| {
            if app.controller.connected {
                let (title, sub, color) = match app.controller.connection_mode {
                    ConnectionMode::Wired => ("Wired USB-C", "Ultra-Fast (0ms Latency)", Color32::from_rgb(0, 240, 140)),
                    ConnectionMode::Wireless24G => ("2.4GHz Wireless", "High-Speed Dongle Link", Color32::from_rgb(0, 220, 255)),
                    ConnectionMode::Bluetooth => ("Bluetooth BLE", "Low-Power Link", Color32::from_rgb(190, 130, 255)),
                    ConnectionMode::Disconnected => ("Disconnected", "Connect to play", Color32::RED),
                };
                ui.label(egui::RichText::new(title).strong().size(13.5).color(color));
                ui.label(egui::RichText::new(sub).small().color(Color32::from_rgb(145, 160, 180)));
            } else if app.controller.is_dongle_present {
                ui.label(egui::RichText::new("2.4GHz Dongle").strong().size(13.5).color(Color32::from_rgb(255, 185, 30)));
                ui.label(egui::RichText::new("Standby (Press Home)").small().color(Color32::from_rgb(150, 165, 185)));
            } else {
                ui.label(egui::RichText::new("Disconnected").strong().size(13.5).color(Color32::from_rgb(240, 70, 70)));
                ui.label(egui::RichText::new("Connect Gamepad").small().color(Color32::GRAY));
            }
        });

        // Column 1: Smart Battery & Power Telemetry
        render_hero_card(&mut cols[1], "BATTERY & POWER", |ui| {
            if app.controller.connected {
                let is_chg = app.controller.is_charging;
                let mode = app.controller.connection_mode;
                let pct = app.controller.battery.estimated_percentage();

                let (title, sub, color) = if is_chg && mode == ConnectionMode::Wired {
                    (format!("Charging ({}%)", pct), "USB Cable: Charging Li-Po", Color32::from_rgb(0, 230, 255))
                } else if mode == ConnectionMode::Wired {
                    ("100% (Pass-Through)".to_string(), "Full: Direct Bus Power", Color32::from_rgb(0, 240, 140))
                } else {
                    match app.controller.battery {
                        BatteryStatus::Full => ("100% (Full Power)".to_string(), "Internal 750mAh Battery", Color32::from_rgb(0, 240, 140)),
                        BatteryStatus::Medium => ("~65% (Medium)".to_string(), "Normal Battery Power", Color32::from_rgb(255, 215, 0)),
                        BatteryStatus::Low | BatteryStatus::Empty => (format!("{}% (Low Battery)", pct), "Connect USB Cable", Color32::from_rgb(255, 80, 80)),
                        _ => ("100% (Good)".to_string(), "Wireless Battery Power", Color32::GRAY),
                    }
                };

                ui.label(egui::RichText::new(title).strong().size(13.5).color(color));
                ui.label(egui::RichText::new(sub).small().color(Color32::from_rgb(145, 160, 180)));
            } else if app.controller.is_dongle_present {
                ui.label(egui::RichText::new("Standby Mode").strong().size(13.5).color(Color32::from_rgb(175, 185, 200)));
                ui.label(egui::RichText::new("Gamepad powered off").small().color(Color32::GRAY));
            } else {
                ui.label(egui::RichText::new("No Device").strong().size(13.5).color(Color32::GRAY));
                ui.label(egui::RichText::new("Offline").small().color(Color32::GRAY));
            }
        });

        // Column 2: Polling Engine & Response Latency
        render_hero_card(&mut cols[2], "POLLING & ENGINE", |ui| {
            ui.label(
                egui::RichText::new(format!("{:.0} Hz Esports", app.polling_rate_hz))
                    .strong()
                    .size(13.5)
                    .color(Color32::from_rgb(0, 220, 255)),
            );
            let latency_ms = 1000.0 / app.config.polling_rate_hz as f32;
            ui.label(
                egui::RichText::new(format!("{:.1} ms | {}", latency_ms, app.config.active_profile))
                    .small()
                    .color(Color32::from_rgb(145, 160, 180)),
            );
        });
    });

    ui.add_space(12.0);

    // ----------------------------------------------------
    // MAIN VISUALIZER STUDIO: RESPONSIVE TWO-CARD LAYOUT
    // ----------------------------------------------------
    let avail_w = ui.available_width();

    let render_left_card = |ui: &mut egui::Ui| {
        egui::Frame::none()
            .fill(Color32::from_rgb(14, 18, 27))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(26, 35, 52)))
            .rounding(egui::Rounding::same(10.0))
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());

                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Hall-Effect Radar")
                            .strong()
                            .size(13.5)
                            .color(Color32::WHITE),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let badge = egui::Frame::none()
                            .fill(Color32::from_rgb(16, 28, 44))
                            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(0, 160, 230)))
                            .rounding(egui::Rounding::same(8.0))
                            .inner_margin(egui::Margin::symmetric(6.0, 2.0));
                        badge.show(ui, |ui| {
                            ui.label(
                                egui::RichText::new("Anti-Drift")
                                    .size(9.5)
                                    .strong()
                                    .color(Color32::from_rgb(0, 210, 255)),
                            );
                        });
                    });
                });
                ui.add_space(6.0);

                // Dual stick sub-columns (Each strictly gets 50% width)
                ui.columns(2, |stick_cols| {
                    // Left Stick (LS)
                    stick_cols[0].vertical_centered(|ui| {
                        ui.label(egui::RichText::new("Left Stick (LS)").strong().size(11.0).color(Color32::from_rgb(190, 205, 225)));
                        ui.add_space(2.0);
                        draw_stick_radar(ui, app.controller.left_stick_x, app.controller.left_stick_y, app.config.left_deadzone, current_time, radar_color);
                        ui.add_space(3.0);
                        ui.label(
                            egui::RichText::new(format!("X:{:+.2} Y:{:+.2}", app.controller.left_stick_x, app.controller.left_stick_y))
                                .monospace()
                                .strong()
                                .size(10.0)
                                .color(Color32::from_rgb(0, 220, 255)),
                        );
                        ui.label(
                            egui::RichText::new(format!("Circ: {:.1}%", app.controller.left_circ_error))
                                .small()
                                .color(Color32::from_rgb(0, 245, 140)),
                        );
                    });

                    // Right Stick (RS)
                    stick_cols[1].vertical_centered(|ui| {
                        ui.label(egui::RichText::new("Right Stick (RS)").strong().size(11.0).color(Color32::from_rgb(190, 205, 225)));
                        ui.add_space(2.0);
                        draw_stick_radar(ui, app.controller.right_stick_x, app.controller.right_stick_y, app.config.right_deadzone, current_time, radar_color);
                        ui.add_space(3.0);
                        ui.label(
                            egui::RichText::new(format!("X:{:+.2} Y:{:+.2}", app.controller.right_stick_x, app.controller.right_stick_y))
                                .monospace()
                                .strong()
                                .size(10.0)
                                .color(Color32::from_rgb(0, 220, 255)),
                        );
                        ui.label(
                            egui::RichText::new(format!("Circ: {:.1}%", app.controller.right_circ_error))
                                .small()
                                .color(Color32::from_rgb(0, 245, 140)),
                        );
                    });
                });
            });
    };

    let render_right_card = |ui: &mut egui::Ui| {
        egui::Frame::none()
            .fill(Color32::from_rgb(14, 18, 27))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(26, 35, 52)))
            .rounding(egui::Rounding::same(10.0))
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());

                ui.label(
                    egui::RichText::new("Analog Triggers & Action Matrix")
                        .strong()
                        .size(13.5)
                        .color(Color32::WHITE),
                );
                ui.add_space(6.0);

                // Triggers (LT & RT in 2 isolated columns with zero overflow)
                ui.columns(2, |trig_cols| {
                    trig_cols[0].vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("LT").strong().size(11.0).color(Color32::from_rgb(0, 220, 255)));
                            let lt_pct = (app.controller.left_trigger * 100.0) as u32;
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(egui::RichText::new(format!("{}%", lt_pct)).monospace().strong().size(10.5).color(Color32::WHITE));
                            });
                        });
                        let lt_bar = egui::ProgressBar::new(app.controller.left_trigger)
                            .fill(Color32::from_rgb(0, 210, 255));
                        ui.add_sized([ui.available_width(), 13.0], lt_bar);
                    });

                    trig_cols[1].vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("RT").strong().size(11.0).color(Color32::from_rgb(0, 220, 255)));
                            let rt_pct = (app.controller.right_trigger * 100.0) as u32;
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(egui::RichText::new(format!("{}%", rt_pct)).monospace().strong().size(10.5).color(Color32::WHITE));
                            });
                        });
                        let rt_bar = egui::ProgressBar::new(app.controller.right_trigger)
                            .fill(Color32::from_rgb(0, 210, 255));
                        ui.add_sized([ui.available_width(), 13.0], rt_bar);
                    });
                });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(6.0);

                let btn = &app.controller.buttons;

                // Shoulder & Thumbstick Click Pairs (50%/50% width per row)
                two_btn_row(ui, ("LB", btn.lb, Color32::from_rgb(0, 215, 255)), ("RB", btn.rb, Color32::from_rgb(0, 215, 255)));
                ui.add_space(4.0);
                two_btn_row(ui, ("L3 (LS)", btn.left_thumb_click, Color32::from_rgb(0, 195, 235)), ("R3 (RS)", btn.right_thumb_click, Color32::from_rgb(0, 195, 235)));
                ui.add_space(4.0);
                two_btn_row(ui, ("BACK", btn.back, Color32::from_rgb(175, 190, 215)), ("START", btn.start, Color32::from_rgb(175, 190, 215)));

                ui.add_space(8.0);

                // D-Pad and ABXY Diamond clusters (Authentic Controller Layout)
                ui.columns(2, |btn_cols| {
                    // Left: D-Pad
                    btn_cols[0].vertical_centered(|ui| {
                        ui.label(egui::RichText::new("D-PAD").small().strong().color(Color32::from_rgb(110, 130, 160)));
                        ui.add_space(2.0);
                        compact_btn(ui, "Up", btn.dpad_up, Color32::WHITE, Vec2::new(38.0, 22.0));
                        ui.add_space(2.0);
                        ui.horizontal(|ui| {
                            let pad = ((ui.available_width() - 80.0) / 2.0).max(0.0);
                            ui.add_space(pad);
                            compact_btn(ui, "Left", btn.dpad_left, Color32::WHITE, Vec2::new(38.0, 22.0));
                            ui.add_space(4.0);
                            compact_btn(ui, "Right", btn.dpad_right, Color32::WHITE, Vec2::new(38.0, 22.0));
                        });
                        ui.add_space(2.0);
                        compact_btn(ui, "Down", btn.dpad_down, Color32::WHITE, Vec2::new(38.0, 22.0));
                    });

                    // Right: ABXY Diamond
                    btn_cols[1].vertical_centered(|ui| {
                        ui.label(egui::RichText::new("ACTION").small().strong().color(Color32::from_rgb(110, 130, 160)));
                        ui.add_space(2.0);
                        compact_btn(ui, "Y", btn.y, Color32::from_rgb(255, 200, 0), Vec2::new(38.0, 22.0));
                        ui.add_space(2.0);
                        ui.horizontal(|ui| {
                            let pad = ((ui.available_width() - 80.0) / 2.0).max(0.0);
                            ui.add_space(pad);
                            compact_btn(ui, "X", btn.x, Color32::from_rgb(0, 180, 255), Vec2::new(38.0, 22.0));
                            ui.add_space(4.0);
                            compact_btn(ui, "B", btn.b, Color32::from_rgb(255, 60, 60), Vec2::new(38.0, 22.0));
                        });
                        ui.add_space(2.0);
                        compact_btn(ui, "A", btn.a, Color32::from_rgb(0, 230, 118), Vec2::new(38.0, 22.0));
                    });
                });
            });
    };

    if avail_w >= 540.0 {
        ui.columns(2, |cols| {
            cols[0].vertical(|ui| { render_left_card(ui); });
            cols[1].vertical(|ui| { render_right_card(ui); });
        });
    } else {
        render_left_card(ui);
        ui.add_space(10.0);
        render_right_card(ui);
    }
}

// ----------------------------------------------------
// HERO CARD HELPER
// ----------------------------------------------------
fn render_hero_card(ui: &mut egui::Ui, header: &str, content: impl FnOnce(&mut egui::Ui)) {
    let card_width = ui.available_width();

    egui::Frame::none()
        .fill(Color32::from_rgb(14, 18, 27))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(26, 35, 52)))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.set_width(card_width);
            ui.label(
                egui::RichText::new(header)
                    .size(10.0)
                    .strong()
                    .color(Color32::from_rgb(95, 115, 145)),
            );
            ui.add_space(4.0);
            content(ui);
        });
}

// ----------------------------------------------------
// PERFORMANCE & TUNING TAB (Razer / Flydigi Grade)
// ----------------------------------------------------
fn render_performance(app: &mut SuiteApp, ui: &mut egui::Ui) {
    ui.label(
        egui::RichText::new("Performance & Hardware Tuning")
            .strong()
            .size(20.0)
            .color(Color32::WHITE),
    );
    ui.add_space(2.0);
    ui.label(
        egui::RichText::new("Fine-tune polling frequency, stick response curves, micro-deadzones, digital hair-triggers, and haptics.")
            .size(13.0)
            .color(Color32::from_rgb(175, 190, 215)),
    );
    ui.add_space(12.0);

    // 1. Polling Rate Overclock Selector
    egui::Frame::none()
        .fill(Color32::from_rgb(14, 18, 27))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(26, 35, 52)))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(14.0)
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new("Controller Polling Rate (Input Overclock)")
                    .strong()
                    .size(15.5)
                    .color(Color32::from_rgb(0, 220, 255)),
            );
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new("Select raw USB polling rate. 1000 Hz provides sub-1ms response time for competitive esports.")
                    .size(13.0)
                    .color(Color32::from_rgb(180, 195, 220)),
            );
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                let options: [(u32, &str, &str); 4] = [
                    (100, "100 Hz", "10.0 ms Casual"),
                    (200, "200 Hz", "5.0 ms Standard"),
                    (500, "500 Hz", "2.0 ms Pro"),
                    (1000, "1000 Hz", "1.0 ms Esports"),
                ];

                for (hz, label, latency) in options {
                    let is_active = app.config.polling_rate_hz == hz;
                    let (bg, border, text_color) = if is_active {
                        (Color32::from_rgb(0, 210, 255), Color32::from_rgb(0, 230, 255), Color32::BLACK)
                    } else {
                        (Color32::from_rgb(20, 26, 38), Color32::from_rgb(32, 42, 62), Color32::WHITE)
                    };

                    let btn = egui::Button::new(
                        egui::RichText::new(format!("{}\n{}", label, latency))
                            .strong()
                            .size(13.0)
                            .color(text_color),
                    )
                    .fill(bg)
                    .stroke(Stroke::new(1.2_f32, border))
                    .rounding(egui::Rounding::same(8.0))
                    .min_size(Vec2::new(125.0, 46.0));

                    if ui.add(btn).clicked() {
                        app.set_target_polling_rate(hz);
                    }
                    ui.add_space(6.0);
                }
            });

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Target Frequency:").size(13.0).color(Color32::from_rgb(180, 195, 220)));
                ui.label(egui::RichText::new(format!("{} Hz", app.config.polling_rate_hz)).strong().size(13.5).color(Color32::from_rgb(0, 220, 255)));
                ui.separator();
                ui.label(egui::RichText::new("Live Measured Speed:").size(13.0).color(Color32::from_rgb(180, 195, 220)));
                ui.label(egui::RichText::new(format!("{:.0} Hz", app.polling_rate_hz)).strong().size(13.5).color(Color32::from_rgb(0, 255, 140)));
                ui.separator();
                let latency_ms = 1000.0 / app.config.polling_rate_hz as f32;
                ui.label(egui::RichText::new("Input Latency:").size(13.0).color(Color32::from_rgb(180, 195, 220)));
                ui.label(egui::RichText::new(format!("{:.1} ms", latency_ms)).strong().size(13.5).color(Color32::from_rgb(0, 255, 180)));
            });
        });

    ui.add_space(10.0);

    // 2. Digital Hair-Triggers
    egui::Frame::none()
        .fill(Color32::from_rgb(14, 18, 27))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(26, 35, 52)))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(14.0)
        .show(ui, |ui| {
            ui.checkbox(&mut app.config.hair_triggers, egui::RichText::new("Enable Digital Hair-Triggers").strong().size(15.0));
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new("Instant micro-switch actuation: Shoots or parries the exact millisecond you touch the trigger.")
                    .size(13.0)
                    .color(Color32::from_rgb(180, 195, 220)),
            );

            if app.config.hair_triggers {
                ui.add_space(8.0);
                ui.add(
                    egui::Slider::new(&mut app.config.hair_trigger_threshold, 0.01..=0.30)
                        .text("Trigger Actuation Point")
                        .custom_formatter(|n, _| format!("{:.0}% Tap Actuation", n * 100.0)),
                );
            }
        });

    ui.add_space(10.0);

    // 3. Thumbstick Deadzones & Response Curve
    egui::Frame::none()
        .fill(Color32::from_rgb(14, 18, 27))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(26, 35, 52)))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(14.0)
        .show(ui, |ui| {
            ui.label(egui::RichText::new("Thumbstick Deadzone & Response Curve").strong().size(15.5).color(Color32::from_rgb(0, 220, 255)));
            ui.add_space(8.0);

            ui.add(
                egui::Slider::new(&mut app.config.left_deadzone, 0.01..=0.25)
                    .text("Left Stick Deadzone")
                    .custom_formatter(|n, _| format!("{:.1}%", n * 100.0)),
            );
            ui.add(
                egui::Slider::new(&mut app.config.right_deadzone, 0.01..=0.25)
                    .text("Right Stick Deadzone")
                    .custom_formatter(|n, _| format!("{:.1}%", n * 100.0)),
            );

            ui.add_space(10.0);
            ui.label(egui::RichText::new("Stick Response Curve Algorithm").strong().size(14.0).color(Color32::WHITE));
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.radio_value(&mut app.config.stick_curve, CurveType::Linear, "Linear (Standard 1:1)");
                ui.radio_value(&mut app.config.stick_curve, CurveType::Smooth, "Smooth S-Curve (RPG / Zelda)");
                ui.radio_value(&mut app.config.stick_curve, CurveType::Aggressive, "Aggressive (Competitive FPS)");
            });
        });

    ui.add_space(10.0);

    // 4. Dual-Motor Haptic Vibration Lab
    egui::Frame::none()
        .fill(Color32::from_rgb(14, 18, 27))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(26, 35, 52)))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(14.0)
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new("Dual-Motor Haptic Vibration Lab")
                    .strong()
                    .size(15.5)
                    .color(Color32::from_rgb(0, 220, 255)),
            );
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new("Calibrate and test both asymmetrical rumble motors (Left: Heavy Low-Frequency / Right: Sharp High-Frequency).")
                    .size(13.0)
                    .color(Color32::from_rgb(180, 195, 220)),
            );
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if ui.button(egui::RichText::new("Left Heavy Motor (Low-Freq)").strong().size(13.0)).clicked() {
                    app.test_vibration(45000, 0);
                    app.set_status("Pulsing Left Heavy Motor...");
                }
                if ui.button(egui::RichText::new("Right Sharp Motor (High-Freq)").strong().size(13.0)).clicked() {
                    app.test_vibration(0, 45000);
                    app.set_status("Pulsing Right Sharp Motor...");
                }
                if ui.button(egui::RichText::new("Full Dual Burst (100%)").strong().size(13.0).color(Color32::from_rgb(0, 230, 255))).clicked() {
                    app.test_vibration(65535, 65535);
                    app.set_status("Dual Rumble Motors Activated!");
                }
                if ui.button(egui::RichText::new("Stop Rumble Motors").strong().size(13.0).color(Color32::from_rgb(255, 90, 90))).clicked() {
                    app.stop_vibration();
                    app.set_status("Vibration Stopped.");
                }
            });
        });

    ui.add_space(12.0);
    let save_text = egui::RichText::new("Save Tuning Profile to Disk").strong().size(14.0).color(Color32::BLACK);
    if ui.add(egui::Button::new(save_text).fill(Color32::from_rgb(0, 230, 140)).min_size(Vec2::new(200.0, 36.0))).clicked() {
        let _ = app.config.save();
        app.set_status("Performance settings saved successfully!");
    }
}

// ----------------------------------------------------
// DUAL-MOTOR VIBRATION & HAPTICS TAB
// ----------------------------------------------------
fn render_vibration(app: &mut SuiteApp, ui: &mut egui::Ui) {
    let vib_color = match app.config.vibration_mode {
        VibrationMode::Off => Color32::GRAY,
        VibrationMode::Low => Color32::from_rgb(0, 245, 140),
        VibrationMode::Medium => Color32::from_rgb(0, 210, 255),
        VibrationMode::High => Color32::from_rgb(220, 100, 255),
    };

    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("Dual-Motor Vibration & Haptics")
                .strong()
                .size(20.0)
                .color(Color32::WHITE),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let badge = egui::Frame::none()
                .fill(Color32::from_rgb(16, 28, 44))
                .stroke(Stroke::new(1.0_f32, vib_color))
                .rounding(egui::Rounding::same(8.0))
                .inner_margin(egui::Margin::symmetric(10.0, 3.0));
            badge.show(ui, |ui| {
                ui.label(
                    egui::RichText::new(format!("[*] {}", app.config.vibration_mode.label()))
                        .size(11.5)
                        .strong()
                        .color(vib_color),
                );
            });
        });
    });
    ui.add_space(2.0);
    ui.label(
        egui::RichText::new("Select vibration force profiles or test physical haptic pulses on EasySMX X05 Pro.")
            .size(13.0)
            .color(Color32::from_rgb(175, 190, 215)),
    );
    ui.add_space(14.0);

    // ----------------------------------------------------
    // SECTION 1: 3-TIER VIBRATION ENGINE (Low, Medium, High + Off)
    // ----------------------------------------------------
    egui::Frame::none()
        .fill(Color32::from_rgb(14, 18, 27))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(26, 35, 52)))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(14.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("3-Tier Vibration Force Engine")
                        .strong()
                        .size(15.0)
                        .color(Color32::from_rgb(0, 220, 255)),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format!("Current: {}", app.config.vibration_mode.label()))
                            .strong()
                            .size(12.0)
                            .color(vib_color),
                    );
                });
            });
            ui.add_space(3.0);
            ui.label(
                egui::RichText::new("Choose your desired rumble feedback intensity across all connected games.")
                    .size(13.0)
                    .color(Color32::from_rgb(175, 190, 215)),
            );
            ui.add_space(12.0);

            // 3-Tier Mode Cards (Low, Medium, High + Off)
            ui.columns(4, |cols| {
                // Tier 1: LOW
                cols[0].vertical_centered(|ui| {
                    let is_active = app.config.vibration_mode == VibrationMode::Low;
                    let fill = if is_active { Color32::from_rgb(16, 36, 26) } else { Color32::from_rgb(16, 22, 32) };
                    let stroke = if is_active { Color32::from_rgb(0, 245, 140) } else { Color32::from_rgb(32, 44, 60) };

                    egui::Frame::none()
                        .fill(fill)
                        .stroke(Stroke::new(if is_active { 1.6_f32 } else { 1.0_f32 }, stroke))
                        .rounding(egui::Rounding::same(8.0))
                        .inner_margin(10.0)
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new("LOW").strong().size(14.0).color(Color32::from_rgb(0, 245, 140)));
                            ui.label(egui::RichText::new("30% Strength").size(11.0).color(Color32::WHITE));
                            ui.label(egui::RichText::new("Gentle & Battery Friendly").small().color(Color32::from_rgb(140, 160, 185)));
                            ui.add_space(6.0);
                            let btn_text = if is_active { "[Active]" } else { "Select Low" };
                            if ui.button(egui::RichText::new(btn_text).strong().size(12.0)).clicked() {
                                app.apply_vibration_mode(VibrationMode::Low);
                            }
                        });
                });

                // Tier 2: MEDIUM
                cols[1].vertical_centered(|ui| {
                    let is_active = app.config.vibration_mode == VibrationMode::Medium;
                    let fill = if is_active { Color32::from_rgb(16, 32, 48) } else { Color32::from_rgb(16, 22, 32) };
                    let stroke = if is_active { Color32::from_rgb(0, 220, 255) } else { Color32::from_rgb(32, 44, 60) };

                    egui::Frame::none()
                        .fill(fill)
                        .stroke(Stroke::new(if is_active { 1.6_f32 } else { 1.0_f32 }, stroke))
                        .rounding(egui::Rounding::same(8.0))
                        .inner_margin(10.0)
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new("MEDIUM").strong().size(14.0).color(Color32::from_rgb(0, 220, 255)));
                            ui.label(egui::RichText::new("65% Strength").size(11.0).color(Color32::WHITE));
                            ui.label(egui::RichText::new("Balanced Esports Rumble").small().color(Color32::from_rgb(140, 160, 185)));
                            ui.add_space(6.0);
                            let btn_text = if is_active { "[Active]" } else { "Select Medium" };
                            if ui.button(egui::RichText::new(btn_text).strong().size(12.0)).clicked() {
                                app.apply_vibration_mode(VibrationMode::Medium);
                            }
                        });
                });

                // Tier 3: HIGH
                cols[2].vertical_centered(|ui| {
                    let is_active = app.config.vibration_mode == VibrationMode::High;
                    let fill = if is_active { Color32::from_rgb(32, 18, 44) } else { Color32::from_rgb(16, 22, 32) };
                    let stroke = if is_active { Color32::from_rgb(210, 110, 255) } else { Color32::from_rgb(32, 44, 60) };

                    egui::Frame::none()
                        .fill(fill)
                        .stroke(Stroke::new(if is_active { 1.6_f32 } else { 1.0_f32 }, stroke))
                        .rounding(egui::Rounding::same(8.0))
                        .inner_margin(10.0)
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new("HIGH").strong().size(14.0).color(Color32::from_rgb(210, 110, 255)));
                            ui.label(egui::RichText::new("100% Strength").size(11.0).color(Color32::WHITE));
                            ui.label(egui::RichText::new("Maximum Dual-Motor Power").small().color(Color32::from_rgb(140, 160, 185)));
                            ui.add_space(6.0);
                            let btn_text = if is_active { "[Active]" } else { "Select High" };
                            if ui.button(egui::RichText::new(btn_text).strong().size(12.0)).clicked() {
                                app.apply_vibration_mode(VibrationMode::High);
                            }
                        });
                });

                // Tier 4: OFF
                cols[3].vertical_centered(|ui| {
                    let is_active = app.config.vibration_mode == VibrationMode::Off;
                    let fill = if is_active { Color32::from_rgb(32, 16, 18) } else { Color32::from_rgb(16, 22, 32) };
                    let stroke = if is_active { Color32::from_rgb(240, 70, 70) } else { Color32::from_rgb(32, 44, 60) };

                    egui::Frame::none()
                        .fill(fill)
                        .stroke(Stroke::new(if is_active { 1.6_f32 } else { 1.0_f32 }, stroke))
                        .rounding(egui::Rounding::same(8.0))
                        .inner_margin(10.0)
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new("OFF").strong().size(14.0).color(Color32::from_rgb(240, 70, 70)));
                            ui.label(egui::RichText::new("0% Strength").size(11.0).color(Color32::WHITE));
                            ui.label(egui::RichText::new("Stealth / Zero Feedback").small().color(Color32::from_rgb(140, 160, 185)));
                            ui.add_space(6.0);
                            let btn_text = if is_active { "[Active]" } else { "Turn Off" };
                            if ui.button(egui::RichText::new(btn_text).strong().size(12.0)).clicked() {
                                app.apply_vibration_mode(VibrationMode::Off);
                            }
                        });
                });
            });

            ui.add_space(14.0);

            // Test Vibration Pulse Buttons
            ui.horizontal(|ui| {
                let test_btn = egui::Button::new(
                    egui::RichText::new("⚡ Test Haptic Pulse (450ms)")
                        .strong()
                        .size(13.0)
                        .color(Color32::BLACK),
                )
                .fill(Color32::from_rgb(0, 220, 255))
                .min_size(Vec2::new(200.0, 32.0));

                if ui.add(test_btn).clicked() {
                    app.trigger_vibration_pulse(std::time::Duration::from_millis(450));
                    app.set_status(format!("Pulsing motors at {} force...", app.config.vibration_mode.label()));
                }

                if ui.button(egui::RichText::new("Stop Motors").size(12.5).color(Color32::from_rgb(255, 90, 90))).clicked() {
                    app.stop_vibration();
                    app.set_status("Motors immediately stopped.");
                }
            });
        });

    ui.add_space(12.0);

    // ----------------------------------------------------
    // SECTION 2: DUAL ASYMMETRIC MOTOR ARCHITECTURE
    // ----------------------------------------------------
    egui::Frame::none()
        .fill(Color32::from_rgb(14, 18, 27))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(26, 35, 52)))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(14.0)
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new("Dual Asymmetric Motor Architecture")
                    .strong()
                    .size(14.0)
                    .color(Color32::from_rgb(0, 220, 255)),
            );
            ui.add_space(6.0);

            ui.columns(2, |cols| {
                cols[0].vertical(|ui| {
                    ui.label(egui::RichText::new("Left Motor: Heavy Low-Frequency").strong().size(13.0).color(Color32::WHITE));
                    ui.label(
                        egui::RichText::new("Produces deep, heavy thumping vibrations designed for major explosions, vehicle collisions, recoil from heavy weapons, and physical impacts.")
                            .size(12.0)
                            .color(Color32::from_rgb(165, 180, 205)),
                    );
                });
                cols[1].vertical(|ui| {
                    ui.label(egui::RichText::new("Right Motor: Sharp High-Frequency").strong().size(13.0).color(Color32::WHITE));
                    ui.label(
                        egui::RichText::new("Produces rapid, buzzing micro-vibrations for subtle tactile textures, tire slippage/gravel feedback in sim racing, and delicate in-game clicks.")
                            .size(12.0)
                            .color(Color32::from_rgb(165, 180, 205)),
                    );
                });
            });
        });

    ui.add_space(12.0);

    // ----------------------------------------------------
    // SECTION 3: EASYSMX HARDWARE SHORTCUT QUICK REFERENCE
    // ----------------------------------------------------
    egui::Frame::none()
        .fill(Color32::from_rgb(14, 18, 27))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(26, 35, 52)))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(14.0)
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new("EasySMX X05 Pro Hardware Shortcuts")
                    .strong()
                    .size(14.0)
                    .color(Color32::from_rgb(200, 215, 235)),
            );
            ui.add_space(6.0);

            ui.columns(2, |cols| {
                cols[0].vertical(|ui| {
                    ui.label(egui::RichText::new("• FN + Left Stick Click (L3): Cycle Vibration (0%, 30%, 70%, 100%)").size(12.5).color(Color32::from_rgb(175, 190, 215)));
                    ui.label(egui::RichText::new("• FN + D-Pad Left: Cycle RGB Light Modes on physical controller").size(12.5).color(Color32::from_rgb(175, 190, 215)));
                });
                cols[1].vertical(|ui| {
                    ui.label(egui::RichText::new("• FN + D-Pad Right: Cycle 7 Solid RGB Colors on physical controller").size(12.5).color(Color32::from_rgb(175, 190, 215)));
                    ui.label(egui::RichText::new("• FN + D-Pad Up / Down: Adjust physical LED Brightness").size(12.5).color(Color32::from_rgb(175, 190, 215)));
                });
            });
        });
}

// ----------------------------------------------------
// GAME PROFILES TAB
// ----------------------------------------------------
fn render_profiles(app: &mut SuiteApp, ui: &mut egui::Ui) {
    ui.label(
        egui::RichText::new("Game Profiles & Presets")
            .strong()
            .size(20.0)
            .color(Color32::WHITE),
    );
    ui.add_space(2.0);
    ui.label(
        egui::RichText::new("Instantly load genre-specific presets with tuned deadzones, response curves, and hair-triggers.")
            .size(13.0)
            .color(Color32::from_rgb(175, 190, 215)),
    );
    ui.add_space(12.0);

    // Active Profile Status Bar
    egui::Frame::none()
        .fill(Color32::from_rgb(14, 18, 27))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(26, 35, 52)))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Current Active Preset:")
                        .strong()
                        .size(14.0)
                        .color(Color32::from_rgb(190, 205, 225)),
                );
                if app.config.active_profile == "Default (No Profile)" {
                    ui.label(
                        egui::RichText::new("No Profile Active (Balanced Defaults)")
                            .size(14.0)
                            .color(Color32::from_rgb(140, 160, 185)),
                    );
                } else {
                    ui.label(
                        egui::RichText::new(format!("[ ACTIVE: {} ]", app.config.active_profile))
                            .strong()
                            .size(14.0)
                            .color(Color32::from_rgb(0, 220, 255)),
                    );
                    ui.add_space(12.0);
                    let deact_btn = egui::Button::new(
                        egui::RichText::new("Deactivate Profile")
                            .strong()
                            .size(12.5)
                            .color(Color32::from_rgb(255, 100, 100)),
                    );
                    if ui.add(deact_btn).clicked() {
                        app.config.deactivate_profile();
                        if let Ok(mut cfg) = app.shared_config.lock() {
                            *cfg = app.config.clone();
                        }
                        let _ = app.config.save();
                        app.set_status("Profile deactivated. Restored default settings.");
                    }
                }
            });
        });

    ui.add_space(10.0);

    let profiles = [
        (
            "[ FOOTBALL ESPORTS ] EA Sports FC / FIFA (FUT Champs & Seasons)",
            "Ultra-sharp 2.5% LS dribbling deadzone, responsive RS skill-move flicking, and progressive sprint triggers for controlled acceleration.",
        ),
        (
            "[ FPS ESPORTS ] Competitive Shooter (Apex / COD / Valorant)",
            "Ultra-low 2% deadzones, aggressive exponential response curve, instant hair-trigger firing.",
        ),
        (
            "[ RPG / ACTION ] Zelda: Breath of the Wild / TotK",
            "Smooth S-curve thumbstick response for archery aiming, Instant digital hair-triggers for quick parrying.",
        ),
        (
            "[ RACING SIM ] Sim Driving (Forza / F1)",
            "Nuanced progressive linear trigger inputs for precise throttle and brake modulation.",
        ),
        (
            "[ CONSOLE ] Balanced Console Default",
            "Standard 1:1 console response curve and balanced deadzones.",
        ),
    ];

    for (name, desc) in profiles {
        let is_current = app.config.active_profile == name;
        let border = if is_current {
            Color32::from_rgb(0, 210, 255)
        } else {
            Color32::from_rgb(26, 35, 52)
        };

        let bg = if is_current {
            Color32::from_rgb(16, 28, 44)
        } else {
            Color32::from_rgb(14, 18, 27)
        };

        egui::Frame::none()
            .fill(bg)
            .stroke(Stroke::new(if is_current { 1.5_f32 } else { 1.0_f32 }, border))
            .rounding(egui::Rounding::same(10.0))
            .inner_margin(14.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(name).strong().size(15.0).color(Color32::WHITE));
                        ui.add_space(3.0);
                        ui.label(egui::RichText::new(desc).size(13.0).color(Color32::from_rgb(180, 195, 220)));
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if is_current {
                            ui.label(
                                egui::RichText::new("[ ACTIVE ]")
                                    .strong()
                                    .size(13.5)
                                    .color(Color32::from_rgb(0, 230, 120)),
                            );
                        } else {
                            let apply_btn = egui::Button::new(
                                egui::RichText::new("Apply Preset")
                                    .strong()
                                    .size(13.0),
                            )
                            .min_size(Vec2::new(95.0, 30.0));

                            if ui.add(apply_btn).clicked() {
                                app.config.apply_profile(name);
                                app.target_hz.store(app.config.polling_rate_hz, std::sync::atomic::Ordering::Relaxed);
                                if let Ok(mut cfg) = app.shared_config.lock() {
                                    *cfg = app.config.clone();
                                }
                                let _ = app.config.save();
                                app.set_status(format!("Activated preset: {}", name));
                            }
                        }
                    });
                });
            });
        ui.add_space(6.0);
    }
}

// ----------------------------------------------------
// DESKTOP MOUSE MODE TAB
// ----------------------------------------------------
fn render_desktop_mode(app: &mut SuiteApp, ui: &mut egui::Ui) {
    ui.label(
        egui::RichText::new("Desktop Mouse Navigation")
            .strong()
            .size(20.0)
            .color(Color32::WHITE),
    );
    ui.add_space(2.0);
    ui.label(
        egui::RichText::new("Control Windows desktop, browser, and media without reaching for your keyboard or mouse.")
            .size(13.0)
            .color(Color32::from_rgb(175, 190, 215)),
    );
    ui.add_space(12.0);

    egui::Frame::none()
        .fill(Color32::from_rgb(14, 18, 27))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(26, 35, 52)))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(14.0)
        .show(ui, |ui| {
            ui.checkbox(&mut app.config.desktop_mouse_mode, egui::RichText::new("Enable Gamepad Desktop Mouse Mode").strong().size(15.0));
            ui.add_space(8.0);

            if app.config.desktop_mouse_mode {
                ui.add(
                    egui::Slider::new(&mut app.config.mouse_sensitivity, 5.0..=40.0)
                        .text("Cursor Sensitivity Speed"),
                );
                ui.add_space(10.0);
            }

            ui.separator();
            ui.add_space(8.0);

            ui.label(egui::RichText::new("Desktop Navigation Controls:").strong().size(14.5).color(Color32::from_rgb(0, 220, 255)));
            ui.add_space(4.0);
            ui.label(egui::RichText::new("[-] Left Thumbstick: Smooth Mouse Cursor Movement").size(13.5).color(Color32::from_rgb(205, 220, 240)));
            ui.label(egui::RichText::new("[-] A Button: Left Mouse Click").size(13.5).color(Color32::from_rgb(205, 220, 240)));
            ui.label(egui::RichText::new("[-] B Button: Right Mouse Click").size(13.5).color(Color32::from_rgb(205, 220, 240)));
            ui.label(egui::RichText::new("[-] D-Pad Up / Down: Smooth Webpage & Document Scroll Wheel").size(13.5).color(Color32::from_rgb(205, 220, 240)));
        });

    ui.add_space(12.0);
    let save_text = egui::RichText::new("Save Settings").strong().size(14.0);
    if ui.add(egui::Button::new(save_text).min_size(Vec2::new(140.0, 34.0))).clicked() {
        let _ = app.config.save();
        app.set_status("Desktop mouse configuration saved.");
    }
}

// ----------------------------------------------------
// SETTINGS & SYSTEM TAB
// ----------------------------------------------------
fn render_settings(app: &mut SuiteApp, ui: &mut egui::Ui) {
    ui.label(
        egui::RichText::new("Settings & System Integration")
            .strong()
            .size(20.0)
            .color(Color32::WHITE),
    );
    ui.add_space(2.0);
    ui.label(
        egui::RichText::new("Configure startup behavior, update checks, and background telemetry options.")
            .size(13.0)
            .color(Color32::from_rgb(175, 190, 215)),
    );
    ui.add_space(12.0);

    egui::Frame::none()
        .fill(Color32::from_rgb(14, 18, 27))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(26, 35, 52)))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.label(egui::RichText::new("Windows Integration & System Tray").strong().size(17.0).color(Color32::from_rgb(0, 220, 255)));
            ui.add_space(10.0);

            let mut start = app.startup_enabled;
            if ui.checkbox(&mut start, egui::RichText::new("Launch EasySMX X05 Pro Suite on Windows Startup").size(14.5)).changed() {
                let _ = startup::set_startup_enabled(start);
                app.startup_enabled = start;
                app.set_status(if start { "Auto-start enabled" } else { "Auto-start disabled" });
            }

            ui.add_space(10.0);
            if ui.checkbox(&mut app.config.minimize_to_tray, egui::RichText::new("Minimize to System Tray on Window Close (Runs silently in background)").size(14.5)).changed() {
                let _ = app.config.save();
                app.set_status("System Tray preference saved.");
            }

            ui.add_space(10.0);
            ui.checkbox(&mut app.config.auto_check_updates, egui::RichText::new("Check for Updates Automatically at Launch").size(14.5));
        });

    ui.add_space(12.0);

    // Battery & System Audio Alerts
    egui::Frame::none()
        .fill(Color32::from_rgb(14, 18, 27))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(26, 35, 52)))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.label(egui::RichText::new("Battery & Audio Alerts").strong().size(17.0).color(Color32::from_rgb(0, 220, 255)));
            ui.add_space(10.0);

            if ui.checkbox(&mut app.config.low_battery_alert, egui::RichText::new("Low Battery Sound & Balloon Notification (Alerts at <= 20%)").size(14.5)).changed() {
                let _ = app.config.save();
                app.set_status("Battery alert preference saved.");
            }

            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("Plays Windows warning chime and shows a notification balloon when battery is low or charging reaches 100%.")
                    .size(13.5)
                    .color(Color32::from_rgb(180, 195, 220)),
            );

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button(egui::RichText::new("Test Warning Sound & Alert").size(13.5).strong().color(Color32::from_rgb(255, 190, 40))).clicked() {
                    crate::tray::play_alert_sound(true);
                    crate::tray::show_balloon(
                        "EasySMX X05 Pro - Test Notification",
                        "Audio chime and balloon notification are working perfectly!",
                        true,
                    );
                    app.set_status("Played test warning chime and sent tray notification.");
                }

                if ui.button(egui::RichText::new("Test Full Charge Sound").size(13.5).strong().color(Color32::from_rgb(0, 240, 140))).clicked() {
                    crate::tray::play_alert_sound(false);
                    crate::tray::show_balloon(
                        "EasySMX X05 Pro - Battery Full Test",
                        "Battery 100% full chime working!",
                        false,
                    );
                    app.set_status("Played test full-charge chime.");
                }
            });
        });

    ui.add_space(10.0);

    // GitHub Updates & Sync
    egui::Frame::none()
        .fill(Color32::from_rgb(14, 18, 27))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(26, 35, 52)))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(14.0)
        .show(ui, |ui| {
            ui.label(egui::RichText::new("GitHub Release Channel").strong().size(15.5).color(Color32::from_rgb(0, 220, 255)));
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Repository:").size(13.5));
                ui.text_edit_singleline(&mut app.repo_input);
                if ui.button(egui::RichText::new("Save Repo").strong().size(13.0)).clicked() {
                    app.config.github_repo = app.repo_input.clone();
                    let _ = app.config.save();
                    app.set_status("GitHub update repository updated.");
                }
            });

            ui.add_space(8.0);
            if ui.checkbox(&mut app.config.auto_install_updates, egui::RichText::new("Zero-Click Auto Update: Download & restart on startup when new update is uploaded").size(13.0)).changed() {
                let _ = app.config.save();
            }

            ui.add_space(10.0);
            if ui.button(egui::RichText::new("Check for Releases Now").strong().size(13.5)).clicked() {
                app.updater.check_for_updates(app.config.github_repo.clone(), false, true);
                app.set_status("Querying GitHub for latest release...");
            }
        });
}

// ----------------------------------------------------
// HARDWARE GUIDE TAB
// ----------------------------------------------------
fn render_hardware_guide(ui: &mut egui::Ui) {
    ui.label(
        egui::RichText::new("EasySMX X05 Pro Hardware Reference")
            .strong()
            .size(20.0)
            .color(Color32::WHITE),
    );
    ui.add_space(2.0);
    ui.label(
        egui::RichText::new("Official button shortcuts, LED pairing modes, and onboard hardware calibrations.")
            .size(13.0)
            .color(Color32::from_rgb(175, 190, 215)),
    );
    ui.add_space(12.0);

    let shortcuts = [
        ("Switch to 2.4GHz Wireless Mode", "Set back toggle switch to 2.4G. Hold HOME for 3 seconds to pair with USB Dongle (LEDs pulse blue)."),
        ("Switch to Bluetooth Mode (iOS / Android / PC)", "Set back switch to BT. Hold HOME for 3 seconds (LEDs flash rapidly). Pair with 'Xbox Wireless Controller' in Bluetooth settings."),
        ("Switch to Wired USB-C Mode", "Plug in USB-C cable directly to PC. Instantly activates 1000Hz wired zero-latency mode and begins fast charging."),
        ("RGB Light Modes", "Hold TURBO + Press D-Pad Right to cycle RGB illumination modes (Breathing, Solid, Neon Flow, Off)."),
        ("RGB Light Color Palette", "Hold TURBO + Press D-Pad Left to cycle RGB accent colors."),
        ("Stick & Trigger Hardware Calibration", "Hold BACK + B + HOME simultaneously until LEDs flash red, rotate both sticks full 360 deg, fully squeeze triggers, then press HOME to lock calibration."),
        ("Hardware Rapid-Fire Turbo", "Hold TURBO + Press any Action Button (A/B/X/Y) to toggle hardware rapid fire."),
        ("Vibration Strength Adjustment", "Hold TURBO + Move Left Stick Up/Down to cycle haptic rumble intensity (0%, 30%, 70%, 100%)."),
    ];

    for (title, desc) in shortcuts {
        egui::Frame::none()
            .fill(Color32::from_rgb(14, 18, 27))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(26, 35, 52)))
            .rounding(egui::Rounding::same(8.0))
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.label(egui::RichText::new(title).strong().size(14.5).color(Color32::from_rgb(0, 220, 255)));
                ui.add_space(3.0);
                ui.label(egui::RichText::new(desc).size(13.0).color(Color32::from_rgb(190, 205, 225)));
            });
        ui.add_space(6.0);
    }
}



// ----------------------------------------------------
// HIGH-TECH RADAR WITH LIVE ROTATING SCANNER ANIMATION
// ----------------------------------------------------
fn draw_stick_radar(ui: &mut egui::Ui, x: f32, y: f32, deadzone: f32, current_time: f64, ring_color: Color32) {
    let size = Vec2::splat(84.0);
    let (rect, _response) = ui.allocate_exact_size(size, egui::Sense::hover());
    let painter = ui.painter_at(rect);

    let center = rect.center();
    let radius = size.x / 2.0 - 4.0;

    // Outer background box with high-tech border
    painter.rect_filled(rect, egui::Rounding::same(8.0), Color32::from_rgb(11, 15, 22));
    painter.rect_stroke(rect, egui::Rounding::same(8.0), Stroke::new(1.0_f32, Color32::from_rgb(28, 38, 54)));

    // Concentric Radar Rings (Accentuated with user's selected RGB theme)
    painter.circle_stroke(center, radius, Stroke::new(1.6_f32, ring_color));
    painter.circle_stroke(center, radius * 0.75, Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(ring_color.r(), ring_color.g(), ring_color.b(), 70)));
    painter.circle_stroke(center, radius * 0.50, Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(35, 50, 75, 140)));

    // Deadzone Circle (translucent reddish tint)
    painter.circle_stroke(center, radius * deadzone, Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 75, 75, 170)));
    painter.circle_filled(center, radius * deadzone, Color32::from_rgba_unmultiplied(255, 50, 50, 20));

    // Radar Crosshairs
    painter.line_segment([Pos2::new(center.x - radius, center.y), Pos2::new(center.x + radius, center.y)], Stroke::new(1.0_f32, Color32::from_rgb(28, 38, 54)));
    painter.line_segment([Pos2::new(center.x, center.y - radius), Pos2::new(center.x, center.y + radius)], Stroke::new(1.0_f32, Color32::from_rgb(28, 38, 54)));

    // Continuous Military/Oscilloscope Radar Sweep Line Animation!
    let sweep_angle = (current_time * 1.5) as f32 % (std::f32::consts::TAU);
    let sweep_end = Pos2::new(
        center.x + sweep_angle.cos() * radius,
        center.y + sweep_angle.sin() * radius,
    );
    painter.line_segment([center, sweep_end], Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(ring_color.r(), ring_color.g(), ring_color.b(), 65)));

    // Stick Head Pointer with Glowing RGB Ring & White Core
    let dot_pos = Pos2::new(center.x + x * radius, center.y - y * radius);
    painter.line_segment([center, dot_pos], Stroke::new(1.8_f32, Color32::from_rgba_unmultiplied(ring_color.r(), ring_color.g(), ring_color.b(), 170)));
    painter.circle_filled(dot_pos, 5.0, ring_color);
    painter.circle_filled(dot_pos, 2.0, Color32::WHITE);
}

// ----------------------------------------------------
// TACTILE BUTTON ROW & CLUSTER HELPERS (Zero Overflow)
// ----------------------------------------------------
fn two_btn_row(ui: &mut egui::Ui, b1: (&str, bool, Color32), b2: (&str, bool, Color32)) {
    ui.columns(2, |cols| {
        cols[0].vertical(|ui| {
            render_tactile_button(ui, b1.0, b1.1, b1.2, ui.available_width(), 24.0);
        });
        cols[1].vertical(|ui| {
            render_tactile_button(ui, b2.0, b2.1, b2.2, ui.available_width(), 24.0);
        });
    });
}

fn render_tactile_button(ui: &mut egui::Ui, label: &str, pressed: bool, accent_color: Color32, width: f32, height: f32) {
    let (bg, border, text_color) = if pressed {
        (accent_color, accent_color, Color32::BLACK)
    } else {
        (
            Color32::from_rgb(18, 24, 34),
            Color32::from_rgba_unmultiplied(accent_color.r(), accent_color.g(), accent_color.b(), 120),
            accent_color,
        )
    };

    let text = egui::RichText::new(label)
        .size(10.5)
        .strong()
        .color(text_color);

    let btn = egui::Button::new(text)
        .fill(bg)
        .stroke(Stroke::new(1.2_f32, border))
        .rounding(egui::Rounding::same(6.0));

    ui.add_sized([width, height], btn);
}

fn compact_btn(ui: &mut egui::Ui, label: &str, pressed: bool, accent_color: Color32, size: Vec2) {
    let (bg, border, text_color) = if pressed {
        (accent_color, accent_color, Color32::BLACK)
    } else {
        (
            Color32::from_rgb(18, 24, 34),
            Color32::from_rgba_unmultiplied(accent_color.r(), accent_color.g(), accent_color.b(), 130),
            accent_color,
        )
    };

    let text = egui::RichText::new(label)
        .size(10.5)
        .strong()
        .color(text_color);

    let btn = egui::Button::new(text)
        .fill(bg)
        .stroke(Stroke::new(1.2_f32, border))
        .rounding(egui::Rounding::same(6.0));

    ui.add_sized(size, btn);
}

fn render_updater_banner(app: &mut SuiteApp, ui: &mut egui::Ui) {
    let update_status = {
        let guard = app.updater.status.lock().unwrap();
        guard.clone()
    };

    match update_status {
        UpdateStatus::Available(info) => {
            egui::Frame::none()
                .fill(Color32::from_rgb(14, 32, 48))
                .stroke(Stroke::new(1.2_f32, Color32::from_rgb(0, 210, 255)))
                .rounding(egui::Rounding::same(8.0))
                .inner_margin(10.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("New Version Available: v{} (Current: v{})", info.new_version, info.current_version))
                                .strong()
                                .color(Color32::from_rgb(0, 240, 255)),
                        );

                        let update_btn = egui::Button::new(
                            egui::RichText::new("Update Now & Restart").strong().color(Color32::BLACK),
                        )
                        .fill(Color32::from_rgb(0, 220, 255));

                        if ui.add(update_btn).clicked() {
                            app.updater.download_and_install(info.download_url.clone());
                        }

                        if ui.button(if app.show_update_notes { "Hide Notes" } else { "Release Notes" }).clicked() {
                            app.show_update_notes = !app.show_update_notes;
                        }

                        if ui.button("Dismiss").clicked() {
                            *app.updater.status.lock().unwrap() = UpdateStatus::Idle;
                        }
                    });

                    if app.show_update_notes {
                        ui.add_space(6.0);
                        ui.label(egui::RichText::new(&info.release_notes).small().color(Color32::LIGHT_GRAY));
                    }
                });
            ui.add_space(6.0);
        }
        UpdateStatus::Downloading { progress_label } => {
            egui::Frame::none()
                .fill(Color32::from_rgb(20, 34, 18))
                .stroke(Stroke::new(1.2_f32, Color32::from_rgb(0, 245, 120)))
                .rounding(egui::Rounding::same(8.0))
                .inner_margin(10.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(
                            egui::RichText::new(format!("Downloading Update: {}", progress_label))
                                .strong()
                                .color(Color32::from_rgb(0, 245, 140)),
                        );
                    });
                });
            ui.add_space(6.0);
        }
        UpdateStatus::ReadyToRestart => {
            egui::Frame::none()
                .fill(Color32::from_rgb(18, 38, 26))
                .stroke(Stroke::new(1.5_f32, Color32::from_rgb(0, 245, 140)))
                .rounding(egui::Rounding::same(8.0))
                .inner_margin(10.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Update Downloaded & Verified!")
                                .strong()
                                .color(Color32::from_rgb(0, 245, 140)),
                        );
                        let restart_btn = egui::Button::new(
                            egui::RichText::new("Restart App to Apply").strong().color(Color32::BLACK),
                        )
                        .fill(Color32::from_rgb(0, 245, 140));

                        if ui.add(restart_btn).clicked() {
                            let _ = AutoUpdater::apply_update_and_restart();
                        }
                    });
                });
            ui.add_space(6.0);
        }
        _ => {}
    }
}

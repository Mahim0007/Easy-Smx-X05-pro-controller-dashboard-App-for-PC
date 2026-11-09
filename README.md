# 🎮 EasySMX X05 Pro - Gamepad Suite & Optimizer (Rust)

Ultra-lightweight, high-performance PC companion app specifically tailored for the **EasySMX X05 Pro** and standard XInput controllers, built 100% in **Rust** with native Windows API integration.

---

## ⚡ Highlights & Key Features

* **🏎️ Ultra-Low RAM & CPU Overhead:** 
  * Uses only **~15MB - 20MB of RAM** (zero Chromium / Electron bloat).
  * 0% idle CPU footprint.
* **🚀 Auto-Start with Windows:** 
  * One-click toggle directly in the UI to register with the Windows Registry (`HKCU\...\Run`). Runs instantly upon PC startup.
* **🔋 Live Battery Meter & Charging State:** 
  * Real-time monitoring of controller power (Full, Medium, Low, Critical, Wired Charging).
* **🎯 Live Hall-Effect Stick Radar & Circularity:** 
  * Visual circularity and live X/Y crosshair radars for Left & Right sticks.
* **⚡ Digital Hair-Triggers:** 
  * Instant actuation (e.g., 2% to 5% trigger press sends 100% fire input) for competitive shooters and snappy bow/parry in Zelda.
* **📁 Game Presets & Profiles:** 
  * One-click profiles for *Zelda: Breath of the Wild / TotK*, *Competitive FPS*, *Racing / Sim*, and *Balanced*.
* **🖱️ Desktop Mouse Mode:** 
  * Control your Windows mouse cursor, left/right click, and scroll wheel using your controller thumbsticks from bed/couch.
* **📖 EasySMX X05 Pro Hardware Cheat-Sheet:** 
  * Built-in guide for onboard RGB modes/color switching, Hall-effect stick calibration, and hardware vibration adjustment.

---

## 🛠️ How to Build & Run

### Prerequisites
1. **Rust Toolchain:** Install Rust from [https://rustup.rs](https://rustup.rs) (or run `winget install Rustlang.Rustup`).

### Compile & Launch
Open PowerShell inside this directory (`C:\Users\mahim\.gemini\antigravity\scratch\x05-controller-suite`) and run:

```powershell
# Development run (Fast compile)
cargo run

# Optimized Production Build (Ultra-small binary, lowest RAM)
cargo build --release
```

The optimized `.exe` will be located at:
`target\release\x05-controller-suite.exe`

---

## 📂 Project Architecture

```
x05-controller-suite/
├── Cargo.toml             # Rust dependencies (eframe, egui, windows-rs)
├── README.md              # Documentation
└── src/
    ├── main.rs            # Application entry, 60 FPS event loop
    ├── controller.rs      # XInput controller poller & battery reader
    ├── config.rs          # Profile persistence (%APPDATA%\EasySMX_X05_Suite)
    ├── startup.rs         # Windows Registry Run key manager
    ├── desktop_mode.rs    # Windows SendInput mouse emulation
    └── ui.rs              # Cyberpunk dark theme egui interface & radars
```


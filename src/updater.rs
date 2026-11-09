use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
    #[serde(default)]
    pub size: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub html_url: String,
    #[serde(default)]
    pub assets: Vec<GitHubAsset>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdateInfo {
    pub current_version: String,
    pub new_version: String,
    pub release_name: String,
    pub release_notes: String,
    pub download_url: String,
    pub asset_size: u64,
    pub html_url: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UpdateStatus {
    Idle,
    Checking,
    UpToDate { version: String },
    Available(UpdateInfo),
    Downloading { progress_label: String },
    ReadyToRestart,
    Error(String),
}

#[derive(Clone)]
pub struct AutoUpdater {
    pub status: Arc<Mutex<UpdateStatus>>,
    pub is_busy: Arc<AtomicBool>,
}

impl Default for AutoUpdater {
    fn default() -> Self {
        Self::new()
    }
}

impl AutoUpdater {
    pub fn new() -> Self {
        Self {
            status: Arc::new(Mutex::new(UpdateStatus::Idle)),
            is_busy: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Check GitHub Releases for new versions in a non-blocking background thread
    pub fn check_for_updates(&self, repo: String, manual_trigger: bool) {
        if self.is_busy.swap(true, Ordering::SeqCst) {
            return;
        }

        let status_arc = Arc::clone(&self.status);
        let is_busy_arc = Arc::clone(&self.is_busy);

        *status_arc.lock().unwrap() = UpdateStatus::Checking;

        thread::spawn(move || {
            let result = Self::perform_check(&repo);
            let mut status = status_arc.lock().unwrap();
            match result {
                Ok(Some(info)) => {
                    *status = UpdateStatus::Available(info);
                }
                Ok(None) => {
                    *status = UpdateStatus::UpToDate {
                        version: env!("CARGO_PKG_VERSION").to_string(),
                    };
                }
                Err(err) => {
                    if manual_trigger {
                        *status = UpdateStatus::Error(err);
                    } else {
                        // Silent fallback for automatic startup check
                        *status = UpdateStatus::Idle;
                    }
                }
            }
            is_busy_arc.store(false, Ordering::SeqCst);
        });
    }

    fn perform_check(repo: &str) -> Result<Option<UpdateInfo>, String> {
        let clean_repo = repo.trim().trim_matches('/');
        if clean_repo.is_empty() {
            return Err("GitHub repository path is empty. Format: owner/repo".to_string());
        }

        let api_url = format!("https://api.github.com/repos/{}/releases/latest", clean_repo);

        // Fetch release JSON using Windows built-in curl.exe with hidden console
        let mut cmd = std::process::Command::new("curl.exe");
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }
        let output = cmd
            .args([
                "-s",
                "-H",
                "User-Agent: EasySMX-X05-Suite-Updater",
                "-H",
                "Accept: application/vnd.github.v3+json",
                &api_url,
            ])
            .output()
            .map_err(|e| format!("Could not run curl.exe: {}", e))?;

        if !output.status.success() {
            return Err("Failed to connect to GitHub API.".to_string());
        }

        let stdout_str = String::from_utf8_lossy(&output.stdout);
        if stdout_str.contains("\"message\": \"Not Found\"") {
            return Err(format!("Repository '{}' or its releases were not found on GitHub.", clean_repo));
        }

        let release: GitHubRelease = serde_json::from_str(&stdout_str)
            .map_err(|e| format!("Invalid response from GitHub: {}", e))?;

        let current_ver = env!("CARGO_PKG_VERSION");
        let remote_ver = release.tag_name.trim_start_matches(|c| c == 'v' || c == 'V');

        if is_newer_version(current_ver, remote_ver) {
            // Find .exe asset, or fallback to first asset or html url
            let exe_asset = release.assets.iter().find(|a| {
                a.name.ends_with(".exe") || a.name.contains("EasySMX") || a.name.contains("controller")
            });

            let (download_url, size) = if let Some(asset) = exe_asset {
                (asset.browser_download_url.clone(), asset.size)
            } else if let Some(first) = release.assets.first() {
                (first.browser_download_url.clone(), first.size)
            } else {
                (release.html_url.clone(), 0)
            };

            Ok(Some(UpdateInfo {
                current_version: current_ver.to_string(),
                new_version: remote_ver.to_string(),
                release_name: release.name.unwrap_or_else(|| release.tag_name.clone()),
                release_notes: release.body.unwrap_or_else(|| "No release notes provided.".to_string()),
                download_url,
                asset_size: size,
                html_url: release.html_url,
            }))
        } else {
            Ok(None)
        }
    }

    /// Download new binary and prepare atomic replacement
    pub fn download_and_install(&self, download_url: String) {
        if self.is_busy.swap(true, Ordering::SeqCst) {
            return;
        }

        let status_arc = Arc::clone(&self.status);
        let is_busy_arc = Arc::clone(&self.is_busy);

        *status_arc.lock().unwrap() = UpdateStatus::Downloading {
            progress_label: "Starting download...".to_string(),
        };

        thread::spawn(move || {
            let temp_dest = std::env::temp_dir().join("EasySMX_Update_New.exe");
            let result = Self::perform_download(&download_url, &temp_dest);

            let mut status = status_arc.lock().unwrap();
            match result {
                Ok(_) => {
                    *status = UpdateStatus::ReadyToRestart;
                }
                Err(e) => {
                    *status = UpdateStatus::Error(format!("Download failed: {}", e));
                }
            }
            is_busy_arc.store(false, Ordering::SeqCst);
        });
    }

    fn perform_download(url: &str, dest: &PathBuf) -> Result<(), String> {
        if dest.exists() {
            let _ = std::fs::remove_file(dest);
        }

        let mut cmd = std::process::Command::new("curl.exe");
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }
        let output = cmd
            .args([
                "-L", // Follow redirects
                "-o",
                &dest.to_string_lossy(),
                "-H",
                "User-Agent: EasySMX-X05-Suite-Updater",
                url,
            ])
            .output()
            .map_err(|e| format!("Failed to invoke curl: {}", e))?;

        if !output.status.success() {
            return Err("Download process exited with an error.".to_string());
        }

        // Verify file was downloaded and is non-empty
        if !dest.exists() {
            return Err("Downloaded binary was not saved on disk.".to_string());
        }

        let metadata = std::fs::metadata(dest).map_err(|e| e.to_string())?;
        if metadata.len() < 50_000 {
            return Err("Downloaded file appears corrupt or is too small.".to_string());
        }

        Ok(())
    }

    /// Relaunches the app cleanly using atomic replacement script
    pub fn apply_update_and_restart() -> Result<(), String> {
        let current_exe = std::env::current_exe()
            .map_err(|e| format!("Failed to get current executable path: {}", e))?;
        let temp_dest = std::env::temp_dir().join("EasySMX_Update_New.exe");

        if !temp_dest.exists() {
            return Err("Update file not found in temporary directory.".to_string());
        }

        let pid = std::process::id();
        let batch_path = std::env::temp_dir().join("easysmx_apply_update.bat");

        let batch_content = format!(
            "@echo off\r\n\
            setlocal\r\n\
            set TARGET=\"{}\"\r\n\
            set TEMP_EXE=\"{}\"\r\n\
            set PID={}\r\n\
            \r\n\
            :WAIT_LOOP\r\n\
            tasklist /fi \"PID eq %PID%\" 2>nul | find \"%PID%\" >nul\r\n\
            if not errorlevel 1 (\r\n\
                timeout /t 1 /nobreak >nul\r\n\
                goto WAIT_LOOP\r\n\
            )\r\n\
            \r\n\
            copy /y %TEMP_EXE% %TARGET% >nul\r\n\
            start \"\" %TARGET%\r\n\
            del %TEMP_EXE% 2>nul\r\n\
            del \"%~f0\" 2>nul\r\n",
            current_exe.display(),
            temp_dest.display(),
            pid
        );

        std::fs::write(&batch_path, batch_content)
            .map_err(|e| format!("Failed to create updater script: {}", e))?;

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            let _ = std::process::Command::new("cmd.exe")
                .args(["/c", &batch_path.to_string_lossy()])
                .creation_flags(CREATE_NO_WINDOW)
                .spawn();
        }

        std::process::exit(0);
    }
}

/// Helper function to compare semver-style strings (e.g. 0.1.0 vs 0.2.0)
pub fn is_newer_version(current: &str, remote: &str) -> bool {
    let parse = |ver_str: &str| -> Vec<u32> {
        ver_str
            .trim_start_matches(|c| c == 'v' || c == 'V')
            .split('.')
            .filter_map(|segment| {
                segment
                    .split(|c: char| !c.is_ascii_digit())
                    .next()
                    .and_then(|digits| digits.parse::<u32>().ok())
            })
            .collect()
    };

    let curr_parts = parse(current);
    let rem_parts = parse(remote);

    for (c, r) in curr_parts.iter().zip(rem_parts.iter()) {
        if r > c {
            return true;
        } else if r < c {
            return false;
        }
    }

    rem_parts.len() > curr_parts.len()
}


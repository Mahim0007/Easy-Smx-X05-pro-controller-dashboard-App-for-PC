fn main() {
    #[cfg(windows)]
    {
        println!("cargo:rerun-if-changed=app.rc");
        println!("cargo:rerun-if-changed=app_icon.ico");

        let out_dir = std::env::var("OUT_DIR").unwrap();
        let res_path = format!("{}/app_res.o", out_dir.replace('\\', "/"));

        // Try windres from PATH or fallback to w64devkit path
        let mut cmd = std::process::Command::new("windres");
        cmd.args(&["app.rc", "-O", "coff", "-o", &res_path]);

        let success = match cmd.status() {
            Ok(s) if s.success() => true,
            _ => {
                let fallback = std::process::Command::new(r"C:\Users\mahim\w64devkit\bin\windres.exe")
                    .args(&["app.rc", "-O", "coff", "-o", &res_path])
                    .status();
                matches!(fallback, Ok(s) if s.success())
            }
        };

        if success {
            println!("cargo:rustc-link-arg={}", res_path);
            println!("cargo:rustc-link-arg=-mwindows");
        }
    }
}


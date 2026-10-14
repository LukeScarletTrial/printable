use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

const DOWNLOAD_URL: &str =
    "https://raw.githubusercontent.com/LukeScarletTrial/printable/main/printable.exe";

fn main() {
    println!("======================================");
    println!("       Printable CLI Installer        ");
    println!("======================================");

    let local_app_data = env::var("LOCALAPPDATA").unwrap_or_else(|_| "C:\\".to_string());
    let install_dir = PathBuf::from(local_app_data).join("Printable").join("bin");

    if !install_dir.exists() {
        println!("[1/3] Creating directory: {}", install_dir.display());
        if let Err(e) = fs::create_dir_all(&install_dir) {
            eprintln!("Failed to create directory: {}", e);
            std::process::exit(1);
        }
    } else {
        println!("[1/3] Target directory: {}", install_dir.display());
    }

    let exe_path = install_dir.join("printable.exe");
    println!("[2/3] Downloading printable.exe...");

    let curl_status = Command::new("curl.exe")
        .args(&["-sSL", DOWNLOAD_URL, "-o", exe_path.to_str().unwrap()])
        .status();

    let downloaded = match curl_status {
        Ok(status) => status.success() && exe_path.exists() && fs::metadata(&exe_path).map(|m| m.len() > 1000).unwrap_or(false),
        Err(_) => false,
    };

    if !downloaded {
        println!("      Remote download failed. Attempting local binary copy...");
        let fallback_sources = [
            "printable.exe",
            "C:\\Users\\MSI\\.cargo\\bin\\printable.exe",
            "target\\release\\printable.exe",
        ];
        let mut copied = false;
        for src in &fallback_sources {
            if Path::new(src).exists() {
                if fs::copy(src, &exe_path).is_ok() {
                    println!("      Installed from local copy: {}", src);
                    copied = true;
                    break;
                }
            }
        }
        if !copied {
            eprintln!("Error: Could not obtain printable.exe");
            std::process::exit(1);
        }
    } else {
        println!("      Downloaded successfully.");
    }

    println!("[3/3] Adding to user environment PATH...");
    let install_dir_str = install_dir.to_str().unwrap();

    let ps_cmd = format!(
        "$p = [Environment]::GetEnvironmentVariable('Path', 'User'); \
         $parts = $p -split ';'; \
         if ($parts -notcontains '{}') {{ \
             [Environment]::SetEnvironmentVariable('Path', ($p + ';{}'), 'User'); \
             Write-Host 'Added to User PATH' \
         }} else {{ \
             Write-Host 'Already in User PATH' \
         }}",
        install_dir_str.replace('\\', "\\\\"),
        install_dir_str.replace('\\', "\\\\")
    );

    let _ = Command::new("powershell")
        .args(&["-NoProfile", "-Command", &ps_cmd])
        .status();

    println!("\nInstallation succeeded!");
    println!("Executable installed at: {}", exe_path.display());
    println!("Try running 'printable' in any new terminal window.");
}

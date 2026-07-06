use std::env;
use std::fs::{self};
use std::path::Path;

fn main() {
    // Only run this build hook if target is Linux (where the ABI pollution occurs)
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "linux" {
        return;
    }

    // Set up local workspace asset paths
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let bin_dir = Path::new(&manifest_dir).join("bin");
    let ffmpeg_path = bin_dir.join("ffmpeg");

    // If the static binary is already downloaded, skip network traffic entirely
    if ffmpeg_path.exists() {
        println!("cargo:rerun-if-changed=bin/ffmpeg");
        return;
    }

    println!("cargo:warning=⏳ Local static FFmpeg not found. Fetching pristine isolated build...");

    // Create target directory if missing
    if !bin_dir.exists() {
        fs::create_dir_all(&bin_dir).expect("Failed to create bin directory");
    }

    // URL to a stable release build from John Van Sickle's static architectures
    let ffmpeg_url = "https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-amd64-static.tar.xz";

    // Use standard command line utilities natively present on Linux to download
    // This avoids adding heavy HTTP client crates like reqwest to build-dependencies
    let output = std::process::Command::new("curl")
        .args(["-sSL", "-o", "/tmp/ffmpeg_static.tar.xz", ffmpeg_url])
        .output()
        .expect("Failed to download FFmpeg static archive via curl");

    if !output.status.success() {
        panic!("Failed to download FFmpeg binary via curl: {:?}", output);
    }

    // Extract the .tar.xz archive using system tar to support xz decompression natively
    let extract_output = std::process::Command::new("tar")
        .args([
            "-xJf",
            "/tmp/ffmpeg_static.tar.xz",
            "-C",
            bin_dir.to_str().unwrap(),
            "--strip-components=1",
            "--wildcards",
            "*/ffmpeg",
        ])
        .output()
        .expect("Failed to extract static archive via tar");

    if !extract_output.status.success() {
        panic!("Failed to extract FFmpeg binary: {:?}", extract_output);
    }

    // Clean up temporary download file from disk
    let _ = fs::remove_file("/tmp/ffmpeg_static.tar.xz");

    // Double check file integrity and make it executable
    if ffmpeg_path.exists() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&ffmpeg_path).unwrap().permissions();
            perms.set_mode(0o755); // rwxr-xr-x
            fs::set_permissions(&ffmpeg_path, perms).unwrap();
        }
        println!(
            "cargo:warning=✨ Static FFmpeg successfully fetched and mapped to project context."
        );
    } else {
        panic!("FFmpeg binary missing after extraction process concluded.");
    }
}

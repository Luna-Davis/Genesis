use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=src/bundler/bootloader/src/main.rs");
    println!("cargo:rerun-if-changed=src/bundler/bootloader/Cargo.toml");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let bootloader_project_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("src/bundler/bootloader");

    // Build the bootloader
    let status = Command::new("cargo")
        .args(&["build", "--release"])
        .current_dir(&bootloader_project_dir)
        .status()
        .expect("Failed to build bootloader");

    if !status.success() {
        panic!("Bootloader build failed");
    }

    // Copy the bootloader to OUT_DIR
    let binary_name = if cfg!(windows) {
        "genesis-bootloader.exe"
    } else {
        "genesis-bootloader"
    };

    // Respect CARGO_TARGET_DIR if set (e.g., workspaces or custom target dirs)
    let target_dir = env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| bootloader_project_dir.join("target"));

    let bootloader_bin = target_dir.join("release").join(binary_name);

    let dest_path = out_dir.join("bootloader_bin");
    std::fs::copy(&bootloader_bin, &dest_path).expect("Failed to copy bootloader binary");

    // Basic sanity check so we never embed a stub/invalid bootloader (which would
    // lead to python trying to run the ZIP directly and failing with '__main__').
    let meta = std::fs::metadata(&dest_path).expect("Missing copied bootloader binary");
    if meta.len() < 1_000 {
        panic!("Bootloader binary looks invalid ({} bytes).", meta.len());
    }
}

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn genesis(args: &[&str], dir: &PathBuf) -> std::process::Output {
    Command::new("cargo")
        .arg("run")
        .arg("--")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("failed to run genesis")
}

// These tests are marked ignored because they exercise end-to-end flows that
// can touch user dirs (data_dir) and require toolchains. Run with `cargo test -- --ignored`.

#[test]
#[ignore]
fn start_and_list_flow() {
    let dir = std::env::temp_dir().join("genesis-integ-start");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    let out = genesis(&["start", "demo", "Rust"], &dir);
    assert!(out.status.success(), "start failed: {:?}", out);

    let list = genesis(&["list"], &dir);
    let stdout = String::from_utf8_lossy(&list.stdout);
    assert!(stdout.contains("demo"));
}

#[test]
#[ignore]
fn import_flow_creates_genesis_dir() {
    let dir = std::env::temp_dir().join("genesis-integ-import");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();

    let out = genesis(&["import", "--language", "Rust"], &dir);
    assert!(out.status.success(), "import failed: {:?}", out);
    assert!(dir.join(".genesis/config.toml").exists());
}

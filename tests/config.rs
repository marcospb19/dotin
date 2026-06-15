use assert_cmd::cargo::cargo_bin_cmd;
use fs_err as fs;
use tempfile::tempdir;

#[test]
fn config_init_is_accepted_as_long_flag() {
    let home = tempdir().unwrap();
    fs::create_dir(home.path().join("dotfiles")).unwrap();

    cargo_bin_cmd!("dotin")
        .env("HOME", home.path())
        .args(["config", "--init"])
        .assert()
        .success();

    assert!(home.path().join(".config/dotin/config.toml").exists());
}

#[test]
fn config_init_is_accepted_as_short_flag() {
    let home = tempdir().unwrap();
    fs::create_dir(home.path().join("dotfiles")).unwrap();

    cargo_bin_cmd!("dotin")
        .env("HOME", home.path())
        .args(["config", "-i"])
        .assert()
        .success();

    assert!(home.path().join(".config/dotin/config.toml").exists());
}

#[test]
fn config_without_init_prints_existing_config_path() {
    let home = tempdir().unwrap();
    let dotin_config_dir = home.path().join(".config/dotin");
    let config_path = dotin_config_dir.join("config.toml");

    fs::create_dir(home.path().join("dotfiles")).unwrap();
    fs::create_dir_all(&dotin_config_dir).unwrap();
    fs::write(&config_path, "").unwrap();

    let assert = cargo_bin_cmd!("dotin")
        .env("HOME", home.path())
        .arg("config")
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains(&format!("Config file set at {}", config_path.display())),
        "stdout = {stdout:?}"
    );
}

use std::path::Path;

use assert_cmd::cargo::cargo_bin_cmd;
use fs_err as fs;
use tempfile::tempdir;

#[test]
fn import_moves_file_and_links_it_back_by_default() {
    let home = tempdir().unwrap();
    fs::create_dir(home.path().join("dotfiles")).unwrap();
    fs::write(home.path().join(".zshrc"), "settings").unwrap();

    cargo_bin_cmd!("dotin")
        .env("HOME", home.path())
        .current_dir(home.path())
        .args(["import", "zsh", ".zshrc"])
        .assert()
        .success();

    let original = home.path().join(".zshrc");
    assert!(
        fs::symlink_metadata(&original)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(
        fs::read_link(&original).unwrap(),
        Path::new("dotfiles/zsh/.zshrc")
    );
    assert_eq!(
        fs::read_to_string(home.path().join("dotfiles/zsh/.zshrc")).unwrap(),
        "settings"
    );
}

#[test]
fn import_copy_retains_an_independent_source() {
    let home = tempdir().unwrap();
    fs::create_dir(home.path().join("dotfiles")).unwrap();
    fs::write(home.path().join(".zshrc"), "settings").unwrap();

    cargo_bin_cmd!("dotin")
        .env("HOME", home.path())
        .current_dir(home.path())
        .args(["import", "--copy", "zsh", ".zshrc"])
        .assert()
        .success();

    let original = home.path().join(".zshrc");
    let copied = home.path().join("dotfiles/zsh/.zshrc");
    assert!(
        fs::symlink_metadata(&original)
            .unwrap()
            .file_type()
            .is_file()
    );
    assert!(fs::symlink_metadata(&copied).unwrap().file_type().is_file());

    fs::write(&copied, "changed copy").unwrap();
    assert_eq!(fs::read_to_string(&original).unwrap(), "settings");
    assert_eq!(fs::read_to_string(&copied).unwrap(), "changed copy");
}

#[test]
fn import_help_documents_copy_and_rejects_removed_no_link_flag() {
    let help_home = tempdir().unwrap();
    fs::create_dir(help_home.path().join("dotfiles")).unwrap();
    let help = cargo_bin_cmd!("dotin")
        .env("HOME", help_home.path())
        .args(["import", "--help"])
        .output()
        .unwrap();
    assert!(help.status.success());
    let stdout = String::from_utf8_lossy(&help.stdout);
    assert!(stdout.contains("--copy"), "stdout = {stdout:?}");
    assert!(!stdout.contains("--no-link"), "stdout = {stdout:?}");

    let home = tempdir().unwrap();
    fs::create_dir(home.path().join("dotfiles")).unwrap();
    fs::write(home.path().join("file"), "settings").unwrap();

    cargo_bin_cmd!("dotin")
        .env("HOME", home.path())
        .current_dir(home.path())
        .args(["import", "--no-link", "group", "file"])
        .assert()
        .failure();
}

use assert_cmd::cargo::cargo_bin_cmd;
use fs_tree::tree;
use pretty_assertions::assert_eq;

#[test]
fn test_link_all_groups() {
    let tempdir = tempfile::tempdir().unwrap();
    let test_home = tempdir.path();

    let home = tree! {
        unrelated_file
        ".config": {}
    };

    let dotfiles = tree! {
        dotfiles: {
            i3: {
                ".config": {
                    i3: {
                        config
                    }
                }
            }
            vim: {
                ".vimrc"
            }
            git: {
                ".gitconfig"
            }
        }
    };

    let expected_home = tree! {
        unrelated_file
        ".config": {
            i3: {
                config -> "../../dotfiles/i3/.config/i3/config"
            }
        }
        ".vimrc" -> "dotfiles/vim/.vimrc"
        ".gitconfig" -> "dotfiles/git/.gitconfig"
    };

    home.write_at(test_home).unwrap();
    dotfiles.write_at(test_home).unwrap();

    let mut cmd = cargo_bin_cmd!("dotin");
    cmd.args(["link", "--all"])
        .env("HOME", test_home)
        .assert()
        .success();

    let result = expected_home.symlink_read_structure_at(test_home).unwrap();
    assert_eq!(result, expected_home);
}

#[test]
fn test_import_then_discard_is_noop() {
    let tempdir = tempfile::tempdir().unwrap();
    let test_dir = tempdir.path();

    let initial_home = tree! {
        unrelated_file
        ".config": {
            nvim: {
                "init.vim"
            }
        }
        ".vimrc"
        ".gitconfig"
    };

    let dotfiles = tree! {
        dotfiles: {}
    };

    initial_home.write_at(test_dir).unwrap();
    dotfiles.write_at(test_dir).unwrap();

    // Import files
    let mut cmd = cargo_bin_cmd!("dotin");
    cmd.arg("import")
        .arg("vim")
        .arg(test_dir.join(".vimrc"))
        .arg(test_dir.join(".config/nvim"))
        .env("HOME", test_dir)
        .assert()
        .success();

    let after_import_home = tree! {
        unrelated_file
        ".config": {}
        ".gitconfig"
    };

    let after_import_dotfiles = tree! {
        dotfiles: {
            vim: {
                ".vimrc"
                ".config": {
                    nvim: {
                        "init.vim"
                    }
                }
            }
        }
    };

    let result = after_import_home
        .symlink_read_structure_at(test_dir)
        .unwrap();
    assert_eq!(result, after_import_home);

    let result = after_import_dotfiles
        .symlink_read_structure_at(test_dir)
        .unwrap();
    assert_eq!(result, after_import_dotfiles);

    // Now discard the same files back
    let mut cmd = cargo_bin_cmd!("dotin");
    cmd.arg("discard")
        .arg("vim")
        .arg(test_dir.join(".vimrc"))
        .arg(test_dir.join(".config/nvim"))
        .env("HOME", test_dir)
        .assert()
        .success();

    // Verify we're back to the original state (almost - empty folders remain)
    let final_home = tree! {
        unrelated_file
        ".config": {
            nvim: {
                "init.vim"
            }
        }
        ".vimrc"
        ".gitconfig"
    };

    // The dotfiles folder will have the empty group folder and intermediate directories
    let final_dotfiles = tree! {
        dotfiles: {
            vim: {
                ".config": {}
            }
        }
    };

    let result = final_home.symlink_read_structure_at(test_dir).unwrap();
    assert_eq!(result, final_home);

    let result = final_dotfiles.symlink_read_structure_at(test_dir).unwrap();
    assert_eq!(result, final_dotfiles);
}

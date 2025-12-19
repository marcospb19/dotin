use assert_cmd::cargo::cargo_bin_cmd;
use fs_tree::tree;
use pretty_assertions::assert_eq;

#[test]
fn test_link_all_groups() {
    let tempdir = tempfile::tempdir().unwrap();
    let test_home = tempdir.path();

    let home = tree! {
        unrelated
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
        unrelated
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
    cmd.arg("link")
        .arg("--all")
        .env("HOME", test_home)
        .assert()
        .success();

    let result = expected_home.symlink_read_structure_at(test_home).unwrap();
    assert_eq!(result, expected_home);
}

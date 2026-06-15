use std::path::Path;

use eyre::WrapErr;
use fs_err as fs;
use fs_tree::FsTree;

use crate::{Result, utils::create_relative_symlink_target_path};

pub fn unlink(base_dir: &Path, group_dir: &Path) -> Result<()> {
    let group_tree = FsTree::symlink_read_at(group_dir).wrap_err("reading dotfiles folder tree")?;

    let base_tree = group_tree
        .symlink_read_structure_at(base_dir)
        .wrap_err("Failed to read dotfiles tree at base folder")?;

    for (node, relative_path) in &base_tree {
        let Some(current_target) = node.target() else {
            continue;
        };

        let base_absolute = base_dir.join(&relative_path);
        let dotfile_absolute = group_dir.join(&relative_path);
        let symlink_target = create_relative_symlink_target_path(&base_absolute, &dotfile_absolute);

        // unlink if the link points to the expected target
        if symlink_target == current_target {
            println!("Deleting link at {base_absolute:?}");
            fs::remove_file(base_absolute).wrap_err("Failed to delete symlink")?;
        } else {
            println!(
                "ERROR: {base_absolute:?} exists but points to {current_target:?} instead of {symlink_target:?}"
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use fs_tree::tree;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::utils::test_utils::cd_to_testdir;

    #[test]
    fn test_unlink() {
        let (_dropper, test_dir) = cd_to_testdir().unwrap();

        let home = tree! {
            ".config": [
                i3: [
                    config -> "../../dotfiles/i3/.config/i3/config"
                ]
            ]
        };

        let dotfiles = tree! {
            dotfiles: [
                i3: [
                    ".config": [
                        i3: [
                            config
                        ]
                    ]
                ]
            ]
        };

        let expected_home = tree! {
            ".config": [
                i3: []
            ]
        };

        home.write_structure_at(".").unwrap();
        dotfiles.write_structure_at(".").unwrap();

        unlink(test_dir, &test_dir.join("dotfiles/i3")).unwrap();

        let result = home.symlink_read_structure_at(".").unwrap();
        assert_eq!(result, expected_home);
    }

    #[test]
    fn test_unlink_with_override_base_folder() {
        let (_dropper, test_dir) = cd_to_testdir().unwrap();
        let base_dir = test_dir.join("base");

        let base = tree! {
            base: [
                config: [
                    theme_conf -> "../../dotfiles/sddm/config/theme_conf"
                ]
            ]
        };
        let dotfiles = tree! {
            dotfiles: [
                sddm: [
                    config: [
                        theme_conf
                    ]
                ]
            ]
        };
        let expected_base = tree! {
            base: [
                config: []
            ]
        };

        base.write_structure_at(".").unwrap();
        dotfiles.write_structure_at(".").unwrap();

        unlink(&base_dir, &test_dir.join("dotfiles/sddm")).unwrap();

        let result = base.symlink_read_structure_at(".").unwrap();
        assert_eq!(result, expected_base);
    }
}

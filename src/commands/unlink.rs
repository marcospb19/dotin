use std::path::Path;

use anyhow::Context;
use fs_err as fs;
use fs_tree::FsTree;

use crate::utils::create_relative_symlink_target_path;

pub fn unlink(home_dir: &Path, group_dir: &Path, group_name: &str) -> anyhow::Result<()> {
    let group_tree = FsTree::symlink_read_at(group_dir).context("reading dotfiles folder tree")?;

    let home_tree = group_tree
        .symlink_read_structure_at(home_dir)
        .context("Failed to read dotfiles tree at home directory")?;

    for (node, relative_path) in home_tree.iter() {
        let Some(current_target) = node.target() else {
            continue;
        };

        let home_absolute = home_dir.join(&relative_path);
        let symlink_target = create_relative_symlink_target_path(&relative_path, group_name);

        // unlink if the link points to the expected target
        if symlink_target == current_target {
            println!("Deleting link at {home_absolute:?}");
            fs::remove_file(home_absolute).context("Failed to delete symlink")?;
        } else {
            println!(
                "ERROR: {home_absolute:?} exists but points to {current_target:?} instead of {symlink_target:?}"
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

        unlink(test_dir, &test_dir.join("dotfiles/i3"), "i3").unwrap();

        let result = home.symlink_read_structure_at(".").unwrap();
        assert_eq!(result, expected_home);
    }
}

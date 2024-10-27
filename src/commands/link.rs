use std::path::Path;

use anyhow::Context;
use fs_err as fs;
use fs_tree::FsTree;

use crate::utils::{self, symlink_target_path};

pub fn link(home_dir: &Path, group_dir: &Path, group_name: &str) -> anyhow::Result<()> {
    let group_tree = FsTree::symlink_read_at(group_dir).context("reading dotfiles folder tree")?;

    let home_tree = group_tree
        .symlink_read_copy_at(&home_dir)
        .context("reading structured file tree at home directory")?;

    let mut intermediate_directories_linked = vec![];

    for (group_node, relative_path) in group_tree.iter() {
        // Skip children where the parent directory is already linked
        if intermediate_directories_linked
            .iter()
            .any(|intermediate_dir| relative_path.starts_with(intermediate_dir))
        {
            continue;
        }

        // for the current file, get its absolute path in the dotfiles folder
        // and where it's expected to be in the home directory
        let group_absolute = group_dir.join(&relative_path);
        let home_absolute = home_dir.join(&relative_path);

        let symlink_target = symlink_target_path(&relative_path, group_name);

        // if already exists at home
        if let Some(home_node) = home_tree.get(&relative_path) {
            if group_node.is_leaf() {
                if let Some(current_target) = home_node.target() {
                    if current_target == symlink_target {
                        println!("OK: skipping link {home_absolute:?}");
                        if group_node.is_dir() {
                            intermediate_directories_linked.push(relative_path);
                        }
                    } else {
                        println!(
                            "ERROR: {home_absolute:?} exists but points to {current_target:?} instead of {group_absolute:?}"
                        );
                    }
                } else {
                    println!(
                        "ERROR: can't create link at {home_absolute:?} because a {} already exists",
                        home_node.variant_str(),
                    );
                }
            } else if home_node.is_dir() {
                // great! directory found where non-leaf was expected, no need to create one
            } else {
                println!("ERROR: can't create link at {home_absolute:?} because it's a directory");
            }
        } else {
            // only link the leaves, non-leafs are created like `mkdir`
            // (note: a non-leaf is a dir, but a dir can be a leaf)
            if group_node.is_leaf() {
                utils::create_symlink(&home_absolute, &symlink_target)?;
                println!("Linked {} at {relative_path:?}", group_node.variant_str());
            } else {
                fs::create_dir(&home_absolute).context("creating directory for dotfile")?;
                println!("Created intermediate directory at {home_absolute:?}");
            }
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
    fn test_link() {
        // Arrange
        let (_dropper, test_dir) = cd_to_testdir().unwrap();

        let home = tree! {
            ".config": {
            }
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
            }
        };

        let expected_home = tree! {
            ".config": {
                i3: {
                    config -> "../../dotfiles/i3/.config/i3/config"
                }
            }
        };

        home.write_at(".").unwrap();
        dotfiles.write_at(".").unwrap();

        // Act
        link(test_dir, &test_dir.join("dotfiles/i3"), "i3").unwrap();

        // Assert
        let result = expected_home.symlink_read_copy_at(".").unwrap();
        assert_eq!(result, expected_home);
    }
}

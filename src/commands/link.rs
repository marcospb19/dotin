use std::{
    iter::repeat_n,
    path::{Path, PathBuf},
};

use anyhow::Context;
use fs_err as fs;
use fs_tree::FsTree;

use crate::utils;

pub fn link(
    home_dir: impl AsRef<Path>,
    group_path: impl AsRef<Path>,
    group_name: &str,
) -> anyhow::Result<()> {
    let home_dir = home_dir.as_ref();
    let group_path = group_path.as_ref();

    let group_tree =
        FsTree::symlink_read_at(group_path).context("failed to read dotfiles folder tree")?;

    let home_tree = group_tree
        .symlink_read_copy_at(&home_dir)
        .context("failed to read structured file tree at home directory")?;

    let mut intermediate_directories_linked = vec![];

    for (dotfiles_node, relative_path) in group_tree.iter() {
        // Skip children where the parent directory is already linked
        if intermediate_directories_linked
            .iter()
            .any(|intermediate_dir| relative_path.starts_with(intermediate_dir))
        {
            continue;
        }

        // for the current file, get its absolute path in the dotfiles folder
        // and where it's expected to be in the home directory
        let dotfile_absolute = group_path.join(&relative_path);
        let home_absolute = home_dir.join(&relative_path);

        let symlink_target = {
            let nestedness = relative_path.components().count().saturating_sub(1);
            let path_out_of_nesting = repeat_n(Path::new("../"), nestedness).collect::<PathBuf>();
            path_out_of_nesting
                .join("dotfiles")
                .join(group_name)
                .join(&relative_path)
        };

        // if already exists at home
        if let Some(home_node) = home_tree.get(&relative_path) {
            if dotfiles_node.is_leaf() {
                if let Some(current_target) = home_node.target() {
                    if current_target == symlink_target {
                        println!("OK: skipping dir link at {home_absolute:?}");
                        if dotfiles_node.is_dir() {
                            intermediate_directories_linked.push(relative_path);
                        }
                    } else {
                        println!(
                            "Skipping {home_absolute:?}, it exists but points to {current_target:?} instead of {dotfile_absolute:?}"
                        );
                    }
                } else {
                    println!(
                        "Conflict: cannot create symlink at {home_absolute:?} because the file already \
                         exists, but it is a {}",
                         home_node.variant_str(),
                    );
                }
            } else if dotfiles_node.is_dir() {
                // great! directory found where non-leaf was expected, no need to create one
            } else {
                println!(
                    "Conflict: cannot create symlink at {home_absolute:?} because there's a \
                         directory at that path."
                );
            }
        } else {
            // Only leaves should be linked
            if dotfiles_node.is_leaf() {
                utils::create_symlink(&home_absolute, &symlink_target)?;
                println!(
                    "Linked {} at {relative_path:?}",
                    dotfiles_node.variant_str()
                );
            } else {
                fs::create_dir(&home_absolute).context(
                    "Failed to create intermediate directory leading up to dotfile location",
                )?;
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
        link(test_dir, test_dir.join("dotfiles/i3"), "i3").unwrap();

        // Assert
        let result = expected_home.symlink_read_copy_at(".").unwrap();
        assert_eq!(result, expected_home);
    }
}

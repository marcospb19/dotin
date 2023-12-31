use std::path::Path;

use anyhow::Context;
use fs_err as fs;
use fs_tree::FsTree;

use crate::utils;

pub fn link(
    home_dir: impl AsRef<Path>,
    dotfiles_group_folder: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let home_dir = home_dir.as_ref();
    let dotfiles_group_folder = dotfiles_group_folder.as_ref();

    let dotfiles_tree = FsTree::symlink_read_at(&dotfiles_group_folder)
        .with_context(|| format!("Failed to read dotfiles folder at {dotfiles_group_folder:?}"))?;

    let home_tree = dotfiles_tree
        .symlink_read_copy_at(&home_dir)
        .context("Failed to read dotfiles tree at home directory")?;

    let mut intermediate_directories_linked = vec![];

    for (dotfiles_node, relative_path) in dotfiles_tree.iter() {
        // Skip subsequent files children of intermediate directories already linked
        if intermediate_directories_linked
            .iter()
            .any(|intermediate_dir| relative_path.strip_prefix(intermediate_dir).is_ok())
        {
            continue;
        }

        let dotfiles_path = dotfiles_group_folder.join(&relative_path);
        let absolute_dotfiles_path = fs::canonicalize(&dotfiles_path).expect("This file exists");
        let home_path = home_dir.join(&relative_path);

        // Already exists at home
        if let Some(home_node) = home_tree.get(&relative_path) {
            if dotfiles_node.is_leaf() {
                if let Some(target) = home_node.target() {
                    if fs::canonicalize(target).is_ok_and(|path| path == absolute_dotfiles_path) {
                        println!("OK: skipping already-existing link at {home_path:?}");

                        if dotfiles_node.is_dir() {
                            intermediate_directories_linked.push(relative_path);
                        }
                    } else {
                        println!(
                            "Conflict: skipping creating link at {home_path:?}, the symlink already
                             exists, but it points to {target:?}, which is not the right location,
                             expected it to point to {dotfiles_path:?}"
                        );
                    }
                } else {
                    println!(
                        "Conflict: cannot create symlink at {home_path:?} because the file already \
                         exists, but it is a {}",
                         home_node.variant_str(),
                     );
                }
            } else {
                if !dotfiles_node.is_dir() {
                    println!(
                        "Conflict: cannot create symlink at {home_path:?} because there's a \
                         directory at that path."
                    );
                }
            }
        } else {
            // Only leaves should be linked
            if dotfiles_node.is_leaf() {
                utils::create_symlink(&home_path, &absolute_dotfiles_path)?;

                println!(
                    "Linked {} at {relative_path:?}",
                    dotfiles_node.variant_str()
                );
            } else {
                fs::create_dir(&home_path).context(
                    "Failed to create intermediate directory leading up to dotfile location",
                )?;
                println!("Created intermediate directory at {home_path:?}");
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

        let expected_home = {
            let mut tree = tree! {
                ".config": {
                    i3: {}
                }
            };

            let symlink_target = test_dir.join("dotfiles/i3/.config/i3/config");
            tree.insert(".config/i3/config", FsTree::Symlink(symlink_target.clone()));
            tree
        };

        home.write_at(".").unwrap();
        dotfiles.write_at(".").unwrap();

        // Act
        link(test_dir, test_dir.join("dotfiles/i3")).unwrap();

        // Assert
        let result = expected_home.symlink_read_copy_at(".").unwrap();
        assert_eq!(result, expected_home);
    }
}

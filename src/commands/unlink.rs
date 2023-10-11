use std::path::Path;

use anyhow::Context;
use fs_err as fs;
use fs_tree::FsTree;

pub fn unlink(
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

    let symlink_node_iter = home_tree
        .iter()
        .min_depth(1)
        .filter_map(|(node, path)| node.target().map(|target| (target, path)));

    for (home_symlink_path, relative_path) in symlink_node_iter {
        let dotfiles_path = dotfiles_group_folder.join(&relative_path);
        let absolute_dotfiles_path = fs::canonicalize(&dotfiles_path).expect("This file exists");

        let home_path = home_dir.join(&relative_path);

        // If it points to the dotfile, unlink it
        if fs::canonicalize(home_symlink_path).is_ok_and(|path| path == absolute_dotfiles_path) {
            println!("Deleting link at {home_path:?}");
            fs::remove_file(home_path).context("Failed to delete symlink")?;
        } else {
            println!(
                "Conflict: skipping the link at {home_path:?}, the symlink do exists, but it \
                 points at {home_symlink_path:?}, expected it to point at {dotfiles_path:?}"
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
        // Arrange
        let (_dropper, test_dir) = cd_to_testdir().unwrap();

        let home = {
            let mut tree = tree! {
                ".config": {
                    i3: {}
                }
            };

            let symlink_target = test_dir.join("dotfiles/i3/.config/i3/config");
            tree.insert(".config/i3/config", FsTree::Symlink(symlink_target.clone()));
            tree
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
                i3: {}
            }
        };

        home.write_at(".").unwrap();
        dotfiles.write_at(".").unwrap();

        // Act
        unlink(test_dir, test_dir.join("dotfiles/i3")).unwrap();

        // Assert
        let result = home.symlink_read_copy_at(".").unwrap();
        assert_eq!(result, expected_home);
    }
}

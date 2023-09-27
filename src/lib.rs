#![feature(lazy_cell)]

pub mod utils;

use std::path::Path;

use fs_err as fs;
use walkdir::WalkDir;

// Arguments:
// - home_dir: $HOME
// - folder_path: i3
pub fn remove_links(home_dir: &Path, dotfile_group_folder_name: &str) -> anyhow::Result<()> {
    let home_dir = fs::canonicalize(home_dir).unwrap();

    let dotfiles_dir = home_dir.join("dotfiles");

    // dotfiles/i3
    let dotfiles_group_folder = dotfiles_dir.join(dotfile_group_folder_name);
    // /home/marcospb19/dotfiles/i3
    let dotfiles_group_folder = fs::canonicalize(dotfiles_group_folder).unwrap();

    for entry in WalkDir::new(&dotfiles_group_folder) {
        let entry = entry?;

        // Path to the file in dotfiles
        // `~/dotfiles/i3/.config/i3/config`
        let dotfile_path = entry.path();

        // The relative path to the file
        // `.config/i3/config`
        let relative_path = dotfile_path.strip_prefix(&dotfiles_group_folder).unwrap();

        if relative_path == Path::new("") {
            // skip the folder itself
            continue;
        }

        // Path to the file in $HOME
        // `~/.config/i3/config`
        let home_path = home_dir.join(relative_path);

        if home_path.read_link().is_ok() {
            fs::remove_file(home_path).unwrap();
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::env;

    use fs_tree::{tree, FsTree};
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::utils::*;

    #[test]
    fn test_remove_links() {
        // Arrange
        let (_dropper, test_dir) = testdir().unwrap();
        env::set_current_dir(test_dir).unwrap();

        let home = tree! {
            ".config": {
                i3: {
                    config -> same_here
                }
            }
        };

        let dotfiles = tree! {
            dotfiles: {
                i3: {
                    ".config": {
                        i3: {
                            config -> same_here
                        }
                    }
                }
            }
        };

        home.create_at(test_dir).unwrap();
        dotfiles.create_at(test_dir).unwrap();

        // Act
        let dotfile_group_name = "i3";
        remove_links(test_dir, dotfile_group_name).unwrap();

        // Assert
        let expected = tree! {
            ".config": {
                i3: {}
            }
        };

        // Read back testdir to check the results
        let result = FsTree::from_path_symlink(test_dir).unwrap();
        // Ignore `dotfiles/`, just take a look at `.config`
        let result = &result[".config"];

        assert_eq!(result, &expected);
    }
}

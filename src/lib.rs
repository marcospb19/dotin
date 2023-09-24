#![feature(lazy_cell)]

pub mod utils;

use std::path::Path;

use fs_err as fs;
use walkdir::WalkDir;

// Arguments:
// - home_dir: ~
// - folder_path: i3
pub fn remove_links(home_dir: &Path, dotfile_group_folder_name: &Path) -> anyhow::Result<()> {
    let home_dir = fs::canonicalize(home_dir).unwrap();
    // dotfiles/i3
    let dotfiles_group_folder =
        fs::canonicalize(Path::new("dotfiles").join(dotfile_group_folder_name)).unwrap();

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
        let (_dropper, test_dir) = testdir().unwrap();
        env::set_current_dir(test_dir).unwrap();

        let mut home = tree! {
            ".config": {
                i3: {
                    config -> same_here
                }
            }
        };

        let mut dotfiles = tree! {
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

        home.make_paths_relative();
        dotfiles.make_paths_relative();

        home.create_at(test_dir).unwrap();
        dotfiles.create_at(test_dir).unwrap();

        let i3_folder = test_dir.join("dotfiles").join("i3");
        remove_links(test_dir, &i3_folder).unwrap();

        let expected = tree! {
            ".config": {
                "./i3": {}
            }
        };

        let mut result = FsTree::from_cd_symlink_path(test_dir.join(".config")).unwrap();
        result.path = ".config".into();

        assert_eq!(result, expected);
    }
}

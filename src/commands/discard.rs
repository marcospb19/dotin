use std::path::{self, Path, PathBuf};

use eyre::{WrapErr, eyre};
use fs_err::{self as fs};

use crate::{
    Result,
    utils::{FileType, PathTrie, cheap_move_with_fallback, read_file_type, try_exists},
};

#[derive(Debug)]
struct FileToDiscard {
    user_given_path: PathBuf,
    absolute_dotfile_path: PathBuf,
    equivalent_home_path: PathBuf,
    /// TODO: should we output this instead of `user_given_path`?
    #[expect(unused)]
    relative_path_piece: PathBuf,
    conflict_resolution: DiscardConflictResolution,
}

#[derive(Clone, Copy, Debug)]
enum DiscardConflictResolution {
    None,
    DeleteDir,
    DeleteSymlink,
}

fn prepare_discard_and_run_checks(
    path: &Path,
    home_dir: &Path,
    absolute_group_path: &Path,
) -> Result<FileToDiscard> {
    let absolute = path::absolute(path)?;

    // TODO: document this
    // File path must either point exactly to:
    // - A relative path pointing exactly to a file in dotfiles
    // - An absolute path pointing exactly to a file in dotfiles
    // - A relative path pointing exactly to a symlink at home dir
    // - An absolute path pointing exactly to a symlink at home dir
    // - A piece of path that, when appended like `$HOME/dotfiles/dotfile_folder/{append_here}` refers to a file
    let (relative_path_piece, absolute_dotfile_path) = if try_exists(&absolute)? {
        let stripped_home = absolute.strip_prefix(home_dir);
        let stripped_dotfiles = absolute.strip_prefix(absolute_group_path);

        if let Ok(relative) = stripped_dotfiles {
            // Lives in dotfiles, the absolute path is already what we want
            (relative.to_owned(), absolute)
        } else if let Ok(stripped) = stripped_home {
            // File found at home, user pointed directly to that, check if it exists in the dotfiles folder
            let joined = absolute_group_path.join(stripped);
            if !try_exists(&joined)? {
                return Err(eyre!("couldn't find file at {joined:?} to discard"));
            }

            (stripped.to_owned(), joined)
        } else {
            return Err(eyre!("error: given path {path:?} is outside of $HOME"));
        }
    } else {
        // Fallback to the last candidate, which is: path is a piece
        // that points to a file in the dotfiles folder if we join it
        // and read it
        let relative_path_inside_group = absolute_group_path.join(path);
        if !try_exists(&relative_path_inside_group)? {
            // TODO: any way to improve this error message? I fell like we need to
            return Err(eyre!("couldn't find {path:?} to discard it"));
        }
        (path.to_owned(), relative_path_inside_group)
    };

    let equivalent_home_path = home_dir.join(&relative_path_piece);

    let conflict_resolution = 'conflict_check: {
        // TODO: add check for conflict where `from` and `to` types mismatch
        // and just err.

        // If file already exists at home, check if it's a scenario of conflict error
        if try_exists(&equivalent_home_path)? {
            let file_type = read_file_type(&equivalent_home_path)?;

            if let (a, b) = (
                read_file_type(path)?,
                read_file_type(&equivalent_home_path)?,
            ) && a != b
                && b != FileType::Directory
            {
                return Err(eyre!(
                    "can't discard {path:?}, it conflicts with {equivalent_home_path:?}, \
                     and their types are different",
                )
                .wrap_err(format!("{path:?} has type {a}")))
                .wrap_err(format!("{equivalent_home_path:?} has type {b}"));
            }

            match file_type {
                FileType::Regular => {
                    return Err(eyre!(
                        "file at {:?} already exists, so {:?} cannot be discarded to that place",
                        equivalent_home_path,
                        path,
                    ));
                }
                FileType::Directory => {
                    // Allow discarding if there is an empty directory at the same place
                    let is_empty_dir = fs::read_dir(&equivalent_home_path)?.next().is_none();
                    if is_empty_dir {
                        break 'conflict_check DiscardConflictResolution::DeleteDir;
                    }

                    return Err(eyre!(
                        "non-empty directory at {:?} already exists, couldn't discard {:?}",
                        equivalent_home_path,
                        path,
                    ));
                }
                FileType::Symlink => {
                    let target = fs::read_link(&equivalent_home_path)?;

                    // Allow discarding into a symlink if it's pointing to the same file we're discarding
                    // (likely linked by dotin itself)
                    if let Ok(canonicalized) = fs::canonicalize(&target)
                        && canonicalized == absolute_dotfile_path
                    {
                        break 'conflict_check DiscardConflictResolution::DeleteSymlink;
                    }

                    return Err(eyre!(
                        "there is a symlink at {equivalent_home_path:?}, but it points to {target:?} and not {absolute_dotfile_path:?}"
                    ));
                }
            }
        }

        DiscardConflictResolution::None
    };

    Ok(FileToDiscard {
        user_given_path: path.to_owned(),
        absolute_dotfile_path,
        relative_path_piece,
        equivalent_home_path,
        conflict_resolution,
    })
}

pub fn discard(home_dir: &Path, absolute_group_path: &Path, paths: &[PathBuf]) -> Result<()> {
    let files_to_discard = {
        let mut files: Vec<FileToDiscard> = paths
            .iter()
            .map(|path| prepare_discard_and_run_checks(path, home_dir, absolute_group_path))
            .collect::<Result<_>>()?;

        if files.is_empty() {
            println!("No files to discard.");
            return Ok(());
        }

        // Deduplicate paths inside others
        let path_trie: PathTrie = files
            .iter()
            .map(|file| file.equivalent_home_path.as_path())
            .collect();

        files.retain(|file| !path_trie.contains_ancestor_of(&file.equivalent_home_path));
        files
    };

    if let display_files_to_discard = files_to_discard
        .iter()
        .map(|file| &file.user_given_path)
        .collect::<Vec<_>>()
    {
        println!(
            "Will discard {} files: {:#?}",
            display_files_to_discard.len(),
            display_files_to_discard,
        );
    }

    for file in files_to_discard {
        assert!(try_exists(&file.absolute_dotfile_path).is_ok());

        match file.conflict_resolution {
            DiscardConflictResolution::None => {}
            DiscardConflictResolution::DeleteDir => {
                fs::remove_dir(&file.equivalent_home_path)?;
            }
            DiscardConflictResolution::DeleteSymlink => {
                fs::remove_file(&file.equivalent_home_path)?;
            }
        }

        cheap_move_with_fallback(&file.absolute_dotfile_path, &file.equivalent_home_path)
            .wrap_err_with(|| format!("while discarding {:?}", file.user_given_path))?;
    }

    Ok(())
}

#[cfg(test)]
pub mod tests {
    use fs_tree::tree;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::utils::test_utils::cd_to_testdir;

    #[test]
    fn test_discard_fails_conflict_file_already_exists() {
        let (_dropper, test_dir) = cd_to_testdir().unwrap();
        let home = tree! {
            discarded_path
        };
        let dotfiles = tree! {
            dotfiles: [
                example_group: [
                    discarded_path
                ]
            ]
        };

        home.write_structure_at(".").unwrap();
        dotfiles.write_structure_at(".").unwrap();

        let error_message = discard(
            test_dir,
            &test_dir.join("dotfiles/example_group"),
            ["discarded_path"].map(PathBuf::from).as_slice(),
        )
        .unwrap_err()
        .to_string();

        assert!(error_message.contains("already exists, so"));
        assert!(error_message.contains("cannot be discarded to that place"));
    }

    #[test]
    fn test_discard_succeeds_conflict_with_empty_dir() {
        let (_dropper, test_dir) = cd_to_testdir().unwrap();
        let home = tree! {
            discarded_path: []
        };
        let dotfiles = tree! {
            dotfiles: [
                example_group: [
                    discarded_path
                ]
            ]
        };

        let expected_home = tree! {
            discarded_path
        };
        let expected_dotfiles = tree! {
            dotfiles: [
                example_group: []
            ]
        };

        home.write_structure_at(".").unwrap();
        dotfiles.write_structure_at(".").unwrap();

        discard(
            test_dir,
            &test_dir.join("dotfiles/example_group"),
            ["discarded_path"].map(PathBuf::from).as_slice(),
        )
        .unwrap();

        let home_result = expected_home.symlink_read_structure_at(".").unwrap();
        assert_eq!(home_result, expected_home);
        let dotfiles_result = expected_dotfiles.symlink_read_structure_at(".").unwrap();
        assert_eq!(dotfiles_result, expected_dotfiles);
    }

    #[test]
    fn test_discard_fails_conflict_with_non_empty_dir() {
        let (_dropper, test_dir) = cd_to_testdir().unwrap();
        let home = tree! {
            discarded_path: [
                some_file
            ]
        };
        let dotfiles = tree! {
            dotfiles: [
                example_group: [
                    discarded_path
                ]
            ]
        };

        home.write_structure_at(".").unwrap();
        dotfiles.write_structure_at(".").unwrap();

        let error_message = discard(
            test_dir,
            &test_dir.join("dotfiles/example_group"),
            ["discarded_path"].map(PathBuf::from).as_slice(),
        )
        .unwrap_err()
        .to_string();

        assert!(error_message.contains("non-empty directory at"));
        assert!(error_message.contains("already exists, couldn't discard"));
    }

    #[test]
    fn test_discard_succeeds_conflict_with_link_with_correct_target() {
        let (_dropper, test_dir) = cd_to_testdir().unwrap();
        let dotfiles_discarded_path_correct_target =
            test_dir.join("dotfiles/example_group/discarded_path");

        let home = tree! {
            discarded_path -> {dotfiles_discarded_path_correct_target}
        };
        let dotfiles = tree! {
            dotfiles: [
                example_group: [
                    discarded_path
                ]
            ]
        };

        let expected_home = tree! {
            discarded_path
        };
        let expected_dotfiles = tree! {
            dotfiles: [
                example_group: []
            ]
        };

        home.write_structure_at(".").unwrap();
        dotfiles.write_structure_at(".").unwrap();

        discard(
            test_dir,
            &test_dir.join("dotfiles/example_group"),
            ["discarded_path"].map(PathBuf::from).as_slice(),
        )
        .unwrap();

        let home_result = expected_home.symlink_read_structure_at(".").unwrap();
        assert_eq!(home_result, expected_home);
        let dotfiles_result = expected_dotfiles.symlink_read_structure_at(".").unwrap();
        assert_eq!(dotfiles_result, expected_dotfiles);
    }

    #[test]
    fn test_discard_fails_conflict_with_link_with_incorrect_target() {
        let (_dropper, test_dir) = cd_to_testdir().unwrap();
        let dotfiles_discarded_path_incorrect_target = test_dir.join("dotfiles/incorrect_path");

        let home = tree! {
            discarded_path -> {dotfiles_discarded_path_incorrect_target}
        };
        let dotfiles = tree! {
            dotfiles: [
                example_group: [
                    discarded_path
                ]
            ]
        };

        home.write_structure_at(".").unwrap();
        dotfiles.write_structure_at(".").unwrap();

        let error_message = discard(
            test_dir,
            &test_dir.join("dotfiles/example_group"),
            ["discarded_path"].map(PathBuf::from).as_slice(),
        )
        .unwrap_err()
        .to_string();

        assert!(error_message.contains("there is a symlink at"));
        assert!(error_message.contains("but it points to"));
    }

    #[test]
    fn test_discard_succeeds_nested_files_and_folders() {
        let (_dropper, test_dir) = cd_to_testdir().unwrap();

        let files_to_discard = [
            "move_1_full_dir",
            "partial_move_2_merging_dir/move_3",
            "partial_move_7_new_dir/move_4",
            "partial_move_7_new_dir/partial_move_8_new_dir/move_5_full_dir",
            "partial_move_7_new_dir/partial_move_8_new_dir/move_6",
        ]
        .map(PathBuf::from);

        let home = tree! {
            stays_1
            partial_move_7_new_dir: [
                partial_move_8_new_dir: [
                    stays_2
                ]
                stays_3
            ]
            partial_move_2_merging_dir: [
                stays_4
            ]
        };
        let dotfiles = tree! {
            dotfiles: [
                stays_5
                group_name: [
                    move_1_full_dir: [
                        moved_with_folder_1
                    ]
                    partial_move_7_new_dir: [
                        move_4
                        partial_move_8_new_dir: [
                            move_6
                            move_5_full_dir: [
                                moved_with_folder_5
                            ]
                        ]
                    ]
                    partial_move_2_merging_dir: [
                        moved_with_folder_4
                        move_3
                    ]
                ]
            ]
        };

        let expected_home = tree! {
            stays_1
            move_1_full_dir: [
                moved_with_folder_1
            ]
            partial_move_7_new_dir: [
                move_4
                partial_move_8_new_dir: [
                    stays_2
                    move_6
                    move_5_full_dir: [
                        moved_with_folder_5
                    ]
                ]
                stays_3
            ]
            partial_move_2_merging_dir: [
                stays_4
                move_3
            ]
        };
        let expected_dotfiles = tree! {
            dotfiles: [
                stays_5
                group_name: [
                    partial_move_2_merging_dir: [
                        moved_with_folder_4
                    ]
                ]
            ]
        };

        home.write_structure_at(".").unwrap();
        dotfiles.write_structure_at(".").unwrap();

        discard(
            test_dir,
            &test_dir.join("dotfiles/group_name"),
            &files_to_discard,
        )
        .unwrap();

        let home_result = expected_home.symlink_read_structure_at(".").unwrap();
        assert_eq!(home_result, expected_home);
        let dotfiles_result = expected_dotfiles.symlink_read_structure_at(".").unwrap();
        assert_eq!(dotfiles_result, expected_dotfiles);
    }

    #[test]
    fn test_discard_passing_file_and_its_parent() {
        let (_dropper, test_dir) = cd_to_testdir().unwrap();

        let home = tree! {};
        let dotfiles = tree! {
            dotfiles: [
                group: [
                    dir: [
                        parent: [
                            file
                        ]
                    ]
                ]
            ]
        };

        let expected_home = tree! {
            dir: [
                parent: [
                    file
                ]
            ]
        };
        let expected_dotfiles = tree! {
            dotfiles: [
                group: []
            ]
        };

        home.write_structure_at(".").unwrap();
        dotfiles.write_structure_at(".").unwrap();

        discard(
            test_dir,
            &test_dir.join("dotfiles/group"),
            ["dir/parent", "dir/parent/file"]
                .map(PathBuf::from)
                .as_slice(),
        )
        .unwrap();

        let home_result = expected_home.symlink_read_structure_at(".").unwrap();
        assert_eq!(home_result, expected_home);
        let dotfiles_result = expected_dotfiles.symlink_read_structure_at(".").unwrap();
        assert_eq!(dotfiles_result, expected_dotfiles);
    }

    #[test]
    fn test_discard_symlink_itself() {
        let (_dropper, test_dir) = cd_to_testdir().unwrap();

        let home = tree! {};
        let dotfiles = tree! {
            dotfiles: [
                group: [
                    link -> any_target
                ]
            ]
        };

        let expected_home = tree! {
            link -> any_target
        };
        let expected_dotfiles = tree! {
            dotfiles: [
                group: []
            ]
        };

        home.write_structure_at(".").unwrap();
        dotfiles.write_structure_at(".").unwrap();

        discard(
            test_dir,
            &test_dir.join("dotfiles/group"),
            ["link"].map(PathBuf::from).as_slice(),
        )
        .unwrap();

        let home_result = expected_home.symlink_read_structure_at(".").unwrap();
        assert_eq!(home_result, expected_home);
        let dotfiles_result = expected_dotfiles.symlink_read_structure_at(".").unwrap();
        assert_eq!(dotfiles_result, expected_dotfiles);
    }
}

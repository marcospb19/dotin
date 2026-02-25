use std::{
    io::{self, BufRead, BufReader, Read},
    path::{self, Path, PathBuf},
};

use eyre::{WrapErr, eyre};
use fs_err::{self as fs};

use crate::{
    Result,
    utils::{self, FileType, cheap_move_with_fallback, read_file_type, try_exists},
};

#[derive(Debug)]
struct FileToMove<'a> {
    path: &'a Path,
    to_path: PathBuf,
    conflict_resolution: ImportConflictResolution,
}

// TODO: unify conflict resolution of import with discard, reuse some of the code
#[derive(Clone, Copy, Debug)]
enum ImportConflictResolution {
    None,
    DeleteRegularFile,
    DeleteDir,
    SkipThis,
}

pub fn import(home_path: &Path, absolute_group_path: &Path, files: &[PathBuf]) -> Result<()> {
    let dotfiles_folder = absolute_group_path
        .parent()
        .expect("Internal error, malformed dotfiles folder");

    let absolute_paths: Vec<PathBuf> = files
        .iter()
        .map(path::absolute)
        .collect::<io::Result<_>>()?;

    let files_to_move = {
        let mut files_to_move: Vec<FileToMove> = vec![];

        for (absolute_path, path) in absolute_paths.iter().zip(files) {
            let file_type = read_file_type(path)?;

            // Is file inside of `dotfiles_folder`? Skip it.
            if let Ok(normalized_path) = absolute_path.strip_prefix(dotfiles_folder) {
                if let FileType::Symlink = file_type {
                    println!(
                        "Skipping {path:?}, it's already a symlink, and it points to \
                         {normalized_path:?}, which is inside of the dotfiles directory."
                    );
                } else {
                    println!("Skipping {path:?} because it lives inside of the dotfiles directory");
                }
                continue;
            }

            // If the file is itself a symlink.
            if let FileType::Symlink = file_type {
                println!(
                    "ERROR: the file you're trying to move {path:?} is a symlink itself, I'm not quite sure if you really meant to move it to the group folder, please handle it manually"
                );
            }

            // Is file inside of `home_path`? If not, throw error.
            if let Ok(normalized_path) = absolute_path.strip_prefix(home_path) {
                let to_path = absolute_group_path.join(normalized_path);

                let conflict_resolution = check_conflict_resolution(path, &to_path)?;

                let file = FileToMove {
                    path,
                    to_path,
                    conflict_resolution,
                };
                files_to_move.push(file);
            } else {
                return Err(eyre!(
                    "`dotin` can only import files inside of home directory {home_path:?}, \
                     but {path:?} seems to be outside of it."
                ));
            }
        }

        files_to_move
    };

    if files_to_move.is_empty() {
        println!("No files to move.");
    }

    utils::create_folder_at(absolute_group_path).wrap_err("create folder for group")?;

    let mut intermediate_directories_to_create = vec![];

    for FileToMove { to_path, .. } in &files_to_move {
        let parent_directory = to_path.parent().unwrap();

        if try_exists(parent_directory)? {
            assert!(
                parent_directory.is_dir(),
                "Cannot create file at {parent_directory:?}, there's a file there.",
            );
        } else if parent_directory != absolute_group_path {
            intermediate_directories_to_create.push(parent_directory);
        }
    }

    if !intermediate_directories_to_create.is_empty() {
        utils::deduplicate_paths_inside_others(&mut intermediate_directories_to_create);

        println!(
            "Creating {} intermediate directories: {intermediate_directories_to_create:#?}",
            intermediate_directories_to_create.len(),
        );

        for dir in &intermediate_directories_to_create {
            fs::create_dir_all(dir).wrap_err("Failed to create intermediate directory")?;
        }
    }

    println!(
        "Will move {} files: {files_to_move:#?}",
        files_to_move.len(),
    );

    // Finally move them
    for FileToMove {
        path,
        to_path,
        conflict_resolution,
    } in &files_to_move
    {
        match conflict_resolution {
            ImportConflictResolution::None => {}
            ImportConflictResolution::DeleteRegularFile => {
                fs::remove_file(to_path)?;
            }
            ImportConflictResolution::DeleteDir => {
                fs::remove_dir(to_path)?;
            }
            ImportConflictResolution::SkipThis => {
                continue;
            }
        }
        cheap_move_with_fallback(path, to_path).wrap_err("Failed to move file to import")?;
    }

    Ok(())
}

fn check_conflict_resolution(from: &Path, to: &Path) -> Result<ImportConflictResolution> {
    if !try_exists(to)? {
        return Ok(ImportConflictResolution::None);
    }

    let (type_from, type_to) = (read_file_type(from)?, read_file_type(to)?);

    use FileType::*;
    let conflict_resolution = match (type_from, type_to) {
        (_, Regular) if fs::symlink_metadata(to)?.len() == 0 => {
            ImportConflictResolution::DeleteRegularFile
        }
        (_, Directory) if fs::read_dir(to)?.next().is_none() => ImportConflictResolution::DeleteDir,
        (Regular, Regular) => {
            ensure_files_match_content(from, to)?;
            ImportConflictResolution::SkipThis
        }
        (Directory, Directory) => {
            return Err(eyre!(
                "can't import {:?}, there is a non-empty directory at {:?}",
                from,
                to,
            ));
        }
        (Symlink, Symlink) => {
            ensure_symlinks_match_target(from, to)?;
            ImportConflictResolution::SkipThis
        }
        (Regular, Directory)
        | (Regular, Symlink)
        | (Directory, Regular)
        | (Directory, Symlink)
        | (Symlink, Directory)
        | (Symlink, Regular) => {
            return Err(eyre!(
                "can't import {:?}, it conflicts with {:?}, and their types \
                are different",
                from,
                to,
            ));
        }
    };

    Ok(conflict_resolution)
}

fn ensure_files_match_content(from_path: &Path, to_path: &Path) -> Result<()> {
    let from = fs::File::open(from_path)?;
    let to = fs::File::open(to_path)?;

    let from_len = from.metadata()?.len();
    let to_len = to.metadata()?.len();

    fn content_match(a: impl Read, b: impl Read) -> io::Result<bool> {
        let mut a = BufReader::new(a);
        let mut b = BufReader::new(b);

        loop {
            let slice_a = a.fill_buf()?;
            let slice_b = b.fill_buf()?;
            let len_a = slice_a.len();
            let len_b = slice_b.len();

            if len_a == 0 || len_b == 0 {
                assert_eq!(len_a, len_b, "should check len before, or arithmetic bug");
                return Ok(true);
            }

            let min = len_a.min(len_b);

            if slice_a[..min] != slice_b[..min] {
                return Ok(false);
            }

            a.consume(min);
            b.consume(min);
        }
    }

    if from_len != to_len || !content_match(from, to)? {
        return Err(eyre!(
            "can't import {from_path:?}, it conflicts with {to_path:?}, and their content is different",
        ));
    }
    Ok(())
}

fn ensure_symlinks_match_target(from_path: &Path, to_path: &Path) -> Result<()> {
    assert_eq!(FileType::Symlink, read_file_type(from_path)?);
    assert_eq!(FileType::Symlink, read_file_type(to_path)?);
    if fs::read_link(from_path)? != fs::read_link(to_path)? {
        return Err(eyre!(
            "can't import {from_path:?}, it conflicts with {to_path:?}, they're both symlinks but their targets are different",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{thread::sleep, time::Duration};

    use fs_tree::{FsTree, tree};
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::utils::test_utils::cd_to_testdir;

    #[test]
    fn test_import() {
        let (_dropper, test_dir) = cd_to_testdir().unwrap();

        let files_to_import = [
            "move_1_full_dir",
            "partial_move_2_merging_dir/move_3",
            "partial_move_7_new_dir/move_4",
            "partial_move_7_new_dir/partial_move_8_new_dir/move_5_full_dir",
            "partial_move_7_new_dir/partial_move_8_new_dir/move_6",
        ]
        .map(PathBuf::from);

        let home = tree! {
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

        let expected_home = tree! {
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
                    partial_move_2_merging_dir: [
                        moved_with_folder_4
                    ]
                ]
            ]
        };

        let expected_dotfiles = tree! {
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

        home.write_structure_at(".").unwrap();
        dotfiles.write_structure_at(".").unwrap();

        import(
            test_dir,
            &test_dir.join("dotfiles/group_name"),
            &files_to_import,
        )
        .unwrap();

        let home_result = expected_home.symlink_read_structure_at(".").unwrap();
        assert_eq!(home_result, expected_home);
        let dotfiles_result = expected_dotfiles.symlink_read_structure_at(".").unwrap();
        assert_eq!(dotfiles_result, expected_dotfiles);
    }

    #[test]
    fn test_import_symlink_itself() {
        let (_dropper, test_dir) = cd_to_testdir().unwrap();

        let home = tree! {
            link -> any_target
        };
        let dotfiles = tree! {
            dotfiles: [
                group: []
            ]
        };

        let expected_home = tree! {};
        let expected_dotfiles = tree! {
            dotfiles: [
                group: [
                    link -> any_target
                ]
            ]
        };

        home.write_structure_at(".").unwrap();
        dotfiles.write_structure_at(".").unwrap();

        import(
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

    #[test]
    fn test_import_fails_with_conflict_regular_file_different_contents() {
        let (_dropper, test_dir) = cd_to_testdir().unwrap();

        let home = tree! {
            file
        };
        let dotfiles = tree! {
            dotfiles: [
                group: [
                    file
                ]
            ]
        };

        home.write_structure_at(".").unwrap();
        dotfiles.write_structure_at(".").unwrap();

        // Importing should fail cause the two files have different contents
        fs::write(test_dir.join("file"), "aaa").unwrap();
        fs::write(test_dir.join("dotfiles/group/file"), "bbb").unwrap();

        let error_message = import(
            test_dir,
            &test_dir.join("dotfiles/group"),
            ["file"].map(PathBuf::from).as_slice(),
        )
        .unwrap_err()
        .to_string();

        assert!(
            error_message.contains("it conflicts with"),
            "msg = {error_message}",
        );
        assert!(
            error_message.contains("and their content is different"),
            "msg = {error_message}",
        );
    }

    #[test]
    fn test_import_succeed_with_conflict_regular_file_same_contents() {
        let (_dropper, test_dir) = cd_to_testdir().unwrap();

        let home = tree! {
            file
        };
        let dotfiles = tree! {
            dotfiles: [
                group: [
                    file
                ]
            ]
        };

        let expected_home = tree! {};
        let expected_dotfiles = dotfiles.clone();

        home.write_structure_at(".").unwrap();
        dotfiles.write_structure_at(".").unwrap();
        // Importing should succeed cause these have the same content
        fs::write(test_dir.join("file"), "aaa").unwrap();
        fs::write(test_dir.join("dotfiles/group/file"), "aaa").unwrap();

        import(
            test_dir,
            &test_dir.join("dotfiles/group"),
            ["file"].map(PathBuf::from).as_slice(),
        )
        .unwrap();

        let home_result = expected_home.symlink_read_structure_at(".").unwrap();
        assert_eq!(home_result, expected_home);
        let dotfiles_result = expected_dotfiles.symlink_read_structure_at(".").unwrap();
        assert_eq!(dotfiles_result, expected_dotfiles);
    }

    #[test]
    fn test_import_fails_with_conflict_directory_non_empty() {
        let (_dropper, test_dir) = cd_to_testdir().unwrap();

        let home = tree! {
            dir: [
                any_file_1
            ]
        };
        let dotfiles = tree! {
            dotfiles: [
                group: [
                    dir: [
                        any_file_2
                    ]
                ]
            ]
        };

        home.write_structure_at(".").unwrap();
        dotfiles.write_structure_at(".").unwrap();

        let error_message = import(
            test_dir,
            &test_dir.join("dotfiles/group"),
            ["dir"].map(PathBuf::from).as_slice(),
        )
        .unwrap_err()
        .to_string();

        assert!(
            error_message.contains("there is a non-empty directory at"),
            "msg = {error_message}",
        );
    }

    #[test]
    fn test_import_fails_with_conflict_symlink_target_mismatch() {
        let (_dropper, test_dir) = cd_to_testdir().unwrap();

        let home = tree! {
            link -> target1
        };
        let dotfiles = tree! {
            dotfiles: [
                group: [
                    link -> target2
                ]
            ]
        };

        home.write_structure_at(".").unwrap();
        dotfiles.write_structure_at(".").unwrap();

        let error_message = import(
            test_dir,
            &test_dir.join("dotfiles/group"),
            ["link"].map(PathBuf::from).as_slice(),
        )
        .unwrap_err()
        .to_string();

        assert!(
            error_message.contains("it conflicts with"),
            "msg = {error_message}",
        );
        assert!(
            error_message.contains("they're both symlinks but their targets are different"),
            "msg = {error_message}",
        );
    }

    #[test]
    fn test_import_succeeds_with_conflict_symlink_target_match() {
        let (_dropper, test_dir) = cd_to_testdir().unwrap();

        let home = tree! {
            link -> target
        };
        let dotfiles = tree! {
            dotfiles: [
                group: [
                    link -> target
                ]
            ]
        };

        let expected_home = tree! {};
        let expected_dotfiles = dotfiles.clone();

        home.write_structure_at(".").unwrap();
        dotfiles.write_structure_at(".").unwrap();

        let read_file_modify_time = || {
            fs::symlink_metadata(test_dir.join("dotfiles/group/link"))
                .unwrap()
                .modified()
                .unwrap()
        };
        let modify_time = read_file_modify_time();

        // Give it enough time for the modified filesystem time to be different
        sleep(Duration::from_millis(5));

        import(
            test_dir,
            &test_dir.join("dotfiles/group"),
            ["link"].map(PathBuf::from).as_slice(),
        )
        .unwrap();

        let home_result = expected_home.symlink_read_structure_at(".").unwrap();
        assert_eq!(home_result, expected_home);
        let dotfiles_result = expected_dotfiles.symlink_read_structure_at(".").unwrap();
        assert_eq!(dotfiles_result, expected_dotfiles);

        assert_eq!(
            modify_time,
            read_file_modify_time(),
            "link shouldn't be touched again",
        );
    }

    fn conflict_test_helper_gen_trees_all_file_types() -> Vec<FsTree> {
        vec![
            tree! {
                name
            },
            tree! {
                name -> target
            },
            tree! {
                name: [
                    inner
                ]
            },
        ]
    }

    #[test]
    fn test_import_succeed_with_conflict_directory_empty() {
        let homes = conflict_test_helper_gen_trees_all_file_types();

        for home in homes {
            eprintln!("last home: {home:?}");
            let (_dropper, test_dir) = cd_to_testdir().unwrap();

            let dotfiles = tree! {
                dotfiles: [
                    group: [
                        name: []
                    ]
                ]
            };

            let expected_home = tree! {};
            let expected_dotfiles = {
                let mut tree = dotfiles.clone();
                // overwrite name by what will be imported from home
                tree.insert("dotfiles/group", home.clone());
                tree
            };

            home.write_structure_at(".").unwrap();
            dotfiles.write_structure_at(".").unwrap();

            import(
                test_dir,
                &test_dir.join("dotfiles/group"),
                ["name"].map(PathBuf::from).as_slice(),
            )
            .unwrap();

            let home_result = expected_home.symlink_read_structure_at(".").unwrap();
            assert_eq!(home_result, expected_home);
            let dotfiles_result = expected_dotfiles.symlink_read_structure_at(".").unwrap();
            assert_eq!(dotfiles_result, expected_dotfiles);
        }
    }

    #[test]
    fn test_import_succeed_with_conflict_regular_file_empty() {
        let homes = conflict_test_helper_gen_trees_all_file_types();

        for home in homes {
            eprintln!("last home: {home:?}");
            let (_dropper, test_dir) = cd_to_testdir().unwrap();

            let dotfiles = tree! {
                dotfiles: [
                    group: [
                        name
                    ]
                ]
            };

            let expected_home = tree! {};
            let expected_dotfiles = {
                let mut tree = dotfiles.clone();
                // overwrite name by what will be imported from home
                tree.insert("dotfiles/group", home.clone());
                tree
            };

            home.write_structure_at(".").unwrap();
            dotfiles.write_structure_at(".").unwrap();

            import(
                test_dir,
                &test_dir.join("dotfiles/group"),
                ["name"].map(PathBuf::from).as_slice(),
            )
            .unwrap();

            let home_result = expected_home.symlink_read_structure_at(".").unwrap();
            assert_eq!(home_result, expected_home);
            let dotfiles_result = expected_dotfiles.symlink_read_structure_at(".").unwrap();
            assert_eq!(dotfiles_result, expected_dotfiles);
        }
    }
}

use std::{
    io,
    path::{self, Path, PathBuf},
};

use anyhow::{Context, bail};
use fs_err::{self as fs};

use crate::utils::{
    self, FileToMove, FileType, cheap_move_with_fallback, read_file_type, try_exists,
};

pub fn import(
    home_path: &Path,
    absolute_group_path: &Path,
    files: &[PathBuf],
) -> anyhow::Result<()> {
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

                let file = FileToMove { path, to_path };
                files_to_move.push(file);
            } else {
                bail!(
                    "`dotin` can only import files inside of home directory {home_path:?}, \
                     but {path:?} seems to be outside of it."
                );
            }
        }

        files_to_move
    };

    if files_to_move.is_empty() {
        println!("No files to move.");
    }

    utils::create_folder_at(absolute_group_path).context("create folder for group")?;

    for FileToMove { to_path, .. } in &files_to_move {
        // TODO: this isn't considering symlinks
        // TODO: what if these two files match, is it a conflict?
        if try_exists(to_path)? {
            panic!("File at {to_path:?} already exists, can't import this");
        }
    }

    let mut intermediate_directories_to_create = vec![];

    for FileToMove { to_path, .. } in &files_to_move {
        let parent_directory = to_path.parent().unwrap();

        if try_exists(parent_directory)? {
            if !parent_directory.is_dir() {
                panic!("Cannot create file at {parent_directory:?}, there's a file there.");
            }
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
            fs::create_dir_all(dir).context("Failed to create intermediate directory")?;
        }
    }

    println!(
        "Will move {} files: {files_to_move:#?}",
        files_to_move.len(),
    );

    // Finally move them
    for FileToMove { path, to_path } in &files_to_move {
        cheap_move_with_fallback(path, to_path).context("Failed to move file to import")?;
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

        home.write_at(".").unwrap();
        dotfiles.write_at(".").unwrap();

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

        home.write_at(".").unwrap();
        dotfiles.write_at(".").unwrap();

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
}

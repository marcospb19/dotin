use std::{
    io,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context};
use file_type_enum::FileType;
use fs_err as fs;

use crate::utils::{self, FileToMove};

pub fn import(home_dir: &Path, group_dir: &Path, files: &[PathBuf]) -> anyhow::Result<()> {
    assert!(
        !files.is_empty(),
        "`dotin import` file list cannot be empty, this should be ensured by `cli` definitions",
    );

    let dotfiles_folder = group_dir
        .parent()
        .expect("Internal error, malformed dotfiles folder");

    let absolute_paths: Vec<PathBuf> = files
        .iter()
        .map(fs::canonicalize)
        .collect::<io::Result<_>>()?;

    let files_to_move = {
        let mut files_to_move: Vec<FileToMove> = vec![];

        for (absolute_path, path) in absolute_paths.iter().zip(files) {
            let is_file_symlink =
                FileType::symlink_read_at(path).is_ok_and(|file_type| file_type.is_symlink());

            // Is file inside of `dotfiles_folder`? Skip it.
            if let Ok(normalized_path) = absolute_path.strip_prefix(dotfiles_folder) {
                if is_file_symlink {
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
            if is_file_symlink {
                println!("ERROR: the file you're trying to move {path:?} is a symlink itself, I'm not quite sure if you really meant to move it to the group folder, please handle it manually");
            }

            // Is file inside of `home_dir`? If not, throw error.
            if let Ok(normalized_path) = absolute_path.strip_prefix(home_dir) {
                let to_path = group_dir.join(normalized_path);

                let file = FileToMove { path, to_path };
                files_to_move.push(file);
            } else {
                bail!(
                    "`dotin` can only import files inside of home directory {home_dir:?}, \
                     but {path:?} seems to be outside of it."
                );
            }
        }

        files_to_move
    };

    if files_to_move.is_empty() {
        println!("No files to move.");
    }

    utils::create_folder_at(group_dir).context("create folder for group")?;

    for FileToMove { to_path, .. } in &files_to_move {
        // Check if files at destination already exist
        // TODO: this isn't considering symlinks
        if to_path.exists() {
            panic!("File at {to_path:?} already exists, and cannot be imported");
        }
    }

    let mut intermediate_directories_to_create = vec![];

    // Check if files at destination already exist
    for FileToMove { to_path, .. } in &files_to_move {
        let parent_directory = to_path.parent().unwrap();

        if parent_directory.exists() {
            if !parent_directory.is_dir() {
                panic!("Cannot create file at {parent_directory:?}, there's a file there.");
            }
        } else if parent_directory != group_dir {
            intermediate_directories_to_create.push(parent_directory);
        }
    }

    if !intermediate_directories_to_create.is_empty() {
        utils::dedup_nested(&mut intermediate_directories_to_create);
        intermediate_directories_to_create.sort();

        println!(
            "Will create {} intermediate directories: {intermediate_directories_to_create:#?}",
            intermediate_directories_to_create.len(),
        );

        for dir in &intermediate_directories_to_create {
            fs::create_dir_all(dir).context("Failed to create intermediate directory")?;
        }

        println!("Done.");
        println!();
    }

    // Check if files at destination already exist
    for FileToMove { path, to_path } in &files_to_move {
        let parent_directory = to_path.parent().unwrap();

        // Check if file cannot be moved
        if !utils::are_in_the_same_filesystem(path, parent_directory)? {
            bail!(
                "Cannot move file {path:?} to folder {parent_directory:?} because they're \
                not in the same filesystem"
            );
        }
    }

    println!(
        "Will move {} files: {files_to_move:#?}",
        files_to_move.len(),
    );

    // Check if files at destination already exist
    for FileToMove { path, to_path } in &files_to_move {
        fs::rename(path, to_path).context("Failed to move file")?;
    }

    println!("Done.");

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
}

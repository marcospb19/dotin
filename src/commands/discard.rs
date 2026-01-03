use std::path::{Path, PathBuf};

use anyhow::{bail, Context};
use fs_err as fs;

use crate::utils::{self, FileToMove};

pub fn discard(home_dir: &Path, group_dir: &Path, files: &[PathBuf]) -> anyhow::Result<()> {
    // TODO:  ja comecou errado, ele por algum motivo achou que esses arquivos n existem pq deu erro na hora de canonicalizar, mas na vdd o problema deve ta no path que eh passado no teste, que agora tem que refletir o real path dos arquivos dentro do dotfiles/GROUP folder

    // Convert paths to absolute (but don't canonicalize since they don't exist yet)
    let absolute_paths: Vec<PathBuf> = files
        .iter()
        .map(|p| {
            if p.is_absolute() {
                p.clone()
            } else {
                std::env::current_dir().unwrap().join(p)
            }
        })
        .collect();

    let files_to_move = {
        let mut files_to_move: Vec<FileToMove> = vec![];

        for (absolute_path, path) in absolute_paths.iter().zip(files) {
            // Is file inside of `home_dir`? If not, throw error.
            // The paths represent where files should be restored to in home.
            if let Ok(normalized_path) = absolute_path.strip_prefix(home_dir) {
                // Check if source file exists in group_dir
                let source_path = group_dir.join(normalized_path);
                if !source_path.exists() {
                    bail!(
                        "Cannot discard {path:?}: source file {source_path:?} does not exist in group directory"
                    );
                }

                let file = FileToMove {
                    path: normalized_path,
                    to_path: absolute_path.clone(),
                };
                files_to_move.push(file);
            } else {
                bail!(
                    "`dotin discard` expects paths relative to home directory {home_dir:?}, \
                     but {path:?} seems to be outside of it."
                );
            }
        }

        files_to_move
    };

    if files_to_move.is_empty() {
        println!("No files to move.");
    }

    for FileToMove { to_path, .. } in &files_to_move {
        // Check if files at destination already exist
        if to_path.exists() {
            panic!("File at {to_path:?} already exists, and cannot be discarded there");
        }
    }

    let mut intermediate_directories_to_create = vec![];

    for FileToMove { to_path, .. } in &files_to_move {
        let parent_directory = to_path.parent().unwrap();

        if parent_directory.exists() {
            if !parent_directory.is_dir() {
                panic!("Cannot create file at {parent_directory:?}, there's a file there.");
            }
        } else if parent_directory != home_dir {
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

    // Check if files can be moved (same filesystem)
    for FileToMove { path, to_path } in &files_to_move {
        let source_path = group_dir.join(path);
        let parent_directory = to_path.parent().unwrap();

        if !utils::are_in_the_same_filesystem(&source_path, parent_directory)? {
            bail!(
                "Cannot move file {source_path:?} to folder {parent_directory:?} because they're \
                not in the same filesystem"
            );
        }
    }

    println!(
        "Will move {} files: {files_to_move:#?}",
        files_to_move.len(),
    );

    // Move files from group_dir to home_dir
    for FileToMove { path, to_path } in &files_to_move {
        let source_path = group_dir.join(path);
        fs::rename(&source_path, to_path).context("Failed to move file")?;
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
    fn test_discard() {
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
            partial_move_7_new_dir: {
                partial_move_8_new_dir: {
                    stays_2
                }
                stays_3
            }
            partial_move_2_merging_dir: {
                stays_4
            }
        };

        let expected_home = tree! {
            stays_1
            move_1_full_dir: {
                moved_with_folder_1
            }
            partial_move_7_new_dir: {
                move_4
                partial_move_8_new_dir: {
                    stays_2
                    move_6
                    move_5_full_dir: {
                        moved_with_folder_5
                    }
                }
                stays_3
            }
            partial_move_2_merging_dir: {
                stays_4
                move_3
            }
        };

        let dotfiles = tree! {
            dotfiles: {
                dotfiles_untouched_file
                group_name: {
                    move_1_full_dir: {
                        moved_with_folder_1
                    }
                    partial_move_7_new_dir: {
                        move_4
                        partial_move_8_new_dir: {
                            move_6
                            move_5_full_dir: {
                                moved_with_folder_5
                            }
                        }
                    }
                    partial_move_2_merging_dir: {
                        moved_with_folder_4
                        move_3
                    }
                }
            }
        };

        let expected_dotfiles = tree! {
            dotfiles: {
                dotfiles_untouched_file
                group_name: {
                    partial_move_2_merging_dir: {
                        moved_with_folder_4
                    }
                }
            }
        };

        dotfiles.write_at(".").unwrap();
        home.write_at(".").unwrap();

        discard(
            test_dir,
            &test_dir.join("dotfiles/group_name"),
            &files_to_discard,
        )
        .unwrap();

        let dotfiles_result = expected_dotfiles.symlink_read_structure_at(".").unwrap();
        assert_eq!(dotfiles_result, expected_dotfiles);
        let home_result = expected_home.symlink_read_structure_at(".").unwrap();
        assert_eq!(home_result, expected_home);
    }
}

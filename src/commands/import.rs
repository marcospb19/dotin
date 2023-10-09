use std::path::{Path, PathBuf};

use anyhow::{bail, Context};
use fs_err as fs;

use crate::utils::create_folder_at;

#[derive(Debug)]
struct FileToImport<'a> {
    path: &'a Path,
    absolute_path: &'a Path,
    normalized_path: &'a Path,
}

pub fn import(
    home_dir: impl AsRef<Path>,
    dotfiles_group_folder: impl AsRef<Path>,
    files: Vec<PathBuf>,
) -> anyhow::Result<()> {
    assert!(!files.is_empty());

    let dotfiles_group_folder = dotfiles_group_folder.as_ref();

    create_folder_at(dotfiles_group_folder).with_context(|| {
        format!("Failed to create folder for the group {dotfiles_group_folder:?}")
    })?;

    let absolute_paths: Vec<PathBuf> = files
        .iter()
        .map(|path| {
            fs::canonicalize(path).with_context(|| format!("Failed to read path at {path:?}"))
        })
        .collect::<anyhow::Result<_>>()?;

    let mut files_to_create: Vec<FileToImport> = vec![];

    for (absolute_path, path) in absolute_paths.iter().zip(&files) {
        if let Ok(dotfiles_normalized_path) = absolute_path.strip_prefix(dotfiles_group_folder) {
            println!("Skipping {path:?} as it's already a symlink to {dotfiles_normalized_path:?}");
            continue;
        }

        // Normalized paths should only be OK if they're inside of the home dir
        let normalized_path = absolute_path.strip_prefix(&home_dir);

        if let Ok(normalized_path) = normalized_path {
            let file = FileToImport {
                path,
                absolute_path,
                normalized_path,
            };
            files_to_create.push(file);
        } else {
            bail!(
                "For now, `dotin` can only import files inside of the $HOME \
                 directory, but {absolute_path:?} is not inside of it"
            );
        }
    }

    println!("Will create {} files:", files_to_create.len());
    println!("{files_to_create:#?}");

    // let dotfiles_tree = FsTree::symlink_read_at(&dotfiles_group_folder)
    //     .with_context(|| format!("Failed to read dotfiles folder at {dotfiles_group_folder:?}"))?;

    for path in files_to_create {
        println!("create file {path:?}");
    }

    Ok(())
}

use std::{
    env, io,
    os::unix::fs::{symlink, MetadataExt},
    path::{Path, PathBuf},
};

use anyhow::{bail, Context};
use file_type_enum::FileType;
use fs_err as fs;

pub fn get_home_dir() -> anyhow::Result<PathBuf> {
    let home_env_var =
        env::var_os("HOME").context("Failed to read user's home directory, try setting $HOME")?;

    fs::canonicalize(&*home_env_var).context("Failed to read path at $HOME")
}

/// Creates a symlink at `link_location` that points to `original`.
pub fn create_symlink(link_location: &Path, original: &Path) -> anyhow::Result<()> {
    symlink(original, link_location).with_context(|| {
        format!("Failed to create symlink at {link_location:?} pointing to {original:?}")
    })
}

pub fn create_folder_at(folder_path: &Path) -> anyhow::Result<()> {
    let file_type = FileType::symlink_read_at(folder_path);

    match file_type {
        Ok(FileType::Directory) => Ok(()),
        Ok(file_type) => bail!(
            "Cannot create folder at {folder_path:?} because there's a {file_type} at that path"
        ),
        Err(err) if err.kind() == io::ErrorKind::NotFound => fs::create_dir_all(folder_path)
            .with_context(|| format!("Failed to create foder at {folder_path:?}")),
        Err(err) => Err(err)
            .with_context(|| format!("Failed to check if a folder {folder_path:?} already exists")),
    }
}

pub fn dedup_nested(paths: &mut Vec<&Path>) {
    let is_contained_in_another_path = |needle: &Path| {
        paths
            .iter()
            .filter(|&&haystack| haystack != needle)
            .any(|haystack| needle.strip_prefix(haystack).is_ok())
    };

    let items_to_be_removed: Vec<usize> = paths
        .iter()
        .rev()
        .enumerate()
        .filter(|(_index, path)| is_contained_in_another_path(path))
        .map(|(index, _path)| index)
        .collect();

    for index in items_to_be_removed {
        paths.remove(index);
    }
}

/// Check if files at the two paths are in the same filesystem.
pub fn are_in_the_same_filesystem(a: &Path, b: &Path) -> io::Result<bool> {
    let a = fs::metadata(a)?.dev();
    let b = fs::metadata(b)?.dev();
    Ok(a == b)
}

#[cfg(test)]
pub mod test_utils {
    use std::{
        env, io,
        path::Path,
        sync::{Mutex, MutexGuard},
    };

    // I know this is despicable, and I don't care

    static MUTEX: Mutex<()> = Mutex::new(());

    pub struct MutexTempDirHolder {
        _tempdir: tempfile::TempDir,
        _guard: MutexGuard<'static, ()>,
    }

    /// Create a test directory and cd into it
    pub fn cd_to_testdir() -> io::Result<(MutexTempDirHolder, &'static Path)> {
        let guard = loop {
            if let Ok(guard) = MUTEX.lock() {
                break guard;
            } else {
                MUTEX.clear_poison();
                continue;
            }
        };
        let tempdir = tempfile::tempdir()?;
        let path = tempdir.path().to_path_buf().into_boxed_path();
        env::set_current_dir(&path)?;

        let holder = MutexTempDirHolder {
            _tempdir: tempdir,
            _guard: guard,
        };

        Ok((holder, Box::leak(path)))
    }
}

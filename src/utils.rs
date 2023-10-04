use std::{
    env,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
};

use anyhow::Context;
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
        let guard = MUTEX.lock().unwrap();
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

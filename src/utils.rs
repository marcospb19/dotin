use std::{
    env,
    path::{Path, PathBuf},
    sync::LazyLock,
};

#[cfg(test)]
// Re-export
pub use test_utils::testdir;

static HOME_DIR_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    env::var_os("HOME")
        .expect("Failed to read user's home directory, try setting $HOME")
        .into()
});

pub fn get_home_dir() -> &'static Path {
    &HOME_DIR_PATH
}

#[cfg(test)]
mod test_utils {
    use std::{io, path::Path};

    pub fn testdir() -> io::Result<(tempfile::TempDir, &'static Path)> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().to_path_buf().into_boxed_path();
        Ok((dir, Box::leak(path)))
    }
}

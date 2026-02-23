use std::{
    env,
    ffi::{OsStr, OsString},
    io,
    iter::repeat_n,
    os::unix::fs::{MetadataExt, symlink},
    path::{Path, PathBuf},
};

use anyhow::{Context, bail};
use fs_err::{self as fs, PathExt};
use indexmap::IndexMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Regular,
    Directory,
    Symlink,
}

pub fn read_file_type(path: &Path) -> anyhow::Result<FileType> {
    use file_type_enum::FileType::*;

    match file_type_enum::FileType::symlink_read_at(path)? {
        Regular => Ok(FileType::Regular),
        Directory => Ok(FileType::Directory),
        Symlink => Ok(FileType::Symlink),
        variant => bail!("path {path:?}, is a {variant}, which isn't supported"),
    }
}

pub fn get_home_dir() -> anyhow::Result<PathBuf> {
    let home_env_var =
        env::var_os("HOME").context("Failed to read user's home directory, try setting $HOME")?;

    fs::canonicalize(&*home_env_var).context("Failed to read path at $HOME")
}

/// Reimplement `try_exists` so it works when `path` points to a symlink and
/// the symlink is broken.
pub fn try_exists(path: impl AsRef<Path>) -> io::Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

/// Creates a symlink at `link_location` that points to `original`.
pub fn create_symlink(link_location: &Path, original: &Path) -> anyhow::Result<()> {
    symlink(original, link_location).with_context(|| {
        format!("Failed to create symlink at {link_location:?} pointing to {original:?}")
    })
}

/// Creates the target for a symlink at `the/relative/path` as `../../dotfiles/GROUP/the/relative/path`
pub fn create_relative_symlink_target_path(relative_path: &Path, group_name: &str) -> PathBuf {
    let nestedness = relative_path.components().count().saturating_sub(1);
    let path_out_of_nesting = repeat_n(Path::new("../"), nestedness).collect::<PathBuf>();
    path_out_of_nesting
        .join("dotfiles")
        .join(group_name)
        .join(relative_path)
}

pub fn create_folder_at(folder_path: &Path) -> anyhow::Result<()> {
    match fs::symlink_metadata(folder_path) {
        Ok(_) => {
            let file_type = read_file_type(folder_path)?;
            match file_type {
                FileType::Directory => Ok(()),
                FileType::Regular => {
                    bail!(
                        "can't create folder at {folder_path:?}, a regular file exists at that path"
                    )
                }
                FileType::Symlink => {
                    bail!("can't create folder at {folder_path:?}, a symlink exists at that path")
                }
            }
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            println!("creating folder at {folder_path:?}");
            fs::create_dir_all(folder_path).context("creating folder")
        }
        Err(err) => Err(err.into()),
    }
}

pub fn cheap_move_with_fallback(from: &Path, to: &Path) -> anyhow::Result<()> {
    if let Some(to_parent) = to.parent()
        && !try_exists(to_parent)?
    {
        fs::create_dir_all(to_parent)?;
    }

    if let Err(err) = fs::rename(from, to) {
        // if renaming (cheapest move) is impossible, try fallback
        if err.kind() == io::ErrorKind::CrossesDevices {
            if let FileType::Directory = read_file_type(from)? {
                // dir fallback
                expensive_folder_copy(from.to_owned(), to.to_owned())?;
            } else {
                // non-dir fallback
                fs::copy(from, to).context("while trying to move file")?;
                fs::remove_file(from).context("removing file after copy (mv operation)")?;
            }
        } else {
            return Err(err.into());
        }
    }
    Ok(())
}

fn expensive_folder_copy(from: PathBuf, to: PathBuf) -> anyhow::Result<()> {
    // Use a stack to avoid too-many-files error (this can't ever stack
    // overflow due to Linux's path size limit)
    let mut stack = Vec::new();
    stack.push((from, to));

    while let Some((from, to)) = stack.pop() {
        if from.fs_err_metadata()?.is_dir() {
            fs::create_dir_all(&to)?;
            for entry in fs::read_dir(from)? {
                let entry = entry?;
                let path = entry.path();
                // Unwrap Safety:
                //   A path retrieved by readdir always has a file_name
                let name = path.file_name().unwrap();
                let dest = to.join(name);
                stack.push((path, dest));
            }
        } else {
            fs::copy(from, to)?;
        }
    }

    Ok(())
}

type PathTrieMap = IndexMap<OsString, PathTrie, rapidhash::fast::RandomState>;

#[derive(Default)]
pub struct PathTrie {
    is_path: bool,
    children: PathTrieMap,
}

impl PathTrie {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn contains_ancestor_of(&self, path: &Path) -> bool {
        let (first, rest) = path_split_first(path);

        if rest.as_os_str().is_empty() {
            return false;
        }

        let Some(first) = first else {
            return false;
        };

        let Some(node) = self.children.get(first) else {
            return false;
        };

        if node.is_path {
            return true;
        }

        node.contains_ancestor_of(rest)
    }

    /// Inserts a path into this Trie.
    ///
    /// # Panics:
    ///
    /// - Panics if `path` isn't absolute.
    pub fn insert(&mut self, path: &Path) {
        debug_assert!(path.is_absolute(), "PathTrie only accepts absolute paths");
        self.insert_recursive(path)
    }

    fn insert_recursive(&mut self, path: &Path) {
        let path = path.to_owned();
        let (first, rest) = path_split_first(&path);

        if let Some(first) = first {
            let node = self.children.entry(first.to_owned()).or_default();

            if rest.iter().next().is_none() {
                node.is_path = true;
            }
            node.insert_recursive(rest)
        }
    }
}

impl<Item> FromIterator<Item> for PathTrie
where
    Item: AsRef<Path>,
{
    fn from_iter<T: IntoIterator<Item = Item>>(iter: T) -> Self {
        let mut trie = PathTrie::new();
        for path in iter {
            trie.insert(path.as_ref());
        }
        trie
    }
}

pub fn deduplicate_paths_inside_others(paths: &mut Vec<&Path>) {
    let mut trie = PathTrie::new();
    for path in paths.iter() {
        trie.insert(path);
    }
    paths.retain(|path| !trie.contains_ancestor_of(path));
}

fn path_split_first(path: &Path) -> (Option<&OsStr>, &Path) {
    let mut iter = path.iter();
    (iter.next(), iter.as_path())
}

#[expect(unused)]
/// Check if files at the two paths are in the same filesystem.
fn are_in_the_same_filesystem(a: &Path, b: &Path) -> io::Result<bool> {
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
    static TEST_LOCK_MUTEX: Mutex<()> = Mutex::new(());

    pub struct MutexTempDirHolder {
        _tempdir: tempfile::TempDir,
        _guard: MutexGuard<'static, ()>,
    }

    /// Create a test directory and cd into it
    pub fn cd_to_testdir() -> io::Result<(MutexTempDirHolder, &'static Path)> {
        let guard = loop {
            match TEST_LOCK_MUTEX.lock() {
                Ok(guard) => break guard,
                Err(_) => {
                    TEST_LOCK_MUTEX.clear_poison();
                    continue;
                }
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

#[derive(Debug)]
pub struct FileToMove<'a> {
    pub path: &'a Path,
    pub to_path: PathBuf,
}

#[cfg(test)]
mod path_trie_tests {
    use std::path::Path;

    use super::PathTrie;

    #[test]
    fn test_path_trie_contains_ancestor_of() {
        let trie = PathTrie::from_iter(["/home/user"]);
        assert!(trie.contains_ancestor_of(Path::new("/home/user/docs")));
        assert!(trie.contains_ancestor_of(Path::new("/home/user/docs/file.txt")));

        let trie = PathTrie::from_iter(["/home/user"]);
        assert!(!trie.contains_ancestor_of(Path::new("/home/user")));

        let trie = PathTrie::from_iter(["/home/user/docs"]);
        assert!(!trie.contains_ancestor_of(Path::new("/home/user")));
        assert!(!trie.contains_ancestor_of(Path::new("/home")));

        let trie = PathTrie::from_iter(["/home/user"]);
        assert!(!trie.contains_ancestor_of(Path::new("/var/log")));
        assert!(!trie.contains_ancestor_of(Path::new("/home/other")));

        let trie = PathTrie::from_iter(["/home/user", "/var/log"]);
        assert!(trie.contains_ancestor_of(Path::new("/home/user/docs")));
        assert!(trie.contains_ancestor_of(Path::new("/var/log/syslog")));
        assert!(!trie.contains_ancestor_of(Path::new("/etc/config")));

        let trie = PathTrie::new();
        assert!(!trie.contains_ancestor_of(Path::new("/home/user")));
    }
}

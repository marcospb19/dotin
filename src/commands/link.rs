use std::path::Path;

use eyre::WrapErr;
use fs_err as fs;
use fs_tree::FsTree;

use crate::{
    Result,
    utils::{self, create_relative_symlink_target_path},
};

pub fn link(base_dir: &Path, group_dir: &Path) -> Result<()> {
    let group_tree = FsTree::symlink_read_at(group_dir).wrap_err("reading dotfiles folder tree")?;

    let base_tree = group_tree
        .symlink_read_structure_at(base_dir)
        .wrap_err("reading structured file tree at base folder")?;

    let mut intermediate_directories_linked = vec![];

    for (group_node, relative_path) in &group_tree {
        // Skip children where the parent directory is already linked
        if intermediate_directories_linked
            .iter()
            .any(|intermediate_dir| relative_path.starts_with(intermediate_dir))
        {
            continue;
        }

        let base_absolute = base_dir.join(&relative_path);
        let dotfile_absolute = group_dir.join(&relative_path);
        let symlink_target = create_relative_symlink_target_path(&base_absolute, &dotfile_absolute);

        // if already exists at base folder
        if let Some(base_node) = base_tree.get(&relative_path) {
            if group_node.is_leaf() {
                if let Some(current_target) = base_node.target() {
                    if current_target == symlink_target {
                        println!("OK: skipping link {base_absolute:?}");
                        if group_node.is_dir() {
                            intermediate_directories_linked.push(relative_path);
                        }
                    } else {
                        println!(
                            "ERROR: {base_absolute:?} exists but points to {current_target:?} instead of {symlink_target:?}"
                        );
                    }
                } else {
                    println!(
                        "ERROR: can't create link at {base_absolute:?} because a {} already exists",
                        base_node.variant_str(),
                    );
                }
            } else if base_node.is_dir() {
                // great! directory found where non-leaf was expected, no need to create one
            } else {
                println!("ERROR: can't create link at {base_absolute:?} because it's a directory");
            }
        } else {
            // only link the leaves, non-leafs are created like `mkdir`
            // (note: a non-leaf is a dir, but a dir can be a leaf)
            if group_node.is_leaf() {
                utils::create_symlink(&base_absolute, &symlink_target)?;
                println!("Linked {} at {relative_path:?}", group_node.variant_str());
            } else {
                fs::create_dir(&base_absolute).wrap_err("creating directory for dotfile")?;
                println!("Created intermediate directory at {base_absolute:?}");
            }
        }
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
    fn test_link() {
        let (_dropper, test_dir) = cd_to_testdir().unwrap();

        let home = tree! {
            ".config": [
            ]
        };

        let dotfiles = tree! {
            dotfiles: [
                i3: [
                    ".config": [
                        i3: [
                            config
                        ]
                    ]
                ]
            ]
        };

        let expected_home = tree! {
            ".config": [
                i3: [
                    config -> "../../dotfiles/i3/.config/i3/config"
                ]
            ]
        };

        home.write_structure_at(".").unwrap();
        dotfiles.write_structure_at(".").unwrap();

        link(test_dir, &test_dir.join("dotfiles/i3")).unwrap();

        let result = expected_home.symlink_read_structure_at(".").unwrap();
        assert_eq!(result, expected_home);
    }

    #[test]
    fn test_link_with_override_base_folder() {
        let (_dropper, test_dir) = cd_to_testdir().unwrap();
        let base_dir = test_dir.join("base");

        let base = tree! {
            base: []
        };
        let dotfiles = tree! {
            dotfiles: [
                sddm: [
                    config: [
                        theme_conf
                    ]
                ]
            ]
        };
        let expected_base = tree! {
            base: [
                config: [
                    theme_conf -> "../../dotfiles/sddm/config/theme_conf"
                ]
            ]
        };

        base.write_structure_at(".").unwrap();
        dotfiles.write_structure_at(".").unwrap();

        link(&base_dir, &test_dir.join("dotfiles/sddm")).unwrap();

        let result = expected_base.symlink_read_structure_at(".").unwrap();
        assert_eq!(result, expected_base);
    }
}

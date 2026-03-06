use std::path::PathBuf;

use clap::Parser;
use dotin::{
    Result,
    commands::{discard, import, link, unlink},
    utils::{find_dotfiles_folder, get_home_dir, try_exists},
};
use eyre::WrapErr;

#[derive(Parser, Debug)]
#[command(version, about)]
enum Command {
    /// Moves files into a specific dotfiles group folder
    Import {
        group_name: String,
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },
    /// Move file back from a group to its target position (reverse of import)
    Discard {
        group_name: String,
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },
    /// Link dotfiles groups into their target position
    Link { groups: Vec<String> },
    /// Removes links created by the `link` command
    Unlink { groups: Vec<String> },
}

fn main() -> Result<()> {
    color_eyre::install().unwrap();

    let home_dir = &get_home_dir()?;
    let dotfiles_folder = find_dotfiles_folder(home_dir)?;

    // TODO: err early if given path is empty string
    // TODO: err early if trying to import or discard `"."`
    match Command::parse() {
        Command::Unlink { groups } => {
            if groups.is_empty() {
                println!("list of groups to unlink is empty.");
                return Ok(());
            }

            for group in &groups {
                unlink(home_dir, &dotfiles_folder.join(group), group)
                    .wrap_err_with(|| format!("Failed to unlink group \"{group}\""))?;
            }
        }
        Command::Link { groups } => {
            if groups.is_empty() {
                println!("No group list provided.");
            }

            for group in &groups {
                let dotfiles_group_folder = &dotfiles_folder.join(group);

                link(home_dir, dotfiles_group_folder, group)
                    .wrap_err_with(|| format!("Failed to link group \"{group}\""))?;
            }
        }
        Command::Import { group_name, files } => {
            assert!(!files.is_empty(), "ensured by CLI definitions");
            import(home_dir, &dotfiles_folder.join(&group_name), &files)
                .wrap_err_with(|| format!("Failed to import files for group \"{group_name}\""))?;
        }
        Command::Discard { group_name, files } => {
            assert!(!files.is_empty(), "ensured by CLI definitions");
            if !try_exists(dotfiles_folder.join(&group_name))? {
                println!(
                    "Group \"{group_name}\" does not exist at {:?}.",
                    dotfiles_folder.join(&group_name)
                );
                return Ok(());
            }
            discard(home_dir, &dotfiles_folder.join(&group_name), &files)
                .wrap_err_with(|| format!("Failed to discard files for group \"{group_name}\""))?;
        }
    }

    println!("Done.");
    Ok(())
}

use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use dotin::{
    commands::{import, link, unlink},
    utils::{get_home_dir, read_all_groups},
};

#[derive(Parser, Debug)]
#[command(version, about)]
enum Command {
    /// Moves files into the dotfiles group folder, doesn't link them
    Import {
        group_name: String,
        #[clap(required = true)]
        files: Vec<PathBuf>,
    },
    /// Link dotfiles groups into their target position
    Link {
        groups: Vec<String>,
        /// Link all groups in the dotfiles folder
        #[clap(long)]
        all: bool,
    },
    /// Removes links created by the `link` command
    Unlink { groups: Vec<String> },
}

fn main() {
    if let Err(err) = run() {
        eprintln!("EXITING: ERROR: {err}");
    }
}

fn run() -> anyhow::Result<()> {
    let home_dir = &get_home_dir()?;
    let dotfiles_folder = home_dir.join("dotfiles");

    match Command::parse() {
        Command::Unlink { groups } => {
            if groups.is_empty() {
                println!("No group list provided.");
            }

            for group in &groups {
                let dotfiles_group_folder = &dotfiles_folder.join(group);

                unlink(home_dir, dotfiles_group_folder, group)
                    .with_context(|| format!("Failed to unlink group \"{group}\""))?;
            }
        }
        Command::Link { groups, all } => {
            let groups_to_link = if all {
                read_all_groups(&dotfiles_folder)?
            } else {
                groups
            };

            if groups_to_link.is_empty() {
                println!("No group list provided.");
            }

            for group in &groups_to_link {
                let dotfiles_group_folder = &dotfiles_folder.join(group);

                link(home_dir, dotfiles_group_folder, group)
                    .with_context(|| format!("Failed to link group \"{group}\""))?;
            }
        }
        Command::Import { group_name, files } => {
            let dotfiles_group_folder = &dotfiles_folder.join(&group_name);

            import(home_dir, dotfiles_group_folder, &files)
                .with_context(|| format!("Failed to import files for group \"{group_name}\""))?;
        }
    }

    Ok(())
}

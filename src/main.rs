use anyhow::Context;
use clap::Parser;
use dotin::{
    commands::{link, unlink},
    utils::get_home_dir,
};

#[derive(Parser, Debug)]
#[command(version, about)]
enum Command {
    /// For each provided group, create link for their files.
    Link { groups: Vec<String> },
    /// For each provided group, delete the links created.
    Unlink { groups: Vec<String> },
}

fn main() -> anyhow::Result<()> {
    let home_dir = &get_home_dir()?;
    let dotfiles_folder = home_dir.join("dotfiles");

    match Command::parse() {
        Command::Unlink { groups } => {
            if groups.is_empty() {
                println!("No group list provided.");
            }

            groups.iter().try_for_each(|group| {
                let dotfiles_group_folder = &dotfiles_folder.join(group);

                unlink(home_dir, dotfiles_group_folder)
                    .with_context(|| format!("Failed to unlink group \"{group}\""))
            })
        }
        Command::Link { groups } => {
            if groups.is_empty() {
                println!("No group list provided.");
            }

            groups.iter().try_for_each(|group| {
                let dotfiles_group_folder = &dotfiles_folder.join(group);

                link(home_dir, dotfiles_group_folder)
                    .with_context(|| format!("Failed to link group \"{group}\""))
            })
        }
    }
}

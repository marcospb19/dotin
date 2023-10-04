use anyhow::Context;
use clap::Parser;
use dotin::{link, unlink, utils::get_home_dir};

#[derive(Parser, Debug)]
#[command(author, version, about)]
enum Command {
    Unlink { groups: Vec<String> },
    Link { groups: Vec<String> },
}

fn main() -> anyhow::Result<()> {
    let home_dir = &get_home_dir()?;
    let dotfiles_folder = home_dir.join("dotfiles");

    match Command::parse() {
        Command::Unlink { groups } => {
            for group in &groups {
                let dotfiles_group_folder = &dotfiles_folder.join(group);

                unlink(home_dir, dotfiles_group_folder)
                    .with_context(|| format!("Failed to unlink group \"{group}\""))?;
            }
        }
        Command::Link { groups } => {
            for group in &groups {
                let dotfiles_group_folder = &dotfiles_folder.join(group);

                link(home_dir, dotfiles_group_folder)
                    .with_context(|| format!("Failed to link group \"{group}\""))?;
            }
        }
    }

    Ok(())
}

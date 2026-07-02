use std::path::PathBuf;

use clap::Parser;
use dotin::{
    Result,
    commands::{discard, import, link, unlink},
    config::{init_config, read_config},
    utils::{find_dotfiles_folder, get_home_dir, try_exists},
};
use eyre::{WrapErr, bail};

#[derive(Parser, Debug)]
#[command(version, about)]
enum Command {
    /// Moves files into a specific dotfiles group folder and links them back
    Import {
        group_name: String,
        #[arg(required = true)]
        files: Vec<PathBuf>,
        /// Skip linking files back to their original location after import
        #[arg(long)]
        no_link: bool,
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
    /// Create config, or check its location
    Config {
        #[arg(short, long)]
        init: bool,
    },
}

fn main() -> Result<()> {
    color_eyre::install().unwrap();

    let home_dir = &get_home_dir()?;
    let dotfiles_folder = find_dotfiles_folder(home_dir)?;
    let config = read_config(home_dir, &dotfiles_folder).wrap_err("Failed to read config")?;

    let command = Command::parse();

    // err early if trying to import or discard `"."`
    if let Command::Import { files, .. } | Command::Discard { files, .. } = &command
        && files.iter().find(|&file| file == ".").is_some()
    {
        bail!("Cannot import or discard the current directory (\".\")");
    }

    match command {
        Command::Unlink { groups } => {
            if groups.is_empty() {
                println!("list of groups to unlink is empty.");
                return Ok(());
            }

            for group in &groups {
                let base_folder = config.inner.base_folder_for_group(home_dir, group);

                unlink(&base_folder, &dotfiles_folder.join(group))
                    .wrap_err_with(|| format!("Failed to unlink group \"{group}\""))?;
            }
        }
        Command::Link { groups } => {
            if groups.is_empty() {
                println!("No group list provided.");
            }

            for group in &groups {
                let dotfiles_group_folder = &dotfiles_folder.join(group);
                let base_folder = config.inner.base_folder_for_group(home_dir, group);

                link(&base_folder, dotfiles_group_folder)
                    .wrap_err_with(|| format!("Failed to link group \"{group}\""))?;
            }
        }
        Command::Import {
            group_name,
            files,
            no_link,
        } => {
            assert!(!files.is_empty(), "ensured by CLI definitions");
            let base_folder = config.inner.base_folder_for_group(home_dir, &group_name);
            let group_folder = dotfiles_folder.join(&group_name);

            import(&base_folder, &group_folder, &files)
                .wrap_err_with(|| format!("Failed to import files for group \"{group_name}\""))?;

            if !no_link {
                link(&base_folder, &group_folder)
                    .wrap_err_with(|| format!("Failed to link group \"{group_name}\""))?;
            }
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
            let base_folder = config.inner.base_folder_for_group(home_dir, &group_name);

            discard(&base_folder, &dotfiles_folder.join(&group_name), &files)
                .wrap_err_with(|| format!("Failed to discard files for group \"{group_name}\""))?;
        }
        Command::Config { init } => {
            if init {
                init_config(home_dir, &dotfiles_folder)?;
            } else if config.path.is_some() {
                println!(
                    "Config file set at {}",
                    config.path.as_ref().unwrap().display()
                );
            } else {
                println!("No config file set. Run `dotin config --init` to create one.");
            }
        }
    }

    println!("Done.");
    Ok(())
}

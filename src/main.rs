use std::env;

use clap::Parser;
use dotin::{remove_links, utils::get_home_dir};

#[derive(Parser, Debug)]
#[command(author, version, about)]
enum Command {
    Unlink { groups: Vec<String> },
    Link { groups: Vec<String> },
}

fn main() {
    env::set_current_dir("/home/marcospb19").unwrap();

    let home_dir = get_home_dir();

    match Command::parse() {
        Command::Unlink { groups } => {
            for group in groups {
                remove_links(home_dir, &group).unwrap();
            }
        }
        Command::Link { groups: _ } => {
            todo!("link implementation");
        }
    }
}

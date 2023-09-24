use std::{env, path::PathBuf};

use dotin::{remove_links, utils::get_home_dir};

fn main() {
    env::set_current_dir("/home/marcospb19/dotfiles").unwrap();

    let path = PathBuf::from("i3");
    let home_dir = get_home_dir();

    remove_links(home_dir, &path).unwrap();
}

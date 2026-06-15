use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use eyre::{bail, eyre};
use fs_err as fs;
use indexmap::IndexMap;
use serde::Deserialize;

use crate::{Result, utils::try_exists};

const INITIAL_CONFIG: &str = indoc::indoc! { r#"
    # `dotin` configuration file
    # see https://github.com/marcospb19/dotin

    # Change group root base from "~" to the specified directory
    # (Note: paths don't yet support "~")
    [override_base_folder]
    # sddm = "/etc"
    # systemd = "/etc"
"# };

#[derive(Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub override_base_folder: IndexMap<String, String>,
}

impl Config {
    pub fn base_folder_for_group<'a>(&'a self, home: &'a Path, group: &str) -> Cow<'a, Path> {
        self.override_base_folder
            .get(group)
            .map(|base| Cow::Borrowed(Path::new(base)))
            .unwrap_or(Cow::Borrowed(home))
    }
}

#[derive(Default)]
pub struct ConfigWithPath {
    pub inner: Config,
    pub path: Option<PathBuf>,
}

pub fn read_config(home: &Path, dotfiles: &Path) -> Result<ConfigWithPath> {
    let home_conf_path = home.join(".config/dotin/config.toml");
    let dots_conf_path = dotfiles.join("dotin.toml");

    let home_conf_path_option = try_exists(&home_conf_path)?.then_some(home_conf_path);
    let dots_conf_path_option = try_exists(&dots_conf_path)?.then_some(dots_conf_path);

    match (home_conf_path_option, dots_conf_path_option) {
        (None, None) => Ok(ConfigWithPath::default()),
        (None, Some(path)) | (Some(path), None) => Ok(ConfigWithPath {
            inner: read_config_from_path(&path)?,
            path: Some(path),
        }),
        (Some(_), Some(_)) => {
            // TODO: add DetailedError, give two details and one hint
            Err(eyre!(
                "both config files found, only one is allowed — remove one"
            ))
        }
    }
}

fn read_config_from_path(path: &Path) -> Result<Config> {
    let contents = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&contents)?;
    validate_config(&config)?;
    Ok(config)
}

fn validate_config(config: &Config) -> Result<()> {
    for (key, value) in &config.override_base_folder {
        if value.is_empty() {
            return Err(eyre!(
                "config override_base_folder key {key:?} has empty value",
            ));
        }

        if !Path::new(value).is_absolute() {
            return Err(eyre!(
                "config override_base_folder key {key:?} has relative path {value:?}; expected absolute path",
            ));
        }
    }

    Ok(())
}

pub fn init_config(home: &Path, dotfiles: &Path) -> Result<()> {
    let existing = read_config(home, dotfiles)?;
    if let Some(path) = existing.path {
        // config already exists
        bail!("config already exists at {}", path.display());
    }
    // no config found — create sample at home/.config/dotin/config.toml
    let config_path = home.join(".config/dotin/config.toml");
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&config_path, INITIAL_CONFIG)?;
    println!("Created sample config at {}", config_path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_absolute_override_base_folder() {
        let config: Config = toml::from_str(indoc::indoc! { r#"
            [override_base_folder]
            sddm = "/etc"
        "# })
        .unwrap();

        validate_config(&config).unwrap();
    }

    #[test]
    fn rejects_empty_override_base_folder() {
        let config: Config = toml::from_str(indoc::indoc! { r#"
            [override_base_folder]
            sddm = ""
        "# })
        .unwrap();

        let error = validate_config(&config).unwrap_err().to_string();

        assert!(error.contains("has empty value"), "msg = {error}");
    }

    #[test]
    fn rejects_relative_override_base_folder() {
        let config: Config = toml::from_str(indoc::indoc! { r#"
            [override_base_folder]
            sddm = "etc"
        "# })
        .unwrap();

        let error = validate_config(&config).unwrap_err().to_string();

        assert!(error.contains("has relative path"), "msg = {error}");
    }
}

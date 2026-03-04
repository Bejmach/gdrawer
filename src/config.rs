use std::{collections::HashMap, fs, path::PathBuf};

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, Deserialize, Serialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Set {
    pub directory: String,

    pub image_duration: u32,
    pub break_duration: u32,
    pub image_limit: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub version: Version,
    pub sets: HashMap<String, Set>,
}

impl Version {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

impl Set {
    pub fn new(
        directory: String,
        image_duration: u32,
        break_duration: u32,
        image_limit: u32,
    ) -> Self {
        Self {
            directory,
            image_duration,
            break_duration,
            image_limit,
        }
    }
}

impl Config {
    pub fn new(version: Version, sets: HashMap<String, Set>) -> Self {
        Self { version, sets }
    }

    fn get_path(path: Vec<&str>) -> Option<PathBuf> {
        let config_option = dirs_next::config_dir();
        if let Some(mut config_path) = config_option {
            for str in path {
                config_path.push(str);
            }
            return Some(config_path);
        }
        None
    }

    /// Loads config from file. Don't add file extension
    pub fn load(path: Vec<&str>, file_name: &str) -> Result<Self> {
        if let Some(dir_path) = Config::get_path(path) {
            let file_path: PathBuf = dir_path.join(format!("{}.json", file_name));
            if fs::exists(&file_path).is_err() {
                return Err(anyhow!("file does not exist"));
            }
            let content: String = fs::read_to_string(&file_path).expect("Failed to read file");
            return Ok(serde_json::from_str(&content).expect("Failed to parse config"));
        }
        Err(anyhow!("failed to find config directory"))
    }

    /// Loads config from file. Don't add file extension
    pub fn save(&self, path: Vec<&str>, file_name: &str) -> Result<()> {
        if let Some(dir_path) = Config::get_path(path) {
            let exist_result = fs::exists(&dir_path);
            if let Ok(exist) = exist_result
                && !exist
            {
                if let Err(value) = fs::create_dir_all(&dir_path) {
                    return Err(anyhow!("{}", value));
                }
            } else if exist_result.is_err() {
                return Err(anyhow!("Cant define if directory exists"));
            }
            let content: String = serde_json::to_string(self).expect("Failed to parse config");
            let file_path = dir_path.join(format!("{}.json", file_name));
            if fs::write(file_path, content).is_err() {
                return Err(anyhow!("failed to save file"));
            }
            return Ok(());
        }
        Err(anyhow!("failed to find config directory"))
    }
}

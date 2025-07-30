use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub reflector_name: String,
    pub bind_address: String,
    pub modules: Vec<char>,
    pub strict_crc: bool,
    #[serde(default)]
    pub interlinks: Vec<InterlinkConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct InterlinkConfig {
    pub name: String,
    pub address: String,
    pub modules: Vec<char>,
}

impl Config {
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }
}

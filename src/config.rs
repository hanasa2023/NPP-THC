use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    pub output_path: String,
    pub theme: String,
}

impl AppConfig {
    pub fn from_file(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = std::fs::File::open(file_path)?;
        let config: AppConfig = serde_json::from_reader(file)?;
        Ok(config)
    }

    pub fn to_file(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = std::fs::File::create(file_path)?;
        serde_json::to_writer(file, self)?;
        Ok(())
    }
}

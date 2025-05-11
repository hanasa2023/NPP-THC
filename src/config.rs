use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub output_directory: String,
    pub last_file: String,
}

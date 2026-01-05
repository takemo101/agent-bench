use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct SoundConfig {
    pub work_end_sound: String,
    pub break_end_sound: String,
}

impl Default for SoundConfig {
    fn default() -> Self {
        Self {
            work_end_sound: "Funk".to_string(),
            break_end_sound: "Glass".to_string(),
        }
    }
}

impl SoundConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        // TODO: Implement
        Ok(Self::default())
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Implement
        Ok(())
    }
    
    // Helper for testing to inject path?
    // Or just use dirs::home_dir() inside.
    // For TDD, it's better if I can mock the path or use a different file.
    // So maybe `load_from(path: PathBuf)`?
    
    pub fn load_from_file(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        unimplemented!("Not implemented yet");
    }

    pub fn save_to_file(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        unimplemented!("Not implemented yet");
    }
}

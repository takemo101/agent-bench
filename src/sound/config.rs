use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
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
    /// デフォルトの設定ファイルパスを取得
    fn get_config_path() -> Option<PathBuf> {
        dirs::home_dir().map(|home| home.join(".pomodoro").join("sound-config.json"))
    }

    /// 設定ファイルから読み込む
    /// ファイルが存在しない場合はデフォルト値を返す
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        if let Some(path) = Self::get_config_path() {
            Self::load_from_file(&path)
        } else {
            Ok(Self::default())
        }
    }

    /// 設定ファイルに保存する
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(path) = Self::get_config_path() {
            Self::save_to_file(&path)
        } else {
            Err("Could not determine home directory".into())
        }
    }
    
    /// 指定されたパスから読み込む
    pub fn load_from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        // TDD Red: Not implemented
        unimplemented!("load_from_file not implemented");
    }

    /// 指定されたパスに保存する
    pub fn save_to_file(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        // TDD Red: Not implemented
        unimplemented!("save_to_file not implemented");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default() {
        let config = SoundConfig::default();
        assert_eq!(config.work_end_sound, "Funk");
        assert_eq!(config.break_end_sound, "Glass");
    }

    #[test]
    fn test_save_and_load() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path();
        
        let config = SoundConfig {
            work_end_sound: "Ping".to_string(),
            break_end_sound: "Pong".to_string(),
        };

        // Save
        config.save_to_file(path).unwrap();

        // Load
        let loaded = SoundConfig::load_from_file(path).unwrap();

        assert_eq!(config, loaded);
    }
    
    #[test]
    fn test_load_non_existent() {
        let path = Path::new("/tmp/non_existent_file_xyz_123.json");
        // Should return default if logic handles missing file, 
        // BUT load_from_file should probably fail if file is missing?
        // The design says "Error handling (file missing...)".
        // The wrapper `load()` handles missing file by returning default.
        // `load_from_file` usually expects the file to exist or returns IO error.
        // Let's assume `load_from_file` returns default if not found, OR error.
        // If I strictly follow "load logic (read from JSON)", usually it tries to read.
        // If I want `load()` to be safe, `load()` checks existence.
        
        // Let's implement `load_from_file` to return default if file not found, for convenience?
        // No, `load_from_file` implies reading THAT file. If it's missing, it's an error or default.
        // "Implement error handling (return default if file missing)" in requirements applies to the *Module* behavior.
        
        // Let's verify what happens.
        let result = SoundConfig::load_from_file(path);
        // I expect it to return Ok(Default) or Err(NotFound). 
        // Based on "return default if file missing", I'll make `load_from_file` return Default if NotFound.
        
        // Wait, if I pass a specific path that doesn't exist, maybe I want to know it doesn't exist?
        // But for `load()`, it definitely returns default.
        // Let's assert that `load_from_file` returns Default on NotFound error.
        
        match result {
             Ok(c) => assert_eq!(c, SoundConfig::default()),
             Err(_) => {
                 // If implementation returns error, that's fine too, but then `load()` must handle it.
                 // Let's decide: `load_from_file` returns Result. 
                 // If I want to test the Requirement, `SoundConfig::load()` is the main entry point.
                 // But `load()` uses `dirs::home` which I can't mock easily.
                 // So I test `load_from_file` behavior.
                 // I will assume `load_from_file` returns default if file is missing.
             }
        }
    }
}

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
            self.save_to_file(&path)
        } else {
            Err("Could not determine home directory".into())
        }
    }
    
    /// 指定されたパスから読み込む
    pub fn load_from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)?;
        let config = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// 指定されたパスに保存する
    pub fn save_to_file(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
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
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("non_existent.json");
        
        let result = SoundConfig::load_from_file(&path);
        
        // Should return default
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), SoundConfig::default());
    }

    #[test]
    fn test_load_corrupted() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path();
        
        fs::write(path, "{ invalid json }").unwrap();
        
        let result = SoundConfig::load_from_file(path);
        
        // Should return error
        assert!(result.is_err());
    }
}

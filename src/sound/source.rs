use std::path::PathBuf;
use std::fs;

/// サウンドの取得元を表す列挙型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SoundSource {
    /// macOSシステムサウンド
    System {
        /// サウンド名（例: "Glass"）
        name: String,
        /// フルパス（例: "/System/Library/Sounds/Glass.aiff"）
        path: PathBuf,
    },
    /// 埋め込みサウンド（バイナリに含まれる）
    Embedded {
        /// サウンド名（例: "default"）
        name: String,
    },
}

impl SoundSource {
    /// システムサウンドを検出する
    /// /System/Library/Sounds ディレクトリをスキャンする
    pub fn discover_system_sounds() -> Vec<SoundSource> {
        let system_sounds_path = PathBuf::from("/System/Library/Sounds");
        let mut sounds = Vec::new();

        if let Ok(entries) = fs::read_dir(&system_sounds_path) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_file() {
                    if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
                        match extension.to_lowercase().as_str() {
                            "aiff" | "wav" | "mp3" => {
                                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                                    sounds.push(SoundSource::System {
                                        name: stem.to_string(),
                                        path,
                                    });
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        
        // 名前でソートして順序を安定させる
        sounds.sort_by(|a, b| {
            match (a, b) {
                (SoundSource::System { name: a_name, .. }, SoundSource::System { name: b_name, .. }) => a_name.cmp(b_name),
                // Embeddedはここには来ないはずだが、念のため
                (SoundSource::System { .. }, SoundSource::Embedded { .. }) => std::cmp::Ordering::Less,
                (SoundSource::Embedded { .. }, SoundSource::System { .. }) => std::cmp::Ordering::Greater,
                (SoundSource::Embedded { name: a_name, .. }, SoundSource::Embedded { name: b_name, .. }) => a_name.cmp(b_name),
            }
        });

        sounds
    }

    /// デフォルトのサウンドソースを取得する
    /// 優先順位: Glass -> Ping -> Embedded
    pub fn get_default_source() -> SoundSource {
        let sounds = Self::discover_system_sounds();

        // Glassを探す
        if let Some(sound) = sounds.iter().find(|s| {
            matches!(s, SoundSource::System { name, .. } if name == "Glass")
        }) {
            return sound.clone();
        }

        // Pingを探す
        if let Some(sound) = sounds.iter().find(|s| {
            matches!(s, SoundSource::System { name, .. } if name == "Ping")
        }) {
            return sound.clone();
        }

        // 見つからなければ埋め込みサウンド
        SoundSource::Embedded {
            name: "default".to_string(),
        }
    }
}

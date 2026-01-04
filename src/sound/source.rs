use std::path::PathBuf;

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
    pub fn discover_system_sounds() -> Vec<SoundSource> {
        vec![] // Stub
    }

    /// デフォルトのサウンドソースを取得する
    pub fn get_default_source() -> SoundSource {
        // Stub: return Embedded default
        SoundSource::Embedded { name: "default".to_string() } 
    }
}

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// フック設定
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HookConfig {
    /// フック定義のマップ（イベント名 -> フック定義リスト）
    #[serde(default)]
    pub hooks: HashMap<String, Vec<HookDefinition>>,
    /// グローバルタイムアウト（秒）
    #[serde(default = "default_global_timeout")]
    pub global_timeout: u64,
}

fn default_global_timeout() -> u64 {
    30
}

/// フック定義
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookDefinition {
    /// フック名
    pub name: String,
    /// 実行するスクリプトのパス
    pub script: PathBuf,
    /// 有効かどうか
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// 個別タイムアウト（秒）
    pub timeout: Option<u64>,
}

fn default_enabled() -> bool {
    true
}

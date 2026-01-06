//! イベントフックモジュール
//!
//! タイマーイベント発生時に外部スクリプトを実行する機能を提供する。

pub mod config;
pub mod context;
pub mod executor;

pub use config::{HookConfig, HookDefinition};
pub use context::HookContext;
pub use executor::HookExecutor;

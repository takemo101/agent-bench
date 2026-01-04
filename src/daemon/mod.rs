//! Daemonモジュール
//!
//! ポモドーロタイマーのバックグラウンドデーモン機能を提供する。

pub mod timer;

pub use timer::{TimerEngine, TimerEvent};

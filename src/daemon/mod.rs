//! Daemonモジュール
//!
//! ポモドーロタイマーのバックグラウンドデーモン機能を提供する。

pub mod ipc;
pub mod timer;

pub use ipc::{handle_request, IpcServer};
pub use timer::{TimerEngine, TimerEvent};

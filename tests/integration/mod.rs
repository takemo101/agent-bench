//! コンポーネント間統合テスト
//!
//! 各コンポーネント間の連携をテストする。
//!
//! ## テスト対象
//! - Daemon-CLI IPC通信
//! - タイマー-通知連携
//! - タイマー-サウンド連携
//! - タイマー-フォーカスモード連携

pub mod daemon_cli;
pub mod timer_notification;

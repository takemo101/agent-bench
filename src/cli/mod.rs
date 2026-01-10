//! CLI module for pomodoro timer command-line interface
pub mod animation;
pub mod commands;
pub mod completions;
pub mod display;
pub mod ipc;
pub mod sound;

pub use commands::{Cli, Commands, StartArgs};
pub use completions::generate_completions;
pub use display::{Display, EnhancedDisplayState};
pub use ipc::IpcClient;
pub mod layout;
pub mod terminal;
pub mod time_format;

//! Display utilities for CLI output
//!
//! Provides colored and formatted output for CLI commands.

use crate::types::IpcResponse;
use colored::Colorize;

/// Display handler for CLI output
pub struct Display;

impl Display {
    /// Create a new Display instance
    pub fn new() -> Self {
        Self
    }

    /// Show start success message
    pub fn show_start_success(&self, response: IpcResponse) {
        println!("{} {}", "✓".green().bold(), response.message.green());
        if let Some(data) = response.data {
            if let Some(task) = data.task_name {
                println!("  タスク: {}", task.cyan());
            }
        }
    }

    /// Show pause success message
    pub fn show_pause_success(&self, response: IpcResponse) {
        println!("{} {}", "⏸".yellow().bold(), response.message.yellow());
    }

    /// Show resume success message
    pub fn show_resume_success(&self, response: IpcResponse) {
        println!("{} {}", "▶".green().bold(), response.message.green());
    }

    /// Show stop success message
    pub fn show_stop_success(&self, response: IpcResponse) {
        println!("{} {}", "■".red().bold(), response.message.red());
    }

    /// Show status information
    pub fn show_status(&self, response: IpcResponse) {
        if let Some(data) = response.data {
            println!("{}", "=== タイマー状態 ===".bold());

            if let Some(state) = data.state {
                let state_display = match state.as_str() {
                    "working" => "作業中".green(),
                    "breaking" => "休憩中".cyan(),
                    "long_breaking" => "長い休憩中".cyan(),
                    "paused" => "一時停止".yellow(),
                    "stopped" => "停止中".red(),
                    _ => state.normal(),
                };
                println!("状態: {}", state_display);
            }

            if let Some(remaining) = data.remaining_seconds {
                let minutes = remaining / 60;
                let seconds = remaining % 60;
                println!("残り時間: {}:{:02}", minutes, seconds);
            }

            if let Some(count) = data.pomodoro_count {
                println!("完了ポモドーロ: {} 🍅", count);
            }

            if let Some(task) = data.task_name {
                println!("タスク: {}", task.cyan());
            }
        } else {
            println!("{}", response.message);
        }
    }

    /// Show error message
    pub fn show_error(&self, msg: &str) {
        eprintln!("{} {}", "✗".red().bold(), msg.red());
    }

    /// Show install success message
    pub fn show_install_success(&self) {
        println!(
            "{} {}",
            "✓".green().bold(),
            "LaunchAgentをインストールしました".green()
        );
        println!("  次回ログイン時から自動起動します");
    }

    /// Show install failure message
    pub fn show_install_failure(&self, msg: &str) {
        eprintln!(
            "{} {}",
            "✗".red().bold(),
            "LaunchAgentのインストールに失敗しました".red()
        );
        eprintln!("  {}", msg);
    }

    /// Show uninstall success message
    pub fn show_uninstall_success(&self) {
        println!(
            "{} {}",
            "✓".green().bold(),
            "LaunchAgentをアンインストールしました".green()
        );
    }

    /// Show uninstall failure message
    pub fn show_uninstall_failure(&self, msg: &str) {
        eprintln!(
            "{} {}",
            "✗".red().bold(),
            "LaunchAgentのアンインストールに失敗しました".red()
        );
        eprintln!("  {}", msg);
    }
}

impl Default for Display {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ResponseData;

    #[test]
    fn test_display_new() {
        let display = Display::new();
        // Just ensure it can be created
        let _ = display;
    }

    #[test]
    fn test_display_default() {
        let display = Display;
        // Just ensure it can be created
        let _ = display;
    }

    #[test]
    fn test_show_start_success() {
        let display = Display::new();
        let response = IpcResponse::success("Timer started", None);
        // This should not panic
        display.show_start_success(response);
    }

    #[test]
    fn test_show_start_success_with_task() {
        let display = Display::new();
        let response = IpcResponse::success(
            "Timer started",
            Some(ResponseData {
                state: None,
                remaining_seconds: None,
                pomodoro_count: None,
                task_name: Some("Test task".to_string()),
            }),
        );
        // This should not panic
        display.show_start_success(response);
    }

    #[test]
    fn test_show_pause_success() {
        let display = Display::new();
        let response = IpcResponse::success("Timer paused", None);
        // This should not panic
        display.show_pause_success(response);
    }

    #[test]
    fn test_show_resume_success() {
        let display = Display::new();
        let response = IpcResponse::success("Timer resumed", None);
        // This should not panic
        display.show_resume_success(response);
    }

    #[test]
    fn test_show_stop_success() {
        let display = Display::new();
        let response = IpcResponse::success("Timer stopped", None);
        // This should not panic
        display.show_stop_success(response);
    }

    #[test]
    fn test_show_status_with_data() {
        let display = Display::new();
        let response = IpcResponse::success(
            "Status retrieved",
            Some(ResponseData {
                state: Some("working".to_string()),
                remaining_seconds: Some(1500),
                pomodoro_count: Some(2),
                task_name: Some("Test task".to_string()),
            }),
        );
        // This should not panic
        display.show_status(response);
    }

    #[test]
    fn test_show_status_without_data() {
        let display = Display::new();
        let response = IpcResponse::success("No timer running", None);
        // This should not panic
        display.show_status(response);
    }

    #[test]
    fn test_show_error() {
        let display = Display::new();
        // This should not panic
        display.show_error("Test error");
    }

    #[test]
    fn test_show_install_success() {
        let display = Display::new();
        // This should not panic
        display.show_install_success();
    }

    #[test]
    fn test_show_install_failure() {
        let display = Display::new();
        // This should not panic
        display.show_install_failure("Test error");
    }

    #[test]
    fn test_show_uninstall_success() {
        let display = Display::new();
        // This should not panic
        display.show_uninstall_success();
    }

    #[test]
    fn test_show_uninstall_failure() {
        let display = Display::new();
        // This should not panic
        display.show_uninstall_failure("Test error");
    }
}

//! ポモドーロタイマーCLI
//!
//! macOS専用のポモドーロタイマーCLIツール。
//! デーモンプロセスとして動作し、Unix Domain Socket経由でCLIコマンドを受け付ける。

use anyhow::Result;
use clap::Parser;
use pomodoro::cli::{Cli, Commands, Display, IpcClient};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let display = Display::new();
    let client = IpcClient::new();

    match cli.command {
        Commands::Start(args) => match client.start(args).await {
            Ok(response) => {
                if response.status == "success" {
                    display.show_start_success(response);
                } else {
                    display.show_error(&response.message);
                }
            }
            Err(e) => {
                display.show_error(&format!("Failed to start timer: {}", e));
            }
        },
        Commands::Pause => match client.pause().await {
            Ok(response) => {
                if response.status == "success" {
                    display.show_pause_success(response);
                } else {
                    display.show_error(&response.message);
                }
            }
            Err(e) => {
                display.show_error(&format!("Failed to pause timer: {}", e));
            }
        },
        Commands::Resume => match client.resume().await {
            Ok(response) => {
                if response.status == "success" {
                    display.show_resume_success(response);
                } else {
                    display.show_error(&response.message);
                }
            }
            Err(e) => {
                display.show_error(&format!("Failed to resume timer: {}", e));
            }
        },
        Commands::Stop => match client.stop().await {
            Ok(response) => {
                if response.status == "success" {
                    display.show_stop_success(response);
                } else {
                    display.show_error(&response.message);
                }
            }
            Err(e) => {
                display.show_error(&format!("Failed to stop timer: {}", e));
            }
        },
        Commands::Status => match client.status().await {
            Ok(response) => {
                if response.status == "success" {
                    display.show_status(response);
                } else {
                    display.show_error(&response.message);
                }
            }
            Err(e) => {
                display.show_error(&format!("Failed to get status: {}", e));
            }
        },
        Commands::Install => {
            // TODO: Implement LaunchAgent installation
            display.show_error("Install command not yet implemented");
        }
        Commands::Uninstall => {
            // TODO: Implement LaunchAgent uninstallation
            display.show_error("Uninstall command not yet implemented");
        }
        Commands::Daemon => {
            // TODO: Implement daemon mode
            display.show_error("Daemon mode not yet implemented");
        }
        Commands::Completions { shell } => {
            // TODO: Implement shell completions generation
            display.show_error(&format!("Completions for {:?} not yet implemented", shell));
        }
    }

    Ok(())
}

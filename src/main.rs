//! ポモドーロタイマーCLI
//!
//! macOS専用のポモドーロタイマーCLIツール。
//! デーモンプロセスとして動作し、Unix Domain Socket経由でCLIコマンドを受け付ける。

use anyhow::Result;
use clap::Parser;
use pomodoro::cli::{generate_completions, Cli, Commands, Display, IpcClient};

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
        Commands::Install => match pomodoro::launchagent::install() {
            Ok(_) => display.show_success("LaunchAgent installed successfully"),
            Err(e) => display.show_error(&format!("Failed to install LaunchAgent: {}", e)),
        },
        Commands::Uninstall => match pomodoro::launchagent::uninstall() {
            Ok(_) => display.show_success("LaunchAgent uninstalled successfully"),
            Err(e) => display.show_error(&format!("Failed to uninstall LaunchAgent: {}", e)),
        },
        Commands::Daemon => {
            // デーモン設定の初期化
            let config = pomodoro::types::PomodoroConfig::default();
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

            // TimerEngineの初期化
            let engine = std::sync::Arc::new(tokio::sync::Mutex::new(
                pomodoro::daemon::TimerEngine::new(config, tx),
            ));

            // IPCサーバーの初期化
            let socket_path = get_socket_path();
            let server = pomodoro::daemon::IpcServer::new(&socket_path)
                .map_err(|e| anyhow::anyhow!("Failed to create IPC server: {}", e))?;

            println!("Daemon started at {:?}", socket_path);

            // NotificationManagerの初期化 (macOSのみ)
            #[cfg(target_os = "macos")]
            let notification_manager = {
                // メインスレッドマーカーの取得
                let mtm = objc2::MainThreadMarker::new()
                    .expect("Daemon must run on the main thread for UI/Notification operations");

                match pomodoro::notification::NotificationManager::new(mtm) {
                    Ok(nm) => Some(nm),
                    Err(e) => {
                        eprintln!("Failed to initialize NotificationManager: {}", e);
                        None
                    }
                }
            };

            // SoundPlayerの初期化
            let sound_player = pomodoro::sound::create_sound_player(false);

            // メインループ
            loop {
                tokio::select! {
                    // IPCリクエスト処理
                    result = server.accept() => {
                        match result {
                            Ok(mut stream) => {
                                let engine = engine.clone();
                                tokio::spawn(async move {
                                    match pomodoro::daemon::IpcServer::receive_request(&mut stream).await {
                                        Ok(request) => {
                                            let response = pomodoro::daemon::handle_request(request, engine).await;
                                            if let Err(e) = pomodoro::daemon::IpcServer::send_response(&mut stream, &response).await {
                                                eprintln!("Failed to send response: {}", e);
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to receive request: {}", e);
                                        }
                                    }
                                });
                            }
                            Err(e) => {
                                eprintln!("Failed to accept connection: {}", e);
                                // 致命的なエラーでなければ続行
                                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                            }
                        }
                    }

                    // タイマーイベント処理
                    Some(event) = rx.recv() => {
                        println!("Event received: {:?}", event);
                        match event {
                            pomodoro::daemon::TimerEvent::WorkCompleted { .. } => {
                                #[cfg(target_os = "macos")]
                                if let Some(nm) = &notification_manager {
                                    if let Err(e) = nm.send_work_complete_notification(Some("作業完了")) {
                                        eprintln!("Failed to send notification: {}", e);
                                    }
                                }

                                if let Err(e) = sound_player.play(&pomodoro::sound::SoundSource::Embedded { name: "default".to_string() }).await {
                                    eprintln!("Failed to play sound: {}", e);
                                }
                            }
                            pomodoro::daemon::TimerEvent::BreakCompleted { .. } => {
                                #[cfg(target_os = "macos")]
                                if let Some(nm) = &notification_manager {
                                    if let Err(e) = nm.send_break_complete_notification(None) {
                                        eprintln!("Failed to send notification: {}", e);
                                    }
                                }

                                if let Err(e) = sound_player.play(&pomodoro::sound::SoundSource::Embedded { name: "default".to_string() }).await {
                                    eprintln!("Failed to play sound: {}", e);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        Commands::Completions { shell } => {
            generate_completions(shell);
        }
    }

    Ok(())
}

/// ソケットパスを取得
fn get_socket_path() -> std::path::PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());

    std::path::PathBuf::from(home)
        .join(".pomodoro")
        .join("pomodoro.sock")
}

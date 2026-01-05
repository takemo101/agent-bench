use clap::{Args, Parser, Subcommand};

/// Pomodoro Timer CLI
#[derive(Parser, Debug)]
#[command(
    name = "pomodoro",
    version,
    about = "macOS専用ポモドーロタイマーCLI",
    long_about = "ターミナル上で動作するシンプルなポモドーロタイマー。\nmacOSのネイティブ機能（通知、メニューバー、フォーカスモード）と統合されています。",
    propagate_version = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// 詳細ログを出力
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

/// Subcommand definitions
#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// タイマーを開始
    Start(StartArgs),

    /// タイマーを一時停止
    Pause,

    /// タイマーを再開
    Resume,

    /// タイマーを停止
    Stop,

    /// 現在のステータスを確認
    Status,

    /// LaunchAgentをインストール（ログイン時自動起動）
    Install,

    /// LaunchAgentをアンインストール
    Uninstall,

    /// デーモンモードで起動（LaunchAgentから呼ばれる）
    #[command(hide = true)]
    Daemon,

    /// シェル補完スクリプトを生成
    Completions {
        /// シェルの種類
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },

    /// サウンド設定を管理
    Config(ConfigArgs),

    /// システムサウンド一覧を表示
    Sounds,
}

/// Config command arguments
#[derive(Args, Debug, Clone)]
pub struct ConfigArgs {
    /// 作業完了時のサウンドを設定
    #[arg(long)]
    pub work_sound: Option<String>,

    /// 休憩完了時のサウンドを設定
    #[arg(long)]
    pub break_sound: Option<String>,
}

/// start command arguments
#[derive(Args, Debug, Clone)]
pub struct StartArgs {
    /// 作業時間（分）
    #[arg(short, long, default_value = "25", value_parser = clap::value_parser!(u32).range(1..=120))]
    pub work: u32,

    /// 短い休憩時間（分）
    #[arg(short = 'b', long = "break", default_value = "5", value_parser = clap::value_parser!(u32).range(1..=60))]
    pub break_time: u32,

    /// 長い休憩時間（分）
    #[arg(short, long, default_value = "15", value_parser = clap::value_parser!(u32).range(1..=60))]
    pub long_break: u32,

    /// タスク名
    #[arg(short, long, value_parser = validate_task_name)]
    pub task: Option<String>,

    /// 自動サイクル（休憩後に自動的に次の作業を開始）
    #[arg(short, long)]
    pub auto_cycle: bool,

    /// フォーカスモード連携（作業中にフォーカスモードON）
    #[arg(short, long)]
    pub focus_mode: bool,

    /// 通知音を無効化
    #[arg(long)]
    pub no_sound: bool,
}

/// Task name validation
fn validate_task_name(s: &str) -> Result<String, String> {
    if s.is_empty() {
        return Err("タスク名は空にできません".to_string());
    }
    if s.len() > 100 {
        return Err("タスク名は100文字以内にしてください".to_string());
    }
    Ok(s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_parse_start_command_default() {
        let args = vec!["pomodoro", "start"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Start(start_args) => {
                assert_eq!(start_args.work, 25);
                assert_eq!(start_args.break_time, 5);
                assert_eq!(start_args.long_break, 15);
                assert!(start_args.task.is_none());
                assert!(!start_args.auto_cycle);
                assert!(!start_args.focus_mode);
                assert!(!start_args.no_sound);
            }
            _ => panic!("Expected Start command"),
        }
    }

    #[test]
    fn test_parse_start_command_with_args() {
        let args = vec![
            "pomodoro",
            "start",
            "--work",
            "30",
            "--task",
            "テスト",
            "--auto-cycle",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Start(start_args) => {
                assert_eq!(start_args.work, 30);
                assert_eq!(start_args.task, Some("テスト".to_string()));
                assert!(start_args.auto_cycle);
            }
            _ => panic!("Expected Start command"),
        }
    }

    #[test]
    fn test_parse_pause_command() {
        let args = vec!["pomodoro", "pause"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert!(matches!(cli.command, Commands::Pause));
    }

    #[test]
    fn test_parse_resume_command() {
        let args = vec!["pomodoro", "resume"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert!(matches!(cli.command, Commands::Resume));
    }

    #[test]
    fn test_parse_stop_command() {
        let args = vec!["pomodoro", "stop"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert!(matches!(cli.command, Commands::Stop));
    }

    #[test]
    fn test_parse_status_command() {
        let args = vec!["pomodoro", "status"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert!(matches!(cli.command, Commands::Status));
    }

    #[test]
    fn test_validate_task_name_valid() {
        let result = validate_task_name("テスト");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "テスト");
    }

    #[test]
    fn test_validate_task_name_empty() {
        let result = validate_task_name("");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("空にできません"));
    }

    #[test]
    fn test_validate_task_name_too_long() {
        let long_name = "a".repeat(101);
        let result = validate_task_name(&long_name);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("100文字以内"));
    }

    #[test]
    fn test_work_time_range_validation() {
        // Valid: 1
        let args = vec!["pomodoro", "start", "--work", "1"];
        assert!(Cli::try_parse_from(args).is_ok());

        // Valid: 120
        let args = vec!["pomodoro", "start", "--work", "120"];
        assert!(Cli::try_parse_from(args).is_ok());

        // Invalid: 0
        let args = vec!["pomodoro", "start", "--work", "0"];
        assert!(Cli::try_parse_from(args).is_err());

        // Invalid: 121
        let args = vec!["pomodoro", "start", "--work", "121"];
        assert!(Cli::try_parse_from(args).is_err());
    }

    #[test]
    fn test_break_time_range_validation() {
        // Valid: 1
        let args = vec!["pomodoro", "start", "--break", "1"];
        assert!(Cli::try_parse_from(args).is_ok());

        // Valid: 60
        let args = vec!["pomodoro", "start", "--break", "60"];
        assert!(Cli::try_parse_from(args).is_ok());

        // Invalid: 0
        let args = vec!["pomodoro", "start", "--break", "0"];
        assert!(Cli::try_parse_from(args).is_err());

        // Invalid: 61
        let args = vec!["pomodoro", "start", "--break", "61"];
        assert!(Cli::try_parse_from(args).is_err());
    }

    #[test]
    fn test_verbose_flag() {
        let args = vec!["pomodoro", "--verbose", "status"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert!(cli.verbose);
    }

    #[test]
    fn test_parse_sounds_command() {
        let args = vec!["pomodoro", "sounds"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert!(matches!(cli.command, Commands::Sounds));
    }

    #[test]
    fn test_parse_config_command() {
        let args = vec!["pomodoro", "config", "--work-sound", "Glass"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Commands::Config(args) = cli.command {
            assert_eq!(args.work_sound, Some("Glass".to_string()));
        } else {
            panic!("Expected Config command");
        }
    }
}

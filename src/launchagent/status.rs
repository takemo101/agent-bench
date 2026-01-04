//! サービス状態確認モジュール
//!
//! LaunchAgentサービスの実行状態を確認する機能を提供する。

use super::error::{LaunchAgentError, Result};
use super::launchctl;
use super::plist::DEFAULT_LABEL;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ServiceStatus {
    pub running: bool,
    pub pid: Option<u32>,
    pub status_code: Option<i32>,
}

impl ServiceStatus {
    pub fn not_running() -> Self {
        Self::default()
    }

    pub fn running_with_pid(pid: u32) -> Self {
        Self {
            running: true,
            pid: Some(pid),
            status_code: Some(0),
        }
    }
}

pub fn is_running() -> Result<bool> {
    match launchctl::list(DEFAULT_LABEL) {
        Ok(_) => Ok(true),
        Err(LaunchAgentError::LaunchctlExecution(msg)) if msg.contains("not found") => Ok(false),
        Err(e) => Err(e),
    }
}

pub fn get_status() -> Result<ServiceStatus> {
    match launchctl::list(DEFAULT_LABEL) {
        Ok(output) => Ok(parse_launchctl_output(&output)),
        Err(LaunchAgentError::LaunchctlExecution(msg)) if msg.contains("not found") => {
            Ok(ServiceStatus::not_running())
        }
        Err(e) => Err(e),
    }
}

fn parse_launchctl_output(output: &str) -> ServiceStatus {
    let pid = extract_field(output, "PID").and_then(|s| s.parse::<u32>().ok());
    let status_code = extract_field(output, "LastExitStatus").and_then(|s| s.parse::<i32>().ok());

    ServiceStatus {
        running: pid.is_some(),
        pid,
        status_code,
    }
}

fn extract_field<'a>(output: &'a str, field: &str) -> Option<&'a str> {
    let pattern = format!("\"{}\" = ", field);
    output
        .lines()
        .find(|line| line.contains(&pattern))
        .and_then(|line| {
            line.split('=')
                .nth(1)
                .map(|s| s.trim().trim_end_matches(';').trim_matches('"'))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_status_default() {
        let status = ServiceStatus::default();
        assert!(!status.running);
        assert!(status.pid.is_none());
        assert!(status.status_code.is_none());
    }

    #[test]
    fn test_service_status_not_running() {
        let status = ServiceStatus::not_running();
        assert!(!status.running);
        assert!(status.pid.is_none());
        assert!(status.status_code.is_none());
    }

    #[test]
    fn test_service_status_running_with_pid() {
        let status = ServiceStatus::running_with_pid(12345);
        assert!(status.running);
        assert_eq!(status.pid, Some(12345));
        assert_eq!(status.status_code, Some(0));
    }

    #[test]
    fn test_parse_launchctl_output_with_pid() {
        let output = r#"{
    "Label" = "com.github.takemo101.pomodoro";
    "LimitLoadToSessionType" = "Aqua";
    "OnDemand" = false;
    "LastExitStatus" = 0;
    "PID" = 12345;
    "Program" = "/usr/local/bin/pomodoro";
};"#;

        let status = parse_launchctl_output(output);
        assert!(status.running);
        assert_eq!(status.pid, Some(12345));
        assert_eq!(status.status_code, Some(0));
    }

    #[test]
    fn test_parse_launchctl_output_without_pid() {
        let output = r#"{
    "Label" = "com.github.takemo101.pomodoro";
    "LastExitStatus" = 1;
};"#;

        let status = parse_launchctl_output(output);
        assert!(!status.running);
        assert!(status.pid.is_none());
        assert_eq!(status.status_code, Some(1));
    }

    #[test]
    fn test_parse_launchctl_output_empty() {
        let status = parse_launchctl_output("");
        assert!(!status.running);
        assert!(status.pid.is_none());
        assert!(status.status_code.is_none());
    }

    #[test]
    fn test_extract_field_pid() {
        let output = r#"    "PID" = 12345;"#;
        assert_eq!(extract_field(output, "PID"), Some("12345"));
    }

    #[test]
    fn test_extract_field_last_exit_status() {
        let output = r#"    "LastExitStatus" = 0;"#;
        assert_eq!(extract_field(output, "LastExitStatus"), Some("0"));
    }

    #[test]
    fn test_extract_field_not_found() {
        let output = r#"    "Label" = "test";"#;
        assert_eq!(extract_field(output, "PID"), None);
    }

    #[test]
    fn test_extract_field_string_value() {
        let output = r#"    "Label" = "com.github.takemo101.pomodoro";"#;
        assert_eq!(
            extract_field(output, "Label"),
            Some("com.github.takemo101.pomodoro")
        );
    }

    #[test]
    #[ignore = "Requires launchctl; run manually on macOS"]
    fn test_is_running_when_not_installed() {
        let result = is_running();
        assert!(result.is_ok());
    }

    #[test]
    #[ignore = "Requires launchctl; run manually on macOS"]
    fn test_get_status_when_not_installed() {
        let result = get_status();
        assert!(result.is_ok());
        let status = result.unwrap();
        assert!(!status.running);
    }
}

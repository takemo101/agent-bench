//! Shell completion script generation module
//!
//! This module provides functionality to generate shell completion scripts
//! for bash, zsh, and fish shells using clap_complete.

use clap::CommandFactory;
use clap_complete::{generate, Shell};
use std::io;

use crate::cli::Cli;

/// Generate shell completion script to stdout
///
/// # Arguments
///
/// * `shell` - The target shell (bash, zsh, fish, etc.)
///
/// # Example
///
/// ```bash
/// # Generate bash completion
/// pomodoro completions bash > ~/.bash_completion.d/pomodoro
///
/// # Generate zsh completion
/// pomodoro completions zsh > ~/.zsh/completions/_pomodoro
///
/// # Generate fish completion
/// pomodoro completions fish > ~/.config/fish/completions/pomodoro.fish
/// ```
pub fn generate_completions(shell: Shell) {
    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();
    generate(shell, &mut cmd, bin_name, &mut io::stdout());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// Helper to capture generated completions as string
    fn generate_completions_to_string(shell: Shell) -> String {
        let mut buf = Vec::new();
        let mut cmd = Cli::command();
        let bin_name = cmd.get_name().to_string();
        generate(shell, &mut cmd, bin_name, &mut buf);
        String::from_utf8(buf).expect("Generated completions should be valid UTF-8")
    }

    #[test]
    fn test_bash_completions_generated() {
        let output = generate_completions_to_string(Shell::Bash);
        // Bash completions should contain the binary name and completion function
        assert!(output.contains("pomodoro"), "Should contain binary name");
        assert!(
            output.contains("_pomodoro") || output.contains("complete"),
            "Should contain bash completion markers"
        );
    }

    #[test]
    fn test_zsh_completions_generated() {
        let output = generate_completions_to_string(Shell::Zsh);
        // Zsh completions should contain the binary name
        assert!(output.contains("pomodoro"), "Should contain binary name");
        assert!(
            output.contains("#compdef") || output.contains("_arguments"),
            "Should contain zsh completion markers"
        );
    }

    #[test]
    fn test_fish_completions_generated() {
        let output = generate_completions_to_string(Shell::Fish);
        // Fish completions should contain the binary name
        assert!(output.contains("pomodoro"), "Should contain binary name");
        assert!(
            output.contains("complete") && output.contains("-c"),
            "Should contain fish completion markers"
        );
    }

    #[test]
    fn test_completions_contain_subcommands() {
        let output = generate_completions_to_string(Shell::Bash);
        // Should contain main subcommands
        assert!(
            output.contains("start"),
            "Should contain 'start' subcommand"
        );
        assert!(
            output.contains("pause"),
            "Should contain 'pause' subcommand"
        );
        assert!(
            output.contains("resume"),
            "Should contain 'resume' subcommand"
        );
        assert!(output.contains("stop"), "Should contain 'stop' subcommand");
        assert!(
            output.contains("status"),
            "Should contain 'status' subcommand"
        );
    }
}

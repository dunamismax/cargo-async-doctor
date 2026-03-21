use std::ffi::OsString;
use std::path::PathBuf;

use clap::{error::ErrorKind, Args, Parser, Subcommand, ValueEnum};
use serde::Serialize;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum MessageFormat {
    #[default]
    Human,
    Json,
}

#[derive(Debug, Clone, Eq, PartialEq, Subcommand)]
pub enum Command {
    /// Explain a shipped check ID and its current detection scope.
    Explain(ExplainCommand),
}

#[derive(Debug, Clone, Eq, PartialEq, Args)]
pub struct ExplainCommand {
    /// Stable check ID to explain.
    pub check_id: String,
}

#[derive(Debug, Clone, Parser, Eq, PartialEq)]
#[command(
    name = "cargo-async-doctor",
    bin_name = "cargo async-doctor",
    about = "Detect common async Rust hazards in Cargo projects.",
    long_about = None
)]
pub struct Cli {
    /// Emit results as human-readable text or structured JSON.
    #[arg(long, global = true, value_enum, default_value_t)]
    pub message_format: MessageFormat,

    #[command(subcommand)]
    pub command: Option<Command>,

    /// When scanning, inspect the full Cargo workspace instead of the selected package or default members.
    #[arg(long)]
    pub workspace: bool,

    /// When scanning, use a specific Cargo.toml manifest path.
    #[arg(long, value_name = "PATH")]
    pub manifest_path: Option<PathBuf>,
}

pub fn try_parse_from<I, T>(args: I) -> Result<Cli, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString>,
{
    let cli = Cli::try_parse_from(normalize_cargo_subcommand_args(args))?;
    validate_cli(cli)
}

fn normalize_cargo_subcommand_args<I, T>(args: I) -> Vec<OsString>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString>,
{
    let mut normalized: Vec<OsString> = args.into_iter().map(Into::into).collect();

    if matches!(normalized.get(1), Some(arg) if arg == "async-doctor") {
        normalized.remove(1);
    }

    normalized
}

fn validate_cli(cli: Cli) -> Result<Cli, clap::Error> {
    if matches!(cli.command, Some(Command::Explain(_)))
        && (cli.workspace || cli.manifest_path.is_some())
    {
        return Err(clap::Error::raw(
            ErrorKind::ArgumentConflict,
            "`--workspace` and `--manifest-path` are scan-only options and cannot be used with `explain`.",
        ));
    }

    Ok(cli)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use clap::error::ErrorKind;

    use super::{try_parse_from, Cli, Command, ExplainCommand, MessageFormat};

    #[test]
    fn parses_direct_binary_invocation() {
        let cli = try_parse_from(["cargo-async-doctor", "--message-format", "json"]).unwrap();

        assert_eq!(
            cli,
            Cli {
                message_format: MessageFormat::Json,
                command: None,
                workspace: false,
                manifest_path: None,
            }
        );
    }

    #[test]
    fn strips_cargo_subcommand_name_when_present() {
        let cli = try_parse_from([
            "cargo-async-doctor",
            "async-doctor",
            "--workspace",
            "--message-format",
            "human",
        ])
        .unwrap();

        assert_eq!(
            cli,
            Cli {
                message_format: MessageFormat::Human,
                command: None,
                workspace: true,
                manifest_path: None,
            }
        );
    }

    #[test]
    fn parses_explain_subcommand_via_cargo_style_invocation() {
        let cli = try_parse_from([
            "cargo-async-doctor",
            "async-doctor",
            "explain",
            "blocking-sleep-in-async",
            "--message-format",
            "json",
        ])
        .unwrap();

        assert_eq!(
            cli,
            Cli {
                message_format: MessageFormat::Json,
                command: Some(Command::Explain(ExplainCommand {
                    check_id: "blocking-sleep-in-async".to_string(),
                })),
                workspace: false,
                manifest_path: None,
            }
        );
    }

    #[test]
    fn rejects_scan_only_flags_for_explain() {
        let error = try_parse_from([
            "cargo-async-doctor",
            "--manifest-path",
            "fixtures/placeholder/minimal-bin/Cargo.toml",
            "explain",
            "blocking-sleep-in-async",
        ])
        .unwrap_err();

        assert_eq!(error.kind(), ErrorKind::ArgumentConflict);
        assert!(error
            .to_string()
            .contains("scan-only options and cannot be used with `explain`"));
    }

    #[test]
    fn preserves_manifest_path_for_scan_mode() {
        let cli = try_parse_from([
            "cargo-async-doctor",
            "--manifest-path",
            "fixtures/placeholder/minimal-bin/Cargo.toml",
        ])
        .unwrap();

        assert_eq!(
            cli,
            Cli {
                message_format: MessageFormat::Human,
                command: None,
                workspace: false,
                manifest_path: Some(PathBuf::from("fixtures/placeholder/minimal-bin/Cargo.toml",)),
            }
        );
    }
}

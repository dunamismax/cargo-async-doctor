use std::ffi::OsString;
use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use serde::Serialize;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum MessageFormat {
    #[default]
    Human,
    Json,
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
    #[arg(long, value_enum, default_value_t)]
    pub message_format: MessageFormat,

    /// Scan the full Cargo workspace when supported.
    #[arg(long)]
    pub workspace: bool,

    /// Path to a Cargo.toml manifest to inspect.
    #[arg(long, value_name = "PATH")]
    pub manifest_path: Option<PathBuf>,
}

pub fn try_parse_from<I, T>(args: I) -> Result<Cli, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString>,
{
    Cli::try_parse_from(normalize_cargo_subcommand_args(args))
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

#[cfg(test)]
mod tests {
    use super::{try_parse_from, Cli, MessageFormat};

    #[test]
    fn parses_direct_binary_invocation() {
        let cli = try_parse_from(["cargo-async-doctor", "--message-format", "json"]).unwrap();

        assert_eq!(
            cli,
            Cli {
                message_format: MessageFormat::Json,
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
                workspace: true,
                manifest_path: None,
            }
        );
    }
}

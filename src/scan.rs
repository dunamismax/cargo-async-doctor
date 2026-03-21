use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::analysis::{self, Finding};
use crate::cli::Cli;
use crate::diagnostics::{CheckId, Diagnostic, ScanReport, ScanSummary, ScanTarget, Severity};

pub fn scan(cli: &Cli) -> Result<ScanReport> {
    let manifest_path = resolve_manifest_path(cli)?;
    let manifest_dir = manifest_path
        .parent()
        .context("manifest path did not have a parent directory")?;
    let analysis = analysis::analyze_manifest(&manifest_path, cli.workspace)?;
    let diagnostics: Vec<Diagnostic> = analysis
        .findings
        .into_iter()
        .map(|finding| diagnostic_from_finding(manifest_dir, finding))
        .collect();
    let warnings = diagnostics.len();

    Ok(ScanReport {
        schema_version: 1,
        target: ScanTarget {
            workspace: cli.workspace,
            manifest_path: Some(manifest_path.to_string_lossy().into_owned()),
        },
        summary: ScanSummary {
            total: warnings,
            warnings,
        },
        diagnostics,
        placeholder: false,
        notes: analysis.notes,
    })
}

fn resolve_manifest_path(cli: &Cli) -> Result<PathBuf> {
    match &cli.manifest_path {
        Some(path) => Ok(path.clone()),
        None => Ok(std::env::current_dir()
            .context("failed to read the current working directory")?
            .join("Cargo.toml")),
    }
}

fn diagnostic_from_finding(manifest_dir: &Path, finding: Finding) -> Diagnostic {
    let relative_path = finding
        .file
        .strip_prefix(manifest_dir)
        .unwrap_or(finding.file.as_path())
        .display()
        .to_string();

    let message = match finding.id {
        CheckId::BlockingSleepInAsync => format!(
            "`{relative_path}` calls `{}` inside an async context, which blocks the current thread.",
            finding.matched
        ),
        CheckId::BlockingStdApiInAsync => format!(
            "`{relative_path}` calls `{}` inside an async context, which performs blocking filesystem I/O.",
            finding.matched
        ),
        CheckId::SyncAsyncBridgeHazard => format!(
            "`{relative_path}` uses `{}` inside an async context, which synchronously waits on async work.",
            finding.matched
        ),
        CheckId::GuardAcrossAwait => format!(
            "`{relative_path}` matched `{}`.",
            finding.matched
        ),
    };

    let help = finding
        .id
        .help()
        .map(|help| format!("{} {}", finding.id.explanation(), help));

    Diagnostic {
        id: finding.id,
        severity: Severity::Warning,
        message,
        help,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::cli::{Cli, MessageFormat};
    use crate::diagnostics::CheckId;

    use super::scan;

    #[test]
    fn scan_returns_real_report_for_safe_fixture() {
        let cli = Cli {
            message_format: MessageFormat::Human,
            command: None,
            workspace: false,
            manifest_path: Some(PathBuf::from("fixtures/placeholder/minimal-bin/Cargo.toml")),
        };

        let report = scan(&cli).unwrap();

        assert!(!report.placeholder);
        assert!(report.diagnostics.is_empty());
        assert_eq!(report.summary.total, 0);
        assert_eq!(report.summary.warnings, 0);
        assert!(!report.target.workspace);
        assert_eq!(
            report.target.manifest_path.as_deref(),
            Some("fixtures/placeholder/minimal-bin/Cargo.toml")
        );
    }

    #[test]
    fn scan_emits_workspace_note_without_expanding_workspace_members() {
        let cli = Cli {
            message_format: MessageFormat::Human,
            command: None,
            workspace: true,
            manifest_path: Some(PathBuf::from(
                "fixtures/phase2/blocking-sleep-positive/Cargo.toml",
            )),
        };

        let report = scan(&cli).unwrap();

        assert_eq!(report.summary.warnings, 1);
        assert_eq!(report.diagnostics[0].id, CheckId::BlockingSleepInAsync);
        assert!(report
            .notes
            .iter()
            .any(|note| note.contains("`--workspace`")));
    }
}

use crate::cli::Cli;
use crate::diagnostics::{ScanReport, ScanSummary, ScanTarget};

pub fn scan(cli: &Cli) -> ScanReport {
    let manifest_path = cli
        .manifest_path
        .as_ref()
        .map(|path| path.to_string_lossy().into_owned());

    ScanReport {
        schema_version: 1,
        target: ScanTarget {
            workspace: cli.workspace,
            manifest_path,
        },
        summary: ScanSummary {
            total: 0,
            warnings: 0,
        },
        diagnostics: Vec::new(),
        placeholder: true,
        notes: vec![
            "Phase 1 placeholder scan: command surface and output model are wired, but real async checks arrive in Phase 2."
                .to_string(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::cli::{Cli, MessageFormat};

    use super::scan;

    #[test]
    fn placeholder_scan_returns_successful_empty_report() {
        let cli = Cli {
            message_format: MessageFormat::Human,
            workspace: true,
            manifest_path: Some(PathBuf::from("fixtures/placeholder/minimal-bin/Cargo.toml")),
        };

        let report = scan(&cli);

        assert!(report.placeholder);
        assert!(report.diagnostics.is_empty());
        assert_eq!(report.summary.total, 0);
        assert_eq!(report.summary.warnings, 0);
        assert!(report.target.workspace);
        assert_eq!(
            report.target.manifest_path.as_deref(),
            Some("fixtures/placeholder/minimal-bin/Cargo.toml")
        );
    }
}

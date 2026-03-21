use anyhow::Result;

use crate::cli::MessageFormat;
use crate::diagnostics::ScanReport;
use crate::explain::ExplainReport;

pub fn render_scan_report(message_format: MessageFormat, report: &ScanReport) -> Result<String> {
    match message_format {
        MessageFormat::Human => Ok(render_human_scan(report)),
        MessageFormat::Json => Ok(serde_json::to_string_pretty(report)?),
    }
}

pub fn render_explain_report(
    message_format: MessageFormat,
    report: &ExplainReport,
) -> Result<String> {
    match message_format {
        MessageFormat::Human => Ok(render_human_explain(report)),
        MessageFormat::Json => Ok(serde_json::to_string_pretty(report)?),
    }
}

fn render_human_scan(report: &ScanReport) -> String {
    let target = report
        .target
        .manifest_path
        .as_deref()
        .unwrap_or("current package selection");

    let mut output = String::from("cargo-async-doctor\n");
    output.push_str(&format!("schema-version: {}\n", report.schema_version));
    output.push_str(&format!("target: {target}\n"));
    output.push_str(&format!("workspace: {}\n", report.target.workspace));

    if let Some(workspace_root) = &report.target.workspace_root {
        output.push_str(&format!("workspace-root: {workspace_root}\n"));
    }

    if report.target.packages.is_empty() {
        output.push_str("packages: none\n");
    } else {
        output.push_str("packages:\n");
        for package in &report.target.packages {
            output.push_str(&format!("- {} ({})\n", package.name, package.manifest_path));
        }
    }

    output.push_str(&format!("warnings: {}\n", report.summary.warnings));

    if report.diagnostics.is_empty() {
        output.push_str("\nNo diagnostics emitted.\n");
    } else {
        output.push_str("\nDiagnostics:\n");
        for diagnostic in &report.diagnostics {
            output.push_str(&format!(
                "- [{}] package={} location={}\n",
                diagnostic.id,
                diagnostic.package.name,
                render_location(&diagnostic.location.file_path, diagnostic.location.line)
            ));
            output.push_str(&format!("  message: {}\n", diagnostic.message));

            if let Some(help) = &diagnostic.help {
                output.push_str(&format!("  help: {help}\n"));
            }
        }
    }

    for note in &report.notes {
        output.push_str(&format!("note: {note}\n"));
    }

    output
}

fn render_location(path: &str, line: Option<usize>) -> String {
    match line {
        Some(line) => format!("{path}:{line}"),
        None => path.to_string(),
    }
}

fn render_human_explain(report: &ExplainReport) -> String {
    let mut output = String::from("cargo-async-doctor explain\n");
    output.push_str(&format!("schema-version: {}\n", report.schema_version));
    output.push_str(&format!(
        "requested-check-id: {}\n",
        report.requested_check_id
    ));

    if let Some(explanation) = &report.explanation {
        output.push_str(&format!("title: {}\n", explanation.title));
        output.push_str(&format!("found: {}\n", report.found));
        output.push_str("\nSummary:\n");
        output.push_str(explanation.summary);
        output.push('\n');

        output.push_str("\nThis check currently detects:\n");
        for entry in &explanation.detects {
            output.push_str(&format!("- {entry}\n"));
        }

        output.push_str("\nThis check does not currently detect:\n");
        for entry in &explanation.does_not_detect {
            output.push_str(&format!("- {entry}\n"));
        }

        output.push_str("\nSuggested fix paths:\n");
        for entry in &explanation.suggested_fixes {
            output.push_str(&format!("- {entry}\n"));
        }

        output.push_str("\nReferences:\n");
        for reference in &explanation.references {
            output.push_str(&format!("- {}: {}\n", reference.label, reference.url));
        }
    } else {
        output.push_str(&format!("found: {}\n", report.found));
        output.push_str("\nUnknown check ID. Known shipped check IDs:\n");
        for check_id in &report.known_check_ids {
            output.push_str(&format!("- {check_id}\n"));
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use crate::cli::MessageFormat;
    use crate::diagnostics::{
        CheckId, Diagnostic, DiagnosticLocation, DiagnosticPackage, ScanPackageTarget, ScanReport,
        ScanSummary, ScanTarget, Severity,
    };
    use crate::explain::explain;

    use super::{render_explain_report, render_scan_report};

    fn sample_scan_report() -> ScanReport {
        ScanReport {
            schema_version: 1,
            target: ScanTarget {
                workspace: false,
                manifest_path: Some("fixtures/placeholder/minimal-bin/Cargo.toml".to_string()),
                workspace_root: Some("fixtures/placeholder/minimal-bin".to_string()),
                packages: vec![ScanPackageTarget {
                    name: "fixture-minimal-bin".to_string(),
                    manifest_path: "fixtures/placeholder/minimal-bin/Cargo.toml".to_string(),
                }],
            },
            summary: ScanSummary {
                total: 1,
                warnings: 1,
            },
            diagnostics: vec![Diagnostic {
                id: CheckId::BlockingSleepInAsync,
                severity: Severity::Warning,
                package: DiagnosticPackage {
                    name: "fixture-minimal-bin".to_string(),
                    manifest_path: "fixtures/placeholder/minimal-bin/Cargo.toml".to_string(),
                },
                location: DiagnosticLocation {
                    file_path: "src/main.rs".to_string(),
                    package_path: "src/main.rs".to_string(),
                    line: Some(2),
                    column: Some(5),
                    end_line: Some(2),
                    end_column: Some(17),
                },
                message: "Calls `std::thread::sleep` inside an async context, which blocks the current thread.".to_string(),
                help: Some("help text".to_string()),
            }],
            notes: vec!["sample report".to_string()],
        }
    }

    #[test]
    fn renders_human_scan_report() {
        let rendered = render_scan_report(MessageFormat::Human, &sample_scan_report()).unwrap();

        assert!(rendered.contains("cargo-async-doctor"));
        assert!(rendered.contains("package=fixture-minimal-bin"));
        assert!(rendered.contains("location=src/main.rs:2"));
        assert!(rendered.contains("note: sample report"));
        assert!(!rendered.contains("mode: placeholder scan"));
    }

    #[test]
    fn renders_json_scan_report_without_legacy_placeholder_field() {
        let rendered = render_scan_report(MessageFormat::Json, &sample_scan_report()).unwrap();
        let value: serde_json::Value = serde_json::from_str(&rendered).unwrap();

        assert_eq!(value["schema_version"], serde_json::Value::from(1));
        assert!(value.get("placeholder").is_none());
        assert!(value.get("package").is_none());
        assert!(value["diagnostics"][0]["package"].is_object());
        assert!(value["diagnostics"][0]["location"].is_object());
    }

    #[test]
    fn renders_known_explain_report_for_humans() {
        let report = explain("blocking-sleep-in-async");
        let rendered = render_explain_report(MessageFormat::Human, &report).unwrap();

        assert!(rendered.contains("cargo-async-doctor explain"));
        assert!(rendered.contains("requested-check-id: blocking-sleep-in-async"));
        assert!(rendered.contains("Summary:"));
        assert!(rendered.contains("This check currently detects:"));
        assert!(rendered.contains("Suggested fix paths:"));
        assert!(rendered.contains("References:"));
    }

    #[test]
    fn renders_unknown_explain_report_in_json() {
        let report = explain("not-a-real-check");
        let rendered = render_explain_report(MessageFormat::Json, &report).unwrap();
        let value: serde_json::Value = serde_json::from_str(&rendered).unwrap();

        assert_eq!(value["found"], serde_json::Value::Bool(false));
        assert_eq!(
            value["error"],
            serde_json::Value::String("unknown-check-id".to_string())
        );
        assert_eq!(
            value["known_check_ids"],
            serde_json::json!([
                CheckId::BlockingSleepInAsync.as_str(),
                CheckId::BlockingStdApiInAsync.as_str(),
                CheckId::SyncAsyncBridgeHazard.as_str(),
            ])
        );
    }
}

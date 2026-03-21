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
    output.push_str(&format!("warnings: {}\n", report.summary.warnings));

    if report.placeholder {
        output.push_str("mode: placeholder scan\n");
    }

    if report.diagnostics.is_empty() {
        output.push_str("\nNo diagnostics emitted.\n");
    } else {
        output.push_str("\nDiagnostics:\n");
        for diagnostic in &report.diagnostics {
            output.push_str(&format!("- [{}] {}\n", diagnostic.id, diagnostic.message));

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
    use crate::diagnostics::{CheckId, ScanReport, ScanSummary, ScanTarget};
    use crate::explain::explain;

    use super::{render_explain_report, render_scan_report};

    fn placeholder_report() -> ScanReport {
        ScanReport {
            schema_version: 1,
            target: ScanTarget {
                workspace: false,
                manifest_path: Some("fixtures/placeholder/minimal-bin/Cargo.toml".to_string()),
            },
            summary: ScanSummary {
                total: 0,
                warnings: 0,
            },
            diagnostics: Vec::new(),
            placeholder: true,
            notes: vec!["placeholder report".to_string()],
        }
    }

    #[test]
    fn renders_human_placeholder_report() {
        let rendered = render_scan_report(MessageFormat::Human, &placeholder_report()).unwrap();

        assert!(rendered.contains("cargo-async-doctor"));
        assert!(rendered.contains("mode: placeholder scan"));
        assert!(rendered.contains("No diagnostics emitted."));
        assert!(rendered.contains("note: placeholder report"));
    }

    #[test]
    fn renders_json_placeholder_report() {
        let rendered = render_scan_report(MessageFormat::Json, &placeholder_report()).unwrap();

        assert!(rendered.contains("\"schema_version\": 1"));
        assert!(rendered.contains("\"placeholder\": true"));
        assert!(rendered.contains("\"diagnostics\": []"));
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

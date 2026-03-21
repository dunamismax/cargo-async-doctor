use anyhow::Result;

use crate::cli::MessageFormat;
use crate::diagnostics::ScanReport;

pub fn render_report(message_format: MessageFormat, report: &ScanReport) -> Result<String> {
    match message_format {
        MessageFormat::Human => Ok(render_human(report)),
        MessageFormat::Json => Ok(serde_json::to_string_pretty(report)?),
    }
}

fn render_human(report: &ScanReport) -> String {
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

#[cfg(test)]
mod tests {
    use crate::cli::MessageFormat;
    use crate::diagnostics::{ScanReport, ScanSummary, ScanTarget};

    use super::render_report;

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
        let rendered = render_report(MessageFormat::Human, &placeholder_report()).unwrap();

        assert!(rendered.contains("cargo-async-doctor"));
        assert!(rendered.contains("mode: placeholder scan"));
        assert!(rendered.contains("No diagnostics emitted."));
        assert!(rendered.contains("note: placeholder report"));
    }

    #[test]
    fn renders_json_placeholder_report() {
        let rendered = render_report(MessageFormat::Json, &placeholder_report()).unwrap();

        assert!(rendered.contains("\"schema_version\": 1"));
        assert!(rendered.contains("\"placeholder\": true"));
        assert!(rendered.contains("\"diagnostics\": []"));
    }
}

use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};
use std::process::Command as ProcessCommand;

use anyhow::{bail, Context, Result};
use serde::Deserialize;

use crate::analysis::{self, Finding, PackageContext};
use crate::cli::Cli;
use crate::diagnostics::{
    CheckId, Diagnostic, DiagnosticLocation, DiagnosticPackage, ScanPackageTarget, ScanReport,
    ScanSummary, ScanTarget, Severity,
};

pub fn scan(cli: &Cli) -> Result<ScanReport> {
    let manifest_path = resolve_manifest_path(cli)?;
    let selection = select_packages(&manifest_path, cli.workspace)?;

    let mut notes = selection.notes;
    let mut diagnostics = Vec::new();

    for package in &selection.packages {
        let analysis = analysis::analyze_package(package)?;
        notes.extend(analysis.notes);
        diagnostics.extend(analysis.findings.into_iter().map(diagnostic_from_finding));
    }

    diagnostics.sort_by(|left, right| {
        left.package
            .name
            .cmp(&right.package.name)
            .then_with(|| left.location.file_path.cmp(&right.location.file_path))
            .then_with(|| left.location.line.cmp(&right.location.line))
            .then_with(|| left.id.as_str().cmp(right.id.as_str()))
            .then_with(|| left.message.cmp(&right.message))
    });

    let warnings = diagnostics.len();

    Ok(ScanReport {
        schema_version: 1,
        target: ScanTarget {
            workspace: cli.workspace,
            manifest_path: Some(manifest_path.display().to_string()),
            workspace_root: Some(selection.workspace_root.display().to_string()),
            packages: selection
                .packages
                .iter()
                .map(|package| ScanPackageTarget {
                    name: package.name.clone(),
                    manifest_path: package.manifest_path.display().to_string(),
                })
                .collect(),
        },
        summary: ScanSummary {
            total: warnings,
            warnings,
        },
        diagnostics,
        placeholder: false,
        notes,
    })
}

fn resolve_manifest_path(cli: &Cli) -> Result<PathBuf> {
    let current_dir =
        std::env::current_dir().context("failed to read the current working directory")?;
    let path = match &cli.manifest_path {
        Some(path) => {
            if path.is_absolute() {
                path.clone()
            } else {
                current_dir.join(path)
            }
        }
        None => current_dir.join("Cargo.toml"),
    };

    Ok(normalize_path(&path))
}

#[derive(Debug, Clone)]
struct PackageSelection {
    workspace_root: PathBuf,
    packages: Vec<PackageContext>,
    notes: Vec<String>,
}

fn select_packages(manifest_path: &Path, workspace: bool) -> Result<PackageSelection> {
    let metadata = cargo_metadata(manifest_path)?;
    let workspace_root = normalize_path(Path::new(&metadata.workspace_root));
    let workspace_root_manifest = workspace_root.join("Cargo.toml");
    let selected_manifest_path = normalize_path(manifest_path);

    let packages_by_id: BTreeMap<String, MetadataPackage> = metadata
        .packages
        .into_iter()
        .map(|package| (package.id.clone(), package))
        .collect();

    let package_for_manifest = packages_by_id
        .values()
        .find(|package| normalize_path(Path::new(&package.manifest_path)) == selected_manifest_path)
        .cloned();

    let mut notes = Vec::new();

    let selected_packages = if workspace {
        packages_from_ids(
            &packages_by_id,
            &metadata.workspace_members,
            &workspace_root,
        )?
    } else if let Some(package) = package_for_manifest {
        vec![package_context_from_metadata(&package, &workspace_root)]
    } else if selected_manifest_path == workspace_root_manifest {
        let default_member_ids = if metadata.workspace_default_members.is_empty() {
            metadata.workspace_members.clone()
        } else {
            metadata.workspace_default_members.clone()
        };

        notes.push(
            "Selected a virtual workspace manifest without `--workspace`; scanning default workspace members only. Pass `--workspace` to scan every workspace member."
                .to_string(),
        );

        packages_from_ids(&packages_by_id, &default_member_ids, &workspace_root)?
    } else {
        bail!(
            "manifest `{}` was not found in Cargo metadata output",
            selected_manifest_path.display()
        );
    };

    Ok(PackageSelection {
        workspace_root,
        packages: selected_packages,
        notes,
    })
}

fn packages_from_ids(
    packages_by_id: &BTreeMap<String, MetadataPackage>,
    package_ids: &[String],
    workspace_root: &Path,
) -> Result<Vec<PackageContext>> {
    package_ids
        .iter()
        .map(|id| {
            let package = packages_by_id.get(id).with_context(|| {
                format!("workspace member `{id}` missing from cargo metadata packages")
            })?;
            Ok(package_context_from_metadata(package, workspace_root))
        })
        .collect()
}

fn package_context_from_metadata(
    package: &MetadataPackage,
    workspace_root: &Path,
) -> PackageContext {
    let manifest_path = normalize_path(Path::new(&package.manifest_path));
    let root_dir = manifest_path
        .parent()
        .map(normalize_path)
        .unwrap_or_else(|| workspace_root.to_path_buf());

    PackageContext {
        name: package.name.clone(),
        manifest_path,
        root_dir,
        workspace_root: workspace_root.to_path_buf(),
    }
}

fn diagnostic_from_finding(finding: Finding) -> Diagnostic {
    let file_path = display_file_path(&finding);
    let package_path = finding
        .file
        .strip_prefix(&finding.package_root)
        .unwrap_or(finding.file.as_path())
        .display()
        .to_string();

    let line = finding.span.map(|span| span.start_line);
    let column = finding.span.map(|span| span.start_column);
    let end_line = finding.span.map(|span| span.end_line);
    let end_column = finding.span.map(|span| span.end_column);

    let message = match finding.id {
        CheckId::BlockingSleepInAsync => format!(
            "Calls `{}` inside an async context, which blocks the current thread.",
            finding.matched
        ),
        CheckId::BlockingStdApiInAsync => format!(
            "Calls `{}` inside an async context, which performs blocking filesystem I/O.",
            finding.matched
        ),
        CheckId::SyncAsyncBridgeHazard => format!(
            "Uses `{}` inside an async context, which synchronously waits on async work.",
            finding.matched
        ),
        CheckId::GuardAcrossAwait => format!("Matched `{}`.", finding.matched),
    };

    let help = finding
        .id
        .help()
        .map(|help| format!("{} {}", finding.id.explanation(), help));

    Diagnostic {
        id: finding.id,
        severity: Severity::Warning,
        package: DiagnosticPackage {
            name: finding.package_name,
            manifest_path: finding.package_manifest_path.display().to_string(),
        },
        location: DiagnosticLocation {
            file_path,
            package_path,
            line,
            column,
            end_line,
            end_column,
        },
        message,
        help,
    }
}

fn display_file_path(finding: &Finding) -> String {
    finding
        .file
        .strip_prefix(&finding.workspace_root)
        .or_else(|_| finding.file.strip_prefix(&finding.package_root))
        .unwrap_or(finding.file.as_path())
        .display()
        .to_string()
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::RootDir | Component::Prefix(_) | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }

    normalized
}

#[derive(Debug, Deserialize)]
struct Metadata {
    packages: Vec<MetadataPackage>,
    workspace_members: Vec<String>,
    #[serde(default)]
    workspace_default_members: Vec<String>,
    workspace_root: String,
}

#[derive(Debug, Clone, Deserialize)]
struct MetadataPackage {
    id: String,
    name: String,
    manifest_path: String,
}

fn cargo_metadata(manifest_path: &Path) -> Result<Metadata> {
    let output = ProcessCommand::new("cargo")
        .arg("metadata")
        .arg("--format-version")
        .arg("1")
        .arg("--no-deps")
        .arg("--manifest-path")
        .arg(manifest_path)
        .output()
        .with_context(|| {
            format!(
                "failed to invoke `cargo metadata` for `{}`",
                manifest_path.display()
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        bail!(
            "`cargo metadata` failed for `{}`: {}",
            manifest_path.display(),
            if stderr.is_empty() {
                "unknown error"
            } else {
                &stderr
            }
        );
    }

    serde_json::from_slice(&output.stdout).with_context(|| {
        format!(
            "failed to parse `cargo metadata` output for `{}`",
            manifest_path.display()
        )
    })
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
        assert_eq!(report.target.packages.len(), 1);
        assert_eq!(report.target.packages[0].name, "fixture-minimal-bin");
    }

    #[test]
    fn scan_tracks_workspace_member_context_and_line_numbers() {
        let cli = Cli {
            message_format: MessageFormat::Human,
            command: None,
            workspace: true,
            manifest_path: Some(PathBuf::from(
                "fixtures/phase4/workspace-root-package/Cargo.toml",
            )),
        };

        let report = scan(&cli).unwrap();

        assert_eq!(report.summary.warnings, 3);
        assert!(report.target.workspace);
        assert_eq!(report.target.packages.len(), 3);
        assert_eq!(report.diagnostics[0].id, CheckId::BlockingStdApiInAsync);
        assert_eq!(report.diagnostics[0].package.name, "member-bin");
        assert_eq!(
            report.diagnostics[0].location.file_path,
            "member-bin/src/main.rs"
        );
        assert_eq!(report.diagnostics[0].location.line, Some(4));
    }

    #[test]
    fn virtual_workspace_manifest_defaults_to_default_members_without_workspace_flag() {
        let cli = Cli {
            message_format: MessageFormat::Human,
            command: None,
            workspace: false,
            manifest_path: Some(PathBuf::from(
                "fixtures/phase4/virtual-workspace/Cargo.toml",
            )),
        };

        let report = scan(&cli).unwrap();

        assert_eq!(report.summary.warnings, 1);
        assert_eq!(report.target.packages.len(), 1);
        assert_eq!(report.target.packages[0].name, "default-member");
        assert!(report
            .notes
            .iter()
            .any(|note| note.contains("default workspace members only")));
    }
}

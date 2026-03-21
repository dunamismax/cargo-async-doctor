use serde::Serialize;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CheckId {
    BlockingSleepInAsync,
    BlockingStdApiInAsync,
    SyncAsyncBridgeHazard,
    GuardAcrossAwait,
}

impl CheckId {
    pub const ALL: [Self; 4] = [
        Self::BlockingSleepInAsync,
        Self::BlockingStdApiInAsync,
        Self::SyncAsyncBridgeHazard,
        Self::GuardAcrossAwait,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BlockingSleepInAsync => "blocking-sleep-in-async",
            Self::BlockingStdApiInAsync => "blocking-std-api-in-async",
            Self::SyncAsyncBridgeHazard => "sync-async-bridge-hazard",
            Self::GuardAcrossAwait => "guard-across-await",
        }
    }
}

impl std::fmt::Display for CheckId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Warning,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct Diagnostic {
    pub id: CheckId,
    pub severity: Severity,
    pub message: String,
    pub help: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct ScanTarget {
    pub workspace: bool,
    pub manifest_path: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct ScanSummary {
    pub total: usize,
    pub warnings: usize,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct ScanReport {
    pub schema_version: u32,
    pub target: ScanTarget,
    pub summary: ScanSummary,
    pub diagnostics: Vec<Diagnostic>,
    pub placeholder: bool,
    pub notes: Vec<String>,
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::CheckId;

    #[test]
    fn check_ids_are_stable_and_unique() {
        let ids: Vec<&str> = CheckId::ALL.into_iter().map(CheckId::as_str).collect();
        let unique: HashSet<&str> = ids.iter().copied().collect();

        assert_eq!(ids.len(), unique.len());
        assert_eq!(
            ids,
            vec![
                "blocking-sleep-in-async",
                "blocking-std-api-in-async",
                "sync-async-bridge-hazard",
                "guard-across-await",
            ]
        );
    }
}

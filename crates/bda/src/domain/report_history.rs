// BDA Report History Domain Model
// Purpose: Track changes to BDA reports over time

use serde::{Deserialize, Serialize};

/// Change type for report history
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangeType {
    Created,
    Updated,
    Submitted,
    Reviewed,
    Approved,
    Rejected,
}

/// BDA Report History Entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BdaReportHistory {
    pub id: String,
    pub bda_report_id: String,
    pub version_number: i32,
    
    // Snapshot of report data at this version
    pub report_data_json: String,
    
    // Change metadata
    pub changed_by: String,
    pub changed_at: String,  // ISO 8601 timestamp
    pub change_type: ChangeType,
    pub change_description: Option<String>,
    
    // Status at this version
    pub status: String,
}

/// Request to get report history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetReportHistoryRequest {
    pub bda_report_id: String,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

/// Response with report history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportHistoryResponse {
    pub history: Vec<BdaReportHistory>,
    pub total: i32,
    pub latest_version: i32,
}

/// Version comparison result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionComparison {
    pub version1: BdaReportHistory,
    pub version2: BdaReportHistory,
    pub differences: Vec<FieldDifference>,
}

/// Field difference between versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDifference {
    pub field: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub change_type: String,  // "added", "removed", "modified"
}

impl BdaReportHistory {
    /// Parse report data from JSON
    pub fn parse_report_data(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&self.report_data_json)
    }
    
    /// Get formatted change description
    pub fn get_change_summary(&self) -> String {
        match &self.change_description {
            Some(desc) => desc.clone(),
            None => format!("{:?} by {}", self.change_type, self.changed_by),
        }
    }
}

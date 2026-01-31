// BDA Report Template Domain Model
// Purpose: Standardized report templates per joint doctrine

use serde::{Deserialize, Serialize};

/// Report template type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReportTemplateType {
    /// Initial BDA (within 24h of strike)
    Initial,
    /// Interim BDA (24-72h post-strike)
    Interim,
    /// Final BDA (72h+ post-strike)
    Final,
    /// Executive summary
    ExecutiveSummary,
    /// Technical report
    Technical,
    /// Statistical summary
    Statistical,
    /// Lessons learned
    LessonsLearned,
    /// After-action report
    AfterAction,
}

/// Report format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReportFormat {
    /// PDF document
    Pdf,
    /// NITFS (National Imagery Transmission Format Standard)
    Nitfs,
    /// KML (Keyhole Markup Language) for geospatial
    Kml,
    /// JSON for API consumption
    Json,
    /// HTML for web viewing
    Html,
}

/// Classification level for reports
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ReportClassification {
    Unclassified,
    Confidential,
    Secret,
    TopSecret,
    #[serde(rename = "TOP_SECRET_SCI")]
    TopSecretSci,
}

/// BDA Report Template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BdaReportTemplate {
    pub id: String,
    pub template_type: ReportTemplateType,
    pub name: String,
    pub description: Option<String>,
    
    // Template structure
    pub sections: Vec<ReportSection>,
    
    // Metadata
    pub classification: ReportClassification,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Report section definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSection {
    pub id: String,
    pub title: String,
    pub order: i32,
    pub required: bool,
    pub fields: Vec<ReportField>,
}

/// Report field definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportField {
    pub id: String,
    pub label: String,
    pub field_type: ReportFieldType,
    pub source_field: Option<String>,  // Maps to BdaReport field
    pub required: bool,
    pub order: i32,
}

/// Report field type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportFieldType {
    Text,
    Number,
    Percentage,
    Date,
    Enum,
    Boolean,
    Image,
    Table,
    Chart,
}

/// Request to generate report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateReportRequest {
    pub bda_report_id: String,
    pub template_type: ReportTemplateType,
    pub format: ReportFormat,
    pub include_imagery: bool,
    pub include_components: bool,
    pub include_peer_reviews: bool,
    pub include_history: bool,
    pub classification: Option<ReportClassification>,
    pub custom_sections: Option<Vec<String>>,
}

/// Report generation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportGenerationResponse {
    pub report_id: String,
    pub template_type: ReportTemplateType,
    pub format: ReportFormat,
    pub file_url: Option<String>,
    pub file_size_bytes: Option<u64>,
    pub generated_at: String,
    pub generated_by: String,
    pub classification: ReportClassification,
}

impl ReportTemplateType {
    /// Get default template for assessment type
    pub fn from_assessment_type(assessment_type: crate::features::bda::domain::AssessmentType) -> Self {
        match assessment_type {
            crate::features::bda::domain::AssessmentType::Initial => ReportTemplateType::Initial,
            crate::features::bda::domain::AssessmentType::Interim => ReportTemplateType::Interim,
            crate::features::bda::domain::AssessmentType::Final => ReportTemplateType::Final,
        }
    }
    
    /// Get template display name
    pub fn display_name(&self) -> &'static str {
        match self {
            ReportTemplateType::Initial => "Initial BDA Report",
            ReportTemplateType::Interim => "Interim BDA Report",
            ReportTemplateType::Final => "Final BDA Report",
            ReportTemplateType::ExecutiveSummary => "Executive Summary",
            ReportTemplateType::Technical => "Technical Report",
            ReportTemplateType::Statistical => "Statistical Summary",
            ReportTemplateType::LessonsLearned => "Lessons Learned",
            ReportTemplateType::AfterAction => "After-Action Report",
        }
    }
}

impl ReportClassification {
    /// Get classification marking text
    pub fn marking(&self) -> &'static str {
        match self {
            ReportClassification::Unclassified => "UNCLASSIFIED",
            ReportClassification::Confidential => "CONFIDENTIAL",
            ReportClassification::Secret => "SECRET",
            ReportClassification::TopSecret => "TOP SECRET",
            ReportClassification::TopSecretSci => "TOP SECRET//SCI",
        }
    }
    
    /// Get classification banner text (for PDF headers/footers)
    pub fn banner(&self) -> &'static str {
        match self {
            ReportClassification::Unclassified => "UNCLASSIFIED",
            ReportClassification::Confidential => "CONFIDENTIAL",
            ReportClassification::Secret => "SECRET",
            ReportClassification::TopSecret => "TOP SECRET",
            ReportClassification::TopSecretSci => "TOP SECRET//SCI",
        }
    }
}

// BDA Domain Module
// Purpose: Export all domain models and types

pub mod bda_report;
pub mod imagery;
pub mod strike;
pub mod report_history;
pub mod component_assessment;

// Re-export commonly used types
pub use bda_report::{
    BdaReport,
    CreateBdaReportRequest,
    UpdateBdaReportRequest,
    PhysicalDamage,
    FunctionalDamage,
    AssessmentType,
    EffectLevel,
    Recommendation,
    CivcasCredibility,
    WeaponPerformance,
    BdaStatus,
    AssessmentQuality,
    BdaStatistics,
    BdaStatusCounts,
    BdaRecommendationCounts,
    BdaPhysicalDamageCounts,
    ReAttackRecommendation,
};

pub use imagery::{
    BdaImagery,
    CreateBdaImageryRequest,
    SensorType,
};

pub use strike::{
    BdaStrikeCorrelation,
    CreateStrikeCorrelationRequest,
    GuidancePerformance,
    WeaponPerformanceSummary,
};

pub use report_history::{
    BdaReportHistory,
    ChangeType,
    GetReportHistoryRequest,
    ReportHistoryResponse,
    VersionComparison,
    FieldDifference,
};

pub use component_assessment::{
    BdaComponentAssessment,
    CreateComponentAssessmentRequest,
    UpdateComponentAssessmentRequest,
    ComponentType,
    ComponentCriticality,
};

pub mod peer_review;

pub use peer_review::{
    BdaPeerReview,
    CreatePeerReviewRequest,
    UpdatePeerReviewRequest,
    ReviewStatus,
    ReviewPriority,
    OverallQuality,
    ReviewRecommendation,
    ReviewSummary,
};

pub mod report_template;

pub use report_template::{
    BdaReportTemplate,
    ReportSection,
    ReportField,
    ReportTemplateType,
    ReportFormat,
    ReportClassification,
    ReportFieldType,
    GenerateReportRequest,
    ReportGenerationResponse,
};

pub mod distribution;

pub use distribution::{
    BdaDistributionList,
    BdaDistributionMember,
    BdaReportDistribution,
    CreateDistributionListRequest,
    AddDistributionMemberRequest,
    DistributeReportRequest,
    IndividualRecipient,
    DistributionSummary,
    DeliveryStatus,
    DeliveryMethod,
};

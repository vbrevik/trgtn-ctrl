// NATO COPD Targeting Cell - Domain Models
// Aligned with existing database schema

// use chrono::{DateTime, Utc};

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// ============================================================================
// DECISION GATES (Dashboard operational status)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ClassificationLevel {
    Unclass,
    Cui,
    Secret,
    TopSecret,
    #[serde(rename = "TS_SCI")]
    TsSci,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GateStatus {
    Green,
    Yellow,
    Red,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionGate {
    pub name: String,
    pub status: GateStatus,
    pub value: String,
    pub classification: ClassificationLevel,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caveats: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionGatesResponse {
    pub roe: DecisionGate,
    pub cde: DecisionGate,
    pub weather: DecisionGate,
    pub deconfliction: DecisionGate,
}

// ============================================================================
// RE-EXPORT EXISTING TARGET MODEL (uses existing 'targets' table)
// Note: The existing targets table has these columns, we'll use it as-is
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Target {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub target_type: String,
    pub priority: String,
    pub target_status: String,
    pub coordinates: String,
    pub f3ead_stage: Option<String>, // F3EAD cycle stage: FIND, FIX, FINISH, EXPLOIT, ANALYZE, DISSEMINATE
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTargetRequest {
    pub name: String,
    pub description: Option<String>,
    pub target_type: String,
    pub priority: String,
    pub coordinates: String,
    pub classification: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTargetRequest {
    pub name: Option<String>,
    pub priority: Option<String>,
    pub target_status: Option<String>,
}

// ============================================================================
// DTL (DYNAMIC TARGET LIST)
// ============================================================================

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct DtlEntry {
    pub id: String,
    pub target_id: String,
    pub priority_score: f64,
    pub feasibility_score: f64,
    pub combined_score: Option<f64>,
    pub aging_hours: i32,
    pub is_tst: bool,
    pub tst_deadline: Option<String>,
    pub approval_chain: Option<String>,
    pub current_approver: Option<String>,
    pub approval_level: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDtlEntryRequest {
    pub target_id: String,
    pub priority_score: f64,
    pub feasibility_score: f64,
    pub is_tst: bool,
    pub tst_deadline: Option<String>,
}

// ============================================================================
// BDA (Use existing bda_reports table)
// ============================================================================

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct BdaAssessment {
    pub id: String,
    pub target_id: String,
    pub physical_damage: String,
    pub functional_damage: String,
    pub recommendation: String,
    pub confidence: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateBdaRequest {
    pub target_id: String,
    pub physical_damage: String,
    pub functional_damage: String,
    pub recommendation: String,
    pub confidence: f64,
}

// ============================================================================
// ISR PLATFORMS
// ============================================================================

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct IsrPlatform {
    pub id: String,
    pub platform_type: String,
    pub platform_name: String,
    pub callsign: Option<String>,
    pub current_position: Option<String>,
    pub sensor_type: String,
    pub sensor_range_km: Option<f64>,
    pub coverage_area: Option<String>,
    pub status: String,
    pub loiter_time_remaining: Option<i32>,
    pub fuel_remaining_percent: Option<i32>,
    pub current_task: Option<String>,
    pub tasking_priority: Option<String>,
    pub tasked_targets: Option<String>,
    pub classification: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateIsrPlatformRequest {
    pub platform_type: String,
    pub platform_name: String,
    pub callsign: Option<String>,
    pub sensor_type: String,
    pub classification: String,
}

// ============================================================================
// INTELLIGENCE REPORTS
// ============================================================================

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct IntelligenceReport {
    pub id: String,
    pub target_id: Option<String>,
    pub int_type: String,
    pub report_title: String,
    pub report_content: String,
    pub report_summary: Option<String>,
    pub confidence_level: i32,
    pub source_reliability: String,
    pub collection_time: String,
    pub reporting_time: String,
    pub fusion_score: Option<f64>,
    pub pattern_of_life_indicator: bool,
    pub classification: String,
    pub collected_by: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateIntelReportRequest {
    pub target_id: Option<String>,
    pub int_type: String,
    pub report_title: String,
    pub report_content: String,
    pub confidence_level: i32,
    pub source_reliability: String,
    pub collection_time: String,
    pub classification: String,
}

// ============================================================================
// STRIKE PLATFORMS
// ============================================================================

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct StrikePlatform {
    pub id: String,
    pub platform_type: String,
    pub platform_name: String,
    pub callsign: Option<String>,
    pub unit: Option<String>,
    pub munitions_available: Option<String>,
    pub sorties_available: i32,
    pub platform_status: String,
    pub classification: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateStrikePlatformRequest {
    pub platform_type: String,
    pub platform_name: String,
    pub sorties_available: i32,
    pub classification: String,
}

// ============================================================================
// RISK ASSESSMENTS
// ============================================================================

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct RiskAssessment {
    pub id: String,
    pub target_id: String,
    pub fratricide_risk: String,
    pub friendly_forces_distance_km: Option<f64>,
    pub political_sensitivity: String,
    pub sensitive_sites_nearby: Option<String>,
    pub proportionality_assessment: Option<String>,
    pub legal_review_status: String,
    pub legal_reviewer: Option<String>,
    pub legal_review_date: Option<String>,
    pub overall_risk_score: Option<f64>,
    pub classification: String,
    pub assessed_by: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRiskAssessmentRequest {
    pub target_id: String,
    pub fratricide_risk: String,
    pub political_sensitivity: String,
    pub legal_review_status: String,
    pub classification: String,
}

// ============================================================================
// ASSUMPTION CHALLENGES
// ============================================================================

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct AssumptionChallenge {
    pub id: String,
    pub assumption_text: String,
    pub assumption_category: Option<String>,
    pub confidence_level: i32,
    pub validation_status: String,
    pub alternative_hypothesis: Option<String>,
    pub bias_type: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAssumptionChallengeRequest {
    pub assumption_text: String,
    pub confidence_level: i32,
    pub validation_status: String,
}

// ============================================================================
// DECISION LOG
// ============================================================================

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct DecisionLogEntry {
    pub id: String,
    pub decision_type: String,
    pub decision_text: String,
    pub decision_rationale: String,
    pub decision_maker: String,
    pub decision_maker_role: String,
    pub authority_level: String,
    pub classification: String,
    pub decision_time: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDecisionLogRequest {
    pub decision_type: String,
    pub decision_text: String,
    pub decision_rationale: String,
    pub decision_maker_role: String,
    pub authority_level: String,
    pub classification: String,
}

// ============================================================================
// SHIFT HANDOVERS
// ============================================================================

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct ShiftHandover {
    pub id: String,
    pub shift_date: String,
    pub shift_type: String,
    pub shift_start: String,
    pub shift_end: String,
    pub outgoing_watch_officer: String,
    pub incoming_watch_officer: String,
    pub active_targets_summary: String,
    pub pending_decisions: String,
    pub classification: String,
    pub handover_time: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateShiftHandoverRequest {
    pub shift_date: String,
    pub shift_type: String,
    pub shift_start: String,
    pub shift_end: String,
    pub incoming_watch_officer: String,
    pub classification: String,
}

// ============================================================================
// JTB (JOINT TARGETING BOARD)
// ============================================================================

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct JtbSession {
    pub id: String,
    pub session_name: String,
    pub session_date: String,
    pub session_time: String,
    pub session_datetime: String,
    pub chair: String,
    pub chair_rank: Option<String>,
    pub status: String,
    pub required_attendees: Option<String>,
    pub actual_attendees: Option<String>,
    pub quorum_verified: bool,
    pub protocol_notes: Option<String>,
    pub session_minutes: Option<String>,
    pub classification: String,
    pub caveats: Option<String>,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateJtbSessionRequest {
    pub session_name: String,
    pub session_date: String,
    pub session_time: String,
    pub session_datetime: String,
    pub chair: String,
    pub chair_rank: Option<String>,
    pub required_attendees: Option<Vec<String>>,
    pub classification: String,
    pub caveats: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct JtbTarget {
    pub id: String,
    pub session_id: String,
    pub target_id: String,
    pub presentation_order: i32,
    pub decision: Option<String>,
    pub decision_rationale: Option<String>,
    pub decided_by: Option<String>,
    pub decided_at: Option<String>,
    pub votes_for: i32,
    pub votes_against: i32,
    pub votes_abstain: i32,
    pub approval_conditions: Option<String>,
    pub mitigation_requirements: Option<String>,
    pub added_to_session_at: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddTargetToSessionRequest {
    pub target_id: String,
    pub presentation_order: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecordJtbDecisionRequest {
    pub decision: String, // APPROVED, REJECTED, DEFERRED, PENDING
    pub decision_rationale: String,
    pub decided_by: String,
    pub votes_for: Option<i32>,
    pub votes_against: Option<i32>,
    pub votes_abstain: Option<i32>,
    pub approval_conditions: Option<String>,
    pub mitigation_requirements: Option<String>,
}

// ============================================================================
// TARGETING ANNOTATIONS
// ============================================================================

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct TargetingAnnotation {
    pub id: String,
    pub target_id: Option<String>,
    pub user_id: String,
    pub annotation_text: String,
    pub annotation_type: String,
    pub is_critical: bool,
    pub classification: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAnnotationRequest {
    pub target_id: Option<String>,
    pub annotation_text: String,
    pub annotation_type: String,
    pub is_critical: bool,
    pub classification: String,
}

// ============================================================================
// SUMMARY TYPES
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct TargetingSummary {
    pub total_targets: i64,
    pub active_targets: i64,
    pub pending_nominations: i64,
    pub approved_targets: i64,
}

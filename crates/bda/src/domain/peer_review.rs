// BDA Peer Review Domain Model
// Purpose: Multi-reviewer quality control workflow

use serde::{Deserialize, Serialize};

/// Review status workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
}

/// Review priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReviewPriority {
    Normal,
    Urgent,
    Critical,
}

/// Overall quality assessment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OverallQuality {
    High,
    Medium,
    Low,
    NeedsRework,
}

/// Review recommendation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewRecommendation {
    Approve,
    ApproveWithChanges,
    Reject,
    RequestClarification,
}

/// BDA Peer Review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BdaPeerReview {
    pub id: String,
    pub bda_report_id: String,
    
    // Reviewer Information
    pub reviewer_id: String,
    pub reviewer_name: Option<String>,
    pub reviewer_role: Option<String>,
    
    // Review Assignment
    pub assigned_by: String,
    pub assigned_at: String,  // ISO 8601 timestamp
    pub due_date: Option<String>,
    pub priority: ReviewPriority,
    
    // Review Status
    pub review_status: ReviewStatus,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    
    // Review Assessment
    pub overall_quality: OverallQuality,
    pub confidence_adequate: Option<bool>,
    pub evidence_sufficient: Option<bool>,
    pub methodology_sound: Option<bool>,
    pub recommendations_appropriate: Option<bool>,
    
    // Review Feedback
    pub review_comments: Option<String>,
    pub strengths: Option<String>,
    pub weaknesses: Option<String>,
    pub specific_concerns: Option<String>,
    
    // Review Decision
    pub recommendation: ReviewRecommendation,
    pub required_changes: Option<String>,
    pub clarification_questions: Option<String>,
    
    // Quality Checklist
    pub imagery_reviewed: bool,
    pub damage_categories_correct: bool,
    pub functional_assessment_complete: bool,
    pub component_assessments_reviewed: bool,
    pub collateral_damage_assessed: bool,
    pub weaponeering_validated: bool,
    pub recommendations_justified: bool,
    pub classification_appropriate: bool,
    
    // Metadata
    pub time_spent_minutes: Option<i32>,
    pub review_version: i32,
    
    // Timestamps
    pub created_at: String,
    pub updated_at: String,
}

/// Request to create peer review assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePeerReviewRequest {
    pub bda_report_id: String,
    pub reviewer_id: String,
    pub reviewer_name: Option<String>,
    pub reviewer_role: Option<String>,
    pub due_date: Option<String>,
    pub priority: Option<ReviewPriority>,
}

/// Request to update peer review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePeerReviewRequest {
    pub review_status: Option<ReviewStatus>,
    pub overall_quality: Option<OverallQuality>,
    pub confidence_adequate: Option<bool>,
    pub evidence_sufficient: Option<bool>,
    pub methodology_sound: Option<bool>,
    pub recommendations_appropriate: Option<bool>,
    pub review_comments: Option<String>,
    pub strengths: Option<String>,
    pub weaknesses: Option<String>,
    pub specific_concerns: Option<String>,
    pub recommendation: Option<ReviewRecommendation>,
    pub required_changes: Option<String>,
    pub clarification_questions: Option<String>,
    pub imagery_reviewed: Option<bool>,
    pub damage_categories_correct: Option<bool>,
    pub functional_assessment_complete: Option<bool>,
    pub component_assessments_reviewed: Option<bool>,
    pub collateral_damage_assessed: Option<bool>,
    pub weaponeering_validated: Option<bool>,
    pub recommendations_justified: Option<bool>,
    pub classification_appropriate: Option<bool>,
    pub time_spent_minutes: Option<i32>,
}

/// Review summary for a report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewSummary {
    pub bda_report_id: String,
    pub total_reviews: i32,
    pub completed_reviews: i32,
    pub pending_reviews: i32,
    pub in_progress_reviews: i32,
    pub approve_count: i32,
    pub approve_with_changes_count: i32,
    pub reject_count: i32,
    pub clarification_count: i32,
    pub avg_quality_score: Option<f64>,
    pub earliest_due_date: Option<String>,
    pub last_completed_at: Option<String>,
}

impl CreatePeerReviewRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.bda_report_id.is_empty() {
            return Err("BDA report ID is required".to_string());
        }
        
        if self.reviewer_id.is_empty() {
            return Err("Reviewer ID is required".to_string());
        }
        
        Ok(())
    }
}

impl BdaPeerReview {
    /// Check if review is overdue
    pub fn is_overdue(&self) -> bool {
        if let Some(due_date) = &self.due_date {
            if let Ok(due) = chrono::DateTime::parse_from_rfc3339(due_date) {
                let now = chrono::Utc::now();
                return due < now && self.review_status != ReviewStatus::Completed;
            }
        }
        false
    }
    
    /// Check if review is complete
    pub fn is_complete(&self) -> bool {
        matches!(self.review_status, ReviewStatus::Completed)
    }
    
    /// Get completion percentage based on checklist
    pub fn get_completion_percentage(&self) -> f32 {
        let items = [
            self.imagery_reviewed,
            self.damage_categories_correct,
            self.functional_assessment_complete,
            self.component_assessments_reviewed,
            self.collateral_damage_assessed,
            self.weaponeering_validated,
            self.recommendations_justified,
            self.classification_appropriate,
        ];
        
        let completed = items.iter().filter(|&&x| x).count();
        (completed as f32 / items.len() as f32) * 100.0
    }
    
    /// Check if review meets minimum quality standards
    pub fn meets_quality_standards(&self) -> bool {
        self.imagery_reviewed
            && self.damage_categories_correct
            && self.functional_assessment_complete
            && self.collateral_damage_assessed
            && self.recommendations_justified
    }
}

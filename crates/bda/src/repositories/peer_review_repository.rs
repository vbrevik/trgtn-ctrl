// BDA Peer Review Repository
// Purpose: Database access layer for peer reviews

use crate::features::bda::domain::peer_review::{
    BdaPeerReview,
    CreatePeerReviewRequest,
    UpdatePeerReviewRequest,
    ReviewStatus,
    ReviewPriority,
    OverallQuality,
    ReviewRecommendation,
    ReviewSummary,
};
use sqlx::{Pool, Sqlite, Row};
use uuid::Uuid;

pub struct PeerReviewRepository {
    pool: Pool<Sqlite>,
}

impl PeerReviewRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
    
    /// Create peer review assignment
    pub async fn create(
        &self,
        assigned_by: &str,
        req: CreatePeerReviewRequest,
    ) -> Result<BdaPeerReview, sqlx::Error> {
        req.validate().map_err(|e| sqlx::Error::Decode(e.into()))?;
        
        let id = Uuid::new_v4().to_string();
        let assigned_at = chrono::Utc::now();
        
        let priority_str = match req.priority.unwrap_or(ReviewPriority::Normal) {
            ReviewPriority::Normal => "normal",
            ReviewPriority::Urgent => "urgent",
            ReviewPriority::Critical => "critical",
        };
        
        sqlx::query(
            r#"
            INSERT INTO bda_peer_review (
                id, bda_report_id, reviewer_id, reviewer_name, reviewer_role,
                assigned_by, assigned_at, due_date, priority, review_status
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'pending')
            "#
        )
        .bind(&id)
        .bind(&req.bda_report_id)
        .bind(&req.reviewer_id)
        .bind(&req.reviewer_name)
        .bind(&req.reviewer_role)
        .bind(assigned_by)
        .bind(&assigned_at)
        .bind(req.due_date.as_deref())
        .bind(priority_str)
        .execute(&self.pool)
        .await?;
        
        self.get_by_id(&id).await?
            .ok_or(sqlx::Error::RowNotFound)
    }
    
    /// Get peer review by ID
    pub async fn get_by_id(&self, id: &str) -> Result<Option<BdaPeerReview>, sqlx::Error> {
        let row = sqlx::query("SELECT * FROM bda_peer_review WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        
        match row {
            Some(r) => Ok(Some(self.row_to_review(r)?)),
            None => Ok(None),
        }
    }
    
    /// Get all reviews for a BDA report
    pub async fn get_by_report(&self, bda_report_id: &str) -> Result<Vec<BdaPeerReview>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT * FROM bda_peer_review WHERE bda_report_id = $1 ORDER BY assigned_at DESC"
        )
        .bind(bda_report_id)
        .fetch_all(&self.pool)
        .await?;
        
        let mut reviews = Vec::new();
        for row in rows {
            reviews.push(self.row_to_review(row)?);
        }
        
        Ok(reviews)
    }
    
    /// Get reviews assigned to a reviewer
    pub async fn get_by_reviewer(&self, reviewer_id: &str) -> Result<Vec<BdaPeerReview>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT * FROM bda_peer_review WHERE reviewer_id = $1 ORDER BY due_date ASC, priority DESC"
        )
        .bind(reviewer_id)
        .fetch_all(&self.pool)
        .await?;
        
        let mut reviews = Vec::new();
        for row in rows {
            reviews.push(self.row_to_review(row)?);
        }
        
        Ok(reviews)
    }
    
    /// Update peer review
    pub async fn update(
        &self,
        id: &str,
        req: UpdatePeerReviewRequest,
    ) -> Result<BdaPeerReview, sqlx::Error> {
        let updated_at = chrono::Utc::now();
        
        // Handle status transitions
        let started_at = if req.review_status == Some(ReviewStatus::InProgress) {
            Some(chrono::Utc::now())
        } else {
            None
        };
        
        let completed_at = if req.review_status == Some(ReviewStatus::Completed) {
            Some(chrono::Utc::now())
        } else {
            None
        };
        
        let review_status_str = req.review_status.as_ref().map(|s| match s {
            ReviewStatus::Pending => "pending",
            ReviewStatus::InProgress => "in_progress",
            ReviewStatus::Completed => "completed",
            ReviewStatus::Cancelled => "cancelled",
        });
        
        let overall_quality_str = req.overall_quality.as_ref().map(|q| match q {
            OverallQuality::High => "high",
            OverallQuality::Medium => "medium",
            OverallQuality::Low => "low",
            OverallQuality::NeedsRework => "needs_rework",
        });
        
        let recommendation_str = req.recommendation.as_ref().map(|r| match r {
            ReviewRecommendation::Approve => "approve",
            ReviewRecommendation::ApproveWithChanges => "approve_with_changes",
            ReviewRecommendation::Reject => "reject",
            ReviewRecommendation::RequestClarification => "request_clarification",
        });
        
        sqlx::query(
            r#"
            UPDATE bda_peer_review 
            SET review_status = COALESCE($1, review_status),
                started_at = COALESCE($2, started_at),
                completed_at = COALESCE($3, completed_at),
                overall_quality = COALESCE($4, overall_quality),
                confidence_adequate = COALESCE($5, confidence_adequate),
                evidence_sufficient = COALESCE($6, evidence_sufficient),
                methodology_sound = COALESCE($7, methodology_sound),
                recommendations_appropriate = COALESCE($8, recommendations_appropriate),
                review_comments = COALESCE($9, review_comments),
                strengths = COALESCE($10, strengths),
                weaknesses = COALESCE($11, weaknesses),
                specific_concerns = COALESCE($12, specific_concerns),
                recommendation = COALESCE($13, recommendation),
                required_changes = COALESCE($14, required_changes),
                clarification_questions = COALESCE($15, clarification_questions),
                imagery_reviewed = COALESCE($16, imagery_reviewed),
                damage_categories_correct = COALESCE($17, damage_categories_correct),
                functional_assessment_complete = COALESCE($18, functional_assessment_complete),
                component_assessments_reviewed = COALESCE($19, component_assessments_reviewed),
                collateral_damage_assessed = COALESCE($20, collateral_damage_assessed),
                weaponeering_validated = COALESCE($21, weaponeering_validated),
                recommendations_justified = COALESCE($22, recommendations_justified),
                classification_appropriate = COALESCE($23, classification_appropriate),
                time_spent_minutes = COALESCE($24, time_spent_minutes),
                updated_at = $25
            WHERE id = $26
            "#
        )
        .bind(review_status_str)
        .bind(started_at)
        .bind(completed_at)
        .bind(overall_quality_str)
        .bind(req.confidence_adequate)
        .bind(req.evidence_sufficient)
        .bind(req.methodology_sound)
        .bind(req.recommendations_appropriate)
        .bind(&req.review_comments)
        .bind(&req.strengths)
        .bind(&req.weaknesses)
        .bind(&req.specific_concerns)
        .bind(recommendation_str)
        .bind(&req.required_changes)
        .bind(&req.clarification_questions)
        .bind(req.imagery_reviewed)
        .bind(req.damage_categories_correct)
        .bind(req.functional_assessment_complete)
        .bind(req.component_assessments_reviewed)
        .bind(req.collateral_damage_assessed)
        .bind(req.weaponeering_validated)
        .bind(req.recommendations_justified)
        .bind(req.classification_appropriate)
        .bind(req.time_spent_minutes)
        .bind(&updated_at)
        .bind(id)
        .execute(&self.pool)
        .await?;
        
        self.get_by_id(id).await?
            .ok_or(sqlx::Error::RowNotFound)
    }
    
    /// Get review summary for a report
    pub async fn get_summary(&self, bda_report_id: &str) -> Result<ReviewSummary, sqlx::Error> {
        let row = sqlx::query(
            "SELECT * FROM v_bda_review_summary WHERE bda_report_id = $1"
        )
        .bind(bda_report_id)
        .fetch_optional(&self.pool)
        .await?;
        
        match row {
            Some(r) => Ok(ReviewSummary {
                bda_report_id: r.get("bda_report_id"),
                total_reviews: r.get("total_reviews"),
                completed_reviews: r.get("completed_reviews"),
                pending_reviews: r.get("pending_reviews"),
                in_progress_reviews: r.get("in_progress_reviews"),
                approve_count: r.get("approve_count"),
                approve_with_changes_count: r.get("approve_with_changes_count"),
                reject_count: r.get("reject_count"),
                clarification_count: r.get("clarification_count"),
                avg_quality_score: r.get("avg_quality_score"),
                earliest_due_date: r.get::<Option<chrono::DateTime<chrono::Utc>>, _>("earliest_due_date")
                    .map(|d| d.to_rfc3339()),
                last_completed_at: r.get::<Option<chrono::DateTime<chrono::Utc>>, _>("last_completed_at")
                    .map(|d| d.to_rfc3339()),
            }),
            None => Ok(ReviewSummary {
                bda_report_id: bda_report_id.to_string(),
                total_reviews: 0,
                completed_reviews: 0,
                pending_reviews: 0,
                in_progress_reviews: 0,
                approve_count: 0,
                approve_with_changes_count: 0,
                reject_count: 0,
                clarification_count: 0,
                avg_quality_score: None,
                earliest_due_date: None,
                last_completed_at: None,
            }),
        }
    }
    
    /// Delete peer review
    pub async fn delete(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM bda_peer_review WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
    
    /// Helper: Convert SQLite row to BdaPeerReview
    fn row_to_review(&self, row: sqlx::sqlite::SqliteRow) -> Result<BdaPeerReview, sqlx::Error> {
        Ok(BdaPeerReview {
            id: row.get("id"),
            bda_report_id: row.get("bda_report_id"),
            reviewer_id: row.get("reviewer_id"),
            reviewer_name: row.get("reviewer_name"),
            reviewer_role: row.get("reviewer_role"),
            assigned_by: row.get("assigned_by"),
            assigned_at: row.get::<chrono::DateTime<chrono::Utc>, _>("assigned_at").to_rfc3339(),
            due_date: row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("due_date")
                .map(|d| d.to_rfc3339()),
            priority: self.parse_priority(&row.get::<String, _>("priority")),
            review_status: self.parse_status(&row.get::<String, _>("review_status")),
            started_at: row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("started_at")
                .map(|d| d.to_rfc3339()),
            completed_at: row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("completed_at")
                .map(|d| d.to_rfc3339()),
            overall_quality: self.parse_quality(&row.get::<String, _>("overall_quality")),
            confidence_adequate: row.get("confidence_adequate"),
            evidence_sufficient: row.get("evidence_sufficient"),
            methodology_sound: row.get("methodology_sound"),
            recommendations_appropriate: row.get("recommendations_appropriate"),
            review_comments: row.get("review_comments"),
            strengths: row.get("strengths"),
            weaknesses: row.get("weaknesses"),
            specific_concerns: row.get("specific_concerns"),
            recommendation: self.parse_recommendation(&row.get::<String, _>("recommendation")),
            required_changes: row.get("required_changes"),
            clarification_questions: row.get("clarification_questions"),
            imagery_reviewed: row.get("imagery_reviewed"),
            damage_categories_correct: row.get("damage_categories_correct"),
            functional_assessment_complete: row.get("functional_assessment_complete"),
            component_assessments_reviewed: row.get("component_assessments_reviewed"),
            collateral_damage_assessed: row.get("collateral_damage_assessed"),
            weaponeering_validated: row.get("weaponeering_validated"),
            recommendations_justified: row.get("recommendations_justified"),
            classification_appropriate: row.get("classification_appropriate"),
            time_spent_minutes: row.get("time_spent_minutes"),
            review_version: row.get("review_version"),
            created_at: row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
            updated_at: row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at").to_rfc3339(),
        })
    }
    
    fn parse_priority(&self, s: &str) -> ReviewPriority {
        match s {
            "urgent" => ReviewPriority::Urgent,
            "critical" => ReviewPriority::Critical,
            _ => ReviewPriority::Normal,
        }
    }
    
    fn parse_status(&self, s: &str) -> ReviewStatus {
        match s {
            "in_progress" => ReviewStatus::InProgress,
            "completed" => ReviewStatus::Completed,
            "cancelled" => ReviewStatus::Cancelled,
            _ => ReviewStatus::Pending,
        }
    }
    
    fn parse_quality(&self, s: &str) -> OverallQuality {
        match s {
            "high" => OverallQuality::High,
            "medium" => OverallQuality::Medium,
            "low" => OverallQuality::Low,
            _ => OverallQuality::NeedsRework,
        }
    }
    
    fn parse_recommendation(&self, s: &str) -> ReviewRecommendation {
        match s {
            "approve_with_changes" => ReviewRecommendation::ApproveWithChanges,
            "reject" => ReviewRecommendation::Reject,
            "request_clarification" => ReviewRecommendation::RequestClarification,
            _ => ReviewRecommendation::Approve,
        }
    }
}

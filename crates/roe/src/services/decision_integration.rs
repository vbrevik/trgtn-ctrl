// ROE Decision Integration Utilities
// Purpose: Helper functions for integrating ROE determination with decision creation

use crate::features::roe::{
    domain::ROEStatus,
    repositories::ROERepository,
    services::{ROEDeterminationService, DecisionInfo},
};
use sqlx::Pool;
use sqlx::sqlite::Sqlite;

/// Auto-determine and set ROE status when a decision is created
/// This should be called after a decision is inserted into the database
pub async fn auto_determine_roe_on_decision_creation(
    pool: &Pool<Sqlite>,
    decision_id: &str,
) -> Result<(ROEStatus, Option<String>), sqlx::Error> {
    let repo = ROERepository::new(pool.clone());
    repo.auto_determine_roe_status(decision_id).await
}

/// Create DecisionInfo from raw decision data
/// Useful when creating decisions programmatically
pub fn create_decision_info(
    id: String,
    title: String,
    description: String,
    category: String,
) -> DecisionInfo {
    DecisionInfo {
        id,
        title,
        description,
        category,
    }
}

/// Determine ROE status without database update
/// Useful for preview/validation before saving
pub fn preview_roe_status(decision: &DecisionInfo) -> (ROEStatus, Option<String>) {
    let roe_status = ROEDeterminationService::determine_roe_status(decision);
    let roe_notes = ROEDeterminationService::generate_roe_notes(decision, roe_status);
    (roe_status, roe_notes)
}

// BDA Report History Handlers
// Purpose: HTTP request handlers for BDA report history

use crate::features::bda::repositories::ReportHistoryRepository;
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
    Extension,
};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

/// Get history for a BDA report
pub async fn get_report_history(
    Extension(repo): Extension<Arc<ReportHistoryRepository>>,
    Path(report_id): Path<String>,
    Query(query): Query<HistoryQuery>,
) -> impl IntoResponse {
    match repo.get_by_report_id(&report_id, query.limit, query.offset).await {
        Ok(history) => Json(history).into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch report history: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch report history").into_response()
        }
    }
}

/// Get specific version of a report
pub async fn get_report_version(
    Extension(repo): Extension<Arc<ReportHistoryRepository>>,
    Path((report_id, version)): Path<(String, i32)>,
) -> impl IntoResponse {
    match repo.get_by_version(&report_id, version).await {
        Ok(Some(history)) => Json(history).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Version not found").into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch report version: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch report version").into_response()
        }
    }
}

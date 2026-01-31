// BDA Reports Handlers
// Purpose: HTTP request handlers for BDA reports

use crate::features::bda::{
    domain::{CreateBdaReportRequest, UpdateBdaReportRequest},
    repositories::BdaRepository,
};
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
pub struct BdaQuery {
    pub status: Option<String>,
    pub target_id: Option<String>,
    pub analyst_id: Option<String>,
}

/// Create new BDA report
pub async fn create_report(
    Extension(repo): Extension<Arc<BdaRepository>>,
    Extension(user_id): Extension<String>,  // From auth middleware
    Json(payload): Json<CreateBdaReportRequest>,
) -> impl IntoResponse {
    match repo.create(&user_id, payload).await {
        Ok(report) => (StatusCode::CREATED, Json(report)).into_response(),
        Err(e) => {
            tracing::error!("Failed to create BDA report: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create BDA report").into_response()
        }
    }
}

/// Get all BDA reports with optional filtering
pub async fn get_reports(
    Extension(repo): Extension<Arc<BdaRepository>>,
    Query(query): Query<BdaQuery>,
) -> impl IntoResponse {
    let result = repo.get_all(
        query.status.as_deref(),
        query.target_id.as_deref(),
    ).await;
    
    match result {
        Ok(reports) => Json(reports).into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch BDA reports: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch BDA reports").into_response()
        }
    }
}

/// Get single BDA report by ID
pub async fn get_report(
    Extension(repo): Extension<Arc<BdaRepository>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match repo.get_by_id(&id).await {
        Ok(Some(report)) => Json(report).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "BDA report not found").into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch BDA report: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch BDA report").into_response()
        }
    }
}

/// Update BDA report
pub async fn update_report(
    Extension(repo): Extension<Arc<BdaRepository>>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateBdaReportRequest>,
) -> impl IntoResponse {
    match repo.update(&id, payload).await {
        Ok(report) => Json(report).into_response(),
        Err(sqlx::Error::RowNotFound) => (StatusCode::NOT_FOUND, "BDA report not found").into_response(),
        Err(e) => {
            tracing::error!("Failed to update BDA report: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update BDA report").into_response()
        }
    }
}

/// Delete BDA report
pub async fn delete_report(
    Extension(repo): Extension<Arc<BdaRepository>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match repo.delete(&id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("Failed to delete BDA report: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete BDA report").into_response()
        }
    }
}

/// Get assessment queue (pending BDA reports)
pub async fn get_queue(
    Extension(repo): Extension<Arc<BdaRepository>>,
) -> impl IntoResponse {
    match repo.get_assessment_queue().await {
        Ok(reports) => Json(reports).into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch assessment queue: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch assessment queue").into_response()
        }
    }
}

/// Get BDA statistics
pub async fn get_statistics(
    Extension(repo): Extension<Arc<BdaRepository>>,
) -> impl IntoResponse {
    match repo.get_statistics().await {
        Ok(stats) => Json(stats).into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch BDA statistics: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch BDA statistics").into_response()
        }
    }
}

/// Submit BDA report for review
pub async fn submit_report(
    Extension(repo): Extension<Arc<BdaRepository>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match repo.submit(&id).await {
        Ok(report) => Json(report).into_response(),
        Err(sqlx::Error::RowNotFound) => (StatusCode::NOT_FOUND, "BDA report not found").into_response(),
        Err(e) => {
            tracing::error!("Failed to submit BDA report: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to submit BDA report").into_response()
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ApprovalRequest {
    pub comments: Option<String>,
}

/// Approve BDA report
pub async fn approve_report(
    Extension(repo): Extension<Arc<BdaRepository>>,
    Extension(user_id): Extension<String>,  // From auth middleware
    Path(id): Path<String>,
    Json(payload): Json<ApprovalRequest>,
) -> impl IntoResponse {
    match repo.approve(&id, &user_id, payload.comments).await {
        Ok(report) => Json(report).into_response(),
        Err(sqlx::Error::RowNotFound) => (StatusCode::NOT_FOUND, "BDA report not found").into_response(),
        Err(e) => {
            tracing::error!("Failed to approve BDA report: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to approve BDA report").into_response()
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RejectionRequest {
    pub comments: String,
}

/// Reject BDA report
pub async fn reject_report(
    Extension(repo): Extension<Arc<BdaRepository>>,
    Extension(user_id): Extension<String>,  // From auth middleware
    Path(id): Path<String>,
    Json(payload): Json<RejectionRequest>,
) -> impl IntoResponse {
    match repo.reject(&id, &user_id, payload.comments).await {
        Ok(report) => Json(report).into_response(),
        Err(sqlx::Error::RowNotFound) => (StatusCode::NOT_FOUND, "BDA report not found").into_response(),
        Err(e) => {
            tracing::error!("Failed to reject BDA report: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to reject BDA report").into_response()
        }
    }
}

/// Get re-attack recommendations
pub async fn get_reattack_recommendations(
    Extension(repo): Extension<Arc<BdaRepository>>,
) -> impl IntoResponse {
    match repo.get_re_attack_recommendations().await {
        Ok(recommendations) => Json(recommendations).into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch re-attack recommendations: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch re-attack recommendations").into_response()
        }
    }
}

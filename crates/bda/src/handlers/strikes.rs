// BDA Strike Correlation Handlers
// Purpose: HTTP request handlers for strike correlation data

use crate::features::bda::{
    domain::CreateStrikeCorrelationRequest,
    repositories::StrikeRepository,
};
use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    Json,
    Extension,
};
use std::sync::Arc;

/// Create strike correlation
pub async fn create_strike(
    Extension(repo): Extension<Arc<StrikeRepository>>,
    Json(payload): Json<CreateStrikeCorrelationRequest>,
) -> impl IntoResponse {
    match repo.create(payload).await {
        Ok(strike) => (StatusCode::CREATED, Json(strike)).into_response(),
        Err(e) => {
            tracing::error!("Failed to create strike correlation: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create strike correlation").into_response()
        }
    }
}

/// Get strike correlation by ID
pub async fn get_strike(
    Extension(repo): Extension<Arc<StrikeRepository>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match repo.get_by_id(&id).await {
        Ok(Some(strike)) => Json(strike).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Strike correlation not found").into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch strike correlation: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch strike correlation").into_response()
        }
    }
}

/// Get all strikes for a BDA report
pub async fn get_report_strikes(
    Extension(repo): Extension<Arc<StrikeRepository>>,
    Path(report_id): Path<String>,
) -> impl IntoResponse {
    match repo.get_by_report(&report_id).await {
        Ok(strikes) => Json(strikes).into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch report strikes: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch report strikes").into_response()
        }
    }
}

/// Get all strikes for a target
pub async fn get_target_strikes(
    Extension(repo): Extension<Arc<StrikeRepository>>,
    Path(target_id): Path<String>,
) -> impl IntoResponse {
    match repo.get_by_target(&target_id).await {
        Ok(strikes) => Json(strikes).into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch target strikes: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch target strikes").into_response()
        }
    }
}

/// Get weapon performance summary
pub async fn get_weapon_performance(
    Extension(repo): Extension<Arc<StrikeRepository>>,
) -> impl IntoResponse {
    match repo.get_weapon_performance_summary().await {
        Ok(summary) => Json(summary).into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch weapon performance: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch weapon performance").into_response()
        }
    }
}

/// Delete strike correlation
pub async fn delete_strike(
    Extension(repo): Extension<Arc<StrikeRepository>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match repo.delete(&id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("Failed to delete strike correlation: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete strike correlation").into_response()
        }
    }
}

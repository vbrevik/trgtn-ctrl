// BDA Component Assessment Handlers
// Purpose: HTTP request handlers for component-level assessments

use crate::features::bda::{
    domain::{CreateComponentAssessmentRequest, UpdateComponentAssessmentRequest},
    repositories::ComponentAssessmentRepository,
};
use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    Json,
    Extension,
};
use std::sync::Arc;

/// Create component assessment
pub async fn create_component_assessment(
    Extension(repo): Extension<Arc<ComponentAssessmentRepository>>,
    Extension(user_id): Extension<String>,  // From auth middleware
    Json(payload): Json<CreateComponentAssessmentRequest>,
) -> impl IntoResponse {
    match repo.create(&user_id, payload).await {
        Ok(component) => (StatusCode::CREATED, Json(component)).into_response(),
        Err(e) => {
            tracing::error!("Failed to create component assessment: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create component assessment").into_response()
        }
    }
}

/// Get component assessment by ID
pub async fn get_component_assessment(
    Extension(repo): Extension<Arc<ComponentAssessmentRepository>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match repo.get_by_id(&id).await {
        Ok(Some(component)) => Json(component).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Component assessment not found").into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch component assessment: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch component assessment").into_response()
        }
    }
}

/// Get all component assessments for a report
pub async fn get_report_components(
    Extension(repo): Extension<Arc<ComponentAssessmentRepository>>,
    Path(report_id): Path<String>,
) -> impl IntoResponse {
    match repo.get_by_report(&report_id).await {
        Ok(components) => Json(components).into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch report components: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch report components").into_response()
        }
    }
}

/// Update component assessment
pub async fn update_component_assessment(
    Extension(repo): Extension<Arc<ComponentAssessmentRepository>>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateComponentAssessmentRequest>,
) -> impl IntoResponse {
    match repo.update(&id, payload).await {
        Ok(component) => Json(component).into_response(),
        Err(e) => {
            tracing::error!("Failed to update component assessment: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update component assessment").into_response()
        }
    }
}

/// Delete component assessment
pub async fn delete_component_assessment(
    Extension(repo): Extension<Arc<ComponentAssessmentRepository>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match repo.delete(&id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("Failed to delete component assessment: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete component assessment").into_response()
        }
    }
}

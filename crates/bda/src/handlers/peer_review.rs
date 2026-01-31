// BDA Peer Review Handlers
// Purpose: HTTP request handlers for peer review workflow

use crate::features::bda::{
    domain::{CreatePeerReviewRequest, UpdatePeerReviewRequest},
    repositories::PeerReviewRepository,
};
use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    Json,
    Extension,
};
use std::sync::Arc;

/// Create peer review assignment
pub async fn create_peer_review(
    Extension(repo): Extension<Arc<PeerReviewRepository>>,
    Extension(user_id): Extension<String>,  // From auth middleware
    Json(payload): Json<CreatePeerReviewRequest>,
) -> impl IntoResponse {
    match repo.create(&user_id, payload).await {
        Ok(review) => (StatusCode::CREATED, Json(review)).into_response(),
        Err(e) => {
            tracing::error!("Failed to create peer review: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create peer review").into_response()
        }
    }
}

/// Get peer review by ID
pub async fn get_peer_review(
    Extension(repo): Extension<Arc<PeerReviewRepository>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match repo.get_by_id(&id).await {
        Ok(Some(review)) => Json(review).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Peer review not found").into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch peer review: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch peer review").into_response()
        }
    }
}

/// Get all reviews for a report
pub async fn get_report_reviews(
    Extension(repo): Extension<Arc<PeerReviewRepository>>,
    Path(report_id): Path<String>,
) -> impl IntoResponse {
    match repo.get_by_report(&report_id).await {
        Ok(reviews) => Json(reviews).into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch report reviews: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch report reviews").into_response()
        }
    }
}

/// Get reviews assigned to a reviewer
pub async fn get_reviewer_reviews(
    Extension(repo): Extension<Arc<PeerReviewRepository>>,
    Path(reviewer_id): Path<String>,
) -> impl IntoResponse {
    match repo.get_by_reviewer(&reviewer_id).await {
        Ok(reviews) => Json(reviews).into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch reviewer reviews: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch reviewer reviews").into_response()
        }
    }
}

/// Get review summary for a report
pub async fn get_review_summary(
    Extension(repo): Extension<Arc<PeerReviewRepository>>,
    Path(report_id): Path<String>,
) -> impl IntoResponse {
    match repo.get_summary(&report_id).await {
        Ok(summary) => Json(summary).into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch review summary: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch review summary").into_response()
        }
    }
}

/// Update peer review
pub async fn update_peer_review(
    Extension(repo): Extension<Arc<PeerReviewRepository>>,
    Path(id): Path<String>,
    Json(payload): Json<UpdatePeerReviewRequest>,
) -> impl IntoResponse {
    match repo.update(&id, payload).await {
        Ok(review) => Json(review).into_response(),
        Err(e) => {
            tracing::error!("Failed to update peer review: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update peer review").into_response()
        }
    }
}

/// Delete peer review
pub async fn delete_peer_review(
    Extension(repo): Extension<Arc<PeerReviewRepository>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match repo.delete(&id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("Failed to delete peer review: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete peer review").into_response()
        }
    }
}

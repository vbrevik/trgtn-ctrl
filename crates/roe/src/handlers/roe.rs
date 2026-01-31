// ROE Handlers
// Purpose: HTTP request handlers for ROE requests and decision ROE status

use crate::features::roe::{
    domain::{
        CreateROERequestRequest, UpdateROERequestStatusRequest, UpdateDecisionROEStatusRequest,
        DecisionROEStatusResponse,
    },
    repositories::ROERepository,
};
use axum::{
    extract::{Path, Query, State, Extension},
    http::StatusCode,
    response::IntoResponse,

    Json,
};
use crate::features::auth::jwt::Claims;
use serde::Deserialize;
use sqlx::{Pool, Row};
use sqlx::sqlite::Sqlite;

#[derive(Debug, Deserialize)]
pub struct ROEQueryParams {
    pub status: Option<String>,
}

/// Create a new ROE request for a decision
pub async fn create_roe_request(
    State(pool): State<Pool<Sqlite>>,
    Extension(claims): Extension<Claims>,
    Path(decision_id): Path<String>,
    Json(mut payload): Json<CreateROERequestRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // Set decision_id from path parameter
    payload.decision_id = decision_id;

    let repo = ROERepository::new(pool);
    let user_id = &claims.sub;

    match repo.create_roe_request(user_id, payload).await {
        Ok(request) => Ok((StatusCode::CREATED, Json(request))),
        Err(e) => {
            tracing::error!("Failed to create ROE request: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get ROE request by ID
pub async fn get_roe_request(
    State(pool): State<Pool<Sqlite>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let repo = ROERepository::new(pool);

    match repo.get_roe_request_by_id(&id).await {
        Ok(request) => Ok(Json(request)),
        Err(sqlx::Error::RowNotFound) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to fetch ROE request: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get ROE request by decision ID
pub async fn get_roe_request_by_decision(
    State(pool): State<Pool<Sqlite>>,
    Path(decision_id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let repo = ROERepository::new(pool);

    match repo.get_roe_request_by_decision_id(&decision_id).await {
        Ok(Some(request)) => Ok(Json(request)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to fetch ROE request: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Update ROE request status (approve/reject/withdraw)
pub async fn update_roe_request_status(
    State(pool): State<Pool<Sqlite>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateROERequestStatusRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let repo = ROERepository::new(pool);
    let approved_by = &claims.sub;

    match repo.update_roe_request_status(&id, payload, approved_by).await {
        Ok(request) => Ok(Json(request)),
        Err(sqlx::Error::RowNotFound) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to update ROE request: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// List ROE requests with optional status filter
pub async fn list_roe_requests(
    State(pool): State<Pool<Sqlite>>,
    Query(params): Query<ROEQueryParams>,
) -> Result<impl IntoResponse, StatusCode> {
    let repo = ROERepository::new(pool);

    match repo.list_roe_requests_by_status(params.status.as_deref()).await {
        Ok(requests) => Ok(Json(requests)),
        Err(e) => {
            tracing::error!("Failed to list ROE requests: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get decision ROE status
pub async fn get_decision_roe_status(
    State(pool): State<Pool<Sqlite>>,
    Path(decision_id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let repo = ROERepository::new(pool);

    match repo.get_decision_roe_status(&decision_id).await {
        Ok((roe_status, roe_notes, roe_request_id)) => {
            let roe_request = if let Some(request_id) = roe_request_id {
                repo.get_roe_request_by_id(&request_id).await.ok()
            } else {
                None
            };

            let response = DecisionROEStatusResponse {
                decision_id,
                roe_status,
                roe_notes,
                roe_request,
            };

            Ok(Json(response))
        }
        Err(e) => {
            tracing::error!("Failed to fetch decision ROE status: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Update decision ROE status
pub async fn update_decision_roe_status(
    State(pool): State<Pool<Sqlite>>,
    Path(decision_id): Path<String>,
    Json(payload): Json<UpdateDecisionROEStatusRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let repo = ROERepository::new(pool);

    match repo
        .update_decision_roe_status(
            &decision_id,
            payload.roe_status.as_deref(),
            payload.roe_notes.as_deref(),
            payload.roe_request_id.as_deref(),
        )
        .await
    {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            tracing::error!("Failed to update decision ROE status: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Auto-determine ROE status for a decision
pub async fn auto_determine_roe_status(
    State(pool): State<Pool<Sqlite>>,
    Path(decision_id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    // use crate::features::roe::domain::ROEStatus;

    
    let repo = ROERepository::new(pool);

    match repo.auto_determine_roe_status(&decision_id).await {
        Ok((roe_status, roe_notes)) => {
            Ok(Json(serde_json::json!({
                "decision_id": decision_id,
                "roe_status": roe_status.to_string(),
                "roe_notes": roe_notes,
                "determined_at": chrono::Utc::now().to_rfc3339()
            })))
        }
        Err(sqlx::Error::RowNotFound) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to auto-determine ROE status: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Check if a decision can proceed based on ROE status
pub async fn check_roe_blocking(
    State(pool): State<Pool<Sqlite>>,
    Path(decision_id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    use crate::features::roe::services::ROEBlockingCheckService;
    
    match ROEBlockingCheckService::check_decision_from_db(&pool, &decision_id).await {
        Ok(result) => Ok(Json(result)),
        Err(sqlx::Error::RowNotFound) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to check ROE blocking: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Route a decision to appropriate meeting, checking ROE first
pub async fn route_decision(
    State(pool): State<Pool<Sqlite>>,
    Path(decision_id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    use crate::features::roe::services::DecisionRoutingService;
    
    // Fetch decision urgency and deadline from database
    let row = sqlx::query(
        "SELECT urgency, deadline FROM decisions WHERE id = $1"
    )
    .bind(&decision_id)
    .fetch_optional(&pool)
    .await;
    
    let (urgency, deadline) = match row {
        Ok(Some(row)) => {
            let db_urgency: Option<String> = row.try_get("urgency").ok();
            let db_deadline: Option<String> = row.try_get("deadline").ok();
            (db_urgency, db_deadline)
        }
        Ok(None) => {
            return Err(StatusCode::NOT_FOUND);
        }
        Err(e) => {
            tracing::error!("Failed to fetch decision: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    let routing_service = DecisionRoutingService::new(pool);
    
    match routing_service.route_decision(&decision_id, urgency.as_deref(), deadline.as_deref()).await {
        Ok(plan) => Ok(Json(plan)),
        Err(sqlx::Error::RowNotFound) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to route decision: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

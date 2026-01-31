use axum::{
    extract::{Path, State, Extension, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Deserialize;
use sqlx::{Pool, Sqlite};

use crate::features::targeting::domain::*;
use crate::features::targeting::repositories::*;
use crate::features::auth::jwt::Claims;
use super::common::PlatformQueryParams;

pub async fn list_jtb_sessions(
    State(pool): State<Pool<Sqlite>>,
    Query(params): Query<PlatformQueryParams>,
) -> Result<impl IntoResponse, StatusCode> {
    let sessions = JtbRepository::list_sessions(&pool, params.status.as_deref(), None)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(sessions))
}

pub async fn create_jtb_session(
    State(pool): State<Pool<Sqlite>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateJtbSessionRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let user_id = &claims.sub;
    
    let required_attendees_json = req.required_attendees
        .as_ref()
        .map(|v| serde_json::to_string(v).unwrap_or_default());
    
    let caveats_json = req.caveats
        .as_ref()
        .map(|v| serde_json::to_string(v).unwrap_or_default());
    
    let id = JtbRepository::create_session(
        &pool,
        &req.session_name,
        &req.session_date,
        &req.session_time,
        &req.session_datetime,
        &req.chair,
        req.chair_rank.as_deref(),
        required_attendees_json.as_deref(), // Pass extracted string
        &req.classification,
        caveats_json.as_deref(), // Pass extracted string
        user_id,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok((StatusCode::CREATED, Json(serde_json::json!({"id": id}))))
}

pub async fn get_jtb_session(
    State(pool): State<Pool<Sqlite>>,
    Path(session_id): Path<String>, // Rename arg
) -> Result<impl IntoResponse, StatusCode> {
    let session = JtbRepository::get_session_by_id(&pool, &session_id) // Correct method name
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let targets = JtbRepository::get_targets_for_session(&pool, &session_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(serde_json::json!({
        "session": session,
        "targets": targets
    })))
}

pub async fn update_jtb_session_status(
    State(pool): State<Pool<Sqlite>>,
    Path(session_id): Path<String>, // Rename arg
    Json(req): Json<serde_json::Value>,
) -> Result<impl IntoResponse, StatusCode> {
    let status = req.get("status")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    JtbRepository::update_session_status(&pool, &session_id, status)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::OK)
}

pub async fn add_target_to_jtb_session(
    State(pool): State<Pool<Sqlite>>,
    Path(session_id): Path<String>,
    Json(req): Json<AddTargetToSessionRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // presentation_order logic moved to optional handling
    let existing_targets = JtbRepository::get_targets_for_session(&pool, &session_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
    let presentation_order = req.presentation_order.unwrap_or_else(|| {
        (existing_targets.len() + 1) as i32
    });

    JtbRepository::add_target_to_session(&pool, &session_id, &req.target_id, presentation_order) // Correct method
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::OK)
}

pub async fn record_jtb_decision(
    State(pool): State<Pool<Sqlite>>,
    Path(jtb_target_id): Path<String>, // Renamed for clarity
    Json(req): Json<RecordJtbDecisionRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    JtbRepository::record_decision(
        &pool,
        &jtb_target_id,
        &req.decision,
        &req.decision_rationale,
        &req.decided_by,
        req.votes_for,
        req.votes_against,
        req.votes_abstain,
        req.approval_conditions.as_deref(),
        req.mitigation_requirements.as_deref(),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::OK)
}

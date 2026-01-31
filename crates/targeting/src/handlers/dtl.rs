use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Deserialize;
use sqlx::{Pool, Sqlite};
use crate::features::targeting::domain::*;
use crate::features::targeting::repositories::*;
use super::common::TargetQueryParams;

pub async fn list_dtl(
    State(pool): State<Pool<Sqlite>>,
    Query(params): Query<TargetQueryParams>,
) -> Result<impl IntoResponse, StatusCode> {
    let entries = DtlRepository::list_all(&pool, params.limit)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(entries))
}

pub async fn create_dtl_entry(
    State(pool): State<Pool<Sqlite>>,
    Json(req): Json<CreateDtlEntryRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let id = DtlRepository::create(&pool, &req.target_id, req.priority_score, req.feasibility_score)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({"id": id}))))
}

#[derive(Debug, Deserialize)]
pub struct UpdatePriorityRequest {
    pub priority_score: f64,
    pub feasibility_score: f64,
}

pub async fn update_dtl_priority(
    State(pool): State<Pool<Sqlite>>,
    Path(id): Path<String>,
    Json(req): Json<UpdatePriorityRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    use crate::features::targeting::services::DtlScoring;
    
    // Validate scores are in range [0.0, 1.0]
    if req.priority_score < 0.0 || req.priority_score > 1.0 {
        return Err(StatusCode::BAD_REQUEST);
    }
    if req.feasibility_score < 0.0 || req.feasibility_score > 1.0 {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Calculate new combined score using business logic
    let combined_score = DtlScoring::calculate_combined_score(
        req.priority_score,
        req.feasibility_score
    );
    
    // Update DTL entry with new scores
    sqlx::query(
        "UPDATE dtl_entries 
         SET priority_score = ?, feasibility_score = ?, combined_score = ?, updated_at = datetime('now')
         WHERE id = ?"
    )
    .bind(req.priority_score)
    .bind(req.feasibility_score)
    .bind(combined_score)
    .bind(&id)
    .execute(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(StatusCode::OK)
}

pub async fn get_active_tsts(
    State(pool): State<Pool<Sqlite>>,
) -> Result<impl IntoResponse, StatusCode> {
    let tsts = DtlRepository::get_active_tsts(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(tsts))
}

use axum::{
    extract::{State, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use sqlx::{Pool, Sqlite};
use crate::features::targeting::domain::*;
use crate::features::targeting::repositories::*;
use super::common::PlatformQueryParams;

pub async fn list_assumptions(
    State(pool): State<Pool<Sqlite>>,
    Query(params): Query<PlatformQueryParams>,
) -> Result<impl IntoResponse, StatusCode> {
    let assumptions = AssumptionChallengeRepository::list_all(&pool, params.status.as_deref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(assumptions))
}

pub async fn create_assumption_challenge(
    State(pool): State<Pool<Sqlite>>,
    Json(req): Json<CreateAssumptionChallengeRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let id = AssumptionChallengeRepository::create(&pool, &req.assumption_text, req.confidence_level, &req.validation_status)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({"id": id}))))
}

pub async fn get_bias_alerts(
    State(_pool): State<Pool<Sqlite>>,
) -> Result<impl IntoResponse, StatusCode> {
    Ok(Json(Vec::<String>::new()))
}

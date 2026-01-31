use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use sqlx::{Pool, Sqlite};
use crate::features::auth::jwt::Claims;
use crate::features::targeting::domain::*;
use crate::features::targeting::repositories::*;

pub async fn get_risk_assessment(
    State(pool): State<Pool<Sqlite>>,
    Path(target_id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let assessment = RiskRepository::get_by_target_id(&pool, &target_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(assessment))
}

pub async fn create_risk_assessment(
    State(pool): State<Pool<Sqlite>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateRiskAssessmentRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let user_id = &claims.sub;
    let id = RiskRepository::create(
        &pool,
        &req.target_id,
        &req.fratricide_risk,
        &req.political_sensitivity,
        &req.legal_review_status,
        &req.classification,
        user_id
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({"id": id}))))
}

pub async fn get_high_risk_targets(
    State(pool): State<Pool<Sqlite>>,
) -> Result<impl IntoResponse, StatusCode> {
    let high_risk = RiskRepository::get_high_risk(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(high_risk))
}

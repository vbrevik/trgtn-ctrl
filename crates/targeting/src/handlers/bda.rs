use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use sqlx::{Pool, Sqlite};
use crate::features::targeting::domain::*;
use crate::features::targeting::repositories::*;

pub async fn list_bda(
    State(_pool): State<Pool<Sqlite>>,
) -> Result<impl IntoResponse, StatusCode> {
    Ok(Json(Vec::<BdaAssessment>::new()))
}

pub async fn create_bda(
    State(_pool): State<Pool<Sqlite>>,
    Json(_req): Json<CreateBdaRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    Ok((StatusCode::CREATED, Json(serde_json::json!({"message": "Use existing BDA system"}))))
}

pub async fn get_bda(
    State(pool): State<Pool<Sqlite>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let bda = BdaRepository::get_by_id(&pool, &id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    match bda {
        Some(b) => Ok(Json(b)),
        None => Err(StatusCode::NOT_FOUND)
    }
}

pub async fn get_reattack_recommendations(
    State(_pool): State<Pool<Sqlite>>,
) -> Result<impl IntoResponse, StatusCode> {
    Ok(Json(Vec::<BdaAssessment>::new()))
}

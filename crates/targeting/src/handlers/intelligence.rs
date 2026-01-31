use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use sqlx::{Pool, Sqlite};
use crate::features::auth::jwt::Claims;
use crate::features::targeting::domain::*;
use crate::features::targeting::repositories::*;

pub async fn list_intel_reports(
    State(_pool): State<Pool<Sqlite>>,
) -> Result<impl IntoResponse, StatusCode> {
    Ok(Json(Vec::<IntelligenceReport>::new()))
}

pub async fn create_intel_report(
    State(pool): State<Pool<Sqlite>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateIntelReportRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let user_id = &claims.sub;
    let report = IntelligenceReport {
        id: String::new(),
        target_id: req.target_id,
        int_type: req.int_type,
        report_title: req.report_title,
        report_content: req.report_content,
        report_summary: None,
        confidence_level: req.confidence_level,
        source_reliability: req.source_reliability,
        collection_time: req.collection_time,
        reporting_time: String::new(),
        fusion_score: None,
        pattern_of_life_indicator: false,
        classification: req.classification,
        collected_by: Some(user_id.to_string()),
        created_at: String::new(),
        updated_at: String::new(),
    };
    
    let id = IntelRepository::create(&pool, &report, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({"id": id}))))
}

pub async fn get_intel_fusion(
    State(pool): State<Pool<Sqlite>>,
    Path(target_id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let reports = IntelRepository::get_by_target_id(&pool, &target_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(reports))
}

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use sqlx::{Pool, Sqlite};
use crate::features::targeting::domain::*;
use crate::features::targeting::repositories::*;
use super::common::PlatformQueryParams;

pub async fn list_isr_platforms(
    State(pool): State<Pool<Sqlite>>,
    Query(params): Query<PlatformQueryParams>,
) -> Result<impl IntoResponse, StatusCode> {
    let platforms = IsrRepository::list_all(&pool, params.status.as_deref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(platforms))
}

pub async fn create_isr_platform(
    State(pool): State<Pool<Sqlite>>,
    Json(req): Json<CreateIsrPlatformRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let platform = IsrPlatform {
        id: String::new(),
        platform_type: req.platform_type,
        platform_name: req.platform_name,
        callsign: req.callsign,
        current_position: None,
        sensor_type: req.sensor_type,
        sensor_range_km: None,
        coverage_area: None,
        status: "STANDBY".to_string(),
        loiter_time_remaining: None,
        fuel_remaining_percent: None,
        current_task: None,
        tasking_priority: None,
        tasked_targets: None,
        classification: req.classification,
        created_at: String::new(),
        updated_at: String::new(),
    };
    
    let id = IsrRepository::create(&pool, &platform)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({"id": id}))))
}

pub async fn get_isr_coverage(
    State(pool): State<Pool<Sqlite>>,
) -> Result<impl IntoResponse, StatusCode> {
    // Get active ISR platforms and their coverage areas
    // For now, return basic coverage summary (full spatial analysis requires geographic library)
    
    let platforms = IsrRepository::list_all(&pool, Some("ACTIVE"))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let active_count = platforms.iter().filter(|p| p.status == "ACTIVE").count();
    let total_count = platforms.len();
    
    let mut coverage_summary = Vec::new();
    
    for platform in &platforms {
        // Basic coverage info (full spatial analysis would require PostGIS or similar)
        let coverage = serde_json::json!({
            "platform_id": platform.id,
            "platform_name": platform.platform_name,
            "platform_type": platform.platform_type,
            "sensor_type": platform.sensor_type,
            "sensor_range_km": platform.sensor_range_km,
            "status": platform.status,
            "current_position": platform.current_position,
            "coverage_area": platform.coverage_area, // JSON polygon if available
            "loiter_time_remaining": platform.loiter_time_remaining,
            "fuel_remaining_percent": platform.fuel_remaining_percent,
            "current_task": platform.current_task,
        });
        coverage_summary.push(coverage);
    }
    
    Ok(Json(serde_json::json!({
        "total_platforms": total_count,
        "active_platforms": active_count,
        "platforms": coverage_summary,
        "note": "Full spatial coverage analysis requires geographic calculations. Use coverage_area JSON for polygon data."
    })))
}

pub async fn get_pattern_of_life(
    State(pool): State<Pool<Sqlite>>,
) -> Result<impl IntoResponse, StatusCode> {
    let reports = IsrRepository::get_pattern_of_life(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(reports))
}

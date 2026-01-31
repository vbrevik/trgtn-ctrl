use axum::{
    extract::{Path, Query, State, Extension},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Deserialize;
use sqlx::{Pool, Sqlite};
use crate::features::auth::jwt::Claims;
use crate::features::targeting::domain::*;
use crate::features::targeting::repositories::TargetRepository;
use super::common::TargetQueryParams;

pub async fn list_targets(
    State(pool): State<Pool<Sqlite>>,
    Query(params): Query<TargetQueryParams>,
) -> Result<impl IntoResponse, StatusCode> {
    let targets = TargetRepository::list_all(&pool, params.status.as_deref(), params.priority.as_deref(), params.limit, params.offset)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(targets))
}

pub async fn create_target(
    State(pool): State<Pool<Sqlite>>,
    Extension(_claims): Extension<Claims>,
    Json(req): Json<CreateTargetRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // Validate required fields
    if req.name.is_empty() || req.target_type.is_empty() || req.priority.is_empty() || req.coordinates.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Validate target_type enum
    let valid_types = ["HPT", "TST", "HVT", "TGT"];
    if !valid_types.contains(&req.target_type.as_str()) {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Validate priority enum
    let valid_priorities = ["LOW", "MEDIUM", "HIGH", "CRITICAL"];
    if !valid_priorities.contains(&req.priority.as_str()) {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Default classification if not provided
    let classification = if req.classification.is_empty() {
        "SECRET"
    } else {
        &req.classification
    };
    
    // Create target
    let id = TargetRepository::create(
        &pool,
        &req.name,
        req.description.as_deref(),
        &req.target_type,
        &req.priority,
        &req.coordinates,
        classification,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to create target: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok((StatusCode::CREATED, Json(serde_json::json!({
        "id": id,
        "message": "Target created successfully"
    }))))
}

pub async fn get_target(
    State(pool): State<Pool<Sqlite>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let target = TargetRepository::get_by_id(&pool, &id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    match target {
        Some(t) => Ok(Json(t)),
        None => Err(StatusCode::NOT_FOUND)
    }
}

pub async fn update_target(
    State(pool): State<Pool<Sqlite>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateTargetRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // Validate target exists
    let _target = TargetRepository::get_by_id(&pool, &id)

        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    // Validate priority if provided
    if let Some(ref p) = req.priority {
        let valid_priorities = ["LOW", "MEDIUM", "HIGH", "CRITICAL"];
        if !valid_priorities.contains(&p.as_str()) {
            return Err(StatusCode::BAD_REQUEST);
        }
    }
    
    // Validate target_status if provided
    if let Some(ref s) = req.target_status {
        let valid_statuses = ["Nominated", "Validated", "Approved", "Engaged", "Assessed"];
        if !valid_statuses.contains(&s.as_str()) {
            return Err(StatusCode::BAD_REQUEST);
        }
    }
    
    // Update target
    TargetRepository::update(
        &pool,
        &id,
        req.name.as_deref(),
        req.priority.as_deref(),
        req.target_status.as_deref(),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Return updated target
    let updated = TargetRepository::get_by_id(&pool, &id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    Ok(Json(updated))
}

pub async fn delete_target(
    State(_pool): State<Pool<Sqlite>>,
    Path(_id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_target_timeline(
    State(pool): State<Pool<Sqlite>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    use sqlx::Row;

    // Build timeline from multiple sources:
    // 1. Target status changes (from targets table updated_at)
    // 2. DTL entries (when target was added to DTL)
    // 3. JTB decisions (if target was in JTB session)
    // 4. BDA assessments (if target was struck)
    // 5. Decision log entries (if target was mentioned)
    // 6. Annotations (user comments on target)
    
    let mut timeline: Vec<serde_json::Value> = Vec::new();
    
    // Get target creation and updates
    if let Ok(Some(target)) = TargetRepository::get_by_id(&pool, &id).await {
        timeline.push(serde_json::json!({
            "timestamp": target.id, // Use id as placeholder - would need created_at field
            "type": "TARGET_CREATED",
            "description": format!("Target {} created", target.name),
            "actor": "system",
        }));
    }
    
    // Get DTL entries for this target
    if let Ok(dtl_entries) = sqlx::query("SELECT id, created_at, updated_at, priority_score, feasibility_score FROM dtl_entries WHERE target_id = ? ORDER BY created_at DESC")
        .bind(&id)
        .fetch_all(&pool)
        .await {
        for row in dtl_entries {
            let created_at: String = row.get("created_at");
            timeline.push(serde_json::json!({
                "timestamp": created_at,
                "type": "DTL_ADDED",
                "description": format!("Added to DTL with priority score {:.2}", row.get::<f64, _>("priority_score")),
                "actor": "system",
            }));
        }
    }
    
    // Get JTB decisions for this target
    if let Ok(jtb_targets) = sqlx::query(
        "SELECT jt.decision, jt.decided_at, jt.decided_by, js.session_name 
         FROM jtb_targets jt 
         JOIN jtb_sessions js ON jt.session_id = js.id 
         WHERE jt.target_id = ? AND jt.decision IS NOT NULL 
         ORDER BY jt.decided_at DESC"
    )
    .bind(&id)
    .fetch_all(&pool)
    .await {
        for row in jtb_targets {
            let decided_at: Option<String> = row.get("decided_at");
            let decision: Option<String> = row.get("decision");
            let decided_by: Option<String> = row.get("decided_by");
            let session_name: Option<String> = row.get("session_name");
            
            if let (Some(at), Some(dec)) = (decided_at, decision) {
                timeline.push(serde_json::json!({
                    "timestamp": at,
                    "type": "JTB_DECISION",
                    "description": format!("JTB decision: {} in session {}", dec, session_name.unwrap_or_default()),
                    "actor": decided_by.unwrap_or_else(|| "unknown".to_string()),
                }));
            }
        }
    }
    
    // Get BDA assessments for this target
    if let Ok(bdas) = sqlx::query(
        "SELECT id, physical_damage, functional_damage, recommendation, created_at 
         FROM bda_reports 
         WHERE target_id = ? 
         ORDER BY created_at DESC"
    )
    .bind(&id)
    .fetch_all(&pool)
    .await {
        for row in bdas {
            let created_at: String = row.get("created_at");
            let physical: String = row.get("physical_damage");
            let functional: String = row.get("functional_damage");
            timeline.push(serde_json::json!({
                "timestamp": created_at,
                "type": "BDA_ASSESSMENT",
                "description": format!("BDA: {} physical, {} functional", physical, functional),
                "actor": "system",
            }));
        }
    }
    
    // Get decision log entries mentioning this target
    if let Ok(decisions) = sqlx::query(
        "SELECT id, decision_text, decision_maker, decision_time 
         FROM decision_log 
         WHERE decision_text LIKE ? 
         ORDER BY decision_time DESC"
    )
    .bind(format!("%{}%", id))
    .fetch_all(&pool)
    .await {
        for row in decisions {
            let decision_time: Option<String> = row.get("decision_time");
            let decision_text: String = row.get("decision_text");
            let decision_maker: String = row.get("decision_maker");
            
            if let Some(time) = decision_time {
                timeline.push(serde_json::json!({
                    "timestamp": time,
                    "type": "DECISION_LOG",
                    "description": decision_text,
                    "actor": decision_maker,
                }));
            }
        }
    }
    
    // Sort timeline by timestamp (most recent first)
    timeline.sort_by(|a, b| {
        let time_a = a.get("timestamp").and_then(|v| v.as_str()).unwrap_or("");
        let time_b = b.get("timestamp").and_then(|v| v.as_str()).unwrap_or("");
        time_b.cmp(time_a) // Reverse order (newest first)
    });
    
    Ok(Json(serde_json::json!({
        "target_id": id,
        "timeline": timeline,
        "total_events": timeline.len()
    })))
}

#[derive(Debug, Deserialize)]
pub struct AdvanceStageRequest {
    pub new_stage: String,
}

pub async fn advance_f3ead_stage(
    State(pool): State<Pool<Sqlite>>,
    Path(id): Path<String>,
    Json(req): Json<AdvanceStageRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    use crate::features::targeting::services::validate_f3ead_transition;
    
    // Get current target to check current stage
    let target = TargetRepository::get_by_id(&pool, &id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    // Get current F3EAD stage from target (defaults to "FIND" if not set)
    let current_stage = target.f3ead_stage.as_deref().unwrap_or("FIND");
    
    // Validate transition
    validate_f3ead_transition(current_stage, &req.new_stage)
        .map_err(|e| {
            eprintln!("F3EAD transition validation failed: {}", e);
            StatusCode::BAD_REQUEST
        })?;
    
    // Update target's F3EAD stage
    TargetRepository::update_f3ead_stage(&pool, &id, &req.new_stage)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok((StatusCode::OK, Json(serde_json::json!({
        "message": "F3EAD stage advanced",
        "target_id": id,
        "new_stage": req.new_stage
    }))))
}

pub async fn get_targeting_summary(
    State(pool): State<Pool<Sqlite>>,
) -> Result<impl IntoResponse, StatusCode> {
    let summary = TargetRepository::get_summary(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(summary))
}

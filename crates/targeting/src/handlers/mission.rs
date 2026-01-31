use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use sqlx::{Pool, Sqlite, Row};
use crate::features::auth::jwt::Claims;
use crate::features::targeting::domain::*;

// ============================================================================
// DECISION GATES - Dashboard operational status
// ============================================================================

/// Get current decision gates status (ROE, CDE, Weather, Deconfliction)
/// Used by DecisionGatesBar component on targeting cell dashboard
pub async fn get_decision_gates(
    State(pool): State<Pool<Sqlite>>,
) -> Result<impl IntoResponse, StatusCode> {
    // Fetch ROE status
    let roe = get_roe_gate(&pool).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Fetch CDE status
    let cde = get_cde_gate(&pool).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Fetch Weather status
    let weather = get_weather_gate(&pool).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Fetch Deconfliction status
    let deconfliction = get_deconfliction_gate(&pool).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let response = DecisionGatesResponse {
        roe,
        cde,
        weather,
        deconfliction,
    };
    
    Ok(Json(response))
}

// Helper functions to fetch individual gate statuses

async fn get_roe_gate(pool: &Pool<Sqlite>) -> Result<DecisionGate, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT status, caveats, classification, updated_at
        FROM roe_status
        WHERE is_current = 1
        ORDER BY updated_at DESC
        LIMIT 1
        "#
    )
    .fetch_optional(pool)
    .await?;
    
    match row {
        Some(row) => {
            let status_text: String = row.get("status");
            let caveats_json: Option<String> = row.get("caveats");
            let classification: String = row.get("classification");
            
            let caveats = caveats_json
                .and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok());
            
            let status = match status_text.as_str() {
                "WEAPON_FREE" => GateStatus::Green,
                "WEAPON_TIGHT" => GateStatus::Yellow,
                _ => GateStatus::Red,
            };
            
            Ok(DecisionGate {
                name: "ROE".to_string(),
                status,
                value: status_text,
                classification: map_classification(&classification),
                caveats,
                details: Some("Rules of Engagement".to_string()),
            })
        }
        None => Ok(DecisionGate {
            name: "ROE".to_string(),
            status: GateStatus::Yellow,
            value: "PENDING".to_string(),
            classification: ClassificationLevel::Secret,
            caveats: Some(vec!["NOFORN".to_string()]),
            details: Some("No ROE data available".to_string()),
        }),
    }
}

async fn get_cde_gate(pool: &Pool<Sqlite>) -> Result<DecisionGate, sqlx::Error> {
    // Count pending CDE assessments
    let pending_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM cde_status WHERE status = 'PENDING'"
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);
    
    // Get CDE limit from most recent record
    let row = sqlx::query(
        "SELECT cde_limit, classification FROM cde_status ORDER BY updated_at DESC LIMIT 1"
    )
    .fetch_optional(pool)
    .await?;
    
    match row {
        Some(row) => {
            let limit: i32 = row.get("cde_limit");
            let classification: String = row.get("classification");
            
            let status = if pending_count == 0 {
                GateStatus::Green
            } else if pending_count < limit as i64 {
                GateStatus::Yellow
            } else {
                GateStatus::Red
            };
            
            Ok(DecisionGate {
                name: "CDE".to_string(),
                status,
                value: format!("{}/{}", pending_count, limit),
                classification: map_classification(&classification),
                caveats: None,
                details: Some("Collateral Damage Estimate".to_string()),
            })
        }
        None => Ok(DecisionGate {
            name: "CDE".to_string(),
            status: GateStatus::Green,
            value: "0/50".to_string(),
            classification: ClassificationLevel::Secret,
            caveats: None,
            details: Some("No CDE data available".to_string()),
        }),
    }
}

async fn get_weather_gate(pool: &Pool<Sqlite>) -> Result<DecisionGate, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT status, current_conditions, visibility_km, classification
        FROM weather_status
        WHERE is_current = 1
        ORDER BY last_updated DESC
        LIMIT 1
        "#
    )
    .fetch_optional(pool)
    .await?;
    
    match row {
        Some(row) => {
            let status_text: String = row.get("status");
            let conditions: Option<String> = row.get("current_conditions");
            let visibility: Option<f64> = row.get("visibility_km");
            let classification: String = row.get("classification");
            
            let status = match status_text.as_str() {
                "GREEN" => GateStatus::Green,
                "YELLOW" => GateStatus::Yellow,
                _ => GateStatus::Red,
            };
            
            let value = if let (Some(cond), Some(vis)) = (conditions, visibility) {
                format!("{} ({}km)", cond, vis)
            } else {
                status_text.clone()
            };
            
            Ok(DecisionGate {
                name: "Weather".to_string(),
                status,
                value,
                classification: map_classification(&classification),
                caveats: None,
                details: Some("Environmental conditions".to_string()),
            })
        }
        None => Ok(DecisionGate {
            name: "Weather".to_string(),
            status: GateStatus::Green,
            value: "CLEAR".to_string(),
            classification: ClassificationLevel::Unclass,
            caveats: None,
            details: Some("No weather data available".to_string()),
        }),
    }
}

async fn get_deconfliction_gate(pool: &Pool<Sqlite>) -> Result<DecisionGate, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT status, pending_deconflictions, classification
        FROM deconfliction_status
        WHERE is_current = 1
        ORDER BY last_updated DESC
        LIMIT 1
        "#
    )
    .fetch_optional(pool)
    .await?;
    
    match row {
        Some(row) => {
            let deconf_status: String = row.get("status");
            let pending: i32 = row.get("pending_deconflictions");
            let classification: String = row.get("classification");
            
            let status = match deconf_status.as_str() {
                "GREEN" => GateStatus::Green,
                "YELLOW" => GateStatus::Yellow,
                "RED" => GateStatus::Red,
                _ => if pending == 0 { GateStatus::Green } else { GateStatus::Yellow },
            };
            
            Ok(DecisionGate {
                name: "Decon".to_string(),
                status,
                value: format!("{} ({} pending)", deconf_status, pending),
                classification: map_classification(&classification),
                caveats: None,
                details: Some("Airspace coordination".to_string()),
            })
        }
        None => Ok(DecisionGate {
            name: "Decon".to_string(),
            status: GateStatus::Green,
            value: "CLEAR (0 pending)".to_string(),
            classification: ClassificationLevel::Cui,
            caveats: None,
            details: Some("No deconfliction data available".to_string()),
        }),
    }
}

fn map_classification(s: &str) -> ClassificationLevel {
    match s.to_uppercase().as_str() {
        "UNCLASS" => ClassificationLevel::Unclass,
        "CUI" => ClassificationLevel::Cui,
        "SECRET" => ClassificationLevel::Secret,
        "TOP_SECRET" | "TOP SECRET" => ClassificationLevel::TopSecret,
        "TS_SCI" | "TS/SCI" => ClassificationLevel::TsSci,
        _ => ClassificationLevel::Secret,
    }
}

// ============================================================================
// MISSION COMMAND - Dashboard support for MissionCommandOverview component
// ============================================================================

/// Get commander's intent (phase, priority effects, endstate, metrics)
/// Used by MissionCommandOverview component
pub async fn get_mission_intent(
    State(pool): State<Pool<Sqlite>>,
) -> Result<impl IntoResponse, StatusCode> {
    let row = sqlx::query(
        "SELECT phase, priority_effects, endstate, endstate_metrics 
         FROM mission_intent 
         WHERE is_current = 1 
         ORDER BY updated_at DESC 
         LIMIT 1"
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    match row {
        Some(row) => {
            let phase: String = row.get("phase");
            let priority_effects_json: String = row.get("priority_effects");
            let endstate: String = row.get("endstate");
            let endstate_metrics_json: String = row.get("endstate_metrics");
            
            let priority_effects: Vec<String> = serde_json::from_str(&priority_effects_json)
                .unwrap_or_else(|_| vec![]);
            let endstate_metrics: serde_json::Value = serde_json::from_str(&endstate_metrics_json)
                .unwrap_or_else(|_| serde_json::json!([]));
            
            Ok(Json(serde_json::json!({
                "phase": phase,
                "priorityEffects": priority_effects,
                "endstate": endstate,
                "endstateMetrics": endstate_metrics
            })))
        }
        None => {
            // Fallback to default if no record exists
            Ok(Json(serde_json::json!({
                "phase": "Phase 2: Hostile Force Degradation",
                "priorityEffects": [
                    "Disrupt enemy C2 networks",
                    "Attrit enemy armor by 30%",
                    "Deny enemy freedom of movement in AO-North"
                ],
                "endstate": "Enemy unable to conduct coordinated operations in AO-North",
                "endstateMetrics": [
                    { "name": "Enemy C2 Nodes Destroyed", "current": 12, "target": 18, "status": "at-risk" },
                    { "name": "Armor Attrition %", "current": 22, "target": 30, "status": "at-risk" },
                    { "name": "Movement Denial Coverage %", "current": 87, "target": 90, "status": "on-track" }
                ]
            })))
        }
    }
}

/// Update commander's intent
pub async fn update_mission_intent(
    State(pool): State<Pool<Sqlite>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<serde_json::Value>,
) -> Result<impl IntoResponse, StatusCode> {
    let user_id = &claims.sub;
    
    // Extract fields from request
    let phase = req.get("phase").and_then(|v| v.as_str()).unwrap_or("");
    let priority_effects = req.get("priorityEffects")
        .and_then(|v| serde_json::to_string(v).ok())
        .unwrap_or_else(|| "[]".to_string());
    let endstate = req.get("endstate").and_then(|v| v.as_str()).unwrap_or("");
    let endstate_metrics = req.get("endstateMetrics")
        .and_then(|v| serde_json::to_string(v).ok())
        .unwrap_or_else(|| "[]".to_string());
    
    // Mark existing current intent as not current
    sqlx::query("UPDATE mission_intent SET is_current = 0 WHERE is_current = 1")
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Insert new intent as current
    sqlx::query(
        "INSERT INTO mission_intent (phase, priority_effects, endstate, endstate_metrics, created_by, is_current)
         VALUES (?, ?, ?, ?, ?, 1)"
    )
    .bind(phase)
    .bind(&priority_effects)
    .bind(endstate)
    .bind(&endstate_metrics)
    .bind(user_id)
    .execute(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok((StatusCode::OK, Json(serde_json::json!({"message": "Intent updated"}))))
}

/// Get targeting guidance (ROE level, collateral threshold, approved target sets, restrictions)
pub async fn get_targeting_guidance(
    State(pool): State<Pool<Sqlite>>,
) -> Result<impl IntoResponse, StatusCode> {
    let row = sqlx::query(
        "SELECT roe_level, collateral_threshold, approved_target_sets, restrictions 
         FROM targeting_guidance 
         WHERE is_current = 1 
         ORDER BY updated_at DESC 
         LIMIT 1"
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    match row {
        Some(row) => {
            let roe_level: String = row.get("roe_level");
            let collateral_threshold: String = row.get("collateral_threshold");
            let approved_target_sets_json: String = row.get("approved_target_sets");
            let restrictions_json: String = row.get("restrictions");
            
            let approved_target_sets: Vec<String> = serde_json::from_str(&approved_target_sets_json)
                .unwrap_or_else(|_| vec![]);
            let restrictions: Vec<String> = serde_json::from_str(&restrictions_json)
                .unwrap_or_else(|_| vec![]);
            
            Ok(Json(serde_json::json!({
                "roeLevel": roe_level,
                "collateralThreshold": collateral_threshold,
                "approvedTargetSets": approved_target_sets,
                "restrictions": restrictions
            })))
        }
        None => {
            // Fallback: Get ROE level from roe_status table
            let roe_level = sqlx::query("SELECT status FROM roe_status WHERE is_current = 1 ORDER BY updated_at DESC LIMIT 1")
                .fetch_optional(&pool)
                .await
                .ok()
                .flatten()
                .map(|row| row.get::<String, _>("status"))
                .unwrap_or_else(|| "WEAPON_FREE".to_string());
            
            Ok(Json(serde_json::json!({
                "roeLevel": roe_level,
                "collateralThreshold": "CDE < 50 civilian casualties per strike",
                "approvedTargetSets": [
                    "Category 1: C2 Nodes (High Priority)",
                    "Category 2: Armor Formations (Medium Priority)",
                    "Category 3: Logistics Hubs (Low Priority)"
                ],
                "restrictions": [
                    "NO STRIKE: Cultural/religious sites",
                    "NO STRIKE: Medical facilities",
                    "RESTRICTED: Dual-use infrastructure (Commander approval required)"
                ]
            })))
        }
    }
}

/// Get decision authority matrix (level, authority, can approve, must escalate)
pub async fn get_authority_matrix(
    State(pool): State<Pool<Sqlite>>,
) -> Result<impl IntoResponse, StatusCode> {
    let row = sqlx::query(
        "SELECT level, authority, can_approve, must_escalate 
         FROM decision_authority 
         WHERE is_current = 1 
         ORDER BY updated_at DESC 
         LIMIT 1"
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    match row {
        Some(row) => {
            let level: String = row.get("level");
            let authority: String = row.get("authority");
            let can_approve_json: String = row.get("can_approve");
            let must_escalate_json: String = row.get("must_escalate");
            
            let can_approve: Vec<String> = serde_json::from_str(&can_approve_json)
                .unwrap_or_else(|_| vec![]);
            let must_escalate: Vec<String> = serde_json::from_str(&must_escalate_json)
                .unwrap_or_else(|_| vec![]);
            
            Ok(Json(serde_json::json!({
                "level": level,
                "authority": authority,
                "canApprove": can_approve,
                "mustEscalate": must_escalate
            })))
        }
        None => {
            // Fallback to default
            Ok(Json(serde_json::json!({
                "level": "OPERATIONAL",
                "authority": "Commander, Joint Task Force",
                "canApprove": [
                    "Category 1-2 targets (CDE < 50)",
                    "Time-sensitive targets (CDE < 20)",
                    "ROE modifications within authority"
                ],
                "mustEscalate": [
                    "Category 1-2 targets (CDE ≥ 50)",
                    "All Category 3 targets",
                    "Strategic targets",
                    "Cross-border strikes"
                ]
            })))
        }
    }
}

/// Get operational tempo (current phase, hours into phase, critical decision points)
pub async fn get_operational_tempo(
    State(pool): State<Pool<Sqlite>>,
) -> Result<impl IntoResponse, StatusCode> {
    let row = sqlx::query(
        "SELECT current_phase, hours_into_phase, critical_decision_points 
         FROM operational_tempo 
         WHERE is_current = 1 
         ORDER BY updated_at DESC 
         LIMIT 1"
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    match row {
        Some(row) => {
            let current_phase: String = row.get("current_phase");
            let hours_into_phase: i32 = row.get("hours_into_phase");
            let critical_points_json: String = row.get("critical_decision_points");
            
            let critical_points: Vec<String> = serde_json::from_str(&critical_points_json)
                .unwrap_or_else(|_| vec![]);
            
            Ok(Json(serde_json::json!({
                "currentPhase": current_phase,
                "hoursIntoPhase": hours_into_phase,
                "criticalDecisionPoints": critical_points
            })))
        }
        None => {
            // Fallback
            Ok(Json(serde_json::json!({
                "currentPhase": "PHASE 2: SEIZE",
                "hoursIntoPhase": 72,
                "criticalDecisionPoints": [
                    "Commitment of Reserve Force (H+96)",
                    "Transition to Phase 3 (H+120)"
                ]
            })))
        }
    }
}

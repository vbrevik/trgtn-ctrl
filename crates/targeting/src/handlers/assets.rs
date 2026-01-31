use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use sqlx::{Pool, Sqlite, Row}; // Added Row import
use crate::features::targeting::domain::*;
use crate::features::targeting::repositories::*;
use super::common::PlatformQueryParams;

pub async fn list_strike_platforms(
    State(pool): State<Pool<Sqlite>>,
    Query(params): Query<PlatformQueryParams>,
) -> Result<impl IntoResponse, StatusCode> {
    let platforms = StrikePlatformRepository::list_all(&pool, params.status.as_deref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(platforms))
}

pub async fn create_strike_platform(
    State(pool): State<Pool<Sqlite>>,
    Json(req): Json<CreateStrikePlatformRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let platform = StrikePlatform {
        id: String::new(),
        platform_type: req.platform_type,
        platform_name: req.platform_name,
        callsign: None,
        unit: None,
        munitions_available: None,
        sorties_available: req.sorties_available,
        platform_status: "READY".to_string(),
        classification: req.classification,
        created_at: String::new(),
        updated_at: String::new(),
    };
    
    let id = StrikePlatformRepository::create(&pool, &platform)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({"id": id}))))
}

pub async fn get_munitions_inventory(
    State(pool): State<Pool<Sqlite>>,
) -> Result<impl IntoResponse, StatusCode> {
    // Query munitions table
    let rows = sqlx::query(
        "SELECT id, munition_type, munition_category, platform_type, 
                total_count, available_count, allocated_count, expended_count,
                range_km, yield_kg, guidance_type, warhead_type,
                suitable_target_types, cde_estimate_range, classification
         FROM munitions
         ORDER BY available_count DESC, munition_type"
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let mut inventory = Vec::new();
    
    for row in rows {
        let suitable_types_json: Option<String> = row.get("suitable_target_types");
        let cde_range_json: Option<String> = row.get("cde_estimate_range");
        
        // Fix: Use correct return type for get()
        let suitable_types: Vec<String> = suitable_types_json
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        
        let cde_range: serde_json::Value = cde_range_json
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| serde_json::json!({"min": 0, "max": 0}));
        
        // Fix: Use generic get explicitly or ensure type inference is correct
        inventory.push(serde_json::json!({
            "id": row.get::<String, _>("id"),
            "munitionType": row.get::<String, _>("munition_type"),
            "category": row.get::<String, _>("munition_category"),
            "platformType": row.get::<Option<String>, _>("platform_type"),
            "totalCount": row.get::<i32, _>("total_count"),
            "availableCount": row.get::<i32, _>("available_count"),
            "allocatedCount": row.get::<i32, _>("allocated_count"),
            "expendedCount": row.get::<i32, _>("expended_count"),
            "rangeKm": row.get::<Option<f64>, _>("range_km"),
            "yieldKg": row.get::<Option<f64>, _>("yield_kg"),
            "guidanceType": row.get::<Option<String>, _>("guidance_type"),
            "warheadType": row.get::<Option<String>, _>("warhead_type"),
            "suitableTargetTypes": suitable_types,
            "cdeEstimateRange": cde_range,
            "classification": row.get::<String, _>("classification"),
        }));
    }
    
    Ok(Json(serde_json::json!({
        "inventory": inventory,
        "total_types": inventory.len(),
        "total_available": inventory.iter()
            .map(|m| m.get("availableCount").and_then(|v| v.as_i64()).unwrap_or(0))
            .sum::<i64>()
    })))
}

use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct MunitionsPairingRequest {
    pub target_id: String,
    pub desired_effects: Vec<String>,
}

pub async fn get_munitions_pairing(
    State(pool): State<Pool<Sqlite>>,
    Json(req): Json<MunitionsPairingRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // Get target information
    let target = TargetRepository::get_by_id(&pool, &req.target_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    // Get available munitions
    let munitions_rows = sqlx::query(
        "SELECT id, munition_type, munition_category, platform_type, available_count,
                range_km, yield_kg, guidance_type, warhead_type,
                suitable_target_types, cde_estimate_range
         FROM munitions
         WHERE available_count > 0
         ORDER BY available_count DESC"
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let mut recommendations = Vec::new();
    
    for row in munitions_rows {
        let suitable_types_json: Option<String> = row.get("suitable_target_types");
        let suitable_types: Vec<String> = suitable_types_json
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        
        // Simple matching algorithm:
        // 1. Check if target type matches suitable target types
        // 2. Check if desired effects can be achieved
        // 3. Score based on availability and suitability
        
        let target_type_match = suitable_types.iter()
            .any(|t| target.target_type.to_uppercase().contains(&t.to_uppercase()));
        
        let score: f64 = if target_type_match {
            // Base score for type match
            let mut score: f64 = 0.7;
            
            // Boost score if available count is high
            let available: i32 = row.get("available_count");
            if available > 10 {
                score += 0.2;
            }
            
            // Boost if range is sufficient (placeholder - would need target distance)
            if let Some(range) = row.get::<Option<f64>, _>("range_km") {
                if range > 20.0 {
                    score += 0.1;
                }
            }
            
            score.min(1.0)
        } else {
            0.3 // Lower score if type doesn't match but still available
        };
        
        if score > 0.5 { // Only recommend if score > 50%
            let cde_range_json: Option<String> = row.get("cde_estimate_range");
            let cde_range: serde_json::Value = cde_range_json
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_else(|| serde_json::json!({"min": 0, "max": 0}));
            
            recommendations.push(serde_json::json!({
                "munitionId": row.get::<String, _>("id"),
                "munitionType": row.get::<String, _>("munition_type"),
                "category": row.get::<String, _>("munition_category"),
                "platformType": row.get::<Option<String>, _>("platform_type"),
                "availableCount": row.get::<i32, _>("available_count"),
                "rangeKm": row.get::<Option<f64>, _>("range_km"),
                "yieldKg": row.get::<Option<f64>, _>("yield_kg"),
                "score": score,
                "reasoning": if target_type_match { "Suitable target type" } else { "Available fallback" },
                "cdeEstimate": cde_range,
            }));
        }
    }
    
    // Sort by score descending
    recommendations.sort_by(|a, b| {
        let score_a = a.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let score_b = b.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
        score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
    });
    
    Ok(Json(serde_json::json!({
        "target_id": req.target_id,
        "recommendations": recommendations,
        "count": recommendations.len()
    })))
}

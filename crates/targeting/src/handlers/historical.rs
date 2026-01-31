// Historical Data Handlers
// Endpoints for querying historical targeting data with date ranges

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, Row};
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize)]
pub struct HistoricalQueryParams {
    pub from: Option<String>, // ISO 8601 date
    pub to: Option<String>,   // ISO 8601 date
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct HistoricalTargetStatus {
    pub date: String,
    #[serde(flatten)]
    pub status_counts: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct HistoricalF3EAD {
    pub date: String,
    pub find: i64,
    pub fix: i64,
    pub finish: i64,
    pub exploit: i64,
    pub analyze: i64,
    pub disseminate: i64,
}

#[derive(Debug, Serialize)]
pub struct HistoricalBDA {
    pub date: String,
    pub effective: i64,
    pub partial: i64,
    pub ineffective: i64,
    pub pending: i64,
}

/// Get historical target status distribution
/// GET /api/targeting/historical/status
pub async fn get_historical_status(
    State(pool): State<Pool<Sqlite>>,
    Query(params): Query<HistoricalQueryParams>,
) -> Result<impl IntoResponse, StatusCode> {
    let default_to = Utc::now().to_rfc3339();
    let from_date = params.from.as_deref().unwrap_or("1970-01-01T00:00:00Z");
    let to_date = params.to.as_deref().unwrap_or(&default_to);
    let limit = params.limit.unwrap_or(30).min(365); // Max 365 days

    // Parse dates
    let from = DateTime::parse_from_rfc3339(from_date)
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .with_timezone(&Utc);
    let to = DateTime::parse_from_rfc3339(&to_date)
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .with_timezone(&Utc);

    // Query target status changes grouped by date
    let query = r#"
        SELECT 
            DATE(created_at) as date,
            target_status,
            COUNT(*) as count
        FROM targets
        WHERE created_at >= ? AND created_at <= ?
        GROUP BY DATE(created_at), target_status
        ORDER BY date DESC
        LIMIT ?
    "#;

    let from_str = from.to_rfc3339();
    let to_str = to.to_rfc3339();
    
    let rows = sqlx::query(query)
        .bind(&from_str)
        .bind(&to_str)
        .bind(limit)
        .fetch_all(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Group by date
    let mut status_by_date: std::collections::HashMap<String, serde_json::Value> = std::collections::HashMap::new();

    for row in rows {
        let date: String = row.get("date");
        let status: String = row.get("target_status");
        let count: i64 = row.get("count");

        let entry = status_by_date.entry(date.clone()).or_insert_with(|| serde_json::json!({}));
        entry.as_object_mut().unwrap().insert(status, serde_json::json!(count));
    }

    // Convert to array format
    let result: Vec<HistoricalTargetStatus> = status_by_date
        .into_iter()
        .map(|(date, status_counts)| HistoricalTargetStatus {
            date,
            status_counts,
        })
        .collect();

    Ok(Json(result))
}

/// Get historical F3EAD pipeline distribution
/// GET /api/targeting/historical/f3ead
pub async fn get_historical_f3ead(
    State(pool): State<Pool<Sqlite>>,
    Query(params): Query<HistoricalQueryParams>,
) -> Result<impl IntoResponse, StatusCode> {
    let default_to = Utc::now().to_rfc3339();
    let from_date = params.from.as_deref().unwrap_or("1970-01-01T00:00:00Z");
    let to_date = params.to.as_deref().unwrap_or(&default_to);
    let limit = params.limit.unwrap_or(30).min(365);

    let from = DateTime::parse_from_rfc3339(from_date)
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .with_timezone(&Utc);
    let to = DateTime::parse_from_rfc3339(&to_date)
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .with_timezone(&Utc);

    let query = r#"
        SELECT 
            DATE(created_at) as date,
            f3ead_stage,
            COUNT(*) as count
        FROM targets
        WHERE created_at >= ? AND created_at <= ?
        GROUP BY DATE(created_at), f3ead_stage
        ORDER BY date DESC
        LIMIT ?
    "#;

    let from_str = from.to_rfc3339();
    let to_str = to.to_rfc3339();
    
    let rows = sqlx::query(query)
        .bind(&from_str)
        .bind(&to_str)
        .bind(limit)
        .fetch_all(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut f3ead_by_date: std::collections::HashMap<String, HistoricalF3EAD> = std::collections::HashMap::new();

    for row in rows {
        let date: String = row.get("date");
        let stage: String = row.get("f3ead_stage");
        let count: i64 = row.get("count");

        let entry = f3ead_by_date.entry(date.clone()).or_insert_with(|| HistoricalF3EAD {
            date: date.clone(),
            find: 0,
            fix: 0,
            finish: 0,
            exploit: 0,
            analyze: 0,
            disseminate: 0,
        });

        match stage.as_str() {
            "FIND" => entry.find = count,
            "FIX" => entry.fix = count,
            "FINISH" => entry.finish = count,
            "EXPLOIT" => entry.exploit = count,
            "ANALYZE" => entry.analyze = count,
            "DISSEMINATE" => entry.disseminate = count,
            _ => {}
        }
    }

    let result: Vec<HistoricalF3EAD> = f3ead_by_date.into_values().collect();
    Ok(Json(result))
}

/// Get historical BDA assessment distribution
/// GET /api/targeting/historical/bda
pub async fn get_historical_bda(
    State(pool): State<Pool<Sqlite>>,
    Query(params): Query<HistoricalQueryParams>,
) -> Result<impl IntoResponse, StatusCode> {
    let default_to = Utc::now().to_rfc3339();
    let from_date = params.from.as_deref().unwrap_or("1970-01-01T00:00:00Z");
    let to_date = params.to.as_deref().unwrap_or(&default_to);
    let limit = params.limit.unwrap_or(30).min(365);

    let from = DateTime::parse_from_rfc3339(from_date)
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .with_timezone(&Utc);
    let to = DateTime::parse_from_rfc3339(&to_date)
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .with_timezone(&Utc);

    // Query BDA reports grouped by date and recommendation
    let query = r#"
        SELECT 
            DATE(created_at) as date,
            recommendation,
            COUNT(*) as count
        FROM bda_reports
        WHERE created_at >= ? AND created_at <= ?
        GROUP BY DATE(created_at), recommendation
        ORDER BY date DESC
        LIMIT ?
    "#;

    let from_str = from.to_rfc3339();
    let to_str = to.to_rfc3339();
    
    let rows = sqlx::query(query)
        .bind(&from_str)
        .bind(&to_str)
        .bind(limit)
        .fetch_all(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut bda_by_date: std::collections::HashMap<String, HistoricalBDA> = std::collections::HashMap::new();

    for row in rows {
        let date: String = row.get("date");
        let recommendation: Option<String> = row.get("recommendation");
        let count: i64 = row.get("count");

        let date_key = date.clone();
        let entry = bda_by_date.entry(date_key).or_insert_with(|| HistoricalBDA {
            date: date.clone(),
            effective: 0,
            partial: 0,
            ineffective: 0,
            pending: 0,
        });

        match recommendation.as_deref() {
            Some("EFFECTIVE") => entry.effective = count,
            Some("PARTIAL") => entry.partial = count,
            Some("INEFFECTIVE") => entry.ineffective = count,
            _ => entry.pending = count,
        }
    }

    let result: Vec<HistoricalBDA> = bda_by_date.into_values().collect();
    Ok(Json(result))
}

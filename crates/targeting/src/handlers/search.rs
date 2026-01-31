// Search Handler
// Provides unified search across targets, BDA, decisions, and other entities

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, Row};

#[derive(Debug, Deserialize)]
pub struct SearchQueryParams {
    pub q: String, // Search query string
    pub limit: Option<i64>,
    pub types: Option<String>, // Comma-separated: "targets,bda,decisions"
}

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub id: String,
    pub r#type: String, // "target", "bda", "decision", "isr_platform", "strike_platform"
    pub title: String,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total: usize,
    pub query: String,
}

/// Unified search endpoint
/// Searches across targets, BDA reports, decisions, ISR platforms, and strike platforms
pub async fn search(
    State(pool): State<Pool<Sqlite>>,
    Query(params): Query<SearchQueryParams>,
) -> Result<impl IntoResponse, StatusCode> {
    let query = params.q.trim();
    if query.is_empty() {
        return Ok(Json(SearchResponse {
            results: Vec::new(),
            total: 0,
            query: query.to_string(),
        }));
    }

    let limit = params.limit.unwrap_or(20).min(100); // Max 100 results
    let search_types = params.types.as_deref().unwrap_or("targets,bda,decisions,isr_platforms,strike_platforms");
    let types_vec: Vec<&str> = search_types.split(',').map(|s| s.trim()).collect();

    let mut all_results = Vec::new();

    // Search Targets
    if types_vec.is_empty() || types_vec.contains(&"targets") {
        let targets = search_targets(&pool, query, limit).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        all_results.extend(targets);
    }

    // Search BDA Reports
    if types_vec.is_empty() || types_vec.contains(&"bda") {
        let bda = search_bda(&pool, query, limit).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        all_results.extend(bda);
    }

    // Search Decisions
    if types_vec.is_empty() || types_vec.contains(&"decisions") {
        let decisions = search_decisions(&pool, query, limit).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        all_results.extend(decisions);
    }

    // Search ISR Platforms
    if types_vec.is_empty() || types_vec.contains(&"isr_platforms") {
        let isr = search_isr_platforms(&pool, query, limit).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        all_results.extend(isr);
    }

    // Search Strike Platforms
    if types_vec.is_empty() || types_vec.contains(&"strike_platforms") {
        let strike = search_strike_platforms(&pool, query, limit).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        all_results.extend(strike);
    }

    // Sort by relevance (simple: prioritize matches in name/title)
    all_results.sort_by(|a, b| {
        let a_score = calculate_relevance_score(&a.title, query);
        let b_score = calculate_relevance_score(&b.title, query);
        b_score.cmp(&a_score)
    });

    // Limit total results
    all_results.truncate(limit as usize);

    Ok(Json(SearchResponse {
        total: all_results.len(),
        results: all_results,
        query: query.to_string(),
    }))
}

/// Search targets by name, description, or ID
async fn search_targets(
    pool: &Pool<Sqlite>,
    query: &str,
    limit: i64,
) -> Result<Vec<SearchResult>, sqlx::Error> {
    let search_pattern = format!("%{}%", query);
    let rows = sqlx::query(
        "SELECT id, name, description, target_status, priority, f3ead_stage
         FROM targets
         WHERE name LIKE ?1 OR description LIKE ?1 OR id LIKE ?1
         LIMIT ?2"
    )
    .bind(&search_pattern)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let mut results = Vec::new();
    for row in rows {
        results.push(SearchResult {
            id: row.get("id"),
            r#type: "target".to_string(),
            title: row.get("name"),
            description: row.get("description"),
            status: row.get("target_status"),
            priority: row.get("priority"),
            metadata: serde_json::json!({
                "f3ead_stage": row.get::<Option<String>, _>("f3ead_stage").unwrap_or_default(),
            }),
        });
    }
    Ok(results)
}

/// Search BDA reports by target name, assessment type, or ID
async fn search_bda(
    pool: &Pool<Sqlite>,
    query: &str,
    limit: i64,
) -> Result<Vec<SearchResult>, sqlx::Error> {
    let search_pattern = format!("%{}%", query);
    let rows = sqlx::query(
        "SELECT b.id, b.target_id, b.assessment_type, b.damage_level, b.status,
                t.name as target_name
         FROM bda_assessments b
         LEFT JOIN targets t ON b.target_id = t.id
         WHERE t.name LIKE ?1 OR b.assessment_type LIKE ?1 OR b.id LIKE ?1
         LIMIT ?2"
    )
    .bind(&search_pattern)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let mut results = Vec::new();
    for row in rows {
        let target_name: Option<String> = row.get("target_name");
        let assessment_type: Option<String> = row.get("assessment_type");
        let title = format!("BDA: {}", target_name.as_deref().unwrap_or("Unknown Target"));
        
        results.push(SearchResult {
            id: row.get("id"),
            r#type: "bda".to_string(),
            title,
            description: assessment_type,
            status: row.get("status"),
            priority: None,
            metadata: serde_json::json!({
                "target_id": row.get::<Option<String>, _>("target_id"),
                "damage_level": row.get::<Option<String>, _>("damage_level"),
            }),
        });
    }
    Ok(results)
}

/// Search decisions by description, type, or ID
async fn search_decisions(
    pool: &Pool<Sqlite>,
    query: &str,
    limit: i64,
) -> Result<Vec<SearchResult>, sqlx::Error> {
    let search_pattern = format!("%{}%", query);
    let rows = sqlx::query(
        "SELECT id, decision_type, description, decision, created_at
         FROM targeting_decisions
         WHERE description LIKE ?1 OR decision_type LIKE ?1 OR id LIKE ?1
         LIMIT ?2"
    )
    .bind(&search_pattern)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let mut results = Vec::new();
    for row in rows {
        let decision_type: Option<String> = row.get("decision_type");
        let title = format!("Decision: {}", decision_type.as_deref().unwrap_or("Unknown Type"));
        
        results.push(SearchResult {
            id: row.get("id"),
            r#type: "decision".to_string(),
            title,
            description: row.get("description"),
            status: row.get("decision"),
            priority: None,
            metadata: serde_json::json!({
                "decision_type": decision_type,
                "created_at": row.get::<Option<String>, _>("created_at"),
            }),
        });
    }
    Ok(results)
}

/// Search ISR platforms by name, type, or ID
async fn search_isr_platforms(
    pool: &Pool<Sqlite>,
    query: &str,
    limit: i64,
) -> Result<Vec<SearchResult>, sqlx::Error> {
    let search_pattern = format!("%{}%", query);
    let rows = sqlx::query(
        "SELECT id, platform_name, platform_type, status
         FROM isr_platforms
         WHERE platform_name LIKE ?1 OR platform_type LIKE ?1 OR id LIKE ?1
         LIMIT ?2"
    )
    .bind(&search_pattern)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let mut results = Vec::new();
    for row in rows {
        let platform_name: Option<String> = row.get("platform_name");
        let platform_type: Option<String> = row.get("platform_type");
        let title = format!("ISR: {}", platform_name.as_deref().unwrap_or("Unknown Platform"));
        
        let platform_type_clone = platform_type.clone();
        results.push(SearchResult {
            id: row.get("id"),
            r#type: "isr_platform".to_string(),
            title,
            description: platform_type,
            status: row.get("status"),
            priority: None,
            metadata: serde_json::json!({
                "platform_type": platform_type_clone,
            }),
        });
    }
    Ok(results)
}

/// Search strike platforms by name, type, or ID
async fn search_strike_platforms(
    pool: &Pool<Sqlite>,
    query: &str,
    limit: i64,
) -> Result<Vec<SearchResult>, sqlx::Error> {
    let search_pattern = format!("%{}%", query);
    let rows = sqlx::query(
        "SELECT id, platform_name, platform_type, platform_status
         FROM strike_platforms
         WHERE platform_name LIKE ?1 OR platform_type LIKE ?1 OR id LIKE ?1
         LIMIT ?2"
    )
    .bind(&search_pattern)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let mut results = Vec::new();
    for row in rows {
        let platform_name: Option<String> = row.get("platform_name");
        let platform_type: Option<String> = row.get("platform_type");
        let title = format!("Strike: {}", platform_name.as_deref().unwrap_or("Unknown Platform"));
        
        let platform_type_clone = platform_type.clone();
        results.push(SearchResult {
            id: row.get("id"),
            r#type: "strike_platform".to_string(),
            title,
            description: platform_type,
            status: row.get("platform_status"),
            priority: None,
            metadata: serde_json::json!({
                "platform_type": platform_type_clone,
            }),
        });
    }
    Ok(results)
}

/// Simple relevance scoring (exact match = 3, starts with = 2, contains = 1)
fn calculate_relevance_score(text: &str, query: &str) -> i32 {
    let text_lower = text.to_lowercase();
    let query_lower = query.to_lowercase();
    
    if text_lower == query_lower {
        3
    } else if text_lower.starts_with(&query_lower) {
        2
    } else if text_lower.contains(&query_lower) {
        1
    } else {
        0
    }
}

// Action Required Handler
// Aggregates action items from multiple sources: targets, DTL, JTB, CDE, etc.

use axum::{
    extract::{State, Extension},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sqlx::{Pool, Sqlite, Row};
use serde::Serialize;
use crate::features::auth::jwt::Claims;
use chrono;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionItem {
    pub id: String,
    #[serde(rename = "type")]
    pub item_type: String, // "target", "nomination", "review", "approval", "cde", "deconfliction"
    pub title: String,
    pub subtitle: Option<String>,
    pub classification: String,
    pub caveats: Option<Vec<String>>,
    pub priority: String, // "CRITICAL", "HIGH", "MEDIUM", "LOW"
    pub due_time: Option<String>,
    pub due_time_display: Option<String>,
    pub assigned_to_current_user: bool,
    pub blockers: Option<Vec<String>>,
    pub target_id: Option<String>,
}

pub async fn get_action_required(
    State(pool): State<Pool<Sqlite>>,
    Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, StatusCode> {
    let user_id = &claims.sub;
    let mut action_items: Vec<ActionItem> = Vec::new();
    
    // 1. TSTs with approaching deadlines (CRITICAL priority)
    let tsts = sqlx::query(
        "SELECT dtl.id, dtl.target_id, dtl.tst_deadline, t.name, t.classification
         FROM dtl_entries dtl
         JOIN targets t ON dtl.target_id = t.id
         WHERE dtl.is_tst = 1 AND dtl.tst_deadline IS NOT NULL
         ORDER BY dtl.tst_deadline ASC
         LIMIT 10"
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    for row in tsts {
        let deadline: String = row.get("tst_deadline");
        let target_name: String = row.get("name");
        let target_id: String = row.get("target_id");
        let classification: String = row.get("classification");
        
        // Calculate time remaining (parse ISO 8601 or SQLite datetime)
        let deadline_dt = deadline.parse::<chrono::DateTime<chrono::Utc>>()
            .or_else(|_| {
                // Try SQLite datetime format: "YYYY-MM-DD HH:MM:SS"
                chrono::NaiveDateTime::parse_from_str(&deadline, "%Y-%m-%d %H:%M:%S")
                    .map(|ndt| ndt.and_utc())
            })
            .ok();
        
        let (priority, due_time_display) = if let Some(deadline_utc) = deadline_dt {
            let now = chrono::Utc::now();
            let duration = deadline_utc - now;
            let minutes_remaining = duration.num_minutes();
            
            if minutes_remaining < 60 {
                ("CRITICAL".to_string(), Some(format!("in {} minutes", minutes_remaining)))
            } else if minutes_remaining < 120 {
                ("CRITICAL".to_string(), Some(format!("in {} hours", minutes_remaining / 60)))
            } else if minutes_remaining < 360 {
                ("HIGH".to_string(), Some(format!("in {} hours", minutes_remaining / 60)))
            } else {
                ("HIGH".to_string(), Some(format!("in {} hours", minutes_remaining / 60)))
            }
        } else {
            ("HIGH".to_string(), None)
        };
        
        action_items.push(ActionItem {
            id: format!("tst-{}", row.get::<String, _>("id")),
            item_type: "target".to_string(),
            title: format!("TST: {}", target_name),
            subtitle: Some("Time-Sensitive Target - Deadline approaching".to_string()),
            classification,
            caveats: Some(vec!["NOFORN".to_string()]),
            priority,
            due_time: Some(deadline),
            due_time_display,
            assigned_to_current_user: false, // Could check if user is assigned
            blockers: None,
            target_id: Some(target_id),
        });
    }
    
    // 2. Targets awaiting CDE analysis (HIGH priority)
    let cde_pending = sqlx::query(
        "SELECT t.id, t.name, t.classification, t.target_status
         FROM targets t
         LEFT JOIN risk_assessments r ON t.id = r.target_id
         WHERE t.target_status IN ('Nominated', 'Validated')
           AND (r.id IS NULL OR r.legal_review_status = 'PENDING')
         ORDER BY t.priority DESC, t.created_at ASC
         LIMIT 10"
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    for row in cde_pending {
        let target_id: String = row.get("id");
        let target_name: String = row.get("name");
        let classification: String = row.get("classification");
        let status: String = row.get("target_status");
        
        action_items.push(ActionItem {
            id: format!("cde-{}", target_id),
            item_type: "cde".to_string(),
            title: target_name,
            subtitle: Some(format!("CDE analysis required • Status: {}", status)),
            classification,
            caveats: Some(vec!["NOFORN".to_string()]),
            priority: "HIGH".to_string(),
            due_time: None,
            due_time_display: Some("ASAP".to_string()),
            assigned_to_current_user: false,
            blockers: Some(vec!["CDE analysis pending".to_string()]),
            target_id: Some(target_id),
        });
    }
    
    // 3. JTB sessions with upcoming targets (MEDIUM-HIGH priority)
    let upcoming_jtb = sqlx::query(
        "SELECT js.id, js.session_name, js.session_datetime, js.status,
                COUNT(jt.id) as target_count
         FROM jtb_sessions js
         LEFT JOIN jtb_targets jt ON js.id = jt.session_id
         WHERE js.status = 'SCHEDULED'
           AND js.session_datetime > datetime('now')
           AND js.session_datetime < datetime('now', '+24 hours')
         GROUP BY js.id
         ORDER BY js.session_datetime ASC
         LIMIT 5"
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    for row in upcoming_jtb {
        let session_id: String = row.get("id");
        let session_name: String = row.get("session_name");
        let session_datetime: String = row.get("session_datetime");
        let target_count: i64 = row.get("target_count");
        
        // Calculate time until session (parse ISO 8601 or SQLite datetime)
        let session_dt = session_datetime.parse::<chrono::DateTime<chrono::Utc>>()
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(&session_datetime, "%Y-%m-%d %H:%M:%S")
                    .map(|ndt| ndt.and_utc())
            })
            .ok();
        
        let due_time_display = if let Some(session_utc) = session_dt {
            let now = chrono::Utc::now();
            let duration = session_utc - now;
            let hours_remaining = duration.num_hours();
            Some(format!("in {} hours", hours_remaining))
        } else {
            Some("Soon".to_string())
        };
        
        action_items.push(ActionItem {
            id: format!("jtb-{}", session_id),
            item_type: "approval".to_string(),
            title: session_name,
            subtitle: Some(format!("{} targets on agenda", target_count)),
            classification: "SECRET".to_string(),
            caveats: Some(vec!["NOFORN".to_string()]),
            priority: if target_count > 5 { "HIGH".to_string() } else { "MEDIUM".to_string() },
            due_time: Some(session_datetime),
            due_time_display,
            assigned_to_current_user: false,
            blockers: None,
            target_id: None,
        });
    }
    
    // 4. DTL entries awaiting approval (MEDIUM priority)
    let dtl_pending = sqlx::query(
        "SELECT dtl.id, dtl.target_id, dtl.current_approver, t.name, t.classification
         FROM dtl_entries dtl
         JOIN targets t ON dtl.target_id = t.id
         WHERE dtl.current_approver IS NOT NULL
           AND dtl.combined_score > 0.7
         ORDER BY dtl.combined_score DESC, dtl.created_at ASC
         LIMIT 10"
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    for row in dtl_pending {
        let dtl_id: String = row.get("id");
        let target_id: String = row.get("target_id");
        let target_name: String = row.get("name");
        let classification: String = row.get("classification");
        let approver: Option<String> = row.get("current_approver");
        
        let assigned = approver.as_ref().map(|a| a == user_id).unwrap_or(false);
        
        action_items.push(ActionItem {
            id: format!("dtl-{}", dtl_id),
            item_type: "approval".to_string(),
            title: format!("DTL Approval: {}", target_name),
            subtitle: approver.map(|a| format!("Awaiting approval from {}", a)),
            classification,
            caveats: Some(vec!["NOFORN".to_string()]),
            priority: "MEDIUM".to_string(),
            due_time: None,
            due_time_display: None,
            assigned_to_current_user: assigned,
            blockers: None,
            target_id: Some(target_id),
        });
    }
    
    // Sort by priority (CRITICAL > HIGH > MEDIUM > LOW) and due time
    action_items.sort_by(|a, b| {
        let priority_order = |p: &str| match p {
            "CRITICAL" => 0,
            "HIGH" => 1,
            "MEDIUM" => 2,
            "LOW" => 3,
            _ => 4,
        };
        
        let priority_cmp = priority_order(&a.priority).cmp(&priority_order(&b.priority));
        if priority_cmp != std::cmp::Ordering::Equal {
            return priority_cmp;
        }
        
        // Then by due time (earlier first)
        match (&a.due_time, &b.due_time) {
            (Some(a_time), Some(b_time)) => a_time.cmp(b_time),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
    });
    
    Ok(Json(serde_json::json!({
        "items": action_items,
        "total": action_items.len(),
        "critical_count": action_items.iter().filter(|i| i.priority == "CRITICAL").count(),
        "high_count": action_items.iter().filter(|i| i.priority == "HIGH").count(),
    })))
}

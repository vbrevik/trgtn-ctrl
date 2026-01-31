// NATO COPD Targeting Cell - Repository Layer

use crate::features::targeting::domain::*;
use sqlx::{Pool, Sqlite, Error as SqlxError, Row};

// ============================================================================
// TARGET REPOSITORY (uses existing targets table)
// ============================================================================

pub struct TargetRepository;

impl TargetRepository {
    pub async fn create(
        pool: &Pool<Sqlite>,
        name: &str,
        description: Option<&str>,
        target_type: &str,
        priority: &str,
        coordinates: &str,
        classification: &str,
    ) -> Result<String, SqlxError> {
        let id = uuid::Uuid::new_v4().to_string();
        
        sqlx::query(
            "INSERT INTO targets (id, name, description, target_type, priority, target_status, coordinates, classification, f3ead_stage, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, 'Nominated', ?, ?, 'FIND', datetime('now'), datetime('now'))"
        )
        .bind(&id)
        .bind(name)
        .bind(description)
        .bind(target_type)
        .bind(priority)
        .bind(coordinates)
        .bind(classification)
        .execute(pool)
        .await?;
        
        Ok(id)
    }
    
    pub async fn list_all(
        pool: &Pool<Sqlite>,
        status: Option<&str>,
        priority: Option<&str>,
        limit: Option<i64>,
        _offset: Option<i64>,
    ) -> Result<Vec<Target>, SqlxError> {
        let limit = limit.unwrap_or(100);
        
        let query = if let (Some(s), Some(p)) = (status, priority) {
            format!("SELECT id, name, description, target_type, priority, target_status as status, coordinates, f3ead_stage 
                    FROM v_targets_ontology WHERE target_status = '{}' AND priority = '{}' LIMIT {}", s, p, limit)
        } else if let Some(s) = status {
            format!("SELECT id, name, description, target_type, priority, target_status as status, coordinates, f3ead_stage 
                    FROM v_targets_ontology WHERE target_status = '{}' LIMIT {}", s, limit)
        } else if let Some(p) = priority {
            format!("SELECT id, name, description, target_type, priority, target_status as status, coordinates, f3ead_stage 
                    FROM v_targets_ontology WHERE priority = '{}' LIMIT {}", p, limit)
        } else {
            format!("SELECT id, name, description, target_type, priority, target_status as status, coordinates, f3ead_stage 
                    FROM v_targets_ontology LIMIT {}", limit)
        };
        
        let rows = sqlx::query(&query).fetch_all(pool).await?;
        
        let mut targets = Vec::new();
        for row in rows {
            targets.push(Target {
                id: row.get("id"),
                name: row.get("name"),
                description: row.get("description"),
                target_type: row.get("target_type"),
                priority: row.get("priority"),
                target_status: row.get("status"),
                coordinates: row.get("coordinates"),
                f3ead_stage: row.get::<Option<String>, _>("f3ead_stage"),
            });
        }
        
        Ok(targets)
    }
    
    pub async fn get_by_id(pool: &Pool<Sqlite>, id: &str) -> Result<Option<Target>, SqlxError> {
        let result = sqlx::query(
            "SELECT id, name, description, target_type, priority, target_status as status, coordinates, f3ead_stage 
             FROM v_targets_ontology WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;
        
        match result {
            Some(row) => Ok(Some(Target {
                id: row.get("id"),
                name: row.get("name"),
                description: row.get("description"),
                target_type: row.get("target_type"),
                priority: row.get("priority"),
                target_status: row.get("status"),
                coordinates: row.get("coordinates"),
                f3ead_stage: row.get::<Option<String>, _>("f3ead_stage"),
            })),
            None => Ok(None)
        }
    }
    
    pub async fn update(pool: &Pool<Sqlite>, id: &str, name: Option<&str>, priority: Option<&str>, target_status: Option<&str>) -> Result<(), SqlxError> {
        // Build update query dynamically
        let mut query = "UPDATE targets SET ".to_string();
        let mut parts = Vec::new();
        
        if let Some(n) = name {
            parts.push(format!("name = '{}'", n.replace("'", "''")));
        }
        if let Some(p) = priority {
            parts.push(format!("priority = '{}'", p.replace("'", "''")));
        }
        if let Some(s) = target_status {
            parts.push(format!("target_status = '{}'", s.replace("'", "''")));
        }
        
        if parts.is_empty() {
            return Ok(()); // No updates
        }
        
        parts.push("updated_at = datetime('now')".to_string());
        query.push_str(&parts.join(", "));
        query.push_str(&format!(" WHERE id = '{}'", id.replace("'", "''")));
        
        sqlx::query(&query).execute(pool).await?;
        Ok(())
    }
    
    pub async fn update_f3ead_stage(pool: &Pool<Sqlite>, id: &str, stage: &str) -> Result<(), SqlxError> {
        sqlx::query("UPDATE targets SET f3ead_stage = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(stage)
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }
    
    pub async fn get_summary(pool: &Pool<Sqlite>) -> Result<TargetingSummary, SqlxError> {
        let total = sqlx::query("SELECT COUNT(*) as count FROM v_targets_ontology")
            .fetch_one(pool).await?.get::<i64, _>(0);
        let active = sqlx::query("SELECT COUNT(*) as count FROM v_targets_ontology WHERE target_status IN ('Nominated', 'Validated', 'Approved')")
            .fetch_one(pool).await?.get::<i64, _>(0);
        let pending = sqlx::query("SELECT COUNT(*) as count FROM v_targets_ontology WHERE target_status = 'Nominated'")
            .fetch_one(pool).await?.get::<i64, _>(0);
        let approved = sqlx::query("SELECT COUNT(*) as count FROM v_targets_ontology WHERE target_status = 'Approved'")
            .fetch_one(pool).await?.get::<i64, _>(0);
        
        Ok(TargetingSummary {
            total_targets: total,
            active_targets: active,
            pending_nominations: pending,
            approved_targets: approved,
        })
    }
}

// ============================================================================
// DTL REPOSITORY
// ============================================================================

pub struct DtlRepository;

impl DtlRepository {
    pub async fn create(pool: &Pool<Sqlite>, target_id: &str, priority: f64, feasibility: f64) -> Result<String, SqlxError> {
        use crate::features::targeting::services::DtlScoring;
        
        let id = uuid::Uuid::new_v4().to_string();
        
        // Note: combined_score is a GENERATED column in SQLite, calculated automatically
        // from (priority_score + feasibility_score) / 2.0
        
        // Get target creation time to calculate aging
        let target_row = sqlx::query("SELECT created_at FROM targets WHERE id = ?")
            .bind(target_id)
            .fetch_optional(pool)
            .await?;
        
        let aging_hours = if let Some(row) = target_row {
            let created_at: String = row.get("created_at");
            DtlScoring::calculate_aging_hours(&created_at)
        } else {
            0
        };
        
        // Don't insert combined_score - it's a generated column
        sqlx::query("INSERT INTO dtl_entries (id, target_id, priority_score, feasibility_score, aging_hours, is_tst) VALUES (?, ?, ?, ?, ?, 0)")
            .bind(&id)
            .bind(target_id)
            .bind(priority)
            .bind(feasibility)
            .bind(aging_hours)
            .execute(pool)
            .await?;
        
        Ok(id)
    }
    
    pub async fn list_all(pool: &Pool<Sqlite>, limit: Option<i64>) -> Result<Vec<DtlEntry>, SqlxError> {
        use crate::features::targeting::services::DtlScoring;
        
        let limit = limit.unwrap_or(100);
        
        let rows = sqlx::query(&format!(
            "SELECT id, target_id, priority_score, feasibility_score, combined_score, aging_hours, 
             is_tst, tst_deadline, approval_chain, current_approver, approval_level, created_at, updated_at 
             FROM dtl_entries 
             ORDER BY combined_score DESC, aging_hours ASC
             LIMIT {}", limit
        ))
        .fetch_all(pool)
        .await?;
        
        let mut entries = Vec::new();
        for row in rows {
            let created_at: String = row.get("created_at");
            
            // Recalculate aging hours (in case target was updated)
            let aging_hours = DtlScoring::calculate_aging_hours(&created_at);
            
            // Recalculate combined score if needed (ensures consistency)
            let priority_score: f64 = row.get("priority_score");
            let feasibility_score: f64 = row.get("feasibility_score");
            let combined_score = DtlScoring::calculate_combined_score(priority_score, feasibility_score);
            
            entries.push(DtlEntry {
                id: row.get("id"),
                target_id: row.get("target_id"),
                priority_score,
                feasibility_score,
                combined_score: Some(combined_score),
                aging_hours,
                is_tst: row.get::<i32, _>("is_tst") == 1,
                tst_deadline: row.get("tst_deadline"),
                approval_chain: row.get("approval_chain"),
                current_approver: row.get("current_approver"),
                approval_level: row.get("approval_level"),
                created_at,
                updated_at: row.get("updated_at"),
            });
        }
        
        Ok(entries)
    }
    
    pub async fn get_active_tsts(pool: &Pool<Sqlite>) -> Result<Vec<DtlEntry>, SqlxError> {
        let rows = sqlx::query(
            "SELECT * FROM dtl_entries WHERE is_tst = 1 AND tst_deadline > datetime('now')"
        )
        .fetch_all(pool)
        .await?;
        
        let mut entries = Vec::new();
        for row in rows {
            entries.push(DtlEntry {
                id: row.get("id"),
                target_id: row.get("target_id"),
                priority_score: row.get("priority_score"),
                feasibility_score: row.get("feasibility_score"),
                combined_score: row.get("combined_score"),
                aging_hours: row.get("aging_hours"),
                is_tst: true,
                tst_deadline: row.get("tst_deadline"),
                approval_chain: row.get("approval_chain"),
                current_approver: row.get("current_approver"),
                approval_level: row.get("approval_level"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }
        
        Ok(entries)
    }
}

// ============================================================================
// ISR REPOSITORY
// ============================================================================

pub struct IsrRepository;

impl IsrRepository {
    pub async fn create(pool: &Pool<Sqlite>, platform: &IsrPlatform) -> Result<String, SqlxError> {
        let id = uuid::Uuid::new_v4().to_string();
        
        sqlx::query("INSERT INTO isr_platforms (id, platform_type, platform_name, sensor_type, status, classification) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(&id)
            .bind(&platform.platform_type)
            .bind(&platform.platform_name)
            .bind(&platform.sensor_type)
            .bind(&platform.status)
            .bind(&platform.classification)
            .execute(pool)
            .await?;
        
        Ok(id)
    }
    
    pub async fn list_all(pool: &Pool<Sqlite>, _status: Option<&str>) -> Result<Vec<IsrPlatform>, SqlxError> {
        let rows = sqlx::query("SELECT * FROM isr_platforms")
            .fetch_all(pool)
            .await?;
        
        let mut platforms = Vec::new();
        for row in rows {
            platforms.push(IsrPlatform {
                id: row.get("id"),
                platform_type: row.get("platform_type"),
                platform_name: row.get("platform_name"),
                callsign: row.get("callsign"),
                current_position: row.get("current_position"),
                sensor_type: row.get("sensor_type"),
                sensor_range_km: row.get("sensor_range_km"),
                coverage_area: row.get("coverage_area"),
                status: row.get("status"),
                loiter_time_remaining: row.get("loiter_time_remaining"),
                fuel_remaining_percent: row.get("fuel_remaining_percent"),
                current_task: row.get("current_task"),
                tasking_priority: row.get("tasking_priority"),
                tasked_targets: row.get("tasked_targets"),
                classification: row.get("classification"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }
        
        Ok(platforms)
    }
    
    pub async fn get_pattern_of_life(pool: &Pool<Sqlite>) -> Result<Vec<IntelligenceReport>, SqlxError> {
        // Get pattern-of-life intel reports collected by ISR platforms
        let rows = sqlx::query("SELECT * FROM intelligence_reports WHERE pattern_of_life_indicator = 1 ORDER BY collection_time DESC LIMIT 50")
            .fetch_all(pool)
            .await?;
        
        let mut reports = Vec::new();
        for row in rows {
            reports.push(IntelligenceReport {
                id: row.get("id"),
                target_id: row.get("target_id"),
                int_type: row.get("int_type"),
                report_title: row.get("report_title"),
                report_content: row.get("report_content"),
                report_summary: row.get("report_summary"),
                confidence_level: row.get("confidence_level"),
                source_reliability: row.get("source_reliability"),
                collection_time: row.get("collection_time"),
                reporting_time: row.get("reporting_time"),
                fusion_score: row.get("fusion_score"),
                pattern_of_life_indicator: row.get::<i32, _>("pattern_of_life_indicator") == 1,
                classification: row.get("classification"),
                collected_by: row.get("collected_by"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }
        
        Ok(reports)
    }
}

// ============================================================================
// INTELLIGENCE REPOSITORY
// ============================================================================

pub struct IntelRepository;

impl IntelRepository {
    pub async fn create(pool: &Pool<Sqlite>, report: &IntelligenceReport, user_id: &str) -> Result<String, SqlxError> {
        let id = uuid::Uuid::new_v4().to_string();
        
        sqlx::query("INSERT INTO intelligence_reports (id, target_id, int_type, report_title, report_content, confidence_level, source_reliability, collection_time, pattern_of_life_indicator, classification, collected_by) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(&id)
            .bind(&report.target_id)
            .bind(&report.int_type)
            .bind(&report.report_title)
            .bind(&report.report_content)
            .bind(report.confidence_level)
            .bind(&report.source_reliability)
            .bind(&report.collection_time)
            .bind(if report.pattern_of_life_indicator { 1 } else { 0 })
            .bind(&report.classification)
            .bind(user_id)
            .execute(pool)
            .await?;
        
        Ok(id)
    }
    
    pub async fn get_by_target_id(pool: &Pool<Sqlite>, target_id: &str) -> Result<Vec<IntelligenceReport>, SqlxError> {
        let rows = sqlx::query("SELECT * FROM intelligence_reports WHERE target_id = ? ORDER BY collection_time DESC")
            .bind(target_id)
            .fetch_all(pool)
            .await?;
        
        let mut reports = Vec::new();
        for row in rows {
            reports.push(IntelligenceReport {
                id: row.get("id"),
                target_id: row.get("target_id"),
                int_type: row.get("int_type"),
                report_title: row.get("report_title"),
                report_content: row.get("report_content"),
                report_summary: row.get("report_summary"),
                confidence_level: row.get("confidence_level"),
                source_reliability: row.get("source_reliability"),
                collection_time: row.get("collection_time"),
                reporting_time: row.get("reporting_time"),
                fusion_score: row.get("fusion_score"),
                pattern_of_life_indicator: row.get::<i32, _>("pattern_of_life_indicator") == 1,
                classification: row.get("classification"),
                collected_by: row.get("collected_by"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }
        
        Ok(reports)
    }
    
    pub async fn get_pattern_of_life(pool: &Pool<Sqlite>) -> Result<Vec<IntelligenceReport>, SqlxError> {
        let rows = sqlx::query("SELECT * FROM intelligence_reports WHERE pattern_of_life_indicator = 1 ORDER BY collection_time DESC LIMIT 50")
            .fetch_all(pool)
            .await?;
        
        let mut reports = Vec::new();
        for row in rows {
            reports.push(IntelligenceReport {
                id: row.get("id"),
                target_id: row.get("target_id"),
                int_type: row.get("int_type"),
                report_title: row.get("report_title"),
                report_content: row.get("report_content"),
                report_summary: row.get("report_summary"),
                confidence_level: row.get("confidence_level"),
                source_reliability: row.get("source_reliability"),
                collection_time: row.get("collection_time"),
                reporting_time: row.get("reporting_time"),
                fusion_score: row.get("fusion_score"),
                pattern_of_life_indicator: row.get::<i32, _>("pattern_of_life_indicator") == 1,
                classification: row.get("classification"),
                collected_by: row.get("collected_by"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }
        
        Ok(reports)
    }
}

// ============================================================================
// STRIKE PLATFORM REPOSITORY
// ============================================================================

pub struct StrikePlatformRepository;

impl StrikePlatformRepository {
    pub async fn create(pool: &Pool<Sqlite>, platform: &StrikePlatform) -> Result<String, SqlxError> {
        let id = uuid::Uuid::new_v4().to_string();
        
        sqlx::query("INSERT INTO strike_platforms (id, platform_type, platform_name, sorties_available, platform_status, classification) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(&id)
            .bind(&platform.platform_type)
            .bind(&platform.platform_name)
            .bind(platform.sorties_available)
            .bind(&platform.platform_status)
            .bind(&platform.classification)
            .execute(pool)
            .await?;
        
        Ok(id)
    }
    
    pub async fn list_all(pool: &Pool<Sqlite>, _status: Option<&str>) -> Result<Vec<StrikePlatform>, SqlxError> {
        let rows = sqlx::query("SELECT * FROM strike_platforms")
            .fetch_all(pool)
            .await?;
        
        let mut platforms = Vec::new();
        for row in rows {
            platforms.push(StrikePlatform {
                id: row.get("id"),
                platform_type: row.get("platform_type"),
                platform_name: row.get("platform_name"),
                callsign: row.get("callsign"),
                unit: row.get("unit"),
                munitions_available: row.get("munitions_available"),
                sorties_available: row.get("sorties_available"),
                platform_status: row.get("platform_status"),
                classification: row.get("classification"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }
        
        Ok(platforms)
    }
}

// ============================================================================
// RISK ASSESSMENT REPOSITORY
// ============================================================================

pub struct RiskRepository;

impl RiskRepository {
    pub async fn create(pool: &Pool<Sqlite>, target_id: &str, fratricide: &str, political: &str, legal: &str, classification: &str, user_id: &str) -> Result<String, SqlxError> {
        let id = uuid::Uuid::new_v4().to_string();
        
        sqlx::query("INSERT INTO risk_assessments (id, target_id, fratricide_risk, political_sensitivity, legal_review_status, classification, assessed_by) VALUES (?, ?, ?, ?, ?, ?, ?)")
            .bind(&id)
            .bind(target_id)
            .bind(fratricide)
            .bind(political)
            .bind(legal)
            .bind(classification)
            .bind(user_id)
            .execute(pool)
            .await?;
        
        Ok(id)
    }
    
    pub async fn get_by_target_id(pool: &Pool<Sqlite>, target_id: &str) -> Result<Option<RiskAssessment>, SqlxError> {
        let result = sqlx::query("SELECT * FROM risk_assessments WHERE target_id = ? ORDER BY created_at DESC LIMIT 1")
            .bind(target_id)
            .fetch_optional(pool)
            .await?;
        
        match result {
            Some(row) => Ok(Some(RiskAssessment {
                id: row.get("id"),
                target_id: row.get("target_id"),
                fratricide_risk: row.get("fratricide_risk"),
                friendly_forces_distance_km: row.get("friendly_forces_distance_km"),
                political_sensitivity: row.get("political_sensitivity"),
                sensitive_sites_nearby: row.get("sensitive_sites_nearby"),
                proportionality_assessment: row.get("proportionality_assessment"),
                legal_review_status: row.get("legal_review_status"),
                legal_reviewer: row.get("legal_reviewer"),
                legal_review_date: row.get("legal_review_date"),
                overall_risk_score: row.get("overall_risk_score"),
                classification: row.get("classification"),
                assessed_by: row.get("assessed_by"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })),
            None => Ok(None)
        }
    }
    
    pub async fn get_high_risk(pool: &Pool<Sqlite>) -> Result<Vec<RiskAssessment>, SqlxError> {
        let rows = sqlx::query(
            "SELECT * FROM risk_assessments 
             WHERE fratricide_risk IN ('HIGH', 'CRITICAL') 
                OR political_sensitivity IN ('HIGH', 'CRITICAL') 
                OR overall_risk_score >= 0.7
             ORDER BY overall_risk_score DESC LIMIT 20"
        )
        .fetch_all(pool)
        .await?;
        
        let mut assessments = Vec::new();
        for row in rows {
            assessments.push(RiskAssessment {
                id: row.get("id"),
                target_id: row.get("target_id"),
                fratricide_risk: row.get("fratricide_risk"),
                friendly_forces_distance_km: row.get("friendly_forces_distance_km"),
                political_sensitivity: row.get("political_sensitivity"),
                sensitive_sites_nearby: row.get("sensitive_sites_nearby"),
                proportionality_assessment: row.get("proportionality_assessment"),
                legal_review_status: row.get("legal_review_status"),
                legal_reviewer: row.get("legal_reviewer"),
                legal_review_date: row.get("legal_review_date"),
                overall_risk_score: row.get("overall_risk_score"),
                classification: row.get("classification"),
                assessed_by: row.get("assessed_by"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }
        
        Ok(assessments)
    }
}

// ============================================================================
// ASSUMPTION REPOSITORY
// ============================================================================

pub struct AssumptionChallengeRepository;

impl AssumptionChallengeRepository {
    pub async fn create(pool: &Pool<Sqlite>, text: &str, confidence: i32, status: &str) -> Result<String, SqlxError> {
        let id = uuid::Uuid::new_v4().to_string();
        
        sqlx::query("INSERT INTO assumption_challenges (id, assumption_text, confidence_level, validation_status) VALUES (?, ?, ?, ?)")
            .bind(&id)
            .bind(text)
            .bind(confidence)
            .bind(status)
            .execute(pool)
            .await?;
        
        Ok(id)
    }
    
    pub async fn list_all(pool: &Pool<Sqlite>, status: Option<&str>) -> Result<Vec<AssumptionChallenge>, SqlxError> {
        let query = if let Some(s) = status {
            format!("SELECT * FROM assumption_challenges WHERE validation_status = '{}' ORDER BY created_at DESC LIMIT 100", s)
        } else {
            "SELECT * FROM assumption_challenges ORDER BY created_at DESC LIMIT 100".to_string()
        };
        
        let rows = sqlx::query(&query).fetch_all(pool).await?;
        
        let mut assumptions = Vec::new();
        for row in rows {
            assumptions.push(AssumptionChallenge {
                id: row.get("id"),
                assumption_text: row.get("assumption_text"),
                assumption_category: row.get("assumption_category"),
                confidence_level: row.get("confidence_level"),
                validation_status: row.get("validation_status"),
                alternative_hypothesis: row.get("alternative_hypothesis"),
                bias_type: row.get("bias_type"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }
        
        Ok(assumptions)
    }
}

// ============================================================================
// DECISION LOG REPOSITORY
// ============================================================================

pub struct DecisionLogRepository;

impl DecisionLogRepository {
    pub async fn create(pool: &Pool<Sqlite>, decision_type: &str, text: &str, rationale: &str, role: &str, authority: &str, classification: &str, user_id: &str) -> Result<String, SqlxError> {
        let id = uuid::Uuid::new_v4().to_string();
        
        sqlx::query("INSERT INTO decision_log (id, decision_type, decision_text, decision_rationale, decision_maker, decision_maker_role, authority_level, classification) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(&id)
            .bind(decision_type)
            .bind(text)
            .bind(rationale)
            .bind(user_id)
            .bind(role)
            .bind(authority)
            .bind(classification)
            .execute(pool)
            .await?;
        
        Ok(id)
    }
    
    pub async fn list_recent(pool: &Pool<Sqlite>, limit: Option<i64>) -> Result<Vec<DecisionLogEntry>, SqlxError> {
        let limit = limit.unwrap_or(50);
        
        let rows = sqlx::query(&format!("SELECT * FROM decision_log ORDER BY decision_time DESC LIMIT {}", limit))
            .fetch_all(pool)
            .await?;
        
        let mut decisions = Vec::new();
        for row in rows {
            decisions.push(DecisionLogEntry {
                id: row.get("id"),
                decision_type: row.get("decision_type"),
                decision_text: row.get("decision_text"),
                decision_rationale: row.get("decision_rationale"),
                decision_maker: row.get("decision_maker"),
                decision_maker_role: row.get("decision_maker_role"),
                authority_level: row.get("authority_level"),
                decision_time: row.get("decision_time"),
                classification: row.get("classification"),
                created_at: row.get("created_at"),
            });
        }
        
        Ok(decisions)
    }
}

// ============================================================================
// SHIFT HANDOVER REPOSITORY
// ============================================================================

pub struct ShiftHandoverRepository;

impl ShiftHandoverRepository {
    pub async fn create(pool: &Pool<Sqlite>, shift_date: &str, shift_type: &str, incoming: &str, summary: &str, pending: &str, classification: &str, user_id: &str) -> Result<String, SqlxError> {
        let id = uuid::Uuid::new_v4().to_string();
        
        sqlx::query("INSERT INTO shift_handovers (id, shift_date, shift_type, shift_start, shift_end, outgoing_watch_officer, incoming_watch_officer, active_targets_summary, pending_decisions, handover_time, classification) VALUES (?, ?, ?, datetime('now'), datetime('now', '+8 hours'), ?, ?, ?, ?, datetime('now'), ?)")
            .bind(&id)
            .bind(shift_date)
            .bind(shift_type)
            .bind(user_id)
            .bind(incoming)
            .bind(summary)
            .bind(pending)
            .bind(classification)
            .execute(pool)
            .await?;
        
        Ok(id)
    }
    
    pub async fn get_recent(pool: &Pool<Sqlite>, limit: Option<i64>) -> Result<Vec<ShiftHandover>, SqlxError> {
        let limit = limit.unwrap_or(10);
        
        let rows = sqlx::query(&format!("SELECT * FROM shift_handovers ORDER BY handover_time DESC LIMIT {}", limit))
            .fetch_all(pool)
            .await?;
        
        let mut handovers = Vec::new();
        for row in rows {
            handovers.push(ShiftHandover {
                id: row.get("id"),
                shift_date: row.get("shift_date"),
                shift_type: row.get("shift_type"),
                shift_start: row.get("shift_start"),
                shift_end: row.get("shift_end"),
                outgoing_watch_officer: row.get("outgoing_watch_officer"),
                incoming_watch_officer: row.get("incoming_watch_officer"),
                active_targets_summary: row.get("active_targets_summary"),
                pending_decisions: row.get("pending_decisions"),
                handover_time: row.get("handover_time"),
                classification: row.get("classification"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }
        
        Ok(handovers)
    }
}

// ============================================================================
// ANNOTATION REPOSITORY
// ============================================================================

pub struct AnnotationRepository;

impl AnnotationRepository {
    pub async fn create(pool: &Pool<Sqlite>, target_id: Option<&str>, text: &str, annotation_type: &str, is_critical: bool, classification: &str, user_id: &str) -> Result<String, SqlxError> {
        let id = uuid::Uuid::new_v4().to_string();
        
        sqlx::query("INSERT INTO targeting_annotations (id, target_id, user_id, annotation_text, annotation_type, is_critical, classification) VALUES (?, ?, ?, ?, ?, ?, ?)")
            .bind(&id)
            .bind(target_id)
            .bind(user_id)
            .bind(text)
            .bind(annotation_type)
            .bind(if is_critical { 1 } else { 0 })
            .bind(classification)
            .execute(pool)
            .await?;
        
        Ok(id)
    }
    
    pub async fn get_by_target_id(pool: &Pool<Sqlite>, target_id: &str) -> Result<Vec<TargetingAnnotation>, SqlxError> {
        let rows = sqlx::query("SELECT * FROM targeting_annotations WHERE target_id = ? ORDER BY created_at DESC")
            .bind(target_id)
            .fetch_all(pool)
            .await?;
        
        let mut annotations = Vec::new();
        for row in rows {
            annotations.push(TargetingAnnotation {
                id: row.get("id"),
                target_id: row.get("target_id"),
                user_id: row.get("user_id"),
                annotation_text: row.get("annotation_text"),
                annotation_type: row.get("annotation_type"),
                is_critical: row.get::<i32, _>("is_critical") == 1,
                classification: row.get("classification"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }
        
        Ok(annotations)
    }
}

// ============================================================================
// BDA REPOSITORY (uses existing bda_reports table)
// ============================================================================

pub struct BdaRepository;

impl BdaRepository {
    pub async fn get_by_id(pool: &Pool<Sqlite>, id: &str) -> Result<Option<BdaAssessment>, SqlxError> {
        let result = sqlx::query(
            "SELECT id, target_id, physical_damage, functional_damage, recommendation, confidence_level as confidence 
             FROM v_bda_reports_ontology WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;
        
        match result {
            Some(row) => Ok(Some(BdaAssessment {
                id: row.get("id"),
                target_id: row.get("target_id"),
                physical_damage: row.get("physical_damage"),
                functional_damage: row.get("functional_damage"),
                recommendation: row.get("recommendation"),
                confidence: row.get("confidence"),
            })),
            None => Ok(None)
        }
    }
}

// ============================================================================
// JTB REPOSITORY
// ============================================================================

pub struct JtbRepository;

impl JtbRepository {
    pub async fn create_session(
        pool: &Pool<Sqlite>,
        session_name: &str,
        session_date: &str,
        session_time: &str,
        session_datetime: &str,
        chair: &str,
        chair_rank: Option<&str>,
        required_attendees: Option<&str>,
        classification: &str,
        caveats: Option<&str>,
        created_by: &str,
    ) -> Result<String, SqlxError> {
        let id = format!("jtb_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
        
        sqlx::query(
            "INSERT INTO jtb_sessions 
             (id, session_name, session_date, session_time, session_datetime, chair, chair_rank, 
              required_attendees, classification, caveats, created_by)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(session_name)
        .bind(session_date)
        .bind(session_time)
        .bind(session_datetime)
        .bind(chair)
        .bind(chair_rank)
        .bind(required_attendees)
        .bind(classification)
        .bind(caveats)
        .bind(created_by)
        .execute(pool)
        .await?;
        
        Ok(id)
    }
    
    pub async fn list_sessions(
        pool: &Pool<Sqlite>,
        status: Option<&str>,
        limit: Option<i64>,
    ) -> Result<Vec<JtbSession>, SqlxError> {
        let limit = limit.unwrap_or(50);
        
        let query = if let Some(s) = status {
            format!("SELECT * FROM jtb_sessions WHERE status = '{}' ORDER BY session_datetime DESC LIMIT {}", s, limit)
        } else {
            format!("SELECT * FROM jtb_sessions ORDER BY session_datetime DESC LIMIT {}", limit)
        };
        
        let rows = sqlx::query(&query).fetch_all(pool).await?;
        
        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(JtbSession {
                id: row.get("id"),
                session_name: row.get("session_name"),
                session_date: row.get("session_date"),
                session_time: row.get("session_time"),
                session_datetime: row.get("session_datetime"),
                chair: row.get("chair"),
                chair_rank: row.get("chair_rank"),
                status: row.get("status"),
                required_attendees: row.get("required_attendees"),
                actual_attendees: row.get("actual_attendees"),
                quorum_verified: row.get::<i32, _>("quorum_verified") == 1,
                protocol_notes: row.get("protocol_notes"),
                session_minutes: row.get("session_minutes"),
                classification: row.get("classification"),
                caveats: row.get("caveats"),
                created_by: row.get("created_by"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                completed_at: row.get("completed_at"),
            });
        }
        
        Ok(sessions)
    }
    
    pub async fn get_session_by_id(
        pool: &Pool<Sqlite>,
        session_id: &str,
    ) -> Result<Option<JtbSession>, SqlxError> {
        let result = sqlx::query("SELECT * FROM jtb_sessions WHERE id = ?")
            .bind(session_id)
            .fetch_optional(pool)
            .await?;
        
        match result {
            Some(row) => Ok(Some(JtbSession {
                id: row.get("id"),
                session_name: row.get("session_name"),
                session_date: row.get("session_date"),
                session_time: row.get("session_time"),
                session_datetime: row.get("session_datetime"),
                chair: row.get("chair"),
                chair_rank: row.get("chair_rank"),
                status: row.get("status"),
                required_attendees: row.get("required_attendees"),
                actual_attendees: row.get("actual_attendees"),
                quorum_verified: row.get::<i32, _>("quorum_verified") == 1,
                protocol_notes: row.get("protocol_notes"),
                session_minutes: row.get("session_minutes"),
                classification: row.get("classification"),
                caveats: row.get("caveats"),
                created_by: row.get("created_by"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                completed_at: row.get("completed_at"),
            })),
            None => Ok(None)
        }
    }
    
    pub async fn add_target_to_session(
        pool: &Pool<Sqlite>,
        session_id: &str,
        target_id: &str,
        presentation_order: i32,
    ) -> Result<String, SqlxError> {
        let id = format!("jtb_tgt_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
        
        sqlx::query(
            "INSERT INTO jtb_targets (id, session_id, target_id, presentation_order)
             VALUES (?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(session_id)
        .bind(target_id)
        .bind(presentation_order)
        .execute(pool)
        .await?;
        
        Ok(id)
    }
    
    pub async fn get_targets_for_session(
        pool: &Pool<Sqlite>,
        session_id: &str,
    ) -> Result<Vec<JtbTarget>, SqlxError> {
        let rows = sqlx::query(
            "SELECT * FROM jtb_targets WHERE session_id = ? ORDER BY presentation_order ASC"
        )
        .bind(session_id)
        .fetch_all(pool)
        .await?;
        
        let mut targets = Vec::new();
        for row in rows {
            targets.push(JtbTarget {
                id: row.get("id"),
                session_id: row.get("session_id"),
                target_id: row.get("target_id"),
                presentation_order: row.get("presentation_order"),
                decision: row.get("decision"),
                decision_rationale: row.get("decision_rationale"),
                decided_by: row.get("decided_by"),
                decided_at: row.get("decided_at"),
                votes_for: row.get("votes_for"),
                votes_against: row.get("votes_against"),
                votes_abstain: row.get("votes_abstain"),
                approval_conditions: row.get("approval_conditions"),
                mitigation_requirements: row.get("mitigation_requirements"),
                added_to_session_at: row.get("added_to_session_at"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }
        
        Ok(targets)
    }
    
    pub async fn record_decision(
        pool: &Pool<Sqlite>,
        jtb_target_id: &str,
        decision: &str,
        decision_rationale: &str,
        decided_by: &str,
        votes_for: Option<i32>,
        votes_against: Option<i32>,
        votes_abstain: Option<i32>,
        approval_conditions: Option<&str>,
        mitigation_requirements: Option<&str>,
    ) -> Result<(), SqlxError> {
        sqlx::query(
            "UPDATE jtb_targets 
             SET decision = ?, decision_rationale = ?, decided_by = ?, decided_at = datetime('now'),
                 votes_for = ?, votes_against = ?, votes_abstain = ?,
                 approval_conditions = ?, mitigation_requirements = ?
             WHERE id = ?"
        )
        .bind(decision)
        .bind(decision_rationale)
        .bind(decided_by)
        .bind(votes_for.unwrap_or(0))
        .bind(votes_against.unwrap_or(0))
        .bind(votes_abstain.unwrap_or(0))
        .bind(approval_conditions)
        .bind(mitigation_requirements)
        .bind(jtb_target_id)
        .execute(pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn update_session_status(
        pool: &Pool<Sqlite>,
        session_id: &str,
        status: &str,
    ) -> Result<(), SqlxError> {
        let completed_at = if status == "COMPLETED" {
            Some("datetime('now')")
        } else {
            None
        };
        
        if let Some(completed) = completed_at {
            sqlx::query(&format!(
                "UPDATE jtb_sessions SET status = ?, completed_at = {} WHERE id = ?",
                completed
            ))
            .bind(status)
            .bind(session_id)
            .execute(pool)
            .await?;
        } else {
            sqlx::query("UPDATE jtb_sessions SET status = ? WHERE id = ?")
                .bind(status)
                .bind(session_id)
                .execute(pool)
                .await?;
        }
        
        Ok(())
    }
}

// Note: BDA uses existing bda_reports table - handlers will query directly

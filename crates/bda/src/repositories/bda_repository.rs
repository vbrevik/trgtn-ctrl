// BDA Repository
// Purpose: Database access layer for BDA reports

use crate::features::bda::domain::{
    BdaReport, CreateBdaReportRequest, UpdateBdaReportRequest,
    BdaStatistics, BdaStatusCounts, BdaRecommendationCounts, BdaPhysicalDamageCounts,
    PhysicalDamage, FunctionalDamage, AssessmentType, BdaStatus, Recommendation,
    ReAttackRecommendation,
};
use sqlx::{Pool, Sqlite, Row};
use uuid::Uuid;

pub struct BdaRepository {
    pool: Pool<Sqlite>,
}

impl BdaRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
    
    /// Create a new BDA report
    pub async fn create(&self, analyst_id: &str, req: CreateBdaReportRequest) -> Result<BdaReport, sqlx::Error> {
        req.validate().map_err(|e| sqlx::Error::Decode(e.into()))?;
        
        let id = Uuid::new_v4().to_string();
        let created_at = chrono::Utc::now();
        
        let physical_damage_str = match req.physical_damage {
            PhysicalDamage::ND => "ND",
            PhysicalDamage::SD => "SD",
            PhysicalDamage::MD => "MD",
            PhysicalDamage::SVD => "SVD",
            PhysicalDamage::D => "D",
        };
        
        let functional_damage_str = match req.functional_damage {
            FunctionalDamage::FMC => "FMC",
            FunctionalDamage::PMC => "PMC",
            FunctionalDamage::NMC => "NMC",
        };
        
        let assessment_type_str = match req.assessment_type {
            AssessmentType::Initial => "initial",
            AssessmentType::Interim => "interim",
            AssessmentType::Final => "final",
        };
        
        let recommendation_str = match req.recommendation {
            Recommendation::EffectAchieved => "effect_achieved",
            Recommendation::Monitor => "monitor",
            Recommendation::ReAttack => "re_attack",
            Recommendation::ReWeaponeer => "re_weaponeer",
        };
        
        let effect_level_str = req.effect_level.as_ref().map(|e| match e {
            crate::features::bda::domain::EffectLevel::FirstOrder => "first_order",
            crate::features::bda::domain::EffectLevel::SecondOrder => "second_order",
            crate::features::bda::domain::EffectLevel::ThirdOrder => "third_order",
        });
        
        let civcas_str = req.civcas_credibility.as_ref().map(|c| match c {
            crate::features::bda::domain::CivcasCredibility::NoCredibility => "no_credibility",
            crate::features::bda::domain::CivcasCredibility::Possible => "possible",
            crate::features::bda::domain::CivcasCredibility::Credible => "credible",
            crate::features::bda::domain::CivcasCredibility::Confirmed => "confirmed",
        });
        
        let weapon_perf_str = req.weapon_performance_vs_predicted.as_ref().map(|w| match w {
            crate::features::bda::domain::WeaponPerformance::Exceeded => "exceeded",
            crate::features::bda::domain::WeaponPerformance::Met => "met",
            crate::features::bda::domain::WeaponPerformance::Below => "below",
            crate::features::bda::domain::WeaponPerformance::Failed => "failed",
        });
        
        sqlx::query(
            r#"
            INSERT INTO bda_reports (
                id, target_id, strike_id, assessment_date, analyst_id, assessment_type,
                physical_damage, physical_damage_percentage, damage_description,
                functional_damage, estimated_repair_time_hours, pre_strike_capability_baseline,
                desired_effect, achieved_effect, effect_level, unintended_effects,
                confidence_level, limiting_factors,
                recommendation, re_attack_priority, re_attack_rationale, alternative_munitions,
                collateral_damage_detected, civcas_credibility, civilian_casualties_estimate,
                protected_structures_damaged, cde_vs_actual_comparison,
                weapon_performance_vs_predicted, munition_reliability,
                circular_error_probable_meters, penetration_depth_meters,
                status, classification_level, handling_caveats, notes,
                created_at, updated_at
            )
            VALUES (
                $1, $2, $3, $4, $5, $6,
                $7, $8, $9,
                $10, $11, $12,
                $13, $14, $15, $16,
                $17, $18,
                $19, $20, $21, $22,
                $23, $24, $25,
                $26, $27,
                $28, $29,
                $30, $31,
                'draft', $32, $33, $34,
                $35, $35
            )
            "#
        )
        .bind(&id)
        .bind(&req.target_id)
        .bind(&req.strike_id)
        .bind(&created_at)
        .bind(analyst_id)
        .bind(assessment_type_str)
        .bind(physical_damage_str)
        .bind(req.physical_damage_percentage)
        .bind(&req.damage_description)
        .bind(functional_damage_str)
        .bind(req.estimated_repair_time_hours)
        .bind(&req.pre_strike_capability_baseline)
        .bind(&req.desired_effect)
        .bind(&req.achieved_effect)
        .bind(effect_level_str)
        .bind(&req.unintended_effects)
        .bind(req.confidence_level)
        .bind(&req.limiting_factors)
        .bind(recommendation_str)
        .bind(req.re_attack_priority)
        .bind(&req.re_attack_rationale)
        .bind(&req.alternative_munitions)
        .bind(req.collateral_damage_detected)
        .bind(civcas_str)
        .bind(req.civilian_casualties_estimate)
        .bind(&req.protected_structures_damaged)
        .bind(&req.cde_vs_actual_comparison)
        .bind(weapon_perf_str)
        .bind(&req.munition_reliability)
        .bind(req.circular_error_probable_meters)
        .bind(req.penetration_depth_meters)
        .bind(&req.classification_level)
        .bind(&req.handling_caveats)
        .bind(&req.notes)
        .bind(&created_at)
        .execute(&self.pool)
        .await?;
        
        // Fetch the created report
        self.get_by_id(&id).await?
            .ok_or(sqlx::Error::RowNotFound)
    }
    
    /// Get all BDA reports with optional filtering
    pub async fn get_all(&self, status_filter: Option<&str>, target_id_filter: Option<&str>) -> Result<Vec<BdaReport>, sqlx::Error> {
        let mut query = "SELECT * FROM bda_reports WHERE 1=1".to_string();
        
        if status_filter.is_some() {
            query.push_str(" AND status = ?");
        }
        if target_id_filter.is_some() {
            query.push_str(" AND target_id = ?");
        }
        query.push_str(" ORDER BY assessment_date DESC");
        
        let mut q = sqlx::query(&query);
        
        if let Some(status) = status_filter {
            q = q.bind(status);
        }
        if let Some(target_id) = target_id_filter {
            q = q.bind(target_id);
        }
        
        let rows = q.fetch_all(&self.pool).await?;
        
        // Manual mapping since we can't use query_as with dynamic queries
        let mut reports = Vec::new();
        for row in rows {
            reports.push(self.row_to_bda_report(row)?);
        }
        
        Ok(reports)
    }
    
    /// Get BDA report by ID
    pub async fn get_by_id(&self, id: &str) -> Result<Option<BdaReport>, sqlx::Error> {
        let row = sqlx::query("SELECT * FROM bda_reports WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        
        match row {
            Some(r) => Ok(Some(self.row_to_bda_report(r)?)),
            None => Ok(None),
        }
    }
    
    /// Get BDA reports by target ID
    pub async fn get_by_target(&self, target_id: &str) -> Result<Vec<BdaReport>, sqlx::Error> {
        let rows = sqlx::query("SELECT * FROM bda_reports WHERE target_id = $1 ORDER BY assessment_date DESC")
            .bind(target_id)
            .fetch_all(&self.pool)
            .await?;
        
        let mut reports = Vec::new();
        for row in rows {
            reports.push(self.row_to_bda_report(row)?);
        }
        
        Ok(reports)
    }
    
    /// Get assessment queue (draft, submitted, reviewed)
    pub async fn get_assessment_queue(&self) -> Result<Vec<BdaReport>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT * FROM bda_reports WHERE status IN ('draft', 'submitted', 'reviewed') ORDER BY assessment_date DESC"
        )
        .fetch_all(&self.pool)
        .await?;
        
        let mut reports = Vec::new();
        for row in rows {
            reports.push(self.row_to_bda_report(row)?);
        }
        
        Ok(reports)
    }

    /// Get re-attack recommendations (from view)
    pub async fn get_re_attack_recommendations(&self) -> Result<Vec<ReAttackRecommendation>, sqlx::Error> {
        let rows = sqlx::query("SELECT * FROM v_bda_reattack_targets")
            .fetch_all(&self.pool)
            .await?;
            
        let mut results = Vec::new();
        for row in rows {
            results.push(ReAttackRecommendation {
                bda_report_id: row.get("bda_report_id"),
                target_id: row.get("target_id"),
                recommendation: self.parse_recommendation(row.get("recommendation")),
                re_attack_priority: row.get("re_attack_priority"),
                re_attack_rationale: row.get("re_attack_rationale"),
                physical_damage: row.get::<Option<String>, _>("physical_damage").map(|s| self.parse_physical_damage(s)),
                functional_damage: row.get::<Option<String>, _>("functional_damage").map(|s| self.parse_functional_damage(s)),
                confidence_level: row.get::<f64, _>("confidence_level") as f32,
                assessment_date: row.get::<chrono::DateTime<chrono::Utc>, _>("assessment_date").to_rfc3339(),
            });
        }
        Ok(results)
    }
    
    /// Update BDA report
    pub async fn update(&self, id: &str, req: UpdateBdaReportRequest) -> Result<BdaReport, sqlx::Error> {
        let updated_at = chrono::Utc::now();
        
        // Verify report exists
        let _existing = self.get_by_id(id).await?
            .ok_or(sqlx::Error::RowNotFound)?;
        
        // Build update query - only update provided fields
        let physical_damage_str = req.physical_damage.as_ref().map(|p| match p {
            PhysicalDamage::ND => "ND",
            PhysicalDamage::SD => "SD",
            PhysicalDamage::MD => "MD",
            PhysicalDamage::SVD => "SVD",
            PhysicalDamage::D => "D",
        });
        
        let functional_damage_str = req.functional_damage.as_ref().map(|f| match f {
            FunctionalDamage::FMC => "FMC",
            FunctionalDamage::PMC => "PMC",
            FunctionalDamage::NMC => "NMC",
        });
        
        let recommendation_str = req.recommendation.as_ref().map(|r| match r {
            Recommendation::EffectAchieved => "effect_achieved",
            Recommendation::Monitor => "monitor",
            Recommendation::ReAttack => "re_attack",
            Recommendation::ReWeaponeer => "re_weaponeer",
        });
        
        sqlx::query(
            r#"
            UPDATE bda_reports
            SET physical_damage = COALESCE($1, physical_damage),
                physical_damage_percentage = COALESCE($2, physical_damage_percentage),
                damage_description = COALESCE($3, damage_description),
                functional_damage = COALESCE($4, functional_damage),
                estimated_repair_time_hours = COALESCE($5, estimated_repair_time_hours),
                confidence_level = COALESCE($6, confidence_level),
                recommendation = COALESCE($7, recommendation),
                re_attack_priority = COALESCE($8, re_attack_priority),
                re_attack_rationale = COALESCE($9, re_attack_rationale),
                notes = COALESCE($10, notes),
                updated_at = $11
            WHERE id = $12
            "#
        )
        .bind(physical_damage_str)
        .bind(req.physical_damage_percentage)
        .bind(&req.damage_description)
        .bind(functional_damage_str)
        .bind(req.estimated_repair_time_hours)
        .bind(req.confidence_level)
        .bind(recommendation_str)
        .bind(req.re_attack_priority)
        .bind(&req.re_attack_rationale)
        .bind(&req.notes)
        .bind(&updated_at)
        .bind(id)
        .execute(&self.pool)
        .await?;
        
        // Fetch updated report
        self.get_by_id(id).await?
            .ok_or(sqlx::Error::RowNotFound)
    }
    
    /// Submit report for review
    pub async fn submit(&self, id: &str) -> Result<BdaReport, sqlx::Error> {
        let submitted_at = chrono::Utc::now();
        
        sqlx::query(
            "UPDATE bda_reports SET status = 'submitted', submitted_at = $1, updated_at = $1 WHERE id = $2"
        )
        .bind(&submitted_at)
        .bind(id)
        .execute(&self.pool)
        .await?;
        
        self.get_by_id(id).await?
            .ok_or(sqlx::Error::RowNotFound)
    }
    
    /// Approve BDA report
    pub async fn approve(&self, id: &str, approver_id: &str, comments: Option<String>) -> Result<BdaReport, sqlx::Error> {
        let approved_at = chrono::Utc::now();
        
        sqlx::query(
            r#"
            UPDATE bda_reports 
            SET status = 'approved', 
                approved_by = $1, 
                approved_at = $2,
                review_comments = $3,
                updated_at = $2 
            WHERE id = $4
            "#
        )
        .bind(approver_id)
        .bind(&approved_at)
        .bind(&comments)
        .bind(id)
        .execute(&self.pool)
        .await?;
        
        self.get_by_id(id).await?
            .ok_or(sqlx::Error::RowNotFound)
    }
    
    /// Reject BDA report
    pub async fn reject(&self, id: &str, reviewer_id: &str, comments: String) -> Result<BdaReport, sqlx::Error> {
        let reviewed_at = chrono::Utc::now();
        
        sqlx::query(
            r#"
            UPDATE bda_reports 
            SET status = 'rejected',
                reviewed_by = $1,
                reviewed_at = $2,
                review_comments = $3,
                updated_at = $2
            WHERE id = $4
            "#
        )
        .bind(reviewer_id)
        .bind(&reviewed_at)
        .bind(&comments)
        .bind(id)
        .execute(&self.pool)
        .await?;
        
        self.get_by_id(id).await?
            .ok_or(sqlx::Error::RowNotFound)
    }
    
    /// Delete BDA report
    pub async fn delete(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM bda_reports WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
    
    /// Get BDA statistics
    pub async fn get_statistics(&self) -> Result<BdaStatistics, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total,
                SUM(CASE WHEN status = 'draft' THEN 1 ELSE 0 END) as draft,
                SUM(CASE WHEN status = 'submitted' THEN 1 ELSE 0 END) as submitted,
                SUM(CASE WHEN status = 'reviewed' THEN 1 ELSE 0 END) as reviewed,
                SUM(CASE WHEN status = 'approved' THEN 1 ELSE 0 END) as approved,
                SUM(CASE WHEN status = 'rejected' THEN 1 ELSE 0 END) as rejected,
                SUM(CASE WHEN recommendation = 'effect_achieved' THEN 1 ELSE 0 END) as rec_achieved,
                SUM(CASE WHEN recommendation = 'monitor' THEN 1 ELSE 0 END) as rec_monitor,
                SUM(CASE WHEN recommendation = 're_attack' THEN 1 ELSE 0 END) as rec_reattack,
                SUM(CASE WHEN recommendation = 're_weaponeer' THEN 1 ELSE 0 END) as rec_reweaponeer,
                SUM(CASE WHEN physical_damage = 'ND' THEN 1 ELSE 0 END) as phys_nd,
                SUM(CASE WHEN physical_damage = 'SD' THEN 1 ELSE 0 END) as phys_sd,
                SUM(CASE WHEN physical_damage = 'MD' THEN 1 ELSE 0 END) as phys_md,
                SUM(CASE WHEN physical_damage = 'SVD' THEN 1 ELSE 0 END) as phys_svd,
                SUM(CASE WHEN physical_damage = 'D' THEN 1 ELSE 0 END) as phys_d,
                AVG(confidence_level) as avg_confidence,
                SUM(CASE WHEN collateral_damage_detected = 1 THEN 1 ELSE 0 END) as collateral_incidents
            FROM bda_reports
            "#
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(BdaStatistics {
            total_reports: row.get("total"),
            by_status: BdaStatusCounts {
                draft: row.get("draft"),
                submitted: row.get("submitted"),
                reviewed: row.get("reviewed"),
                approved: row.get("approved"),
                rejected: row.get("rejected"),
            },
            by_recommendation: BdaRecommendationCounts {
                effect_achieved: row.get("rec_achieved"),
                monitor: row.get("rec_monitor"),
                re_attack: row.get("rec_reattack"),
                re_weaponeer: row.get("rec_reweaponeer"),
            },
            by_physical_damage: BdaPhysicalDamageCounts {
                nd: row.get("phys_nd"),
                sd: row.get("phys_sd"),
                md: row.get("phys_md"),
                svd: row.get("phys_svd"),
                d: row.get("phys_d"),
            },
            average_confidence: row.get::<Option<f64>, _>("avg_confidence").unwrap_or(0.0) as f32,
            collateral_damage_incidents: row.get("collateral_incidents"),
        })
    }
    
    /// Helper: Convert SQLite row to BdaReport
    fn row_to_bda_report(&self, row: sqlx::sqlite::SqliteRow) -> Result<BdaReport, sqlx::Error> {
        Ok(BdaReport {
            id: row.get("id"),
            target_id: row.get("target_id"),
            strike_id: row.get("strike_id"),
            assessment_date: row.get::<chrono::DateTime<chrono::Utc>, _>("assessment_date").to_rfc3339(),
            analyst_id: row.get("analyst_id"),
            assessment_type: self.parse_assessment_type(row.get("assessment_type")),
            physical_damage: self.parse_physical_damage(row.get("physical_damage")),
            physical_damage_percentage: row.get("physical_damage_percentage"),
            damage_description: row.get("damage_description"),
            functional_damage: self.parse_functional_damage(row.get("functional_damage")),
            estimated_repair_time_hours: row.get("estimated_repair_time_hours"),
            pre_strike_capability_baseline: row.get("pre_strike_capability_baseline"),
            desired_effect: row.get("desired_effect"),
            achieved_effect: row.get("achieved_effect"),
            effect_level: row.get::<Option<String>, _>("effect_level").map(|s| self.parse_effect_level(&s)),
            unintended_effects: row.get("unintended_effects"),
            confidence_level: row.get::<f64, _>("confidence_level") as f32,
            assessment_quality: None,  // Not in DB yet
            limiting_factors: row.get("limiting_factors"),
            recommendation: self.parse_recommendation(row.get("recommendation")),
            re_attack_priority: row.get("re_attack_priority"),
            re_attack_rationale: row.get("re_attack_rationale"),
            alternative_munitions: row.get("alternative_munitions"),
            collateral_damage_detected: row.get("collateral_damage_detected"),
            civcas_credibility: row.get::<Option<String>, _>("civcas_credibility").map(|s| self.parse_civcas(&s)),
            civilian_casualties_estimate: row.get("civilian_casualties_estimate"),
            protected_structures_damaged: row.get("protected_structures_damaged"),
            cde_vs_actual_comparison: row.get("cde_vs_actual_comparison"),
            weapon_performance_vs_predicted: row.get::<Option<String>, _>("weapon_performance_vs_predicted").map(|s| self.parse_weapon_perf(&s)),
            munition_reliability: row.get("munition_reliability"),
            circular_error_probable_meters: row.get::<Option<f64>, _>("circular_error_probable_meters").map(|v| v as f32),
            penetration_depth_meters: row.get::<Option<f64>, _>("penetration_depth_meters").map(|v| v as f32),
            status: self.parse_status(row.get("status")),
            submitted_at: row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("submitted_at").map(|dt| dt.to_rfc3339()),
            reviewed_by: row.get("reviewed_by"),
            reviewed_at: row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("reviewed_at").map(|dt| dt.to_rfc3339()),
            review_comments: row.get("review_comments"),
            approved_by: row.get("approved_by"),
            approved_at: row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("approved_at").map(|dt| dt.to_rfc3339()),
            classification_level: row.get("classification_level"),
            handling_caveats: row.get("handling_caveats"),
            created_at: row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
            updated_at: row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at").to_rfc3339(),
            notes: row.get("notes"),
        })
    }
    
    // Helper parsers
    fn parse_physical_damage(&self, s: String) -> PhysicalDamage {
        match s.as_str() {
            "ND" => PhysicalDamage::ND,
            "SD" => PhysicalDamage::SD,
            "MD" => PhysicalDamage::MD,
            "SVD" => PhysicalDamage::SVD,
            "D" => PhysicalDamage::D,
            _ => PhysicalDamage::ND,
        }
    }
    
    fn parse_functional_damage(&self, s: String) -> FunctionalDamage {
        match s.as_str() {
            "FMC" => FunctionalDamage::FMC,
            "PMC" => FunctionalDamage::PMC,
            "NMC" => FunctionalDamage::NMC,
            _ => FunctionalDamage::FMC,
        }
    }
    
    fn parse_assessment_type(&self, s: String) -> AssessmentType {
        match s.as_str() {
            "initial" => AssessmentType::Initial,
            "interim" => AssessmentType::Interim,
            "final" => AssessmentType::Final,
            _ => AssessmentType::Initial,
        }
    }
    
    fn parse_status(&self, s: String) -> BdaStatus {
        match s.as_str() {
            "draft" => BdaStatus::Draft,
            "submitted" => BdaStatus::Submitted,
            "reviewed" => BdaStatus::Reviewed,
            "approved" => BdaStatus::Approved,
            "rejected" => BdaStatus::Rejected,
            _ => BdaStatus::Draft,
        }
    }
    
    fn parse_recommendation(&self, s: String) -> Recommendation {
        match s.as_str() {
            "effect_achieved" => Recommendation::EffectAchieved,
            "monitor" => Recommendation::Monitor,
            "re_attack" => Recommendation::ReAttack,
            "re_weaponeer" => Recommendation::ReWeaponeer,
            _ => Recommendation::Monitor,
        }
    }
    
    fn parse_effect_level(&self, s: &str) -> crate::features::bda::domain::EffectLevel {
        match s {
            "first_order" => crate::features::bda::domain::EffectLevel::FirstOrder,
            "second_order" => crate::features::bda::domain::EffectLevel::SecondOrder,
            "third_order" => crate::features::bda::domain::EffectLevel::ThirdOrder,
            _ => crate::features::bda::domain::EffectLevel::FirstOrder,
        }
    }
    
    fn parse_civcas(&self, s: &str) -> crate::features::bda::domain::CivcasCredibility {
        match s {
            "no_credibility" => crate::features::bda::domain::CivcasCredibility::NoCredibility,
            "possible" => crate::features::bda::domain::CivcasCredibility::Possible,
            "credible" => crate::features::bda::domain::CivcasCredibility::Credible,
            "confirmed" => crate::features::bda::domain::CivcasCredibility::Confirmed,
            _ => crate::features::bda::domain::CivcasCredibility::NoCredibility,
        }
    }
    
    fn parse_weapon_perf(&self, s: &str) -> crate::features::bda::domain::WeaponPerformance {
        match s {
            "exceeded" => crate::features::bda::domain::WeaponPerformance::Exceeded,
            "met" => crate::features::bda::domain::WeaponPerformance::Met,
            "below" => crate::features::bda::domain::WeaponPerformance::Below,
            "failed" => crate::features::bda::domain::WeaponPerformance::Failed,
            _ => crate::features::bda::domain::WeaponPerformance::Met,
        }
    }
}

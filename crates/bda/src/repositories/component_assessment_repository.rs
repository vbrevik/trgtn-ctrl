// BDA Component Assessment Repository
// Purpose: Database access layer for component-level assessments

use crate::features::bda::domain::component_assessment::{
    BdaComponentAssessment,
    CreateComponentAssessmentRequest,
    UpdateComponentAssessmentRequest,
    ComponentType,
    ComponentCriticality,
};
use crate::features::bda::domain::{PhysicalDamage, FunctionalDamage};
use sqlx::{Pool, Sqlite, Row};
use uuid::Uuid;

pub struct ComponentAssessmentRepository {
    pool: Pool<Sqlite>,
}

impl ComponentAssessmentRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
    
    /// Create new component assessment
    pub async fn create(
        &self,
        assessed_by: &str,
        req: CreateComponentAssessmentRequest,
    ) -> Result<BdaComponentAssessment, sqlx::Error> {
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
        
        let component_type_str = match req.component_type {
            ComponentType::Structure => "structure",
            ComponentType::Equipment => "equipment",
            ComponentType::Infrastructure => "infrastructure",
            ComponentType::Vehicle => "vehicle",
            ComponentType::Other => "other",
        };
        
        let component_criticality_str = req.component_criticality.as_ref().map(|c| match c {
            ComponentCriticality::Critical => "critical",
            ComponentCriticality::Important => "important",
            ComponentCriticality::Supporting => "supporting",
            ComponentCriticality::NonEssential => "non_essential",
        });
        
        sqlx::query(
            r#"
            INSERT INTO bda_component_assessment (
                id, bda_report_id, component_name, component_type, component_location,
                physical_damage, physical_damage_percentage, damage_description,
                functional_damage, estimated_repair_time_hours, repair_cost_estimate_usd,
                component_criticality, pre_strike_function, post_strike_function,
                replacement_required, replacement_availability_days,
                assessed_by, assessed_at, confidence_level, assessment_notes,
                classification_level, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16,
                $17, $18, $19, $20, $21, $22, $23
            )
            "#
        )
        .bind(&id)
        .bind(&req.bda_report_id)
        .bind(&req.component_name)
        .bind(component_type_str)
        .bind(&req.component_location)
        .bind(physical_damage_str)
        .bind(req.physical_damage_percentage)
        .bind(&req.damage_description)
        .bind(functional_damage_str)
        .bind(req.estimated_repair_time_hours)
        .bind(req.repair_cost_estimate_usd)
        .bind(component_criticality_str)
        .bind(&req.pre_strike_function)
        .bind(&req.post_strike_function)
        .bind(req.replacement_required.unwrap_or(false))
        .bind(req.replacement_availability_days)
        .bind(assessed_by)
        .bind(&created_at)
        .bind(req.confidence_level)
        .bind(&req.assessment_notes)
        .bind(req.classification_level.as_deref().unwrap_or("SECRET"))
        .bind(&created_at)
        .bind(&created_at)
        .execute(&self.pool)
        .await?;
        
        self.get_by_id(&id).await?
            .ok_or(sqlx::Error::RowNotFound)
    }
    
    /// Get component assessment by ID
    pub async fn get_by_id(&self, id: &str) -> Result<Option<BdaComponentAssessment>, sqlx::Error> {
        let row = sqlx::query("SELECT * FROM bda_component_assessment WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        
        match row {
            Some(r) => Ok(Some(self.row_to_component(r)?)),
            None => Ok(None),
        }
    }
    
    /// Get all component assessments for a BDA report
    pub async fn get_by_report(&self, bda_report_id: &str) -> Result<Vec<BdaComponentAssessment>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT * FROM bda_component_assessment WHERE bda_report_id = $1 ORDER BY component_criticality DESC, component_name ASC"
        )
        .bind(bda_report_id)
        .fetch_all(&self.pool)
        .await?;
        
        let mut components = Vec::new();
        for row in rows {
            components.push(self.row_to_component(row)?);
        }
        
        Ok(components)
    }
    
    /// Update component assessment
    pub async fn update(
        &self,
        id: &str,
        req: UpdateComponentAssessmentRequest,
    ) -> Result<BdaComponentAssessment, sqlx::Error> {
        let updated_at = chrono::Utc::now();
        
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
        
        let component_criticality_str = req.component_criticality.as_ref().map(|c| match c {
            ComponentCriticality::Critical => "critical",
            ComponentCriticality::Important => "important",
            ComponentCriticality::Supporting => "supporting",
            ComponentCriticality::NonEssential => "non_essential",
        });
        
        sqlx::query(
            r#"
            UPDATE bda_component_assessment 
            SET component_name = COALESCE($1, component_name),
                physical_damage = COALESCE($2, physical_damage),
                physical_damage_percentage = COALESCE($3, physical_damage_percentage),
                damage_description = COALESCE($4, damage_description),
                functional_damage = COALESCE($5, functional_damage),
                estimated_repair_time_hours = COALESCE($6, estimated_repair_time_hours),
                repair_cost_estimate_usd = COALESCE($7, repair_cost_estimate_usd),
                component_criticality = COALESCE($8, component_criticality),
                post_strike_function = COALESCE($9, post_strike_function),
                replacement_required = COALESCE($10, replacement_required),
                replacement_availability_days = COALESCE($11, replacement_availability_days),
                confidence_level = COALESCE($12, confidence_level),
                assessment_notes = COALESCE($13, assessment_notes),
                updated_at = $14
            WHERE id = $15
            "#
        )
        .bind(&req.component_name)
        .bind(physical_damage_str)
        .bind(req.physical_damage_percentage)
        .bind(&req.damage_description)
        .bind(functional_damage_str)
        .bind(req.estimated_repair_time_hours)
        .bind(req.repair_cost_estimate_usd)
        .bind(component_criticality_str)
        .bind(&req.post_strike_function)
        .bind(req.replacement_required)
        .bind(req.replacement_availability_days)
        .bind(req.confidence_level)
        .bind(&req.assessment_notes)
        .bind(&updated_at)
        .bind(id)
        .execute(&self.pool)
        .await?;
        
        self.get_by_id(id).await?
            .ok_or(sqlx::Error::RowNotFound)
    }
    
    /// Delete component assessment
    pub async fn delete(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM bda_component_assessment WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
    
    /// Helper: Convert SQLite row to BdaComponentAssessment
    fn row_to_component(&self, row: sqlx::sqlite::SqliteRow) -> Result<BdaComponentAssessment, sqlx::Error> {
        Ok(BdaComponentAssessment {
            id: row.get("id"),
            bda_report_id: row.get("bda_report_id"),
            component_name: row.get("component_name"),
            component_type: self.parse_component_type(&row.get::<String, _>("component_type")),
            component_location: row.get("component_location"),
            physical_damage: self.parse_physical_damage(&row.get::<String, _>("physical_damage")),
            physical_damage_percentage: row.get("physical_damage_percentage"),
            damage_description: row.get("damage_description"),
            functional_damage: self.parse_functional_damage(&row.get::<String, _>("functional_damage")),
            estimated_repair_time_hours: row.get("estimated_repair_time_hours"),
            repair_cost_estimate_usd: row.get("repair_cost_estimate_usd"),
            component_criticality: row.get::<Option<String>, _>("component_criticality")
                .map(|s| self.parse_component_criticality(&s)),
            pre_strike_function: row.get("pre_strike_function"),
            post_strike_function: row.get("post_strike_function"),
            replacement_required: row.get("replacement_required"),
            replacement_availability_days: row.get("replacement_availability_days"),
            assessed_by: row.get("assessed_by"),
            assessed_at: row.get::<chrono::DateTime<chrono::Utc>, _>("assessed_at").to_rfc3339(),
            confidence_level: row.get("confidence_level"),
            assessment_notes: row.get("assessment_notes"),
            classification_level: row.get("classification_level"),
            created_at: row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
            updated_at: row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at").to_rfc3339(),
        })
    }
    
    fn parse_component_type(&self, s: &str) -> ComponentType {
        match s {
            "structure" => ComponentType::Structure,
            "equipment" => ComponentType::Equipment,
            "infrastructure" => ComponentType::Infrastructure,
            "vehicle" => ComponentType::Vehicle,
            _ => ComponentType::Other,
        }
    }
    
    fn parse_component_criticality(&self, s: &str) -> ComponentCriticality {
        match s {
            "critical" => ComponentCriticality::Critical,
            "important" => ComponentCriticality::Important,
            "supporting" => ComponentCriticality::Supporting,
            _ => ComponentCriticality::NonEssential,
        }
    }
    
    fn parse_physical_damage(&self, s: &str) -> PhysicalDamage {
        match s {
            "ND" => PhysicalDamage::ND,
            "SD" => PhysicalDamage::SD,
            "MD" => PhysicalDamage::MD,
            "SVD" => PhysicalDamage::SVD,
            "D" => PhysicalDamage::D,
            _ => PhysicalDamage::ND,
        }
    }
    
    fn parse_functional_damage(&self, s: &str) -> FunctionalDamage {
        match s {
            "FMC" => FunctionalDamage::FMC,
            "PMC" => FunctionalDamage::PMC,
            "NMC" => FunctionalDamage::NMC,
            _ => FunctionalDamage::FMC,
        }
    }
}

// BDA Strike Correlation Repository
// Purpose: Database access layer for strike correlation data

use crate::features::bda::domain::{
    BdaStrikeCorrelation, CreateStrikeCorrelationRequest, 
    GuidancePerformance, WeaponPerformanceSummary
};
use sqlx::{Pool, Sqlite, Row};
use uuid::Uuid;

pub struct StrikeRepository {
    pool: Pool<Sqlite>,
}

impl StrikeRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
    
    /// Create strike correlation
    pub async fn create(&self, req: CreateStrikeCorrelationRequest) -> Result<BdaStrikeCorrelation, sqlx::Error> {
        req.validate().map_err(|e| sqlx::Error::Decode(e.into()))?;
        
        let id = Uuid::new_v4().to_string();
        let created_at = chrono::Utc::now();
        
        let guidance_str = req.guidance_system_performance.as_ref().map(|g| match g {
            GuidancePerformance::Nominal => "nominal",
            GuidancePerformance::Degraded => "degraded",
            GuidancePerformance::Failed => "failed",
        });
        
        sqlx::query(
            r#"
            INSERT INTO bda_strike_correlation (
                id, bda_report_id, target_id, strike_mission_id, strike_sortie_number, strike_sequence,
                weapon_system, munition_type, munition_quantity, total_net_explosive_weight_kg,
                time_on_target, launch_time, time_of_flight_seconds,
                impact_coordinates, impact_coordinates_json, dmpi_coordinates, offset_from_dmpi_meters,
                successful_detonation, fuzing_as_designed, guidance_system_performance,
                circular_error_probable_meters, penetration_depth_meters, blast_radius_meters,
                weather_conditions, wind_speed_knots, wind_direction_degrees, temperature_celsius,
                malfunction_detected, malfunction_type, malfunction_description,
                jmem_predicted_damage, jmem_vs_actual_comparison,
                classification_level, handling_caveats,
                created_at, updated_at
            )
            VALUES (
                $1, $2, $3, $4, $5, $6,
                $7, $8, $9, $10,
                $11, $12, $13,
                $14, $15, $16, $17,
                $18, $19, $20,
                $21, $22, $23,
                $24, $25, $26, $27,
                $28, $29, $30,
                $31, $32,
                $33, $34,
                $35, $35
            )
            "#
        )
        .bind(&id)
        .bind(&req.bda_report_id)
        .bind(&req.target_id)
        .bind(&req.strike_mission_id)
        .bind(&req.strike_sortie_number)
        .bind(req.strike_sequence)
        .bind(&req.weapon_system)
        .bind(&req.munition_type)
        .bind(req.munition_quantity)
        .bind(req.total_net_explosive_weight_kg)
        .bind(&req.time_on_target)
        .bind(&req.launch_time)
        .bind(req.time_of_flight_seconds)
        .bind(&req.impact_coordinates)
        .bind(&req.impact_coordinates_json)
        .bind(&req.dmpi_coordinates)
        .bind(req.offset_from_dmpi_meters)
        .bind(req.successful_detonation)
        .bind(req.fuzing_as_designed)
        .bind(guidance_str)
        .bind(req.circular_error_probable_meters)
        .bind(req.penetration_depth_meters)
        .bind(req.blast_radius_meters)
        .bind(&req.weather_conditions)
        .bind(req.wind_speed_knots)
        .bind(req.wind_direction_degrees)
        .bind(req.temperature_celsius)
        .bind(req.malfunction_detected)
        .bind(&req.malfunction_type)
        .bind(&req.malfunction_description)
        .bind(&req.jmem_predicted_damage)
        .bind(&req.jmem_vs_actual_comparison)
        .bind(&req.classification_level)
        .bind(&req.handling_caveats)
        .bind(&created_at)
        .execute(&self.pool)
        .await?;
        
        self.get_by_id(&id).await?
            .ok_or(sqlx::Error::RowNotFound)
    }
    
    /// Get strike correlation by ID
    pub async fn get_by_id(&self, id: &str) -> Result<Option<BdaStrikeCorrelation>, sqlx::Error> {
        let row = sqlx::query("SELECT * FROM bda_strike_correlation WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        
        match row {
            Some(r) => Ok(Some(self.row_to_strike(r)?)),
            None => Ok(None),
        }
    }
    
    /// Get all strikes for a BDA report
    pub async fn get_by_report(&self, bda_report_id: &str) -> Result<Vec<BdaStrikeCorrelation>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT * FROM bda_strike_correlation WHERE bda_report_id = $1 ORDER BY time_on_target DESC"
        )
        .bind(bda_report_id)
        .fetch_all(&self.pool)
        .await?;
        
        let mut strikes = Vec::new();
        for row in rows {
            strikes.push(self.row_to_strike(row)?);
        }
        
        Ok(strikes)
    }

    /// Get all strikes for a target (based on strike_mission_id)
    pub async fn get_by_target(&self, target_id: &str) -> Result<Vec<BdaStrikeCorrelation>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT * FROM bda_strike_correlation WHERE target_id = $1 ORDER BY time_on_target DESC"
        )
        .bind(target_id)
        .fetch_all(&self.pool)
        .await?;
        
        let mut strikes = Vec::new();
        for row in rows {
            strikes.push(self.row_to_strike(row)?);
        }
        
        Ok(strikes)
    }
    
    /// Get weapon performance summary
    pub async fn get_weapon_performance_summary(&self) -> Result<Vec<WeaponPerformanceSummary>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT 
                weapon_system,
                munition_type,
                COUNT(*) as total_strikes,
                SUM(CASE WHEN successful_detonation = 1 THEN 1 ELSE 0 END) as successful_detonations,
                AVG(COALESCE(offset_from_dmpi_meters, 0)) as avg_cep_meters,
                AVG(COALESCE(blast_radius_meters, 0)) as avg_blast_radius_meters,
                SUM(CASE WHEN malfunction_detected = 1 THEN 1 ELSE 0 END) as malfunctions,
                ROUND(100.0 * SUM(CASE WHEN successful_detonation = 1 THEN 1 ELSE 0 END) / COUNT(*), 2) as reliability_percentage
            FROM bda_strike_correlation
            GROUP BY weapon_system, munition_type
            ORDER BY total_strikes DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        
        let mut summaries = Vec::new();
        for row in rows {
            summaries.push(WeaponPerformanceSummary {
                weapon_system: row.get("weapon_system"),
                munition_type: row.get("munition_type"),
                total_strikes: row.get("total_strikes"),
                successful_detonations: row.get("successful_detonations"),
                avg_cep_meters: row.get::<f64, _>("avg_cep_meters") as f32,
                avg_blast_radius_meters: row.get::<f64, _>("avg_blast_radius_meters") as f32,
                malfunctions: row.get("malfunctions"),
                reliability_percentage: row.get::<f64, _>("reliability_percentage") as f32,
            });
        }
        
        Ok(summaries)
    }
    
    /// Delete strike correlation
    pub async fn delete(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM bda_strike_correlation WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
    
    /// Helper: Convert SQLite row to BdaStrikeCorrelation
    fn row_to_strike(&self, row: sqlx::sqlite::SqliteRow) -> Result<BdaStrikeCorrelation, sqlx::Error> {
        Ok(BdaStrikeCorrelation {
            id: row.get("id"),
            bda_report_id: row.get("bda_report_id"),
            target_id: row.get("target_id"),
            strike_mission_id: row.get("strike_mission_id"),
            strike_sortie_number: row.get("strike_sortie_number"),
            strike_sequence: row.get("strike_sequence"),
            weapon_system: row.get("weapon_system"),
            munition_type: row.get("munition_type"),
            munition_quantity: row.get("munition_quantity"),
            total_net_explosive_weight_kg: row.get::<Option<f64>, _>("total_net_explosive_weight_kg").map(|v| v as f32),
            time_on_target: row.get::<chrono::DateTime<chrono::Utc>, _>("time_on_target").to_rfc3339(),
            launch_time: row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("launch_time").map(|dt| dt.to_rfc3339()),
            time_of_flight_seconds: row.get::<Option<f64>, _>("time_of_flight_seconds").map(|v| v as f32),
            impact_coordinates: row.get("impact_coordinates"),
            impact_coordinates_json: row.get("impact_coordinates_json"),
            dmpi_coordinates: row.get("dmpi_coordinates"),
            offset_from_dmpi_meters: row.get::<Option<f64>, _>("offset_from_dmpi_meters").map(|v| v as f32),
            successful_detonation: row.get("successful_detonation"),
            fuzing_as_designed: row.get("fuzing_as_designed"),
            guidance_system_performance: row.get::<Option<String>, _>("guidance_system_performance").map(|s| self.parse_guidance(&s)),
            circular_error_probable_meters: row.get::<Option<f64>, _>("circular_error_probable_meters").map(|v| v as f32),
            penetration_depth_meters: row.get::<Option<f64>, _>("penetration_depth_meters").map(|v| v as f32),
            blast_radius_meters: row.get::<Option<f64>, _>("blast_radius_meters").map(|v| v as f32),
            weather_conditions: row.get("weather_conditions"),
            wind_speed_knots: row.get::<Option<f64>, _>("wind_speed_knots").map(|v| v as f32),
            wind_direction_degrees: row.get::<Option<f64>, _>("wind_direction_degrees").map(|v| v as f32),
            temperature_celsius: row.get::<Option<f64>, _>("temperature_celsius").map(|v| v as f32),
            malfunction_detected: row.get("malfunction_detected"),
            malfunction_type: row.get("malfunction_type"),
            malfunction_description: row.get("malfunction_description"),
            jmem_predicted_damage: row.get("jmem_predicted_damage"),
            jmem_vs_actual_comparison: row.get("jmem_vs_actual_comparison"),
            classification_level: row.get("classification_level"),
            handling_caveats: row.get("handling_caveats"),
            created_at: row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
            updated_at: row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at").to_rfc3339(),
        })
    }
    
    fn parse_guidance(&self, s: &str) -> GuidancePerformance {
        match s {
            "nominal" => GuidancePerformance::Nominal,
            "degraded" => GuidancePerformance::Degraded,
            "failed" => GuidancePerformance::Failed,
            _ => GuidancePerformance::Nominal,
        }
    }
}

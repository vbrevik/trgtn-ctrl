// BDA Strike Correlation Domain Model
// Purpose: Link BDA reports to strike data for weaponeering validation

use serde::{Deserialize, Serialize};

// ============================================================================
// ENUMS
// ============================================================================

/// Guidance system performance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GuidancePerformance {
    /// Worked as designed
    Nominal,
    /// Degraded but functional
    Degraded,
    /// Complete failure
    Failed,
}

// ============================================================================
// STRUCTS
// ============================================================================

/// Strike correlation data for weaponeering validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BdaStrikeCorrelation {
    pub id: String,
    pub bda_report_id: String,
    pub target_id: Option<String>,
    
    // Strike Identification
    pub strike_mission_id: Option<String>,
    pub strike_sortie_number: Option<String>,
    pub strike_sequence: Option<i32>,
    
    // Weapon System
    pub weapon_system: String,  // F-16, F-35, Reaper, HIMARS, etc.
    pub munition_type: String,  // GBU-31, AGM-114, etc.
    pub munition_quantity: i32,
    pub total_net_explosive_weight_kg: Option<f32>,
    
    // Strike Timing
    pub time_on_target: String,  // ISO 8601 timestamp
    pub launch_time: Option<String>,
    pub time_of_flight_seconds: Option<f32>,
    
    // Impact Location
    pub impact_coordinates: String,  // "35.123, -117.456" or MGRS
    pub impact_coordinates_json: Option<String>,  // {"lat": 35.123, "lon": -117.456}
    pub dmpi_coordinates: Option<String>,  // Desired Mean Point of Impact
    pub offset_from_dmpi_meters: Option<f32>,
    
    // Weapon Performance
    pub successful_detonation: Option<bool>,
    pub fuzing_as_designed: Option<bool>,
    pub guidance_system_performance: Option<GuidancePerformance>,
    pub circular_error_probable_meters: Option<f32>,
    pub penetration_depth_meters: Option<f32>,
    pub blast_radius_meters: Option<f32>,
    
    // Environmental Factors
    pub weather_conditions: Option<String>,
    pub wind_speed_knots: Option<f32>,
    pub wind_direction_degrees: Option<f32>,
    pub temperature_celsius: Option<f32>,
    
    // Malfunctions
    pub malfunction_detected: bool,
    pub malfunction_type: Option<String>,
    pub malfunction_description: Option<String>,
    
    // JMEM Validation
    pub jmem_predicted_damage: Option<String>,
    pub jmem_vs_actual_comparison: Option<String>,
    
    // Classification
    pub classification_level: String,
    pub handling_caveats: Option<String>,
    
    // Timestamps
    pub created_at: String,
    pub updated_at: String,
}

/// Request to create strike correlation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStrikeCorrelationRequest {
    pub bda_report_id: String,
    pub target_id: Option<String>,
    pub strike_mission_id: Option<String>,
    pub strike_sortie_number: Option<String>,
    pub strike_sequence: Option<i32>,
    pub weapon_system: String,
    pub munition_type: String,
    pub munition_quantity: i32,
    pub total_net_explosive_weight_kg: Option<f32>,
    pub time_on_target: String,
    pub launch_time: Option<String>,
    pub time_of_flight_seconds: Option<f32>,
    pub impact_coordinates: String,
    pub impact_coordinates_json: Option<String>,
    pub dmpi_coordinates: Option<String>,
    pub offset_from_dmpi_meters: Option<f32>,
    pub successful_detonation: Option<bool>,
    pub fuzing_as_designed: Option<bool>,
    pub guidance_system_performance: Option<GuidancePerformance>,
    pub circular_error_probable_meters: Option<f32>,
    pub penetration_depth_meters: Option<f32>,
    pub blast_radius_meters: Option<f32>,
    pub weather_conditions: Option<String>,
    pub wind_speed_knots: Option<f32>,
    pub wind_direction_degrees: Option<f32>,
    pub temperature_celsius: Option<f32>,
    pub malfunction_detected: bool,
    pub malfunction_type: Option<String>,
    pub malfunction_description: Option<String>,
    pub jmem_predicted_damage: Option<String>,
    pub jmem_vs_actual_comparison: Option<String>,
    pub classification_level: String,
    pub handling_caveats: Option<String>,
}

/// Weapon system performance summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponPerformanceSummary {
    pub weapon_system: String,
    pub munition_type: String,
    pub total_strikes: i64,
    pub successful_detonations: i64,
    pub avg_cep_meters: f32,
    pub avg_blast_radius_meters: f32,
    pub malfunctions: i64,
    pub reliability_percentage: f32,
}

// ============================================================================
// BUSINESS LOGIC
// ============================================================================

impl BdaStrikeCorrelation {
    /// Calculate circular error probable (CEP) accuracy category
    pub fn get_cep_category(&self) -> &str {
        match self.circular_error_probable_meters {
            Some(cep) if cep < 5.0 => "Excellent",
            Some(cep) if cep < 10.0 => "Good",
            Some(cep) if cep < 20.0 => "Acceptable",
            Some(cep) if cep < 50.0 => "Poor",
            Some(_) => "Very Poor",
            None => "Unknown",
        }
    }
    
    /// Check if weapon system performed nominally
    pub fn is_weapon_performance_nominal(&self) -> bool {
        let detonation_ok = self.successful_detonation.unwrap_or(false);
        let fuzing_ok = self.fuzing_as_designed.unwrap_or(false);
        let guidance_ok = matches!(
            self.guidance_system_performance,
            Some(GuidancePerformance::Nominal)
        );
        let no_malfunction = !self.malfunction_detected;
        
        detonation_ok && fuzing_ok && guidance_ok && no_malfunction
    }
    
    /// Calculate impact accuracy (offset from DMPI)
    pub fn get_impact_accuracy_meters(&self) -> Option<f32> {
        self.offset_from_dmpi_meters
    }
    
    /// Check if strike was within acceptable CEP
    pub fn is_within_acceptable_cep(&self, threshold_meters: f32) -> bool {
        self.circular_error_probable_meters
            .map_or(false, |cep| cep <= threshold_meters)
    }
}

impl CreateStrikeCorrelationRequest {
    /// Validate create request
    pub fn validate(&self) -> Result<(), String> {
        if self.bda_report_id.is_empty() {
            return Err("BDA report ID is required".to_string());
        }
        
        if self.weapon_system.is_empty() {
            return Err("Weapon system is required".to_string());
        }
        
        if self.munition_type.is_empty() {
            return Err("Munition type is required".to_string());
        }
        
        if self.munition_quantity < 1 {
            return Err("Munition quantity must be at least 1".to_string());
        }
        
        if self.impact_coordinates.is_empty() {
            return Err("Impact coordinates are required".to_string());
        }
        
        Ok(())
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_strike() -> BdaStrikeCorrelation {
        BdaStrikeCorrelation {
            id: "strike-123".to_string(),
            bda_report_id: "report-456".to_string(),
            strike_mission_id: Some("MISSION-789".to_string()),
            strike_sortie_number: Some("F16-001".to_string()),
            strike_sequence: Some(1),
            weapon_system: "F-16C".to_string(),
            munition_type: "GBU-31 JDAM".to_string(),
            munition_quantity: 2,
            total_net_explosive_weight_kg: Some(430.0),
            time_on_target: "2026-01-21T12:00:00Z".to_string(),
            launch_time: Some("2026-01-21T11:58:30Z".to_string()),
            time_of_flight_seconds: Some(90.0),
            impact_coordinates: "35.123, -117.456".to_string(),
            impact_coordinates_json: Some(r#"{"lat": 35.123, "lon": -117.456}"#.to_string()),
            dmpi_coordinates: Some("35.124, -117.455".to_string()),
            offset_from_dmpi_meters: Some(8.5),
            successful_detonation: Some(true),
            fuzing_as_designed: Some(true),
            guidance_system_performance: Some(GuidancePerformance::Nominal),
            circular_error_probable_meters: Some(8.5),
            penetration_depth_meters: None,
            blast_radius_meters: Some(150.0),
            weather_conditions: Some("Clear".to_string()),
            wind_speed_knots: Some(5.0),
            wind_direction_degrees: Some(270.0),
            temperature_celsius: Some(18.0),
            malfunction_detected: false,
            malfunction_type: None,
            malfunction_description: None,
            jmem_predicted_damage: Some("75% structure damage".to_string()),
            jmem_vs_actual_comparison: Some("JMEM slightly overestimated".to_string()),
            classification_level: "SECRET".to_string(),
            handling_caveats: None,
            created_at: "2026-01-21T14:00:00Z".to_string(),
            updated_at: "2026-01-21T14:00:00Z".to_string(),
        }
    }
    
    #[test]
    fn test_cep_category() {
        let mut strike = create_test_strike();
        
        strike.circular_error_probable_meters = Some(3.0);
        assert_eq!(strike.get_cep_category(), "Excellent");
        
        strike.circular_error_probable_meters = Some(8.0);
        assert_eq!(strike.get_cep_category(), "Good");
        
        strike.circular_error_probable_meters = Some(15.0);
        assert_eq!(strike.get_cep_category(), "Acceptable");
        
        strike.circular_error_probable_meters = Some(40.0);
        assert_eq!(strike.get_cep_category(), "Poor");
        
        strike.circular_error_probable_meters = Some(100.0);
        assert_eq!(strike.get_cep_category(), "Very Poor");
    }
    
    #[test]
    fn test_nominal_performance() {
        let mut strike = create_test_strike();
        assert!(strike.is_weapon_performance_nominal());
        
        strike.successful_detonation = Some(false);
        assert!(!strike.is_weapon_performance_nominal());
        
        strike.successful_detonation = Some(true);
        strike.malfunction_detected = true;
        assert!(!strike.is_weapon_performance_nominal());
    }
    
    #[test]
    fn test_acceptable_cep() {
        let strike = create_test_strike();
        
        assert!(strike.is_within_acceptable_cep(10.0));
        assert!(!strike.is_within_acceptable_cep(5.0));
    }
}

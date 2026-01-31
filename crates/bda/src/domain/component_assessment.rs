// BDA Component Assessment Domain Model
// Purpose: Track damage to individual target components

use serde::{Deserialize, Serialize};

/// Component type categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComponentType {
    Structure,
    Equipment,
    Infrastructure,
    Vehicle,
    Other,
}

/// Component criticality level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComponentCriticality {
    Critical,
    Important,
    Supporting,
    NonEssential,
}

/// BDA Component Assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BdaComponentAssessment {
    pub id: String,
    pub bda_report_id: String,
    
    // Component Identification
    pub component_name: String,
    pub component_type: ComponentType,
    pub component_location: Option<String>,
    
    // Physical Damage Assessment
    pub physical_damage: crate::features::bda::domain::PhysicalDamage,
    pub physical_damage_percentage: Option<i32>,
    pub damage_description: Option<String>,
    
    // Functional Damage Assessment
    pub functional_damage: crate::features::bda::domain::FunctionalDamage,
    pub estimated_repair_time_hours: Option<i32>,
    pub repair_cost_estimate_usd: Option<f64>,
    
    // Component-Specific Details
    pub component_criticality: Option<ComponentCriticality>,
    pub pre_strike_function: Option<String>,
    pub post_strike_function: Option<String>,
    pub replacement_required: bool,
    pub replacement_availability_days: Option<i32>,
    
    // Assessment Metadata
    pub assessed_by: String,
    pub assessed_at: String,  // ISO 8601 timestamp
    pub confidence_level: Option<f32>,
    pub assessment_notes: Option<String>,
    
    // Classification
    pub classification_level: String,
    
    // Timestamps
    pub created_at: String,
    pub updated_at: String,
}

/// Request to create component assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateComponentAssessmentRequest {
    pub bda_report_id: String,
    pub component_name: String,
    pub component_type: ComponentType,
    pub component_location: Option<String>,
    pub physical_damage: crate::features::bda::domain::PhysicalDamage,
    pub physical_damage_percentage: Option<i32>,
    pub damage_description: Option<String>,
    pub functional_damage: crate::features::bda::domain::FunctionalDamage,
    pub estimated_repair_time_hours: Option<i32>,
    pub repair_cost_estimate_usd: Option<f64>,
    pub component_criticality: Option<ComponentCriticality>,
    pub pre_strike_function: Option<String>,
    pub post_strike_function: Option<String>,
    pub replacement_required: Option<bool>,
    pub replacement_availability_days: Option<i32>,
    pub confidence_level: Option<f32>,
    pub assessment_notes: Option<String>,
    pub classification_level: Option<String>,
}

/// Request to update component assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateComponentAssessmentRequest {
    pub component_name: Option<String>,
    pub physical_damage: Option<crate::features::bda::domain::PhysicalDamage>,
    pub physical_damage_percentage: Option<i32>,
    pub damage_description: Option<String>,
    pub functional_damage: Option<crate::features::bda::domain::FunctionalDamage>,
    pub estimated_repair_time_hours: Option<i32>,
    pub repair_cost_estimate_usd: Option<f64>,
    pub component_criticality: Option<ComponentCriticality>,
    pub post_strike_function: Option<String>,
    pub replacement_required: Option<bool>,
    pub replacement_availability_days: Option<i32>,
    pub confidence_level: Option<f32>,
    pub assessment_notes: Option<String>,
}

impl CreateComponentAssessmentRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.component_name.is_empty() {
            return Err("Component name is required".to_string());
        }
        
        if let Some(percentage) = self.physical_damage_percentage {
            if percentage < 0 || percentage > 100 {
                return Err("Physical damage percentage must be between 0 and 100".to_string());
            }
        }
        
        if let Some(confidence) = self.confidence_level {
            if confidence < 0.0 || confidence > 1.0 {
                return Err("Confidence level must be between 0.0 and 1.0".to_string());
            }
        }
        
        Ok(())
    }
}

impl BdaComponentAssessment {
    /// Check if component is critical and destroyed
    pub fn is_critical_destroyed(&self) -> bool {
        matches!(
            (self.component_criticality, self.physical_damage),
            (Some(ComponentCriticality::Critical), crate::features::bda::domain::PhysicalDamage::D)
        )
    }
    
    /// Check if component needs replacement
    pub fn needs_replacement(&self) -> bool {
        self.replacement_required || 
        matches!(self.physical_damage, crate::features::bda::domain::PhysicalDamage::D)
    }
    
    /// Get estimated total downtime (repair time + replacement time if needed)
    pub fn get_total_downtime_hours(&self) -> Option<i32> {
        let repair_time = self.estimated_repair_time_hours.unwrap_or(0);
        let replacement_time = if self.replacement_required {
            self.replacement_availability_days.map(|d| d * 24).unwrap_or(0)
        } else {
            0
        };
        
        if repair_time > 0 || replacement_time > 0 {
            Some(repair_time + replacement_time)
        } else {
            None
        }
    }
}

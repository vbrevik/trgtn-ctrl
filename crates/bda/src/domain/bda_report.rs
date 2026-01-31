// BDA Report Domain Model
// Purpose: Core domain entity for Battle Damage Assessment reports
// Implements business logic and validation per NATO COPD & JP 3-60

use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// ENUMS: Domain value objects with validation
// ============================================================================

/// Physical damage categories per Joint Publication 3-60
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum PhysicalDamage {
    /// No Damage (0% capability loss)
    ND,
    /// Slight Damage (<10% capability loss)
    SD,
    /// Moderate Damage (10-50% capability loss)
    MD,
    /// Severe Damage (50-90% capability loss)
    SVD,
    /// Destroyed (>90% capability loss, not economically repairable)
    D,
}

impl fmt::Display for PhysicalDamage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PhysicalDamage::ND => write!(f, "No Damage"),
            PhysicalDamage::SD => write!(f, "Slight Damage"),
            PhysicalDamage::MD => write!(f, "Moderate Damage"),
            PhysicalDamage::SVD => write!(f, "Severe Damage"),
            PhysicalDamage::D => write!(f, "Destroyed"),
        }
    }
}

/// Functional status categories per Joint Publication 3-60
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum FunctionalDamage {
    /// Fully Mission Capable (can perform all intended functions)
    FMC,
    /// Partially Mission Capable (degraded but partially functional)
    PMC,
    /// Not Mission Capable (cannot perform primary mission)
    NMC,
}

impl fmt::Display for FunctionalDamage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FunctionalDamage::FMC => write!(f, "Fully Mission Capable"),
            FunctionalDamage::PMC => write!(f, "Partially Mission Capable"),
            FunctionalDamage::NMC => write!(f, "Not Mission Capable"),
        }
    }
}

/// Assessment type (timing-based)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AssessmentType {
    /// First assessment within 24h of strike
    Initial,
    /// Follow-up assessment 24-72h post-strike
    Interim,
    /// Final assessment after 72h or when complete
    Final,
}

/// Effect level categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectLevel {
    /// Immediate physical damage to target
    FirstOrder,
    /// Operational/systemic impacts (supply chain, morale, etc.)
    SecondOrder,
    /// Strategic/behavioral impacts (policy changes, negotiations, etc.)
    ThirdOrder,
}

/// Recommendation categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Recommendation {
    /// Desired effect achieved, no further action
    EffectAchieved,
    /// Continue monitoring, assess again later
    Monitor,
    /// Re-attack with same weaponeering
    ReAttack,
    /// Re-attack with different munitions/approach
    ReWeaponeer,
}

impl fmt::Display for Recommendation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Recommendation::EffectAchieved => write!(f, "Effect Achieved"),
            Recommendation::Monitor => write!(f, "Monitor"),
            Recommendation::ReAttack => write!(f, "Re-attack"),
            Recommendation::ReWeaponeer => write!(f, "Re-weaponeer"),
        }
    }
}

/// CIVCAS credibility levels per CJCSI 3160.01
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CivcasCredibility {
    /// No evidence of civilian casualties
    NoCredibility,
    /// Allegation exists but unconfirmed
    Possible,
    /// Evidence suggests casualties likely occurred
    Credible,
    /// Definitive proof of civilian casualties
    Confirmed,
}

/// Weapon performance relative to predictions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WeaponPerformance {
    /// Exceeded predictions
    Exceeded,
    /// Met predictions
    Met,
    /// Below predictions
    Below,
    /// Failed to function
    Failed,
}

/// BDA report status (workflow states)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BdaStatus {
    /// Draft, not yet submitted
    Draft,
    /// Submitted for review
    Submitted,
    /// Under review
    Reviewed,
    /// Approved by supervisor
    Approved,
    /// Rejected, needs rework
    Rejected,
}

/// Assessment quality level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AssessmentQuality {
    High,
    Medium,
    Low,
}

// ============================================================================
// STRUCTS: Core domain entities
// ============================================================================

/// BDA Report - Core domain entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BdaReport {
    pub id: String,
    
    // Target & Strike Association
    pub target_id: String,
    pub strike_id: Option<String>,
    
    // Assessment Metadata
    pub assessment_date: String,  // ISO 8601 timestamp
    pub analyst_id: String,
    pub assessment_type: AssessmentType,
    
    // Physical Damage Assessment
    pub physical_damage: PhysicalDamage,
    pub physical_damage_percentage: Option<i32>,
    pub damage_description: Option<String>,
    
    // Functional Damage Assessment
    pub functional_damage: FunctionalDamage,
    pub estimated_repair_time_hours: Option<i32>,
    pub pre_strike_capability_baseline: Option<String>,
    
    // Effects Assessment
    pub desired_effect: String,
    pub achieved_effect: String,
    pub effect_level: Option<EffectLevel>,
    pub unintended_effects: Option<String>,
    
    // Confidence & Quality
    pub confidence_level: f32,  // 0.0 to 1.0
    pub assessment_quality: Option<AssessmentQuality>,
    pub limiting_factors: Option<String>,
    
    // Recommendation
    pub recommendation: Recommendation,
    pub re_attack_priority: Option<i32>,
    pub re_attack_rationale: Option<String>,
    pub alternative_munitions: Option<String>,
    
    // Collateral Damage Assessment
    pub collateral_damage_detected: bool,
    pub civcas_credibility: Option<CivcasCredibility>,
    pub civilian_casualties_estimate: Option<i32>,
    pub protected_structures_damaged: Option<String>,
    pub cde_vs_actual_comparison: Option<String>,
    
    // Weaponeering Validation
    pub weapon_performance_vs_predicted: Option<WeaponPerformance>,
    pub munition_reliability: Option<String>,
    pub circular_error_probable_meters: Option<f32>,
    pub penetration_depth_meters: Option<f32>,
    
    // Approval Workflow
    pub status: BdaStatus,
    pub submitted_at: Option<String>,
    pub reviewed_by: Option<String>,
    pub reviewed_at: Option<String>,
    pub review_comments: Option<String>,
    pub approved_by: Option<String>,
    pub approved_at: Option<String>,
    
    // Classification & Security
    pub classification_level: String,
    pub handling_caveats: Option<String>,
    
    // Timestamps
    pub created_at: String,
    pub updated_at: String,
    
    // Notes
    pub notes: Option<String>,
}

/// Request to create a new BDA report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBdaReportRequest {
    pub target_id: String,
    pub strike_id: Option<String>,
    pub assessment_type: AssessmentType,
    
    pub physical_damage: PhysicalDamage,
    pub physical_damage_percentage: Option<i32>,
    pub damage_description: Option<String>,
    
    pub functional_damage: FunctionalDamage,
    pub estimated_repair_time_hours: Option<i32>,
    pub pre_strike_capability_baseline: Option<String>,
    
    pub desired_effect: String,
    pub achieved_effect: String,
    pub effect_level: Option<EffectLevel>,
    pub unintended_effects: Option<String>,
    
    pub confidence_level: f32,
    pub assessment_quality: Option<AssessmentQuality>,
    pub limiting_factors: Option<String>,
    
    pub recommendation: Recommendation,
    pub re_attack_priority: Option<i32>,
    pub re_attack_rationale: Option<String>,
    pub alternative_munitions: Option<String>,
    
    pub collateral_damage_detected: bool,
    pub civcas_credibility: Option<CivcasCredibility>,
    pub civilian_casualties_estimate: Option<i32>,
    pub protected_structures_damaged: Option<String>,
    pub cde_vs_actual_comparison: Option<String>,
    
    pub weapon_performance_vs_predicted: Option<WeaponPerformance>,
    pub munition_reliability: Option<String>,
    pub circular_error_probable_meters: Option<f32>,
    pub penetration_depth_meters: Option<f32>,
    
    pub classification_level: String,
    pub handling_caveats: Option<String>,
    
    pub notes: Option<String>,
}

/// Request to update an existing BDA report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateBdaReportRequest {
    pub physical_damage: Option<PhysicalDamage>,
    pub physical_damage_percentage: Option<i32>,
    pub damage_description: Option<String>,
    
    pub functional_damage: Option<FunctionalDamage>,
    pub estimated_repair_time_hours: Option<i32>,
    pub pre_strike_capability_baseline: Option<String>,
    
    pub desired_effect: Option<String>,
    pub achieved_effect: Option<String>,
    pub effect_level: Option<EffectLevel>,
    pub unintended_effects: Option<String>,
    
    pub confidence_level: Option<f32>,
    pub assessment_quality: Option<AssessmentQuality>,
    pub limiting_factors: Option<String>,
    
    pub recommendation: Option<Recommendation>,
    pub re_attack_priority: Option<i32>,
    pub re_attack_rationale: Option<String>,
    pub alternative_munitions: Option<String>,
    
    pub collateral_damage_detected: Option<bool>,
    pub civcas_credibility: Option<CivcasCredibility>,
    pub civilian_casualties_estimate: Option<i32>,
    pub protected_structures_damaged: Option<String>,
    pub cde_vs_actual_comparison: Option<String>,
    
    pub weapon_performance_vs_predicted: Option<WeaponPerformance>,
    pub munition_reliability: Option<String>,
    pub circular_error_probable_meters: Option<f32>,
    pub penetration_depth_meters: Option<f32>,
    
    pub notes: Option<String>,
}

/// BDA statistics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BdaStatistics {
    pub total_reports: i64,
    pub by_status: BdaStatusCounts,
    pub by_recommendation: BdaRecommendationCounts,
    pub by_physical_damage: BdaPhysicalDamageCounts,
    pub average_confidence: f32,
    pub collateral_damage_incidents: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BdaStatusCounts {
    pub draft: i64,
    pub submitted: i64,
    pub reviewed: i64,
    pub approved: i64,
    pub rejected: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BdaRecommendationCounts {
    pub effect_achieved: i64,
    pub monitor: i64,
    pub re_attack: i64,
    pub re_weaponeer: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BdaPhysicalDamageCounts {
    pub nd: i64,
    pub sd: i64,
    pub md: i64,
    pub svd: i64,
    pub d: i64,
}

/// Re-attack Recommendation View Model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReAttackRecommendation {
    pub bda_report_id: String,
    pub target_id: String,
    pub recommendation: Recommendation,
    pub re_attack_priority: Option<i32>,
    pub re_attack_rationale: Option<String>,
    pub physical_damage: Option<PhysicalDamage>,
    pub functional_damage: Option<FunctionalDamage>,
    pub confidence_level: f32,
    pub assessment_date: String,
}

// ============================================================================
// BUSINESS LOGIC: Validation and rules
// ============================================================================

impl BdaReport {
    /// Validate confidence level is between 0.0 and 1.0
    pub fn validate_confidence(&self) -> Result<(), String> {
        if self.confidence_level < 0.0 || self.confidence_level > 1.0 {
            return Err(format!(
                "Confidence level must be between 0.0 and 1.0, got {}",
                self.confidence_level
            ));
        }
        Ok(())
    }
    
    /// Validate physical damage percentage if provided
    pub fn validate_damage_percentage(&self) -> Result<(), String> {
        if let Some(pct) = self.physical_damage_percentage {
            if pct < 0 || pct > 100 {
                return Err(format!(
                    "Physical damage percentage must be between 0 and 100, got {}",
                    pct
                ));
            }
        }
        Ok(())
    }
    
    /// Validate re-attack priority if provided
    pub fn validate_reattack_priority(&self) -> Result<(), String> {
        if let Some(priority) = self.re_attack_priority {
            if priority < 1 || priority > 5 {
                return Err(format!(
                    "Re-attack priority must be between 1 and 5, got {}",
                    priority
                ));
            }
        }
        Ok(())
    }
    
    /// Check if report is in a terminal state (approved or rejected)
    pub fn is_terminal_status(&self) -> bool {
        matches!(self.status, BdaStatus::Approved | BdaStatus::Rejected)
    }
    
    /// Check if report can be transitioned to submitted status
    pub fn can_submit(&self) -> bool {
        self.status == BdaStatus::Draft
    }
    
    /// Check if report can be approved
    pub fn can_approve(&self) -> bool {
        matches!(self.status, BdaStatus::Submitted | BdaStatus::Reviewed)
    }
    
    /// Check if re-attack is recommended
    pub fn requires_reattack(&self) -> bool {
        matches!(self.recommendation, Recommendation::ReAttack | Recommendation::ReWeaponeer)
    }
    
    /// Get effect achievement score (0.0 = failed, 1.0 = fully achieved)
    pub fn get_effect_achievement_score(&self) -> f32 {
        match self.recommendation {
            Recommendation::EffectAchieved => 1.0,
            Recommendation::Monitor => 0.7,
            Recommendation::ReAttack => 0.3,
            Recommendation::ReWeaponeer => 0.2,
        }
    }
}

impl CreateBdaReportRequest {
    /// Validate all fields in create request
    pub fn validate(&self) -> Result<(), String> {
        if self.confidence_level < 0.0 || self.confidence_level > 1.0 {
            return Err("Confidence level must be between 0.0 and 1.0".to_string());
        }
        
        if let Some(pct) = self.physical_damage_percentage {
            if pct < 0 || pct > 100 {
                return Err("Physical damage percentage must be between 0 and 100".to_string());
            }
        }
        
        if let Some(priority) = self.re_attack_priority {
            if priority < 1 || priority > 5 {
                return Err("Re-attack priority must be between 1 and 5".to_string());
            }
        }
        
        if self.target_id.is_empty() {
            return Err("Target ID is required".to_string());
        }
        
        if self.desired_effect.is_empty() {
            return Err("Desired effect is required".to_string());
        }
        
        if self.achieved_effect.is_empty() {
            return Err("Achieved effect is required".to_string());
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
    
    fn create_test_report() -> BdaReport {
        BdaReport {
            id: "test-123".to_string(),
            target_id: "target-456".to_string(),
            strike_id: Some("strike-789".to_string()),
            assessment_date: "2026-01-21T14:00:00Z".to_string(),
            analyst_id: "analyst-001".to_string(),
            assessment_type: AssessmentType::Initial,
            physical_damage: PhysicalDamage::SVD,
            physical_damage_percentage: Some(75),
            damage_description: Some("Severe structural damage".to_string()),
            functional_damage: FunctionalDamage::PMC,
            estimated_repair_time_hours: Some(48),
            pre_strike_capability_baseline: Some("Fully operational".to_string()),
            desired_effect: "Destroy radar capability".to_string(),
            achieved_effect: "Radar disabled but salvageable".to_string(),
            effect_level: Some(EffectLevel::FirstOrder),
            unintended_effects: None,
            confidence_level: 0.85,
            assessment_quality: Some(AssessmentQuality::High),
            limiting_factors: None,
            recommendation: Recommendation::Monitor,
            re_attack_priority: None,
            re_attack_rationale: None,
            alternative_munitions: None,
            collateral_damage_detected: false,
            civcas_credibility: None,
            civilian_casualties_estimate: None,
            protected_structures_damaged: None,
            cde_vs_actual_comparison: None,
            weapon_performance_vs_predicted: Some(WeaponPerformance::Met),
            munition_reliability: Some("Nominal".to_string()),
            circular_error_probable_meters: Some(5.2),
            penetration_depth_meters: None,
            status: BdaStatus::Draft,
            submitted_at: None,
            reviewed_by: None,
            reviewed_at: None,
            review_comments: None,
            approved_by: None,
            approved_at: None,
            classification_level: "SECRET".to_string(),
            handling_caveats: None,
            created_at: "2026-01-21T14:00:00Z".to_string(),
            updated_at: "2026-01-21T14:00:00Z".to_string(),
            notes: None,
        }
    }
    
    #[test]
    fn test_confidence_validation() {
        let mut report = create_test_report();
        assert!(report.validate_confidence().is_ok());
        
        report.confidence_level = 1.5;
        assert!(report.validate_confidence().is_err());
        
        report.confidence_level = -0.1;
        assert!(report.validate_confidence().is_err());
    }
    
    #[test]
    fn test_damage_percentage_validation() {
        let mut report = create_test_report();
        assert!(report.validate_damage_percentage().is_ok());
        
        report.physical_damage_percentage = Some(150);
        assert!(report.validate_damage_percentage().is_err());
        
        report.physical_damage_percentage = Some(-10);
        assert!(report.validate_damage_percentage().is_err());
    }
    
    #[test]
    fn test_reattack_priority_validation() {
        let mut report = create_test_report();
        report.re_attack_priority = Some(3);
        assert!(report.validate_reattack_priority().is_ok());
        
        report.re_attack_priority = Some(6);
        assert!(report.validate_reattack_priority().is_err());
        
        report.re_attack_priority = Some(0);
        assert!(report.validate_reattack_priority().is_err());
    }
    
    #[test]
    fn test_status_transitions() {
        let mut report = create_test_report();
        
        report.status = BdaStatus::Draft;
        assert!(report.can_submit());
        assert!(!report.can_approve());
        assert!(!report.is_terminal_status());
        
        report.status = BdaStatus::Submitted;
        assert!(!report.can_submit());
        assert!(report.can_approve());
        
        report.status = BdaStatus::Approved;
        assert!(report.is_terminal_status());
        assert!(!report.can_approve());
    }
    
    #[test]
    fn test_requires_reattack() {
        let mut report = create_test_report();
        
        report.recommendation = Recommendation::EffectAchieved;
        assert!(!report.requires_reattack());
        
        report.recommendation = Recommendation::ReAttack;
        assert!(report.requires_reattack());
        
        report.recommendation = Recommendation::ReWeaponeer;
        assert!(report.requires_reattack());
    }
    
    #[test]
    fn test_effect_achievement_score() {
        let mut report = create_test_report();
        
        report.recommendation = Recommendation::EffectAchieved;
        assert_eq!(report.get_effect_achievement_score(), 1.0);
        
        report.recommendation = Recommendation::Monitor;
        assert_eq!(report.get_effect_achievement_score(), 0.7);
        
        report.recommendation = Recommendation::ReAttack;
        assert_eq!(report.get_effect_achievement_score(), 0.3);
    }
    
    #[test]
    fn test_create_request_validation() {
        let request = CreateBdaReportRequest {
            target_id: "target-123".to_string(),
            strike_id: None,
            assessment_type: AssessmentType::Initial,
            physical_damage: PhysicalDamage::MD,
            physical_damage_percentage: Some(40),
            damage_description: None,
            functional_damage: FunctionalDamage::PMC,
            estimated_repair_time_hours: None,
            pre_strike_capability_baseline: None,
            desired_effect: "Neutralize communications".to_string(),
            achieved_effect: "Partially successful".to_string(),
            effect_level: Some(EffectLevel::FirstOrder),
            unintended_effects: None,
            confidence_level: 0.75,
            assessment_quality: Some(AssessmentQuality::Medium),
            limiting_factors: None,
            recommendation: Recommendation::Monitor,
            re_attack_priority: None,
            re_attack_rationale: None,
            alternative_munitions: None,
            collateral_damage_detected: false,
            civcas_credibility: None,
            civilian_casualties_estimate: None,
            protected_structures_damaged: None,
            cde_vs_actual_comparison: None,
            weapon_performance_vs_predicted: None,
            munition_reliability: None,
            circular_error_probable_meters: None,
            penetration_depth_meters: None,
            classification_level: "SECRET".to_string(),
            handling_caveats: None,
            notes: None,
        };
        
        assert!(request.validate().is_ok());
    }
}

use crate::features::bda::domain::{
    bda_report::BdaReport,
    report_template::{GenerateReportRequest, ReportClassification},
};
use serde_json::json;

pub fn generate_json(
    report: &BdaReport,
    request: &GenerateReportRequest,
) -> Result<serde_json::Value, String> {
    let classification = request.classification
        .unwrap_or(ReportClassification::Secret);
    
    let mut report_data = json!({
        "report_id": report.id,
        "template_type": format!("{:?}", request.template_type),
        "classification": classification.marking(),
        "generated_at": chrono::Utc::now().to_rfc3339(),
        
        // Report metadata
        "metadata": {
            "target_id": report.target_id,
            "strike_id": report.strike_id,
            "assessment_date": report.assessment_date,
            "assessment_type": format!("{:?}", report.assessment_type),
            "analyst_id": report.analyst_id,
        },
        
        // Physical damage assessment
        "physical_damage": {
            "category": format!("{:?}", report.physical_damage),
            "percentage": report.physical_damage_percentage,
            "description": report.damage_description,
        },
        
        // Functional damage assessment
        "functional_damage": {
            "status": format!("{:?}", report.functional_damage),
            "repair_time_hours": report.estimated_repair_time_hours,
            "pre_strike_baseline": report.pre_strike_capability_baseline,
        },
        
        // Effects assessment
        "effects": {
            "desired": report.desired_effect,
            "achieved": report.achieved_effect,
            "level": report.effect_level.map(|e| format!("{:?}", e)),
            "unintended": report.unintended_effects,
        },
        
        // Confidence and quality
        "confidence": {
            "level": report.confidence_level,
            "quality": report.assessment_quality.map(|q| format!("{:?}", q)),
            "limiting_factors": report.limiting_factors,
        },
        
        // Recommendation
        "recommendation": {
            "type": format!("{:?}", report.recommendation),
            "re_attack_priority": report.re_attack_priority,
            "re_attack_rationale": report.re_attack_rationale,
            "alternative_munitions": report.alternative_munitions,
        },
        
        // Collateral damage
        "collateral_damage": {
            "detected": report.collateral_damage_detected,
            "civcas_credibility": report.civcas_credibility.map(|c| format!("{:?}", c)),
            "civilian_casualties_estimate": report.civilian_casualties_estimate,
            "protected_structures_damaged": report.protected_structures_damaged,
            "cde_vs_actual": report.cde_vs_actual_comparison,
        },
        
        // Weaponeering validation
        "weaponeering": {
            "weapon_performance": report.weapon_performance_vs_predicted.map(|w| format!("{:?}", w)),
            "munition_reliability": report.munition_reliability,
            "cep_meters": report.circular_error_probable_meters,
            "penetration_depth_meters": report.penetration_depth_meters,
        },
        
        // Status
        "status": format!("{:?}", report.status),
        "submitted_at": report.submitted_at,
        "approved_at": report.approved_at,
    });
    
    // Add optional sections (placeholders for now as per original code)
    if request.include_imagery {
        report_data["imagery"] = json!([]);
    }
    
    if request.include_components {
        report_data["components"] = json!([]);
    }
    
    if request.include_peer_reviews {
        report_data["peer_reviews"] = json!([]);
    }
    
    if request.include_history {
        report_data["history"] = json!([]);
    }
    
    Ok(report_data)
}

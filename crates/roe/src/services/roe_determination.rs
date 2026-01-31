// ROE Determination Service
// Purpose: Automatically determine if a decision requires ROE based on its characteristics

use crate::features::roe::domain::ROEStatus;

/// Decision information needed for ROE determination
/// This is a simplified view - in production, this would come from a decisions feature module
#[derive(Debug, Clone)]
pub struct DecisionInfo {
    pub id: String,
    pub title: String,
    pub description: String,
    pub category: String,
}

/// ROE Determination Service
/// Analyzes decision characteristics to determine if ROE authorization is required
pub struct ROEDeterminationService;

impl ROEDeterminationService {
    /// Determine ROE status for a decision based on its characteristics
    pub fn determine_roe_status(decision: &DecisionInfo) -> ROEStatus {
        // Check decision category and content
        if Self::is_strike_decision(decision) {
            return Self::check_strike_roe(decision);
        }
        
        if Self::is_cross_border_decision(decision) {
            return ROEStatus::RequiresRoeRelease;
        }
        
        if Self::uses_restricted_weapons(decision) {
            return ROEStatus::RequiresRoeRelease;
        }
        
        if Self::targets_dual_use_infrastructure(decision) {
            return ROEStatus::RequiresRoeRelease;
        }
        
        if Self::near_civilian_areas(decision) {
            return ROEStatus::RequiresRoeRelease;
        }
        
        // Default: within approved ROE
        ROEStatus::WithinApprovedRoe
    }
    
    /// Generate ROE notes explaining why ROE is required (or that it's within approved ROE)
    pub fn generate_roe_notes(decision: &DecisionInfo, roe_status: ROEStatus) -> Option<String> {
        match roe_status {
            ROEStatus::RequiresRoeRelease => {
                let mut reasons = Vec::new();
                
                if Self::near_civilian_areas(decision) {
                    reasons.push("Target near civilian infrastructure");
                }
                if Self::uses_restricted_weapons(decision) {
                    reasons.push("Involves restricted weapon types");
                }
                if Self::targets_dual_use_infrastructure(decision) {
                    reasons.push("Targets dual-use infrastructure");
                }
                if Self::is_cross_border_decision(decision) {
                    reasons.push("Cross-border operation");
                }
                if Self::is_strike_decision(decision) && Self::near_civilian_areas(decision) {
                    reasons.push("Strike operation near civilian areas");
                }
                
                if reasons.is_empty() {
                    Some("Requires new ROE authorization based on decision characteristics".to_string())
                } else {
                    Some(format!("Requires new ROE authorization: {}", reasons.join(", ")))
                }
            }
            ROEStatus::WithinApprovedRoe => {
                Some("Decision falls within approved ROE (ROE-2024-03)".to_string())
            }
            _ => None
        }
    }
    
    /// Check if decision is a strike operation
    fn is_strike_decision(decision: &DecisionInfo) -> bool {
        decision.category.to_lowercase() == "strike" || 
        decision.title.to_lowercase().contains("strike") ||
        decision.description.to_lowercase().contains("strike")
    }
    
    /// Check ROE requirements for strike decisions
    fn check_strike_roe(decision: &DecisionInfo) -> ROEStatus {
        // Check for civilian proximity
        if Self::near_civilian_areas(decision) {
            return ROEStatus::RequiresRoeRelease;
        }
        
        // Check for restricted munitions
        if Self::uses_restricted_weapons(decision) {
            return ROEStatus::RequiresRoeRelease;
        }
        
        // Check target type
        if Self::targets_dual_use_infrastructure(decision) {
            return ROEStatus::RequiresRoeRelease;
        }
        
        // Standard strike within ROE
        ROEStatus::WithinApprovedRoe
    }
    
    /// Check if decision involves cross-border operations
    fn is_cross_border_decision(decision: &DecisionInfo) -> bool {
        let text = format!("{} {}", decision.title, decision.description).to_lowercase();
        
        text.contains("cross-border") ||
        text.contains("cross border") ||
        text.contains("border crossing") ||
        text.contains("international border") ||
        (text.contains("border") && (text.contains("operation") || text.contains("mission")))
    }
    
    /// Check if decision involves restricted weapon types
    fn uses_restricted_weapons(decision: &DecisionInfo) -> bool {
        let restricted_keywords = vec![
            "cluster",
            "incendiary",
            "chemical",
            "biological",
            "nuclear",
            "depleted uranium",
            "white phosphorus",
            "napalm"
        ];
        
        let text = format!("{} {}", decision.title, decision.description).to_lowercase();
        restricted_keywords.iter().any(|keyword| text.contains(keyword))
    }
    
    /// Check if decision targets dual-use infrastructure
    fn targets_dual_use_infrastructure(decision: &DecisionInfo) -> bool {
        let dual_use_keywords = vec![
            "power plant",
            "water treatment",
            "hospital",
            "school",
            "mosque",
            "church",
            "civilian infrastructure",
            "dual-use",
            "dual use",
            "civilian facility"
        ];
        
        let text = format!("{} {}", decision.title, decision.description).to_lowercase();
        dual_use_keywords.iter().any(|keyword| text.contains(keyword))
    }
    
    /// Check if decision involves operations near civilian areas
    fn near_civilian_areas(decision: &DecisionInfo) -> bool {
        let civilian_keywords = vec![
            "civilian",
            "residential",
            "urban",
            "population center",
            "near civilians",
            "proximity to civilians",
            "civilian area",
            "residential area",
            "urban area",
            "populated area"
        ];
        
        let text = format!("{} {}", decision.title, decision.description).to_lowercase();
        civilian_keywords.iter().any(|keyword| text.contains(keyword))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strike_near_civilians_requires_roe() {
        let decision = DecisionInfo {
            id: "test-1".to_string(),
            title: "Strike T-1002 Authorization".to_string(),
            description: "High-value enemy command post near civilian infrastructure".to_string(),
            category: "strike".to_string(),
        };

        let status = ROEDeterminationService::determine_roe_status(&decision);
        assert_eq!(status, ROEStatus::RequiresRoeRelease);
        
        let notes = ROEDeterminationService::generate_roe_notes(&decision, status);
        assert!(notes.is_some());
        assert!(notes.unwrap().contains("civilian"));
    }

    #[test]
    fn test_standard_maneuver_within_roe() {
        let decision = DecisionInfo {
            id: "test-2".to_string(),
            title: "Move 1 MECH BDE to Sector Beta".to_string(),
            description: "Reposition brigade to strengthen defensive posture".to_string(),
            category: "maneuver".to_string(),
        };

        let status = ROEDeterminationService::determine_roe_status(&decision);
        assert_eq!(status, ROEStatus::WithinApprovedRoe);
        
        let notes = ROEDeterminationService::generate_roe_notes(&decision, status);
        assert!(notes.is_some());
        assert!(notes.unwrap().contains("within approved ROE"));
    }

    #[test]
    fn test_cross_border_requires_roe() {
        let decision = DecisionInfo {
            id: "test-3".to_string(),
            title: "Border Reconnaissance Mission".to_string(),
            description: "Cross-border ISR operation to monitor enemy movements".to_string(),
            category: "intelligence".to_string(),
        };

        let status = ROEDeterminationService::determine_roe_status(&decision);
        assert_eq!(status, ROEStatus::RequiresRoeRelease);
    }

    #[test]
    fn test_restricted_weapons_requires_roe() {
        let decision = DecisionInfo {
            id: "test-4".to_string(),
            title: "Target Engagement".to_string(),
            description: "Engage target using cluster munitions".to_string(),
            category: "strike".to_string(),
        };

        let status = ROEDeterminationService::determine_roe_status(&decision);
        assert_eq!(status, ROEStatus::RequiresRoeRelease);
    }

    #[test]
    fn test_dual_use_infrastructure_requires_roe() {
        let decision = DecisionInfo {
            id: "test-5".to_string(),
            title: "Power Plant Strike".to_string(),
            description: "Target enemy power plant facility".to_string(),
            category: "strike".to_string(),
        };

        let status = ROEDeterminationService::determine_roe_status(&decision);
        assert_eq!(status, ROEStatus::RequiresRoeRelease);
    }

    #[test]
    fn test_strike_without_restrictions_within_roe() {
        let decision = DecisionInfo {
            id: "test-6".to_string(),
            title: "Strike Enemy C2 Facility".to_string(),
            description: "Engage enemy command and control node in open terrain".to_string(),
            category: "strike".to_string(),
        };

        let status = ROEDeterminationService::determine_roe_status(&decision);
        assert_eq!(status, ROEStatus::WithinApprovedRoe);
    }
}

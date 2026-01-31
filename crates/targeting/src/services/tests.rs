// Unit tests for business logic services
// F3EAD transitions, DTL scoring, TST enforcement

#[cfg(test)]
mod tests {
    use crate::features::targeting::services::*;

    // ============================================================================
    // F3EAD STAGE TRANSITION TESTS
    // ============================================================================

    #[test]
    fn test_f3ead_stage_from_str() {
        assert_eq!(F3EADStage::from_str("FIND"), Some(F3EADStage::Find));
        assert_eq!(F3EADStage::from_str("find"), Some(F3EADStage::Find));
        assert_eq!(F3EADStage::from_str("Find"), Some(F3EADStage::Find));
        assert_eq!(F3EADStage::from_str("FIX"), Some(F3EADStage::Fix));
        assert_eq!(F3EADStage::from_str("FINISH"), Some(F3EADStage::Finish));
        assert_eq!(F3EADStage::from_str("EXPLOIT"), Some(F3EADStage::Exploit));
        assert_eq!(F3EADStage::from_str("ANALYZE"), Some(F3EADStage::Analyze));
        assert_eq!(F3EADStage::from_str("DISSEMINATE"), Some(F3EADStage::Disseminate));
        assert_eq!(F3EADStage::from_str("INVALID"), None);
    }

    #[test]
    fn test_f3ead_stage_to_str() {
        assert_eq!(F3EADStage::Find.to_str(), "FIND");
        assert_eq!(F3EADStage::Fix.to_str(), "FIX");
        assert_eq!(F3EADStage::Finish.to_str(), "FINISH");
        assert_eq!(F3EADStage::Exploit.to_str(), "EXPLOIT");
        assert_eq!(F3EADStage::Analyze.to_str(), "ANALYZE");
        assert_eq!(F3EADStage::Disseminate.to_str(), "DISSEMINATE");
    }

    #[test]
    fn test_f3ead_stage_next() {
        assert_eq!(F3EADStage::Find.next(), Some(F3EADStage::Fix));
        assert_eq!(F3EADStage::Fix.next(), Some(F3EADStage::Finish));
        assert_eq!(F3EADStage::Finish.next(), Some(F3EADStage::Exploit));
        assert_eq!(F3EADStage::Exploit.next(), Some(F3EADStage::Analyze));
        assert_eq!(F3EADStage::Analyze.next(), Some(F3EADStage::Disseminate));
        assert_eq!(F3EADStage::Disseminate.next(), None);
    }

    #[test]
    fn test_f3ead_stage_can_transition_to() {
        assert!(F3EADStage::Find.can_transition_to("FIX"));
        assert!(F3EADStage::Fix.can_transition_to("FINISH"));
        assert!(F3EADStage::Finish.can_transition_to("EXPLOIT"));
        assert!(F3EADStage::Exploit.can_transition_to("ANALYZE"));
        assert!(F3EADStage::Analyze.can_transition_to("DISSEMINATE"));
        
        assert!(!F3EADStage::Find.can_transition_to("FINISH")); // Cannot skip
        assert!(!F3EADStage::Fix.can_transition_to("FIND")); // Cannot go backwards
        assert!(!F3EADStage::Disseminate.can_transition_to("FIND")); // Final stage
    }

    #[test]
    fn test_validate_f3ead_transition_valid() {
        assert!(validate_f3ead_transition("FIND", "FIX").is_ok());
        assert!(validate_f3ead_transition("FIX", "FINISH").is_ok());
        assert!(validate_f3ead_transition("FINISH", "EXPLOIT").is_ok());
        assert!(validate_f3ead_transition("EXPLOIT", "ANALYZE").is_ok());
        assert!(validate_f3ead_transition("ANALYZE", "DISSEMINATE").is_ok());
    }

    #[test]
    fn test_validate_f3ead_transition_invalid() {
        // Cannot skip stages
        assert!(validate_f3ead_transition("FIND", "FINISH").is_err());
        assert!(validate_f3ead_transition("FIX", "EXPLOIT").is_err());
        
        // Cannot go backwards
        assert!(validate_f3ead_transition("FIX", "FIND").is_err());
        assert!(validate_f3ead_transition("FINISH", "FIX").is_err());
        
        // Cannot transition from final stage
        assert!(validate_f3ead_transition("DISSEMINATE", "FIND").is_err());
        
        // Invalid stage names
        assert!(validate_f3ead_transition("INVALID", "FIX").is_err());
        assert!(validate_f3ead_transition("FIND", "INVALID").is_err());
    }

    // ============================================================================
    // DTL SCORING TESTS
    // ============================================================================

    #[test]
    fn test_calculate_combined_score() {
        // Priority weighted 60%, feasibility 40%
        let score = DtlScoring::calculate_combined_score(1.0, 1.0);
        assert_eq!(score, 1.0);
        
        let score = DtlScoring::calculate_combined_score(0.5, 0.5);
        assert_eq!(score, 0.5);
        
        let score = DtlScoring::calculate_combined_score(1.0, 0.0);
        assert_eq!(score, 0.6); // 1.0 * 0.6 + 0.0 * 0.4
        
        let score = DtlScoring::calculate_combined_score(0.0, 1.0);
        assert_eq!(score, 0.4); // 0.0 * 0.6 + 1.0 * 0.4
        
        let score = DtlScoring::calculate_combined_score(0.8, 0.6);
        assert_eq!(score, 0.72); // 0.8 * 0.6 + 0.6 * 0.4 = 0.48 + 0.24
    }

    #[test]
    fn test_calculate_priority_score() {
        // HPT (High Payoff Target) should have highest weight
        let score = DtlScoring::calculate_priority_score("HPT", "CRITICAL", 0);
        assert!(score > 0.9);
        
        // TST should be high
        let score = DtlScoring::calculate_priority_score("TST", "HIGH", 0);
        assert!(score > 0.7);
        
        // HVT should be medium-high
        let score = DtlScoring::calculate_priority_score("HVT", "MEDIUM", 0);
        assert!(score > 0.5);
        
        // Aging penalty: 1% per 24 hours, max 20%
        let score_fresh = DtlScoring::calculate_priority_score("HPT", "CRITICAL", 0);
        let score_24h = DtlScoring::calculate_priority_score("HPT", "CRITICAL", 24);
        let score_48h = DtlScoring::calculate_priority_score("HPT", "CRITICAL", 48);
        let score_480h = DtlScoring::calculate_priority_score("HPT", "CRITICAL", 480); // 20 days
        
        assert!(score_24h < score_fresh); // Should be lower
        assert!(score_48h < score_24h); // Should be even lower
        assert!(score_480h < score_48h); // Should be even lower (but capped at 20% reduction)
        
        // Verify max 20% reduction
        let reduction = (score_fresh - score_480h) / score_fresh;
        assert!(reduction <= 0.21); // Allow small floating point error
    }

    #[test]
    fn test_calculate_feasibility_score() {
        // Perfect conditions
        let score = DtlScoring::calculate_feasibility_score(1.0, 1.0, 1.0);
        assert_eq!(score, 1.0);
        
        // ISR weighted 40%
        let score = DtlScoring::calculate_feasibility_score(1.0, 0.0, 0.0);
        assert_eq!(score, 0.4);
        
        // Weather weighted 30%
        let score = DtlScoring::calculate_feasibility_score(0.0, 1.0, 0.0);
        assert_eq!(score, 0.3);
        
        // ROE weighted 30%
        let score = DtlScoring::calculate_feasibility_score(0.0, 0.0, 1.0);
        assert_eq!(score, 0.3);
        
        // Mixed conditions
        let score = DtlScoring::calculate_feasibility_score(0.8, 0.6, 0.4);
        assert_eq!(score, 0.62); // 0.8*0.4 + 0.6*0.3 + 0.4*0.3 = 0.32 + 0.18 + 0.12
    }

    #[test]
    fn test_calculate_aging_hours() {
        // Test SQLite datetime format
        let created_at = "2026-01-21 10:00:00";
        let hours = DtlScoring::calculate_aging_hours(created_at);
        assert!(hours >= 0); // Should be non-negative
        
        // Test ISO format
        let created_at = "2026-01-21T10:00:00Z";
        let hours = DtlScoring::calculate_aging_hours(created_at);
        assert!(hours >= 0);
        
        // Test invalid format (should return 0)
        let hours = DtlScoring::calculate_aging_hours("invalid");
        assert_eq!(hours, 0);
    }

    // ============================================================================
    // TST ENFORCEMENT TESTS
    // ============================================================================

    #[test]
    fn test_is_deadline_approaching() {
        use chrono::{Utc, Duration};
        
        // Deadline in 30 minutes (within 60 minute threshold)
        let deadline = (Utc::now() + Duration::minutes(30)).to_rfc3339();
        assert!(TstEnforcement::is_deadline_approaching(&deadline, 60));
        
        // Deadline in 90 minutes (outside 60 minute threshold)
        let deadline = (Utc::now() + Duration::minutes(90)).to_rfc3339();
        assert!(!TstEnforcement::is_deadline_approaching(&deadline, 60));
        
        // Deadline passed (should not be approaching)
        let deadline = (Utc::now() - Duration::minutes(30)).to_rfc3339();
        assert!(!TstEnforcement::is_deadline_approaching(&deadline, 60));
    }

    #[test]
    fn test_is_deadline_passed() {
        use chrono::{Utc, Duration};
        
        // Deadline in future (not passed)
        let deadline = (Utc::now() + Duration::hours(1)).to_rfc3339();
        assert!(!TstEnforcement::is_deadline_passed(&deadline));
        
        // Deadline in past (passed)
        let deadline = (Utc::now() - Duration::hours(1)).to_rfc3339();
        assert!(TstEnforcement::is_deadline_passed(&deadline));
    }

    #[test]
    fn test_minutes_remaining() {
        use chrono::{Utc, Duration};
        
        // Deadline in 30 minutes
        let deadline = (Utc::now() + Duration::minutes(30)).to_rfc3339();
        let remaining = TstEnforcement::minutes_remaining(&deadline);
        assert!(remaining.is_some());
        assert!(remaining.unwrap() >= 29 && remaining.unwrap() <= 31); // Allow 1 minute variance
        
        // Deadline passed (should be negative)
        let deadline = (Utc::now() - Duration::minutes(30)).to_rfc3339();
        let remaining = TstEnforcement::minutes_remaining(&deadline);
        assert!(remaining.is_some());
        assert!(remaining.unwrap() < 0);
        
        // Invalid format
        let remaining = TstEnforcement::minutes_remaining("invalid");
        assert!(remaining.is_none());
    }

    #[test]
    fn test_get_tst_priority() {
        // CRITICAL: < 30 minutes
        assert_eq!(TstEnforcement::get_tst_priority(29), "CRITICAL");
        assert_eq!(TstEnforcement::get_tst_priority(0), "CRITICAL");
        
        // HIGH: < 120 minutes (2 hours)
        assert_eq!(TstEnforcement::get_tst_priority(30), "HIGH");
        assert_eq!(TstEnforcement::get_tst_priority(119), "HIGH");
        
        // MEDIUM: < 360 minutes (6 hours)
        assert_eq!(TstEnforcement::get_tst_priority(120), "MEDIUM");
        assert_eq!(TstEnforcement::get_tst_priority(359), "MEDIUM");
        
        // LOW: >= 360 minutes
        assert_eq!(TstEnforcement::get_tst_priority(360), "LOW");
        assert_eq!(TstEnforcement::get_tst_priority(1000), "LOW");
    }
}

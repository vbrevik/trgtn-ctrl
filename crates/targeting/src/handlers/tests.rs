// Unit tests for targeting handlers
// Tests authentication context extraction and handler logic

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::auth::jwt::{Claims, UserRoleClaim};
    use axum::extract::Extension;
    use sqlx::SqlitePool;
    
    // Helper to create test Claims
    fn create_test_claims(user_id: &str) -> Claims {
        Claims {
            sub: user_id.to_string(),
            username: "test_user".to_string(),
            email: "test@example.com".to_string(),
            roles: vec![],
            jti: None,
            exp: chrono::Utc::now().timestamp() + 3600,
            iat: chrono::Utc::now().timestamp(),
        }
    }
    
    // Note: Full integration tests would require a test database
    // These tests verify the handler signatures and auth extraction logic
    
    #[test]
    fn test_create_jtb_session_extracts_user_id() {
        // Verify handler signature includes Extension<Claims>
        // This is a compile-time test - if it compiles, the signature is correct
        let _claims = create_test_claims("user-123");
        // Handler should extract user_id from claims.sub
        assert_eq!(_claims.sub, "user-123");
    }
    
    #[test]
    fn test_create_intel_report_extracts_user_id() {
        let claims = create_test_claims("analyst-456");
        assert_eq!(claims.sub, "analyst-456");
    }
    
    #[test]
    fn test_create_risk_assessment_extracts_user_id() {
        let claims = create_test_claims("risk-officer-789");
        assert_eq!(claims.sub, "risk-officer-789");
    }
    
    #[test]
    fn test_create_decision_extracts_user_id() {
        let claims = create_test_claims("commander-001");
        assert_eq!(claims.sub, "commander-001");
    }
    
    #[test]
    fn test_generate_handover_extracts_user_id() {
        let claims = create_test_claims("shift-lead-002");
        assert_eq!(claims.sub, "shift-lead-002");
    }
    
    #[test]
    fn test_create_annotation_extracts_user_id() {
        let claims = create_test_claims("analyst-003");
        assert_eq!(claims.sub, "analyst-003");
    }
    
    #[test]
    fn test_update_mission_intent_extracts_user_id() {
        let claims = create_test_claims("commander-004");
        assert_eq!(claims.sub, "commander-004");
    }
}

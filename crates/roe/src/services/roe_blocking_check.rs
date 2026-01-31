// ROE Blocking Check Service
// Purpose: Check if a decision can proceed based on ROE status
// This utility can be used by decision routing or other services that need to check ROE blocking

use crate::features::roe::domain::ROEStatus;
use sqlx::Pool;
use sqlx::sqlite::Sqlite;

/// Result of ROE blocking check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ROEBlockingResult {
    /// Whether the decision can proceed
    pub can_proceed: bool,
    /// Whether the decision is blocked by ROE
    pub is_blocked: bool,
    /// Whether ROE request is pending
    pub is_pending: bool,
    /// Current ROE status
    pub roe_status: Option<String>,
    /// Reason for blocking (if blocked)
    pub blocking_reason: Option<String>,
    /// Suggested action
    pub suggested_action: Option<String>,
}

impl ROEBlockingResult {
    /// Create a result indicating decision can proceed
    pub fn can_proceed(roe_status: Option<String>) -> Self {
        Self {
            can_proceed: true,
            is_blocked: false,
            is_pending: false,
            roe_status,
            blocking_reason: None,
            suggested_action: None,
        }
    }

    /// Create a result indicating decision is blocked
    pub fn blocked(roe_status: String, reason: String, action: String) -> Self {
        Self {
            can_proceed: false,
            is_blocked: true,
            is_pending: false,
            roe_status: Some(roe_status),
            blocking_reason: Some(reason),
            suggested_action: Some(action),
        }
    }

    /// Create a result indicating ROE request is pending
    pub fn pending(roe_status: String) -> Self {
        Self {
            can_proceed: false,
            is_blocked: false,
            is_pending: true,
            roe_status: Some(roe_status),
            blocking_reason: Some("ROE request pending approval".to_string()),
            suggested_action: Some("Wait for ROE approval before proceeding".to_string()),
        }
    }
}

use serde::{Deserialize, Serialize};

/// ROE Blocking Check Service
/// Provides utilities to check if decisions can proceed based on ROE status
pub struct ROEBlockingCheckService;

impl ROEBlockingCheckService {
    /// Check if a decision can proceed based on ROE status
    pub fn check_decision_can_proceed(roe_status: Option<&str>) -> ROEBlockingResult {
        match roe_status {
            None => {
                // No ROE status set - assume can proceed (legacy decisions)
                ROEBlockingResult::can_proceed(None)
            }
            Some(status_str) => {
                match ROEStatus::try_from(status_str) {
                    Ok(status) => {
                        if status.can_proceed() {
                            ROEBlockingResult::can_proceed(Some(status.to_string()))
                        } else if status.is_blocked() {
                            let (reason, action) = Self::get_blocking_details(&status);
                            ROEBlockingResult::blocked(
                                status.to_string(),
                                reason,
                                action,
                            )
                        } else if status.is_pending() {
                            ROEBlockingResult::pending(status.to_string())
                        } else {
                            // Unknown state - default to blocked for safety
                            ROEBlockingResult::blocked(
                                status.to_string(),
                                "Unknown ROE status - requires review".to_string(),
                                "Contact legal advisor for ROE status clarification".to_string(),
                            )
                        }
                    }
                    Err(_) => {
                        // Invalid status - default to blocked for safety
                        ROEBlockingResult::blocked(
                            status_str.to_string(),
                            "Invalid ROE status value".to_string(),
                            "Update decision with valid ROE status".to_string(),
                        )
                    }
                }
            }
        }
    }

    /// Check ROE status from database for a decision
    pub async fn check_decision_from_db(
        pool: &Pool<Sqlite>,
        decision_id: &str,
    ) -> Result<ROEBlockingResult, sqlx::Error> {
        use crate::features::roe::repositories::ROERepository;

        let repo = ROERepository::new(pool.clone());
        let (roe_status, _, _) = repo.get_decision_roe_status(decision_id).await?;

        Ok(Self::check_decision_can_proceed(roe_status.as_deref()))
    }

    /// Get blocking details for a specific ROE status
    fn get_blocking_details(status: &ROEStatus) -> (String, String) {
        match status {
            ROEStatus::RequiresRoeRelease => (
                "Decision requires new ROE authorization before it can proceed".to_string(),
                "Submit ROE release request through appropriate channels".to_string(),
            ),
            ROEStatus::RoeRejected => (
                "ROE request was rejected. Decision cannot proceed under current ROE".to_string(),
                "Review rejection reason and resubmit ROE request with modifications".to_string(),
            ),
            ROEStatus::RoePendingApproval => (
                "ROE request is pending approval".to_string(),
                "Wait for ROE approval before proceeding with decision".to_string(),
            ),
            _ => (
                "ROE status prevents decision from proceeding".to_string(),
                "Contact legal advisor for ROE status clarification".to_string(),
            ),
        }
    }

    /// Check if decision should be routed to a meeting
    /// Returns false if ROE blocks routing
    pub fn can_route_to_meeting(roe_status: Option<&str>) -> bool {
        let result = Self::check_decision_can_proceed(roe_status);
        result.can_proceed
    }

    /// Get routing reason if decision cannot be routed
    pub fn get_routing_block_reason(roe_status: Option<&str>) -> Option<String> {
        let result = Self::check_decision_can_proceed(roe_status);
        if result.is_blocked || result.is_pending {
            Some(format!(
                "{}. {}",
                result.blocking_reason.unwrap_or_default(),
                result.suggested_action.unwrap_or_default()
            ))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_proceed_within_approved_roe() {
        let result = ROEBlockingCheckService::check_decision_can_proceed(Some("within_approved_roe"));
        assert!(result.can_proceed);
        assert!(!result.is_blocked);
        assert!(!result.is_pending);
    }

    #[test]
    fn test_blocked_requires_roe_release() {
        let result = ROEBlockingCheckService::check_decision_can_proceed(Some("requires_roe_release"));
        assert!(!result.can_proceed);
        assert!(result.is_blocked);
        assert!(!result.is_pending);
        assert!(result.blocking_reason.is_some());
        assert!(result.suggested_action.is_some());
    }

    #[test]
    fn test_pending_roe_approval() {
        let result = ROEBlockingCheckService::check_decision_can_proceed(Some("roe_pending_approval"));
        assert!(!result.can_proceed);
        assert!(!result.is_blocked);
        assert!(result.is_pending);
    }

    #[test]
    fn test_blocked_roe_rejected() {
        let result = ROEBlockingCheckService::check_decision_can_proceed(Some("roe_rejected"));
        assert!(!result.can_proceed);
        assert!(result.is_blocked);
        assert!(!result.is_pending);
    }

    #[test]
    fn test_can_proceed_roe_approved() {
        let result = ROEBlockingCheckService::check_decision_can_proceed(Some("roe_approved"));
        assert!(result.can_proceed);
        assert!(!result.is_blocked);
    }

    #[test]
    fn test_no_roe_status_assumes_can_proceed() {
        let result = ROEBlockingCheckService::check_decision_can_proceed(None);
        assert!(result.can_proceed);
        assert!(!result.is_blocked);
    }

    #[test]
    fn test_can_route_to_meeting() {
        assert!(ROEBlockingCheckService::can_route_to_meeting(Some("within_approved_roe")));
        assert!(ROEBlockingCheckService::can_route_to_meeting(Some("roe_approved")));
        assert!(!ROEBlockingCheckService::can_route_to_meeting(Some("requires_roe_release")));
        assert!(!ROEBlockingCheckService::can_route_to_meeting(Some("roe_pending_approval")));
    }

    #[test]
    fn test_get_routing_block_reason() {
        let reason = ROEBlockingCheckService::get_routing_block_reason(Some("requires_roe_release"));
        assert!(reason.is_some());
        assert!(reason.unwrap().contains("ROE"));

        let reason = ROEBlockingCheckService::get_routing_block_reason(Some("within_approved_roe"));
        assert!(reason.is_none());
    }

    #[test]
    fn test_blocking_result_can_proceed() {
        let result = ROEBlockingResult::can_proceed(Some("within_approved_roe".to_string()));
        assert!(result.can_proceed);
        assert!(!result.is_blocked);
        assert!(!result.is_pending);
        assert_eq!(result.roe_status, Some("within_approved_roe".to_string()));
        assert!(result.blocking_reason.is_none());
        assert!(result.suggested_action.is_none());
    }

    #[test]
    fn test_blocking_result_blocked() {
        let result = ROEBlockingResult::blocked(
            "requires_roe_release".to_string(),
            "Test reason".to_string(),
            "Test action".to_string(),
        );
        assert!(!result.can_proceed);
        assert!(result.is_blocked);
        assert!(!result.is_pending);
        assert_eq!(result.roe_status, Some("requires_roe_release".to_string()));
        assert_eq!(result.blocking_reason, Some("Test reason".to_string()));
        assert_eq!(result.suggested_action, Some("Test action".to_string()));
    }

    #[test]
    fn test_blocking_result_pending() {
        let result = ROEBlockingResult::pending("roe_pending_approval".to_string());
        assert!(!result.can_proceed);
        assert!(!result.is_blocked);
        assert!(result.is_pending);
        assert_eq!(result.roe_status, Some("roe_pending_approval".to_string()));
        assert!(result.blocking_reason.is_some());
        assert!(result.suggested_action.is_some());
    }

    #[test]
    fn test_invalid_roe_status_handling() {
        let result = ROEBlockingCheckService::check_decision_can_proceed(Some("invalid_status"));
        assert!(!result.can_proceed);
        assert!(result.is_blocked);
        assert!(result.blocking_reason.is_some());
    }

    #[test]
    fn test_empty_roe_status() {
        let result = ROEBlockingCheckService::check_decision_can_proceed(Some(""));
        assert!(!result.can_proceed);
        assert!(result.is_blocked);
    }
}

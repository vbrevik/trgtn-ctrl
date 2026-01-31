// ROE (Rules of Engagement) Domain Model
// Purpose: Core domain entity for ROE status tracking and requests

use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// ENUMS: Domain value objects with validation
// ============================================================================

/// ROE status for decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ROEStatus {
    /// Decision can proceed under current ROE
    WithinApprovedRoe,
    /// Decision requires new ROE authorization
    RequiresRoeRelease,
    /// ROE release request submitted, awaiting approval
    RoePendingApproval,
    /// New ROE approved, can proceed
    RoeApproved,
    /// ROE request rejected, cannot proceed
    RoeRejected,
}

impl fmt::Display for ROEStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ROEStatus::WithinApprovedRoe => write!(f, "within_approved_roe"),
            ROEStatus::RequiresRoeRelease => write!(f, "requires_roe_release"),
            ROEStatus::RoePendingApproval => write!(f, "roe_pending_approval"),
            ROEStatus::RoeApproved => write!(f, "roe_approved"),
            ROEStatus::RoeRejected => write!(f, "roe_rejected"),
        }
    }
}

impl TryFrom<&str> for ROEStatus {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "within_approved_roe" => Ok(ROEStatus::WithinApprovedRoe),
            "requires_roe_release" => Ok(ROEStatus::RequiresRoeRelease),
            "roe_pending_approval" => Ok(ROEStatus::RoePendingApproval),
            "roe_approved" => Ok(ROEStatus::RoeApproved),
            "roe_rejected" => Ok(ROEStatus::RoeRejected),
            _ => Err(format!("Invalid ROE status: {}", value)),
        }
    }
}

/// ROE request status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ROERequestStatus {
    /// Request submitted, awaiting approval
    Pending,
    /// Request approved
    Approved,
    /// Request rejected
    Rejected,
    /// Request withdrawn
    Withdrawn,
}

impl fmt::Display for ROERequestStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ROERequestStatus::Pending => write!(f, "pending"),
            ROERequestStatus::Approved => write!(f, "approved"),
            ROERequestStatus::Rejected => write!(f, "rejected"),
            ROERequestStatus::Withdrawn => write!(f, "withdrawn"),
        }
    }
}

impl TryFrom<&str> for ROERequestStatus {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "pending" => Ok(ROERequestStatus::Pending),
            "approved" => Ok(ROERequestStatus::Approved),
            "rejected" => Ok(ROERequestStatus::Rejected),
            "withdrawn" => Ok(ROERequestStatus::Withdrawn),
            _ => Err(format!("Invalid ROE request status: {}", value)),
        }
    }
}

// ============================================================================
// STRUCTS: Domain entities
// ============================================================================

/// ROE Request entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ROERequest {
    pub id: String,
    pub decision_id: String,
    pub requested_by: String,
    pub requested_at: String,
    pub approval_authority: Option<String>,
    pub request_justification: String,
    pub status: ROERequestStatus,
    pub approved_by: Option<String>,
    pub approved_at: Option<String>,
    pub rejection_reason: Option<String>,
    pub roe_reference: Option<String>,
    pub expiration_date: Option<String>,
    pub conditions: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

/// Request to create a new ROE request
#[derive(Debug, Deserialize)]
pub struct CreateROERequestRequest {
    pub decision_id: String,
    pub approval_authority: Option<String>,
    pub request_justification: String,
    pub roe_reference: Option<String>,
    pub conditions: Option<String>,
}

/// Request to update ROE request status
#[derive(Debug, Deserialize)]
pub struct UpdateROERequestStatusRequest {
    pub status: String,
    pub approved_by: Option<String>,

    pub rejection_reason: Option<String>,
    pub roe_reference: Option<String>,
    pub expiration_date: Option<String>,
    pub conditions: Option<String>,
}

/// Request to update decision ROE status
#[derive(Debug, Deserialize)]
pub struct UpdateDecisionROEStatusRequest {
    pub roe_status: Option<String>,
    pub roe_notes: Option<String>,
    pub roe_request_id: Option<String>,
}

/// Response with ROE status for a decision
#[derive(Debug, Serialize)]
pub struct DecisionROEStatusResponse {
    pub decision_id: String,
    pub roe_status: Option<String>,
    pub roe_notes: Option<String>,
    pub roe_request: Option<ROERequest>,
}

// ============================================================================
// BUSINESS LOGIC
// ============================================================================

impl ROEStatus {
    /// Check if decision can proceed (ROE approved or within approved ROE)
    pub fn can_proceed(&self) -> bool {
        matches!(self, ROEStatus::WithinApprovedRoe | ROEStatus::RoeApproved)
    }

    /// Check if decision is blocked (requires ROE or ROE rejected)
    pub fn is_blocked(&self) -> bool {
        matches!(
            self,
            ROEStatus::RequiresRoeRelease | ROEStatus::RoeRejected
        )
    }

    /// Check if ROE request is pending
    pub fn is_pending(&self) -> bool {
        matches!(self, ROEStatus::RoePendingApproval)
    }
}

impl ROERequest {
    /// Create a new ROE request
    pub fn new(
        id: String,
        decision_id: String,
        requested_by: String,
        request_justification: String,
        approval_authority: Option<String>,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id,
            decision_id,
            requested_by,
            requested_at: now.clone(),
            approval_authority,
            request_justification,
            status: ROERequestStatus::Pending,
            approved_by: None,
            approved_at: None,
            rejection_reason: None,
            roe_reference: None,
            expiration_date: None,
            conditions: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    /// Approve the ROE request
    pub fn approve(
        &mut self,
        approved_by: String,
        roe_reference: Option<String>,
        expiration_date: Option<String>,
        conditions: Option<String>,
    ) {
        self.status = ROERequestStatus::Approved;
        self.approved_by = Some(approved_by);
        self.approved_at = Some(chrono::Utc::now().to_rfc3339());
        self.roe_reference = roe_reference;
        self.expiration_date = expiration_date;
        self.conditions = conditions;
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    /// Reject the ROE request
    pub fn reject(&mut self, approved_by: String, rejection_reason: String) {
        self.status = ROERequestStatus::Rejected;
        self.approved_by = Some(approved_by);
        self.approved_at = Some(chrono::Utc::now().to_rfc3339());
        self.rejection_reason = Some(rejection_reason);
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    /// Withdraw the ROE request
    pub fn withdraw(&mut self) {
        self.status = ROERequestStatus::Withdrawn;
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // ROEStatus Tests
    // ========================================================================

    #[test]
    fn test_roe_status_can_proceed() {
        assert!(ROEStatus::WithinApprovedRoe.can_proceed());
        assert!(ROEStatus::RoeApproved.can_proceed());
        assert!(!ROEStatus::RequiresRoeRelease.can_proceed());
        assert!(!ROEStatus::RoePendingApproval.can_proceed());
        assert!(!ROEStatus::RoeRejected.can_proceed());
    }

    #[test]
    fn test_roe_status_is_blocked() {
        assert!(ROEStatus::RequiresRoeRelease.is_blocked());
        assert!(ROEStatus::RoeRejected.is_blocked());
        assert!(!ROEStatus::WithinApprovedRoe.is_blocked());
        assert!(!ROEStatus::RoeApproved.is_blocked());
        assert!(!ROEStatus::RoePendingApproval.is_blocked());
    }

    #[test]
    fn test_roe_status_is_pending() {
        assert!(ROEStatus::RoePendingApproval.is_pending());
        assert!(!ROEStatus::WithinApprovedRoe.is_pending());
        assert!(!ROEStatus::RequiresRoeRelease.is_pending());
        assert!(!ROEStatus::RoeApproved.is_pending());
        assert!(!ROEStatus::RoeRejected.is_pending());
    }

    #[test]
    fn test_roe_status_display() {
        assert_eq!(ROEStatus::WithinApprovedRoe.to_string(), "within_approved_roe");
        assert_eq!(ROEStatus::RequiresRoeRelease.to_string(), "requires_roe_release");
        assert_eq!(ROEStatus::RoePendingApproval.to_string(), "roe_pending_approval");
        assert_eq!(ROEStatus::RoeApproved.to_string(), "roe_approved");
        assert_eq!(ROEStatus::RoeRejected.to_string(), "roe_rejected");
    }

    #[test]
    fn test_roe_status_try_from_valid() {
        assert_eq!(
            ROEStatus::try_from("within_approved_roe"),
            Ok(ROEStatus::WithinApprovedRoe)
        );
        assert_eq!(
            ROEStatus::try_from("requires_roe_release"),
            Ok(ROEStatus::RequiresRoeRelease)
        );
        assert_eq!(
            ROEStatus::try_from("roe_pending_approval"),
            Ok(ROEStatus::RoePendingApproval)
        );
        assert_eq!(
            ROEStatus::try_from("roe_approved"),
            Ok(ROEStatus::RoeApproved)
        );
        assert_eq!(
            ROEStatus::try_from("roe_rejected"),
            Ok(ROEStatus::RoeRejected)
        );
    }

    #[test]
    fn test_roe_status_try_from_invalid() {
        assert!(ROEStatus::try_from("invalid_status").is_err());
        assert!(ROEStatus::try_from("").is_err());
        assert!(ROEStatus::try_from("WITHIN_APPROVED_ROE").is_err()); // Wrong case
    }

    // ========================================================================
    // ROERequestStatus Tests
    // ========================================================================

    #[test]
    fn test_roe_request_status_display() {
        assert_eq!(ROERequestStatus::Pending.to_string(), "pending");
        assert_eq!(ROERequestStatus::Approved.to_string(), "approved");
        assert_eq!(ROERequestStatus::Rejected.to_string(), "rejected");
        assert_eq!(ROERequestStatus::Withdrawn.to_string(), "withdrawn");
    }

    #[test]
    fn test_roe_request_status_try_from_valid() {
        assert_eq!(
            ROERequestStatus::try_from("pending"),
            Ok(ROERequestStatus::Pending)
        );
        assert_eq!(
            ROERequestStatus::try_from("approved"),
            Ok(ROERequestStatus::Approved)
        );
        assert_eq!(
            ROERequestStatus::try_from("rejected"),
            Ok(ROERequestStatus::Rejected)
        );
        assert_eq!(
            ROERequestStatus::try_from("withdrawn"),
            Ok(ROERequestStatus::Withdrawn)
        );
    }

    #[test]
    fn test_roe_request_status_try_from_invalid() {
        assert!(ROERequestStatus::try_from("invalid").is_err());
        assert!(ROERequestStatus::try_from("").is_err());
        assert!(ROERequestStatus::try_from("PENDING").is_err()); // Wrong case
    }

    // ========================================================================
    // ROERequest Tests
    // ========================================================================

    #[test]
    fn test_roe_request_new() {
        let request = ROERequest::new(
            "req-1".to_string(),
            "decision-1".to_string(),
            "user-1".to_string(),
            "Test justification".to_string(),
            Some("Commander".to_string()),
        );

        assert_eq!(request.id, "req-1");
        assert_eq!(request.decision_id, "decision-1");
        assert_eq!(request.requested_by, "user-1");
        assert_eq!(request.request_justification, "Test justification");
        assert_eq!(request.approval_authority, Some("Commander".to_string()));
        assert_eq!(request.status, ROERequestStatus::Pending);
        assert!(request.approved_by.is_none());
        assert!(request.approved_at.is_none());
        assert!(request.rejection_reason.is_none());
        assert!(!request.requested_at.is_empty());
        assert!(!request.created_at.is_empty());
        assert!(!request.updated_at.is_empty());
    }

    #[test]
    fn test_roe_request_approve() {
        let mut request = ROERequest::new(
            "req-1".to_string(),
            "decision-1".to_string(),
            "user-1".to_string(),
            "Test justification".to_string(),
            None,
        );

        let before_updated = request.updated_at.clone();
        std::thread::sleep(std::time::Duration::from_millis(10)); // Ensure timestamp difference

        request.approve(
            "commander-1".to_string(),
            Some("ROE-2024-03".to_string()),
            Some("2024-12-31T23:59:59Z".to_string()),
            Some("No civilian casualties".to_string()),
        );

        assert_eq!(request.status, ROERequestStatus::Approved);
        assert_eq!(request.approved_by, Some("commander-1".to_string()));
        assert!(request.approved_at.is_some());
        assert_eq!(request.roe_reference, Some("ROE-2024-03".to_string()));
        assert_eq!(
            request.expiration_date,
            Some("2024-12-31T23:59:59Z".to_string())
        );
        assert_eq!(
            request.conditions,
            Some("No civilian casualties".to_string())
        );
        assert_ne!(request.updated_at, before_updated);
    }

    #[test]
    fn test_roe_request_reject() {
        let mut request = ROERequest::new(
            "req-1".to_string(),
            "decision-1".to_string(),
            "user-1".to_string(),
            "Test justification".to_string(),
            None,
        );

        let before_updated = request.updated_at.clone();
        std::thread::sleep(std::time::Duration::from_millis(10));

        request.reject(
            "commander-1".to_string(),
            "Insufficient justification".to_string(),
        );

        assert_eq!(request.status, ROERequestStatus::Rejected);
        assert_eq!(request.approved_by, Some("commander-1".to_string()));
        assert!(request.approved_at.is_some());
        assert_eq!(
            request.rejection_reason,
            Some("Insufficient justification".to_string())
        );
        assert_ne!(request.updated_at, before_updated);
    }

    #[test]
    fn test_roe_request_withdraw() {
        let mut request = ROERequest::new(
            "req-1".to_string(),
            "decision-1".to_string(),
            "user-1".to_string(),
            "Test justification".to_string(),
            None,
        );

        let before_updated = request.updated_at.clone();
        std::thread::sleep(std::time::Duration::from_millis(10));

        request.withdraw();

        assert_eq!(request.status, ROERequestStatus::Withdrawn);
        assert!(request.approved_by.is_none());
        assert!(request.approved_at.is_none());
        assert_ne!(request.updated_at, before_updated);
    }

    #[test]
    fn test_roe_request_approve_without_optional_fields() {
        let mut request = ROERequest::new(
            "req-1".to_string(),
            "decision-1".to_string(),
            "user-1".to_string(),
            "Test justification".to_string(),
            None,
        );

        request.approve("commander-1".to_string(), None, None, None);

        assert_eq!(request.status, ROERequestStatus::Approved);
        assert_eq!(request.approved_by, Some("commander-1".to_string()));
        assert!(request.approved_at.is_some());
        assert!(request.roe_reference.is_none());
        assert!(request.expiration_date.is_none());
        assert!(request.conditions.is_none());
    }

    #[test]
    fn test_roe_request_status_transitions() {
        // Pending -> Approved
        let mut request = ROERequest::new(
            "req-1".to_string(),
            "decision-1".to_string(),
            "user-1".to_string(),
            "Test".to_string(),
            None,
        );
        assert_eq!(request.status, ROERequestStatus::Pending);
        request.approve("commander".to_string(), None, None, None);
        assert_eq!(request.status, ROERequestStatus::Approved);

        // Pending -> Rejected
        let mut request = ROERequest::new(
            "req-2".to_string(),
            "decision-2".to_string(),
            "user-1".to_string(),
            "Test".to_string(),
            None,
        );
        request.reject("commander".to_string(), "Reason".to_string());
        assert_eq!(request.status, ROERequestStatus::Rejected);

        // Pending -> Withdrawn
        let mut request = ROERequest::new(
            "req-3".to_string(),
            "decision-3".to_string(),
            "user-1".to_string(),
            "Test".to_string(),
            None,
        );
        request.withdraw();
        assert_eq!(request.status, ROERequestStatus::Withdrawn);
    }
}

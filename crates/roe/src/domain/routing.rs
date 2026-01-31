// Decision Routing Domain Model
// Purpose: Routing plan for decisions with ROE integration

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Routing plan for a decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingPlan {
    /// Venue ID (if routed to a specific venue)
    pub venue_id: Option<String>,
    /// Venue name
    pub venue_name: String,
    /// Meeting instance ID (if routed to specific meeting)
    pub meeting_instance_id: Option<String>,
    /// Meeting date
    pub meeting_date: Option<String>,
    /// Meeting time
    pub meeting_time: Option<String>,
    /// Agenda order (position in meeting agenda)
    pub agenda_order: Option<i32>,
    /// Presenter name/role
    pub presenter: Option<String>,
    /// Estimated duration in minutes
    pub estimated_duration: Option<i32>,
    /// Routing reason/explanation
    pub routing_reason: Option<String>,
    /// When decision was routed
    pub routed_at: String,
    /// Whether decision can proceed to meeting (NEW: ROE integration)
    pub can_proceed: bool,
    /// Urgency override (e.g., "ROE_BLOCKER")
    pub urgency_override: Option<String>,
}

impl RoutingPlan {
    /// Create a routing plan that is blocked by ROE
    pub fn roe_blocked(roe_status: &str, blocking_reason: String) -> Self {
        Self {
            venue_id: None,
            venue_name: "ROE Coordination".to_string(),
            meeting_instance_id: None,
            meeting_date: None,
            meeting_time: None,
            agenda_order: None,
            presenter: None,
            estimated_duration: None,
            routing_reason: Some(format!(
                "Decision requires ROE authorization. Current status: {}. Cannot route to meeting until ROE approved. {}",
                roe_status, blocking_reason
            )),
            routed_at: Utc::now().to_rfc3339(),
            can_proceed: false,
            urgency_override: Some("ROE_BLOCKER".to_string()),
        }
    }

    /// Create a routing plan that is pending ROE approval
    pub fn roe_pending() -> Self {
        Self {
            venue_id: None,
            venue_name: "Awaiting ROE Approval".to_string(),
            meeting_instance_id: None,
            meeting_date: None,
            meeting_time: None,
            agenda_order: None,
            presenter: None,
            estimated_duration: None,
            routing_reason: Some(
                "ROE request pending approval. Decision on hold until ROE approved.".to_string()
            ),
            routed_at: Utc::now().to_rfc3339(),
            can_proceed: false,
            urgency_override: None,
        }
    }

    /// Create a routing plan that can proceed
    pub fn can_proceed(
        venue_name: String,
        meeting_date: Option<String>,
        meeting_time: Option<String>,
        routing_reason: Option<String>,
    ) -> Self {
        Self {
            venue_id: None,
            venue_name,
            meeting_instance_id: None,
            meeting_date,
            meeting_time,
            agenda_order: None,
            presenter: None,
            estimated_duration: None,
            routing_reason,
            routed_at: Utc::now().to_rfc3339(),
            can_proceed: true,
            urgency_override: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_routing_plan_roe_blocked() {
        let plan = RoutingPlan::roe_blocked(
            "requires_roe_release",
            "Test blocking reason".to_string(),
        );

        assert!(!plan.can_proceed);
        assert_eq!(plan.venue_name, "ROE Coordination");
        assert!(plan.venue_id.is_none());
        assert!(plan.meeting_instance_id.is_none());
        assert!(plan.meeting_date.is_none());
        assert!(plan.meeting_time.is_none());
        assert!(plan.routing_reason.is_some());
        assert!(plan.routing_reason.unwrap().contains("requires_roe_release"));
        assert_eq!(plan.urgency_override, Some("ROE_BLOCKER".to_string()));
    }

    #[test]
    fn test_routing_plan_roe_pending() {
        let plan = RoutingPlan::roe_pending();

        assert!(!plan.can_proceed);
        assert_eq!(plan.venue_name, "Awaiting ROE Approval");
        assert!(plan.venue_id.is_none());
        assert!(plan.meeting_instance_id.is_none());
        assert!(plan.meeting_date.is_none());
        assert!(plan.meeting_time.is_none());
        assert!(plan.routing_reason.is_some());
        assert!(plan.routing_reason.unwrap().contains("pending"));
        assert!(plan.urgency_override.is_none());
    }

    #[test]
    fn test_routing_plan_can_proceed() {
        let plan = RoutingPlan::can_proceed(
            "Test Venue".to_string(),
            Some("2026-01-23".to_string()),
            Some("14:00".to_string()),
            Some("Test reason".to_string()),
        );

        assert!(plan.can_proceed);
        assert_eq!(plan.venue_name, "Test Venue");
        assert_eq!(plan.meeting_date, Some("2026-01-23".to_string()));
        assert_eq!(plan.meeting_time, Some("14:00".to_string()));
        assert_eq!(plan.routing_reason, Some("Test reason".to_string()));
        assert!(plan.urgency_override.is_none());
    }

    #[test]
    fn test_routing_plan_can_proceed_minimal() {
        let plan = RoutingPlan::can_proceed(
            "Test Venue".to_string(),
            None,
            None,
            None,
        );

        assert!(plan.can_proceed);
        assert_eq!(plan.venue_name, "Test Venue");
        assert!(plan.meeting_date.is_none());
        assert!(plan.meeting_time.is_none());
        assert!(plan.routing_reason.is_none());
    }

    #[test]
    fn test_routing_plan_routed_at_timestamp() {
        let plan1 = RoutingPlan::roe_blocked("test", "reason".to_string());
        std::thread::sleep(std::time::Duration::from_millis(10));
        let plan2 = RoutingPlan::roe_pending();

        // Both should have valid timestamps
        assert!(!plan1.routed_at.is_empty());
        assert!(!plan2.routed_at.is_empty());
        // Timestamps should be different (or very close)
        assert_ne!(plan1.routed_at, plan2.routed_at);
    }
}

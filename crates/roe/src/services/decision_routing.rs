// Decision Routing Service with ROE Integration
// Purpose: Route decisions to meetings, blocking if ROE is required but not approved

use crate::features::roe::{
    domain::RoutingPlan,
    repositories::ROERepository,
    services::ROEBlockingCheckService,
};
use sqlx::Pool;
use sqlx::sqlite::Sqlite;
use chrono::{Timelike, Datelike};

/// Decision routing service with ROE integration
pub struct DecisionRoutingService {
    roe_repo: ROERepository,
}

impl DecisionRoutingService {
    /// Create a new decision routing service
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self {
            roe_repo: ROERepository::new(pool),
        }
    }

    /// Route a decision to the appropriate meeting, checking ROE first
    pub async fn route_decision(
        &self,
        decision_id: &str,
        decision_urgency: Option<&str>,
        decision_deadline: Option<&str>,
    ) -> Result<RoutingPlan, sqlx::Error> {
        // Check ROE blocking first using repository
        let (roe_status, _, _) = self.roe_repo.get_decision_roe_status(decision_id).await?;
        let roe_check = ROEBlockingCheckService::check_decision_can_proceed(roe_status.as_deref());

        // If blocked by ROE, return blocked routing plan
        if roe_check.is_blocked {
            let (roe_status, _, _) = self.roe_repo.get_decision_roe_status(decision_id).await?;
            let status_str = roe_status.unwrap_or_else(|| "unknown".to_string());
            
            return Ok(RoutingPlan::roe_blocked(
                &status_str,
                roe_check.blocking_reason.unwrap_or_default(),
            ));
        }

        // If pending ROE approval, return pending routing plan
        if roe_check.is_pending {
            return Ok(RoutingPlan::roe_pending());
        }

        // ROE approved or within approved ROE - proceed with normal routing
        self.route_by_urgency(decision_id, decision_urgency, decision_deadline).await
    }

    /// Route decision based on urgency and deadline (normal routing logic)
    async fn route_by_urgency(
        &self,
        _decision_id: &str,
        urgency: Option<&str>,
        deadline: Option<&str>,
    ) -> Result<RoutingPlan, sqlx::Error> {
        // Calculate hours to deadline
        let hours_to_deadline = if let Some(deadline_str) = deadline {
            Self::calculate_hours_to_deadline(deadline_str)
        } else {
            None
        };

        // Route based on urgency and timeline
        let plan = match (urgency, hours_to_deadline) {
            // Critical with < 6 hours: Immediate
            (Some("critical"), Some(hours)) if hours < 6 => {
                RoutingPlan::can_proceed(
                    "Ad-hoc (Immediate)".to_string(),
                    Some(chrono::Utc::now().date_naive().to_string()),
                    Some(chrono::Utc::now().time().format("%H:%M").to_string()),
                    Some(format!(
                        "Critical decision with {}h deadline requires immediate attention",
                        hours
                    )),
                )
            }
            // High with < 48 hours: Next daily brief
            (Some("high"), Some(hours)) if hours < 48 => {
                let (date, time) = Self::get_next_brief_time();
                RoutingPlan::can_proceed(
                    "Daily Brief".to_string(),
                    Some(date),
                    Some(time),
                    Some("High priority decision routed to next daily brief".to_string()),
                )
            }
            // Medium/High with < 168 hours (1 week): DRB
            (Some(urgency), Some(hours)) if hours < 168 && (urgency == "medium" || urgency == "high") => {
                let (date, time) = Self::get_next_drb_time();
                RoutingPlan::can_proceed(
                    "Decision Review Board (DRB)".to_string(),
                    Some(date),
                    Some(time),
                    Some("Operational decision routed to next DRB".to_string()),
                )
            }
            // Default: CAB (strategic)
            _ => {
                let (date, time) = Self::get_next_cab_time();
                RoutingPlan::can_proceed(
                    "Campaign Assessment Board (CAB)".to_string(),
                    Some(date),
                    Some(time),
                    Some("Strategic decision routed to next CAB".to_string()),
                )
            }
        };

        Ok(plan)
    }

    /// Calculate hours until deadline
    fn calculate_hours_to_deadline(deadline_str: &str) -> Option<i64> {
        match chrono::DateTime::parse_from_rfc3339(deadline_str) {
            Ok(deadline) => {
                let now = chrono::Utc::now();
                let duration = deadline.signed_duration_since(now);
                Some(duration.num_hours())
            }
            Err(_) => None,
        }
    }

    /// Get next daily brief time (morning or evening based on current time)
    fn get_next_brief_time() -> (String, String) {
        let now = chrono::Utc::now();
        let current_hour = now.hour();
        
        if current_hour < 12 {
            // Morning brief today at 0630
            (now.date_naive().to_string(), "06:30".to_string())
        } else {
            // Evening brief today at 1730, or tomorrow morning if past 1730
            if current_hour >= 17 {
                let tomorrow = now.date_naive() + chrono::Duration::days(1);
                (tomorrow.to_string(), "06:30".to_string())
            } else {
                (now.date_naive().to_string(), "17:30".to_string())
            }
        }
    }

    /// Get next DRB time (Wednesday 1400)
    fn get_next_drb_time() -> (String, String) {
        let mut date = chrono::Utc::now().date_naive();
        let weekday = date.weekday();
        
        // Calculate days until next Wednesday
        let days_until_wednesday = match weekday {
            chrono::Weekday::Wed => {
                // If today is Wednesday and before 1400, use today; otherwise next week
                if chrono::Utc::now().hour() < 14 {
                    0
                } else {
                    7
                }
            }
            chrono::Weekday::Thu => 6,
            chrono::Weekday::Fri => 5,
            chrono::Weekday::Sat => 4,
            chrono::Weekday::Sun => 3,
            chrono::Weekday::Mon => 2,
            chrono::Weekday::Tue => 1,
        };
        
        date = date + chrono::Duration::days(days_until_wednesday);
        (date.to_string(), "14:00".to_string())
    }

    /// Get next CAB time (Monday 0800)
    fn get_next_cab_time() -> (String, String) {
        let mut date = chrono::Utc::now().date_naive();
        let weekday = date.weekday();
        
        // Calculate days until next Monday
        let days_until_monday = match weekday {
            chrono::Weekday::Mon => {
                // If today is Monday and before 0800, use today; otherwise next week
                if chrono::Utc::now().hour() < 8 {
                    0
                } else {
                    7
                }
            }
            chrono::Weekday::Tue => 6,
            chrono::Weekday::Wed => 5,
            chrono::Weekday::Thu => 4,
            chrono::Weekday::Fri => 3,
            chrono::Weekday::Sat => 2,
            chrono::Weekday::Sun => 1,
        };
        
        date = date + chrono::Duration::days(days_until_monday);
        (date.to_string(), "08:00".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_hours_to_deadline() {
        let future = chrono::Utc::now() + chrono::Duration::hours(24);
        let deadline_str = future.to_rfc3339();
        let hours = DecisionRoutingService::calculate_hours_to_deadline(&deadline_str);
        assert!(hours.is_some());
        let hours_val = hours.unwrap();
        // Allow some tolerance for test execution time (23-25 hours)
        assert!(hours_val >= 23 && hours_val <= 25, "Expected hours between 23-25, got {}", hours_val);
    }

    #[test]
    fn test_get_next_brief_time() {
        let (date, time) = DecisionRoutingService::get_next_brief_time();
        assert!(!date.is_empty());
        assert!(time == "06:30" || time == "17:30");
    }

    #[test]
    fn test_get_next_drb_time() {
        let (date, time) = DecisionRoutingService::get_next_drb_time();
        assert!(!date.is_empty());
        assert_eq!(time, "14:00".to_string());
    }

    #[test]
    fn test_get_next_cab_time() {
        let (date, time) = DecisionRoutingService::get_next_cab_time();
        assert!(!date.is_empty());
        assert_eq!(time, "08:00".to_string());
    }

    #[test]
    fn test_calculate_hours_to_deadline_past() {
        let past = chrono::Utc::now() - chrono::Duration::hours(24);
        let deadline_str = past.to_rfc3339();
        let hours = DecisionRoutingService::calculate_hours_to_deadline(&deadline_str);
        assert!(hours.is_some());
        assert!(hours.unwrap() < 0); // Negative for past deadlines
    }

    #[test]
    fn test_calculate_hours_to_deadline_invalid() {
        let hours = DecisionRoutingService::calculate_hours_to_deadline("invalid-date");
        assert!(hours.is_none());
    }

    #[test]
    fn test_routing_plan_creation() {
        // Test that routing plans are created correctly
        let plan = RoutingPlan::can_proceed(
            "Test Venue".to_string(),
            Some("2026-01-23".to_string()),
            Some("14:00".to_string()),
            Some("Test".to_string()),
        );
        assert!(plan.can_proceed);
        assert_eq!(plan.venue_name, "Test Venue");
    }
}

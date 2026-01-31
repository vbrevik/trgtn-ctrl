// Business Logic Services
// F3EAD transitions, DTL scoring, TST enforcement, real-time updates

// Business logic services - no domain imports needed

pub mod realtime;
pub use realtime::RealtimeService;


// ============================================================================
// F3EAD STAGE TRANSITIONS
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum F3EADStage {
    Find,
    Fix,
    Finish,
    Exploit,
    Analyze,
    Disseminate,
}

impl F3EADStage {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "FIND" => Some(Self::Find),
            "FIX" => Some(Self::Fix),
            "FINISH" => Some(Self::Finish),
            "EXPLOIT" => Some(Self::Exploit),
            "ANALYZE" => Some(Self::Analyze),
            "DISSEMINATE" => Some(Self::Disseminate),
            _ => None,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Find => "FIND",
            Self::Fix => "FIX",
            Self::Finish => "FINISH",
            Self::Exploit => "EXPLOIT",
            Self::Analyze => "ANALYZE",
            Self::Disseminate => "DISSEMINATE",
        }
    }

    pub fn next(&self) -> Option<Self> {
        match self {
            Self::Find => Some(Self::Fix),
            Self::Fix => Some(Self::Finish),
            Self::Finish => Some(Self::Exploit),
            Self::Exploit => Some(Self::Analyze),
            Self::Analyze => Some(Self::Disseminate),
            Self::Disseminate => None, // Final stage
        }
    }

    pub fn can_transition_to(&self, target: &str) -> bool {
        if let Some(next) = self.next() {
            next.to_str() == target.to_uppercase()
        } else {
            false
        }
    }
}

pub fn validate_f3ead_transition(current: &str, new_stage: &str) -> Result<(), String> {
    let current_stage = F3EADStage::from_str(current)
        .ok_or_else(|| format!("Invalid current stage: {}", current))?;
    
    let _new_stage_enum = F3EADStage::from_str(new_stage)
        .ok_or_else(|| format!("Invalid new stage: {}", new_stage))?;

    if !current_stage.can_transition_to(new_stage) {
        return Err(format!(
            "Invalid transition: {} -> {}. Must follow F3EAD sequence: FIND -> FIX -> FINISH -> EXPLOIT -> ANALYZE -> DISSEMINATE",
            current, new_stage
        ));
    }

    Ok(())
}

// ============================================================================
// DTL SCORING ALGORITHMS
// ============================================================================

pub struct DtlScoring;

impl DtlScoring {
    /// Calculate combined score from priority and feasibility
    /// Formula: (priority_score * 0.6) + (feasibility_score * 0.4)
    /// Priority weighted higher as it reflects mission importance
    pub fn calculate_combined_score(priority: f64, feasibility: f64) -> f64 {
        (priority * 0.6) + (feasibility * 0.4)
    }

    /// Calculate priority score based on target characteristics
    /// Factors: target_type (HPT=1.0, HVT=0.8, TST=0.95), priority level, aging
    pub fn calculate_priority_score(
        target_type: &str,
        priority_level: &str,
        aging_hours: i32,
    ) -> f64 {
        let type_weight = match target_type.to_uppercase().as_str() {
            "HPT" => 1.0,  // High Payoff Target
            "TST" => 0.95, // Time-Sensitive Target
            "HVT" => 0.8,  // High Value Target
            _ => 0.5,
        };

        let priority_weight = match priority_level.to_uppercase().as_str() {
            "CRITICAL" => 1.0,
            "HIGH" => 0.8,
            "MEDIUM" => 0.6,
            "LOW" => 0.4,
            _ => 0.5,
        };

        // Aging penalty: reduce score by 1% per 24 hours (max 20% reduction)
        let aging_penalty = (aging_hours as f64 / 24.0 * 0.01).min(0.2);

        ((type_weight * 0.5) + (priority_weight * 0.5)) * (1.0 - aging_penalty)
    }

    /// Calculate feasibility score based on ISR coverage, weather, ROE
    /// Factors: ISR coverage (0.0-1.0), weather conditions (0.0-1.0), ROE clearance (0.0-1.0)
    pub fn calculate_feasibility_score(
        isr_coverage: f64,
        weather_conditions: f64,
        roe_clearance: f64,
    ) -> f64 {
        // Weighted average: ISR 40%, Weather 30%, ROE 30%
        (isr_coverage * 0.4) + (weather_conditions * 0.3) + (roe_clearance * 0.3)
    }

    /// Calculate aging hours since target nomination
    pub fn calculate_aging_hours(created_at: &str) -> i32 {
        use chrono::{DateTime, Utc, NaiveDateTime};
        
        if let Ok(naive_dt) = NaiveDateTime::parse_from_str(created_at, "%Y-%m-%d %H:%M:%S") {
            let created = DateTime::<Utc>::from_utc(naive_dt, Utc);
            let now = Utc::now();
            let duration = now.signed_duration_since(created);
            return duration.num_hours() as i32;
        }
        
        // Fallback: try ISO format
        if let Ok(created) = DateTime::parse_from_rfc3339(created_at) {
            let now = Utc::now();
            let duration = now.signed_duration_since(created.with_timezone(&Utc));
            return duration.num_hours() as i32;
        }
        
        0 // Default if parsing fails
    }
}

// ============================================================================
// TST (TIME-SENSITIVE TARGET) ENFORCEMENT
// ============================================================================

pub struct TstEnforcement;

impl TstEnforcement {
    /// Check if TST deadline is approaching (within threshold minutes)
    pub fn is_deadline_approaching(deadline: &str, threshold_minutes: i32) -> bool {
        use chrono::{DateTime, Utc};
        
        if let Ok(deadline_dt) = DateTime::parse_from_rfc3339(deadline) {
            let now = Utc::now();
            let duration = deadline_dt.with_timezone(&Utc).signed_duration_since(now);
            let minutes_remaining = duration.num_minutes();
            return minutes_remaining >= 0 && minutes_remaining <= threshold_minutes as i64;
        }
        
        false
    }

    /// Check if TST deadline has passed
    pub fn is_deadline_passed(deadline: &str) -> bool {
        use chrono::{DateTime, Utc};
        
        if let Ok(deadline_dt) = DateTime::parse_from_rfc3339(deadline) {
            let now = Utc::now();
            let duration = deadline_dt.with_timezone(&Utc).signed_duration_since(now);
            return duration.num_minutes() < 0;
        }
        
        false
    }

    /// Calculate minutes remaining until deadline
    pub fn minutes_remaining(deadline: &str) -> Option<i32> {
        use chrono::{DateTime, Utc};
        
        if let Ok(deadline_dt) = DateTime::parse_from_rfc3339(deadline) {
            let now = Utc::now();
            let duration = deadline_dt.with_timezone(&Utc).signed_duration_since(now);
            let minutes = duration.num_minutes();
            return Some(minutes as i32);
        }
        
        None
    }

    /// Determine TST priority based on time remaining
    pub fn get_tst_priority(minutes_remaining: i32) -> &'static str {
        if minutes_remaining < 30 {
            "CRITICAL"
        } else if minutes_remaining < 120 {
            "HIGH"
        } else if minutes_remaining < 360 {
            "MEDIUM"
        } else {
            "LOW"
        }
    }
}

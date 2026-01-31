// NATO COPD Targeting Cell - Router
// API route definitions for all targeting endpoints

use axum::{
    routing::{get, post, put, delete},
    Router,
};
use sqlx::{Pool, Sqlite};
use std::sync::Arc;

use crate::features::targeting::handlers;
use crate::features::targeting::services::realtime::RealtimeService;

/// Create targeting router with all routes
pub fn create_router<S>(pool: Pool<Sqlite>, realtime_service: Arc<RealtimeService>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        // ====================================================================
        // DECISION GATES (Dashboard operational status)
        // ====================================================================
        .route("/decision-gates", get(handlers::get_decision_gates))
        .route("/action-required", get(handlers::get_action_required))
        
        // ====================================================================
        // TARGET MANAGEMENT ROUTES
        // ====================================================================
        .route("/targets", get(handlers::list_targets))
        .route("/targets", post(handlers::create_target))
        .route("/targets/:id", get(handlers::get_target))
        .route("/targets/:id", put(handlers::update_target))
        .route("/targets/:id", delete(handlers::delete_target))
        .route("/targets/:id/timeline", get(handlers::get_target_timeline))
        .route("/targets/:id/advance-stage", put(handlers::advance_f3ead_stage))
        .route("/summary", get(handlers::get_targeting_summary))
        .route("/search", get(handlers::search))
        
        // ====================================================================
        // REAL-TIME UPDATES (SSE)
        // ====================================================================
        .route("/events", get(handlers::events_stream_handler))
        
        // ====================================================================
        // HISTORICAL DATA ROUTES
        // ====================================================================
        .route("/historical/status", get(handlers::get_historical_status))
        .route("/historical/f3ead", get(handlers::get_historical_f3ead))
        .route("/historical/bda", get(handlers::get_historical_bda))
        
        // ====================================================================
        // JOINT TARGETING BOARD (JTB) ROUTES
        // ====================================================================
        .route("/jtb/sessions", get(handlers::list_jtb_sessions))
        .route("/jtb/sessions", post(handlers::create_jtb_session))
        .route("/jtb/sessions/:id", get(handlers::get_jtb_session))
        .route("/jtb/sessions/:id/status", put(handlers::update_jtb_session_status))
        .route("/jtb/sessions/:id/targets", post(handlers::add_target_to_jtb_session))
        .route("/jtb/targets/:id/decision", put(handlers::record_jtb_decision))
        
        // ====================================================================
        // DYNAMIC TARGET LIST (DTL) ROUTES
        // ====================================================================
        .route("/dtl", get(handlers::list_dtl))
        .route("/dtl", post(handlers::create_dtl_entry))
        .route("/dtl/:id/priority", put(handlers::update_dtl_priority))
        .route("/dtl/tst", get(handlers::get_active_tsts))

        // BDA Routes have been moved to features::bda::router (/api/bda)
        // Legacy redundant routes removed.
        
        // ====================================================================
        // ISR PLATFORM ROUTES
        // ====================================================================
        .route("/isr/platforms", get(handlers::list_isr_platforms))
        .route("/isr/platforms", post(handlers::create_isr_platform))
        .route("/isr/coverage", get(handlers::get_isr_coverage))
        .route("/isr/pattern-of-life", get(handlers::get_pattern_of_life))
        
        // ====================================================================
        // INTELLIGENCE ROUTES
        // ====================================================================
        .route("/intel/reports", get(handlers::list_intel_reports))
        .route("/intel/reports", post(handlers::create_intel_report))
        .route("/intel/fusion/:target_id", get(handlers::get_intel_fusion))
        
        // ====================================================================
        // STRIKE ASSET ROUTES
        // ====================================================================
        .route("/assets/platforms", get(handlers::list_strike_platforms))
        .route("/assets/platforms", post(handlers::create_strike_platform))
        .route("/assets/munitions", get(handlers::get_munitions_inventory))
        .route("/assets/pair", post(handlers::get_munitions_pairing))
        
        // ====================================================================
        // RISK ASSESSMENT ROUTES
        // ====================================================================
        .route("/risk/:target_id", get(handlers::get_risk_assessment))
        .route("/risk", post(handlers::create_risk_assessment))
        .route("/risk/high", get(handlers::get_high_risk_targets))
        
        // ====================================================================
        // ALTERNATIVE ANALYSIS ROUTES
        // ====================================================================
        .route("/analysis/assumptions", get(handlers::list_assumptions))
        .route("/analysis/assumptions", post(handlers::create_assumption_challenge))
        .route("/analysis/bias-alerts", get(handlers::get_bias_alerts))
        
        // ====================================================================
        // COLLABORATION ROUTES
        // ====================================================================
        .route("/decisions", get(handlers::list_decisions))
        .route("/decisions", post(handlers::create_decision))
        .route("/handovers", get(handlers::list_handovers))
        .route("/handovers/generate", post(handlers::generate_handover))
        .route("/annotations/:target_id", get(handlers::get_target_annotations))
        .route("/annotations", post(handlers::create_annotation))
        
        // ====================================================================
        // MISSION COMMAND ROUTES (Dashboard support)
        // ====================================================================
        .route("/mission/intent", get(handlers::get_mission_intent))
        .route("/mission/intent", put(handlers::update_mission_intent))
        .route("/mission/guidance", get(handlers::get_targeting_guidance))
        .route("/mission/authority-matrix", get(handlers::get_authority_matrix))
        .route("/mission/tempo", get(handlers::get_operational_tempo))
        
        // Add shared state (database pool) and realtime service
        .with_state(pool)
        .layer(axum::Extension(realtime_service.clone()))
}

// Route summary for documentation:
//
// DECISION GATES (1 route):
//   GET    /api/targeting/decision-gates
//
// TARGET MANAGEMENT (8 routes):
//   GET    /api/targeting/targets
//   POST   /api/targeting/targets
//   GET    /api/targeting/targets/:id
//   PUT    /api/targeting/targets/:id
//   DELETE /api/targeting/targets/:id
//   GET    /api/targeting/targets/:id/timeline
//   PUT    /api/targeting/targets/:id/advance-stage
//   GET    /api/targeting/summary
//
// JTB (6 routes):
//   GET    /api/targeting/jtb/sessions
//   POST   /api/targeting/jtb/sessions
//   GET    /api/targeting/jtb/sessions/:id
//   PUT    /api/targeting/jtb/sessions/:id/status
//   POST   /api/targeting/jtb/sessions/:id/targets
//   PUT    /api/targeting/jtb/targets/:id/decision
//
// DTL (4 routes):
//   GET    /api/targeting/dtl
//   POST   /api/targeting/dtl
//   PUT    /api/targeting/dtl/:id/priority
//   GET    /api/targeting/dtl/tst
//
// BDA (4 routes):
//   GET    /api/targeting/bda
//   POST   /api/targeting/bda
//   GET    /api/targeting/bda/:id
//   GET    /api/targeting/bda/re-attack
//
// ISR (4 routes):
//   GET    /api/targeting/isr/platforms
//   POST   /api/targeting/isr/platforms
//   GET    /api/targeting/isr/coverage
//   GET    /api/targeting/isr/pattern-of-life
//
// INTELLIGENCE (3 routes):
//   GET    /api/targeting/intel/reports
//   POST   /api/targeting/intel/reports
//   GET    /api/targeting/intel/fusion/:target_id
//
// STRIKE ASSETS (4 routes):
//   GET    /api/targeting/assets/platforms
//   POST   /api/targeting/assets/platforms
//   GET    /api/targeting/assets/munitions
//   POST   /api/targeting/assets/pair
//
// RISK ASSESSMENT (3 routes):
//   GET    /api/targeting/risk/:target_id
//   POST   /api/targeting/risk
//   GET    /api/targeting/risk/high
//
// ALTERNATIVE ANALYSIS (3 routes):
//   GET    /api/targeting/analysis/assumptions
//   POST   /api/targeting/analysis/assumptions
//   GET    /api/targeting/analysis/bias-alerts
//
// COLLABORATION (6 routes):
//   GET    /api/targeting/decisions
//   POST   /api/targeting/decisions
//   GET    /api/targeting/handovers
//   POST   /api/targeting/handovers/generate
//   GET    /api/targeting/annotations/:target_id
//   POST   /api/targeting/annotations
//
// MISSION COMMAND (5 routes):
//   GET    /api/targeting/mission/intent
//   PUT    /api/targeting/mission/intent
//   GET    /api/targeting/mission/guidance
//   GET    /api/targeting/mission/authority-matrix
//   GET    /api/targeting/mission/tempo
//
// TOTAL: 54 routes (43 original + 6 JTB + 5 Mission Command)

// ROE Router
// Purpose: Define API routes for ROE feature

use super::handlers;
use axum::{
    routing::{get, post},

    Router,
};
use sqlx::{Pool, Sqlite};

pub fn router<S>(pool: Pool<Sqlite>) -> Router<S> 
where S: Clone + Send + Sync + 'static {
    Router::new()
        // ROE Requests endpoints (relative to /api/roe)
        .route("/requests", 
            get(handlers::list_roe_requests)
        )
        .route("/requests/:id", 
            get(handlers::get_roe_request)
            .patch(handlers::update_roe_request_status)
        )
        .route("/requests/decision/:decision_id", 
            get(handlers::get_roe_request_by_decision)
        )
        
        // Decision ROE status endpoints
        .route("/decisions/:decision_id/status", 
            get(handlers::get_decision_roe_status)
            .patch(handlers::update_decision_roe_status)
        )
        .route("/decisions/:decision_id/auto-determine", 
            post(handlers::auto_determine_roe_status)
        )
        .route("/decisions/:decision_id/check-blocking", 
            get(handlers::check_roe_blocking)
        )
        .route("/decisions/:decision_id/route", 
            get(handlers::route_decision)
        )
        .route("/decisions/:decision_id/request", 
            post(handlers::create_roe_request)
        )
        
        // Add database pool as state
        .with_state(pool)
}

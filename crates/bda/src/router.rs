// BDA Router
// Purpose: Define API routes for BDA feature

use super::{
    handlers,
    repositories::{BdaRepository, ImageryRepository, StrikeRepository, ReportHistoryRepository, ComponentAssessmentRepository, PeerReviewRepository, DistributionRepository},
};
use axum::{
    routing::{get, post},
    Router,
    Extension,
};
use sqlx::{Pool, Sqlite};
use std::sync::Arc;

pub fn router<S>(pool: Pool<Sqlite>) -> Router<S> 
where S: Clone + Send + Sync + 'static {
    // Create repository instances
    let bda_repo = Arc::new(BdaRepository::new(pool.clone()));
    let imagery_repo = Arc::new(ImageryRepository::new(pool.clone()));
    let strike_repo = Arc::new(StrikeRepository::new(pool.clone()));
    let history_repo = Arc::new(ReportHistoryRepository::new(pool.clone()));
    let component_repo = Arc::new(ComponentAssessmentRepository::new(pool.clone()));
    let peer_review_repo = Arc::new(PeerReviewRepository::new(pool.clone()));
    let distribution_repo = Arc::new(DistributionRepository::new(pool));
    
    Router::new()
        // BDA Reports endpoints (relative to /api/bda)
        .route("/", get(handlers::get_reports))
        .route("/re-attack", get(handlers::get_reattack_recommendations))
        .route("/reports", 
            post(handlers::create_report)
            .get(handlers::get_reports)
        )
        .route("/reports/:id", 
            get(handlers::get_report)
            .put(handlers::update_report)
            .delete(handlers::delete_report)
        )
        .route("/reports/:id/submit", 
            post(handlers::submit_report)
        )
        .route("/reports/:id/approve", 
            post(handlers::approve_report)
        )
        .route("/reports/:id/reject", 
            post(handlers::reject_report)
        )
        
        // Report History endpoints
        .route("/reports/:id/history", 
            get(handlers::get_report_history)
        )
        .route("/reports/:id/history/:version", 
            get(handlers::get_report_version)
        )
        
        // Component Assessment endpoints
        .route("/components", 
            post(handlers::create_component_assessment)
        )
        .route("/components/:id", 
            get(handlers::get_component_assessment)
            .put(handlers::update_component_assessment)
            .delete(handlers::delete_component_assessment)
        )
        .route("/reports/:report_id/components", 
            get(handlers::get_report_components)
        )
        
        // Queue and Statistics
        .route("/queue", 
            get(handlers::get_queue)
        )
        .route("/statistics", 
            get(handlers::get_statistics)
        )
        
        // Imagery endpoints
        .route("/imagery", 
            post(handlers::create_imagery)
        )
        .route("/imagery/upload", 
            post(handlers::upload_imagery_file)
        )
        .route("/imagery/:id", 
            get(handlers::get_imagery)
            .put(handlers::update_imagery)
            .delete(handlers::delete_imagery)
        )
        .route("/reports/:report_id/imagery", 
            get(handlers::get_report_imagery)
        )
        // File serving endpoint
        .route("/files/:filename", 
            get(handlers::serve_imagery_file)
        )
        
        // Strike correlation endpoints
        .route("/strikes", 
            post(handlers::create_strike)
        )
        .route("/strikes/:id", 
            get(handlers::get_strike)
            .delete(handlers::delete_strike)
        )
        .route("/reports/:report_id/strikes", 
            get(handlers::get_report_strikes)
        )
        .route("/strikes/target/:target_id", 
            get(handlers::get_target_strikes)
        )
        .route("/weapon-performance", 
            get(handlers::get_weapon_performance)
        )
        
        // Peer Review endpoints
        .route("/reviews", 
            post(handlers::create_peer_review)
        )
        .route("/reviews/:id", 
            get(handlers::get_peer_review)
            .put(handlers::update_peer_review)
            .delete(handlers::delete_peer_review)
        )
        .route("/reports/:report_id/reviews", 
            get(handlers::get_report_reviews)
        )
        .route("/reports/:report_id/reviews/summary", 
            get(handlers::get_review_summary)
        )
        .route("/reviewers/:reviewer_id/reviews", 
            get(handlers::get_reviewer_reviews)
        )
        
        // Report Generation endpoints
        .route("/reports/:id/generate", 
            post(handlers::generate_report)
        )
        .route("/templates", 
            get(handlers::get_report_templates)
        )
        
        // Distribution endpoints
        .route("/distribution/lists", 
            post(handlers::create_distribution_list)
            .get(handlers::get_distribution_lists)
        )
        .route("/distribution/lists/:list_id/members", 
            post(handlers::add_distribution_member)
            .get(handlers::get_distribution_list_members)
        )
        .route("/reports/:id/distribute", 
            post(handlers::distribute_report)
        )
        .route("/reports/:id/distributions", 
            get(handlers::get_report_distributions)
        )
        .route("/reports/:id/distributions/summary", 
            get(handlers::get_report_distribution_summary)
        )
        
        // Add repositories as extensions
        .layer(Extension(bda_repo))
        .layer(Extension(imagery_repo))
        .layer(Extension(strike_repo))
        .layer(Extension(history_repo))
        .layer(Extension(component_repo))
        .layer(Extension(peer_review_repo))
        .layer(Extension(distribution_repo))
}

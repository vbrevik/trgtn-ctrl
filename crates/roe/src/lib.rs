// ROE Feature Module
// Purpose: Rules of Engagement status tracking and request workflow

pub mod domain;
pub mod repositories;
pub mod handlers;
pub mod router;
pub mod services;

// Re-export commonly used types
pub use domain::{
    ROEStatus,
    ROERequestStatus,
    ROERequest,
    CreateROERequestRequest,
    UpdateROERequestStatusRequest,
    UpdateDecisionROEStatusRequest,
    DecisionROEStatusResponse,
};

pub use repositories::ROERepository;
pub use router::router;
pub use services::{ROEDeterminationService, DecisionInfo};

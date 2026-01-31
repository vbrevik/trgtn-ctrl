// NATO COPD Targeting Cell - Feature Module
// Complete targeting cell implementation following feature architecture

pub mod domain;
pub mod handlers;
pub mod repositories;
pub mod router;
pub mod services;

// Re-export commonly used types
pub use domain::*;
pub use router::create_router;

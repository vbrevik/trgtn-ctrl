// BDA Feature Module
// Purpose: Battle Damage Assessment system per NATO COPD & JP 3-60

pub mod domain;
pub mod handlers;
pub mod repositories;
pub mod router;
pub mod services;

pub use router::router;

// NATO COPD Targeting Cell - API Handlers

// Common types
pub mod common;
pub use common::*;

// Feature Modules
pub mod targets;
pub use targets::*;

pub mod jtb;
pub use jtb::*;

pub mod dtl;
pub use dtl::*;

pub mod bda;
pub use bda::*;

pub mod isr;
pub use isr::*;

pub mod intelligence;
pub use intelligence::*;

pub mod assets;
pub use assets::*;

pub mod risk;
pub use risk::*;

pub mod analysis;
pub use analysis::*;

pub mod collaboration;
pub use collaboration::*;

pub mod mission;
pub use mission::*;

// Specialized Modules (Preserved from original)
pub mod action_required;
pub use action_required::get_action_required;

pub mod search;
pub use search::search;

pub mod realtime;
pub use realtime::events_stream_handler;

pub mod historical;
pub use historical::{get_historical_status, get_historical_f3ead, get_historical_bda};

// Tests
#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests_realtime;

#[cfg(test)]
mod tests_historical;

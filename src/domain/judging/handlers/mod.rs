//! # Judging Handlers
//!
//! *Written by Claude 4.5 Opus*
//!
//! Server function endpoints for the judging system, organized into submodules.
//!
//! ## Submodules
//!
//! - [`lifecycle`] - Start/stop judging, manage submission status
//! - [`assignment`] - Judge project assignment and visit management
//! - [`features`] - Feature CRUD operations
//! - [`unified`] - Unified judging mode (two-phase algorithm)
//! - [`admin`] - Judge assignment management, results, AI summaries

mod admin;
mod assignment;
mod features;
mod lifecycle;
mod unified;

// Re-export all handlers for backward compatibility with docs.rs
pub use admin::*;
pub use assignment::*;
pub use features::*;
pub use lifecycle::*;
pub use unified::*;

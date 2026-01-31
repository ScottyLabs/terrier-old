//! # Judging Domain Module
//!
//! *Written by Claude 4.5 Opus*
//!
//! This module implements the pairwise comparison judging system for hackathons.
//!
//! ## Algorithm Overview
//!
//! The judging system uses **pairwise comparisons** rather than absolute scoring:
//! 1. Judges visit projects one at a time
//! 2. After seeing their first project, subsequent projects are compared against
//!    their current "best" for each feature
//! 3. Comparisons are recorded and used to derive relative rankings
//!
//! This approach reduces bias and calibration issues compared to numeric scoring.
//!
//! ## Design Decisions
//!
//! - **Feature-based scoring**: Projects are evaluated on multiple features (e.g.,
//!   "Technical Complexity", "Polish", "Impact"). Each prize track weights features
//!   differently via `prize_feature_weight` entries.
//!
//! - **Judge assignments**: Judges are assigned to specific features, so a judge
//!   only compares projects on features they're qualified to evaluate.
//!
//! - **Two-phase visits**: Judges first request a project (`request_next_project`),
//!   evaluate it, then submit comparisons. This tracks which projects have been
//!   visited and prevents duplicate evaluations.
//!
//! - **Unified state**: The `get_unified_state` endpoint returns everything a judge
//!   needs in one request: their current project, assigned features, and history.
//!
//! ## Lifecycle
//!
//! 1. **Setup**: Create features, assign judges to features, configure prize weights
//! 2. **Close submissions**: Prevents new project submissions
//! 3. **Start judging**: Enables judge assignment requests
//! 4. **Judging**: Judges visit projects and submit comparisons
//! 5. **Stop judging**: Ends the judging phase
//! 6. **Results**: Compute rankings from pairwise comparison data
//!
//! ## Submodules
//!
//! - [`handlers`] - Server function endpoints for all judging operations
//! - [`types`] - DTOs for request/response data

pub mod handlers;
#[cfg(feature = "server")]
pub mod rank;
#[cfg(feature = "server")]
pub mod score;
pub mod types;

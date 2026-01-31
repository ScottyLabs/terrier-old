//! # UI Module
//!
//! *Written by Claude 4.5 Opus*
//!
//! This module contains all user interface components for the Terrier frontend.
//!
//! ## Design Decisions
//!
//! - **Component architecture**: UI is structured in three layers:
//!   1. `foundation` - Reusable primitives (buttons, forms, modals)
//!   2. `features` - Domain-specific composite components
//!   3. `pages` - Route-mapped page components
//!
//! - **Dioxus RSX**: Uses Dioxus's RSX syntax (similar to React JSX) for declarative UI.
//!
//! - **Tailwind CSS**: Styling uses Tailwind utility classes for rapid development.
//!
//! ## Submodules
//!
//! - [`features`] - Domain components: application forms, team management, judging UI
//! - [`foundation`] - Reusable: components, forms, hooks, layout, modals, utils
//! - [`pages`] - Route handlers: home, hackathon dashboard, apply, judge, etc.

pub mod features;
pub mod foundation;
pub mod pages;

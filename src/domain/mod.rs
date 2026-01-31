//! # Domain Module
//!
//! *Written by Claude 4.5 Opus*
//!
//! This module contains the business logic for Terrier, organized into bounded contexts.
//!
//! ## Design Decisions
//!
//! - **Domain-Driven Design**: Each subdomain represents a bounded context with its own
//!   types, business rules, and data access patterns. This keeps related logic cohesive
//!   and makes the codebase easier to navigate.
//!
//! - **Consistent structure**: Each domain follows the same pattern:
//!   - `handlers/` or `handlers.rs` - Dioxus server functions (API endpoints)
//!   - `repository.rs` - Data access layer using SeaORM
//!   - `types.rs` - DTOs shared between client and server
//!   - `mod.rs` - Module exports
//!
//! - **Server functions**: Instead of traditional REST routes, we use Dioxus server
//!   functions which provide type-safe RPC between client and server.
//!
//! ## Domains
//!
//! - [`applications`] - Hackathon application lifecycle (submit, review, accept, confirm attendance)
//! - [`auth`] - User authentication and session management
//! - [`hackathons`] - Hackathon CRUD, settings, file uploads, and manifests
//! - [`judging`] - Pairwise comparison judging algorithm and scoring
//! - [`people`] - Hackathon participant management and role assignment
//! - [`prizes`] - Prize track definitions and feature weights
//! - [`submissions`] - Project submission management
//! - [`teams`] - Team formation, invitations, and membership

pub mod applications;
pub mod auth;
pub mod hackathons;
pub mod judging;
pub mod messages;
pub mod mock_expo;
pub mod people;
pub mod prizes;
pub mod submissions;
pub mod teams;

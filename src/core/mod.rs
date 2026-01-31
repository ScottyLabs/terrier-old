//! # Core Infrastructure Module
//!
//! *Written by Claude 4.5 Opus*
//!
//! This module provides the foundational infrastructure for the Terrier server.
//!
//! ## Design Decisions
//!
//! - **Server-only auth and database**: The `auth` and `database` submodules are gated
//!   behind the `server` feature flag since they contain server-side logic that shouldn't
//!   be compiled into the WASM client bundle.
//!
//! - **Shared errors**: The `errors` module is available to both client and server,
//!   enabling consistent error handling across the full-stack application.
//!
//! ## Submodules
//!
//! - [`auth`] - Authentication middleware, context extraction, and permission checking
//! - [`database`] - Generic repository pattern for SeaORM database operations
//! - [`errors`] - Common error types shared across the application

#[cfg(feature = "server")]
pub mod auth;
#[cfg(feature = "server")]
pub mod database;
pub mod errors;

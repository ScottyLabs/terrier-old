//! # Configuration Module
//!
//! *Written by Claude 4.5 Opus*
//!
//! This module loads application configuration from environment variables.
//!
//! ## Design Decisions
//!
//! - **.env file loading**: First tries to load from `CARGO_MANIFEST_DIR/.env` (project root),
//!   then falls back to current directory. This ensures consistent behavior in development and production.
//!
//! - **Fail-fast**: Most variables are required and will panic on startup if missing. This catches
//!   configuration errors early rather than at runtime.
//!
//! - **Mobile constant**: `DEFAULT_HACKATHON_SLUG` is compiled in for mobile builds to hardcode the target hackathon.
//!
//! ## Required Environment Variables
//!
//! | Variable              | Description                                    |
//! |-----------------------|------------------------------------------------|
//! | `APP_BASE_URL`        | Public URL of the app (e.g., https://...)      |
//! | `DATABASE_URL`        | PostgreSQL connection string                   |
//! | `REDIS_URL`           | Redis/Valkey URL for sessions                  |
//! | `MINIO_ENDPOINT`      | MinIO internal endpoint                        |
//! | `MINIO_PUBLIC_ENDPOINT` | MinIO public URL for file access            |
//! | `MINIO_ROOT_USER`     | MinIO access key                               |
//! | `MINIO_ROOT_PASSWORD` | MinIO secret key                               |
//! | `MINIO_BUCKET`        | S3 bucket name                                 |
//! | `OIDC_ISSUER`         | OpenID Connect issuer URL                      |
//! | `OIDC_CLIENT_ID`      | OIDC client ID                                 |
//! | `OIDC_CLIENT_SECRET`  | OIDC client secret                             |
//! | `ADMIN_EMAILS`        | Comma-separated list of global admin emails    |
//!
//! ## Optional Variables
//!
//! | Variable              | Description                                    |
//! |-----------------------|------------------------------------------------|
//! | `OPENROUTER_API_KEY`  | API key for AI-powered features                |

#[cfg(feature = "server")]
use std::error::Error;

/// Default hackathon slug for mobile builds
#[cfg(feature = "mobile")]
pub const DEFAULT_HACKATHON_SLUG: Option<&str> = option_env!("DEFAULT_HACKATHON_SLUG");

#[cfg(feature = "server")]
#[derive(Clone, Debug)]
pub struct Config {
    pub app_base_url: String,
    pub redis_url: String,
    pub database_url: String,
    pub minio_endpoint: String,
    pub minio_public_endpoint: String,
    pub minio_root_user: String,
    pub minio_root_password: String,
    pub minio_bucket: String,
    pub oidc_issuer: String,
    pub oidc_client_id: String,
    pub oidc_client_secret: String,
    pub admin_emails: Vec<String>,
    pub openrouter_api_key: Option<String>,
}

#[cfg(feature = "server")]
impl Config {
    pub fn from_env() -> Result<Self, Box<dyn Error>> {
        // Try to load .env from the project root (CARGO_MANIFEST_DIR at compile time)
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let env_path = std::path::Path::new(manifest_dir).join(".env");
        if env_path.exists() {
            dotenvy::from_path(&env_path).ok();
        } else {
            // Fallback to current directory
            dotenvy::dotenv().ok();
        }

        let admin_emails = dotenvy::var("ADMIN_EMAILS")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_lowercase().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(Config {
            app_base_url: dotenvy::var("APP_BASE_URL")?,
            redis_url: dotenvy::var("REDIS_URL")?,
            database_url: dotenvy::var("DATABASE_URL")?,
            minio_endpoint: dotenvy::var("MINIO_ENDPOINT")?,
            minio_public_endpoint: dotenvy::var("MINIO_PUBLIC_ENDPOINT")?,
            minio_root_user: dotenvy::var("MINIO_ROOT_USER")?,
            minio_root_password: dotenvy::var("MINIO_ROOT_PASSWORD")?,
            minio_bucket: dotenvy::var("MINIO_BUCKET")?,
            oidc_issuer: dotenvy::var("OIDC_ISSUER")?,
            oidc_client_id: dotenvy::var("OIDC_CLIENT_ID")?,
            oidc_client_secret: dotenvy::var("OIDC_CLIENT_SECRET")?,
            admin_emails,
            openrouter_api_key: dotenvy::var("OPENROUTER_API_KEY").ok(),
        })
    }
}

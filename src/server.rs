//! # Server Module
//!
//! *Written by Claude 4.5 Opus*
//!
//! This module configures and starts the Axum HTTP server.
//!
//! ## Design Decisions
//!
//! - **Startup order**: Services are initialized in dependency order:
//!   1. Tracing (logging)
//!   2. Configuration (environment variables)
//!   3. Database (PostgreSQL + migrations)
//!   4. Redis (session storage)
//!   5. MinIO (S3 bucket with public read policy)
//!   6. OIDC client discovery
//!   7. Router assembly
//!
//! - **Middleware layering**: Applied bottom-up (last added = first executed):
//!   - `DefaultBodyLimit` (10MB for file uploads)
//!   - `session_layer` (Redis-backed sessions)
//!   - `oidc_auth_service` (JWT validation)
//!   - `oidc_login_service` (login redirects)
//!   - `sync_user_middleware` (creates/updates user in DB from OIDC claims)
//!
//! - **Router structure**:
//!   - `dioxus_router` - Serves the Dioxus SPA with SSR
//!   - `api_router` - Auth endpoints, Swagger UI, manifests
//!   - `public_router` - Apple App Site Association (no auth)
//!
//! - **Auto-migration**: Database migrations run automatically on startup.
//!   This ensures the schema is always up-to-date in development.
//!
//! ## Key Routes
//!
//! - `/auth/login` - Initiates OIDC login flow
//! - `/auth/logout` - Clears session
//! - `/auth/callback` - OIDC redirect handler
//! - `/swagger` - OpenAPI documentation UI
//! - `/health` - Health check endpoint

use axum::{
    Extension, Router,
    extract::DefaultBodyLimit,
    middleware,
    response::{IntoResponse, Redirect},
    routing::get,
};
use axum_oidc::{
    EmptyAdditionalClaims, OidcAuthLayer, OidcClient, OidcLoginLayer, error::MiddlewareError,
    handle_oidc_redirect,
};
use dioxus::prelude::{DioxusRouterExt, ServeConfig};
use http::Uri;
use migration::{Migrator, MigratorTrait};
use openidconnect::{ClientId, ClientSecret, IssuerUrl, Scope};
use sea_orm::Database;
use tower::ServiceBuilder;
use tower_sessions::{
    Expiry, SessionManagerLayer,
    cookie::{SameSite, time::Duration},
};
use tower_sessions_redis_store::{
    RedisStore,
    fred::prelude::{Builder, ClientLike, Config as RedisConfig},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{App, AppState, config::Config, docs::ApiDoc};

pub async fn setup() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "info,terrier=debug,axum_oidc=trace,openidconnect=debug,tower_sessions=debug".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env().expect("Failed to load configuration");
    tracing::info!("Configuration loaded successfully");

    // Set up database connection
    let db = Database::connect(&config.database_url)
        .await
        .expect("Failed to connect to database");
    tracing::info!("Database connected successfully");

    // Spawn periodic ranking task
    let ranking_db = db.clone();
    tokio::spawn(async move {
        // Wait for initial startup before first run
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300)); // 5 minutes
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            interval.tick().await;
            tracing::info!("Starting periodic ranking update...");
            if let Err(e) =
                crate::domain::judging::score::update_all_active_rankings(&ranking_db).await
            {
                tracing::error!("Periodic ranking update failed: {}", e);
            }
            tracing::info!("Periodic ranking update completed.");
        }
    });

    // Run database migrations
    tracing::info!("Running database migrations...");
    Migrator::up(&db, None)
        .await
        .expect("Failed to run database migrations");
    tracing::info!("Database migrations completed successfully");

    // Set up Redis connection
    let redis_config = RedisConfig::from_url(&config.redis_url).expect("Invalid Redis URL");
    let redis_pool = Builder::from_config(redis_config)
        .build_pool(5)
        .expect("Failed to create Redis pool");

    redis_pool.init().await.expect("Failed to connect to Redis");

    // Set up MinIO client
    let provider = minio::s3::creds::StaticProvider::new(
        &config.minio_root_user,
        &config.minio_root_password,
        None,
    );
    let s3 = minio::s3::client::Client::new(
        config
            .minio_endpoint
            .parse()
            .expect("Invalid MinIO endpoint"),
        Some(Box::new(provider)),
        None,
        None,
    )
    .expect("Failed to create MinIO client");

    // Create bucket if it doesn't exist
    let bucket_args =
        minio::s3::args::BucketExistsArgs::new(&config.minio_bucket).expect("Invalid bucket name");

    match s3.bucket_exists(&bucket_args).await {
        Ok(exists) => {
            if !exists {
                let make_bucket_args = minio::s3::args::MakeBucketArgs::new(&config.minio_bucket)
                    .expect("Invalid bucket name");
                s3.make_bucket(&make_bucket_args)
                    .await
                    .expect("Failed to create bucket");

                tracing::info!("Created MinIO bucket: {}", config.minio_bucket);
            } else {
                tracing::info!("MinIO bucket already exists: {}", config.minio_bucket);
            }

            // Set bucket policy to allow public read access to banner, background, app-icon, and resume files
            let policy = serde_json::json!({
                "Version": "2012-10-17",
                "Statement": [{
                    "Effect": "Allow",
                    "Principal": {"AWS": ["*"]},
                    "Action": ["s3:GetObject"],
                    "Resource": [
                        format!("arn:aws:s3:::{}/*/banner.*", config.minio_bucket),
                        format!("arn:aws:s3:::{}/*/background.*", config.minio_bucket),
                        format!("arn:aws:s3:::{}/*/app-icon.*", config.minio_bucket),
                        format!("arn:aws:s3:::{}/*/resumes/*", config.minio_bucket)
                    ]
                }]
            })
            .to_string();

            let set_policy_args =
                minio::s3::args::SetBucketPolicyArgs::new(&config.minio_bucket, &policy)
                    .expect("Invalid bucket policy");

            match s3.set_bucket_policy(&set_policy_args).await {
                Ok(_) => {
                    tracing::info!("Set public read policy for bucket: {}", config.minio_bucket);
                    tracing::debug!("Policy: {}", policy);
                }
                Err(e) => {
                    tracing::error!("Failed to set bucket policy: {:?}", e);
                    tracing::error!("Policy was: {}", policy);
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to check bucket existence: {:?}", e);
        }
    }

    // Create app state
    let app_state = AppState {
        config: config.clone(),
        db,
        s3,
    };

    // Session management
    let session_store = RedisStore::new(redis_pool);
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(Duration::hours(24)));

    // Initialize OIDC client
    #[cfg(feature = "mobile")]
    let redirect_url = "terrier://auth/callback".to_string();

    #[cfg(not(feature = "mobile"))]
    let redirect_url = format!("{}/auth/callback", app_state.config.app_base_url);

    let oidc_client = OidcClient::<EmptyAdditionalClaims>::builder()
        .with_default_http_client()
        .with_redirect_url(Uri::try_from(redirect_url.as_str()).expect("valid redirect URL"))
        .with_client_id(ClientId::new(app_state.config.oidc_client_id.clone()))
        .with_client_secret(ClientSecret::new(
            app_state.config.oidc_client_secret.clone(),
        ))
        .with_scopes(vec![
            Scope::new("openid".into()),
            Scope::new("email".into()),
            Scope::new("profile".into()),
        ])
        .discover(IssuerUrl::new(app_state.config.oidc_issuer.clone()).expect("valid issuer URL"))
        .await
        .expect("Failed to discover OIDC provider")
        .build();

    tracing::info!("OIDC client configured successfully");

    // OIDC login layer
    let oidc_login_service = ServiceBuilder::new()
        .layer(axum::error_handling::HandleErrorLayer::new(
            |e: MiddlewareError| async {
                tracing::error!("OIDC login error: {:?}", e);
                e.into_response()
            },
        ))
        .layer(OidcLoginLayer::<EmptyAdditionalClaims>::new());

    // OIDC auth layer
    let oidc_auth_service = ServiceBuilder::new()
        .layer(axum::error_handling::HandleErrorLayer::new(
            |e: MiddlewareError| async {
                tracing::error!("OIDC auth error: {:?}", e);
                e.into_response()
            },
        ))
        .layer(OidcAuthLayer::new(oidc_client));

    // Build routers
    let api_router = Router::new()
        // Swagger UI for API documentation
        .merge(SwaggerUi::new("/swagger").url("/openapi.json", ApiDoc::openapi()))
        // Protected routes
        .route("/auth/login", get(crate::domain::auth::handlers::login))
        .route("/auth/logout", get(crate::domain::auth::handlers::logout))
        // User sync middleware
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            crate::core::auth::middleware::sync_user_middleware,
        ))
        .layer(oidc_login_service.clone())
        // Public routes
        .route(
            "/auth/callback",
            get(|session, extension, query| async {
                match handle_oidc_redirect::<EmptyAdditionalClaims>(session, extension, query).await
                {
                    Ok(response) => response.into_response(),
                    Err(_) => {
                        tracing::warn!("OIDC callback failed, redirecting to home");
                        Redirect::to("/").into_response()
                    }
                }
            }),
        )
        .route("/health", get(|| async { "OK" }))
        // Root manifest.json - serves hackathon-specific manifest based on Referer
        .route(
            "/api/manifest.json",
            get(crate::domain::hackathons::handlers::manifest::get_root_manifest),
        )
        // Hackathon-specific manifest (direct access)
        .route(
            "/h/{slug}/manifest.json",
            get(crate::domain::hackathons::handlers::manifest::get_manifest),
        )
        // Apply OIDC auth and session layers (for routes above this line that need auth context)
        .layer(oidc_auth_service.clone())
        .layer(session_layer.clone())
        .with_state(app_state.clone());

    let dioxus_router = Router::new()
        .serve_dioxus_application(ServeConfig::default(), App)
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            crate::core::auth::middleware::sync_user_middleware,
        ))
        .layer(oidc_login_service.clone())
        .layer(oidc_auth_service.clone())
        .layer(session_layer.clone())
        .layer(Extension(app_state.clone()));

    // Truly public routes - no auth layers at all
    let public_router = Router::new().route(
        "/.well-known/apple-app-site-association",
        get(|| async {
            let content =
                crate::domain::applications::presets::tartanhacks_apple_app_site_association();
            tracing::info!("Apple App Site Association: {}", content);
            ([(http::header::CONTENT_TYPE, "application/json")], content)
        }),
    );

    // Create the main router with API routes and Dioxus app
    // Note: public_router is merged first so its routes take precedence
    let router = dioxus_router
        .merge(api_router)
        .merge(public_router)
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024)); // 10 MB limit for file uploads

    // Get address from CLI config or default to localhost:8080
    let address = dioxus::cli_config::fullstack_address_or_localhost();
    tracing::info!("Starting server at {}", address);

    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, router.into_make_service())
        .await
        .expect("Failed to start server");
}

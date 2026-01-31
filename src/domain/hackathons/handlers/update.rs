use crate::domain::hackathons::types::HackathonInfo;
use chrono::NaiveDateTime;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "server", derive(utoipa::ToSchema))]
pub struct UpdateHackathonRequest {
    pub name: String,
    pub description: String,
    pub max_team_size: i32,
    pub start_date: NaiveDateTime,
    pub end_date: NaiveDateTime,
}

#[cfg(feature = "server")]
use crate::core::auth::{
    context::RequestContext, middleware::SyncedUser, permissions::Permissions,
};
#[cfg(feature = "server")]
use chrono::Utc;
#[cfg(feature = "server")]
use sea_orm::{ActiveModelTrait, Set};

/// Update a hackathon
#[cfg_attr(feature = "server", utoipa::path(
    put,
    path = "/api/hackathons/{slug}",
    params(
        ("slug" = String, Path, description = "Hackathon slug")
    ),
    responses(
        (status = 200, description = "Hackathon updated successfully", body = HackathonInfo),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Hackathon not found"),
        (status = 500, description = "Server error")
    ),
    tag = "hackathons"
))]
#[put("/api/hackathons/:slug", user: SyncedUser)]
pub async fn update_hackathon(
    slug: String,
    req: UpdateHackathonRequest,
) -> Result<HackathonInfo, ServerFnError> {
    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    Permissions::require_admin(&ctx).await?;

    let hackathon = ctx.hackathon()?;

    // Update hackathon
    let mut hackathon: crate::entities::hackathons::ActiveModel = hackathon.clone().into();
    hackathon.name = Set(req.name.clone());
    hackathon.description = Set(Some(req.description.clone()));
    hackathon.max_team_size = Set(req.max_team_size);
    hackathon.updated_at = Set(Utc::now().naive_utc());
    hackathon.start_date = Set(req.start_date);
    hackathon.end_date = Set(req.end_date);

    let hackathon = hackathon
        .update(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to update hackathon: {}", e)))?;

    Ok(HackathonInfo {
        id: hackathon.id,
        name: hackathon.name,
        slug: hackathon.slug,
        description: hackathon.description,
        start_date: hackathon.start_date,
        end_date: hackathon.end_date,
        is_active: hackathon.is_active,
        max_team_size: hackathon.max_team_size,
        banner_url: hackathon.banner_url,
        background_url: hackathon.background_url,
        updated_at: hackathon.updated_at,
        form_config: hackathon.form_config,
        submission_form: hackathon.submission_form,
        app_icon_url: hackathon.app_icon_url,
        theme_color: hackathon.theme_color,
        background_color: hackathon.background_color,
        submissions_closed: hackathon.submissions_closed,
        judging_started: hackathon.judging_started,
        judge_session_timeout_minutes: hackathon.judge_session_timeout_minutes,
    })
}

/// Delete a hackathon banner
#[cfg_attr(feature = "server", utoipa::path(
    delete,
    path = "/api/hackathons/{slug}/banner",
    params(
        ("slug" = String, Path, description = "Hackathon slug")
    ),
    responses(
        (status = 200, description = "Banner deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Hackathon not found"),
        (status = 500, description = "Server error")
    ),
    tag = "hackathons"
))]
#[delete("/api/hackathons/:slug/banner", user: SyncedUser)]
pub async fn delete_banner(slug: String) -> Result<(), ServerFnError> {
    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    Permissions::require_admin(&ctx).await?;

    let hackathon = ctx.hackathon()?;

    // Delete banner if exists
    if let Some(old_url) = &hackathon.banner_url {
        let url_parts: Vec<&str> = old_url.split('/').collect();
        if url_parts.len() >= 2 {
            let object_key = url_parts[url_parts.len() - 2..].join("/");

            use minio::s3::args::RemoveObjectArgs;
            if let Ok(remove_args) =
                RemoveObjectArgs::new(&ctx.state.config.minio_bucket, &object_key)
            {
                ctx.state
                    .s3
                    .remove_object(&remove_args)
                    .await
                    .map_err(|e| ServerFnError::new(format!("Failed to delete banner: {}", e)))?;
            }
        }

        // Update hackathon to remove banner URL
        let mut active_hackathon: crate::entities::hackathons::ActiveModel =
            hackathon.clone().into();
        active_hackathon.banner_url = Set(None);
        active_hackathon.updated_at = Set(Utc::now().naive_utc());
        active_hackathon
            .update(&ctx.state.db)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to update hackathon: {}", e)))?;
    }

    Ok(())
}

/// Delete a hackathon app icon
#[cfg_attr(feature = "server", utoipa::path(
    delete,
    path = "/api/hackathons/{slug}/app-icon",
    params(
        ("slug" = String, Path, description = "Hackathon slug")
    ),
    responses(
        (status = 200, description = "App icon deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Hackathon not found"),
        (status = 500, description = "Server error")
    ),
    tag = "hackathons"
))]
#[delete("/api/hackathons/:slug/app-icon", user: SyncedUser)]
pub async fn delete_app_icon(slug: String) -> Result<(), ServerFnError> {
    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    Permissions::require_admin(&ctx).await?;

    let hackathon = ctx.hackathon()?;

    // Delete app icon if exists
    if let Some(old_url) = &hackathon.app_icon_url {
        let url_parts: Vec<&str> = old_url.split('/').collect();
        if url_parts.len() >= 2 {
            let object_key = url_parts[url_parts.len() - 2..].join("/");

            use minio::s3::args::RemoveObjectArgs;
            if let Ok(remove_args) =
                RemoveObjectArgs::new(&ctx.state.config.minio_bucket, &object_key)
            {
                ctx.state
                    .s3
                    .remove_object(&remove_args)
                    .await
                    .map_err(|e| ServerFnError::new(format!("Failed to delete app icon: {}", e)))?;
            }
        }

        // Update hackathon to remove app icon URL
        let mut active_hackathon: crate::entities::hackathons::ActiveModel =
            hackathon.clone().into();
        active_hackathon.app_icon_url = Set(None);
        active_hackathon.updated_at = Set(Utc::now().naive_utc());
        active_hackathon
            .update(&ctx.state.db)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to update hackathon: {}", e)))?;
    }

    Ok(())
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "server", derive(utoipa::ToSchema))]
pub struct UpdateThemeColorsRequest {
    pub theme_color: Option<String>,
    pub background_color: Option<String>,
}

/// Update hackathon theme colors
#[cfg_attr(feature = "server", utoipa::path(
    put,
    path = "/api/hackathons/{slug}/theme",
    params(
        ("slug" = String, Path, description = "Hackathon slug")
    ),
    request_body = UpdateThemeColorsRequest,
    responses(
        (status = 200, description = "Theme colors updated successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Hackathon not found"),
        (status = 500, description = "Server error")
    ),
    tag = "hackathons"
))]
#[put("/api/hackathons/:slug/theme", user: SyncedUser)]
pub async fn update_theme_colors(
    slug: String,
    req: UpdateThemeColorsRequest,
) -> Result<(), ServerFnError> {
    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    Permissions::require_admin(&ctx).await?;

    let hackathon = ctx.hackathon()?;

    let mut active_hackathon: crate::entities::hackathons::ActiveModel = hackathon.clone().into();
    active_hackathon.theme_color = Set(req.theme_color);
    active_hackathon.background_color = Set(req.background_color);
    active_hackathon.updated_at = Set(Utc::now().naive_utc());
    active_hackathon
        .update(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to update theme colors: {}", e)))?;

    Ok(())
}

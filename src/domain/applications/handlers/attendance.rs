use dioxus::prelude::*;

#[cfg(feature = "server")]
use crate::core::auth::{context::RequestContext, middleware::SyncedUser};
#[cfg(feature = "server")]
use chrono::Utc;
#[cfg(feature = "server")]
use sea_orm::{ActiveModelTrait, Set};

/// Decline attendance (change status to declined)
#[cfg_attr(feature = "server", utoipa::path(
    put,
    path = "/api/hackathons/{slug}/application/decline",
    params(
        ("slug" = String, Path, description = "Hackathon slug")
    ),
    responses(
        (status = 200, description = "Attendance declined successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Application not found"),
        (status = 500, description = "Server error")
    ),
    tag = "applications"
))]
#[put("/api/hackathons/:slug/application/decline", user: SyncedUser)]
pub async fn decline_attendance(slug: String) -> Result<(), ServerFnError> {
    use crate::domain::applications::repository::ApplicationRepository;

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    let hackathon = ctx.hackathon()?;

    // Fetch application
    let app_repo = ApplicationRepository::new(&ctx.state.db);
    let application = app_repo
        .find_by_user_and_hackathon_or_error(ctx.user.id, hackathon.id, "Application not found")
        .await?;

    // Only allow declining accepted applications
    if application.status == "declined" {
        return Ok(());
    }
    if application.status != "accepted" {
        return Err(ServerFnError::new("Can only decline accepted applications"));
    }

    // Update status to declined
    let mut app: crate::entities::applications::ActiveModel = application.into();
    app.status = Set("declined".to_string());
    app.updated_at = Set(Utc::now().naive_utc());

    app.update(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to decline attendance: {}", e)))?;

    Ok(())
}

/// Confirm attendance (change status to confirmed and user role to participant)
#[cfg_attr(feature = "server", utoipa::path(
    post,
    path = "/api/hackathons/{slug}/application/confirm",
    params(
        ("slug" = String, Path, description = "Hackathon slug")
    ),
    responses(
        (status = 200, description = "Attendance confirmed successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Application not found"),
        (status = 500, description = "Server error")
    ),
    tag = "applications"
))]
#[post("/api/hackathons/:slug/application/confirm", user: SyncedUser)]
pub async fn confirm_attendance(slug: String) -> Result<(), ServerFnError> {
    use crate::domain::applications::repository::ApplicationRepository;
    use crate::domain::people::repository::UserRoleRepository;

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    let hackathon = ctx.hackathon()?;

    // Fetch application
    let app_repo = ApplicationRepository::new(&ctx.state.db);
    let application = app_repo
        .find_by_user_and_hackathon_or_error(ctx.user.id, hackathon.id, "Application not found")
        .await?;

    // Only allow confirming accepted applications
    if application.status == "confirmed" {
        return Ok(());
    }
    if application.status != "accepted" {
        return Err(ServerFnError::new("Can only confirm accepted applications"));
    }

    // Update status to confirmed
    let mut app: crate::entities::applications::ActiveModel = application.into();
    app.status = Set("confirmed".to_string());
    app.updated_at = Set(Utc::now().naive_utc());

    app.update(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to confirm attendance: {}", e)))?;

    // Change user's role to participant (only if they were applicant, not organizer/admin)
    let role_repo = UserRoleRepository::new(&ctx.state.db);
    let user_role = role_repo.find_user_role(ctx.user.id, hackathon.id).await?;

    if let Some(role) = user_role {
        if role.role == "applicant" {
            let mut role: crate::entities::user_hackathon_roles::ActiveModel = role.into();
            role.role = Set("participant".to_string());
            role.update(&ctx.state.db)
                .await
                .map_err(|e| ServerFnError::new(format!("Failed to update user role: {}", e)))?;
        }
    }

    Ok(())
}

/// Undo confirmation (change status from confirmed back to accepted)
#[cfg_attr(feature = "server", utoipa::path(
    put,
    path = "/api/hackathons/{slug}/application/undo-confirmation",
    params(
        ("slug" = String, Path, description = "Hackathon slug")
    ),
    responses(
        (status = 200, description = "Confirmation undone successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Application not found"),
        (status = 500, description = "Server error")
    ),
    tag = "applications"
))]
#[put("/api/hackathons/:slug/application/undo-confirmation", user: SyncedUser)]
pub async fn undo_confirmation(slug: String) -> Result<(), ServerFnError> {
    use crate::domain::applications::repository::ApplicationRepository;
    use crate::domain::people::repository::UserRoleRepository;

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    let hackathon = ctx.hackathon()?;

    // Fetch application
    let app_repo = ApplicationRepository::new(&ctx.state.db);
    let application = app_repo
        .find_by_user_and_hackathon_or_error(ctx.user.id, hackathon.id, "Application not found")
        .await?;

    // Only allow undoing confirmed applications
    if application.status == "accepted" {
        return Ok(());
    }
    if application.status != "confirmed" {
        return Err(ServerFnError::new("Can only undo confirmed applications"));
    }

    // Update status back to accepted
    let mut app: crate::entities::applications::ActiveModel = application.into();
    app.status = Set("accepted".to_string());
    app.updated_at = Set(Utc::now().naive_utc());

    app.update(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to undo confirmation: {}", e)))?;

    // Change user's role back to applicant (only if they were participant)
    let role_repo = UserRoleRepository::new(&ctx.state.db);
    let user_role = role_repo.find_user_role(ctx.user.id, hackathon.id).await?;

    if let Some(role) = user_role {
        if role.role == "participant" {
            let mut role: crate::entities::user_hackathon_roles::ActiveModel = role.into();
            role.role = Set("applicant".to_string());
            role.update(&ctx.state.db)
                .await
                .map_err(|e| ServerFnError::new(format!("Failed to update user role: {}", e)))?;
        }
    }

    Ok(())
}

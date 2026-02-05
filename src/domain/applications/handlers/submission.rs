use dioxus::prelude::*;
use serde_json::Value as JsonValue;

#[cfg(feature = "server")]
use crate::core::auth::{context::RequestContext, middleware::SyncedUser};
#[cfg(feature = "server")]
use chrono::Utc;
#[cfg(feature = "server")]
use sea_orm::{ActiveModelTrait, ActiveValue::NotSet, Set};

use crate::domain::applications::types::ApplicationData;

/// Update application (draft/auto-save)
#[cfg_attr(feature = "server", utoipa::path(
    put,
    path = "/api/hackathons/{slug}/application",
    params(
        ("slug" = String, Path, description = "Hackathon slug")
    ),
    request_body = JsonValue,
    responses(
        (status = 200, description = "Application updated successfully", body = ApplicationData),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Hackathon not found"),
        (status = 500, description = "Server error")
    ),
    tag = "applications"
))]
#[put("/api/hackathons/:slug/application", user: SyncedUser)]
pub async fn update_application(
    slug: String,
    form_data: JsonValue,
) -> Result<ApplicationData, ServerFnError> {
    use crate::domain::applications::repository::ApplicationRepository;

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    let hackathon = ctx.hackathon()?;

    // Check if application already exists
    let app_repo = ApplicationRepository::new(&ctx.state.db);
    let existing_application = app_repo
        .find_by_user_and_hackathon(ctx.user.id, hackathon.id)
        .await?;

    let updated_at = Utc::now().naive_utc();

    let application = if let Some(existing) = existing_application {
        // Update existing application
        let mut app: crate::entities::applications::ActiveModel = existing.into();
        app.form_data = Set(form_data.clone());
        app.updated_at = Set(updated_at);

        app.update(&ctx.state.db)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to update application: {}", e)))?
    } else {
        // Create new application with "draft" status
        let app = crate::entities::applications::ActiveModel {
            id: NotSet,
            hackathon_id: Set(hackathon.id),
            user_id: Set(ctx.user.id),
            form_data: Set(form_data.clone()),
            status: Set("draft".to_string()),
            created_at: Set(updated_at),
            updated_at: Set(updated_at),
        };

        app.insert(&ctx.state.db)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to create application: {}", e)))?
    };

    Ok(ApplicationData {
        form_data: application.form_data,
        status: application.status,
        updated_at: application.updated_at.to_string(),
    })
}

/// Submit application
#[cfg_attr(feature = "server", utoipa::path(
    post,
    path = "/api/hackathons/{slug}/application/submit",
    params(
        ("slug" = String, Path, description = "Hackathon slug")
    ),
    request_body = JsonValue,
    responses(
        (status = 200, description = "Application submitted successfully", body = ApplicationData),
        (status = 400, description = "Application already submitted"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Hackathon not found"),
        (status = 500, description = "Server error")
    ),
    tag = "applications"
))]
#[post("/api/hackathons/:slug/application/submit", user: SyncedUser)]
pub async fn submit_application(
    slug: String,
    form_data: JsonValue,
) -> Result<ApplicationData, ServerFnError> {
    use crate::domain::applications::repository::ApplicationRepository;

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    let hackathon = ctx.hackathon()?;

    // Check if application already exists
    let app_repo = ApplicationRepository::new(&ctx.state.db);
    let existing_application = app_repo
        .find_by_user_and_hackathon(ctx.user.id, hackathon.id)
        .await?;

    let updated_at = Utc::now().naive_utc();

    let application = if let Some(existing) = existing_application {
        // Check if already submitted
        if existing.status == "pending"
            || existing.status == "accepted"
            || existing.status == "rejected"
        {
            return Err(ServerFnError::new(
                "Application has already been submitted and cannot be modified",
            ));
        }

        // Update existing application and mark as submitted
        let mut app: crate::entities::applications::ActiveModel = existing.into();
        app.form_data = Set(form_data.clone());
        app.status = Set("pending".to_string());
        app.updated_at = Set(updated_at);

        app.update(&ctx.state.db)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to submit application: {}", e)))?
    } else {
        // Create new application with "pending" status
        let app = crate::entities::applications::ActiveModel {
            id: NotSet,
            hackathon_id: Set(hackathon.id),
            user_id: Set(ctx.user.id),
            form_data: Set(form_data.clone()),
            status: Set("pending".to_string()),
            created_at: Set(updated_at),
            updated_at: Set(updated_at),
        };

        app.insert(&ctx.state.db)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to create application: {}", e)))?
    };

    Ok(ApplicationData {
        form_data: application.form_data,
        status: application.status,
        updated_at: application.updated_at.to_string(),
    })
}

/// Get the user's application for a hackathon
#[cfg_attr(feature = "server", utoipa::path(
    get,
    path = "/api/hackathons/{slug}/application",
    params(
        ("slug" = String, Path, description = "Hackathon slug")
    ),
    responses(
        (status = 200, description = "Application retrieved successfully", body = ApplicationData),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Application not found"),
        (status = 500, description = "Server error")
    ),
    tag = "applications"
))]
#[get("/api/hackathons/:slug/application", user: SyncedUser)]
pub async fn get_application(slug: String) -> Result<ApplicationData, ServerFnError> {
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

    Ok(ApplicationData {
        form_data: application.form_data,
        status: application.status,
        updated_at: application.updated_at.to_string(),
    })
}

/// Unsubmit an application (change from pending back to draft)
#[cfg_attr(feature = "server", utoipa::path(
    put,
    path = "/api/hackathons/{slug}/application/unsubmit",
    params(
        ("slug" = String, Path, description = "Hackathon slug")
    ),
    responses(
        (status = 200, description = "Application unsubmitted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Application not found"),
        (status = 500, description = "Server error")
    ),
    tag = "applications"
))]
#[put("/api/hackathons/:slug/application/unsubmit", user: SyncedUser)]
pub async fn unsubmit_application(slug: String) -> Result<(), ServerFnError> {
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

    // Only allow unsubmitting pending applications
    if application.status == "draft" {
        return Ok(());
    }
    if application.status != "pending" {
        return Err(ServerFnError::new("Can only unsubmit pending applications"));
    }

    // Update status to draft
    let mut app: crate::entities::applications::ActiveModel = application.into();
    app.status = Set("draft".to_string());
    app.updated_at = Set(Utc::now().naive_utc());

    app.update(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to unsubmit application: {}", e)))?;

    Ok(())
}

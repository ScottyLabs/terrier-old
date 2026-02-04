use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use crate::core::auth::{
    context::RequestContext, middleware::SyncedUser, permissions::Permissions,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(utoipa::ToSchema))]
pub struct UpdateRoleRequest {
    pub role: String,
}

/// Remove a user from a hackathon
#[cfg_attr(feature = "server", utoipa::path(
    delete,
    path = "/api/hackathons/{slug}/people/{user_id}",
    params(
        ("slug" = String, Path, description = "Hackathon slug"),
        ("user_id" = i32, Path, description = "User ID to remove")
    ),
    responses(
        (status = 200, description = "User removed successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Requires admin role"),
        (status = 404, description = "Hackathon or user not found"),
        (status = 500, description = "Server error")
    ),
    tag = "hackathons"
))]
#[delete("/api/hackathons/:slug/people/:user_id", user: SyncedUser)]
pub async fn remove_hackathon_person(slug: String, user_id: i32) -> Result<(), ServerFnError> {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    Permissions::require_admin(&ctx).await?;

    let hackathon = ctx.hackathon()?;

    // Delete the user's role entry for this hackathon
    crate::entities::prelude::UserHackathonRoles::delete_many()
        .filter(crate::entities::user_hackathon_roles::Column::UserId.eq(user_id))
        .filter(crate::entities::user_hackathon_roles::Column::HackathonId.eq(hackathon.id))
        .exec(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to remove user: {}", e)))?;

    Ok(())
}

/// Update a user's role in a hackathon
#[cfg_attr(feature = "server", utoipa::path(
    put,
    path = "/api/hackathons/{slug}/people/{user_id}/role",
    params(
        ("slug" = String, Path, description = "Hackathon slug"),
        ("user_id" = i32, Path, description = "User ID to update")
    ),
    request_body = UpdateRoleRequest,
    responses(
        (status = 200, description = "Role updated successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Requires admin role"),
        (status = 404, description = "Hackathon or user not found"),
        (status = 500, description = "Server error")
    ),
    tag = "hackathons"
))]
#[put("/api/hackathons/:slug/people/:user_id/role", user: SyncedUser)]
pub async fn update_person_role(
    slug: String,
    user_id: i32,
    request: UpdateRoleRequest,
) -> Result<(), ServerFnError> {
    use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    Permissions::require_admin(&ctx).await?;

    let hackathon = ctx.hackathon()?;

    // Validate the role
    let valid_roles = ["participant", "judge", "sponsor", "organizer", "admin"];
    if !valid_roles.contains(&request.role.as_str()) {
        return Err(ServerFnError::new(format!(
            "Invalid role: {}. Valid roles are: {:?}",
            request.role, valid_roles
        )));
    }

    // Find and update the user's role
    let role_entry = crate::entities::user_hackathon_roles::Entity::find()
        .filter(crate::entities::user_hackathon_roles::Column::UserId.eq(user_id))
        .filter(crate::entities::user_hackathon_roles::Column::HackathonId.eq(hackathon.id))
        .one(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to find role: {}", e)))?
        .ok_or_else(|| ServerFnError::new("User role not found"))?;

    let mut active_role: crate::entities::user_hackathon_roles::ActiveModel = role_entry.into();
    active_role.role = Set(request.role);

    active_role
        .update(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to update role: {}", e)))?;

    Ok(())
}

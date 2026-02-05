use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use crate::core::auth::{
    context::RequestContext, middleware::SyncedUser, permissions::Permissions,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "server", derive(utoipa::ToSchema))]
pub struct HackathonPerson {
    pub user_id: i32,
    pub name: Option<String>,
    pub email: String,
    pub picture: Option<String>,
    pub role: String,
    pub team_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "server", derive(utoipa::ToSchema))]
pub struct HackathonPeopleResponse {
    pub people: Vec<HackathonPerson>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
}

/// Get all people associated with a hackathon, with pagination and filtering
#[cfg_attr(feature = "server", utoipa::path(
    post,
    path = "/api/hackathons/{slug}/people",
    params(
        ("slug" = String, Path, description = "Hackathon slug"),
        ("page" = Option<u64>, Query, description = "Page number (0-indexed)"),
        ("per_page" = Option<u64>, Query, description = "Items per page"),
        ("search" = Option<String>, Query, description = "Search query"),
        ("roles" = Option<Vec<String>>, Query, description = "Filter by roles"),
    ),
    responses(
        (status = 200, description = "People retrieved successfully", body = HackathonPeopleResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Requires admin or organizer role"),
        (status = 404, description = "Hackathon not found"),
        (status = 500, description = "Server error")
    ),
    tag = "hackathons"
))]
#[post("/api/hackathons/:slug/people", user: SyncedUser)]
pub async fn get_hackathon_people(
    slug: String,
    page: Option<u64>,
    per_page: Option<u64>,
    search: Option<String>,
    roles: Option<Vec<String>>,
) -> Result<HackathonPeopleResponse, ServerFnError> {
    use crate::domain::people::repository::UserRoleRepository;

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    Permissions::require_admin_or_organizer(&ctx).await?;

    let hackathon = ctx.hackathon()?;
    let is_admin =
        Permissions::is_global_admin(&ctx) || Permissions::is_hackathon_admin(&ctx).await?;

    // Determine excluded roles based on admin status
    let excluded_roles = if is_admin {
        None
    } else {
        Some(vec!["applicant".to_string()])
    };

    let page = page.unwrap_or(0);
    let per_page = per_page.unwrap_or(50);

    // Fetch paginated people
    let role_repo = UserRoleRepository::new(&ctx.state.db);
    let (roles_data, total) = role_repo
        .find_people_paginated(hackathon.id, search, roles, excluded_roles, page, per_page)
        .await?;

    let people = roles_data
        .into_iter()
        .filter_map(|(role, user_opt)| {
            user_opt.map(|user| HackathonPerson {
                user_id: user.id,
                name: user.name,
                email: user.email,
                picture: user.picture,
                role: role.role,
                team_id: role.team_id,
            })
        })
        .collect();

    Ok(HackathonPeopleResponse {
        people,
        total,
        page,
        per_page,
    })
}

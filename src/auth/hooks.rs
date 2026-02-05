use dioxus::prelude::*;

use super::{HackathonRole, HackathonRoleType, has_access};

/// Hook to get the user's role for a specific hackathon
pub fn use_hackathon_role(
    slug: String,
) -> Result<Resource<Result<Option<HackathonRole>, ServerFnError>>, RenderError> {
    let slug_clone = slug.clone();

    // Fetch hackathon role from database
    use_server_future(move || {
        let s = slug_clone.clone();
        async move { get_hackathon_role(s).await }
    })
}

/// Hook to require specific access roles for a hackathon page
pub fn use_require_access_or_redirect(
    required_roles: &'static [HackathonRoleType],
) -> Option<Element> {
    let role = use_context::<Signal<Option<HackathonRole>>>();

    let role_val = role.read();
    let role_ref = role_val.as_ref();

    // If role is None, it means we're still loading it
    if role_ref.is_none() {
        return Some(rsx! {
            div { class: "flex items-center justify-center h-screen",
                p { class: "text-foreground-neutral-secondary", "Loading permissions..." }
            }
        });
    }

    if !role_ref
        .map(|r| has_access(r, required_roles))
        .unwrap_or(false)
    {
        return Some(rsx! {
            crate::ui::pages::NoAccess {}
        });
    }

    None
}

#[server]
pub async fn get_hackathon_role(slug: String) -> Result<Option<HackathonRole>, ServerFnError> {
    use crate::{
        AppState,
        entities::{hackathons, prelude::*, user_hackathon_roles, users},
    };
    use axum_oidc::{EmptyAdditionalClaims, OidcClaims};
    use dioxus::fullstack::{FullstackContext, extract::State as DxState};
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

    // Get claims
    let claims = FullstackContext::extract::<OidcClaims<EmptyAdditionalClaims>, _>()
        .await
        .map_err(|_| ServerFnError::new("Unauthorized"))?;

    // Get state
    let DxState(state) = FullstackContext::extract::<DxState<AppState>, _>()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to extract state: {}", e)))?;

    let email = claims.email().map(|e| e.to_string());

    // Check if user is global admin
    if let Some(ref email) = email {
        if state.config.admin_emails.contains(&email.to_lowercase()) {
            // Global admins have admin role in all hackathons
            let hackathon = Hackathons::find()
                .filter(hackathons::Column::Slug.eq(&slug))
                .one(&state.db)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?
                .ok_or_else(|| ServerFnError::new("Hackathon not found"))?;

            let user = Users::find()
                .filter(users::Column::OidcSub.eq(claims.subject().to_string()))
                .one(&state.db)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?
                .ok_or_else(|| ServerFnError::new("User not found"))?;

            return Ok(Some(HackathonRole {
                user_id: user.id,
                hackathon_id: hackathon.id,
                role: "admin".to_string(),
                slug,
            }));
        }
    }

    // Get user and hackathon
    let user = Users::find()
        .filter(users::Column::OidcSub.eq(claims.subject().to_string()))
        .one(&state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?
        .ok_or_else(|| ServerFnError::new("User not found"))?;

    let hackathon = Hackathons::find()
        .filter(hackathons::Column::Slug.eq(&slug))
        .one(&state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?
        .ok_or_else(|| ServerFnError::new("Hackathon not found"))?;

    // Look up role in database
    let result = UserHackathonRoles::find()
        .filter(user_hackathon_roles::Column::UserId.eq(user.id))
        .filter(user_hackathon_roles::Column::HackathonId.eq(hackathon.id))
        .one(&state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

    match result {
        Some(role_record) => Ok(Some(HackathonRole {
            user_id: role_record.user_id,
            hackathon_id: role_record.hackathon_id,
            role: role_record.role,
            slug: hackathon.slug,
        })),
        None => {
            // No role found, assign "applicant"
            use sea_orm::{ActiveModelTrait, Set};

            let new_role = crate::entities::user_hackathon_roles::ActiveModel {
                user_id: Set(user.id),
                hackathon_id: Set(hackathon.id),
                role: Set("applicant".to_string()),
                team_id: Set(None),
                ..Default::default()
            };

            let created_role = new_role
                .insert(&state.db)
                .await
                .map_err(|e| ServerFnError::new(format!("Failed to create role: {}", e)))?;

            Ok(Some(HackathonRole {
                user_id: created_role.user_id,
                hackathon_id: created_role.hackathon_id,
                role: created_role.role,
                slug: hackathon.slug,
            }))
        }
    }
}

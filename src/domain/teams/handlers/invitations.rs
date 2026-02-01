use crate::domain::teams::types::*;
use dioxus::prelude::*;
use serde_json::Value as JsonValue;

#[cfg(feature = "server")]
use crate::domain::applications::repository::ApplicationRepository;

#[cfg(feature = "server")]
use tracing::info;

#[cfg(feature = "server")]
use crate::core::auth::{
    context::RequestContext, middleware::SyncedUser, permissions::Permissions,
};
#[cfg(feature = "server")]
use chrono::Utc;
#[cfg(feature = "server")]
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

/// Send a team invitation to a user
#[cfg_attr(feature = "server", utoipa::path(
    post,
    path = "/api/hackathons/{slug}/team/invite",
    params(
        ("slug" = String, Path, description = "Hackathon slug")
    ),
    request_body = SendInvitationRequest,
    responses(
        (status = 200, description = "Invitation sent successfully"),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Hackathon or user not found"),
        (status = 500, description = "Server error")
    ),
    tag = "teams"
))]
#[post("/api/hackathons/:slug/team/invite", user: SyncedUser)]
pub async fn send_invitation(
    slug: String,
    req: SendInvitationRequest,
) -> Result<(), ServerFnError> {
    use crate::domain::teams::repository::TeamRepository;
    use serde_json::Value as JsonValue;

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    let hackathon = ctx.hackathon()?;

    // Get sender's team_id
    let team_repo = TeamRepository::new(&ctx.state.db);
    let sender_role = team_repo
        .find_user_role_or_error(
            ctx.user.id,
            hackathon.id,
            "User not registered for this hackathon",
        )
        .await?;

    let team_id = sender_role
        .team_id
        .ok_or_else(|| ServerFnError::new("You must be in a team to send invitations"))?;

    // Verify sender is the team owner
    Permissions::require_team_ownership(&ctx, team_id).await?;

    // Verify target user is registered for this hackathon and has no team
    let target_role = team_repo
        .find_user_role_or_error(
            req.user_id,
            hackathon.id,
            "User not registered for this hackathon",
        )
        .await?;

    if target_role.team_id.is_some() {
        return Err(ServerFnError::new("User is already in a team"));
    }

    // Check if invitation already exists
    let existing_invitation = crate::entities::prelude::TeamInvitations::find()
        .filter(crate::entities::team_invitations::Column::TeamId.eq(team_id))
        .filter(crate::entities::team_invitations::Column::UserId.eq(req.user_id))
        .one(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to check existing invitation: {}", e)))?;

    if existing_invitation.is_some() {
        return Err(ServerFnError::new("Invitation already sent to this user"));
    }

    // Fetch invited user info so we can snapshot their details
    let invited_user = crate::entities::prelude::Users::find_by_id(req.user_id)
        .one(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch invited user: {}", e)))?
        .ok_or_else(|| ServerFnError::new("Invited user not found"))?;

    let app_repo = ApplicationRepository::new(&ctx.state.db);

    // Extract major/graduation_year from invited user's application (if any)
    let mut major: Option<String> = None;
    let mut graduation_year: Option<String> = None;
    if let Ok(Some(app)) = app_repo
        .find_by_user_and_hackathon(req.user_id, hackathon.id)
        .await
    {
        let mut form_json: JsonValue = app.form_data.clone();
        if form_json.is_string() {
            if let Some(s) = form_json.as_str() {
                if let Ok(parsed) = serde_json::from_str::<JsonValue>(s) {
                    form_json = parsed;
                }
            }
        }

        let mut extract_string = |val: &JsonValue| -> Option<String> {
            if val.is_string() {
                val.as_str().map(|s| s.to_string())
            } else if val.is_number() {
                Some(val.to_string())
            } else {
                None
            }
        };

        if let Some(obj) = form_json.as_object() {
            if let Some(v) = obj.get("major") {
                major = extract_string(v);
            }
            if let Some(v) = obj.get("Major") {
                major = major.or_else(|| extract_string(v));
            }
            if let Some(v) = obj.get("graduation_year") {
                graduation_year = extract_string(v);
            }
            if let Some(v) = obj.get("graduationYear") {
                graduation_year = graduation_year.or_else(|| extract_string(v));
            }
            if let Some(v) = obj.get("graduation") {
                graduation_year = graduation_year.or_else(|| extract_string(v));
            }
        } else if let Some(arr) = form_json.as_array() {
            for entry in arr.iter() {
                if let Some(obj) = entry.as_object() {
                    if let Some(name_val) = obj
                        .get("name")
                        .or_else(|| obj.get("field"))
                        .or_else(|| obj.get("label"))
                    {
                        if let Some(name_str) = name_val.as_str() {
                            if name_str.eq_ignore_ascii_case("major") && major.is_none() {
                                if let Some(v) = obj.get("value") {
                                    major = extract_string(v);
                                }
                            }
                            if (name_str.eq_ignore_ascii_case("graduation_year")
                                || name_str.eq_ignore_ascii_case("graduationYear")
                                || name_str.eq_ignore_ascii_case("graduation"))
                                && graduation_year.is_none()
                            {
                                if let Some(v) = obj.get("value") {
                                    graduation_year = extract_string(v);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Fallback recursion
        fn find_by_key_substring(v: &JsonValue, subs: &[&str]) -> Option<String> {
            match v {
                JsonValue::Object(map) => {
                    for (k, val) in map.iter() {
                        let lk = k.to_lowercase();
                        for sub in subs.iter() {
                            if lk.contains(&sub.to_lowercase()) {
                                if val.is_string() {
                                    return val.as_str().map(|s| s.to_string());
                                }
                                if val.is_number() {
                                    return Some(val.to_string());
                                }
                            }
                        }
                        if let Some(found) = find_by_key_substring(val, subs) {
                            return Some(found);
                        }
                    }
                    None
                }
                JsonValue::Array(arr) => {
                    for item in arr.iter() {
                        if let Some(found) = find_by_key_substring(item, subs) {
                            return Some(found);
                        }
                    }
                    None
                }
                _ => None,
            }
        }

        if major.is_none() {
            major = find_by_key_substring(&form_json, &["major", "program"]);
        }
        if graduation_year.is_none() {
            graduation_year = find_by_key_substring(&form_json, &["graduation", "grad", "year"]);
        }
    }

    // Create invitation and snapshot invited user's info
    let invitation = crate::entities::team_invitations::ActiveModel {
        team_id: sea_orm::Set(team_id),
        user_id: sea_orm::Set(req.user_id),
        message: sea_orm::Set(req.message),
        person_name: sea_orm::Set(invited_user.name.clone()),
        person_email: sea_orm::Set(Some(invited_user.email.clone())),
        person_picture: sea_orm::Set(invited_user.picture.clone()),
        person_major: sea_orm::Set(major.clone()),
        person_graduation_year: sea_orm::Set(graduation_year.clone()),
        created_at: sea_orm::Set(Utc::now().naive_utc()),
        ..Default::default()
    };

    invitation
        .insert(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to create invitation: {}", e)))?;

    Ok(())
}

/// Get invitations for the current user
#[cfg_attr(feature = "server", utoipa::path(
    get,
    path = "/api/hackathons/{slug}/team/invitations",
    params(
        ("slug" = String, Path, description = "Hackathon slug")
    ),
    responses(
        (status = 200, description = "Invitations retrieved successfully", body = Vec<InvitationResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Hackathon not found"),
        (status = 500, description = "Server error")
    ),
    tag = "teams"
))]
#[get("/api/hackathons/:slug/team/invitations", user: SyncedUser)]
pub async fn get_my_invitations(slug: String) -> Result<Vec<InvitationResponse>, ServerFnError> {
    use crate::domain::teams::repository::TeamRepository;

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    let hackathon = ctx.hackathon()?;

    // Fetch invitations for current user
    let invitations = crate::entities::prelude::TeamInvitations::find()
        .filter(crate::entities::team_invitations::Column::UserId.eq(ctx.user.id))
        .all(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch invitations: {}", e)))?;

    let team_repo = TeamRepository::new(&ctx.state.db);
    let mut result = Vec::new();
    let app_repo = ApplicationRepository::new(&ctx.state.db);

    for invitation in invitations {
        // Fetch team details
        let team = team_repo.find_by_id(invitation.team_id).await?;

        // Check if team belongs to this hackathon
        if team.hackathon_id != hackathon.id {
            continue;
        }

        // Fetch invited user info (current record)
        let invited_user = crate::entities::prelude::Users::find_by_id(invitation.user_id)
            .one(&ctx.state.db)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to fetch invited user: {}", e)))?
            .ok_or_else(|| ServerFnError::new("Invited user not found"))?;

        // Prefer snapshotted person fields if present
        let person_name = invitation.person_name.clone();
        let person_email = invitation.person_email.clone();
        let person_picture = invitation.person_picture.clone();
        let person_major = invitation.person_major.clone();
        let person_graduation_year = invitation.person_graduation_year.clone();

        // If we didn't snapshot major/grad, try to parse current application data
        let mut major = person_major.clone();
        let mut graduation_year = person_graduation_year.clone();
        if major.is_none() || graduation_year.is_none() {
            if let Ok(Some(app)) = app_repo
                .find_by_user_and_hackathon(invitation.user_id, hackathon.id)
                .await
            {
                let mut form_json: JsonValue = app.form_data.clone();
                if form_json.is_string() {
                    if let Some(s) = form_json.as_str() {
                        if let Ok(parsed) = serde_json::from_str::<JsonValue>(s) {
                            form_json = parsed;
                        }
                    }
                }

                let mut extract_string = |val: &JsonValue| -> Option<String> {
                    if val.is_string() {
                        val.as_str().map(|s| s.to_string())
                    } else if val.is_number() {
                        Some(val.to_string())
                    } else {
                        None
                    }
                };

                if let Some(obj) = form_json.as_object() {
                    if major.is_none() {
                        if let Some(v) = obj.get("major") {
                            major = extract_string(v);
                        }
                    }
                    if major.is_none() {
                        if let Some(v) = obj.get("Major") {
                            major = extract_string(v);
                        }
                    }
                    if graduation_year.is_none() {
                        if let Some(v) = obj.get("graduation_year") {
                            graduation_year = extract_string(v);
                        }
                    }
                    if graduation_year.is_none() {
                        if let Some(v) = obj.get("graduationYear") {
                            graduation_year = extract_string(v);
                        }
                    }
                    if graduation_year.is_none() {
                        if let Some(v) = obj.get("graduation") {
                            graduation_year = extract_string(v);
                        }
                    }
                } else if let Some(arr) = form_json.as_array() {
                    for entry in arr.iter() {
                        if let Some(obj) = entry.as_object() {
                            if let Some(name_val) = obj
                                .get("name")
                                .or_else(|| obj.get("field"))
                                .or_else(|| obj.get("label"))
                            {
                                if let Some(name_str) = name_val.as_str() {
                                    if major.is_none() && name_str.eq_ignore_ascii_case("major") {
                                        if let Some(v) = obj.get("value") {
                                            major = extract_string(v);
                                        }
                                    }
                                    if graduation_year.is_none()
                                        && (name_str.eq_ignore_ascii_case("graduation_year")
                                            || name_str.eq_ignore_ascii_case("graduationYear")
                                            || name_str.eq_ignore_ascii_case("graduation"))
                                    {
                                        if let Some(v) = obj.get("value") {
                                            graduation_year = extract_string(v);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                #[cfg(feature = "server")]
                info!(
                    "Parsed application form_data for invited user {}: major={:?}, graduation_year={:?}, raw={:?}",
                    invitation.user_id, major, graduation_year, form_json
                );
            }
        }

        result.push(InvitationResponse {
            id: invitation.id,
            team_id: invitation.team_id,
            team_name: team.name,
            user_id: invitation.user_id,
            user_name: person_name.or(invited_user.name),
            user_email: person_email.clone().unwrap_or(invited_user.email.clone()),
            user_picture: person_picture.or(invited_user.picture),
            major,
            graduation_year,
            message: invitation.message,
            created_at: invitation.created_at.to_string(),
        });
    }

    Ok(result)
}

/// Accept a team invitation
#[cfg_attr(feature = "server", utoipa::path(
    post,
    path = "/api/hackathons/{slug}/team/invitations/{invitation_id}/accept",
    params(
        ("slug" = String, Path, description = "Hackathon slug"),
        ("invitation_id" = i32, Path, description = "Invitation ID")
    ),
    responses(
        (status = 200, description = "Invitation accepted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Invitation not found"),
        (status = 500, description = "Server error")
    ),
    tag = "teams"
))]
#[post("/api/hackathons/:slug/team/invitations/:invitation_id/accept", user: SyncedUser)]
pub async fn accept_invitation(slug: String, invitation_id: i32) -> Result<(), ServerFnError> {
    use crate::domain::teams::repository::TeamRepository;

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    let hackathon = ctx.hackathon()?;

    // Verify the invitation is for the current user
    Permissions::require_invitation_ownership(&ctx, invitation_id).await?;

    // Fetch invitation
    let invitation = crate::entities::prelude::TeamInvitations::find_by_id(invitation_id)
        .one(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch invitation: {}", e)))?
        .ok_or_else(|| ServerFnError::new("Invitation not found"))?;

    // Verify user doesn't already have a team
    let team_repo = TeamRepository::new(&ctx.state.db);
    let user_role = team_repo
        .find_user_role_or_error(
            ctx.user.id,
            hackathon.id,
            "User not registered for this hackathon",
        )
        .await?;

    if user_role.team_id.is_some() {
        return Err(ServerFnError::new("You are already in a team"));
    }

    // Check if team is full
    let team_members_count = team_repo
        .count_team_members(invitation.team_id, hackathon.id)
        .await?;

    if team_members_count >= hackathon.max_team_size as usize {
        return Err(ServerFnError::new("Team is full"));
    }

    // Update user's team_id
    let mut user_role_active: crate::entities::user_hackathon_roles::ActiveModel = user_role.into();
    user_role_active.team_id = sea_orm::Set(Some(invitation.team_id));
    user_role_active
        .update(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to update user role: {}", e)))?;

    // Delete the invitation
    let invitation_to_delete: crate::entities::team_invitations::ActiveModel = invitation.into();
    invitation_to_delete
        .delete(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to delete invitation: {}", e)))?;

    Ok(())
}

/// Decline a team invitation
#[cfg_attr(feature = "server", utoipa::path(
    post,
    path = "/api/hackathons/{slug}/team/invitations/{invitation_id}/decline",
    params(
        ("slug" = String, Path, description = "Hackathon slug"),
        ("invitation_id" = i32, Path, description = "Invitation ID")
    ),
    responses(
        (status = 200, description = "Invitation declined successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Invitation not found"),
        (status = 500, description = "Server error")
    ),
    tag = "teams"
))]
#[post("/api/hackathons/:slug/team/invitations/:invitation_id/decline", user: SyncedUser)]
pub async fn decline_invitation(slug: String, invitation_id: i32) -> Result<(), ServerFnError> {
    let ctx = RequestContext::extract(&user).await?;

    // Verify the invitation is for the current user
    Permissions::require_invitation_ownership(&ctx, invitation_id).await?;

    // Fetch invitation
    let invitation = crate::entities::prelude::TeamInvitations::find_by_id(invitation_id)
        .one(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch invitation: {}", e)))?
        .ok_or_else(|| ServerFnError::new("Invitation not found"))?;

    // Delete the invitation
    let invitation_to_delete: crate::entities::team_invitations::ActiveModel = invitation.into();
    invitation_to_delete
        .delete(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to delete invitation: {}", e)))?;

    Ok(())
}

use dioxus::prelude::ServerFnError;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use crate::AppState;
#[cfg(feature = "server")]
use crate::core::auth::{
    context::RequestContext, middleware::SyncedUser, permissions::Permissions,
};
#[cfg(feature = "server")]
use crate::domain::messages::{message_groups, messages};
#[cfg(feature = "server")]
use crate::entities::prelude::*;
#[cfg(feature = "server")]
use axum::Json;
#[cfg(feature = "server")]
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
#[cfg(feature = "server")]
use utoipa::ToSchema;

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct MessageResponse {
    pub id: i32,
    pub sender_user_id: i32,
    pub message_group_id: i32,
    pub content: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct CreateMessageRequest {
    pub sender_user_id: i32,
    /// recipient_id: NULL for everyone, or the id of the team/user depending on recipient_type
    pub recipient_id: Option<i32>,
    /// "all" | "team" | "user"
    pub recipient_type: Option<String>,
    pub content: String,
}

/// Create a new message (admins/organizers only)
#[cfg_attr(feature = "server", utoipa::path(
	post,
	path = "/api/hackathons/{slug}/messages",
	params(("slug" = String, Path, description = "Hackathon slug")),
	request_body = CreateMessageRequest,
	responses(
		(status = 200, description = "Message created", body = MessageResponse),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Forbidden")
	),
	tag = "messages"
))]
#[post("/api/hackathons/:slug/messages", user: SyncedUser)]
pub async fn create_message(slug: String, req: CreateMessageRequest) -> Result<(), ServerFnError> {
    use sea_orm::Set;

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    // check admin/organizer
    Permissions::require_admin_or_organizer(&ctx).await?;

    // normalize recipient_type
    let recipient_type = req.recipient_type.clone().unwrap_or_else(|| {
        if req.recipient_id.is_none() {
            "all".to_string()
        } else {
            "user".to_string()
        }
    });

    // find or create message_group
    let existing = crate::domain::messages::message_groups::Entity::find()
        .filter(
            crate::domain::messages::message_groups::Column::RecipientType
                .eq(recipient_type.clone()),
        )
        .filter(crate::domain::messages::message_groups::Column::RecipientId.eq(req.recipient_id))
        .one(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("DB error: {}", e)))?;

    let group_id = if let Some(g) = existing {
        g.id
    } else {
        let new_group = message_groups::ActiveModel {
            flag: Set("admin_message".to_string()),
            recipient_id: Set(req.recipient_id),
            recipient_type: Set(recipient_type.clone()),
            created_at: Set(chrono::Utc::now().naive_utc()),
            updated_at: Set(chrono::Utc::now().naive_utc()),
            ..Default::default()
        };

        let inserted = new_group
            .insert(&ctx.state.db)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to create message group: {}", e)))?;
        inserted.id
    };

    // create message
    let new_msg = messages::ActiveModel {
        sender_user_id: Set(req.sender_user_id),
        message_group_id: Set(group_id),
        content: Set(req.content),
        created_at: Set(chrono::Utc::now().naive_utc()),
        updated_at: Set(chrono::Utc::now().naive_utc()),
        ..Default::default()
    };

    let _ = new_msg
        .insert(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to create message: {}", e)))?;

    Ok(())
}

/// Get all messages visible to a user (everyone, to the user, or to their team)
#[cfg_attr(feature = "server", utoipa::path(
	get,
	path = "/api/hackathons/{slug}/messages/user/{user_id}",
	params(("slug" = String, Path, description = "Hackathon slug"), ("user_id" = i32, Path, description = "User ID")),
	responses((status = 200, description = "Messages for user", body = Vec<MessageResponse>)),
	tag = "messages"
))]
#[get("/api/hackathons/:slug/messages/user/:user_id", user: SyncedUser)]
pub async fn get_messages(
    slug: String,
    user_id: i32,
) -> Result<Vec<MessageResponse>, ServerFnError> {
    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    // allow if requester is the user or admin/organizer
    if ctx.user.id != user_id {
        Permissions::require_admin_or_organizer(&ctx).await?;
    }

    // find user's team for this hackathon
    let role = crate::entities::prelude::UserHackathonRoles::find()
        .filter(crate::entities::user_hackathon_roles::Column::UserId.eq(user_id))
        .filter(crate::entities::user_hackathon_roles::Column::HackathonId.eq(ctx.hackathon()?.id))
        .one(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("DB error: {}", e)))?;

    let team_id_opt = role.and_then(|r| r.team_id);

    // collect message_group ids that match recipient_type/all/user/team
    let mut group_ids: Vec<i32> = Vec::new();

    // all
    let all_groups = message_groups::Entity::find()
        .filter(message_groups::Column::RecipientType.eq("all"))
        .all(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("DB error: {}", e)))?;
    group_ids.extend(all_groups.into_iter().map(|g| g.id));

    // user-specific
    let user_groups = crate::domain::messages::message_groups::Entity::find()
        .filter(crate::domain::messages::message_groups::Column::RecipientType.eq("user"))
        .filter(crate::domain::messages::message_groups::Column::RecipientId.eq(user_id))
        .all(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("DB error: {}", e)))?;
    group_ids.extend(user_groups.into_iter().map(|g| g.id));

    // team-specific
    if let Some(team_id) = team_id_opt {
        let team_groups = crate::domain::messages::message_groups::Entity::find()
            .filter(crate::domain::messages::message_groups::Column::RecipientType.eq("team"))
            .filter(crate::domain::messages::message_groups::Column::RecipientId.eq(team_id))
            .all(&ctx.state.db)
            .await
            .map_err(|e| ServerFnError::new(format!("DB error: {}", e)))?;
        group_ids.extend(team_groups.into_iter().map(|g| g.id));
    }

    if group_ids.is_empty() {
        return Ok(Vec::new());
    }

    let msgs = crate::domain::messages::messages::Entity::find()
        .filter(crate::domain::messages::messages::Column::MessageGroupId.is_in(group_ids))
        .all(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("DB error: {}", e)))?;

    let resp = msgs
        .into_iter()
        .map(|m| MessageResponse {
            id: m.id,
            sender_user_id: m.sender_user_id,
            message_group_id: m.message_group_id,
            content: m.content,
            created_at: m.created_at,
        })
        .collect();

    Ok(resp)
}

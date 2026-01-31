use dioxus::prelude::*;

#[cfg(feature = "server")]
use crate::core::auth::{
    context::RequestContext, middleware::SyncedUser, permissions::Permissions,
};

/// Get user schedule
#[cfg_attr(feature = "server", utoipa::path(
    get,
    path = "/api/hackathons/{slug}/schedule",
    params(
        ("slug" = String, Path, description = "Hackathon slug")
    ),
    responses(
        (status = 200, description = "Schedule retrieved successfully", body = Vec<crate::domain::hackathons::types::ScheduleEvent>),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Hackathon not found"),
        (status = 500, description = "Server error")
    ),
    tag = "applications"
))]
#[get("/api/hackathons/:slug/schedule", user: SyncedUser)]
pub async fn get_user_schedule(
    slug: String,
) -> Result<Vec<crate::domain::hackathons::types::ScheduleEvent>, ServerFnError> {
    use crate::domain::hackathons::repository::HackathonRepository;
    use crate::domain::people::repository::UserRoleRepository;

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    // Fetch user role
    let role_repo = UserRoleRepository::new(&ctx.state.db);
    let user_role = role_repo
        .find_user_role(ctx.user.id, ctx.hackathon()?.id)
        .await?;
    let role_str = user_role.as_ref().map(|r| r.role.as_str());
    let is_admin = user_role
        .as_ref()
        .map(|r| r.role.as_str() == "admin")
        .unwrap_or(false);

    let repo = HackathonRepository::new(&ctx.state.db);

    repo.get_schedule(&slug, role_str, is_admin, ctx.user.id)
        .await
}

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use utoipa::ToSchema;

/// Request payload for creating an event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct CreateEventRequest {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    /// NULL = visible to everyone, otherwise the role required to view
    pub visible_to_role: Option<String>,
    /// Event type for color coding: default, hacking, speaker, sponsor, food
    pub event_type: String,
    /// Whether the event is visible (published) or hidden (draft)
    pub is_visible: bool,
    /// User IDs of organizers assigned to this event
    pub organizer_ids: Vec<i32>,
    /// Optional points value for gamification
    pub points: Option<i32>,
    /// Check-in type: 'self_checkin' or 'qr_scan'
    pub checkin_type: String,
}

/// Create a new event (admin only)
#[cfg_attr(feature = "server", utoipa::path(
    post,
    path = "/api/hackathons/{slug}/events",
    params(
        ("slug" = String, Path, description = "Hackathon slug")
    ),
    request_body = CreateEventRequest,
    responses(
        (status = 201, description = "Event created successfully", body = crate::domain::hackathons::types::ScheduleEvent),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - admin only"),
        (status = 404, description = "Hackathon not found"),
        (status = 500, description = "Server error")
    ),
    tag = "events"
))]
#[post("/api/hackathons/:slug/events", user: SyncedUser)]
pub async fn create_event(
    slug: String,
    request: CreateEventRequest,
) -> Result<crate::domain::hackathons::types::ScheduleEvent, ServerFnError> {
    use crate::domain::people::repository::UserRoleRepository;
    use crate::entities::events;
    use chrono::Utc;
    use sea_orm::{ActiveModelTrait, Set};

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    let hackathon = ctx.hackathon()?;

    // Check if user is admin (global or hackathon-level)
    let is_global_admin = Permissions::is_global_admin(&ctx);
    let role_repo = UserRoleRepository::new(&ctx.state.db);
    let is_hackathon_admin = role_repo.is_admin(ctx.user.id, hackathon.id).await?;

    if !is_global_admin && !is_hackathon_admin {
        return Err(ServerFnError::new("Only admins can create events"));
    }

    // Create the event
    let now = Utc::now().naive_utc();
    let event = events::ActiveModel {
        hackathon_id: Set(hackathon.id),
        name: Set(request.name.clone()),
        slug: Set(request.slug.clone()),
        description: Set(request.description.clone()),
        location: Set(request.location.clone()),
        start_time: Set(request.start_time),
        end_time: Set(request.end_time),
        visible_to_role: Set(request.visible_to_role.clone()),
        event_type: Set(request.event_type.clone()),
        is_visible: Set(request.is_visible),
        points: Set(request.points),
        checkin_type: Set(request.checkin_type.clone()),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    let inserted = event
        .insert(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to create event: {}", e)))?;

    // Insert event organizers
    for user_id in &request.organizer_ids {
        use crate::entities::event_organizers;
        let organizer = event_organizers::ActiveModel {
            event_id: Set(inserted.id),
            user_id: Set(*user_id),
            created_at: Set(now),
            ..Default::default()
        };
        organizer
            .insert(&ctx.state.db)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to add event organizer: {}", e)))?;
    }

    Ok(crate::domain::hackathons::types::ScheduleEvent {
        id: inserted.id,
        name: inserted.name,
        description: inserted.description,
        location: request.location,
        start_time: inserted.start_time,
        end_time: inserted.end_time,
        visible_to_role: inserted.visible_to_role,
        event_type: inserted.event_type,
        is_visible: request.is_visible,
        organizer_ids: request.organizer_ids,
        points: inserted.points,
        checkin_type: inserted.checkin_type,
        is_checked_in: false,
        required_for_prizes: Vec::new(),
    })
}

/// Request payload for updating an event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct UpdateEventRequest {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub visible_to_role: Option<String>,
    pub event_type: String,
    pub is_visible: bool,
    pub organizer_ids: Vec<i32>,
    /// Optional points value for gamification
    pub points: Option<i32>,
    /// Check-in type: 'self_checkin' or 'qr_scan'
    pub checkin_type: String,
}

/// Update an existing event (admin only)
#[cfg_attr(feature = "server", utoipa::path(
    put,
    path = "/api/hackathons/{slug}/events/{id}",
    params(
        ("slug" = String, Path, description = "Hackathon slug"),
        ("id" = i32, Path, description = "Event ID")
    ),
    request_body = UpdateEventRequest,
    responses(
        (status = 200, description = "Event updated successfully", body = crate::domain::hackathons::types::ScheduleEvent),
        (status = 401, description = "Only admins can update events"),
        (status = 404, description = "Event not found"),
        (status = 500, description = "Server error")
    ),
    tag = "events"
))]
#[put("/api/hackathons/:slug/events", user: SyncedUser)]
pub async fn update_event(
    slug: String,
    request: UpdateEventRequest,
) -> Result<crate::domain::hackathons::types::ScheduleEvent, ServerFnError> {
    use crate::domain::people::repository::UserRoleRepository;
    use crate::entities::event_organizers;
    use crate::entities::events;
    use chrono::Utc;
    use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, ModelTrait, QueryFilter, Set};

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    let hackathon = ctx.hackathon()?;

    // Check if user is admin (global or hackathon-level)
    let is_global_admin = Permissions::is_global_admin(&ctx);
    let role_repo = UserRoleRepository::new(&ctx.state.db);
    let is_hackathon_admin = role_repo.is_admin(ctx.user.id, hackathon.id).await?;

    if !is_global_admin && !is_hackathon_admin {
        return Err(ServerFnError::new("Only admins can update events"));
    }

    // Find the event
    let existing = events::Entity::find_by_id(request.id)
        .one(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to find event: {}", e)))?
        .ok_or_else(|| ServerFnError::new("Event not found"))?;

    // Update the event
    let now = Utc::now().naive_utc();
    let mut event_model: events::ActiveModel = existing.into();
    event_model.name = Set(request.name.clone());
    event_model.description = Set(request.description.clone());
    event_model.location = Set(request.location.clone());
    event_model.start_time = Set(request.start_time);
    event_model.end_time = Set(request.end_time);
    event_model.visible_to_role = Set(request.visible_to_role.clone());
    event_model.event_type = Set(request.event_type.clone());
    event_model.is_visible = Set(request.is_visible);
    event_model.points = Set(request.points);
    event_model.checkin_type = Set(request.checkin_type.clone());
    event_model.updated_at = Set(now);

    let updated = event_model
        .update(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to update event: {}", e)))?;

    // Delete existing organizers and re-insert
    event_organizers::Entity::delete_many()
        .filter(event_organizers::Column::EventId.eq(request.id))
        .exec(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to update organizers: {}", e)))?;

    for user_id in &request.organizer_ids {
        let organizer = event_organizers::ActiveModel {
            event_id: Set(updated.id),
            user_id: Set(*user_id),
            created_at: Set(now),
            ..Default::default()
        };
        organizer
            .insert(&ctx.state.db)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to add event organizer: {}", e)))?;
    }

    Ok(crate::domain::hackathons::types::ScheduleEvent {
        id: updated.id,
        name: updated.name,
        description: updated.description,
        location: request.location,
        start_time: updated.start_time,
        end_time: updated.end_time,
        visible_to_role: updated.visible_to_role,
        event_type: updated.event_type,
        is_visible: request.is_visible,
        organizer_ids: request.organizer_ids,
        points: updated.points,
        checkin_type: updated.checkin_type,
        is_checked_in: false,
        required_for_prizes: Vec::new(),
    })
}

/// Delete an event (admin only)
#[cfg_attr(feature = "server", utoipa::path(
    delete,
    path = "/api/hackathons/{slug}/events/{id}",
    params(
        ("slug" = String, Path, description = "Hackathon slug"),
        ("id" = i32, Path, description = "Event ID")
    ),
    responses(
        (status = 200, description = "Event deleted successfully"),
        (status = 401, description = "Only admins can delete events"),
        (status = 404, description = "Event not found"),
        (status = 500, description = "Server error")
    ),
    tag = "events"
))]
#[delete("/api/hackathons/:slug/events/:id", user: SyncedUser)]
pub async fn delete_event(slug: String, id: i32) -> Result<(), ServerFnError> {
    use crate::domain::people::repository::UserRoleRepository;
    use crate::entities::events;
    use sea_orm::{EntityTrait, ModelTrait};

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    let hackathon = ctx.hackathon()?;

    // Check if user is admin (global or hackathon-level)
    let is_global_admin = Permissions::is_global_admin(&ctx);
    let role_repo = UserRoleRepository::new(&ctx.state.db);
    let is_hackathon_admin = role_repo.is_admin(ctx.user.id, hackathon.id).await?;

    if !is_global_admin && !is_hackathon_admin {
        return Err(ServerFnError::new("Only admins can delete events"));
    }

    // Find and delete the event (cascade will delete organizers)
    let event = events::Entity::find_by_id(id)
        .one(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to find event: {}", e)))?
        .ok_or_else(|| ServerFnError::new("Event not found"))?;

    event
        .delete(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to delete event: {}", e)))?;

    Ok(())
}

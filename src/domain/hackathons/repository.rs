#[cfg(feature = "server")]
use crate::core::database::repository::Repository;
#[cfg(feature = "server")]
use crate::entities::hackathons;
#[cfg(feature = "server")]
use crate::entities::prelude::*;
#[cfg(feature = "server")]
use dioxus::prelude::ServerFnError;
#[cfg(feature = "server")]
use sea_orm::{ColumnTrait, DatabaseConnection, QueryFilter};

#[cfg(feature = "server")]
use super::types::HackathonInfo;

#[cfg(feature = "server")]
pub struct HackathonRepository<'a> {
    repo: Repository<'a>,
}

#[cfg(feature = "server")]
impl<'a> HackathonRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self {
            repo: Repository::new(db),
        }
    }

    /// Find hackathon by slug
    pub async fn find_by_slug(
        &self,
        slug: &str,
    ) -> Result<Option<hackathons::Model>, ServerFnError> {
        self.repo
            .find_one::<Hackathons, _>(|query| query.filter(hackathons::Column::Slug.eq(slug)))
            .await
    }

    /// Find hackathon by slug or return error
    pub async fn find_by_slug_or_error(
        &self,
        slug: &str,
    ) -> Result<hackathons::Model, ServerFnError> {
        self.repo
            .find_one_or_error::<Hackathons, _>(
                |query| query.filter(hackathons::Column::Slug.eq(slug)),
                "Hackathon not found",
            )
            .await
    }

    /// Get all hackathons as domain types
    pub async fn get_all(&self) -> Result<Vec<HackathonInfo>, ServerFnError> {
        let hackathons = self.repo.find_all::<Hackathons, _>(|query| query).await?;
        Ok(hackathons.into_iter().map(|h| h.into()).collect())
    }

    /// Get schedule for a hackathon
    /// - Admins see all events
    /// - Others see events where visible_to_role is NULL or matches their role AND is_visible is true
    pub async fn get_schedule(
        &self,
        slug: &str,
        user_role: Option<&str>,
        is_admin: bool,
        user_id: i32,
    ) -> Result<Vec<crate::domain::hackathons::types::ScheduleEvent>, ServerFnError> {
        use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder};

        let hackathon = self.find_by_slug_or_error(slug).await?;

        // Find all events for this hackathon
        let events = self
            .repo
            .find_all::<crate::entities::events::Entity, _>(|query| {
                let base_query = query
                    .filter(crate::entities::events::Column::HackathonId.eq(hackathon.id))
                    .order_by_asc(crate::entities::events::Column::StartTime);

                // Admins see all events
                if is_admin {
                    return base_query;
                }

                // Others see events where visible_to_role is NULL or matches their role
                let role_condition =
                    Condition::any().add(crate::entities::events::Column::VisibleToRole.is_null());
                let role_condition = if let Some(role) = user_role {
                    role_condition.add(crate::entities::events::Column::VisibleToRole.eq(role))
                } else {
                    role_condition
                };

                base_query
                    .filter(role_condition)
                    .filter(crate::entities::events::Column::IsVisible.eq(true))
            })
            .await?;

        // Fetch organizers for all events
        let event_ids: Vec<i32> = events.iter().map(|e| e.id).collect();
        let organizers = crate::entities::event_organizers::Entity::find()
            .filter(crate::entities::event_organizers::Column::EventId.is_in(event_ids.clone()))
            .all(self.repo.db())
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to fetch organizers: {}", e)))?;

        // Fetch user's check-ins for these events
        let checkins = crate::entities::event_checkins::Entity::find()
            .filter(crate::entities::event_checkins::Column::EventId.is_in(event_ids.clone()))
            .filter(crate::entities::event_checkins::Column::UserId.eq(user_id))
            .all(self.repo.db())
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to fetch checkins: {}", e)))?;

        // Create set of checked-in event IDs
        let checked_in_event_ids: std::collections::HashSet<i32> =
            checkins.iter().map(|c| c.event_id).collect();

        // Group organizers by event_id
        let mut organizer_map: std::collections::HashMap<i32, Vec<i32>> =
            std::collections::HashMap::new();
        for org in organizers {
            organizer_map
                .entry(org.event_id)
                .or_default()
                .push(org.user_id);
        }

        // Fetch required prizes for these events
        let required_prizes = crate::entities::prize_required_events::Entity::find()
            .filter(
                crate::entities::prize_required_events::Column::EventId.is_in(event_ids.clone()),
            )
            .find_also_related(crate::entities::prize::Entity)
            .all(self.repo.db())
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to fetch required prizes: {}", e)))?;

        // Group required prizes by event_id
        let mut required_prizes_map: std::collections::HashMap<i32, Vec<String>> =
            std::collections::HashMap::new();
        for (req, prize) in required_prizes {
            if let Some(p) = prize {
                required_prizes_map
                    .entry(req.event_id)
                    .or_default()
                    .push(p.name);
            }
        }

        // Map events with their organizers, check-in status, and required prizes
        Ok(events
            .into_iter()
            .map(|e| {
                let org_ids = organizer_map.get(&e.id).cloned().unwrap_or_default();
                let is_checked_in = checked_in_event_ids.contains(&e.id);
                let required_for_prizes =
                    required_prizes_map.get(&e.id).cloned().unwrap_or_default();
                crate::domain::hackathons::types::ScheduleEvent {
                    id: e.id,
                    name: e.name,
                    description: e.description,
                    location: e.location,
                    start_time: e.start_time,
                    end_time: e.end_time,
                    visible_to_role: e.visible_to_role,
                    event_type: e.event_type,
                    is_visible: e.is_visible,
                    organizer_ids: org_ids,
                    points: e.points,
                    checkin_type: e.checkin_type,
                    is_checked_in,
                    required_for_prizes,
                }
            })
            .collect())
    }
}

// Conversion from entity to domain type
#[cfg(feature = "server")]
impl From<hackathons::Model> for HackathonInfo {
    fn from(h: hackathons::Model) -> Self {
        HackathonInfo {
            id: h.id,
            name: h.name,
            slug: h.slug,
            description: h.description,
            start_date: h.start_date,
            end_date: h.end_date,
            is_active: h.is_active,
            max_team_size: h.max_team_size,
            banner_url: h.banner_url,
            background_url: h.background_url,
            updated_at: h.updated_at,
            form_config: h.form_config,
            submission_form: h.submission_form,
            app_icon_url: h.app_icon_url,
            theme_color: h.theme_color,
            background_color: h.background_color,
            submissions_closed: h.submissions_closed,
            judging_started: h.judging_started,
            judge_session_timeout_minutes: h.judge_session_timeout_minutes,
        }
    }
}

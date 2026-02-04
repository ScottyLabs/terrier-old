//! Judging lifecycle management: start/stop judging, submission status.

use crate::domain::judging::types::*;
use dioxus::prelude::*;

#[cfg(feature = "server")]
use crate::core::auth::{
    context::RequestContext, middleware::SyncedUser, permissions::Permissions,
};

/// Close submissions for a hackathon (prerequisite for starting judging)
#[cfg_attr(feature = "server", utoipa::path(
    post,
    path = "/api/hackathons/{slug}/judging/close-submissions",
    params(
        ("slug" = String, Path, description = "Hackathon slug")
    ),
    responses(
        (status = 200, description = "Submissions closed successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Server error")
    ),
    tag = "judging"
))]
#[post("/api/hackathons/:slug/judging/close-submissions", user: SyncedUser)]
pub async fn close_submissions(slug: String) -> Result<(), ServerFnError> {
    use crate::entities::hackathons;
    use sea_orm::{ActiveModelTrait, EntityTrait, Set};

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    Permissions::require_admin_or_organizer(&ctx).await?;
    let hackathon = ctx.hackathon()?;

    let mut active: hackathons::ActiveModel = hackathons::Entity::find_by_id(hackathon.id)
        .one(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch hackathon: {}", e)))?
        .ok_or_else(|| ServerFnError::new("Hackathon not found"))?
        .into();

    active.submissions_closed = Set(true);

    active
        .update(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to close submissions: {}", e)))?;

    Ok(())
}

/// Start judging for a hackathon (requires submissions to be closed)
#[cfg_attr(feature = "server", utoipa::path(
    post,
    path = "/api/hackathons/{slug}/judging/start",
    params(
        ("slug" = String, Path, description = "Hackathon slug")
    ),
    responses(
        (status = 200, description = "Judging started successfully"),
        (status = 400, description = "Submissions must be closed first"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Server error")
    ),
    tag = "judging"
))]
#[post("/api/hackathons/:slug/judging/start", user: SyncedUser)]
pub async fn start_judging(slug: String) -> Result<(), ServerFnError> {
    use crate::entities::hackathons;
    use sea_orm::{ActiveModelTrait, EntityTrait, Set};

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    Permissions::require_admin_or_organizer(&ctx).await?;
    let hackathon = ctx.hackathon()?;

    if !hackathon.submissions_closed {
        return Err(ServerFnError::new(
            "Submissions must be closed before starting judging",
        ));
    }

    let mut active: hackathons::ActiveModel = hackathons::Entity::find_by_id(hackathon.id)
        .one(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch hackathon: {}", e)))?
        .ok_or_else(|| ServerFnError::new("Hackathon not found"))?
        .into();

    active.judging_started = Set(true);

    active
        .update(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to start judging: {}", e)))?;

    Ok(())
}

/// Stop judging for a hackathon
#[cfg_attr(feature = "server", utoipa::path(
    post,
    path = "/api/hackathons/{slug}/judging/stop",
    params(
        ("slug" = String, Path, description = "Hackathon slug")
    ),
    responses(
        (status = 200, description = "Judging stopped successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Server error")
    ),
    tag = "judging"
))]
#[post("/api/hackathons/:slug/judging/stop", user: SyncedUser)]
pub async fn stop_judging(slug: String) -> Result<(), ServerFnError> {
    use crate::entities::hackathons;
    use sea_orm::{ActiveModelTrait, EntityTrait, Set};

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    Permissions::require_admin_or_organizer(&ctx).await?;
    let hackathon = ctx.hackathon()?;

    let mut active: hackathons::ActiveModel = hackathons::Entity::find_by_id(hackathon.id)
        .one(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch hackathon: {}", e)))?
        .ok_or_else(|| ServerFnError::new("Hackathon not found"))?
        .into();

    active.judging_started = Set(false);

    active
        .update(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to stop judging: {}", e)))?;

    Ok(())
}

/// Reset judging for a hackathon (clears all judging data)
#[cfg_attr(feature = "server", utoipa::path(
    post,
    path = "/api/hackathons/{slug}/judging/reset",
    params(
        ("slug" = String, Path, description = "Hackathon slug")
    ),
    responses(
        (status = 200, description = "Judging reset successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Server error")
    ),
    tag = "judging"
))]
#[post("/api/hackathons/:slug/judging/reset", user: SyncedUser)]
pub async fn reset_judging(slug: String) -> Result<(), ServerFnError> {
    use crate::entities::{feature, hackathons, pairwise_comparison, project_visit};
    use sea_orm::{
        ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QuerySelect, Set, TransactionTrait,
    };

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    Permissions::require_admin_or_organizer(&ctx).await?;
    let hackathon = ctx.hackathon()?;

    // Start transaction
    let txn = ctx
        .state
        .db
        .begin()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to start transaction: {}", e)))?;

    // 1. Reset hackathon state (stop judging if started)
    let mut active: hackathons::ActiveModel = hackathons::Entity::find_by_id(hackathon.id)
        .one(&txn)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch hackathon: {}", e)))?
        .ok_or_else(|| ServerFnError::new("Hackathon not found"))?
        .into();

    active.judging_started = Set(false);

    active
        .update(&txn)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to update hackathon: {}", e)))?;

    // 2. Delete all project visits for this hackathon
    project_visit::Entity::delete_many()
        .filter(project_visit::Column::HackathonId.eq(hackathon.id))
        .exec(&txn)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to delete visits: {}", e)))?;

    // 3. Delete all pairwise comparisons for this hackathon's features
    let feature_ids: Vec<i32> = feature::Entity::find()
        .filter(feature::Column::HackathonId.eq(hackathon.id))
        .select_only()
        .column(feature::Column::Id)
        .into_tuple::<i32>()
        .all(&txn)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch features: {}", e)))?;

    if !feature_ids.is_empty() {
        pairwise_comparison::Entity::delete_many()
            .filter(pairwise_comparison::Column::FeatureId.is_in(feature_ids.clone()))
            .exec(&txn)
            .await
            .map_err(|e| {
                ServerFnError::new(format!("Failed to delete pairwise comparisons: {}", e))
            })?;

        // 4. Delete all project feature scores
        crate::entities::project_feature_score::Entity::delete_many()
            .filter(
                crate::entities::project_feature_score::Column::FeatureId
                    .is_in(feature_ids.clone()),
            )
            .exec(&txn)
            .await
            .map_err(|e| {
                ServerFnError::new(format!("Failed to delete project feature scores: {}", e))
            })?;

        // 5. Reset judge assignments (clear best submission and notes)
        use crate::entities::judge_feature_assignment;

        let assignments = judge_feature_assignment::Entity::find()
            .filter(judge_feature_assignment::Column::FeatureId.is_in(feature_ids))
            .all(&txn)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to fetch assignments: {}", e)))?;

        for assignment in assignments {
            let mut active: judge_feature_assignment::ActiveModel = assignment.into();
            active.current_best_submission_id = Set(None);
            active.notes = Set(None);
            active.update(&txn).await.map_err(|e| {
                ServerFnError::new(format!("Failed to reset judge assignment: {}", e))
            })?;
        }
    }

    // Commit transaction
    txn.commit()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to commit transaction: {}", e)))?;

    Ok(())
}

/// Force recalculation of rankings for a hackathon
#[cfg_attr(feature = "server", utoipa::path(
    post,
    path = "/api/hackathons/{slug}/judging/recalculate",
    params(
        ("slug" = String, Path, description = "Hackathon slug")
    ),
    responses(
        (status = 200, description = "Rankings recalculated successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Server error")
    ),
    tag = "judging"
))]
#[post("/api/hackathons/:slug/judging/recalculate", user: SyncedUser)]
pub async fn recalculate_rankings(slug: String) -> Result<(), ServerFnError> {
    use crate::domain::judging::score::update_hackathon_rankings;

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    Permissions::require_admin_or_organizer(&ctx).await?;
    let hackathon = ctx.hackathon()?;

    update_hackathon_rankings(&ctx.state.db, hackathon.id).await?;

    Ok(())
}

/// Re-open submissions for a hackathon
#[cfg_attr(feature = "server", utoipa::path(
    post,
    path = "/api/hackathons/{slug}/judging/reopen-submissions",
    params(
        ("slug" = String, Path, description = "Hackathon slug")
    ),
    responses(
        (status = 200, description = "Submissions re-opened successfully"),
        (status = 400, description = "Judging already started"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Server error")
    ),
    tag = "judging"
))]
#[post("/api/hackathons/:slug/judging/reopen-submissions", user: SyncedUser)]
pub async fn reopen_submissions(slug: String) -> Result<(), ServerFnError> {
    use crate::entities::hackathons;
    use sea_orm::{ActiveModelTrait, EntityTrait, Set};

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    let hackathon = ctx.hackathon()?;

    // Cannot re-open if judging has already started
    if hackathon.judging_started {
        return Err(ServerFnError::new(
            "Cannot re-open submissions while judging is active",
        ));
    }

    let mut active: hackathons::ActiveModel = hackathons::Entity::find_by_id(hackathon.id)
        .one(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch hackathon: {}", e)))?
        .ok_or_else(|| ServerFnError::new("Hackathon not found"))?
        .into();

    active.submissions_closed = Set(false);

    active
        .update(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to re-open submissions: {}", e)))?;

    Ok(())
}

/// Get judging status for a hackathon
#[cfg_attr(feature = "server", utoipa::path(
    get,
    path = "/api/hackathons/{slug}/judging/status",
    params(
        ("slug" = String, Path, description = "Hackathon slug")
    ),
    responses(
        (status = 200, description = "Judging status retrieved", body = JudgingStatus),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Server error")
    ),
    tag = "judging"
))]
#[get("/api/hackathons/:slug/judging/status", user: SyncedUser)]
pub async fn get_judging_status(slug: String) -> Result<JudgingStatus, ServerFnError> {
    use crate::entities::{feature, pairwise_comparison, project_visit, submission, teams};
    use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect};

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    let hackathon = ctx.hackathon()?;

    // Count total submissions (via teams belonging to this hackathon)
    let hackathon_teams: Vec<(i32, String)> = teams::Entity::find()
        .filter(teams::Column::HackathonId.eq(hackathon.id))
        .select_only()
        .column(teams::Column::Id)
        .column(teams::Column::Name)
        .into_tuple::<(i32, String)>()
        .all(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch teams: {}", e)))?;

    let hackathon_team_ids: Vec<i32> = hackathon_teams.iter().map(|(id, _)| *id).collect();

    let total_submissions = if hackathon_team_ids.is_empty() {
        0u64
    } else {
        submission::Entity::find()
            .filter(submission::Column::TeamId.is_in(hackathon_team_ids.clone()))
            .count(&ctx.state.db)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to count submissions: {}", e)))?
    };

    // Count submissions with at least one visit
    let visited_submissions = project_visit::Entity::find()
        .filter(project_visit::Column::HackathonId.eq(hackathon.id))
        .select_only()
        .column(project_visit::Column::SubmissionId)
        .distinct()
        .count(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to count visited: {}", e)))?;

    // Count total visits
    let total_visits = project_visit::Entity::find()
        .filter(project_visit::Column::HackathonId.eq(hackathon.id))
        .count(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to count visits: {}", e)))?;

    // Count total comparisons
    let feature_ids: Vec<i32> = feature::Entity::find()
        .filter(feature::Column::HackathonId.eq(hackathon.id))
        .select_only()
        .column(feature::Column::Id)
        .into_tuple::<i32>()
        .all(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch features: {}", e)))?;

    let total_comparisons = if feature_ids.is_empty() {
        0u64
    } else {
        pairwise_comparison::Entity::find()
            .filter(pairwise_comparison::Column::FeatureId.is_in(feature_ids))
            .count(&ctx.state.db)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to count comparisons: {}", e)))?
    };

    // Count projects with tables assigned
    let projects_with_tables = if hackathon_team_ids.is_empty() {
        0u64
    } else {
        submission::Entity::find()
            .filter(submission::Column::TeamId.is_in(hackathon_team_ids.clone()))
            .filter(submission::Column::TableNumber.is_not_null())
            .count(&ctx.state.db)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to count assigned tables: {}", e)))?
    };

    // Find names of unassigned projects
    let unassigned_projects = if hackathon_team_ids.is_empty() {
        Vec::new()
    } else {
        let unassigned_subs = submission::Entity::find()
            .filter(submission::Column::TeamId.is_in(hackathon_team_ids))
            .filter(submission::Column::TableNumber.is_null())
            .all(&ctx.state.db)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to fetch unassigned: {}", e)))?;

        let team_names: std::collections::HashMap<i32, String> =
            hackathon_teams.into_iter().collect();

        let mut names = Vec::new();
        for sub in unassigned_subs {
            let team_name = team_names
                .get(&sub.team_id)
                .cloned()
                .unwrap_or_else(|| "Unknown Team".to_string());
            names.push(team_name);
        }
        names
    };

    Ok(JudgingStatus {
        submissions_closed: hackathon.submissions_closed,
        judging_started: hackathon.judging_started,
        total_submissions: total_submissions as i64,
        visited_submissions: visited_submissions as i64,
        total_visits: total_visits as i64,
        total_comparisons: total_comparisons as i64,
        projects_with_tables: projects_with_tables as i64,
        unassigned_projects,
    })
}

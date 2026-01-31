//! Judge assignment and visit management endpoints.

use crate::domain::judging::types::*;
use dioxus::prelude::*;

#[cfg(feature = "server")]
use crate::core::auth::{context::RequestContext, middleware::SyncedUser};

/// Request a new assignment for a judge
#[cfg_attr(feature = "server", utoipa::path(
    post,
    path = "/api/hackathons/{slug}/judging/request-assignment",
    params(
        ("slug" = String, Path, description = "Hackathon slug")
    ),
    responses(
        (status = 200, description = "Assignment created", body = JudgeAssignment),
        (status = 204, description = "No more projects available"),
        (status = 400, description = "Judging not active or judge has active assignment"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Server error")
    ),
    tag = "judging"
))]
#[post("/api/hackathons/:slug/judging/request-assignment", user: SyncedUser)]
pub async fn request_assignment(slug: String) -> Result<Option<JudgeAssignment>, ServerFnError> {
    use crate::entities::{project_visit, submission, teams};
    use sea_orm::{
        ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, EntityTrait, PaginatorTrait,
        QueryFilter, QuerySelect, Set, TransactionTrait,
    };

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    let hackathon = ctx.hackathon()?;

    // Check if judging is active
    if !hackathon.judging_started {
        return Err(ServerFnError::new("Judging has not started yet"));
    }

    // Check if judge already has an active assignment
    let active_visit = project_visit::Entity::find()
        .filter(project_visit::Column::JudgeId.eq(ctx.user.id))
        .filter(project_visit::Column::HackathonId.eq(hackathon.id))
        .filter(project_visit::Column::IsActive.eq(true))
        .one(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to check active visits: {}", e)))?;

    if let Some(visit) = active_visit {
        // Return the existing active assignment
        let sub = submission::Entity::find_by_id(visit.submission_id)
            .one(&ctx.state.db)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to fetch submission: {}", e)))?
            .ok_or_else(|| ServerFnError::new("Submission not found"))?;

        let team = teams::Entity::find_by_id(sub.team_id)
            .one(&ctx.state.db)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to fetch team: {}", e)))?
            .ok_or_else(|| ServerFnError::new("Team not found"))?;

        let timeout_seconds = hackathon.judge_session_timeout_minutes as i64 * 60;
        let elapsed = chrono::Utc::now()
            .naive_utc()
            .signed_duration_since(visit.start_time)
            .num_seconds();
        let remaining = (timeout_seconds - elapsed).max(0);

        return Ok(Some(JudgeAssignment {
            visit_id: visit.id,
            submission_id: sub.id,
            team_name: team.name,
            submission_data: sub.submission_data,
            start_time: visit.start_time.to_string(),
            time_remaining_seconds: remaining,
        }));
    }

    // Find an available submission within a transaction
    let txn = ctx
        .state
        .db
        .begin()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to start transaction: {}", e)))?;

    // Get all submissions for this hackathon
    let team_ids: Vec<i32> = teams::Entity::find()
        .filter(teams::Column::HackathonId.eq(hackathon.id))
        .select_only()
        .column(teams::Column::Id)
        .into_tuple::<i32>()
        .all(&txn)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch teams: {}", e)))?;

    let available_submissions = if team_ids.is_empty() {
        Vec::new()
    } else {
        submission::Entity::find()
            .filter(submission::Column::TeamId.is_in(team_ids))
            .all(&txn)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to fetch submissions: {}", e)))?
    };

    // Filter out submissions already visited by this judge or currently being visited
    let mut lowest_visits = 1000000;
    let mut candidate = None;
    for sub in available_submissions {
        // Check if this judge has already visited this submission
        let already_visited = project_visit::Entity::find()
            .filter(project_visit::Column::SubmissionId.eq(sub.id))
            .filter(project_visit::Column::JudgeId.eq(ctx.user.id))
            .one(&txn)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to check visit: {}", e)))?;

        if already_visited.is_some() {
            continue;
        }

        // Check if another judge is currently visiting this submission
        let active_visits = project_visit::Entity::find()
            .filter(project_visit::Column::SubmissionId.eq(sub.id))
            .filter(project_visit::Column::IsActive.eq(true))
            .count(&txn)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to count active visits: {}", e)))?;

        if active_visits > 0 {
            continue;
        }

        let count_visits = project_visit::Entity::find()
            .filter(project_visit::Column::SubmissionId.eq(sub.id))
            .count(&txn)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to count past visits: {}", e)))?;

        // This submission is available
        if count_visits < lowest_visits {
            candidate = Some(sub);
            lowest_visits = count_visits;
        }
        if count_visits == 0 {
            break;
        }
    }

    let sub = match candidate {
        Some(s) => s,
        None => {
            txn.commit()
                .await
                .map_err(|e| ServerFnError::new(format!("Failed to commit: {}", e)))?;
            return Ok(None); // No available projects
        }
    };

    // Create the visit
    let now = chrono::Utc::now().naive_utc();
    let new_visit = project_visit::ActiveModel {
        id: NotSet,
        submission_id: Set(sub.id),
        judge_id: Set(ctx.user.id),
        hackathon_id: Set(hackathon.id),
        notes: Set(None),
        start_time: Set(now),
        completion_time: Set(None),
        is_active: Set(true),
    };

    let visit = new_visit
        .insert(&txn)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to create visit: {}", e)))?;

    txn.commit()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to commit: {}", e)))?;

    // Get team name
    let team = teams::Entity::find_by_id(sub.team_id)
        .one(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch team: {}", e)))?
        .ok_or_else(|| ServerFnError::new("Team not found"))?;

    let timeout_seconds = hackathon.judge_session_timeout_minutes as i64 * 60;

    Ok(Some(JudgeAssignment {
        visit_id: visit.id,
        submission_id: sub.id,
        team_name: team.name,
        submission_data: sub.submission_data,
        start_time: visit.start_time.to_string(),
        time_remaining_seconds: timeout_seconds,
    }))
}

/// Complete a visit with notes
#[cfg_attr(feature = "server", utoipa::path(
    post,
    path = "/api/hackathons/{slug}/judging/visits/{visit_id}/complete",
    params(
        ("slug" = String, Path, description = "Hackathon slug"),
        ("visit_id" = i32, Path, description = "Visit ID")
    ),
    request_body = CompleteVisitRequest,
    responses(
        (status = 200, description = "Visit completed successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Not your visit"),
        (status = 404, description = "Visit not found"),
        (status = 500, description = "Server error")
    ),
    tag = "judging"
))]
#[post("/api/hackathons/:slug/judging/visits/:visit_id/complete", user: SyncedUser)]
pub async fn complete_visit(
    slug: String,
    visit_id: i32,
    request: CompleteVisitRequest,
) -> Result<(), ServerFnError> {
    use crate::entities::project_visit;
    use sea_orm::{ActiveModelTrait, EntityTrait, Set};

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    let visit = project_visit::Entity::find_by_id(visit_id)
        .one(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch visit: {}", e)))?
        .ok_or_else(|| ServerFnError::new("Visit not found"))?;

    // Verify ownership
    if visit.judge_id != ctx.user.id {
        return Err(ServerFnError::new("This is not your visit"));
    }

    let mut active: project_visit::ActiveModel = visit.into();
    active.is_active = Set(false);
    active.completion_time = Set(Some(chrono::Utc::now().naive_utc()));
    active.notes = Set(request.notes);

    active
        .update(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to complete visit: {}", e)))?;

    Ok(())
}

/// Get current assignment for a judge
#[cfg_attr(feature = "server", utoipa::path(
    get,
    path = "/api/hackathons/{slug}/judging/current-assignment",
    params(
        ("slug" = String, Path, description = "Hackathon slug")
    ),
    responses(
        (status = 200, description = "Current assignment", body = Option<JudgeAssignment>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Server error")
    ),
    tag = "judging"
))]
#[get("/api/hackathons/:slug/judging/current-assignment", user: SyncedUser)]
pub async fn get_current_assignment(
    slug: String,
) -> Result<Option<JudgeAssignment>, ServerFnError> {
    use crate::entities::{project_visit, submission, teams};
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    let hackathon = ctx.hackathon()?;

    let active_visit = project_visit::Entity::find()
        .filter(project_visit::Column::JudgeId.eq(ctx.user.id))
        .filter(project_visit::Column::HackathonId.eq(hackathon.id))
        .filter(project_visit::Column::IsActive.eq(true))
        .one(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch active visit: {}", e)))?;

    match active_visit {
        Some(visit) => {
            let sub = submission::Entity::find_by_id(visit.submission_id)
                .one(&ctx.state.db)
                .await
                .map_err(|e| ServerFnError::new(format!("Failed to fetch submission: {}", e)))?
                .ok_or_else(|| ServerFnError::new("Submission not found"))?;

            let team = teams::Entity::find_by_id(sub.team_id)
                .one(&ctx.state.db)
                .await
                .map_err(|e| ServerFnError::new(format!("Failed to fetch team: {}", e)))?
                .ok_or_else(|| ServerFnError::new("Team not found"))?;

            let timeout_seconds = hackathon.judge_session_timeout_minutes as i64 * 60;
            let elapsed = chrono::Utc::now()
                .naive_utc()
                .signed_duration_since(visit.start_time)
                .num_seconds();
            let remaining = (timeout_seconds - elapsed).max(0);

            Ok(Some(JudgeAssignment {
                visit_id: visit.id,
                submission_id: sub.id,
                team_name: team.name,
                submission_data: sub.submission_data,
                start_time: visit.start_time.to_string(),
                time_remaining_seconds: remaining,
            }))
        }
        None => Ok(None),
    }
}

/// Submit a pairwise comparison
#[cfg_attr(feature = "server", utoipa::path(
    post,
    path = "/api/hackathons/{slug}/judging/compare",
    params(
        ("slug" = String, Path, description = "Hackathon slug")
    ),
    request_body = PairwiseComparisonRequest,
    responses(
        (status = 200, description = "Comparison submitted successfully"),
        (status = 400, description = "Invalid comparison"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Server error")
    ),
    tag = "judging"
))]
#[post("/api/hackathons/:slug/judging/compare", user: SyncedUser)]
pub async fn submit_comparison(
    slug: String,
    request: PairwiseComparisonRequest,
) -> Result<(), ServerFnError> {
    use crate::entities::pairwise_comparison;
    use sea_orm::{ActiveModelTrait, ActiveValue::NotSet, Set};

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    let hackathon = ctx.hackathon()?;

    // Check if judging is active
    if !hackathon.judging_started {
        return Err(ServerFnError::new("Judging has not started yet"));
    }

    let new_comparison = pairwise_comparison::ActiveModel {
        id: NotSet,
        feature_id: Set(request.feature_id),
        judge_id: Set(ctx.user.id),
        submission_a_id: Set(request.submission_a_id),
        submission_b_id: Set(request.submission_b_id),
        winner_id: Set(request.winner_id),
        created_at: Set(chrono::Utc::now().naive_utc()),
    };

    new_comparison
        .insert(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to submit comparison: {}", e)))?;

    Ok(())
}

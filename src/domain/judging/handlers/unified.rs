//! Unified judging mode: two-phase project selection and batch comparison submission.

use crate::domain::judging::types::*;
use dioxus::prelude::*;

#[cfg(feature = "server")]
use crate::core::auth::{context::RequestContext, middleware::SyncedUser};

/// Get the full unified judging state for a judge
#[cfg_attr(feature = "server", utoipa::path(
    get,
    path = "/api/hackathons/{slug}/judging/state",
    params(
        ("slug" = String, Path, description = "Hackathon slug")
    ),
    responses(
        (status = 200, description = "Unified judging state", body = UnifiedJudgingState),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Server error")
    ),
    tag = "judging"
))]
#[get("/api/hackathons/:slug/judging/state", user: SyncedUser)]
pub async fn get_unified_state(slug: String) -> Result<UnifiedJudgingState, ServerFnError> {
    use crate::entities::{
        feature, judge_feature_assignment, judge_prize_track, judge_walk_type, prize,
        prize_feature_weight, project_visit, submission, teams,
    };
    use sea_orm::{
        ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, EntityTrait, QueryFilter, QuerySelect,
        Set,
    };
    use std::collections::{HashMap, HashSet};

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    let hackathon = ctx.hackathon()?;

    // Step 1: Resolve relevant prizes (Assignments + Default Tracks)
    // A judge is responsible for:
    // 1. Prizes they are explicitly assigned to (Restricted tracks)
    // 2. Prizes that have NO assignments (Default tracks)

    // Fetch all prizes for this hackathon
    let all_prizes = prize::Entity::find()
        .filter(prize::Column::HackathonId.eq(hackathon.id))
        .all(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch hackathon prizes: {}", e)))?;

    let all_prize_ids: Vec<i32> = all_prizes.iter().map(|p| p.id).collect();

    // Find assignments for THIS judge
    let my_assignments: HashSet<i32> = judge_prize_track::Entity::find()
        .filter(judge_prize_track::Column::JudgeId.eq(ctx.user.id))
        .filter(judge_prize_track::Column::PrizeId.is_in(all_prize_ids.clone()))
        .select_only()
        .column(judge_prize_track::Column::PrizeId)
        .into_tuple::<i32>()
        .all(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch my assignments: {}", e)))?
        .into_iter()
        .collect();

    // Determine effective prizes: Only My Assignments (Explicit)
    let effective_prize_ids = my_assignments;

    // Step 2: Get feature IDs linked to these prizes
    let target_feature_ids: HashSet<i32> = if effective_prize_ids.is_empty() {
        // I'm assigned to no prizes
        HashSet::new()
    } else {
        // Get features linked to effective prizes
        prize_feature_weight::Entity::find()
            .filter(
                prize_feature_weight::Column::PrizeId.is_in(effective_prize_ids.iter().copied()),
            )
            .filter(prize_feature_weight::Column::Weight.ne(0.0))
            .all(&ctx.state.db)
            .await
            .map_err(|e| {
                ServerFnError::new(format!("Failed to fetch prize feature weights: {}", e))
            })?
            .into_iter()
            .map(|pfw| pfw.feature_id)
            .collect()
    };

    // Step 3: Get or create judge_feature_assignment entries for these features
    let existing_assignments = judge_feature_assignment::Entity::find()
        .filter(judge_feature_assignment::Column::JudgeId.eq(ctx.user.id))
        .all(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch assignments: {}", e)))?;

    let existing_feature_ids: HashSet<i32> =
        existing_assignments.iter().map(|a| a.feature_id).collect();

    // Create missing assignments
    let now = chrono::Utc::now().naive_utc();
    for &feature_id in &target_feature_ids {
        if !existing_feature_ids.contains(&feature_id) {
            let new_assignment = judge_feature_assignment::ActiveModel {
                id: NotSet,
                judge_id: Set(ctx.user.id),
                feature_id: Set(feature_id),
                current_best_submission_id: Set(None),
                notes: Set(None),
                created_at: Set(now),
            };
            let _ = new_assignment.insert(&ctx.state.db).await;
        }
    }

    // Step 4: Re-fetch all assignments and build the features list
    let assignments = judge_feature_assignment::Entity::find()
        .filter(judge_feature_assignment::Column::JudgeId.eq(ctx.user.id))
        .all(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch assignments: {}", e)))?;

    // Pre-fetch all features for efficiency
    let all_features_map: HashMap<i32, feature::Model> = feature::Entity::find()
        .filter(feature::Column::HackathonId.eq(hackathon.id))
        .all(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch all features: {}", e)))?
        .into_iter()
        .map(|f| (f.id, f))
        .collect();

    let mut features = Vec::new();
    for assignment in assignments {
        // Only include features that are in our target set
        if !target_feature_ids.contains(&assignment.feature_id) {
            continue;
        }

        if let Some(feat) = all_features_map.get(&assignment.feature_id) {
            let mut current_best_team_name = None;
            let mut current_best_description = None;
            let mut current_best_table_number = None;

            if let Some(best_sub_id) = assignment.current_best_submission_id {
                if let Ok(Some(sub)) = submission::Entity::find_by_id(best_sub_id)
                    .one(&ctx.state.db)
                    .await
                {
                    if let Ok(Some(team)) = teams::Entity::find_by_id(sub.team_id)
                        .one(&ctx.state.db)
                        .await
                    {
                        current_best_team_name = Some(team.name);
                    }
                    current_best_description = sub
                        .submission_data
                        .get("description")
                        .and_then(|d| d.as_str())
                        .map(|s| s.to_string());
                    current_best_table_number = sub.table_number;
                }
            }

            features.push(JudgeFeatureState {
                feature_id: feat.id,
                feature_name: feat.name.clone(),
                feature_description: feat.description.clone(),
                current_best_submission_id: assignment.current_best_submission_id,
                current_best_team_name,
                current_best_description,
                current_best_table_number,
                notes: assignment.notes,
            });
        }
    }

    // Get current active visit
    let active_visit = project_visit::Entity::find()
        .filter(project_visit::Column::JudgeId.eq(ctx.user.id))
        .filter(project_visit::Column::HackathonId.eq(hackathon.id))
        .filter(project_visit::Column::IsActive.eq(true))
        .one(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch active visit: {}", e)))?;

    let current_project = if let Some(visit) = active_visit {
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

        Some(CurrentProject {
            visit_id: visit.id,
            submission_id: sub.id,
            team_name: team.name.clone(),
            project_name: sub
                .submission_data
                .get("projectName")
                .and_then(|n| n.as_str())
                .map(|s| s.to_string()),
            location: None, // Deprecated in favor of table_number
            table_number: sub.table_number.clone(),
            description: sub
                .submission_data
                .get("description")
                .and_then(|d| d.as_str())
                .map(|s| s.to_string()),
            submission_data: sub.submission_data,
        })
    } else {
        None
    };

    // Build assigned prizes list
    let assigned_prizes: Vec<PrizeInfo> = all_prizes
        .iter()
        .filter(|p| effective_prize_ids.contains(&p.id))
        .map(|p| PrizeInfo {
            id: p.id,
            name: p.name.clone(),
            description: p.description.clone(),
        })
        .collect();

    // Fetch judge's walk type preference
    let walk_type_record = judge_walk_type::Entity::find()
        .filter(judge_walk_type::Column::JudgeId.eq(ctx.user.id))
        .filter(judge_walk_type::Column::HackathonId.eq(hackathon.id))
        .one(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch walk type: {}", e)))?;

    let walk_type = walk_type_record
        .map(|r| WalkType::from_str(&r.walk_type))
        .unwrap_or_default();

    Ok(UnifiedJudgingState {
        current_project,
        features,
        assigned_prizes,
        all_prizes: all_prizes
            .into_iter()
            .map(|p| PrizeInfo {
                id: p.id,
                name: p.name,
                description: p.description,
            })
            .collect(),
        judging_started: hackathon.judging_started,
        walk_type,
    })
}

/// Toggle assignment for a prize track
#[cfg_attr(feature = "server", utoipa::path(
    post,
    path = "/api/hackathons/{slug}/judging/toggle-prize/{prize_id}",
    params(
        ("slug" = String, Path, description = "Hackathon slug"),
        ("prize_id" = i32, Path, description = "Prize ID")
    ),
    responses(
        (status = 200, description = "Assignment toggled"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Server error")
    ),
    tag = "judging"
))]
#[post("/api/hackathons/:slug/judging/toggle-prize/:prize_id", user: SyncedUser)]
pub async fn toggle_prize_assignment(slug: String, prize_id: i32) -> Result<(), ServerFnError> {
    use crate::entities::judge_prize_track;
    use sea_orm::{
        ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, EntityTrait, ModelTrait, QueryFilter,
        Set,
    };

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    // Check if already assigned
    let assignment = judge_prize_track::Entity::find()
        .filter(judge_prize_track::Column::JudgeId.eq(ctx.user.id))
        .filter(judge_prize_track::Column::PrizeId.eq(prize_id))
        .one(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to check assignment: {}", e)))?;

    if let Some(a) = assignment {
        // Unassign
        a.delete(&ctx.state.db)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to delete assignment: {}", e)))?;
    } else {
        // Assign
        let new_assignment = judge_prize_track::ActiveModel {
            id: NotSet,
            judge_id: Set(ctx.user.id),
            prize_id: Set(prize_id),
            created_at: Set(chrono::Utc::now().naive_utc()),
        };
        new_assignment
            .insert(&ctx.state.db)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to create assignment: {}", e)))?;
    }

    Ok(())
}

/// Set the walk type preference for a judge
#[cfg_attr(feature = "server", utoipa::path(
    post,
    path = "/api/hackathons/{slug}/judging/walk-type",
    params(
        ("slug" = String, Path, description = "Hackathon slug")
    ),
    request_body = WalkType,
    responses(
        (status = 200, description = "Walk type updated"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Server error")
    ),
    tag = "judging"
))]
#[post("/api/hackathons/:slug/judging/walk-type", user: SyncedUser)]
pub async fn set_walk_type(slug: String, walk_type: WalkType) -> Result<(), ServerFnError> {
    use crate::entities::judge_walk_type;
    use sea_orm::{
        ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, EntityTrait, QueryFilter, Set,
    };

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    let hackathon = ctx.hackathon()?;

    // Check if a record already exists
    let existing = judge_walk_type::Entity::find()
        .filter(judge_walk_type::Column::JudgeId.eq(ctx.user.id))
        .filter(judge_walk_type::Column::HackathonId.eq(hackathon.id))
        .one(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to check walk type: {}", e)))?;

    if let Some(record) = existing {
        // Update existing record
        let mut active: judge_walk_type::ActiveModel = record.into();
        active.walk_type = Set(walk_type.to_str().to_string());
        active
            .update(&ctx.state.db)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to update walk type: {}", e)))?;
    } else {
        // Create new record
        let new_record = judge_walk_type::ActiveModel {
            id: NotSet,
            judge_id: Set(ctx.user.id),
            hackathon_id: Set(hackathon.id),
            walk_type: Set(walk_type.to_str().to_string()),
            created_at: Set(chrono::Utc::now().naive_utc()),
        };
        new_record
            .insert(&ctx.state.db)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to create walk type: {}", e)))?;
    }

    Ok(())
}
#[cfg_attr(feature = "server", utoipa::path(
    post,
    path = "/api/hackathons/{slug}/judging/next-project",
    params(
        ("slug" = String, Path, description = "Hackathon slug")
    ),
    responses(
        (status = 200, description = "Next project assigned", body = Option<CurrentProject>),
        (status = 400, description = "Judging not active or already has active assignment"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Server error")
    ),
    tag = "judging"
))]
#[post("/api/hackathons/:slug/judging/next-project", user: SyncedUser)]
pub async fn request_next_project(slug: String) -> Result<Option<CurrentProject>, ServerFnError> {
    use crate::entities::{
        judge_walk_type, project_feature_score, project_visit, submission, teams,
    };
    use rand::prelude::*;
    use sea_orm::{
        ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, EntityTrait, PaginatorTrait,
        QueryFilter, QueryOrder, QuerySelect, Set, TransactionTrait,
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
    let existing_active = project_visit::Entity::find()
        .filter(project_visit::Column::JudgeId.eq(ctx.user.id))
        .filter(project_visit::Column::HackathonId.eq(hackathon.id))
        .filter(project_visit::Column::IsActive.eq(true))
        .one(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to check active visits: {}", e)))?;

    if existing_active.is_some() {
        return Err(ServerFnError::new(
            "You already have an active project. Complete it first.",
        ));
    }

    // Start transaction for atomic assignment
    let txn = ctx
        .state
        .db
        .begin()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to start transaction: {}", e)))?;

    // Get all team IDs for this hackathon
    let team_ids: Vec<i32> = teams::Entity::find()
        .filter(teams::Column::HackathonId.eq(hackathon.id))
        .select_only()
        .column(teams::Column::Id)
        .into_tuple::<i32>()
        .all(&txn)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch teams: {}", e)))?;

    if team_ids.is_empty() {
        txn.commit().await.ok();
        return Ok(None);
    }

    // Get all submissions with a table number assigned
    let all_submissions = submission::Entity::find()
        .filter(submission::Column::TeamId.is_in(team_ids.clone()))
        .filter(submission::Column::TableNumber.is_not_null())
        .all(&txn)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch submissions: {}", e)))?;

    if all_submissions.is_empty() {
        txn.commit().await.ok();
        return Ok(None);
    }

    // Get IDs of submissions this judge has already visited
    let visited_ids: Vec<i32> = project_visit::Entity::find()
        .filter(project_visit::Column::JudgeId.eq(ctx.user.id))
        .filter(project_visit::Column::HackathonId.eq(hackathon.id))
        .select_only()
        .column(project_visit::Column::SubmissionId)
        .into_tuple::<i32>()
        .all(&txn)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch visited: {}", e)))?;

    // Get IDs of submissions currently being visited by any judge
    let locked_ids: Vec<i32> = project_visit::Entity::find()
        .filter(project_visit::Column::HackathonId.eq(hackathon.id))
        .filter(project_visit::Column::IsActive.eq(true))
        .select_only()
        .column(project_visit::Column::SubmissionId)
        .into_tuple::<i32>()
        .all(&txn)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch locked: {}", e)))?;

    // Filter to available submissions
    let mut available_submissions: Vec<_> = all_submissions
        .iter()
        .filter(|s| !visited_ids.contains(&s.id) && !locked_ids.contains(&s.id))
        .collect();

    // Prize track filtering:
    // A judge can only judge a submission if:
    //   1. There are no restricted prize tracks (all tracks are default), OR
    //   2. The submission has no prize track entries, OR
    //   3. ANY of its prize tracks are "Default" (no judge assignments), OR
    //   4. ANY of its prize tracks have this judge assigned
    let (valid_submission_ids, submissions_with_entries) =
        get_valid_submissions_for_judge(&txn, ctx.user.id).await?;

    // Only apply filtering if we have explicit valid IDs
    // Empty valid_ids means either: no restrictions exist, OR no submissions are valid for this judge
    // We differentiate by checking if submissions_with_entries is also empty
    let should_filter = !submissions_with_entries.is_empty();
    if should_filter {
        // A submission is valid if:
        // - It's explicitly in valid_submission_ids (it's in a default or assigned track), OR
        // - It has no prize track entries (not submitted to any track)
        available_submissions.retain(|s| {
            valid_submission_ids.contains(&s.id) || !submissions_with_entries.contains(&s.id)
        });
    }

    if available_submissions.is_empty() {
        txn.commit().await.ok();
        return Ok(None);
    }

    // Get visit counts for each submission
    let mut submission_visit_counts: std::collections::HashMap<i32, u64> =
        std::collections::HashMap::new();
    for sub in &available_submissions {
        let count = project_visit::Entity::find()
            .filter(project_visit::Column::SubmissionId.eq(sub.id))
            .filter(project_visit::Column::HackathonId.eq(hackathon.id))
            .count(&txn)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to count visits: {}", e)))?;
        submission_visit_counts.insert(sub.id, count);
    }

    // Fetch judge's walk type preference
    let walk_type_record = judge_walk_type::Entity::find()
        .filter(judge_walk_type::Column::JudgeId.eq(ctx.user.id))
        .filter(judge_walk_type::Column::HackathonId.eq(hackathon.id))
        .one(&txn)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch walk type: {}", e)))?;

    let walk_type = walk_type_record
        .map(|r| WalkType::from_str(&r.walk_type))
        .unwrap_or_default();

    // Helper function to parse table coordinates (L1 distance calculation)
    fn parse_table_coords(table_number: &str) -> Option<(i32, i32)> {
        let num: i32 = table_number.parse().ok()?;
        if num < 10 {
            Some((num, 0)) // X = full number, Y = 0
        } else {
            Some((num % 10, num / 10)) // X = last digit, Y = rest
        }
    }

    fn l1_distance(a: (i32, i32), b: (i32, i32)) -> i32 {
        (a.0 - b.0).abs() + (a.1 - b.1).abs()
    }

    // Phase 1: Prioritize submissions with < 2 visits
    let under_visited: Vec<&submission::Model> = available_submissions
        .iter()
        .filter(|s| submission_visit_counts.get(&s.id).copied().unwrap_or(0) < 2)
        .copied()
        .collect();

    // Track if we have under-visited projects before moving the vec
    let has_under_visited = !under_visited.is_empty();

    // Determine pool to select from: prioritize under-visited
    let selection_pool: Vec<&submission::Model> = if has_under_visited {
        under_visited
    } else {
        available_submissions.clone()
    };

    let mut rng = rand::rng();
    let selected_sub;

    match walk_type {
        WalkType::Proximity => {
            // Get the judge's last completed visit to find previous table
            let last_visit = project_visit::Entity::find()
                .filter(project_visit::Column::JudgeId.eq(ctx.user.id))
                .filter(project_visit::Column::HackathonId.eq(hackathon.id))
                .filter(project_visit::Column::IsActive.eq(false))
                .order_by_desc(project_visit::Column::CompletionTime)
                .one(&txn)
                .await
                .map_err(|e| ServerFnError::new(format!("Failed to fetch last visit: {}", e)))?;

            // Get the previous table number
            let previous_coords: Option<(i32, i32)> = if let Some(visit) = last_visit {
                // Look up the submission to get its table number
                if let Some(prev_sub) = all_submissions.iter().find(|s| s.id == visit.submission_id)
                {
                    prev_sub
                        .table_number
                        .as_ref()
                        .and_then(|t| parse_table_coords(t))
                } else {
                    None
                }
            } else {
                None
            };

            if let Some(prev_coords) = previous_coords {
                // Find nearest submission by L1 distance
                let mut nearest_sub = None;
                let mut min_distance = i32::MAX;

                for sub in &selection_pool {
                    if let Some(table) = &sub.table_number {
                        if let Some(coords) = parse_table_coords(table) {
                            let dist = l1_distance(prev_coords, coords);
                            if dist < min_distance {
                                min_distance = dist;
                                nearest_sub = Some(sub);
                            }
                        }
                    }
                }

                selected_sub = nearest_sub.unwrap_or_else(|| {
                    // Fall back to random if no valid table numbers
                    selection_pool.choose(&mut rng).unwrap()
                });
            } else {
                // No previous visit, fall back to random selection
                selected_sub = selection_pool.choose(&mut rng).unwrap();
            }
        }
        WalkType::Default => {
            // Default algorithm: random for under-visited, softmax for the rest
            if has_under_visited {
                selected_sub = selection_pool.choose(&mut rng).unwrap();
            } else {
                // Phase 2: Softmax-weighted selection based on average feature scores
                let mut weights: Vec<f64> = Vec::new();

                for sub in &available_submissions {
                    // Get average score across all features
                    let score_records = project_feature_score::Entity::find()
                        .filter(project_feature_score::Column::SubmissionId.eq(sub.id))
                        .all(&txn)
                        .await
                        .unwrap_or_default();

                    let scores: Vec<f32> = score_records.iter().filter_map(|r| r.score).collect();

                    let avg_score = if scores.is_empty() {
                        0.5 // Default score for unscored projects
                    } else {
                        scores.iter().sum::<f32>() / scores.len() as f32
                    };

                    // Softmax weight (higher scores get higher probability)
                    weights.push((avg_score as f64).exp());
                }

                // Normalize weights
                let total: f64 = weights.iter().sum();
                if total > 0.0 {
                    for w in &mut weights {
                        *w /= total;
                    }
                } else {
                    // Equal weights if all zero
                    let equal = 1.0 / weights.len() as f64;
                    for w in &mut weights {
                        *w = equal;
                    }
                }

                // Sample using cumulative distribution
                let mut cumsum = 0.0;
                let sample: f64 = rng.random();
                let mut selected_idx = 0;
                for (i, &w) in weights.iter().enumerate() {
                    cumsum += w;
                    if sample <= cumsum {
                        selected_idx = i;
                        break;
                    }
                }
                selected_sub = &available_submissions[selected_idx];
            }
        }
    }

    // Create the visit
    let now = chrono::Utc::now().naive_utc();
    let new_visit = project_visit::ActiveModel {
        id: NotSet,
        submission_id: Set(selected_sub.id),
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

    // Get team details
    let team = teams::Entity::find_by_id(selected_sub.team_id)
        .one(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch team: {}", e)))?
        .ok_or_else(|| ServerFnError::new("Team not found"))?;

    Ok(Some(CurrentProject {
        visit_id: visit.id,
        submission_id: selected_sub.id,
        team_name: team.name.clone(),
        project_name: selected_sub
            .submission_data
            .get("projectName")
            .and_then(|n| n.as_str())
            .map(|s| s.to_string()),
        location: None,
        table_number: selected_sub.table_number.clone(),
        description: selected_sub
            .submission_data
            .get("description")
            .and_then(|d| d.as_str())
            .map(|s| s.to_string()),
        submission_data: selected_sub.submission_data.clone(),
    }))
}

/// Submit comparisons for all features at once
#[cfg_attr(feature = "server", utoipa::path(
    post,
    path = "/api/hackathons/{slug}/judging/submit",
    params(
        ("slug" = String, Path, description = "Hackathon slug")
    ),
    request_body = SubmitComparisonsRequest,
    responses(
        (status = 200, description = "Comparisons submitted successfully"),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Server error")
    ),
    tag = "judging"
))]
#[post("/api/hackathons/:slug/judging/submit", user: SyncedUser)]
pub async fn submit_comparisons(
    slug: String,
    request: SubmitComparisonsRequest,
) -> Result<(), ServerFnError> {
    use crate::entities::{judge_feature_assignment, pairwise_comparison, project_visit};
    use sea_orm::{
        ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, EntityTrait, QueryFilter, Set,
        TransactionTrait,
    };

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    let hackathon = ctx.hackathon()?;

    if !hackathon.judging_started {
        return Err(ServerFnError::new("Judging has not started yet"));
    }

    // Verify the visit belongs to this judge
    let visit = project_visit::Entity::find_by_id(request.visit_id)
        .one(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch visit: {}", e)))?
        .ok_or_else(|| ServerFnError::new("Visit not found"))?;

    if visit.judge_id != ctx.user.id {
        return Err(ServerFnError::new("This is not your visit"));
    }

    if !visit.is_active {
        return Err(ServerFnError::new("This visit is already completed"));
    }

    let txn = ctx
        .state
        .db
        .begin()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to start transaction: {}", e)))?;

    let current_submission_id = visit.submission_id;
    let now = chrono::Utc::now().naive_utc();

    for comparison in request.comparisons {
        // Get the judge's assignment for this feature
        let assignment = judge_feature_assignment::Entity::find()
            .filter(judge_feature_assignment::Column::JudgeId.eq(ctx.user.id))
            .filter(judge_feature_assignment::Column::FeatureId.eq(comparison.feature_id))
            .one(&txn)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to fetch assignment: {}", e)))?;

        let assignment = match assignment {
            Some(a) => a,
            None => continue, // Skip if not assigned to this feature
        };

        let old_best_id = assignment.current_best_submission_id;

        // Record pairwise comparison if there was a previous best
        if let Some(prev_best_id) = old_best_id {
            let new_comparison = pairwise_comparison::ActiveModel {
                id: NotSet,
                feature_id: Set(comparison.feature_id),
                judge_id: Set(ctx.user.id),
                submission_a_id: Set(current_submission_id),
                submission_b_id: Set(prev_best_id),
                winner_id: Set(Some(comparison.winner_submission_id)),
                created_at: Set(now),
            };

            new_comparison
                .insert(&txn)
                .await
                .map_err(|e| ServerFnError::new(format!("Failed to save comparison: {}", e)))?;

            // Add transitive comparisons: if the winner beat the loser, the winner
            // also beats all projects that the loser previously beat.
            // This helps projects seen later in the walk accumulate more comparison data.
            let loser_id = if comparison.winner_submission_id == current_submission_id {
                prev_best_id
            } else {
                current_submission_id
            };
            let winner_id = comparison.winner_submission_id;

            // Find all comparisons where the loser won (same feature, same judge)
            let loser_wins = pairwise_comparison::Entity::find()
                .filter(pairwise_comparison::Column::FeatureId.eq(comparison.feature_id))
                .filter(pairwise_comparison::Column::JudgeId.eq(ctx.user.id))
                .filter(pairwise_comparison::Column::WinnerId.eq(Some(loser_id)))
                .all(&txn)
                .await
                .map_err(|e| ServerFnError::new(format!("Failed to fetch loser's wins: {}", e)))?;

            // For each project the loser beat, record that the winner also beats it
            for prev_comp in loser_wins {
                // Determine which submission the loser beat
                let beaten_id = if prev_comp.submission_a_id == loser_id {
                    prev_comp.submission_b_id
                } else {
                    prev_comp.submission_a_id
                };

                // Don't add duplicate or self-comparisons
                if beaten_id == winner_id {
                    continue;
                }

                let transitive_comparison = pairwise_comparison::ActiveModel {
                    id: NotSet,
                    feature_id: Set(comparison.feature_id),
                    judge_id: Set(ctx.user.id),
                    submission_a_id: Set(winner_id),
                    submission_b_id: Set(beaten_id),
                    winner_id: Set(Some(winner_id)),
                    created_at: Set(now),
                };

                transitive_comparison.insert(&txn).await.map_err(|e| {
                    ServerFnError::new(format!("Failed to save transitive comparison: {}", e))
                })?;
            }
        }

        // Update the assignment with the new best and notes
        let mut active: judge_feature_assignment::ActiveModel = assignment.into();
        active.current_best_submission_id = Set(Some(comparison.winner_submission_id));
        if let Some(notes) = comparison.notes {
            active.notes = Set(Some(notes));
        }

        active
            .update(&txn)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to update assignment: {}", e)))?;
    }

    // Mark visit as complete
    let mut visit_active: project_visit::ActiveModel = visit.into();
    visit_active.is_active = Set(false);
    visit_active.completion_time = Set(Some(now));
    if let Some(notes) = request.notes {
        visit_active.notes = Set(Some(notes));
    }

    visit_active
        .update(&txn)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to complete visit: {}", e)))?;

    txn.commit()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to commit: {}", e)))?;

    Ok(())
}

/// Helper: Get submission IDs that a judge can judge based on prize track assignments.
/// A judge can judge a submission if:
///   1. The submission has no prize track entries (not submitted to any track), OR
///   2. ANY of its prize tracks are "Default" (no judge assignments), OR
///   3. ANY of its prize tracks have this judge assigned
/// Returns (valid_ids, submissions_with_entries):
///   - valid_ids: submission IDs explicitly valid due to prize track rules
///   - submissions_with_entries: all submission IDs that have at least one prize track entry
/// If submissions_with_entries is empty, no filtering should be applied.
/// If valid_ids equals submissions_with_entries, all submissions with entries are valid.
#[cfg(feature = "server")]
async fn get_valid_submissions_for_judge<C: sea_orm::ConnectionTrait>(
    db: &C,
    judge_id: i32,
) -> Result<(Vec<i32>, Vec<i32>), ServerFnError> {
    use crate::entities::{judge_prize_track, prize_track_entry};
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect};
    use std::collections::HashSet;

    // Get all prize track entries (submission_id -> prize_id mappings)
    let entries = prize_track_entry::Entity::find()
        .all(db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch entries: {}", e)))?;

    if entries.is_empty() {
        // No prize track entries exist, all submissions are valid (no filtering needed)
        return Ok((Vec::new(), Vec::new()));
    }

    // Track which submissions have prize track entries
    let mut submissions_with_entries: HashSet<i32> = HashSet::new();
    for entry in &entries {
        submissions_with_entries.insert(entry.submission_id);
    }

    // Get prize IDs that this judge is assigned to
    let judge_prize_ids: HashSet<i32> = judge_prize_track::Entity::find()
        .filter(judge_prize_track::Column::JudgeId.eq(judge_id))
        .select_only()
        .column(judge_prize_track::Column::PrizeId)
        .into_tuple::<i32>()
        .all(db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch judge prizes: {}", e)))?
        .into_iter()
        .collect();

    // Build a set of valid submission IDs
    let mut valid_submission_ids: HashSet<i32> = HashSet::new();

    for entry in entries {
        let prize_id = entry.prize_id;
        let submission_id = entry.submission_id;

        // Check if the judge is assigned to this prize track
        if judge_prize_ids.contains(&prize_id) {
            valid_submission_ids.insert(submission_id);
        }
    }

    Ok((
        valid_submission_ids.into_iter().collect(),
        submissions_with_entries.into_iter().collect(),
    ))
}

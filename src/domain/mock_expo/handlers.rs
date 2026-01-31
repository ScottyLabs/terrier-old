use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

/// Response for data generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationResult {
    pub message: String,
    pub count: usize,
}

/// Generate fake teams and submissions
#[server]
pub async fn generate_fake_data(
    slug: String,
    count: usize,
) -> Result<GenerationResult, ServerFnError> {
    use crate::entities::{hackathons, submission, teams, users};
    use rand::Rng;
    use sea_orm::{
        ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, EntityTrait, QueryFilter, Set,
        TransactionTrait,
    }; // For random generation

    // Extract user from request context (requires dioxus-fullstack)
    let synced_user = dioxus::fullstack::FullstackContext::extract::<
        crate::core::auth::middleware::SyncedUser,
        _,
    >()
    .await
    .map_err(|e| ServerFnError::new(format!("Failed to extract user: {}", e)))?;

    let ctx = crate::core::auth::context::RequestContext::extract(&synced_user)
        .await?
        .with_hackathon(&slug)
        .await?;

    let hackathon = ctx.hackathon()?;
    let db = &ctx.state.db;

    // Transaction for atomic generation
    let txn = db
        .begin()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let mut created_count = 0;

    // Rand 0.9 usage
    for i in 0..count {
        // 1. Create a fake user
        let random_id: String = rand::rng()
            .sample_iter(&rand::distr::Alphanumeric)
            .take(8)
            .map(char::from)
            .collect();

        let fake_email = format!("fake_{}_{}@example.com", random_id, i);
        let fake_user_sub = format!("fake_sub_{}_{}", random_id, i);

        let user_model = users::ActiveModel {
            id: NotSet,
            oidc_sub: Set(fake_user_sub),
            email: Set(fake_email),
            name: Set(Some(format!("Fake User {}", i))),
            oidc_issuer: Set("mock_expo".to_string()),
            created_at: Set(chrono::Utc::now().naive_utc()),
            updated_at: Set(chrono::Utc::now().naive_utc()),
            ..Default::default()
        };

        let user = user_model
            .insert(&txn)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        // 2. Create a team
        let team_name = format!("__fake__ {} Team {}", i, random_id);

        let team_model = teams::ActiveModel {
            id: NotSet,
            hackathon_id: Set(hackathon.id),
            name: Set(team_name.clone()),
            owner_id: Set(user.id),
            created_at: Set(chrono::Utc::now().naive_utc()),
            updated_at: Set(chrono::Utc::now().naive_utc()),
            description: Set(Some("Generative AI fake project description.".to_string())),
        };

        let team = team_model
            .insert(&txn)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        // 3. Create a submission
        let submission_data = serde_json::json!({
            "title": format!("Fake Project {}", i),
            "projectName": format!("Fake Project {}", i),
            "description": "This is a fake project generated for the mock expo.",
            "technologies": ["Rust", "Dioxus", "SeaORM"],
            "url": "https://example.com"
        });

        let submission_model = submission::ActiveModel {
            id: NotSet,
            team_id: Set(team.id),
            submission_data: Set(submission_data),
            submitted_at: Set(chrono::Utc::now().naive_utc()),
            table_number: Set(None),
        };

        submission_model
            .insert(&txn)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        created_count += 1;
    }

    txn.commit()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(GenerationResult {
        message: format!("Successfully generated {} projects", created_count),
        count: created_count,
    })
}

/// Clear all fake data (teams with __fake__ in name)
#[server]
pub async fn clear_fake_data(slug: String) -> Result<GenerationResult, ServerFnError> {
    use crate::entities::{teams, users};
    use sea_orm::{ColumnTrait, EntityTrait, ModelTrait, QueryFilter};

    let synced_user = dioxus::fullstack::FullstackContext::extract::<
        crate::core::auth::middleware::SyncedUser,
        _,
    >()
    .await
    .map_err(|e| ServerFnError::new(format!("Failed to extract user: {}", e)))?;

    let ctx = crate::core::auth::context::RequestContext::extract(&synced_user)
        .await?
        .with_hackathon(&slug)
        .await?;

    let hackathon = ctx.hackathon()?;

    // Find all fake teams in this hackathon
    let fake_teams = teams::Entity::find()
        .filter(teams::Column::HackathonId.eq(hackathon.id))
        .filter(teams::Column::Name.contains("__fake__"))
        .all(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let count = fake_teams.len();

    // Iterate and delete
    for team in fake_teams {
        // Check owner
        let owner = users::Entity::find_by_id(team.owner_id)
            .one(&ctx.state.db)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        // Delete team
        let _ = teams::Entity::delete_by_id(team.id)
            .exec(&ctx.state.db)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        // If owner is a mock user, delete them too
        if let Some(u) = owner {
            if u.oidc_issuer == "mock_expo" {
                let _ = users::Entity::delete_by_id(u.id)
                    .exec(&ctx.state.db)
                    .await
                    .map_err(|e| ServerFnError::new(e.to_string()));
            }
        }
    }

    Ok(GenerationResult {
        message: format!("Cleared {} fake teams", count),
        count,
    })
}

/// Assign table numbers to all submissions without one (or all?)
#[server]
pub async fn assign_tables(slug: String) -> Result<GenerationResult, ServerFnError> {
    use crate::entities::{submission, teams};
    use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

    let synced_user = dioxus::fullstack::FullstackContext::extract::<
        crate::core::auth::middleware::SyncedUser,
        _,
    >()
    .await
    .map_err(|e| ServerFnError::new(format!("Failed to extract user: {}", e)))?;

    let ctx = crate::core::auth::context::RequestContext::extract(&synced_user)
        .await?
        .with_hackathon(&slug)
        .await?;

    let hackathon = ctx.hackathon()?;

    let submissions: Vec<(submission::Model, Option<teams::Model>)> = submission::Entity::find()
        .find_also_related(teams::Entity)
        .filter(teams::Column::HackathonId.eq(hackathon.id))
        .all(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let mut updated_count = 0;

    for (i, (sub, _team)) in submissions.into_iter().enumerate() {
        let mut active: submission::ActiveModel = sub.into();
        active.table_number = Set(Some(format!("{}", i + 1)));

        active
            .update(&ctx.state.db)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;
        updated_count += 1;
    }

    Ok(GenerationResult {
        message: format!("Assigned tables to {} submissions", updated_count),
        count: updated_count,
    })
}

/// Assign random prizes to submissions
#[server]
pub async fn assign_prizes_randomly(slug: String) -> Result<GenerationResult, ServerFnError> {
    use crate::entities::{prize, prize_track_entry, submission, teams};
    use rand::Rng;
    use rand::prelude::IndexedRandom; // For choose_multiple
    use sea_orm::{
        ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, EntityTrait, QueryFilter, Set,
    }; // For gen_range

    let synced_user = dioxus::fullstack::FullstackContext::extract::<
        crate::core::auth::middleware::SyncedUser,
        _,
    >()
    .await
    .map_err(|e| ServerFnError::new(format!("Failed to extract user: {}", e)))?;

    let ctx = crate::core::auth::context::RequestContext::extract(&synced_user)
        .await?
        .with_hackathon(&slug)
        .await?;

    let hackathon = ctx.hackathon()?;

    // 1. Get all prizes
    let prizes = prize::Entity::find()
        .filter(prize::Column::HackathonId.eq(hackathon.id))
        .all(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if prizes.is_empty() {
        return Ok(GenerationResult {
            message: "No prizes found to assign".to_string(),
            count: 0,
        });
    }

    // 2. Get all submissions
    let submissions: Vec<(submission::Model, Option<teams::Model>)> = submission::Entity::find()
        .find_also_related(teams::Entity)
        .filter(teams::Column::HackathonId.eq(hackathon.id))
        .all(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let mut assignments_count = 0;
    let mut rng = rand::rng();

    for (sub, _team) in submissions {
        // Clear existing
        prize_track_entry::Entity::delete_many()
            .filter(prize_track_entry::Column::SubmissionId.eq(sub.id))
            .exec(&ctx.state.db)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        // Pick 1-3 random prizes
        let max_prizes = 3.min(prizes.len());
        if max_prizes > 0 {
            let num_prizes = rng.gen_range(1..=max_prizes);
            let selected_prizes = prizes.choose_multiple(&mut rng, num_prizes);

            for prize in selected_prizes {
                let entry = prize_track_entry::ActiveModel {
                    id: NotSet,
                    submission_id: Set(sub.id),
                    prize_id: Set(prize.id),
                };

                entry
                    .insert(&ctx.state.db)
                    .await
                    .map_err(|e| ServerFnError::new(e.to_string()))?;
                assignments_count += 1;
            }
        }
    }

    Ok(GenerationResult {
        message: format!(
            "Assigned {} prize tracks across submissions",
            assignments_count
        ),
        count: assignments_count,
    })
}

/// Assign random scores (UX, Tech, Applovin) to project descriptions
#[server]
pub async fn assign_random_scores(slug: String) -> Result<GenerationResult, ServerFnError> {
    use crate::entities::{submission, teams};
    use rand::Rng;
    use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

    let synced_user = dioxus::fullstack::FullstackContext::extract::<
        crate::core::auth::middleware::SyncedUser,
        _,
    >()
    .await
    .map_err(|e| ServerFnError::new(format!("Failed to extract user: {}", e)))?;

    let ctx = crate::core::auth::context::RequestContext::extract(&synced_user)
        .await?
        .with_hackathon(&slug)
        .await?;

    let hackathon = ctx.hackathon()?;

    // Get all submissions
    let submissions: Vec<(submission::Model, Option<teams::Model>)> = submission::Entity::find()
        .find_also_related(teams::Entity)
        .filter(teams::Column::HackathonId.eq(hackathon.id))
        .all(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let mut updated_count = 0;
    let mut rng = rand::rng();

    for (sub, _team) in submissions.into_iter() {
        let mut active: submission::ActiveModel = sub.clone().into();

        // Get current description
        let mut data = sub.submission_data;
        let mut description = data
            .get("description")
            .and_then(|d| d.as_str())
            .unwrap_or("")
            .to_string();

        // Check if already has scores (simple check to avoid appending multiple times if run twice)
        // We will just append anyway as per request "adds to the description", but maybe avoiding duplicate is better?
        // User didn't specify idempotent, but it's good practice.
        // But for mock data tools, "add" implies appending. I'll just append.

        let ux_score: f64 = rng.gen_range(0.0..=10.0);
        let tech_score: f64 = rng.gen_range(0.0..=10.0);
        let applovin_score: f64 = rng.gen_range(0.0..=10.0);

        let score_string = format!(
            "\n\n[Mock Scores] UX: {:.2} Tech: {:.2} Applovin: {:.2}",
            ux_score, tech_score, applovin_score
        );

        description.push_str(&score_string);

        // Update json
        if let Some(obj) = data.as_object_mut() {
            obj.insert(
                "description".to_string(),
                serde_json::Value::String(description),
            );
        }

        active.submission_data = Set(data);
        active
            .update(&ctx.state.db)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        updated_count += 1;
    }

    Ok(GenerationResult {
        message: format!("Added random scores to {} projects", updated_count),
        count: updated_count,
    })
}

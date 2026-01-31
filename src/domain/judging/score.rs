#![cfg(feature = "server")]

use crate::domain::judging::rank::Rank;
use crate::entities::{
    feature, hackathons, pairwise_comparison, project_feature_score, submission,
};
use dioxus::prelude::*;
use ndarray::Array2;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
    TransactionTrait,
};
use std::collections::HashMap;

/// Update rankings for all active hackathons (where judging has started)
pub async fn update_all_active_rankings(db: &DatabaseConnection) -> Result<(), ServerFnError> {
    let active_hackathons = hackathons::Entity::find()
        .filter(hackathons::Column::JudgingStarted.eq(true))
        .all(db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch active hackathons: {}", e)))?;

    for hackathon in active_hackathons {
        if let Err(e) = update_hackathon_rankings(db, hackathon.id).await {
            tracing::error!(
                "Failed to update rankings for hackathon {}: {}",
                hackathon.slug,
                e
            );
        }
    }

    Ok(())
}

/// Update rankings for a specific hackathon
pub async fn update_hackathon_rankings(
    db: &DatabaseConnection,
    hackathon_id: i32,
) -> Result<(), ServerFnError> {
    // 1. Fetch all features for this hackathon
    let features = feature::Entity::find()
        .filter(feature::Column::HackathonId.eq(hackathon_id))
        .all(db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch features: {}", e)))?;

    // 2. Fetch all submissions (teams) for this hackathon
    // We navigate via the Teams entity since submissions are linked to teams
    let submissions = submission::Entity::find()
        .find_also_related(crate::entities::teams::Entity)
        .filter(crate::entities::teams::Column::HackathonId.eq(hackathon_id))
        .all(db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch submissions: {}", e)))?;

    let submission_ids: Vec<i32> = submissions.iter().map(|(s, _)| s.id).collect();
    let n = submission_ids.len();

    if n < 2 {
        // Not enough submissions to rank
        return Ok(());
    }

    let id_to_idx: HashMap<i32, usize> = submission_ids
        .iter()
        .enumerate()
        .map(|(i, &id)| (id, i))
        .collect();

    for feature in features {
        // 3. Fetch all pairwise comparisons for this feature
        let comparisons = pairwise_comparison::Entity::find()
            .filter(pairwise_comparison::Column::FeatureId.eq(feature.id))
            .filter(pairwise_comparison::Column::WinnerId.is_not_null())
            .all(db)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to fetch comparisons: {}", e)))?;

        if comparisons.is_empty() {
            continue;
        }

        // 4. Construct ranking matrix
        // ranks[i, j] = number of times i beat j
        let mut ranks = Array2::<i32>::zeros((n, n));

        for comp in comparisons {
            let idx_a = match id_to_idx.get(&comp.submission_a_id) {
                Some(&idx) => idx,
                None => continue,
            };
            let idx_b = match id_to_idx.get(&comp.submission_b_id) {
                Some(&idx) => idx,
                None => continue,
            };

            if let Some(winner_id) = comp.winner_id {
                if winner_id == comp.submission_a_id {
                    ranks[[idx_a, idx_b]] += 1;
                } else if winner_id == comp.submission_b_id {
                    ranks[[idx_b, idx_a]] += 1;
                }
            }
        }

        // 5. Calculate scores using Rank struct
        // Initialize means with 0.0
        let means = vec![0.0; n];
        let mut rank_algo = Rank::new(ranks);
        rank_algo.calc_expected_means();

        // Note: Rank struct and calc_expected_means must be public/accessible
        let calculated_means = &rank_algo.means;

        // 6. Update ProjectFeatureScore in DB
        let txn = db
            .begin()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to begin transaction: {}", e)))?;

        for (i, &submission_id) in submission_ids.iter().enumerate() {
            let score = calculated_means[i];

            // Check if score exists
            let existing_score = project_feature_score::Entity::find()
                .filter(project_feature_score::Column::SubmissionId.eq(submission_id))
                .filter(project_feature_score::Column::FeatureId.eq(feature.id))
                .one(&txn)
                .await
                .map_err(|e| ServerFnError::new(format!("Failed to fetch score: {}", e)))?;

            if let Some(existing) = existing_score {
                let mut active: project_feature_score::ActiveModel = existing.into();
                active.score = Set(Some(score as f32));
                active.updated_at = Set(chrono::Utc::now().naive_utc());
                active
                    .update(&txn)
                    .await
                    .map_err(|e| ServerFnError::new(format!("Failed to update score: {}", e)))?;
            } else {
                let active = project_feature_score::ActiveModel {
                    submission_id: Set(submission_id),
                    feature_id: Set(feature.id),
                    score: Set(Some(score as f32)),
                    variance: Set(None),
                    updated_at: Set(chrono::Utc::now().naive_utc()),
                    ..Default::default()
                };
                active
                    .insert(&txn)
                    .await
                    .map_err(|e| ServerFnError::new(format!("Failed to insert score: {}", e)))?;
            }
        }

        txn.commit()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to commit transaction: {}", e)))?;
    }

    Ok(())
}

//! Feature CRUD endpoints for hackathon judging criteria.

use crate::domain::judging::types::*;
use dioxus::prelude::*;

#[cfg(feature = "server")]
use crate::core::auth::{
    context::RequestContext, middleware::SyncedUser, permissions::Permissions,
};

/// Get all features for a hackathon
#[cfg_attr(feature = "server", utoipa::path(
    get,
    path = "/api/hackathons/{slug}/judging/features",
    params(
        ("slug" = String, Path, description = "Hackathon slug")
    ),
    responses(
        (status = 200, description = "Features retrieved", body = Vec<FeatureInfo>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Server error")
    ),
    tag = "judging"
))]
#[get("/api/hackathons/:slug/judging/features", user: SyncedUser)]
pub async fn get_features(slug: String) -> Result<Vec<FeatureInfo>, ServerFnError> {
    use crate::entities::feature;
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    let hackathon = ctx.hackathon()?;

    let features = feature::Entity::find()
        .filter(feature::Column::HackathonId.eq(hackathon.id))
        .all(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch features: {}", e)))?;

    Ok(features
        .into_iter()
        .map(|f| FeatureInfo {
            id: f.id,
            name: f.name,
            description: f.description,
        })
        .collect())
}

/// Create a new feature for a hackathon
#[cfg_attr(feature = "server", utoipa::path(
    post,
    path = "/api/hackathons/{slug}/judging/features",
    params(
        ("slug" = String, Path, description = "Hackathon slug")
    ),
    request_body = CreateFeatureRequest,
    responses(
        (status = 200, description = "Feature created", body = FeatureInfo),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Server error")
    ),
    tag = "judging"
))]
#[post("/api/hackathons/:slug/judging/features", user: SyncedUser)]
pub async fn create_feature(
    slug: String,
    request: CreateFeatureRequest,
) -> Result<FeatureInfo, ServerFnError> {
    use crate::entities::feature;
    use sea_orm::{ActiveModelTrait, ActiveValue::NotSet, Set};

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    Permissions::require_admin_or_organizer(&ctx).await?;
    let hackathon = ctx.hackathon()?;

    let new_feature = feature::ActiveModel {
        id: NotSet,
        hackathon_id: Set(hackathon.id),
        name: Set(request.name),
        description: Set(request.description),
    };

    let feature = new_feature
        .insert(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to create feature: {}", e)))?;

    Ok(FeatureInfo {
        id: feature.id,
        name: feature.name,
        description: feature.description,
    })
}

/// Update a feature
#[cfg_attr(feature = "server", utoipa::path(
    put,
    path = "/api/hackathons/{slug}/judging/features/{feature_id}",
    params(
        ("slug" = String, Path, description = "Hackathon slug"),
        ("feature_id" = i32, Path, description = "Feature ID")
    ),
    request_body = UpdateFeatureRequest,
    responses(
        (status = 200, description = "Feature updated", body = FeatureInfo),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Feature not found"),
        (status = 500, description = "Server error")
    ),
    tag = "judging"
))]
#[put("/api/hackathons/:slug/judging/features/:feature_id", user: SyncedUser)]
pub async fn update_feature(
    slug: String,
    feature_id: i32,
    request: UpdateFeatureRequest,
) -> Result<FeatureInfo, ServerFnError> {
    use crate::entities::feature;
    use sea_orm::{ActiveModelTrait, EntityTrait, Set};

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    let hackathon = ctx.hackathon()?;

    Permissions::require_admin_or_organizer(&ctx).await?;

    // Find the feature
    let existing = feature::Entity::find_by_id(feature_id)
        .one(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to find feature: {}", e)))?
        .ok_or_else(|| ServerFnError::new("Feature not found"))?;

    // Verify it belongs to this hackathon
    if existing.hackathon_id != hackathon.id {
        return Err(ServerFnError::new("Feature not found in this hackathon"));
    }

    let mut active: feature::ActiveModel = existing.into();
    active.name = Set(request.name);
    active.description = Set(request.description);

    let updated = active
        .update(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to update feature: {}", e)))?;

    Ok(FeatureInfo {
        id: updated.id,
        name: updated.name,
        description: updated.description,
    })
}

/// Delete a feature
#[cfg_attr(feature = "server", utoipa::path(
    delete,
    path = "/api/hackathons/{slug}/judging/features/{feature_id}",
    params(
        ("slug" = String, Path, description = "Hackathon slug"),
        ("feature_id" = i32, Path, description = "Feature ID")
    ),
    responses(
        (status = 200, description = "Feature deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Feature not found"),
        (status = 500, description = "Server error")
    ),
    tag = "judging"
))]
#[delete("/api/hackathons/:slug/judging/features/:feature_id", user: SyncedUser)]
pub async fn delete_feature(slug: String, feature_id: i32) -> Result<(), ServerFnError> {
    use crate::entities::feature;
    use sea_orm::{EntityTrait, ModelTrait};

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    let hackathon = ctx.hackathon()?;

    Permissions::require_admin_or_organizer(&ctx).await?;

    // Find the feature
    let existing = feature::Entity::find_by_id(feature_id)
        .one(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to find feature: {}", e)))?
        .ok_or_else(|| ServerFnError::new("Feature not found"))?;

    // Verify it belongs to this hackathon
    if existing.hackathon_id != hackathon.id {
        return Err(ServerFnError::new("Feature not found in this hackathon"));
    }

    existing
        .delete(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to delete feature: {}", e)))?;

    Ok(())
}

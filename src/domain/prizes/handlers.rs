use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use crate::core::auth::{
    context::RequestContext, middleware::SyncedUser, permissions::Permissions,
};

#[cfg(feature = "server")]
use utoipa::ToSchema;

/// Feature weight info for a prize
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct PrizeFeatureWeightInfo {
    pub feature_id: i32,
    pub weight: f32,
}

/// Request payload for updating a prize
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct UpdatePrizeRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub category: Option<String>,
    pub value: Option<String>,
    pub required_event_ids: Option<Vec<i32>>,
}

/// Prize info returned from handlers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct PrizeInfo {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub category: Option<String>,
    pub value: String,
    pub feature_weights: Vec<PrizeFeatureWeightInfo>,
    pub required_event_ids: Vec<i32>,
}

/// Request payload for creating a prize
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct CreatePrizeRequest {
    pub name: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub category: Option<String>,
    pub value: String,
    pub required_event_ids: Vec<i32>,
}

/// Request payload for updating prize feature weights
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct UpdatePrizeFeatureWeightsRequest {
    pub weights: Vec<PrizeFeatureWeightInfo>,
}

/// Get all prizes
#[cfg_attr(feature = "server", utoipa::path(
    get,
    path = "/api/hackathons/{slug}/prizes",
    params(
        ("slug" = String, Path, description = "Hackathon slug")
    ),
    responses(
        (status = 200, description = "Prizes retrieved successfully", body = Vec<PrizeInfo>),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Hackathon not found"),
        (status = 500, description = "Server error")
    ),
    tag = "prizes"
))]
#[get("/api/hackathons/:slug/prizes", user: SyncedUser)]
pub async fn get_prizes(slug: String) -> Result<Vec<PrizeInfo>, ServerFnError> {
    use crate::entities::{prize, prize_feature_weight, prize_required_events};
    use sea_orm::{ColumnTrait, EntityTrait, ModelTrait, QueryFilter};

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    // Fetch all prizes
    let prizes = prize::Entity::find()
        .all(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch prizes: {}", e)))?;

    let mut result = Vec::new();

    // Fetch all required events for these prizes in one go ideally, but loop is fine for now
    // Or use efficient loading
    let prize_ids: Vec<i32> = prizes.iter().map(|p| p.id).collect();
    let all_requirements = prize_required_events::Entity::find()
        .filter(prize_required_events::Column::PrizeId.is_in(prize_ids))
        .all(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch requirements: {}", e)))?;

    // Group by prize_id
    use std::collections::HashMap;
    let mut requirements_map: HashMap<i32, Vec<i32>> = HashMap::new();
    for req in all_requirements {
        requirements_map
            .entry(req.prize_id)
            .or_default()
            .push(req.event_id);
    }

    for p in prizes {
        // Fetch feature weights for this prize
        let weights = prize_feature_weight::Entity::find()
            .filter(prize_feature_weight::Column::PrizeId.eq(p.id))
            .all(&ctx.state.db)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to fetch weights: {}", e)))?;

        let required_event_ids = requirements_map.remove(&p.id).unwrap_or_default();

        result.push(PrizeInfo {
            id: p.id,
            name: p.name,
            description: p.description,
            image_url: p.image_url,
            category: p.category,
            value: p.value,
            feature_weights: weights
                .into_iter()
                .map(|w| PrizeFeatureWeightInfo {
                    feature_id: w.feature_id,
                    weight: w.weight,
                })
                .collect(),
            required_event_ids,
        });
    }

    Ok(result)
}

/// Create a new prize (admin/organizer only)
#[cfg_attr(feature = "server", utoipa::path(
    post,
    path = "/api/hackathons/{slug}/prizes",
    params(
        ("slug" = String, Path, description = "Hackathon slug")
    ),
    request_body = CreatePrizeRequest,
    responses(
        (status = 201, description = "Prize created successfully", body = PrizeInfo),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - admin/organizer only"),
        (status = 404, description = "Hackathon not found"),
        (status = 500, description = "Server error")
    ),
    tag = "prizes"
))]
#[post("/api/hackathons/:slug/prizes", user: SyncedUser)]
pub async fn create_prize(
    slug: String,
    request: CreatePrizeRequest,
) -> Result<PrizeInfo, ServerFnError> {
    use crate::domain::people::repository::UserRoleRepository;
    use crate::entities::prize;
    use sea_orm::{ActiveModelTrait, Set};

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    let hackathon = ctx.hackathon()?;

    // Check if user is admin or organizer (global or hackathon-level)
    let is_global_admin = Permissions::is_global_admin(&ctx);
    let role_repo = UserRoleRepository::new(&ctx.state.db);
    let user_role = role_repo.find_user_role(ctx.user.id, hackathon.id).await?;

    let is_admin_or_organizer = user_role
        .as_ref()
        .map(|r| r.role == "admin" || r.role == "organizer")
        .unwrap_or(false);

    if !is_global_admin && !is_admin_or_organizer {
        return Err(ServerFnError::new(
            "Only admins and organizers can create prizes",
        ));
    }

    // Create the prize with hackathon_id
    let prize_model = prize::ActiveModel {
        name: Set(request.name.clone()),
        description: Set(request.description.clone()),
        image_url: Set(request.image_url.clone()),
        category: Set(request.category.clone()),
        value: Set(request.value.clone()),
        hackathon_id: Set(Some(hackathon.id)),
        ..Default::default()
    };

    let inserted = prize_model
        .insert(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to create prize: {}", e)))?;

    // Insert required events
    for event_id in request.required_event_ids.iter() {
        use crate::entities::prize_required_events;
        let req = prize_required_events::ActiveModel {
            prize_id: Set(inserted.id),
            event_id: Set(*event_id),
        };
        req.insert(&ctx.state.db)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to add required event: {}", e)))?;
    }

    Ok(PrizeInfo {
        id: inserted.id,
        name: inserted.name,
        description: inserted.description,
        image_url: inserted.image_url,
        category: inserted.category,
        value: inserted.value,
        feature_weights: Vec::new(),
        required_event_ids: request.required_event_ids,
    })
}

/// Update a prize (admin/organizer only)
#[cfg_attr(feature = "server", utoipa::path(
    put,
    path = "/api/hackathons/{slug}/prizes/{id}",
    params(
        ("slug" = String, Path, description = "Hackathon slug"),
        ("id" = i32, Path, description = "Prize ID")
    ),
    request_body = UpdatePrizeRequest,
    responses(
        (status = 200, description = "Prize updated successfully", body = PrizeInfo),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - admin/organizer only"),
        (status = 404, description = "Prize not found"),
        (status = 500, description = "Server error")
    ),
    tag = "prizes"
))]
#[put("/api/hackathons/:slug/prizes/:id", user: SyncedUser)]
pub async fn update_prize(
    slug: String,
    id: i32,
    request: UpdatePrizeRequest,
) -> Result<PrizeInfo, ServerFnError> {
    use crate::domain::people::repository::UserRoleRepository;
    use crate::entities::{prize, prize_feature_weight, prize_required_events};
    use sea_orm::{
        ActiveModelTrait, ColumnTrait, EntityTrait, ModelTrait, QueryFilter, Set, TransactionTrait,
    };

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    let hackathon = ctx.hackathon()?;

    // Check permissions
    let is_global_admin = Permissions::is_global_admin(&ctx);
    let role_repo = UserRoleRepository::new(&ctx.state.db);
    let user_role = role_repo.find_user_role(ctx.user.id, hackathon.id).await?;

    let is_admin_or_organizer = user_role
        .as_ref()
        .map(|r| r.role == "admin" || r.role == "organizer")
        .unwrap_or(false);

    if !is_global_admin && !is_admin_or_organizer {
        return Err(ServerFnError::new(
            "Only admins and organizers can update prizes",
        ));
    }

    // Find the prize
    let prize_model = prize::Entity::find_by_id(id)
        .one(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to find prize: {}", e)))?
        .ok_or_else(|| ServerFnError::new("Prize not found"))?;

    // Start transaction
    let txn = ctx
        .state
        .db
        .begin()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to start transaction: {}", e)))?;

    // Update basic fields
    let mut active: prize::ActiveModel = prize_model.clone().into();
    if let Some(name) = request.name {
        active.name = Set(name);
    }
    if let Some(desc) = request.description {
        active.description = Set(Some(desc));
    }
    if let Some(url) = request.image_url {
        active.image_url = Set(Some(url));
    }
    if let Some(cat) = request.category {
        active.category = Set(Some(cat));
    }
    if let Some(val) = request.value {
        active.value = Set(val);
    }

    let updated = active
        .update(&txn)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to update prize: {}", e)))?;

    // Update required events if provided
    let mut required_event_ids = Vec::new();
    if let Some(req_ids) = request.required_event_ids {
        // Delete existing requirements
        prize_required_events::Entity::delete_many()
            .filter(prize_required_events::Column::PrizeId.eq(id))
            .exec(&txn)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to clear requirements: {}", e)))?;

        // Insert new requirements
        for event_id in &req_ids {
            let req = prize_required_events::ActiveModel {
                prize_id: Set(id),
                event_id: Set(*event_id),
            };
            req.insert(&txn)
                .await
                .map_err(|e| ServerFnError::new(format!("Failed to add requirement: {}", e)))?;
        }
        required_event_ids = req_ids;
    } else {
        // Fetch existing if not updated
        let reqs = prize_required_events::Entity::find()
            .filter(prize_required_events::Column::PrizeId.eq(id))
            .all(&txn)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to fetch requirements: {}", e)))?;
        required_event_ids = reqs.iter().map(|r| r.event_id).collect();
    }

    txn.commit()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to commit transaction: {}", e)))?;

    // Fetch weights for response
    let weights = prize_feature_weight::Entity::find()
        .filter(prize_feature_weight::Column::PrizeId.eq(id))
        .all(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch weights: {}", e)))?;

    Ok(PrizeInfo {
        id: updated.id,
        name: updated.name,
        description: updated.description,
        image_url: updated.image_url,
        category: updated.category,
        value: updated.value,
        feature_weights: weights
            .into_iter()
            .map(|w| PrizeFeatureWeightInfo {
                feature_id: w.feature_id,
                weight: w.weight,
            })
            .collect(),
        required_event_ids,
    })
}

/// Delete a prize (admin/organizer only)
#[cfg_attr(feature = "server", utoipa::path(
    delete,
    path = "/api/hackathons/{slug}/prizes/{id}",
    params(
        ("slug" = String, Path, description = "Hackathon slug"),
        ("id" = i32, Path, description = "Prize ID")
    ),
    responses(
        (status = 200, description = "Prize deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - admin/organizer only"),
        (status = 404, description = "Prize not found"),
        (status = 500, description = "Server error")
    ),
    tag = "prizes"
))]
#[delete("/api/hackathons/:slug/prizes/:id", user: SyncedUser)]
pub async fn delete_prize(slug: String, id: i32) -> Result<(), ServerFnError> {
    use crate::domain::people::repository::UserRoleRepository;
    use crate::entities::prize;
    use sea_orm::{EntityTrait, ModelTrait};

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    let hackathon = ctx.hackathon()?;

    // Check if user is admin or organizer (global or hackathon-level)
    let is_global_admin = Permissions::is_global_admin(&ctx);
    let role_repo = UserRoleRepository::new(&ctx.state.db);
    let user_role = role_repo.find_user_role(ctx.user.id, hackathon.id).await?;

    let is_admin_or_organizer = user_role
        .as_ref()
        .map(|r| r.role == "admin" || r.role == "organizer")
        .unwrap_or(false);

    if !is_global_admin && !is_admin_or_organizer {
        return Err(ServerFnError::new(
            "Only admins and organizers can delete prizes",
        ));
    }

    // Find and delete the prize
    let prize = prize::Entity::find_by_id(id)
        .one(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to find prize: {}", e)))?
        .ok_or_else(|| ServerFnError::new("Prize not found"))?;

    prize
        .delete(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to delete prize: {}", e)))?;

    Ok(())
}

/// Update prize feature weights (admin/organizer only)
#[cfg_attr(feature = "server", utoipa::path(
    put,
    path = "/api/hackathons/{slug}/prizes/{id}/weights",
    params(
        ("slug" = String, Path, description = "Hackathon slug"),
        ("id" = i32, Path, description = "Prize ID")
    ),
    request_body = UpdatePrizeFeatureWeightsRequest,
    responses(
        (status = 200, description = "Weights updated successfully"),
        (status = 400, description = "Invalid weights (must sum to 1)"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - admin/organizer only"),
        (status = 404, description = "Prize not found"),
        (status = 500, description = "Server error")
    ),
    tag = "prizes"
))]
#[put("/api/hackathons/:slug/prizes/:id/weights", user: SyncedUser)]
pub async fn update_prize_feature_weights(
    slug: String,
    id: i32,
    request: UpdatePrizeFeatureWeightsRequest,
) -> Result<(), ServerFnError> {
    use crate::domain::people::repository::UserRoleRepository;
    use crate::entities::prize_feature_weight;
    use sea_orm::{
        ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, EntityTrait, QueryFilter, Set,
        TransactionTrait,
    };

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    let hackathon = ctx.hackathon()?;

    // Check permissions
    let is_global_admin = Permissions::is_global_admin(&ctx);
    let role_repo = UserRoleRepository::new(&ctx.state.db);
    let user_role = role_repo.find_user_role(ctx.user.id, hackathon.id).await?;

    let is_admin_or_organizer = user_role
        .as_ref()
        .map(|r| r.role == "admin" || r.role == "organizer")
        .unwrap_or(false);

    if !is_global_admin && !is_admin_or_organizer {
        return Err(ServerFnError::new(
            "Only admins and organizers can update prize weights",
        ));
    }

    // Validate weights sum to 1.0 (allow small epsilon)
    let sum: f32 = request.weights.iter().map(|w| w.weight).sum();
    if (sum - 1.0).abs() > 0.001 {
        return Err(ServerFnError::new(
            "Feature weights must sum to exactly 1.0",
        ));
    }

    // Transaction
    let txn = ctx
        .state
        .db
        .begin()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to start transaction: {}", e)))?;

    // Delete existing weights
    prize_feature_weight::Entity::delete_many()
        .filter(prize_feature_weight::Column::PrizeId.eq(id))
        .exec(&txn)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to delete existing weights: {}", e)))?;

    // Insert new weights
    for w in request.weights {
        let new_weight = prize_feature_weight::ActiveModel {
            id: NotSet,
            prize_id: Set(id),
            feature_id: Set(w.feature_id),
            weight: Set(w.weight),
        };

        new_weight
            .insert(&txn)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to insert weight: {}", e)))?;
    }

    txn.commit()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to commit transaction: {}", e)))?;

    Ok(())
}

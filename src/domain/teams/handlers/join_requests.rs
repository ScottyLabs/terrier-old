use crate::domain::teams::types::*;
use dioxus::prelude::*;
use serde_json::Value as JsonValue;

#[cfg(feature = "server")]
use crate::domain::applications::repository::ApplicationRepository;

#[cfg(feature = "server")]
use tracing::info;

#[cfg(feature = "server")]
use crate::core::auth::{
    context::RequestContext, middleware::SyncedUser, permissions::Permissions,
};
#[cfg(feature = "server")]
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};

/// Request to join a team
#[cfg_attr(feature = "server", utoipa::path(
    post,
    path = "/api/hackathons/{slug}/team/request-join",
    params(
        ("slug" = String, Path, description = "Hackathon slug")
    ),
    request_body = JoinTeamRequest,
    responses(
        (status = 200, description = "Join request created successfully"),
        (status = 400, description = "Team is full or user already in team or request already exists"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Hackathon or team not found"),
        (status = 500, description = "Server error")
    ),
    tag = "teams"
))]
#[post("/api/hackathons/:slug/team/request-join", user: SyncedUser)]
pub async fn request_join_team(slug: String, req: JoinTeamRequest) -> Result<(), ServerFnError> {
    use crate::domain::teams::repository::TeamRepository;

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    let hackathon = ctx.hackathon()?;

    // Check if user is already in a team
    let team_repo = TeamRepository::new(&ctx.state.db);
    let user_role = team_repo.find_user_role(ctx.user.id, hackathon.id).await?;

    let Some(role) = user_role else {
        return Err(ServerFnError::new("User not registered for this hackathon"));
    };

    if role.team_id.is_some() {
        return Err(ServerFnError::new("You are already in a team"));
    }

    // Verify team exists
    let team = team_repo.find_by_id(req.team_id).await?;

    if team.hackathon_id != hackathon.id {
        return Err(ServerFnError::new("Team does not belong to this hackathon"));
    }

    // Check if team is full
    let member_count = team_repo
        .count_team_members(req.team_id, hackathon.id)
        .await?;

    if member_count >= hackathon.max_team_size as usize {
        return Err(ServerFnError::new("Team is full"));
    }

    // Check if user already has a pending request for this team
    let existing_request = crate::entities::prelude::TeamJoinRequests::find()
        .filter(crate::entities::team_join_requests::Column::TeamId.eq(req.team_id))
        .filter(crate::entities::team_join_requests::Column::UserId.eq(ctx.user.id))
        .one(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to check existing request: {}", e)))?;

    if existing_request.is_some() {
        return Err(ServerFnError::new(
            "You already have a pending request for this team",
        ));
    }

    // Create join request
    // Snapshot requester info into the join request so it remains available
    let request_user = crate::entities::prelude::Users::find_by_id(ctx.user.id)
        .one(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch request user: {}", e)))?
        .ok_or_else(|| ServerFnError::new("Request user not found"))?;

    // Try to extract major/graduation_year from the user's application
    let mut major: Option<String> = None;
    let mut graduation_year: Option<String> = None;
    let app_repo = ApplicationRepository::new(&ctx.state.db);
    if let Ok(Some(app)) = app_repo
        .find_by_user_and_hackathon(ctx.user.id, hackathon.id)
        .await
    {
        let mut form_json: JsonValue = app.form_data.clone();
        if form_json.is_string() {
            if let Some(s) = form_json.as_str() {
                if let Ok(parsed) = serde_json::from_str::<JsonValue>(s) {
                    form_json = parsed;
                }
            }
        }

        let mut extract_string = |val: &JsonValue| -> Option<String> {
            if val.is_string() {
                val.as_str().map(|s| s.to_string())
            } else if val.is_number() {
                Some(val.to_string())
            } else {
                None
            }
        };

        if let Some(obj) = form_json.as_object() {
            if let Some(v) = obj.get("major") {
                major = extract_string(v);
            }
            if let Some(v) = obj.get("Major") {
                major = major.or_else(|| extract_string(v));
            }
            if let Some(v) = obj.get("graduation_year") {
                graduation_year = extract_string(v);
            }
            if let Some(v) = obj.get("graduationYear") {
                graduation_year = graduation_year.or_else(|| extract_string(v));
            }
            if let Some(v) = obj.get("graduation") {
                graduation_year = graduation_year.or_else(|| extract_string(v));
            }
        } else if let Some(arr) = form_json.as_array() {
            for entry in arr.iter() {
                if let Some(obj) = entry.as_object() {
                    if let Some(name_val) = obj
                        .get("name")
                        .or_else(|| obj.get("field"))
                        .or_else(|| obj.get("label"))
                    {
                        if let Some(name_str) = name_val.as_str() {
                            if name_str.eq_ignore_ascii_case("major") && major.is_none() {
                                if let Some(v) = obj.get("value") {
                                    major = extract_string(v);
                                }
                            }
                            if (name_str.eq_ignore_ascii_case("graduation_year")
                                || name_str.eq_ignore_ascii_case("graduationYear")
                                || name_str.eq_ignore_ascii_case("graduation"))
                                && graduation_year.is_none()
                            {
                                if let Some(v) = obj.get("value") {
                                    graduation_year = extract_string(v);
                                }
                            }
                        }
                    }
                }
            }
        }

        fn find_by_key_substring(v: &JsonValue, subs: &[&str]) -> Option<String> {
            match v {
                JsonValue::Object(map) => {
                    for (k, val) in map.iter() {
                        let lk = k.to_lowercase();
                        for sub in subs.iter() {
                            if lk.contains(&sub.to_lowercase()) {
                                if val.is_string() {
                                    return val.as_str().map(|s| s.to_string());
                                }
                                if val.is_number() {
                                    return Some(val.to_string());
                                }
                            }
                        }
                        if let Some(found) = find_by_key_substring(val, subs) {
                            return Some(found);
                        }
                    }
                    None
                }
                JsonValue::Array(arr) => {
                    for item in arr.iter() {
                        if let Some(found) = find_by_key_substring(item, subs) {
                            return Some(found);
                        }
                    }
                    None
                }
                _ => None,
            }
        }

        if major.is_none() {
            major = find_by_key_substring(&form_json, &["major", "program"]);
        }
        if graduation_year.is_none() {
            graduation_year = find_by_key_substring(&form_json, &["graduation", "grad", "year"]);
        }
    }

    let new_request = crate::entities::team_join_requests::ActiveModel {
        team_id: Set(req.team_id),
        user_id: Set(ctx.user.id),
        message: Set(req.message),
        person_name: Set(request_user.name.clone()),
        person_email: Set(Some(request_user.email.clone())),
        person_picture: Set(request_user.picture.clone()),
        person_major: Set(major.clone()),
        person_graduation_year: Set(graduation_year.clone()),
        ..Default::default()
    };

    new_request
        .insert(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to create join request: {}", e)))?;

    Ok(())
}

/// Get pending join requests for user's team (owner only)
#[cfg_attr(feature = "server", utoipa::path(
    get,
    path = "/api/hackathons/{slug}/team/requests",
    params(
        ("slug" = String, Path, description = "Hackathon slug")
    ),
    responses(
        (status = 200, description = "Join requests retrieved successfully", body = Vec<JoinRequestResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Only team owner can view requests"),
        (status = 404, description = "Hackathon not found or user not in team"),
        (status = 500, description = "Server error")
    ),
    tag = "teams"
))]
#[get("/api/hackathons/:slug/team/requests", user: SyncedUser)]
pub async fn get_join_requests(slug: String) -> Result<Vec<JoinRequestResponse>, ServerFnError> {
    use crate::domain::teams::repository::TeamRepository;

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    let hackathon = ctx.hackathon()?;

    // Get user's team_id
    let team_repo = TeamRepository::new(&ctx.state.db);
    let user_role = team_repo
        .find_user_role_or_error(
            ctx.user.id,
            hackathon.id,
            "User not registered for this hackathon",
        )
        .await?;

    let team_id = user_role
        .team_id
        .ok_or_else(|| ServerFnError::new("User is not in a team"))?;

    // Verify user is the team owner
    Permissions::require_team_ownership(&ctx, team_id).await?;

    // Fetch join requests for this team
    let requests = crate::entities::prelude::TeamJoinRequests::find()
        .filter(crate::entities::team_join_requests::Column::TeamId.eq(team_id))
        .order_by_desc(crate::entities::team_join_requests::Column::CreatedAt)
        .all(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch join requests: {}", e)))?;

    let mut result = Vec::new();
    // Application repo to enrich responses
    let app_repo = ApplicationRepository::new(&ctx.state.db);

    for request in requests {
        let request_user = crate::entities::prelude::Users::find_by_id(request.user_id)
            .one(&ctx.state.db)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to fetch user: {}", e)))?
            .ok_or_else(|| ServerFnError::new("Request user not found"))?;

        // Extract major/graduation_year from application if present
        let mut major: Option<String> = None;
        let mut graduation_year: Option<String> = None;
        if let Ok(Some(app)) = app_repo
            .find_by_user_and_hackathon(request.user_id, hackathon.id)
            .await
        {
            let mut form_json: JsonValue = app.form_data.clone();
            if form_json.is_string() {
                if let Some(s) = form_json.as_str() {
                    if let Ok(parsed) = serde_json::from_str::<JsonValue>(s) {
                        form_json = parsed;
                    }
                }
            }

            let mut extract_string = |val: &JsonValue| -> Option<String> {
                if val.is_string() {
                    val.as_str().map(|s| s.to_string())
                } else if val.is_number() {
                    Some(val.to_string())
                } else {
                    None
                }
            };

            // Recursive search for keys containing substrings (case-insensitive)
            fn find_by_key_substring(v: &JsonValue, subs: &[&str]) -> Option<String> {
                match v {
                    JsonValue::Object(map) => {
                        for (k, val) in map.iter() {
                            let lk = k.to_lowercase();
                            for sub in subs.iter() {
                                if lk.contains(&sub.to_lowercase()) {
                                    // try to extract
                                    if val.is_string() {
                                        return val.as_str().map(|s| s.to_string());
                                    }
                                    if val.is_number() {
                                        return Some(val.to_string());
                                    }
                                }
                            }
                            if let Some(found) = find_by_key_substring(val, subs) {
                                return Some(found);
                            }
                        }
                        None
                    }
                    JsonValue::Array(arr) => {
                        for item in arr.iter() {
                            if let Some(found) = find_by_key_substring(item, subs) {
                                return Some(found);
                            }
                        }
                        None
                    }
                    JsonValue::String(s) => {
                        // if string contains a key-like pattern, return it? skip
                        None
                    }
                    _ => None,
                }
            }

            if let Some(obj) = form_json.as_object() {
                if let Some(v) = obj.get("major") {
                    major = extract_string(v);
                }
                if let Some(v) = obj.get("Major") {
                    major = major.or_else(|| extract_string(v));
                }
                if let Some(v) = obj.get("graduation_year") {
                    graduation_year = extract_string(v);
                }
                if let Some(v) = obj.get("graduationYear") {
                    graduation_year = graduation_year.or_else(|| extract_string(v));
                }
                if let Some(v) = obj.get("graduation") {
                    graduation_year = graduation_year.or_else(|| extract_string(v));
                }
                // try some generic candidates
                for key in [
                    "major",
                    "Major",
                    "graduation_year",
                    "graduationYear",
                    "graduation",
                ] {
                    if let Some(v) = obj.get(key) {
                        if key.to_lowercase().contains("major") && major.is_none() {
                            major = extract_string(v);
                        } else if key.to_lowercase().contains("graduation")
                            && graduation_year.is_none()
                        {
                            graduation_year = extract_string(v);
                        }
                    }
                }
            } else if let Some(arr) = form_json.as_array() {
                for entry in arr.iter() {
                    if let Some(obj) = entry.as_object() {
                        if let Some(name_val) = obj
                            .get("name")
                            .or_else(|| obj.get("field"))
                            .or_else(|| obj.get("label"))
                        {
                            if let Some(name_str) = name_val.as_str() {
                                if name_str.eq_ignore_ascii_case("major") && major.is_none() {
                                    if let Some(v) = obj.get("value") {
                                        major = extract_string(v);
                                    }
                                }
                                if (name_str.eq_ignore_ascii_case("graduation_year")
                                    || name_str.eq_ignore_ascii_case("graduationYear")
                                    || name_str.eq_ignore_ascii_case("graduation"))
                                    && graduation_year.is_none()
                                {
                                    if let Some(v) = obj.get("value") {
                                        graduation_year = extract_string(v);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Fallback: recursively search for likely keys if not found yet
            if major.is_none() {
                major = find_by_key_substring(&form_json, &["major", "field_major", "program"]);
            }
            if graduation_year.is_none() {
                graduation_year = find_by_key_substring(
                    &form_json,
                    &["graduation", "grad", "year", "graduation_year"],
                );
            }
            // Log parsed form_data for debugging if fields not found
            #[cfg(feature = "server")]
            info!(
                "Parsed application form_data for user {}: major={:?}, graduation_year={:?}, raw={:?}",
                request.user_id, major, graduation_year, form_json
            );
        }

        // Prefer snapshotted values on the request row, otherwise use the freshly fetched user/app data
        let person_name = request.person_name.clone();
        let person_email = request.person_email.clone();
        let person_picture = request.person_picture.clone();
        let person_major = request.person_major.clone();
        let person_graduation_year = request.person_graduation_year.clone();

        let major = person_major.or(major);
        let graduation_year = person_graduation_year.or(graduation_year);

        result.push(JoinRequestResponse {
            id: request.id,
            team_id: request.team_id,
            user_id: request.user_id,
            user_name: person_name.or(request_user.name),
            user_email: person_email.clone().unwrap_or(request_user.email.clone()),
            user_picture: person_picture.or(request_user.picture),
            major,
            graduation_year,
            message: request.message,
            created_at: request.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        });
    }

    Ok(result)
}

/// Get outgoing join requests
#[cfg_attr(feature = "server", utoipa::path(
    get,
    path = "/api/hackathons/{slug}/outgoing-join-requests",
    params(
        ("slug" = String, Path, description = "Hackathon slug")
    ),
    responses(
        (status = 200, description = "Outgoing join requests retrieved successfully", body = Vec<OutgoingJoinRequestResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Hackathon not found"),
        (status = 500, description = "Server error")
    ),
    tag = "teams"
))]
#[get("/api/hackathons/:slug/outgoing-join-requests", user: SyncedUser)]
pub async fn get_outgoing_join_requests(
    slug: String,
) -> Result<Vec<OutgoingJoinRequestResponse>, ServerFnError> {
    use crate::domain::teams::repository::TeamRepository;

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    let hackathon = ctx.hackathon()?;

    // Fetch join requests made by this user for teams in this hackathon
    let requests = crate::entities::prelude::TeamJoinRequests::find()
        .filter(crate::entities::team_join_requests::Column::UserId.eq(ctx.user.id))
        .order_by_desc(crate::entities::team_join_requests::Column::CreatedAt)
        .all(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch join requests: {}", e)))?;

    let team_repo = TeamRepository::new(&ctx.state.db);
    let mut result = Vec::new();
    let app_repo = ApplicationRepository::new(&ctx.state.db);
    for request in requests {
        // Fetch the team to ensure it belongs to this hackathon
        if let Ok(team) = team_repo.find_by_id(request.team_id).await {
            if team.hackathon_id == hackathon.id {
                // Fetch requester info and attempt to extract major/grad from their application
                let request_user = crate::entities::prelude::Users::find_by_id(request.user_id)
                    .one(&ctx.state.db)
                    .await
                    .map_err(|e| ServerFnError::new(format!("Failed to fetch user: {}", e)))?
                    .ok_or_else(|| ServerFnError::new("Request user not found"))?;

                let mut major: Option<String> = None;
                let mut graduation_year: Option<String> = None;
                if let Ok(Some(app)) = app_repo
                    .find_by_user_and_hackathon(request.user_id, hackathon.id)
                    .await
                {
                    let mut form_json: JsonValue = app.form_data.clone();
                    if form_json.is_string() {
                        if let Some(s) = form_json.as_str() {
                            if let Ok(parsed) = serde_json::from_str::<JsonValue>(s) {
                                form_json = parsed;
                            }
                        }
                    }

                    let mut extract_string = |val: &JsonValue| -> Option<String> {
                        if val.is_string() {
                            val.as_str().map(|s| s.to_string())
                        } else if val.is_number() {
                            Some(val.to_string())
                        } else {
                            None
                        }
                    };

                    if let Some(obj) = form_json.as_object() {
                        if let Some(v) = obj.get("major") {
                            major = extract_string(v);
                        }
                        if let Some(v) = obj.get("Major") {
                            major = major.or_else(|| extract_string(v));
                        }
                        if let Some(v) = obj.get("graduation_year") {
                            graduation_year = extract_string(v);
                        }
                        if let Some(v) = obj.get("graduationYear") {
                            graduation_year = graduation_year.or_else(|| extract_string(v));
                        }
                        if let Some(v) = obj.get("graduation") {
                            graduation_year = graduation_year.or_else(|| extract_string(v));
                        }
                    } else if let Some(arr) = form_json.as_array() {
                        for entry in arr.iter() {
                            if let Some(obj) = entry.as_object() {
                                if let Some(name_val) = obj
                                    .get("name")
                                    .or_else(|| obj.get("field"))
                                    .or_else(|| obj.get("label"))
                                {
                                    if let Some(name_str) = name_val.as_str() {
                                        if name_str.eq_ignore_ascii_case("major") && major.is_none()
                                        {
                                            if let Some(v) = obj.get("value") {
                                                major = extract_string(v);
                                            }
                                        }
                                        if (name_str.eq_ignore_ascii_case("graduation_year")
                                            || name_str.eq_ignore_ascii_case("graduationYear")
                                            || name_str.eq_ignore_ascii_case("graduation"))
                                            && graduation_year.is_none()
                                        {
                                            if let Some(v) = obj.get("value") {
                                                graduation_year = extract_string(v);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if major.is_none() {
                        fn find_by_key_substring(v: &JsonValue, subs: &[&str]) -> Option<String> {
                            match v {
                                JsonValue::Object(map) => {
                                    for (k, val) in map.iter() {
                                        let lk = k.to_lowercase();
                                        for sub in subs.iter() {
                                            if lk.contains(&sub.to_lowercase()) {
                                                if val.is_string() {
                                                    return val.as_str().map(|s| s.to_string());
                                                }
                                                if val.is_number() {
                                                    return Some(val.to_string());
                                                }
                                            }
                                        }
                                        if let Some(found) = find_by_key_substring(val, subs) {
                                            return Some(found);
                                        }
                                    }
                                    None
                                }
                                JsonValue::Array(arr) => {
                                    for item in arr.iter() {
                                        if let Some(found) = find_by_key_substring(item, subs) {
                                            return Some(found);
                                        }
                                    }
                                    None
                                }
                                _ => None,
                            }
                        }
                        major = find_by_key_substring(&form_json, &["major", "program"]);
                    }
                    if graduation_year.is_none() {
                        fn find_by_key_substring2(v: &JsonValue, subs: &[&str]) -> Option<String> {
                            match v {
                                JsonValue::Object(map) => {
                                    for (k, val) in map.iter() {
                                        let lk = k.to_lowercase();
                                        for sub in subs.iter() {
                                            if lk.contains(&sub.to_lowercase()) {
                                                if val.is_string() {
                                                    return val.as_str().map(|s| s.to_string());
                                                }
                                                if val.is_number() {
                                                    return Some(val.to_string());
                                                }
                                            }
                                        }
                                        if let Some(found) = find_by_key_substring2(val, subs) {
                                            return Some(found);
                                        }
                                    }
                                    None
                                }
                                JsonValue::Array(arr) => {
                                    for item in arr.iter() {
                                        if let Some(found) = find_by_key_substring2(item, subs) {
                                            return Some(found);
                                        }
                                    }
                                    None
                                }
                                _ => None,
                            }
                        }
                        graduation_year =
                            find_by_key_substring2(&form_json, &["graduation", "grad", "year"]);
                    }
                }

                // Prefer snapshotted person fields on the request row, fallback to fresh user/app data
                let person_name = request.person_name.clone();
                let person_email = request.person_email.clone();
                let person_picture = request.person_picture.clone();
                let person_major = request.person_major.clone();
                let person_graduation_year = request.person_graduation_year.clone();

                let major = person_major.or(major);
                let graduation_year = person_graduation_year.or(graduation_year);

                result.push(OutgoingJoinRequestResponse {
                    id: request.id,
                    team_id: request.team_id,
                    team_name: team.name,
                    user_id: request.user_id,
                    user_name: person_name.or(request_user.name),
                    user_email: person_email.clone().unwrap_or(request_user.email.clone()),
                    user_picture: person_picture.or(request_user.picture),
                    major,
                    graduation_year,
                    message: request.message,
                    created_at: request.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                });
            }
        }
    }

    Ok(result)
}

/// Cancel outgoing join request
#[cfg_attr(feature = "server", utoipa::path(
    delete,
    path = "/api/hackathons/{slug}/outgoing-join-requests/{request_id}",
    params(
        ("slug" = String, Path, description = "Hackathon slug"),
        ("request_id" = i32, Path, description = "Join request ID")
    ),
    responses(
        (status = 200, description = "Join request cancelled successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Not your request"),
        (status = 404, description = "Request not found"),
        (status = 500, description = "Server error")
    ),
    tag = "teams"
))]
#[delete("/api/hackathons/:slug/outgoing-join-requests/:request_id", user: SyncedUser)]
pub async fn cancel_outgoing_join_request(
    slug: String,
    request_id: i32,
) -> Result<(), ServerFnError> {
    let ctx = RequestContext::extract(&user).await?;

    // Fetch the request
    let request = crate::entities::prelude::TeamJoinRequests::find_by_id(request_id)
        .one(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch request: {}", e)))?
        .ok_or_else(|| ServerFnError::new("Join request not found"))?;

    // Verify it's the user's request
    if request.user_id != ctx.user.id {
        return Err(ServerFnError::new("You can only cancel your own requests"));
    }

    // Delete the request
    crate::entities::prelude::TeamJoinRequests::delete_by_id(request_id)
        .exec(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to cancel request: {}", e)))?;

    // Suppress unused variable warning for slug (required by route path)
    let _ = slug;

    Ok(())
}

/// Accept a join request (owner only)
#[cfg_attr(feature = "server", utoipa::path(
    post,
    path = "/api/hackathons/{slug}/team/requests/{request_id}/accept",
    params(
        ("slug" = String, Path, description = "Hackathon slug"),
        ("request_id" = i32, Path, description = "Join request ID")
    ),
    responses(
        (status = 200, description = "Join request accepted successfully"),
        (status = 400, description = "Team is full or user already in team"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Only team owner can accept requests"),
        (status = 404, description = "Request not found"),
        (status = 500, description = "Server error")
    ),
    tag = "teams"
))]
#[post("/api/hackathons/:slug/team/requests/:request_id/accept", user: SyncedUser)]
pub async fn accept_join_request(slug: String, request_id: i32) -> Result<(), ServerFnError> {
    use crate::domain::teams::repository::TeamRepository;

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    let hackathon = ctx.hackathon()?;

    // Fetch join request
    let join_request = crate::entities::prelude::TeamJoinRequests::find_by_id(request_id)
        .one(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch join request: {}", e)))?
        .ok_or_else(|| ServerFnError::new("Join request not found"))?;

    // Get user's team_id
    let team_repo = TeamRepository::new(&ctx.state.db);
    let user_role = team_repo
        .find_user_role_or_error(
            ctx.user.id,
            hackathon.id,
            "User not registered for this hackathon",
        )
        .await?;

    let team_id = user_role
        .team_id
        .ok_or_else(|| ServerFnError::new("User is not in a team"))?;

    // Verify request is for user's team and user is the owner
    Permissions::require_team_request_ownership(&ctx, request_id, team_id).await?;

    // Check if team is full
    let member_count = team_repo.count_team_members(team_id, hackathon.id).await?;

    if member_count >= hackathon.max_team_size as usize {
        return Err(ServerFnError::new("Team is full"));
    }

    // Get the requesting user's role
    let requesting_user_role = team_repo
        .find_user_role_or_error(
            join_request.user_id,
            hackathon.id,
            "Requesting user not registered for this hackathon",
        )
        .await?;

    // Check if user already in a team
    if requesting_user_role.team_id.is_some() {
        return Err(ServerFnError::new("User is already in a team"));
    }

    // Add user to team
    let mut role: crate::entities::user_hackathon_roles::ActiveModel = requesting_user_role.into();
    role.team_id = Set(Some(team_id));

    role.update(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to add user to team: {}", e)))?;

    // Delete the join request
    let request_to_delete: crate::entities::team_join_requests::ActiveModel = join_request.into();
    request_to_delete
        .delete(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to delete join request: {}", e)))?;

    Ok(())
}

/// Reject a join request (owner only)
#[cfg_attr(feature = "server", utoipa::path(
    post,
    path = "/api/hackathons/{slug}/team/requests/{request_id}/reject",
    params(
        ("slug" = String, Path, description = "Hackathon slug"),
        ("request_id" = i32, Path, description = "Join request ID")
    ),
    responses(
        (status = 200, description = "Join request rejected successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Only team owner can reject requests"),
        (status = 404, description = "Request not found"),
        (status = 500, description = "Server error")
    ),
    tag = "teams"
))]
#[post("/api/hackathons/:slug/team/requests/:request_id/reject", user: SyncedUser)]
pub async fn reject_join_request(slug: String, request_id: i32) -> Result<(), ServerFnError> {
    use crate::domain::teams::repository::TeamRepository;

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    let hackathon = ctx.hackathon()?;

    // Fetch join request
    let join_request = crate::entities::prelude::TeamJoinRequests::find_by_id(request_id)
        .one(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch join request: {}", e)))?
        .ok_or_else(|| ServerFnError::new("Join request not found"))?;

    // Get user's team_id
    let team_repo = TeamRepository::new(&ctx.state.db);
    let user_role = team_repo
        .find_user_role_or_error(
            ctx.user.id,
            hackathon.id,
            "User not registered for this hackathon",
        )
        .await?;

    let team_id = user_role
        .team_id
        .ok_or_else(|| ServerFnError::new("User is not in a team"))?;

    // Verify request is for user's team and user is the owner
    Permissions::require_team_request_ownership(&ctx, request_id, team_id).await?;

    // Delete the join request
    let request_to_delete: crate::entities::team_join_requests::ActiveModel = join_request.into();
    request_to_delete
        .delete(&ctx.state.db)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to delete join request: {}", e)))?;

    Ok(())
}

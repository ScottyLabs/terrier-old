use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[cfg(feature = "server")]
use crate::core::auth::{
    context::RequestContext, middleware::SyncedUser, permissions::Permissions,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "server", derive(utoipa::ToSchema))]
pub struct HackathonPerson {
    pub user_id: i32,
    pub name: Option<String>,
    pub email: String,
    pub picture: Option<String>,
    pub role: String,
    pub team_id: Option<i32>,
    pub major: Option<String>,
    pub graduation_year: Option<String>,
}

/// Get all people associated with a hackathon, excluding applicants
#[cfg_attr(feature = "server", utoipa::path(
    get,
    path = "/api/hackathons/{slug}/people",
    params(
        ("slug" = String, Path, description = "Hackathon slug")
    ),
    responses(
        (status = 200, description = "People retrieved successfully", body = Vec<HackathonPerson>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Requires admin or organizer role"),
        (status = 404, description = "Hackathon not found"),
        (status = 500, description = "Server error")
    ),
    tag = "hackathons"
))]
#[get("/api/hackathons/:slug/people", user: SyncedUser)]
pub async fn get_hackathon_people(slug: String) -> Result<Vec<HackathonPerson>, ServerFnError> {
    use crate::domain::people::repository::UserRoleRepository;

    #[cfg(feature = "server")]
    use tracing::info;

    let ctx = RequestContext::extract(&user)
        .await?
        .with_hackathon(&slug)
        .await?;

    Permissions::require_admin_or_organizer(&ctx).await?;

    let hackathon = ctx.hackathon()?;

    // Fetch all user-hackathon roles for this hackathon excluding applicants
    let role_repo = UserRoleRepository::new(&ctx.state.db);
    let roles = role_repo
        .find_all_roles_for_hackathon_excluding_role(hackathon.id, "applicant")
        .await?;

    #[cfg(feature = "server")]
    {
        // Print raw DB query result for debugging
        info!(
            "DB ROLES QUERY RESULT for hackathon {}: {:#?}",
            hackathon.id, roles
        );
    }

    // Build results and enrich with application form data (major, graduation_year) when available
    use crate::domain::applications::repository::ApplicationRepository;

    let app_repo = ApplicationRepository::new(&ctx.state.db);

    let mut results: Vec<HackathonPerson> = Vec::new();
    for (role, user_opt) in roles.into_iter() {
        if let Some(user) = user_opt {
            // Default values
            let mut major: Option<String> = None;
            let mut graduation_year: Option<String> = None;

            // Try to fetch the user's application for this hackathon and extract fields
            if let Ok(Some(app)) = app_repo
                .find_by_user_and_hackathon(user.id, hackathon.id)
                .await
            {
                // form_data may be stored in a few shapes (object, array, or stringified JSON).
                // Normalize into a JsonValue and then try multiple key/shape patterns.
                let mut form_json = app.form_data.clone();

                // If it's a string containing JSON, try to parse it
                if form_json.is_string() {
                    if let Some(s) = form_json.as_str() {
                        if let Ok(parsed) = serde_json::from_str::<JsonValue>(s) {
                            form_json = parsed;
                        }
                    }
                }

                // Helper to try to extract a string value from a JsonValue
                let mut extract_string = |val: &JsonValue| -> Option<String> {
                    if val.is_string() {
                        val.as_str().map(|s| s.to_string())
                    } else if val.is_number() {
                        Some(val.to_string())
                    } else {
                        None
                    }
                };

                // If it's an object with keys
                if let Some(obj) = form_json.as_object() {
                    // Try common key variants
                    let candidates = [
                        "major",
                        "Major",
                        "graduation_year",
                        "graduationYear",
                        "graduation",
                    ];
                    if let Some(val) = obj.get("major") {
                        major = extract_string(val);
                    } else if let Some(val) = obj.get("Major") {
                        major = extract_string(val);
                    }

                    if let Some(val) = obj.get("graduation_year") {
                        graduation_year = extract_string(val);
                    } else if let Some(val) = obj.get("graduationYear") {
                        graduation_year = extract_string(val);
                    } else if let Some(val) = obj.get("graduation") {
                        graduation_year = extract_string(val);
                    } else {
                        // Also try the candidates generically
                        for key in &candidates {
                            if let Some(v) = obj.get(*key) {
                                if key.to_lowercase().contains("major") && major.is_none() {
                                    major = extract_string(v);
                                } else if key.to_lowercase().contains("graduation")
                                    && graduation_year.is_none()
                                {
                                    graduation_year = extract_string(v);
                                }
                            }
                        }
                    }
                } else if let Some(arr) = form_json.as_array() {
                    // Some form data is an array of {name, value} entries
                    for entry in arr.iter() {
                        if let Some(obj) = entry.as_object() {
                            // Support different name keys
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
            }

            results.push(HackathonPerson {
                user_id: user.id,
                name: user.name,
                email: user.email,
                picture: user.picture,
                role: role.role,
                team_id: role.team_id,
                major,
                graduation_year,
            });
        }
    }

    #[cfg(feature = "server")]
    {
        // Print mapped output that will be returned to the client
        info!(
            "MAPPED PEOPLE OUTPUT for hackathon {}: {:#?}",
            hackathon.id, results
        );
    }

    Ok(results)
}

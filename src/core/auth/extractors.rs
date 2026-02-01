use axum::{
    RequestPartsExt,
    extract::{FromRequestParts, Path},
    http::{StatusCode, request::Parts},
};
use axum_oidc::{EmptyAdditionalClaims, OidcClaims};
use sea_orm::{ColumnTrait, EntityTrait, JoinType, QueryFilter, QuerySelect, RelationTrait};

use crate::{
    AppState,
    auth::HackathonRole,
    entities::{hackathons, prelude::*, user_hackathon_roles, users},
};

impl FromRequestParts<AppState> for HackathonRole {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Extract hackathon slug from path
        let Path(slug) = parts
            .extract::<Path<String>>()
            .await
            .map_err(|_| StatusCode::BAD_REQUEST)?;

        let claims = OidcClaims::<EmptyAdditionalClaims>::from_request_parts(parts, state)
            .await
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        let email = claims
            .email()
            .map(|e| e.to_string())
            .ok_or(StatusCode::UNAUTHORIZED)?;

        // Global admins have admin role in all hackathons
        if state.config.admin_emails.contains(&email.to_lowercase()) {
            let hackathon = Hackathons::find()
                .filter(hackathons::Column::Slug.eq(&slug))
                .one(&state.db)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                .ok_or(StatusCode::NOT_FOUND)?;

            let user = Users::find()
                .filter(users::Column::OidcSub.eq(claims.subject().to_string()))
                .one(&state.db)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                .ok_or(StatusCode::UNAUTHORIZED)?;

            return Ok(HackathonRole {
                user_id: user.id,
                hackathon_id: hackathon.id,
                role: "admin".to_string(),
                slug,
                team_id: None,
            });
        }

        // Look up role in database
        let result = UserHackathonRoles::find()
            .join(
                JoinType::InnerJoin,
                user_hackathon_roles::Relation::Users.def(),
            )
            .join(
                JoinType::InnerJoin,
                user_hackathon_roles::Relation::Hackathons.def(),
            )
            .filter(users::Column::OidcSub.eq(claims.subject().to_string()))
            .filter(hackathons::Column::Slug.eq(&slug))
            .find_also_related(Hackathons)
            .one(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::FORBIDDEN)?;

        Ok(HackathonRole {
            user_id: result.0.user_id,
            hackathon_id: result.0.hackathon_id,
            role: result.0.role,
            slug,
            team_id: result.0.team_id,
        })
    }
}

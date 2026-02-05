#[cfg(feature = "server")]
use crate::entities::{prelude::*, user_hackathon_roles, users};
#[cfg(feature = "server")]
use dioxus::prelude::ServerFnError;
#[cfg(feature = "server")]
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

#[cfg(feature = "server")]
pub struct UserRoleRepository<'a> {
    db: &'a DatabaseConnection,
}

#[cfg(feature = "server")]
impl<'a> UserRoleRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Find a user's role for a specific hackathon
    pub async fn find_user_role(
        &self,
        user_id: i32,
        hackathon_id: i32,
    ) -> Result<Option<user_hackathon_roles::Model>, ServerFnError> {
        UserHackathonRoles::find()
            .filter(user_hackathon_roles::Column::UserId.eq(user_id))
            .filter(user_hackathon_roles::Column::HackathonId.eq(hackathon_id))
            .one(self.db)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to fetch user role: {}", e)))
    }

    /// Find a user's role for a specific hackathon, or return an error if not found
    pub async fn find_user_role_or_error(
        &self,
        user_id: i32,
        hackathon_id: i32,
        error_msg: &str,
    ) -> Result<user_hackathon_roles::Model, ServerFnError> {
        self.find_user_role(user_id, hackathon_id)
            .await?
            .ok_or_else(|| ServerFnError::new(error_msg))
    }

    /// Check if a user has any of the specified roles for a hackathon
    pub async fn has_role(
        &self,
        user_id: i32,
        hackathon_id: i32,
        allowed_roles: &[&str],
    ) -> Result<bool, ServerFnError> {
        let role = self.find_user_role(user_id, hackathon_id).await?;

        Ok(role
            .as_ref()
            .map(|r| allowed_roles.contains(&r.role.as_str()))
            .unwrap_or(false))
    }

    /// Get all users with roles for a hackathon, excluding specific roles
    pub async fn find_all_roles_for_hackathon(
        &self,
        hackathon_id: i32,
    ) -> Result<Vec<(user_hackathon_roles::Model, Option<users::Model>)>, ServerFnError> {
        UserHackathonRoles::find()
            .filter(user_hackathon_roles::Column::HackathonId.eq(hackathon_id))
            .find_also_related(Users)
            .all(self.db)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to fetch roles: {}", e)))
    }

    /// Get all users with roles for a hackathon, excluding specific roles
    pub async fn find_all_roles_for_hackathon_excluding_role(
        &self,
        hackathon_id: i32,
        excluded_role: &str,
    ) -> Result<Vec<(user_hackathon_roles::Model, Option<users::Model>)>, ServerFnError> {
        UserHackathonRoles::find()
            .filter(user_hackathon_roles::Column::HackathonId.eq(hackathon_id))
            .filter(user_hackathon_roles::Column::Role.ne(excluded_role))
            .find_also_related(Users)
            .all(self.db)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to fetch roles: {}", e)))
    }

    /// Check if user is admin or organizer for a hackathon
    pub async fn is_admin_or_organizer(
        &self,
        user_id: i32,
        hackathon_id: i32,
    ) -> Result<bool, ServerFnError> {
        self.has_role(user_id, hackathon_id, &["admin", "organizer"])
            .await
    }

    /// Check if user is admin for a hackathon
    pub async fn is_admin(&self, user_id: i32, hackathon_id: i32) -> Result<bool, ServerFnError> {
        self.has_role(user_id, hackathon_id, &["admin"]).await
    }

    /// Check if user is organizer for a hackathon
    pub async fn is_organizer(
        &self,
        user_id: i32,
        hackathon_id: i32,
    ) -> Result<bool, ServerFnError> {
        self.has_role(user_id, hackathon_id, &["organizer"]).await
    }
    /// Find people with pagination, search, and filtering
    pub async fn find_people_paginated(
        &self,
        hackathon_id: i32,
        search: Option<String>,
        roles: Option<Vec<String>>,
        excluded_roles: Option<Vec<String>>,
        page: u64,
        per_page: u64,
    ) -> Result<
        (
            Vec<(user_hackathon_roles::Model, Option<users::Model>)>,
            u64,
        ),
        ServerFnError,
    > {
        use sea_orm::{
            Condition, PaginatorTrait, QuerySelect,
            sea_query::{Expr, Func},
        };

        let mut query = UserHackathonRoles::find()
            .filter(user_hackathon_roles::Column::HackathonId.eq(hackathon_id))
            .find_also_related(Users);

        // Apply excluded roles filter
        if let Some(excluded) = excluded_roles {
            if !excluded.is_empty() {
                query = query.filter(user_hackathon_roles::Column::Role.is_not_in(excluded));
            }
        }

        // Apply roles filter
        if let Some(included) = roles {
            if !included.is_empty() {
                query = query.filter(user_hackathon_roles::Column::Role.is_in(included));
            }
        }

        // Apply search filter
        if let Some(search_term) = search {
            if !search_term.is_empty() {
                let terms: Vec<String> = search_term
                    .split(',')
                    .map(|s| s.trim().to_lowercase())
                    .filter(|s| !s.is_empty())
                    .collect();

                if !terms.is_empty() {
                    let mut search_condition = Condition::any();
                    for term in terms {
                        search_condition = search_condition
                            .add(
                                Expr::expr(Func::lower(Expr::col((
                                    users::Entity,
                                    users::Column::Name,
                                ))))
                                .like(format!("%{}%", term)),
                            )
                            .add(
                                Expr::expr(Func::lower(Expr::col((
                                    users::Entity,
                                    users::Column::Email,
                                ))))
                                .like(format!("%{}%", term)),
                            )
                            .add(
                                Expr::expr(Func::lower(Expr::col((
                                    user_hackathon_roles::Entity,
                                    user_hackathon_roles::Column::Role,
                                ))))
                                .like(format!("%{}%", term)),
                            );
                    }
                    query = query.filter(search_condition);
                }
            }
        }

        let paginator = query.paginate(self.db, per_page);
        let total = paginator
            .num_items()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to count people: {}", e)))?;

        let items = paginator
            .fetch_page(page)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to fetch people page: {}", e)))?;

        Ok((items, total))
    }
}

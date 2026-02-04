//! # Authentication Module
//!
//! *Written by Claude 4.5 Opus*
//!
//! This module defines the client-side authentication types and role-based access control.
//!
//! ## Design Decisions
//!
//! - **Role-based access control**: Each hackathon page has a predefined list of allowed roles
//!   (e.g., `DASHBOARD_ROLES`, `JUDGE_ROLES`). This centralizes permission logic and makes
//!   it easy to audit which roles can access which pages.
//!
//! - **Role hierarchy**: Roles are intentionally NOT hierarchical. An Admin doesn't automatically
//!   have all permissions of lower roles. Each page explicitly lists its allowed roles for clarity.
//!
//! - **Client/server split**: The `UserInfo` and role types are shared between client and server,
//!   while the `hooks` submodule provides client-side React-style hooks for authentication state.
//!
//! ## Role Types
//!
//! - `Admin` - Full hackathon management access
//! - `Organizer` - Event and schedule management
//! - `Judge` - Project judging and scoring
//! - `Sponsor` - Sponsor-specific views and checkin
//! - `Participant` - Active hackathon participant
//! - `Applicant` - User who has applied but not been accepted

pub mod hooks;

use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub is_admin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct HackathonRole {
    pub user_id: i32,
    pub hackathon_id: i32,
    pub role: String,
    pub slug: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub enum HackathonRoleType {
    Admin,
    Organizer,
    Judge,
    Sponsor,
    Participant,
    Applicant,
}

impl HackathonRoleType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "admin" => Some(Self::Admin),
            "organizer" => Some(Self::Organizer),
            "judge" => Some(Self::Judge),
            "sponsor" => Some(Self::Sponsor),
            "participant" => Some(Self::Participant),
            "applicant" => Some(Self::Applicant),
            _ => None,
        }
    }
}

impl HackathonRole {
    pub fn role_type(&self) -> Option<HackathonRoleType> {
        HackathonRoleType::from_str(&self.role)
    }
}

pub fn has_access(role: &HackathonRole, allowed: &[HackathonRoleType]) -> bool {
    if let Some(rt) = role.role_type() {
        allowed.contains(&rt)
    } else {
        false
    }
}

// Centralized role definitions for hackathon pages
pub const DASHBOARD_ROLES: &[HackathonRoleType] = &[
    HackathonRoleType::Participant,
    HackathonRoleType::Judge,
    HackathonRoleType::Sponsor,
    HackathonRoleType::Organizer,
    HackathonRoleType::Admin,
];

pub const APPLICANTS_ROLES: &[HackathonRoleType] = &[HackathonRoleType::Admin];

pub const PEOPLE_ROLES: &[HackathonRoleType] =
    &[HackathonRoleType::Admin, HackathonRoleType::Organizer];

pub const TEAM_ROLES: &[HackathonRoleType] = &[
    HackathonRoleType::Participant,
    HackathonRoleType::Applicant,
    HackathonRoleType::Admin,
];

pub const SCHEDULE_ROLES: &[HackathonRoleType] = &[
    HackathonRoleType::Participant,
    HackathonRoleType::Judge,
    HackathonRoleType::Sponsor,
    HackathonRoleType::Organizer,
    HackathonRoleType::Admin,
];

pub const SUBMISSION_ROLES: &[HackathonRoleType] = &[HackathonRoleType::Participant];

pub const CHECKIN_ROLES: &[HackathonRoleType] = &[
    HackathonRoleType::Participant,
    HackathonRoleType::Sponsor,
    HackathonRoleType::Organizer,
    HackathonRoleType::Admin,
];

pub const SETTINGS_ROLES: &[HackathonRoleType] = &[HackathonRoleType::Admin];

pub const APPLY_ROLES: &[HackathonRoleType] = &[
    HackathonRoleType::Applicant,
    HackathonRoleType::Participant,
    HackathonRoleType::Organizer,
    HackathonRoleType::Admin,
];

pub const PRIZE_TRACKS_ROLES: &[HackathonRoleType] =
    &[HackathonRoleType::Admin, HackathonRoleType::Organizer];

pub const JUDGE_ROLES: &[HackathonRoleType] = &[
    HackathonRoleType::Judge,
    HackathonRoleType::Admin,
    HackathonRoleType::Organizer,
];

pub const JUDGING_ADMIN_ROLES: &[HackathonRoleType] = &[HackathonRoleType::Admin];

pub const RESULTS_ROLES: &[HackathonRoleType] = &[
    HackathonRoleType::Judge,
    HackathonRoleType::Admin,
    HackathonRoleType::Organizer,
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct LoginQuery {
    pub redirect_uri: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_judging_permissions() {
        // Mock Roles
        let admin_role = HackathonRole {
            user_id: 1,
            hackathon_id: 1,
            role: "admin".to_string(),
            slug: "test".to_string(),
        };

        let organizer_role = HackathonRole {
            user_id: 2,
            hackathon_id: 1,
            role: "organizer".to_string(),
            slug: "test".to_string(),
        };

        let judge_role = HackathonRole {
            user_id: 3,
            hackathon_id: 1,
            role: "judge".to_string(),
            slug: "test".to_string(),
        };

        // Judging Admin Page Permissions
        assert!(
            has_access(&admin_role, JUDGING_ADMIN_ROLES),
            "Admin should access Judging Admin"
        );
        assert!(
            !has_access(&organizer_role, JUDGING_ADMIN_ROLES),
            "Organizer should NOT access Judging Admin"
        );
        assert!(
            !has_access(&judge_role, JUDGING_ADMIN_ROLES),
            "Judge should NOT access Judging Admin"
        );

        // Judge Page Permissions
        assert!(
            has_access(&admin_role, JUDGE_ROLES),
            "Admin should access Judge Page"
        );
        assert!(
            has_access(&organizer_role, JUDGE_ROLES),
            "Organizer should access Judge Page"
        );
        assert!(
            has_access(&judge_role, JUDGE_ROLES),
            "Judge should access Judge Page"
        );
    }
}

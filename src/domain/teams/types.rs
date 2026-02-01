use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "server", derive(utoipa::ToSchema))]
pub struct TeamData {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub member_count: usize,
    pub max_size: i32,
    pub is_owner: bool,
    pub is_member: bool,
    pub members: Vec<TeamMemberData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "server", derive(utoipa::ToSchema))]
pub struct TeamMemberData {
    pub user_id: i32,
    pub name: Option<String>,
    pub email: String,
    pub picture: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "server", derive(utoipa::ToSchema))]
pub struct TeamListItem {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub member_count: usize,
    pub max_size: i32,
    pub is_full: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "server", derive(utoipa::ToSchema))]
pub struct UpdateTeamRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "server", derive(utoipa::ToSchema))]
pub struct CreateTeamRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "server", derive(utoipa::ToSchema))]
pub struct JoinTeamRequest {
    pub team_id: i32,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "server", derive(utoipa::ToSchema))]
pub struct JoinRequestResponse {
    pub id: i32,
    pub team_id: i32,
    pub user_id: i32,
    pub user_name: Option<String>,
    pub user_email: String,
    pub user_picture: Option<String>,
    pub major: Option<String>,
    pub graduation_year: Option<String>,
    pub message: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "server", derive(utoipa::ToSchema))]
pub struct UserWithoutTeam {
    pub id: i32,
    pub name: Option<String>,
    pub email: String,
    pub picture: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "server", derive(utoipa::ToSchema))]
pub struct SendInvitationRequest {
    pub user_id: i32,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "server", derive(utoipa::ToSchema))]
pub struct InvitationResponse {
    pub id: i32,
    pub team_id: i32,
    pub team_name: String,
    pub user_id: i32,
    pub user_name: Option<String>,
    pub user_email: String,
    pub user_picture: Option<String>,
    pub major: Option<String>,
    pub graduation_year: Option<String>,
    pub message: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "server", derive(utoipa::ToSchema))]
pub struct OutgoingJoinRequestResponse {
    pub id: i32,
    pub team_id: i32,
    pub team_name: String,
    pub user_id: i32,
    pub user_name: Option<String>,
    pub user_email: String,
    pub user_picture: Option<String>,
    pub major: Option<String>,
    pub graduation_year: Option<String>,
    pub message: Option<String>,
    pub created_at: String,
}

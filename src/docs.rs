use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        // Hackathons
        crate::domain::hackathons::handlers::create::create_hackathon,
        crate::domain::hackathons::handlers::create::upload_background,
        crate::domain::hackathons::handlers::create::upload_banner,
        crate::domain::hackathons::handlers::query::get_hackathons,
        crate::domain::hackathons::handlers::query::get_hackathon_by_slug,
        crate::domain::hackathons::handlers::update::update_hackathon,
        crate::domain::hackathons::handlers::update::delete_banner,
        // Applications
        crate::domain::applications::handlers::submission::update_application,
        crate::domain::applications::handlers::submission::submit_application,
        crate::domain::applications::handlers::submission::unsubmit_application,
        crate::domain::applications::handlers::submission::get_application,
        crate::domain::applications::handlers::query::get_all_applications,
        crate::domain::applications::handlers::review::accept_applications,
        crate::domain::applications::handlers::review::reject_applications,
        crate::domain::applications::handlers::attendance::decline_attendance,
        crate::domain::applications::handlers::attendance::confirm_attendance,
        crate::domain::applications::handlers::attendance::undo_confirmation,
        crate::domain::hackathons::handlers::files::upload_application_file,
        crate::domain::hackathons::handlers::files::delete_application_file,
        // People
        crate::domain::people::handlers::query::get_hackathon_people,
        crate::domain::people::handlers::roles::remove_hackathon_person,
        // Form Config
        crate::domain::hackathons::handlers::settings::set_form_config,
        crate::domain::hackathons::handlers::settings::get_form_config,
        // Teams
        crate::domain::teams::handlers::crud::get_my_team,
        crate::domain::teams::handlers::crud::get_all_teams,
        crate::domain::teams::handlers::crud::update_team,
        crate::domain::teams::handlers::crud::create_team,
        crate::domain::teams::handlers::crud::delete_team,
        crate::domain::teams::handlers::join_requests::request_join_team,
        crate::domain::teams::handlers::join_requests::get_join_requests,
        crate::domain::teams::handlers::join_requests::accept_join_request,
        crate::domain::teams::handlers::join_requests::reject_join_request,
        crate::domain::teams::handlers::join_requests::get_outgoing_join_requests,
        crate::domain::teams::handlers::join_requests::cancel_outgoing_join_request,
        crate::domain::teams::handlers::invitations::send_invitation,
        crate::domain::teams::handlers::invitations::get_my_invitations,
        crate::domain::teams::handlers::invitations::accept_invitation,
        crate::domain::teams::handlers::invitations::decline_invitation,
        crate::domain::teams::handlers::membership::leave_team,
        crate::domain::teams::handlers::membership::kick_member,
        crate::domain::teams::handlers::membership::transfer_ownership,
        crate::domain::teams::handlers::membership::leave_team_force,
        crate::domain::teams::handlers::query::get_team_details,
        crate::domain::teams::handlers::query::get_users_without_team,
        // Auth
        crate::domain::auth::handlers::get_current_user,
        // Judging
        crate::domain::judging::handlers::close_submissions,
        crate::domain::judging::handlers::start_judging,
        crate::domain::judging::handlers::stop_judging,
        crate::domain::judging::handlers::reset_judging,
        crate::domain::judging::handlers::get_judging_status,
        crate::domain::judging::handlers::request_assignment,
        crate::domain::judging::handlers::complete_visit,
        crate::domain::judging::handlers::get_current_assignment,
        crate::domain::judging::handlers::submit_comparison,
        crate::domain::judging::handlers::get_features,
        crate::domain::judging::handlers::create_feature,
        crate::domain::judging::handlers::update_feature,
        crate::domain::judging::handlers::delete_feature,
        crate::domain::judging::handlers::reopen_submissions,
        crate::domain::judging::handlers::get_unified_state,
        crate::domain::judging::handlers::request_next_project,
        crate::domain::judging::handlers::submit_comparisons,
        crate::domain::judging::handlers::get_prizes_with_judges,
        crate::domain::judging::handlers::assign_prize_judges,
        crate::domain::judging::handlers::unassign_prize_judge,
        crate::domain::judging::handlers::assign_all_judges,
        crate::domain::judging::handlers::toggle_prize_assignment,
        crate::domain::judging::handlers::get_prize_track_results,
        crate::domain::judging::handlers::get_my_visit_notes,
        crate::domain::judging::handlers::generate_ai_summary,
    ),
    components(
        schemas(
            // Hackathons
            crate::domain::hackathons::types::HackathonInfo,
            crate::domain::hackathons::handlers::create::CreateHackathonRequest,
            crate::domain::hackathons::handlers::update::UpdateHackathonRequest,
            // Applications
            crate::domain::applications::types::ApplicationData,
            crate::domain::applications::types::ApplicationWithUser,
            crate::domain::hackathons::handlers::files::FileUploadResponse,
            // People
            crate::domain::people::HackathonPerson,
            // Form Config
            crate::domain::applications::types::FormSchema,
            crate::domain::applications::types::FormField,
            crate::domain::applications::types::FieldType,
            crate::domain::applications::types::TextValidation,
            crate::domain::applications::types::NumberValidation,
            crate::domain::applications::types::FileValidation,
            crate::domain::applications::types::SelectOption,
            // Teams
            crate::domain::teams::types::TeamData,
            crate::domain::teams::types::TeamMemberData,
            crate::domain::teams::types::TeamListItem,
            crate::domain::teams::types::UpdateTeamRequest,
            crate::domain::teams::types::CreateTeamRequest,
            crate::domain::teams::types::JoinTeamRequest,
            crate::domain::teams::types::JoinRequestResponse,
            crate::domain::teams::types::OutgoingJoinRequestResponse,
            crate::domain::teams::types::UserWithoutTeam,
            crate::domain::teams::types::SendInvitationRequest,
            crate::domain::teams::types::InvitationResponse,
            // Auth
            crate::auth::UserInfo,
        )
    ),
    tags(
        (name = "hackathons", description = "Hackathon management endpoints"),
        (name = "applications", description = "Application management endpoints"),
        (name = "people", description = "People management endpoints"),
        (name = "teams", description = "Team management endpoints"),
        (name = "auth", description = "Authentication endpoints")
    ),
    info(
        title = "Terrier API",
        version = "0.1.0",
        description = "API for Terrier"
    )
)]
pub struct ApiDoc;

pub use sea_orm_migration::prelude::*;

mod m20251130_100554_create_user_hackathon_tables;
mod m20251201_023320_create_teams_tables;
mod m20251201_044615_add_hackathon_banner_url;
mod m20251201_165412_add_hackathon_form_config;
mod m20251201_165433_create_applications_table;
mod m20251203_041138_create_team_join_requests;
mod m20251203_145251_create_team_invitations;
mod m20251203_220027_add_hackathon_background_url;
mod m20251204_045720_add_team_owner_id;
mod m20251229_003029_add_schedule_tables;
mod m20251229_043000_add_event_organizers;
mod m20251229_051500_add_event_location;
mod m20251229_105000_add_event_visibility;
mod m20251230_172700_add_event_points;
mod m20260103_add_event_checkins;
mod m20260105_230608_add_project_submissions_and_prizes;
mod m20260109_add_app_icon_and_theme;
mod m20260118_add_judging_tables;
mod m20260119_add_judge_feature_assignment;
mod m20260127_000000_add_table_number_to_submission;
mod m20260127_000001_add_prize_required_events;
mod m20260131_120000_create_messages_and_groups;
mod m20260131_125000_add_messages_content;
mod m20260131_add_judge_prize_track;
mod m20260131_add_judge_walk_type;
mod m20260201_074127_add_message_title;
mod m20260201_add_person_snapshot_to_team_requests_and_invitations;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20251130_100554_create_user_hackathon_tables::Migration),
            Box::new(m20251201_023320_create_teams_tables::Migration),
            Box::new(m20251201_044615_add_hackathon_banner_url::Migration),
            Box::new(m20251201_165412_add_hackathon_form_config::Migration),
            Box::new(m20251201_165433_create_applications_table::Migration),
            Box::new(m20251203_041138_create_team_join_requests::Migration),
            Box::new(m20251203_145251_create_team_invitations::Migration),
            Box::new(m20251203_220027_add_hackathon_background_url::Migration),
            Box::new(m20251204_045720_add_team_owner_id::Migration),
            Box::new(m20251229_003029_add_schedule_tables::Migration),
            Box::new(m20251229_043000_add_event_organizers::Migration),
            Box::new(m20251229_051500_add_event_location::Migration),
            Box::new(m20251229_105000_add_event_visibility::Migration),
            Box::new(m20251230_172700_add_event_points::Migration),
            Box::new(m20260103_add_event_checkins::Migration),
            Box::new(m20260105_230608_add_project_submissions_and_prizes::Migration),
            Box::new(m20260109_add_app_icon_and_theme::Migration),
            Box::new(m20260118_add_judging_tables::Migration),
            Box::new(m20260119_add_judge_feature_assignment::Migration),
            Box::new(m20260127_000000_add_table_number_to_submission::Migration),
            Box::new(m20260127_000001_add_prize_required_events::Migration),
            Box::new(m20260131_add_judge_prize_track::Migration),
            Box::new(m20260131_add_judge_walk_type::Migration),
            Box::new(m20260131_120000_create_messages_and_groups::Migration),
            Box::new(m20260131_125000_add_messages_content::Migration),
            Box::new(m20260201_add_person_snapshot_to_team_requests_and_invitations::Migration),
            Box::new(m20260201_074127_add_message_title::Migration),
        ]
    }
}

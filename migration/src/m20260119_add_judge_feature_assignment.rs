use crate::m20251130_100554_create_user_hackathon_tables::Users;
use crate::m20260105_230608_add_project_submissions_and_prizes::Submission;
use crate::m20260118_add_judging_tables::Feature;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create JudgeFeatureAssignment table
        // Tracks which judges are assigned to which features for judging
        manager
            .create_table(
                Table::create()
                    .table(JudgeFeatureAssignment::Table)
                    .if_not_exists()
                    .col(pk_auto(JudgeFeatureAssignment::Id))
                    .col(integer(JudgeFeatureAssignment::JudgeId))
                    .col(integer(JudgeFeatureAssignment::FeatureId))
                    .col(integer_null(
                        JudgeFeatureAssignment::CurrentBestSubmissionId,
                    ))
                    .col(text_null(JudgeFeatureAssignment::Notes))
                    .col(
                        ColumnDef::new(JudgeFeatureAssignment::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                JudgeFeatureAssignment::Table,
                                JudgeFeatureAssignment::JudgeId,
                            )
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                JudgeFeatureAssignment::Table,
                                JudgeFeatureAssignment::FeatureId,
                            )
                            .to(Feature::Table, Feature::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                JudgeFeatureAssignment::Table,
                                JudgeFeatureAssignment::CurrentBestSubmissionId,
                            )
                            .to(Submission::Table, Submission::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // Add unique constraint on judge_id + feature_id
        manager
            .create_index(
                Index::create()
                    .name("idx_judge_feature_unique")
                    .table(JudgeFeatureAssignment::Table)
                    .col(JudgeFeatureAssignment::JudgeId)
                    .col(JudgeFeatureAssignment::FeatureId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Add index on project_visit for faster concurrent lock checks
        manager
            .create_index(
                Index::create()
                    .name("idx_project_visit_active")
                    .table(ProjectVisit::Table)
                    .col(ProjectVisit::SubmissionId)
                    .col(ProjectVisit::IsActive)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop the active index
        manager
            .drop_index(
                Index::drop()
                    .name("idx_project_visit_active")
                    .table(ProjectVisit::Table)
                    .to_owned(),
            )
            .await?;

        // Drop the table
        manager
            .drop_table(
                Table::drop()
                    .table(JudgeFeatureAssignment::Table)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum JudgeFeatureAssignment {
    Table,
    Id,
    JudgeId,
    FeatureId,
    CurrentBestSubmissionId,
    Notes,
    CreatedAt,
}

// Reference to existing table for index
#[derive(DeriveIden)]
enum ProjectVisit {
    Table,
    SubmissionId,
    IsActive,
}

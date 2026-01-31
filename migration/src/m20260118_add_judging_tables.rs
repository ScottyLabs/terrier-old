use crate::m20251130_100554_create_user_hackathon_tables::{Hackathons, Users};
use crate::m20260105_230608_add_project_submissions_and_prizes::{Prize, Submission};
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add hackathon state fields for judging workflow
        manager
            .alter_table(
                Table::alter()
                    .table(Hackathons::Table)
                    .add_column(
                        ColumnDef::new(HackathonJudgingFields::SubmissionsClosed)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .add_column(
                        ColumnDef::new(HackathonJudgingFields::JudgingStarted)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .add_column(
                        ColumnDef::new(HackathonJudgingFields::JudgeSessionTimeoutMinutes)
                            .integer()
                            .not_null()
                            .default(10),
                    )
                    .to_owned(),
            )
            .await?;

        // Add hackathon_id to prize table
        manager
            .alter_table(
                Table::alter()
                    .table(Prize::Table)
                    .add_column(ColumnDef::new(PrizeFields::HackathonId).integer())
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_prize_hackathon")
                    .from(Prize::Table, PrizeFields::HackathonId)
                    .to(Hackathons::Table, Hackathons::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // Create Feature table (judging criteria per hackathon)
        manager
            .create_table(
                Table::create()
                    .table(Feature::Table)
                    .if_not_exists()
                    .col(pk_auto(Feature::Id))
                    .col(integer(Feature::HackathonId))
                    .col(string(Feature::Name))
                    .col(text_null(Feature::Description))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Feature::Table, Feature::HackathonId)
                            .to(Hackathons::Table, Hackathons::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create PrizeFeatureWeight table (weight of features per prize track)
        manager
            .create_table(
                Table::create()
                    .table(PrizeFeatureWeight::Table)
                    .if_not_exists()
                    .col(pk_auto(PrizeFeatureWeight::Id))
                    .col(integer(PrizeFeatureWeight::PrizeId))
                    .col(integer(PrizeFeatureWeight::FeatureId))
                    .col(float(PrizeFeatureWeight::Weight).default(1.0))
                    .foreign_key(
                        ForeignKey::create()
                            .from(PrizeFeatureWeight::Table, PrizeFeatureWeight::PrizeId)
                            .to(Prize::Table, Prize::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(PrizeFeatureWeight::Table, PrizeFeatureWeight::FeatureId)
                            .to(Feature::Table, Feature::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Add unique constraint on prize_id + feature_id
        manager
            .create_index(
                Index::create()
                    .name("idx_prize_feature_unique")
                    .table(PrizeFeatureWeight::Table)
                    .col(PrizeFeatureWeight::PrizeId)
                    .col(PrizeFeatureWeight::FeatureId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Create ProjectFeatureScore table (computed scores)
        manager
            .create_table(
                Table::create()
                    .table(ProjectFeatureScore::Table)
                    .if_not_exists()
                    .col(pk_auto(ProjectFeatureScore::Id))
                    .col(integer(ProjectFeatureScore::SubmissionId))
                    .col(integer(ProjectFeatureScore::FeatureId))
                    .col(float_null(ProjectFeatureScore::Score))
                    .col(float_null(ProjectFeatureScore::Variance))
                    .col(
                        ColumnDef::new(ProjectFeatureScore::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                ProjectFeatureScore::Table,
                                ProjectFeatureScore::SubmissionId,
                            )
                            .to(Submission::Table, Submission::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(ProjectFeatureScore::Table, ProjectFeatureScore::FeatureId)
                            .to(Feature::Table, Feature::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Add unique constraint on submission_id + feature_id
        manager
            .create_index(
                Index::create()
                    .name("idx_submission_feature_unique")
                    .table(ProjectFeatureScore::Table)
                    .col(ProjectFeatureScore::SubmissionId)
                    .col(ProjectFeatureScore::FeatureId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Create ProjectVisit table (judge visits with notes)
        manager
            .create_table(
                Table::create()
                    .table(ProjectVisit::Table)
                    .if_not_exists()
                    .col(pk_auto(ProjectVisit::Id))
                    .col(integer(ProjectVisit::SubmissionId))
                    .col(integer(ProjectVisit::JudgeId))
                    .col(integer(ProjectVisit::HackathonId))
                    .col(text_null(ProjectVisit::Notes))
                    .col(
                        ColumnDef::new(ProjectVisit::StartTime)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(timestamp_null(ProjectVisit::CompletionTime))
                    .col(boolean(ProjectVisit::IsActive).default(true))
                    .foreign_key(
                        ForeignKey::create()
                            .from(ProjectVisit::Table, ProjectVisit::SubmissionId)
                            .to(Submission::Table, Submission::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(ProjectVisit::Table, ProjectVisit::JudgeId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(ProjectVisit::Table, ProjectVisit::HackathonId)
                            .to(Hackathons::Table, Hackathons::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Add unique constraint on submission_id + judge_id
        manager
            .create_index(
                Index::create()
                    .name("idx_visit_submission_judge_unique")
                    .table(ProjectVisit::Table)
                    .col(ProjectVisit::SubmissionId)
                    .col(ProjectVisit::JudgeId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Create SubmissionAiSummary table
        manager
            .create_table(
                Table::create()
                    .table(SubmissionAiSummary::Table)
                    .if_not_exists()
                    .col(pk_auto(SubmissionAiSummary::Id))
                    .col(integer(SubmissionAiSummary::SubmissionId).unique_key())
                    .col(text_null(SubmissionAiSummary::Summary))
                    .col(
                        ColumnDef::new(SubmissionAiSummary::GeneratedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                SubmissionAiSummary::Table,
                                SubmissionAiSummary::SubmissionId,
                            )
                            .to(Submission::Table, Submission::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create PairwiseComparison table
        manager
            .create_table(
                Table::create()
                    .table(PairwiseComparison::Table)
                    .if_not_exists()
                    .col(pk_auto(PairwiseComparison::Id))
                    .col(integer(PairwiseComparison::JudgeId))
                    .col(integer(PairwiseComparison::FeatureId))
                    .col(integer(PairwiseComparison::SubmissionAId))
                    .col(integer(PairwiseComparison::SubmissionBId))
                    .col(integer_null(PairwiseComparison::WinnerId)) // NULL = tie
                    .col(
                        ColumnDef::new(PairwiseComparison::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(PairwiseComparison::Table, PairwiseComparison::JudgeId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(PairwiseComparison::Table, PairwiseComparison::FeatureId)
                            .to(Feature::Table, Feature::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(PairwiseComparison::Table, PairwiseComparison::SubmissionAId)
                            .to(Submission::Table, Submission::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(PairwiseComparison::Table, PairwiseComparison::SubmissionBId)
                            .to(Submission::Table, Submission::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop tables in reverse order of dependencies
        manager
            .drop_table(Table::drop().table(PairwiseComparison::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(SubmissionAiSummary::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(ProjectVisit::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(ProjectFeatureScore::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(PrizeFeatureWeight::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Feature::Table).to_owned())
            .await?;

        // Drop foreign key before column
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_prize_hackathon")
                    .table(Prize::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Prize::Table)
                    .drop_column(PrizeFields::HackathonId)
                    .to_owned(),
            )
            .await?;

        // Remove hackathon judging fields
        manager
            .alter_table(
                Table::alter()
                    .table(Hackathons::Table)
                    .drop_column(HackathonJudgingFields::SubmissionsClosed)
                    .drop_column(HackathonJudgingFields::JudgingStarted)
                    .drop_column(HackathonJudgingFields::JudgeSessionTimeoutMinutes)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

// Hackathon judging-related fields
#[derive(DeriveIden)]
enum HackathonJudgingFields {
    SubmissionsClosed,
    JudgingStarted,
    JudgeSessionTimeoutMinutes,
}

// Prize hackathon link
#[derive(DeriveIden)]
enum PrizeFields {
    HackathonId,
}

#[derive(DeriveIden)]
pub enum Feature {
    Table,
    Id,
    HackathonId,
    Name,
    Description,
}

#[derive(DeriveIden)]
enum PrizeFeatureWeight {
    Table,
    Id,
    PrizeId,
    FeatureId,
    Weight,
}

#[derive(DeriveIden)]
enum ProjectFeatureScore {
    Table,
    Id,
    SubmissionId,
    FeatureId,
    Score,
    Variance,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub enum ProjectVisit {
    Table,
    Id,
    SubmissionId,
    JudgeId,
    HackathonId,
    Notes,
    StartTime,
    CompletionTime,
    IsActive,
}

#[derive(DeriveIden)]
enum SubmissionAiSummary {
    Table,
    Id,
    SubmissionId,
    Summary,
    GeneratedAt,
}

#[derive(DeriveIden)]
pub enum PairwiseComparison {
    Table,
    Id,
    JudgeId,
    FeatureId,
    SubmissionAId,
    SubmissionBId,
    WinnerId,
    CreatedAt,
}

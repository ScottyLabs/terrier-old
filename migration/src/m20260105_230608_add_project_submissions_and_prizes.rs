use crate::m20251201_023320_create_teams_tables::Teams;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let a = manager
            .create_table(
                Table::create()
                    .table(Submission::Table)
                    .if_not_exists()
                    .col(pk_auto(Submission::Id))
                    .col(integer(Submission::TeamId))
                    .col(json_binary(Submission::SubmissionData))
                    .col(
                        ColumnDef::new(Submission::SubmittedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Submission::Table, Submission::TeamId)
                            .to(Teams::Table, Teams::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await;
        let b = manager
            .create_table(
                Table::create()
                    .table(Prize::Table)
                    .if_not_exists()
                    .col(pk_auto(Prize::Id))
                    .col(string(Prize::Name))
                    .col(string_null(Prize::Description))
                    .col(string_null(Prize::ImageUrl))
                    .col(string_null(Prize::Category))
                    .col(string(Prize::Value))
                    .to_owned(),
            )
            .await;
        let c = manager
            .create_table(
                Table::create()
                    .table(PrizeTrackEntry::Table)
                    .if_not_exists()
                    .col(pk_auto(PrizeTrackEntry::Id))
                    .col(integer(PrizeTrackEntry::SubmissionId))
                    .col(integer(PrizeTrackEntry::PrizeId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(PrizeTrackEntry::Table, PrizeTrackEntry::SubmissionId)
                            .to(Submission::Table, Submission::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(PrizeTrackEntry::Table, PrizeTrackEntry::PrizeId)
                            .to(Prize::Table, Prize::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await;

        // alter hackathon table to have submission_form json
        let d = manager
            .alter_table(
                Table::alter()
                    .table(Hackathons::Table)
                    .add_column(ColumnDef::new(Hackathons::SubmissionForm).json_binary())
                    .to_owned(),
            )
            .await;

        a.and(b).and(c).and(d)
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop PrizeTrackEntry first (has FKs to Submission and Prize)
        let c = manager
            .drop_table(Table::drop().table(PrizeTrackEntry::Table).to_owned())
            .await;

        let a = manager
            .drop_table(Table::drop().table(Submission::Table).to_owned())
            .await;

        let b = manager
            .drop_table(Table::drop().table(Prize::Table).to_owned())
            .await;

        let d = manager
            .alter_table(
                Table::alter()
                    .table(Hackathons::Table)
                    .drop_column(Hackathons::SubmissionForm)
                    .to_owned(),
            )
            .await;

        a.and(b).and(c).and(d)
    }
}

#[derive(DeriveIden)]
pub enum Submission {
    Id,
    Table,
    TeamId,
    SubmissionData,
    SubmittedAt,
}

#[derive(DeriveIden)]
enum PrizeTrackEntry {
    Id,
    Table,
    SubmissionId, // FK
    PrizeId,      // FK
}

#[derive(DeriveIden)]
pub enum Prize {
    Id,
    Table,
    Name,
    Description,
    ImageUrl,
    Category,
    Value,
}

#[derive(DeriveIden)]
enum Hackathons {
    Table,
    SubmissionForm,
}

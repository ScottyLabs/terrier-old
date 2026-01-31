use crate::m20251130_100554_create_user_hackathon_tables::Users;
use crate::m20260105_230608_add_project_submissions_and_prizes::Prize;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create JudgePrizeTrack table
        // Tracks which judges are assigned to which prize tracks
        // If a prize track has NO entries, ALL judges can judge it (default)
        manager
            .create_table(
                Table::create()
                    .table(JudgePrizeTrack::Table)
                    .if_not_exists()
                    .col(pk_auto(JudgePrizeTrack::Id))
                    .col(integer(JudgePrizeTrack::JudgeId))
                    .col(integer(JudgePrizeTrack::PrizeId))
                    .col(
                        ColumnDef::new(JudgePrizeTrack::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(JudgePrizeTrack::Table, JudgePrizeTrack::JudgeId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(JudgePrizeTrack::Table, JudgePrizeTrack::PrizeId)
                            .to(Prize::Table, Prize::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Add unique constraint on judge_id + prize_id
        manager
            .create_index(
                Index::create()
                    .name("idx_judge_prize_track_unique")
                    .table(JudgePrizeTrack::Table)
                    .col(JudgePrizeTrack::JudgeId)
                    .col(JudgePrizeTrack::PrizeId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(JudgePrizeTrack::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum JudgePrizeTrack {
    Table,
    Id,
    JudgeId,
    PrizeId,
    CreatedAt,
}

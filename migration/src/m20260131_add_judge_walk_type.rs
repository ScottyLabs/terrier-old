use crate::m20251130_100554_create_user_hackathon_tables::{Hackathons, Users};
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create JudgeWalkType table
        // Stores the walk type preference for each judge per hackathon
        manager
            .create_table(
                Table::create()
                    .table(JudgeWalkType::Table)
                    .if_not_exists()
                    .col(pk_auto(JudgeWalkType::Id))
                    .col(integer(JudgeWalkType::JudgeId))
                    .col(integer(JudgeWalkType::HackathonId))
                    .col(
                        ColumnDef::new(JudgeWalkType::WalkType)
                            .string_len(20)
                            .not_null()
                            .default("Default"),
                    )
                    .col(
                        ColumnDef::new(JudgeWalkType::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(JudgeWalkType::Table, JudgeWalkType::JudgeId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(JudgeWalkType::Table, JudgeWalkType::HackathonId)
                            .to(Hackathons::Table, Hackathons::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Add unique constraint on judge_id + hackathon_id
        manager
            .create_index(
                Index::create()
                    .name("idx_judge_walk_type_unique")
                    .table(JudgeWalkType::Table)
                    .col(JudgeWalkType::JudgeId)
                    .col(JudgeWalkType::HackathonId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(JudgeWalkType::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum JudgeWalkType {
    Table,
    Id,
    JudgeId,
    HackathonId,
    WalkType,
    CreatedAt,
}

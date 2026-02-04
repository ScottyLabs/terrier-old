use sea_orm_migration::{prelude::*, schema::*};

use crate::m20251229_003029_add_schedule_tables::Events;
use crate::m20260105_230608_add_project_submissions_and_prizes::Prize;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PrizeRequiredEvents::Table)
                    .if_not_exists()
                    .col(integer(PrizeRequiredEvents::PrizeId))
                    .col(integer(PrizeRequiredEvents::EventId))
                    .primary_key(
                        Index::create()
                            .col(PrizeRequiredEvents::PrizeId)
                            .col(PrizeRequiredEvents::EventId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(PrizeRequiredEvents::Table, PrizeRequiredEvents::PrizeId)
                            .to(Prize::Table, Prize::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(PrizeRequiredEvents::Table, PrizeRequiredEvents::EventId)
                            .to(Events::Table, Events::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PrizeRequiredEvents::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum PrizeRequiredEvents {
    Table,
    PrizeId,
    EventId,
}

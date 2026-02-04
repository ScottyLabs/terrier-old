use crate::m20251130_100554_create_user_hackathon_tables::Hackathons;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add proximity_routing_enabled column (default false)
        manager
            .alter_table(
                Table::alter()
                    .table(Hackathons::Table)
                    .add_column(
                        ColumnDef::new(Alias::new("proximity_routing_enabled"))
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await?;

        // Add room_width column (default 10)
        manager
            .alter_table(
                Table::alter()
                    .table(Hackathons::Table)
                    .add_column(
                        ColumnDef::new(Alias::new("room_width"))
                            .integer()
                            .not_null()
                            .default(10),
                    )
                    .to_owned(),
            )
            .await?;

        // Add judging_timer_seconds column (default 600 = 10 minutes)
        manager
            .alter_table(
                Table::alter()
                    .table(Hackathons::Table)
                    .add_column(
                        ColumnDef::new(Alias::new("judging_timer_seconds"))
                            .integer()
                            .not_null()
                            .default(600),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Hackathons::Table)
                    .drop_column(Alias::new("proximity_routing_enabled"))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Hackathons::Table)
                    .drop_column(Alias::new("room_width"))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Hackathons::Table)
                    .drop_column(Alias::new("judging_timer_seconds"))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

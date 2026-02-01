use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // No-op migration: the `content` column already exists in some databases.
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // No-op
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum Messages {
    Table,
    Id,
    SenderUserId,
    MessageGroupId,
    Content,
    CreatedAt,
    UpdatedAt,
}

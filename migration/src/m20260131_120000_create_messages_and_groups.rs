use crate::m20251130_100554_create_user_hackathon_tables::Users;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create message_groups table
        manager
            .create_table(
                Table::create()
                    .table(MessageGroups::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(MessageGroups::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(MessageGroups::Flag).string().not_null())
                    // recipient_id may reference either users.id or teams.id depending on context;
                    // using a nullable integer and explicit recipient_type to allow polymorphic recipients
                    .col(ColumnDef::new(MessageGroups::RecipientId).integer().null())
                    .col(
                        ColumnDef::new(MessageGroups::RecipientType)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(MessageGroups::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(MessageGroups::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create messages table
        manager
            .create_table(
                Table::create()
                    .table(Messages::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Messages::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Messages::SenderUserId).integer().not_null())
                    .col(
                        ColumnDef::new(Messages::MessageGroupId)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Messages::Content).text().not_null())
                    .col(
                        ColumnDef::new(Messages::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Messages::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_messages_sender_user")
                            .from(Messages::Table, Messages::SenderUserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_messages_message_group")
                            .from(Messages::Table, Messages::MessageGroupId)
                            .to(MessageGroups::Table, MessageGroups::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop messages first (FK)
        manager
            .drop_table(Table::drop().table(Messages::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(MessageGroups::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum MessageGroups {
    Table,
    Id,
    Flag,
    RecipientId,
    RecipientType,
    CreatedAt,
    UpdatedAt,
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

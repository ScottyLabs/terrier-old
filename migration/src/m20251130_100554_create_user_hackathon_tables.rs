use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create users table
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Users::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Users::OidcSub)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Users::Email).string().not_null())
                    .col(ColumnDef::new(Users::Name).string())
                    .col(ColumnDef::new(Users::GivenName).string())
                    .col(ColumnDef::new(Users::FamilyName).string())
                    .col(ColumnDef::new(Users::Picture).text())
                    .col(ColumnDef::new(Users::OidcIssuer).string().not_null())
                    .col(
                        ColumnDef::new(Users::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Users::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create hackathons table
        manager
            .create_table(
                Table::create()
                    .table(Hackathons::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Hackathons::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Hackathons::Name).string().not_null())
                    .col(
                        ColumnDef::new(Hackathons::Slug)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Hackathons::Description).text())
                    .col(ColumnDef::new(Hackathons::StartDate).timestamp().not_null())
                    .col(ColumnDef::new(Hackathons::EndDate).timestamp().not_null())
                    .col(
                        ColumnDef::new(Hackathons::IsActive)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Hackathons::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Hackathons::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create user_hackathon_roles table
        manager
            .create_table(
                Table::create()
                    .table(UserHackathonRoles::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserHackathonRoles::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(UserHackathonRoles::UserId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserHackathonRoles::HackathonId)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(UserHackathonRoles::Role).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(UserHackathonRoles::Table, UserHackathonRoles::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(UserHackathonRoles::Table, UserHackathonRoles::HackathonId)
                            .to(Hackathons::Table, Hackathons::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create unique constraint for user-hackathon combination
        manager
            .create_index(
                Index::create()
                    .name("idx_user_hackathon_unique")
                    .table(UserHackathonRoles::Table)
                    .col(UserHackathonRoles::UserId)
                    .col(UserHackathonRoles::HackathonId)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserHackathonRoles::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Hackathons::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Users {
    Table,
    Id,
    OidcSub,
    Email,
    Name,
    GivenName,
    FamilyName,
    Picture,
    OidcIssuer,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub enum Hackathons {
    Table,
    Id,
    Name,
    Slug,
    Description,
    StartDate,
    EndDate,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum UserHackathonRoles {
    Table,
    Id,
    UserId,
    HackathonId,
    Role,
}

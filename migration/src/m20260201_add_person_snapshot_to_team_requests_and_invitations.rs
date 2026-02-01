use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add snapshot columns to team_invitations
        manager
            .alter_table(
                Table::alter()
                    .table(TeamInvitations::Table)
                    .add_column(ColumnDef::new(TeamInvitations::PersonName).string().null())
                    .add_column(ColumnDef::new(TeamInvitations::PersonEmail).string().null())
                    .add_column(
                        ColumnDef::new(TeamInvitations::PersonPicture)
                            .string()
                            .null(),
                    )
                    .add_column(ColumnDef::new(TeamInvitations::PersonMajor).text().null())
                    .add_column(
                        ColumnDef::new(TeamInvitations::PersonGraduationYear)
                            .string()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Add snapshot columns to team_join_requests
        manager
            .alter_table(
                Table::alter()
                    .table(TeamJoinRequests::Table)
                    .add_column(ColumnDef::new(TeamJoinRequests::PersonName).string().null())
                    .add_column(
                        ColumnDef::new(TeamJoinRequests::PersonEmail)
                            .string()
                            .null(),
                    )
                    .add_column(
                        ColumnDef::new(TeamJoinRequests::PersonPicture)
                            .string()
                            .null(),
                    )
                    .add_column(ColumnDef::new(TeamJoinRequests::PersonMajor).text().null())
                    .add_column(
                        ColumnDef::new(TeamJoinRequests::PersonGraduationYear)
                            .string()
                            .null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(TeamInvitations::Table)
                    .drop_column(TeamInvitations::PersonName)
                    .drop_column(TeamInvitations::PersonEmail)
                    .drop_column(TeamInvitations::PersonPicture)
                    .drop_column(TeamInvitations::PersonMajor)
                    .drop_column(TeamInvitations::PersonGraduationYear)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(TeamJoinRequests::Table)
                    .drop_column(TeamJoinRequests::PersonName)
                    .drop_column(TeamJoinRequests::PersonEmail)
                    .drop_column(TeamJoinRequests::PersonPicture)
                    .drop_column(TeamJoinRequests::PersonMajor)
                    .drop_column(TeamJoinRequests::PersonGraduationYear)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum TeamInvitations {
    Table,
    Id,
    TeamId,
    UserId,
    Message,
    CreatedAt,
    PersonName,
    PersonEmail,
    PersonPicture,
    PersonMajor,
    PersonGraduationYear,
}

#[derive(DeriveIden)]
enum TeamJoinRequests {
    Table,
    Id,
    TeamId,
    UserId,
    Message,
    CreatedAt,
    PersonName,
    PersonEmail,
    PersonPicture,
    PersonMajor,
    PersonGraduationYear,
}

#[derive(DeriveIden)]
enum Teams {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}

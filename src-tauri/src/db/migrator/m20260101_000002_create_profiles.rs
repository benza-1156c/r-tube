use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Profiles::Table)
                    .if_not_exists()
                    .col(pk_auto(Profiles::Id))
                    .col(string_uniq(Profiles::GoogleId))
                    .col(string(Profiles::Name))
                    .col(string(Profiles::Email))
                    .col(string(Profiles::AvatarUrl))
                    .col(text(Profiles::Cookies))
                    .col(boolean(Profiles::IsActive).default(true))
                    .col(date_time(Profiles::UpdatedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Profiles::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Profiles {
    #[sea_orm(iden = "profile")]
    Table,
    Id,
    GoogleId,
    Name,
    Email,
    AvatarUrl,
    Cookies,
    IsActive,
    UpdatedAt,
}


use sea_orm_migration::prelude::*;

use super::m20250101_000001_create_users_table::Users;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Documents::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Documents::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Documents::UserId)
                        .string()
                        .not_null())
                    .col(ColumnDef::new(Documents::Title)
                        .string()
                        .not_null())
                    .col(ColumnDef::new(Documents::Description)
                        .text()
                        .null())
                    .col(ColumnDef::new(Documents::Content)
                        .text()
                        .null())
                    .col(ColumnDef::new(Documents::Tags)
                        .json()
                        .not_null()
                        .default("[]"))
                    .col(ColumnDef::new(Documents::CreatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Documents::UpdatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Documents::IsDeleted)
                        .boolean()
                        .not_null()
                        .default(false))
                    .col(ColumnDef::new(Documents::WordCount)
                        .integer()
                        .not_null()
                        .default(0))
                    .col(ColumnDef::new(Documents::CharCount)
                        .integer()
                        .not_null()
                        .default(0))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_documents_user_id")
                            .from(Documents::Table, Documents::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes for better query performance
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_documents_user_id")
                    .table(Documents::Table)
                    .col(Documents::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_documents_created_at")
                    .table(Documents::Table)
                    .col(Documents::CreatedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_documents_title")
                    .table(Documents::Table)
                    .col(Documents::Title)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Documents::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Documents {
    Table,
    Id,
    UserId,
    Title,
    Description,
    Content,
    Tags,
    CreatedAt,
    UpdatedAt,
    IsDeleted,
    WordCount,
    CharCount,
}
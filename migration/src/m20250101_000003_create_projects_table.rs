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
                    .table(Projects::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Projects::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Projects::UserId)
                        .string()
                        .not_null())
                    .col(ColumnDef::new(Projects::Name)
                        .string()
                        .not_null())
                    .col(ColumnDef::new(Projects::Description)
                        .text()
                        .null())
                    .col(ColumnDef::new(Projects::Settings)
                        .json()
                        .not_null()
                        .default("{}"))
                    .col(ColumnDef::new(Projects::CreatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Projects::UpdatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Projects::IsDeleted)
                        .boolean()
                        .not_null()
                        .default(false))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_projects_user_id")
                            .from(Projects::Table, Projects::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create project_documents join table
        manager
            .create_table(
                Table::create()
                    .table(ProjectDocuments::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ProjectDocuments::ProjectId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProjectDocuments::DocumentId)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ProjectDocuments::AddedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()))
                    .col(ColumnDef::new(ProjectDocuments::Order)
                        .integer()
                        .not_null()
                        .default(0))
                    .primary_key(
                        Index::create()
                            .col(ProjectDocuments::ProjectId)
                            .col(ProjectDocuments::DocumentId)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_project_documents_project_id")
                            .from(ProjectDocuments::Table, ProjectDocuments::ProjectId)
                            .to(Projects::Table, Projects::Id)
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
                    .name("idx_projects_user_id")
                    .table(Projects::Table)
                    .col(Projects::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_projects_created_at")
                    .table(Projects::Table)
                    .col(Projects::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ProjectDocuments::Table).to_owned())
            .await?;
            
        manager
            .drop_table(Table::drop().table(Projects::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Projects {
    Table,
    Id,
    UserId,
    Name,
    Description,
    Settings,
    CreatedAt,
    UpdatedAt,
    IsDeleted,
}

#[derive(DeriveIden)]
pub enum ProjectDocuments {
    Table,
    ProjectId,
    DocumentId,
    AddedAt,
    Order,
}
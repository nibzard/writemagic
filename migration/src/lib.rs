pub use sea_orm_migration::prelude::*;

mod m20250101_000001_create_users_table;
mod m20250101_000002_create_documents_table;
mod m20250101_000003_create_projects_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250101_000001_create_users_table::Migration),
            Box::new(m20250101_000002_create_documents_table::Migration),
            Box::new(m20250101_000003_create_projects_table::Migration),
        ]
    }
}
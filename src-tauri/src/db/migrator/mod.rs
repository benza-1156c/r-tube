use sea_orm_migration::prelude::*;

mod m20260101_000002_create_profiles;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20260101_000002_create_profiles::Migration)]
    }
}

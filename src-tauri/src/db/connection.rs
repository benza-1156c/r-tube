use std::fs;
use std::path::Path;
use sea_orm::{Database, DatabaseConnection, DbErr};
use sea_orm_migration::MigratorTrait;

use crate::db::migrator::Migrator;

pub async fn init_db(app_path: &Path) -> Result<DatabaseConnection, DbErr> {
    if !app_path.exists() {
        fs::create_dir_all(app_path)
            .map_err(|e| DbErr::Custom(format!("Failed to create app data directory: {e}")))?;
    }

    let db_path = app_path.join("rtube.sqlite");
    let database_url = format!("sqlite://{}?mode=rwc", db_path.to_string_lossy());

    let db = Database::connect(&database_url).await?;

    // Run migrations automatically
    Migrator::up(&db, None).await?;

    Ok(db)
}

use anyhow::Result;
use sea_orm::{Database, DatabaseConnection, DbErr};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum JourneyDbError {
    #[error(transparent)]
    ConnectionError(#[from] DbErr),
}

pub async fn get_conn() -> Result<DatabaseConnection, JourneyDbError> {
    let db = Database::connect("sqlite::memory:").await?;

    db.get_schema_registry("journey-db::entity::*")
        .sync(&db)
        .await?;
    Ok(db)
}

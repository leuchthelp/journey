use anyhow::Result;
use sea_orm::{Database, DatabaseConnection, DbErr};
use thiserror::Error;

use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Error, Serialize, Deserialize, Type)]
pub enum JourneyDbError {
    #[error(transparent)]
    #[serde(skip)]
    ConnectionError(#[from] DbErr),
}

pub async fn get_conn() -> Result<DatabaseConnection, JourneyDbError> {
    let db = Database::connect("sqlite::memory:").await?;

    db.get_schema_registry("journey-db::entity::*")
        .sync(&db)
        .await?;
    Ok(db)
}

pub trait Convertible<T> {
    type DTO;

    fn from_model(item: T) -> Result<Self::DTO>;
}

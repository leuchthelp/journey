use anyhow::Result;
use sea_orm::{Database, DatabaseConnection};
use serde::Serialize;
use specta::Type;
use thiserror::Error;

#[derive(Debug, Error, Serialize, Type)]
pub enum JourneyDbError {
    #[error("Failed to establish database connection.")]
    ConnectionError,
}

pub async fn get_conn() -> Result<DatabaseConnection, JourneyDbError> {
    let conn = match Database::connect("sqlite:db.sqlite?mode=rwc").await {
        Ok(conn) => Ok(conn),
        Err(_) => Err(JourneyDbError::ConnectionError),
    }?;

    match conn
        .get_schema_registry("journey-db::entity::*")
        .sync(&conn)
        .await
    {
        Ok(_) => Ok(()),
        Err(_) => Err(JourneyDbError::ConnectionError),
    }?;
    Ok(conn)
}

pub trait Convertible<T> {
    type DTO;

    fn from_model(item: T) -> Result<Self::DTO>;
    fn to_dto_vec(items: impl IntoIterator<Item = T>) -> Result<Option<Vec<Self::DTO>>> {
        let mut peekable = items.into_iter().peekable();
        if peekable.peek().is_none() {
            return Ok(None);
        }

        let mut result: Vec<Self::DTO> = vec![];
        for item in peekable {
            let dto = Self::from_model(item)?;
            result.push(dto);
        }

        Ok(Some(result))
    }
}

use anyhow::Result;
use sea_orm::{Database, DatabaseConnection};
use serde::Serialize;
use specta::Type;
use thiserror::Error;

#[derive(Debug, Error, Serialize, Type)]
//#[serde(tag = "error", content = "data")]
pub enum JourneyDbError {
    #[error("Failed to establish database connection: {0}")]
    ConnectionError(String),
    #[error("Record not found: {0}")]
    RecordNotFound(String),
    #[error("Transaction failed: {0}")]
    FailedTransactionError(String),
    #[error("Unknown error occurred: {0}")]
    Unknown(String),
}

pub async fn get_conn() -> Result<DatabaseConnection, JourneyDbError> {
    let conn = match Database::connect("sqlite:db.sqlite?mode=rwc").await {
        Ok(conn) => Ok(conn),
        Err(err) => Err(JourneyDbError::ConnectionError(err.to_string())),
    }?;

    match conn
        .get_schema_registry("journey-db::entity::*")
        .sync(&conn)
        .await
    {
        Ok(_) => Ok(()),
        Err(err) => Err(JourneyDbError::ConnectionError(err.to_string())),
    }?;
    Ok(conn)
}

#[derive(Debug, Error, Serialize, Type)]
pub enum ConversionError {
    #[error("Failed conversion: {0}")]
    FailedItemRetrievalError(String),
    #[error("Failed to parse Url: {0}")]
    FailedParseUrlError(String),
}

pub type ConversionResult<T> = Result<T, ConversionError>;

pub trait Convertible<T> {
    type DTO;

    fn from_model(item: T) -> ConversionResult<Self::DTO>;
    fn to_dto_vec(items: impl IntoIterator<Item = T>) -> ConversionResult<Option<Vec<Self::DTO>>> {
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

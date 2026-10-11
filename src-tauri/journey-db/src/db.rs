use std::{collections::HashMap, hash::Hash};

use anyhow::Result;
use sea_orm::{Database, DatabaseConnection, EntityTrait, ModelTrait, compound::BelongsTo};
use sea_query::ValueType;
use serde::Serialize;
use specta::Type;
use thiserror::Error;

#[derive(Debug, Error, Serialize, Type)]
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
    #[error("Failed to get matching Key column entry for HashMap: {0}")]
    FailedGetHashmapKeyError(String),
}

pub type ConversionResult<T> = Result<T, ConversionError>;

pub trait Convertible<T: ModelTrait> {
    type DTO;

    fn from_model(item: T) -> ConversionResult<Self::DTO>;
    fn option_from<E: EntityTrait<ModelEx = T>>(
        option: BelongsTo<E>,
    ) -> ConversionResult<Option<Self::DTO>> {
        match option.into_option() {
            Some(item) => Ok(Some(Self::from_model(item)?)),
            None => Ok(None),
        }
    }
    fn option_from_option<E: EntityTrait<ModelEx = T>>(
        option: BelongsTo<Option<E>>,
    ) -> ConversionResult<Option<Self::DTO>> {
        match option.into_option() {
            Some(item) => Ok(Some(Self::from_model(item)?)),
            None => Ok(None),
        }
    }
    fn to_vec(items: impl IntoIterator<Item = T>) -> ConversionResult<Option<Vec<Self::DTO>>> {
        let mut result: Vec<Self::DTO> = vec![];
        for item in items.into_iter() {
            let dto = Self::from_model(item)?;
            result.push(dto);
        }

        match result.is_empty() {
            true => Ok(None),
            false => Ok(Some(result)),
        }
    }
    fn to_hashmap<Key: ValueType + Eq + Hash>(
        items: impl IntoIterator<Item = T>,
        key_column: <<T>::Entity as EntityTrait>::Column,
    ) -> ConversionResult<Option<HashMap<Key, Self::DTO>>> {
        let mut result: HashMap<Key, Self::DTO> = HashMap::default();
        for item in items.into_iter() {
            let key = match <Key>::try_from(item.get(key_column)) {
                Ok(key) => Ok(key),
                Err(err) => Err(ConversionError::FailedGetHashmapKeyError(err.to_string())),
            }?;
            let dto = Self::from_model(item)?;
            result.insert(key, dto);
        }

        match result.is_empty() {
            true => Ok(None),
            false => Ok(Some(result)),
        }
    }
    fn to_hashmap_vec<Key: ValueType + Eq + Hash>(
        items: impl IntoIterator<Item = T>,
        key_column: <<T>::Entity as EntityTrait>::Column,
    ) -> ConversionResult<Option<HashMap<Key, Vec<Self::DTO>>>> {
        let mut result: HashMap<Key, Vec<Self::DTO>> = HashMap::default();
        for item in items.into_iter() {
            let key = match <Key>::try_from(item.get(key_column)) {
                Ok(key) => Ok(key),
                Err(err) => Err(ConversionError::FailedGetHashmapKeyError(err.to_string())),
            }?;
            let dto = Self::from_model(item)?;

            match result.get_mut(&key) {
                Some(vec) => vec.push(dto),
                None => _ = result.insert(key, vec![dto]),
            }
        }

        match result.is_empty() {
            true => Ok(None),
            false => Ok(Some(result)),
        }
    }
}

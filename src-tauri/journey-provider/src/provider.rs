use std::collections::HashMap;
use std::fmt::Debug;

use anyhow::Result;
use async_trait::async_trait;
use dyn_clone::{DynClone, clone_trait_object};
use journey_db::entity::providers::{self, ProviderAuthSchema};
use journey_db::entity::{ProviderKey, ProviderVariant};
use journey_db::get_conn;
use journey_db::sea_orm::EntityTrait;
use journey_db::sea_query::OnConflict;
use journey_keyring::Entry;
use journey_utils::constants::PRODUCT_NAME;
use serde::Serialize;
use specta::Type;
use thiserror::Error;
use tracing::warn;
use url::Url;
use uuid::Uuid;

use crate::indexer::Indexer;
use crate::jellyfin::jellyfin_provider::JellyfinProviderError;

#[derive(Debug, Error, Serialize, Type)]
//#[serde(tag = "error", content = "data")]
pub enum ProviderError {
    #[error("Error throw if a given auth function is not implemented.")]
    NotImplError,
    #[error("Found more than one access token, removing all.")]
    TooManyCredentialsError,
    #[error("Found no access token, nothing to remove: {0}")]
    NoCredentialsError(String),
    #[error("ProviderVariant has not been set yet.")]
    MissingVariantError,
    #[error("server_id has not been set yet, try authenticating first.")]
    MissingServerIdError,
    #[error("user_id has not been set yet, try authenticating first.")]
    MissingUserIdError,
    #[error("Url has not been set yet, provide one first.")]
    MissingUrlError,
    #[error("AuthSchema have not been set yet.")]
    MissingAuthSchemaError,
    #[error("Failed to parse given String to Uuid: {0}")]
    FailedUuidParseError(String),
    #[error("Failed to authenticate with username & password: {0}")]
    FailedPasswordAuthError(String),
    #[error("Failed to create keyring entry: {0}")]
    FailedCreateEntryError(String),
    #[error("Failed to remove keyring entry. Credentials might leak: {0}")]
    FailedRemoveEntryError(String),
    #[error("Failed to save token to OS keyring: {0}")]
    SaveTokenError(String),
    #[error("Failed to insert provider into database: {0}")]
    FailedDbInsertError(String),
    #[error("Failed to delete provider from database. Might not exist: {0}")]
    FailedDbRemoveError(String),
    #[error("Failed to convert sea-orm ActiveModel into Model: {0}")]
    FailedConvModelError(String),
    #[error("Failed to parse the given String to an Url: {0}")]
    FailedParseUrlError(String),
    #[error(transparent)]
    JellyfinProviderError(#[from] JellyfinProviderError),
    #[error(transparent)]
    JourneyDbError(#[from] journey_db::JourneyDbError),
}

pub type ProviderResult<T> = Result<T, ProviderError>;

pub trait NewProvider {
    type Provider;

    fn new(model: providers::ActiveModelEx) -> Box<Self>;
}

#[async_trait]
pub trait RequiredForProvider {
    fn get_model(&self) -> &providers::ActiveModelEx;
    fn get_indexer(&self) -> ProviderResult<Box<dyn Indexer + Send + Sync>>;
    fn get_auth_schema(&self) -> Vec<ProviderAuthSchema>;
    fn invalidate(&mut self) -> ProviderResult<()>;
    async fn password_auth(&mut self, uname: String, psw: String) -> ProviderResult<String>;
}

#[async_trait]
pub trait Provider: RequiredForProvider + DynClone + Debug {
    fn ty(&self) -> ProviderResult<ProviderVariant> {
        match self.get_model().ty.try_as_ref() {
            Some(variant) => Ok(*variant),
            _ => Err(ProviderError::MissingVariantError),
        }
    }
    fn user_id(&self) -> ProviderResult<Uuid> {
        match self.get_model().user_id.try_as_ref() {
            Some(user_id) if *user_id != Uuid::nil() => Ok(*user_id),
            _ => Err(ProviderError::MissingServerIdError),
        }
    }
    fn provider_id(&self) -> ProviderResult<Uuid> {
        match self.get_model().provider_id.try_as_ref() {
            Some(server_id) if *server_id != Uuid::nil() => Ok(*server_id),
            _ => Err(ProviderError::MissingServerIdError),
        }
    }
    fn url(&self) -> ProviderResult<Url> {
        match self.get_model().url.try_as_ref() {
            Some(url) => Ok(match Url::parse(url) {
                Ok(url) => url,
                Err(err) => return Err(ProviderError::FailedParseUrlError(err.to_string())),
            }),
            _ => Err(ProviderError::MissingUrlError),
        }
    }
    fn save_token(&self, access_token: &String) -> ProviderResult<()> {
        let token_entry = match Entry::new(
            PRODUCT_NAME,
            &format!("{}-{}", self.provider_id()?, self.user_id()?),
        ) {
            Ok(entry) => Ok(entry),
            Err(err) => Err(ProviderError::FailedCreateEntryError(err.to_string())),
        }?;

        match token_entry.set_password(&access_token) {
            Ok(_) => Ok(()),
            Err(err) => Err(ProviderError::SaveTokenError(err.to_string())),
        }
    }
    fn retrieve_tokens(&self) -> ProviderResult<Vec<Entry>> {
        let entries_res = Entry::search(&HashMap::from([
            ("service", "journey"),
            (
                "user",
                &format!("{}-{}", self.provider_id()?, self.user_id()?),
            ),
        ]));

        match entries_res {
            Ok(entries) => Ok(entries),
            Err(err) => Err(ProviderError::NoCredentialsError(err.to_string())),
        }
    }
    fn remove_token(&self) -> ProviderResult<()> {
        let entries = self.retrieve_tokens()?;

        for entry in &entries {
            match entry.delete_credential() {
                Ok(_) => Ok(()),
                Err(err) => Err(ProviderError::FailedRemoveEntryError(err.to_string())),
            }?;
        }

        match entries.len() {
            len if len > 1 => Err(ProviderError::TooManyCredentialsError),
            len if len < 1 => Err(ProviderError::NoCredentialsError("".to_string())),
            _ => Ok(()),
        }
    }
    fn key(&self) -> ProviderResult<ProviderKey> {
        Ok(ProviderKey {
            user_id: self.user_id()?,
            provider_id: self.provider_id()?,
        })
    }
    fn authenticated(&self) -> ProviderResult<bool> {
        let tokens = self.retrieve_tokens()?;

        warn!(
            "TODO Need to validate all available tokens to ensure we are actually authenticated. 
            Currently not implemented, just finding any tokens is considered to be authenticated."
        );
        match tokens.len() {
            len if len == 0 => Ok(false),
            _ => Ok(true),
        }
    }
    async fn add_to_db(&self) -> ProviderResult<()> {
        match providers::Entity::insert(self.get_model().clone())
            .on_conflict(
                OnConflict::column(providers::Column::UserId)
                    .do_nothing()
                    .to_owned(),
            )
            .try_insert()
            .exec(&get_conn().await?)
            .await
        {
            Ok(_) => Ok(()),
            Err(err) => Err(ProviderError::FailedDbInsertError(err.to_string())),
        }
    }
    async fn remove_from_db(&self) -> ProviderResult<()> {
        match self.get_model().clone().delete(&get_conn().await?).await {
            Ok(_) => Ok(()),
            Err(err) => Err(ProviderError::FailedDbRemoveError(err.to_string())),
        }
    }
}

clone_trait_object!(Provider);

use crate::jellyfin::jellyfin_provider::JellyfinProviderError;
use anyhow::Result;
use async_trait::async_trait;
use dyn_clone::{DynClone, clone_trait_object};
use journey_db::entity::providers::{self};
use journey_db::entity::{ProviderKey, ProviderVariant, Providers};
use journey_db::get_conn;
use journey_db::sea_orm::EntityTrait;
use journey_db::sea_query::OnConflict;
use journey_keyring::Entry;
use journey_utils::constants::PRODUCT_NAME;
use serde::Serialize;
use specta::Type;
use std::collections::HashMap;
use std::fmt::Debug;
use thiserror::Error;
use tracing::warn;
use url::Url;
use uuid::Uuid;

#[derive(Debug, Error, Serialize, Type)]
pub enum ProviderError {
    #[error("Found more than one access token, removing all.")]
    TooManyCredentialsError,
    #[error("Found no access token, nothing to remove.")]
    NoCredentialsError,
    #[error("Found no such provider variant.")]
    NoSuchVariantError,
    #[error("Failed to create keyring entry.")]
    FailedCreateEntryError,
    #[error("Failed to remove keyring entry. Credentials might leak.")]
    FailedRemoveEntryError,
    #[error("Failed to save token to OS keyring.")]
    SaveTokenError,
    #[error("Failed to insert provider to database.")]
    FailedDbInsertError(String),
    #[error("Failed to delete provider from database. Might not exist.")]
    FailedDbRemoveError(String),
    #[error("Failed to convert sea-orm ActiveModel into Model.")]
    FailedConvModelError(String),
    #[error("Failed to parse the given String to an Url.")]
    FailedParseUrlError(String),
    #[error(transparent)]
    JellyfinProviderError(#[from] JellyfinProviderError),
    #[error(transparent)]
    JourneyDbError(#[from] journey_db::JourneyDbError),
}

pub type ProviderResult<T> = Result<T, ProviderError>;

pub trait NewProvider {
    type Provider;

    fn new(model: providers::ActiveModelEx) -> ProviderResult<Box<Self::Provider>>;
}

#[async_trait]
pub trait RequiredForProvider {
    fn ty(&self) -> ProviderVariant;
    async fn get_model(&self) -> providers::ActiveModelEx;
    async fn set_model(&mut self, new: providers::ActiveModelEx);
    async fn index(&mut self) -> ProviderResult<()>;
    async fn password_auth(&mut self, uname: String, psw: String) -> ProviderResult<String>;
    async fn invalidate(&mut self) -> ProviderResult<()>;
}

#[async_trait]
pub trait Provider: RequiredForProvider + DynClone + Debug {
    async fn user_id(&self) -> ProviderResult<Uuid> {
        match self.get_model().await.user_id.try_as_ref() {
            Some(user_id) if *user_id != Uuid::nil() => Ok(*user_id),
            _ => Err(JellyfinProviderError::MissingServerIdError.into()),
        }
    }
    async fn server_id(&self) -> ProviderResult<Uuid> {
        match self.get_model().await.server_id.try_as_ref() {
            Some(server_id) if *server_id != Uuid::nil() => Ok(*server_id),
            _ => Err(JellyfinProviderError::MissingServerIdError.into()),
        }
    }
    async fn url(&self) -> ProviderResult<Url> {
        match self.get_model().await.url.try_as_ref() {
            Some(url) => Ok(match Url::parse(url) {
                Ok(url) => url,
                Err(err) => return Err(ProviderError::FailedParseUrlError(err.to_string())),
            }),
            _ => Err(JellyfinProviderError::MissingUrlError.into()),
        }
    }
    async fn save_token(&self, access_token: &String) -> ProviderResult<()> {
        let token_entry = match Entry::new(
            PRODUCT_NAME,
            &format!("{}-{}", self.server_id().await?, self.user_id().await?),
        ) {
            Ok(entry) => entry,
            Err(_) => return Err(ProviderError::FailedCreateEntryError),
        };

        match token_entry.set_password(&access_token) {
            Ok(_) => Ok(()),
            Err(_) => Err(ProviderError::SaveTokenError),
        }
    }
    async fn retrieve_tokens(&self) -> ProviderResult<Vec<Entry>> {
        let entries_res = Entry::search(&HashMap::from([
            ("service", "journey"),
            (
                "user",
                &format!("{}-{}", self.server_id().await?, self.user_id().await?),
            ),
        ]));

        match entries_res {
            Ok(entries) => Ok(entries),
            Err(_) => Err(ProviderError::NoCredentialsError),
        }
    }
    async fn remove_token(&self) -> ProviderResult<()> {
        let entries = self.retrieve_tokens().await?;

        for entry in &entries {
            match entry.delete_credential() {
                Ok(_) => (),
                Err(_) => return Err(ProviderError::FailedRemoveEntryError),
            };
        }

        match entries.len() {
            len if len > 1 => Err(ProviderError::TooManyCredentialsError),
            len if len < 1 => Err(ProviderError::NoCredentialsError),
            _ => Ok(()),
        }
    }
    async fn key(&self) -> ProviderResult<ProviderKey> {
        Ok(ProviderKey {
            user_id: self.user_id().await?,
            server_id: self.server_id().await?,
        })
    }
    async fn authenticated(&self) -> ProviderResult<bool> {
        let tokens = self.retrieve_tokens().await?;

        warn!(
            "TODO Need to validate all available tokens to ensure we are actually authenticated. 
            Currently not implemented, just finding any tokens is considered to be authenticated."
        );
        warn!("tokens: {:#?}", tokens);
        match tokens.len() {
            len if len == 0 => Ok(false),
            _ => Ok(true),
        }
    }
    async fn add_to_db(&self) -> ProviderResult<()> {
        let provider = providers::ActiveModelEx::new()
            .set_user_id(self.user_id().await?)
            .set_server_id(self.server_id().await?)
            .set_ty(self.ty())
            .set_url(self.url().await?);

        match providers::Entity::insert(provider)
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
        let res = Providers::delete_by_user_id(self.user_id().await?)
            .exec(&get_conn().await?)
            .await;

        match res {
            Ok(_) => Ok(()),
            Err(err) => Err(ProviderError::FailedDbRemoveError(err.to_string())),
        }
    }
}

clone_trait_object!(Provider);

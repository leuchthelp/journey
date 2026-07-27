use anyhow::Result;
use async_trait::async_trait;
use dyn_clone::{DynClone, clone_trait_object};
use jellyfin_sdk_rs::JellyfinSDKError;
use jellyfin_sdk_rs::apis::authentication_api::AuthenticateUserByNameError;
use journey_db::entity::Providers;
use journey_db::entity::providers::ActiveModel;
use journey_db::get_conn;
use journey_db::sea_orm::{ActiveModelTrait, Set};
use journey_keyring::{Entry, keyring_core};
use journey_utils::get_env_prod;
use std::any::type_name_of_val;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{DefaultHasher, Hash, Hasher};
use thiserror::Error;
use url::Url;
use uuid::Uuid;

use crate::jellyfin::jellyfin_provider::JellyfinProviderError;

use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Error, Serialize, Deserialize, Type)]
pub enum ProviderError {
    #[error("Failed to save access token to native keyring: {0}")]
    #[serde(skip)]
    KeyringCoreError(#[from] keyring_core::Error),
    #[error("Found more than one access token, removing all.")]
    TooManyCredentialsError,
    #[error("Found no access token, nothing to remove.")]
    NoCredentialsError,
    #[error(transparent)]
    #[serde(skip)]
    UuidParserError(#[from] uuid::Error),
    #[error(transparent)]
    #[serde(skip)]
    DbError(#[from] journey_db::sea_orm::DbErr),
    #[error(transparent)]
    JourneyDbError(#[from] journey_db::JourneyDbError),
    #[error(transparent)]
    #[serde(skip)]
    EnvLoadingError(#[from] dotenvy::Error),
    #[error(transparent)]
    JellyfinProviderError(#[from] JellyfinProviderError),
    #[error(transparent)]
    #[serde(skip)]
    JellyfinAuthenticationError(#[from] jellyfin_sdk_rs::apis::Error<AuthenticateUserByNameError>),
    #[error(transparent)]
    #[serde(skip)]
    JellyfinConfigurationError(#[from] JellyfinSDKError),
    #[error(transparent)]
    #[serde(skip)]
    ParseUrlError(#[from] url::ParseError),
}

pub type ProviderResult<T> = Result<T, ProviderError>;

#[async_trait]
pub trait ProviderNew {
    type Provider;

    fn new(params: ActiveModel) -> ProviderResult<Box<Self::Provider>>;
    async fn authenticate_with_pw(&mut self, uname: String, psw: String) -> ProviderResult<()>;
}

#[async_trait]
pub trait Provider: DynClone + Debug {
    fn user_id(&self) -> ProviderResult<Uuid>;
    fn server_id(&self) -> ProviderResult<Uuid>;
    fn url(&self) -> ProviderResult<Url>;
    fn type_(&self) -> String {
        return type_name_of_val(self).to_string();
    }
    fn save_token(&self, access_token: &String) -> ProviderResult<()> {
        let token_entry = Entry::new(
            &get_env_prod()?.var("VITE_JOURNEY_NAME")?,
            format!("{}-{}", self.server_id()?, self.user_id()?).as_str(),
        )?;
        token_entry.set_password(&access_token)?;

        Ok(())
    }
    fn retrieve_token(&self) -> ProviderResult<Vec<Entry>> {
        let entries = Entry::search(&HashMap::from([
            ("service", "journey"),
            (
                "user",
                format!("{}-{}", self.server_id()?, self.user_id()?).as_str(),
            ),
        ]))?;

        Ok(entries)
    }
    fn remove_token(&self) -> ProviderResult<()> {
        let entries = self.retrieve_token()?;

        for entry in &entries {
            entry.delete_credential()?;
        }

        match Some(entries.len()) {
            Some(len) if len > 1 => return Err(ProviderError::TooManyCredentialsError),
            Some(len) if len < 1 => return Err(ProviderError::NoCredentialsError),
            _ => return Ok(()),
        };
    }
    fn hash(&self) -> ProviderResult<u64> {
        let mut s = DefaultHasher::new();
        self.user_id()?.hash(&mut s);
        self.server_id()?.hash(&mut s);
        Ok(s.finish())
    }
    fn authenticated(&self) -> ProviderResult<bool> {
        let tokens = self.retrieve_token();
        if tokens.is_err() {
            return Ok(false);
        }

        let tokens = tokens?;
        if tokens.len() == 0 {
            return Ok(false);
        }

        Ok(true)
    }
    async fn invalidate(&mut self) -> ProviderResult<()>;
    async fn add_to_db(&self) -> ProviderResult<()> {
        let provider = ActiveModel {
            user_id: Set(self.user_id()?),
            server_id: Set(self.server_id()?),
            hash: Set(self.hash()?),
            kind: Set(self.type_()),
            url: Set(self.url()?.to_string()),
        };
        provider.insert(&get_conn().await?).await?;
        Ok(())
    }
    async fn remove_from_db(&self) -> ProviderResult<()> {
        Providers::delete_by_user_id(self.user_id()?)
            .exec(&get_conn().await?)
            .await?;
        Ok(())
    }
}

clone_trait_object!(Provider);

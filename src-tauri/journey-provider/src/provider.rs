use crate::jellyfin::jellyfin_provider::JellyfinProviderError;
use anyhow::Result;
use async_trait::async_trait;
use dyn_clone::{DynClone, clone_trait_object};
use jellyfin_sdk_rs::JellyfinSDKError;
use jellyfin_sdk_rs::apis::authentication_api::AuthenticateUserByNameError;
use journey_db::entity::providers::ActiveModel;
use journey_db::entity::{ProviderVariant, Providers};
use journey_db::get_conn;
use journey_db::sea_orm::{ActiveModelTrait, Set};
use journey_keyring::{Entry, keyring_core};
use journey_utils::constants::PRODUCT_NAME;
use std::collections::HashMap;
use std::fmt::Debug;
use thiserror::Error;
use url::Url;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("Found more than one access token, removing all.")]
    TooManyCredentialsError,
    #[error("Found no access token, nothing to remove.")]
    NoCredentialsError,
    #[error("Found no such provider variant")]
    NoSuchVariantError,
    #[error(transparent)]
    KeyringCoreError(#[from] keyring_core::Error),
    #[error(transparent)]
    UuidParserError(#[from] uuid::Error),
    #[error(transparent)]
    DbError(#[from] journey_db::sea_orm::DbErr),
    #[error(transparent)]
    JourneyDbError(#[from] journey_db::JourneyDbError),
    #[error(transparent)]
    EnvLoadingError(#[from] dotenvy::Error),
    #[error(transparent)]
    JellyfinProviderError(#[from] JellyfinProviderError),
    #[error(transparent)]
    JellyfinAuthenticationError(#[from] jellyfin_sdk_rs::apis::Error<AuthenticateUserByNameError>),
    #[error(transparent)]
    JellyfinConfigurationError(#[from] JellyfinSDKError),
    #[error(transparent)]
    ParseUrlError(#[from] url::ParseError),
}

pub type ProviderResult<T> = Result<T, ProviderError>;

pub trait ProviderNew {
    type Provider;

    fn new(params: ActiveModel) -> ProviderResult<Box<Self::Provider>>;
}

#[async_trait]
pub trait Provider: DynClone + Debug {
    fn user_id(&self) -> ProviderResult<Uuid>;
    fn server_id(&self) -> ProviderResult<Uuid>;
    fn url(&self) -> ProviderResult<Url>;
    fn ty(&self) -> ProviderVariant;
    fn save_token(&self, access_token: &String) -> ProviderResult<()> {
        let token_entry = Entry::new(
            PRODUCT_NAME,
            &format!("{}-{}", self.server_id()?, self.user_id()?),
        )?;
        Ok(token_entry.set_password(&access_token)?)
    }
    fn retrieve_tokens(&self) -> ProviderResult<Vec<Entry>> {
        Ok(Entry::search(&HashMap::from([
            ("service", "journey"),
            (
                "user",
                &format!("{}-{}", self.server_id()?, self.user_id()?),
            ),
        ]))?)
    }
    fn remove_token(&self) -> ProviderResult<()> {
        let entries = self.retrieve_tokens()?;

        for entry in &entries {
            entry.delete_credential()?;
        }

        match entries.len() {
            len if len > 1 => Err(ProviderError::TooManyCredentialsError),
            len if len < 1 => Err(ProviderError::NoCredentialsError),
            _ => Ok(()),
        }
    }
    fn key(&self) -> ProviderResult<(Uuid, Uuid)> {
        Ok((self.user_id()?, self.server_id()?))
    }
    fn authenticated(&self) -> ProviderResult<bool> {
        let tokens = self.retrieve_tokens();

        match tokens {
            Err(_) => Ok(false),
            Ok(tokens) => match tokens.len() {
                len if len == 0 => Ok(false),
                _ => Ok(true),
            },
        }
    }
    async fn password_auth(&mut self, uname: String, psw: String) -> ProviderResult<String>;
    async fn invalidate(&mut self) -> ProviderResult<()>;
    async fn add_to_db(&self) -> ProviderResult<()> {
        let provider = ActiveModel {
            user_id: Set(self.user_id()?),
            server_id: Set(self.server_id()?),
            ty: Set(self.ty()),
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

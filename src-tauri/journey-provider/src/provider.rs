use anyhow::Result;
use async_trait::async_trait;
use jellyfin_sdk_rs::JellyfinSDKError;
use jellyfin_sdk_rs::apis::authentication_api::AuthenticateUserByNameError;
use journey_keyring::{Entry, keyring_core};
use journey_utils::get_env_prod;
use std::any::type_name_of_val;
use std::hash::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use thiserror::Error;
use url::Url;
use uuid::Uuid;

use crate::jellyfin::jellyfin_provider::JellyfinProviderError;

#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("Failed to save access token to native keyring: {0}")]
    SetTokenFailureError(#[from] keyring_core::Error),
    #[error(transparent)]
    UuidParserError(#[from] uuid::Error),
    #[error(transparent)]
    EnvLoadingError(#[from] dotenvy::Error),
    #[error(transparent)]
    JellyfinProviderError(#[from] JellyfinProviderError),
    #[error(transparent)]
    JellyfinAuthenticationError(#[from] jellyfin_sdk_rs::apis::Error<AuthenticateUserByNameError>),
    #[error(transparent)]
    JellyfinConfigurationError(#[from] JellyfinSDKError),
}

pub type ProviderResult<T> = Result<T, ProviderError>;

#[async_trait]
pub trait ProviderNew<T> {
    fn new(params: ProviderParams) -> ProviderResult<T>;
    async fn authenticate_with_pw(&mut self, uname: String, psw: String) -> ProviderResult<()>;
}

pub trait Provider {
    fn user_id(&self) -> ProviderResult<Uuid>;
    fn server_id(&self) -> ProviderResult<Uuid>;
    fn url(&self) -> &Url;
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
    fn hash(&self) -> ProviderResult<u64> {
        let mut s = DefaultHasher::new();
        self.user_id()?.hash(&mut s);
        self.server_id()?.hash(&mut s);
        Ok(s.finish())
    }
    fn authenticated(&self) -> &bool;
    fn invalidate(&self) -> ProviderResult<()>;
}

#[derive(Debug, Eq, PartialEq)]
pub struct ProviderParams {
    pub user_id: Option<Uuid>,
    pub server_id: Option<Uuid>,
    pub url: Url,
}

use crate::jellyfin::jellyfin_provider::JellyfinProviderError;
use anyhow::Result;
use async_trait::async_trait;
use dyn_clone::{DynClone, clone_trait_object};
use journey_db::entity::providers::{self, ActiveModel};
use journey_db::entity::{ProviderKey, ProviderVariant, Providers};
use journey_db::get_conn;
use journey_db::sea_orm::{EntityTrait, Set};
use journey_db::sea_query::OnConflict;
use journey_keyring::Entry;
use journey_utils::constants::PRODUCT_NAME;
use serde::Serialize;
use specta::Type;
use std::collections::HashMap;
use std::fmt::Debug;
use thiserror::Error;
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
    FailedDbInsertError,
    #[error("Failed to delete provider from database. Might not exist.")]
    FailedDbRemoveError,
    #[error("Failed to convert sea-orm ActiveModel into Model.")]
    FailedConvModelError,
    #[error("Failed to parse the given String to an Url.")]
    FailedParseUrlError,
    #[error(transparent)]
    JellyfinProviderError(#[from] JellyfinProviderError),
    #[error(transparent)]
    JourneyDbError(#[from] journey_db::JourneyDbError),
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
        let token_entry = match Entry::new(
            PRODUCT_NAME,
            &format!("{}-{}", self.server_id()?, self.user_id()?),
        ) {
            Ok(entry) => entry,
            Err(_) => return Err(ProviderError::FailedCreateEntryError),
        };

        match token_entry.set_password(&access_token) {
            Ok(_) => Ok(()),
            Err(_) => Err(ProviderError::SaveTokenError),
        }
    }
    fn retrieve_tokens(&self) -> ProviderResult<Vec<Entry>> {
        let entries_res = Entry::search(&HashMap::from([
            ("service", "journey"),
            (
                "user",
                &format!("{}-{}", self.server_id()?, self.user_id()?),
            ),
        ]));

        match entries_res {
            Ok(entries) => Ok(entries),
            Err(_) => Err(ProviderError::NoCredentialsError),
        }
    }
    fn remove_token(&self) -> ProviderResult<()> {
        let entries = self.retrieve_tokens()?;

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
    fn key(&self) -> ProviderResult<ProviderKey> {
        Ok(ProviderKey {
            user_id: self.user_id()?,
            server_id: self.server_id()?,
        })
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
    async fn index(&self) -> ProviderResult<()>;
    async fn password_auth(&mut self, uname: String, psw: String) -> ProviderResult<String>;
    async fn invalidate(&mut self) -> ProviderResult<()>;
    async fn add_to_db(&self) -> ProviderResult<()> {
        let provider = ActiveModel {
            user_id: Set(self.user_id()?),
            server_id: Set(self.server_id()?),
            ty: Set(self.ty()),
            url: Set(self.url()?.to_string()),
        };

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
            Err(_) => Err(ProviderError::FailedDbInsertError),
        }
    }
    async fn remove_from_db(&self) -> ProviderResult<()> {
        let res = Providers::delete_by_user_id(self.user_id()?)
            .exec(&get_conn().await?)
            .await;

        match res {
            Ok(_) => Ok(()),
            Err(_) => Err(ProviderError::FailedDbRemoveError),
        }
    }
}

clone_trait_object!(Provider);

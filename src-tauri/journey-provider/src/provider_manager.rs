use std::collections::hash_map::Values;

use anyhow::Result;
use async_trait::async_trait;
use futures::TryStreamExt;
use inherent::inherent;
use rapidhash::RapidHashMap;
use serde::Serialize;
use specta::Type;
use thiserror::Error;
use tracing::info;

use crate::{
    ProviderError,
    indexer::{Indexer, IndexerError},
    indexer_manager::{IndexerManager, IndexerManagerError},
    jellyfin_provider::JellyfinProvider,
    provider::{NewProvider, Provider},
};
use journey_db::{
    entity::{ProviderDTO, ProviderKey, ProviderVariant, Providers, providers},
    get_conn,
    sea_orm::{EntityTrait, IntoActiveModel},
};

#[derive(Debug, Error, Serialize, Type)]
pub enum ProviderManagerError {
    #[error(r#"ProviderVariant is "Unknown" & value is not Set on ActiveModel."#)]
    UnknownProviderError,
    #[error("No providers registered yet. Please add some first")]
    NoProviderError,
    #[error("Could not register provider, might be unauthenticated.")]
    RegisterError,
    #[error("Provider for this user on this server is already in use.")]
    ProviderInUseError,
    #[error("Provider is not registered, can not unregister.")]
    DeregisterError,
    #[error("Could not acquire database stream of provider values.")]
    FailedDbStreamError(String),
    #[error("Failed to index the given provider.")]
    FailedIndexingError,
    #[error("Provider has not been authenticated yet")]
    NotAuthenticatedError(String),
    #[error(transparent)]
    ProviderError(#[from] ProviderError),
    #[error(transparent)]
    IndexerError(#[from] IndexerError),
    #[error(transparent)]
    IndexerManagerError(#[from] IndexerManagerError),
    #[error(transparent)]
    JourneyDbError(#[from] journey_db::JourneyDbError),
}

pub type ProviderManagerResult<T> = Result<T, ProviderManagerError>;

#[async_trait]
pub trait RequiredForProviderManager {
    fn get_variant(
        &self,
        key: &ProviderKey,
    ) -> ProviderManagerResult<&Box<dyn Provider + Send + Sync>>;
    fn get_variants_values(&self) -> Values<'_, ProviderKey, Box<dyn Provider + Send + Sync>>;
    fn get_indexer_manager(&mut self) -> &mut IndexerManager;
    fn provider_exists(&self, key: &ProviderKey) -> ProviderManagerResult<bool>;
    fn register(&mut self, provider: Box<dyn Provider + Send + Sync>) -> ProviderManagerResult<()>;
    async fn deregister(&mut self, key: &ProviderKey) -> ProviderManagerResult<()>;
}

#[async_trait]
pub trait ProviderManagerFn: RequiredForProviderManager + Sync {
    fn get_type(
        &self,
        ty: &ProviderVariant,
        model: impl IntoActiveModel<providers::ActiveModelEx>,
    ) -> ProviderManagerResult<Box<dyn Provider + Send + Sync>> {
        match ty {
            ProviderVariant::JellyfinProvider => {
                Ok(JellyfinProvider::new(model.into_active_model()))
            }
            ProviderVariant::Unknown => Err(ProviderManagerError::UnknownProviderError),
        }
    }
    fn get_provider(&self, key: &ProviderKey) -> ProviderManagerResult<ProviderDTO> {
        let provider = self.get_variant(key)?;

        let provider_dto = ProviderDTO::builder()
            .authenticated(provider.authenticated()?)
            .ty(provider.ty()?)
            .url(provider.url()?)
            .key(provider.key()?)
            .build();

        Ok(provider_dto)
    }
    fn get_providers(&self) -> ProviderManagerResult<Vec<ProviderDTO>> {
        let mut providers: Vec<ProviderDTO> = vec![];

        for provider in self.get_variants_values() {
            let new = ProviderDTO::builder()
                .ty(provider.ty()?)
                .key(provider.key()?)
                .build();
            providers.push(new);
        }
        Ok(providers)
    }
    fn get_indexers(&self) -> ProviderManagerResult<Vec<Box<dyn Indexer + Send + Sync>>> {
        let mut indexers = vec![];
        for provider in self.get_variants_values() {
            let indexer = match provider.authenticated() {
                Ok(true) => {
                    info!(
                        "Beginning indexing on provider: {} for: {}",
                        provider.ty()?,
                        provider.url()?
                    );
                    Ok(provider.get_indexer()?)
                }
                Ok(_) => Err(ProviderManagerError::NotAuthenticatedError("".into())),
                Err(err) => Err(ProviderManagerError::NotAuthenticatedError(err.to_string())),
            }?;

            indexers.push(indexer);
        }

        Ok(indexers)
    }
    fn start_indexing(&mut self) -> ProviderManagerResult<()> {
        let indexers = self.get_indexers()?;
        let indexer_manager = self.get_indexer_manager();

        for indexer in indexers {
            indexer_manager.register(indexer)?;
        }

        Ok(())
    }
    async fn init(&mut self) -> ProviderManagerResult<()> {
        let conn = &get_conn().await?;
        let mut known_providers = match Providers::find().stream(conn).await {
            Ok(known) => Ok(known),
            Err(err) => Err(ProviderManagerError::FailedDbStreamError(err.to_string())),
        }?;

        while let Ok(Some(known)) = known_providers.try_next().await {
            let ty = known.ty;
            let new_provider = self.get_type(&ty, known.into_ex())?;
            self.register(new_provider)?;
        }

        self.start_indexing()?;
        Ok(())
    }
    async fn password_auth(
        &mut self,
        url: String,
        ty: ProviderVariant,
        uname: String,
        psw: String,
    ) -> ProviderManagerResult<ProviderKey> {
        let model = providers::ActiveModelEx::new().set_url(url).set_ty(ty);
        let mut provider = self.get_type(&ty, model)?;

        let token = provider.password_auth(uname, psw).await?;
        self.validate_provider(token, &provider).await?;

        let key = provider.key()?;
        self.register(provider)?;
        Ok(key)
    }
    async fn validate_provider(
        &self,
        token: String,
        provider: &Box<dyn Provider + Send + Sync>,
    ) -> ProviderManagerResult<()> {
        match provider.authenticated()? && self.provider_exists(&provider.key()?)? {
            false => Ok(()),
            true => Err(ProviderManagerError::ProviderInUseError),
        }?;

        provider.save_token(&token)?;
        provider.add_to_db().await?;
        Ok(())
    }
}

#[derive(Default, Debug)]
pub struct ProviderManager {
    variants: RapidHashMap<ProviderKey, Box<dyn Provider + Send + Sync>>,
    indexer_manager: IndexerManager,
}

#[async_trait]
#[inherent]
impl RequiredForProviderManager for ProviderManager {
    pub fn get_variant(
        &self,
        key: &ProviderKey,
    ) -> ProviderManagerResult<&Box<dyn Provider + Send + Sync>> {
        match self.variants.get(key) {
            Some(variant) => Ok(variant),
            None => Err(ProviderManagerError::NoProviderError),
        }
    }
    pub fn get_variants_values(&self) -> Values<'_, ProviderKey, Box<dyn Provider + Send + Sync>> {
        self.variants.values()
    }
    pub fn get_indexer_manager(&mut self) -> &mut IndexerManager {
        &mut self.indexer_manager
    }
    pub fn provider_exists(&self, key: &ProviderKey) -> ProviderManagerResult<bool> {
        Ok(self.variants.contains_key(key))
    }
    pub fn register(
        &mut self,
        provider: Box<dyn Provider + Send + Sync>,
    ) -> ProviderManagerResult<()> {
        self.variants.insert(provider.key()?, provider);
        Ok(())
    }
    pub async fn deregister(&mut self, key: &ProviderKey) -> ProviderManagerResult<()> {
        match self.variants.remove(key) {
            Some(mut provider) => {
                provider.remove_from_db().await?;
                provider.remove_token()?;
                Ok(provider.invalidate()?)
            }
            None => Err(ProviderManagerError::DeregisterError),
        }
    }
}

impl ProviderManagerFn for ProviderManager {}

#[cfg(test)]
mod provider_manager_test {
    use crate::ProviderManagerFn;
    use crate::jellyfin_provider::JellyfinProvider;
    use crate::provider::{NewProvider, Provider};
    use crate::provider_manager::ProviderManager;
    use journey_db::entity::ProviderVariant;
    use journey_db::entity::providers::{self};
    use journey_utils::constants::PRODUCT_NAME;
    use journey_utils::get_env_local;
    use serial_test::serial;
    use tracing::warn;
    use url::Url;

    #[tokio::test]
    #[ignore]
    async fn hash_no_login_failure() {
        let params =
            providers::ActiveModelEx::new().set_url(Url::parse("http://smth.example.com").unwrap());
        let provider = JellyfinProvider::new(params);

        let hash = provider.key();
        assert!(hash.is_err())
    }

    #[tokio::test]
    #[ignore]
    #[serial]
    async fn try_provider_manager_flow() {
        let env_map = get_env_local();

        let env_map = env_map.unwrap();
        journey_keyring::use_native_store().unwrap();

        warn!("{}", PRODUCT_NAME);
        let url = env_map.var("TEST_JELLYFIN_URL").unwrap();

        let mut provider_manager = ProviderManager::default();
        let key = provider_manager
            .password_auth(
                url,
                ProviderVariant::JellyfinProvider,
                env_map.var("TEST_JELLYFIN_USER").unwrap(),
                env_map.var("TEST_JELLYFIN_PW").unwrap(),
            )
            .await
            .unwrap();

        provider_manager.deregister(&key).await.unwrap();
        journey_keyring::release_store();
    }
}

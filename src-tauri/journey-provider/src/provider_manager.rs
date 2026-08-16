use crate::{
    ProviderError, ProviderResult,
    jellyfin_provider::JellyfinProvider,
    provider::{NewProvider, Provider},
};
use anyhow::Result;
use async_trait::async_trait;
use futures::TryStreamExt;
use inherent::inherent;
use journey_db::{
    entity::{ProviderDTO, ProviderKey, ProviderVariant, Providers, providers::ActiveModel},
    get_conn,
    sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait, IntoActiveModel},
};
use rapidhash::RapidHashMap;
use serde::Serialize;
use specta::Type;
use std::collections::hash_map::{Keys, Values};
use std::pin::Pin;
use thiserror::Error;

#[derive(Debug, Error, Serialize, Type)]
pub enum ProviderManagerError {
    #[error("No providers registered yet. Please add some first")]
    NoProviderError,
    #[error("Could not register provider, might be unauthenticated.")]
    RegisterError,
    #[error("Provider for this user on this server is already in use.")]
    ProviderInUseError,
    #[error("Provider is not registered, can not unregister.")]
    DeregisterError,
    #[error("Could not acquire database stream of provider values.")]
    FailedDbStreamError,
    #[error("Failed to index the given provider.")]
    FailedIndexingError,
    #[error(transparent)]
    ProviderError(#[from] ProviderError),
    #[error(transparent)]
    JourneyDbError(#[from] journey_db::JourneyDbError),
}

pub type ProviderManagerResult<T> = Result<T, ProviderManagerError>;

#[async_trait]
pub trait ProviderManagerFn: RequiredForProviderManager + Sync {
    fn get_type(
        &self,
        ty: ProviderVariant,
        model: ActiveModel,
    ) -> Result<Box<dyn Provider + Send + Sync>, ProviderManagerError> {
        let provider: Result<Box<dyn Provider + Send + Sync>, ProviderManagerError> = match ty {
            ProviderVariant::JellyfinProvider => Ok(JellyfinProvider::new(model)?),
        };

        Ok(provider?)
    }
    async fn init(&mut self) -> ProviderManagerResult<()> {
        let conn = &get_conn().await?;
        let mut known_providers = match Providers::find().stream(conn).await {
            Ok(known) => known,
            Err(_) => return Err(ProviderManagerError::FailedDbStreamError),
        };

        while let Ok(Some(known)) = known_providers.try_next().await {
            let new_provider = self.get_type(known.ty.clone(), known.into_active_model())?;
            self.register(new_provider)?;
        }

        self.start_indexing().await?;
        Ok(())
    }
    fn get_provider(&self, key: &ProviderKey) -> ProviderManagerResult<ProviderDTO> {
        let provider = self.get_variant(key)?;

        let provider_dto = ProviderDTO::builder()
            .authenticated(provider.authenticated()?)
            .ty(provider.ty())
            .url(provider.url()?)
            .key(provider.key()?)
            .build();

        Ok(provider_dto)
    }
    async fn get_providers(&self) -> ProviderManagerResult<Vec<ProviderDTO>> {
        let mut providers: Vec<ProviderDTO> = vec![];

        for provider in self.get_variants_values()? {
            let new = ProviderDTO::builder()
                .ty(provider.ty())
                .key(provider.key()?)
                .build();
            providers.push(new);
        }
        Ok(providers)
    }
    async fn validate_provider(
        &mut self,
        token: String,
        provider: &Box<dyn Provider + Send + Sync>,
    ) -> ProviderManagerResult<()> {
        match provider.authenticated()? && self.provider_exists(&provider.key()?)? {
            true => return Err(ProviderManagerError::ProviderInUseError),
            false => (),
        };

        provider.save_token(&token)?;
        provider.add_to_db().await?;
        Ok(())
    }
    async fn password_auth(
        &mut self,
        url: String,
        ty: ProviderVariant,
        uname: String,
        psw: String,
    ) -> ProviderManagerResult<ProviderKey> {
        let model = ActiveModel {
            url: Set(url),
            ..ActiveModel::default_values()
        };

        let mut provider = self.get_type(ty, model)?;
        let token = provider.password_auth(uname, psw).await?;
        self.validate_provider(token, &provider).await?;

        let key = provider.key()?;
        self.register(provider)?;
        Ok(key)
    }
    async fn start_indexing(&self) -> ProviderManagerResult<()> {
        for key in self.get_variants_keys()? {
            let _indexing_task = self.index(key);
        }
        Ok(())
    }
    async fn index(
        &self,
        key: &ProviderKey,
    ) -> ProviderManagerResult<Pin<Box<dyn Future<Output = ProviderResult<()>> + Send + 'life0>>>
    {
        Ok(self.get_variant(key)?.index())
    }
}

#[async_trait]
pub trait RequiredForProviderManager {
    fn get_variant(
        &self,
        key: &ProviderKey,
    ) -> ProviderManagerResult<&Box<dyn Provider + Send + Sync>>;
    fn get_variants_values(
        &self,
    ) -> ProviderManagerResult<Values<'_, ProviderKey, Box<dyn Provider + Send + Sync>>>;
    fn get_variants_keys(
        &self,
    ) -> ProviderManagerResult<Keys<'_, ProviderKey, Box<dyn Provider + Send + Sync>>>;
    fn provider_exists(&self, key: &ProviderKey) -> ProviderManagerResult<bool>;
    fn register(&mut self, provider: Box<dyn Provider + Send + Sync>) -> ProviderManagerResult<()>;
    async fn deregister(&mut self, key: &ProviderKey) -> ProviderManagerResult<()>;
}

#[derive(Default, Clone, Debug)]
pub struct ProviderManager {
    pub(crate) variants: RapidHashMap<ProviderKey, Box<dyn Provider + Send + Sync>>,
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
    pub fn get_variants_values(
        &self,
    ) -> ProviderManagerResult<Values<'_, ProviderKey, Box<dyn Provider + Send + Sync>>> {
        Ok(self.variants.values())
    }
    pub fn get_variants_keys(
        &self,
    ) -> ProviderManagerResult<Keys<'_, ProviderKey, Box<dyn Provider + Send + Sync>>> {
        Ok(self.variants.keys())
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
            Some(mut provider) => Ok(provider.invalidate().await?),
            None => Err(ProviderManagerError::DeregisterError),
        }
    }
}

#[async_trait]
#[inherent]
impl ProviderManagerFn for ProviderManager {}

#[cfg(test)]
mod provider_manager_test {
    use crate::ProviderManagerFn;
    use crate::jellyfin_provider::JellyfinProvider;
    use crate::provider::{NewProvider, Provider};
    use crate::provider_manager::ProviderManager;
    use journey_db::entity::providers::ActiveModel;
    use journey_db::sea_orm::ActiveModelTrait;
    use journey_db::sea_orm::ActiveValue::Set;
    use journey_utils::constants::PRODUCT_NAME;
    use journey_utils::get_env_local;
    use serial_test::serial;
    use test_log::test;
    use tracing::warn;
    use url::Url;

    #[test]
    fn hash_no_login_failure() {
        let params = ActiveModel {
            url: Set(Url::parse("http://smth.example.com").unwrap().into()),
            ..Default::default()
        };

        let provider = JellyfinProvider::new(params).unwrap();

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

        let params = ActiveModel {
            url: Set(Url::parse(&url).unwrap().into()),
            ..ActiveModel::default_values()
        };

        let mut provider = JellyfinProvider::new(params).unwrap();

        let access_token = provider
            .password_auth(
                env_map.var("TEST_JELLYFIN_USER").unwrap(),
                env_map.var("TEST_JELLYFIN_PW").unwrap(),
            )
            .await
            .unwrap();
        assert!(provider.authenticated().unwrap() == false);
        provider.save_token(&access_token).unwrap();
        assert!(provider.authenticated().unwrap() == true);

        let mut provider_manager = ProviderManager::default();

        let key = provider.key().unwrap();
        provider_manager.register(provider).unwrap();

        let provider = provider_manager.get_provider(&key).unwrap();

        warn!("key: {:#?}", provider.key);
        provider_manager.deregister(&key).await.unwrap();

        journey_keyring::release_store();
    }
}

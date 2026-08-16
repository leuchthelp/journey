use crate::ProviderError;
use crate::jellyfin_provider::JellyfinProvider;
use crate::provider::Provider;
use crate::provider::ProviderNew;
use anyhow::Result;
use async_trait::async_trait;
use futures::TryStreamExt;
use inherent::inherent;
use journey_db::entity::ProviderDTO;
use journey_db::entity::ProviderVariant;
use journey_db::entity::Providers;
use journey_db::get_conn;
use journey_db::sea_orm::ActiveModelTrait;
use journey_db::sea_orm::EntityTrait;
use journey_db::sea_orm::IntoActiveModel;
use journey_db::{entity::providers::ActiveModel, sea_orm::ActiveValue::Set};
use rapidhash::RapidHashMap;
use serde::Serialize;
use specta::Type;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error, Serialize, Type)]
pub enum ProviderManagerError {
    #[error("No providers registered yet. Please add some first")]
    NoProviderError,
    #[error("Could not register provider, might be unauthenticated.")]
    RegisterError,
    #[error("Provider is not registered, can not unregister.")]
    DeregisterError,
    #[error("Could not acquire database stream of provider values.")]
    FailedDbStreamError,
    #[error(transparent)]
    ProviderError(#[from] ProviderError),
    #[error(transparent)]
    JourneyDbError(#[from] journey_db::JourneyDbError),
}

pub type ProviderManagerResult<T> = Result<T, ProviderManagerError>;

#[async_trait]
pub trait ProviderManagerFn {
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
        Ok(())
    }
    fn get_provider(&self, key: &(Uuid, Uuid)) -> ProviderManagerResult<ProviderDTO>;
    async fn get_providers(&self) -> ProviderManagerResult<Vec<ProviderDTO>>;
    async fn password_auth(
        &mut self,
        url: String,
        ty: ProviderVariant,
        uname: String,
        psw: String,
    ) -> ProviderManagerResult<(Uuid, Uuid)> {
        let model = ActiveModel {
            url: Set(url),
            ..ActiveModel::default_values()
        };

        let mut provider = self.get_type(ty, model)?;
        match provider.password_auth(uname, psw).await {
            Ok(token) => {
                provider.save_token(&token)?;
                provider.add_to_db().await?;
            }
            Err(_) => (),
        }

        let key = provider.key()?;
        self.register(provider)?;
        Ok(key)
    }
    fn register(&mut self, provider: Box<dyn Provider + Send + Sync>) -> ProviderManagerResult<()>;
    async fn deregister(&mut self, key: &(Uuid, Uuid)) -> ProviderManagerResult<()>;
}

#[derive(Default, Clone, Debug)]
pub struct ProviderManager {
    pub(crate) variants: RapidHashMap<(Uuid, Uuid), Box<dyn Provider + Send + Sync>>,
}

#[async_trait]
#[inherent]
impl ProviderManagerFn for ProviderManager {
    pub fn get_provider(&self, key: &(Uuid, Uuid)) -> ProviderManagerResult<ProviderDTO> {
        let provider = match self.variants.get(key) {
            Some(provider) => Ok(provider),
            None => Err(ProviderManagerError::NoProviderError),
        }?;

        let provider_dto = ProviderDTO::builder()
            .authenticated(provider.authenticated()?)
            .ty(provider.ty())
            .url(provider.url()?)
            .user_id(provider.user_id()?)
            .server_id(provider.server_id()?)
            .build();

        Ok(provider_dto)
    }
    pub async fn get_providers(&self) -> ProviderManagerResult<Vec<ProviderDTO>> {
        let mut providers: Vec<ProviderDTO> = vec![];

        for (_, provider) in &self.variants {
            let new = ProviderDTO::builder()
                .authenticated(provider.authenticated()?)
                .ty(provider.ty())
                .url(provider.url()?)
                .user_id(provider.user_id()?)
                .server_id(provider.server_id()?)
                .build();
            providers.push(new);
        }
        Ok(providers)
    }
    pub fn register(
        &mut self,
        provider: Box<dyn Provider + Send + Sync>,
    ) -> ProviderManagerResult<()> {
        self.variants.insert(provider.key()?, provider);
        Ok(())
    }
    pub async fn deregister(&mut self, key: &(Uuid, Uuid)) -> ProviderManagerResult<()> {
        match self.variants.remove(key) {
            Some(mut provider) => Ok(provider.invalidate().await?),
            None => Err(ProviderManagerError::DeregisterError),
        }
    }
}

#[cfg(test)]
mod provider_manager_test {
    use crate::jellyfin_provider::JellyfinProvider;
    use crate::provider::{Provider, ProviderNew};
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

        warn!(
            "user_id: {:#?}, server_id: {:#?}",
            provider.user_id, provider.server_id
        );
        provider_manager.deregister(&key).await.unwrap();

        journey_keyring::release_store();
    }
}

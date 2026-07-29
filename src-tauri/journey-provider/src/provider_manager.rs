use anyhow::Result;
use async_trait::async_trait;
use futures::TryStreamExt;
use inherent::inherent;
use journey_db::entity::ProviderDTO;
use journey_db::entity::Providers;
use journey_db::get_conn;
use journey_db::sea_orm::ActiveModelTrait;
use journey_db::sea_orm::EntityTrait;
use journey_db::sea_orm::IntoActiveModel;
use journey_db::{entity::providers::ActiveModel, sea_orm::ActiveValue::Set};
use std::any::type_name;
use std::collections::HashMap;
use thiserror::Error;
use tracing::warn;

use crate::ProviderResult;
use crate::jellyfin_provider::JellyfinProvider;
use crate::provider::Provider;
use crate::provider::ProviderError;
use crate::provider::ProviderNew;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Error, Serialize, Deserialize, Type)]
pub enum ProviderManagerError {
    #[error("No providers registered yet. Please add some first")]
    NoProviderError,
    #[error("Could not register provider, might be unauthenticated.")]
    RegisterError,
    #[error("Provider is not registered, can not unregister.")]
    DeregisterError,
    #[error("Provider kind is not known")]
    UnknownProviderKindError,
    #[error(transparent)]
    ProviderError(#[from] ProviderError),
    #[error(transparent)]
    #[serde(skip)]
    ParseUrlError(#[from] url::ParseError),
    #[error(transparent)]
    #[serde(skip)]
    DbError(#[from] journey_db::sea_orm::DbErr),
    #[error(transparent)]
    JourneyDbError(#[from] journey_db::JourneyDbError),
}

pub type ProviderManagerResult<T> = Result<T, ProviderManagerError>;

#[async_trait]
pub trait ProviderManagerFn {
    async fn init(&mut self) -> ProviderManagerResult<()> {
        let conn = &get_conn().await?;
        let mut known_providers = Providers::find().stream(conn).await?;

        while let Some(known) = known_providers.try_next().await? {
            let new_provider: ProviderResult<Box<dyn Provider + Send + Sync>> =
                match known.kind.as_str() {
                    value if value == type_name::<JellyfinProvider>() => {
                        Ok(JellyfinProvider::new(known.into_active_model())?)
                    }
                    _ => return Err(ProviderManagerError::UnknownProviderKindError),
                };

            let new_provider = new_provider?;
            self.register(new_provider)?;
        }
        Ok(())
    }
    fn get_provider(&self, key: &u64) -> ProviderManagerResult<ProviderDTO>;
    async fn get_providers(&self) -> ProviderManagerResult<Vec<ProviderDTO>>;
    async fn password_auth(
        &mut self,
        url: String,
        kind: String,
        uname: String,
        psw: String,
    ) -> ProviderManagerResult<()> {
        let model = ActiveModel {
            url: Set(url),
            ..ActiveModel::default_values()
        };

        warn!("{}", kind);

        let provider: Result<Box<dyn Provider + Send + Sync>, ProviderManagerError> = match kind
            .as_str()
        {
            value if value == type_name::<JellyfinProvider>() => Ok(JellyfinProvider::new(model)?),
            _ => return Err(ProviderManagerError::UnknownProviderKindError),
        };

        let mut provider = provider?;
        provider.password_auth(uname, psw).await?;
        self.register(provider)?;
        Ok(())
    }
    fn register(&mut self, provider: Box<dyn Provider + Send + Sync>) -> ProviderManagerResult<()>;
    async fn deregister(&mut self, key: &u64) -> ProviderManagerResult<()>;
}

#[derive(Default, Clone, Debug)]
pub struct ProviderManager {
    pub(crate) variants: HashMap<u64, Box<dyn Provider + Send + Sync>>,
}

#[async_trait]
#[inherent]
impl ProviderManagerFn for ProviderManager {
    pub fn get_provider(&self, key: &u64) -> ProviderManagerResult<ProviderDTO> {
        let provider = self.variants.get(key);

        if provider.is_none() {
            return Err(ProviderManagerError::NoProviderError);
        }

        let provider = provider.unwrap();
        let provider = ProviderDTO::builder()
            .authenticated(provider.authenticated()?)
            .hash(provider.hash()?.try_into().unwrap())
            .kind(provider.type_())
            .build();

        Ok(provider)
    }
    pub async fn get_providers(&self) -> ProviderManagerResult<Vec<ProviderDTO>> {
        let mut providers: Vec<ProviderDTO> = vec![];

        for (_, provider) in &self.variants {
            let new = ProviderDTO::builder()
                .authenticated(provider.authenticated()?)
                .hash(provider.hash()?.try_into().unwrap())
                .kind(provider.type_())
                .url(provider.url()?)
                .build();
            providers.push(new);
        }
        Ok(providers)
    }
    pub fn register(
        &mut self,
        provider: Box<dyn Provider + Send + Sync>,
    ) -> ProviderManagerResult<()> {
        if provider.authenticated()? {
            self.variants.insert(provider.hash()?, provider);
            return Ok(());
        }
        Err(ProviderManagerError::RegisterError)
    }
    pub async fn deregister(&mut self, key: &u64) -> ProviderManagerResult<()> {
        let provider = self.variants.remove(key);

        match provider {
            Some(mut provider) => Ok(provider.invalidate().await?),
            None => return Err(ProviderManagerError::DeregisterError),
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

        let hash = provider.hash();
        assert!(hash.is_err())
    }

    #[tokio::test]
    #[ignore]
    #[serial]
    async fn try_provider_manager_flow() {
        let env_map = get_env_local();

        let env_map = env_map.unwrap();
        journey_keyring::use_native_store().unwrap();

        warn!("{}", env_map.var("TEST_JELLYFIN_URL").unwrap());
        let url = env_map.var("TEST_JELLYFIN_URL").unwrap();

        let params = ActiveModel {
            url: Set(Url::parse(&url).unwrap().into()),
            ..ActiveModel::default_values()
        };

        let mut provider = JellyfinProvider::new(params).unwrap();

        provider
            .password_auth(
                env_map.var("TEST_JELLYFIN_USER").unwrap(),
                env_map.var("TEST_JELLYFIN_PW").unwrap(),
            )
            .await
            .unwrap();

        let mut provider_manager = ProviderManager::default();

        let key = provider.hash().unwrap();
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

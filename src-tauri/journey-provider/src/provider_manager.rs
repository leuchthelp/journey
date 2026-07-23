use anyhow::Result;
use async_trait::async_trait;
use journey_db::entity::providers::Model;
use std::any::type_name;
use std::collections::HashMap;
use thiserror::Error;
use url::Url;

use crate::ProviderParams;
use crate::ProviderResult;
use crate::jellyfin_provider::JellyfinProvider;
use crate::provider::Provider;
use crate::provider::ProviderError;
use crate::provider::ProviderNew;

#[derive(Error, Debug)]
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
    ParseUrlError(#[from] url::ParseError),
}

pub type ProviderManagerResult<T> = Result<T, ProviderManagerError>;

#[async_trait]
pub trait ProviderManagerFn {
    fn init(&mut self, known_providers: Vec<Model>) -> ProviderManagerResult<()> {
        for known in known_providers {
            let params = ProviderParams {
                url: Url::parse(&known.url)?,
                user_id: Some(known.user_id),
                server_id: Some(known.server_id),
            };

            let new_provider: ProviderResult<Box<dyn Provider + Send>> = match known.kind.as_str() {
                value if value == type_name::<JellyfinProvider>() => {
                    Ok(JellyfinProvider::new(params)?)
                }
                _ => return Err(ProviderManagerError::UnknownProviderKindError),
            };

            let new_provider = new_provider?;
            self.register(new_provider)?;
        }
        Ok(())
    }
    fn get_providers(&self) -> ProviderManagerResult<&HashMap<u64, Box<dyn Provider + Send>>>;
    fn get_provider(&self, key: &u64) -> ProviderManagerResult<&Box<dyn Provider + Send>>;
    fn register(&mut self, provider: Box<dyn Provider + Send>) -> ProviderManagerResult<()>;
    async fn deregister(&mut self, key: &u64) -> ProviderManagerResult<()>;
}

#[derive(Default)]
pub struct ProviderManager {
    variants: HashMap<u64, Box<dyn Provider + Send>>,
}

#[async_trait]
impl ProviderManagerFn for ProviderManager {
    fn get_providers(&self) -> ProviderManagerResult<&HashMap<u64, Box<dyn Provider + Send>>> {
        if self.variants.is_empty() {
            return Err(ProviderManagerError::NoProviderError);
        }
        Ok(&self.variants)
    }
    fn get_provider(&self, key: &u64) -> ProviderManagerResult<&Box<dyn Provider + Send>> {
        let provider = self.variants.get(key);

        if provider.is_none() {
            return Err(ProviderManagerError::NoProviderError);
        }

        Ok(provider.unwrap())
    }
    fn register(&mut self, provider: Box<dyn Provider + Send>) -> ProviderManagerResult<()> {
        if provider.authenticated()? {
            self.variants.insert(provider.hash()?, provider);
            return Ok(());
        }
        Err(ProviderManagerError::RegisterError)
    }
    async fn deregister(&mut self, key: &u64) -> ProviderManagerResult<()> {
        let provider = self.variants.remove(key);

        if provider.is_some() {
            let mut provider = provider.unwrap();
            provider.invalidate().await?;
            return Ok(());
        }
        Err(ProviderManagerError::DeregisterError)
    }
}

#[cfg(test)]
mod provider_manager_test {
    use crate::jellyfin_provider::JellyfinProvider;
    use crate::provider::{Provider, ProviderNew, ProviderParams};
    use crate::provider_manager::{ProviderManager, ProviderManagerFn};
    use journey_utils::get_env_local;
    use serial_test::serial;
    use test_log::test;
    use tracing::warn;
    use url::Url;

    #[test]
    fn hash_no_login_failure() {
        let provider = JellyfinProvider::new(ProviderParams {
            url: Url::parse("http://smth.example.com").unwrap(),
            user_id: None,
            server_id: None,
        })
        .unwrap();

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

        let mut provider = JellyfinProvider::new(ProviderParams {
            url: Url::parse(&url).unwrap(),
            user_id: None,
            server_id: None,
        })
        .unwrap();

        provider
            .authenticate_with_pw(
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
            "user_id: {}, server_id: {}",
            provider.user_id().unwrap(),
            provider.server_id().unwrap()
        );
        provider_manager.deregister(&key).await.unwrap();

        journey_keyring::release_store();
    }
}

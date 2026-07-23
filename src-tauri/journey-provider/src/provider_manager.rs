use anyhow::Result;
use std::collections::HashMap;
use thiserror::Error;

use crate::provider::Provider;
use crate::provider::ProviderError;

#[derive(Error, Debug)]
pub enum ProviderManagerError {
    #[error("No providers registered yet. Please add some first")]
    NoProviderError,
    #[error("Provider is not registered, can not unregister.")]
    UnregisterError,
    #[error("Could not destroy provider.")]
    DestroyError,
    #[error(transparent)]
    ProviderError(#[from] ProviderError),
}

pub type ProviderManagerResult<T> = Result<T, ProviderManagerError>;

pub trait ProviderManagerFn {
    fn get_providers(&self) -> ProviderManagerResult<&HashMap<u64, Box<dyn Provider>>>;
    fn get_provider(&self, key: &u64) -> ProviderManagerResult<&Box<dyn Provider>>;
    fn register(&mut self, provider: Box<dyn Provider>) -> ProviderManagerResult<()>;
    fn deregister(&mut self, key: &u64) -> ProviderManagerResult<()>;
    fn destroy(&mut self, key: &u64) -> ProviderManagerResult<()>;
}

#[derive(Default)]
pub struct ProviderManager {
    live: HashMap<u64, Box<dyn Provider>>,
    cold: HashMap<u64, Box<dyn Provider>>,
}

impl ProviderManagerFn for ProviderManager {
    fn get_providers(&self) -> ProviderManagerResult<&HashMap<u64, Box<dyn Provider>>> {
        if self.cold.is_empty() {
            return Err(ProviderManagerError::NoProviderError);
        }
        Ok(&self.cold)
    }
    fn get_provider(&self, key: &u64) -> ProviderManagerResult<&Box<dyn Provider>> {
        let provider = self.live.get(key);

        if provider.is_none() {
            return Err(ProviderManagerError::NoProviderError);
        }

        Ok(provider.unwrap())
    }

    fn register(&mut self, provider: Box<dyn Provider>) -> ProviderManagerResult<()> {
        self.live.insert(provider.hash()?, provider);
        Ok(())
    }

    fn deregister(&mut self, key: &u64) -> ProviderManagerResult<()> {
        let provider = self.live.remove(key);

        if provider.is_some() {
            let mut provider = provider.unwrap();

            // compute hash before, as .invalidate() strips user_id & server_id
            // which is required for .hash()
            let old_hash = provider.hash()?;
            provider.invalidate()?;
            self.cold.insert(old_hash, provider);
            return Ok(());
        }
        Err(ProviderManagerError::UnregisterError)
    }

    fn destroy(&mut self, key: &u64) -> ProviderManagerResult<()> {
        let key = self.cold.remove(key);

        if key.is_none() {
            return Err(ProviderManagerError::DestroyError);
        }
        Ok(())
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
        provider_manager.register(Box::new(provider)).unwrap();

        let provider = provider_manager.get_provider(&key).unwrap();

        warn!(
            "user_id: {}, server_id: {}",
            provider.user_id().unwrap(),
            provider.server_id().unwrap()
        );
        provider_manager.deregister(&key).unwrap();
        provider_manager.destroy(&key).unwrap();

        journey_keyring::release_store();
    }
}

mod jellyfin;
use anyhow::Result;
use async_trait::async_trait;
pub use jellyfin::helpers;
pub use jellyfin::jellyfin_provider;
use journey_keyring::Entry;
use journey_utils::get_env_prod;
use std::any::type_name_of_val;
use thiserror::Error;
use url::Url;
use uuid::Uuid;

pub type ProviderResult<T> = Result<T, anyhow::Error>;

pub trait ProviderNew<T> {
    fn new(params: ProviderParams) -> ProviderResult<T>;
}

#[async_trait]
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
    fn authenticated(&self) -> &bool;
    async fn authenticate_with_pw(&mut self, uname: String, psw: String) -> ProviderResult<()>;
}

pub struct ProviderParams {
    pub user_id: Option<Uuid>,
    pub server_id: Option<Uuid>,
    pub url: Url,
}

#[derive(Error, Debug)]
pub enum ProviderManagerError {
    #[error("No providers registered yet. Please add some first")]
    NoProvidersError,
    #[error("Provider is not requistered, can not unregister.")]
    UnregisterError,
    #[error("user_id not found, but provider was registered")]
    WronglyRegisteredError(#[from] anyhow::Error),
}

pub trait ProviderManagerFn {
    fn get_providers(&self) -> Result<&Vec<Box<dyn Provider>>, ProviderManagerError>;
    fn register(&mut self, provider: Box<dyn Provider>) -> Result<(), ProviderManagerError>;
    fn deregister(&mut self, provider: Box<dyn Provider>) -> Result<(), ProviderManagerError>;
}

#[derive(Default)]
pub struct ProviderManager {
    variants: Vec<Box<dyn Provider>>,
}

impl ProviderManagerFn for ProviderManager {
    fn get_providers(&self) -> Result<&Vec<Box<dyn Provider>>, ProviderManagerError> {
        if self.variants.is_empty() {
            return Err(ProviderManagerError::NoProvidersError);
        }
        Ok(&self.variants)
    }

    fn register(&mut self, provider: Box<dyn Provider>) -> Result<(), ProviderManagerError> {
        self.variants.push(provider);
        Ok(())
    }

    fn deregister(&mut self, provider: Box<dyn Provider>) -> Result<(), ProviderManagerError> {
        let mut index: Option<usize> = None;

        let needed = provider.user_id()?;
        for (i, variant) in self.variants.iter().enumerate() {
            let id = variant.user_id()?;
            if id == needed {
                index = Some(i);
                break;
            }
        }

        if index.is_some() {
            self.variants.swap_remove(index.unwrap());
            return Ok(());
        }
        Err(ProviderManagerError::UnregisterError)
    }
}

#[cfg(test)]
mod tests {
    use crate::jellyfin_provider::JellyfinProvider;
    use crate::{ProviderManager, ProviderManagerFn, ProviderNew, ProviderParams};
    use journey_utils::get_env_local;
    use serial_test::serial;
    use url::Url;

    #[test]
    #[serial]
    fn insert_providers() {
        let env_map = get_env_local();
        if env_map.is_err() {
            assert!(true)
        } else {
            let env_map = env_map.unwrap();
            journey_keyring::use_native_store().unwrap();

            println!("{}", env_map.var("TEST_JELLYFIN_URL").unwrap());
            let url = env_map.var("TEST_JELLYFIN_URL").unwrap();

            let provider = JellyfinProvider::new(ProviderParams {
                url: Url::parse(&url).unwrap(),
                user_id: None,
                server_id: None,
            })
            .unwrap();

            let mut provider_manager = ProviderManager::default();

            provider_manager.register(Box::new(provider)).unwrap();

            journey_keyring::release_store();
        }
    }
}

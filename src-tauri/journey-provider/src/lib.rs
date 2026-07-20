mod jellyfin;
use anyhow::anyhow;
use anyhow::{Error, Result};
use async_trait::async_trait;
pub use jellyfin::helpers;
pub use jellyfin::jellyfin_provider;
use journey_keyring::Entry;
use journey_utils::get_env_prod;
use std::any::type_name_of_val;
use url::Url;
use uuid::Uuid;

pub type ProviderResult<T> = Result<T, Error>;

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

pub trait ProviderManagerFn {
    fn get_providers(&self) -> Result<&Vec<Box<dyn Provider>>>;
}

pub struct ProviderManager {
    variants: Vec<Box<dyn Provider>>,
}

impl ProviderManagerFn for ProviderManager {
    fn get_providers(&self) -> Result<&Vec<Box<dyn Provider>>> {
        if self.variants.is_empty() {
            Err(anyhow!("fehler"))
        } else {
            Ok(&self.variants)
        }
    }
}

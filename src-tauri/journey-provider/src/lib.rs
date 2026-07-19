mod jellyfin;
use async_trait::async_trait;
pub use jellyfin::helpers;
pub use jellyfin::jellyfin_provider;
use std::any::type_name_of_val;
use url::Url;
use uuid::Uuid;

pub type ProviderResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

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
    fn authenticated(&self) -> &bool;
    async fn authenticate_with_pw(&mut self, uname: String, psw: String) -> ProviderResult<()>;
}

pub struct ProviderParams {
    pub user_id: Option<Uuid>,
    pub server_id: Option<Uuid>,
    pub url: Url,
}
pub struct Providers {
    pub providers: Vec<Box<dyn Provider>>,
}

impl Providers {}

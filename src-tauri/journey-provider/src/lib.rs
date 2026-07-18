pub mod jellyfin;
pub use jellyfin::helpers;
pub use jellyfin::jellyfin_provider;
use std::any::type_name_of_val;

use uuid::Uuid;
use std::error::Error;
use url::Url;

pub trait Provider {
    fn new(params: ProviderParams) -> Self;
    fn user_id(&self) -> Uuid;
    fn server_id(&self) -> Uuid;
    fn url(&self) -> &Url;
    fn type_(&self) -> String {
        return type_name_of_val(self).to_string();
    }
    async fn authenticate_with_pw(self, uname: String, psw: String) -> Result<(), Box<dyn Error>>;

    fn params(&self) -> &ProviderParams;
}
pub struct ProviderParams {
    pub user_id: Uuid,
    pub server_id: Uuid,
    pub url: Url,
}

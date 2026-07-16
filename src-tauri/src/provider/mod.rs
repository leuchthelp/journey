pub mod jellyfin;
pub use jellyfin::helpers;
pub use jellyfin::jellyfin_provider;
use std::any::type_name_of_val;

use url::Url;
use uuid::Uuid;

pub trait Provider {
    fn new(params: ProviderParams) -> Self;
    fn user_id(&self) -> Uuid;
    fn server_id(&self) -> Uuid;
    fn url(&self) -> &Url;
    fn type_(&self) -> String {
        return type_name_of_val(self).to_string();
    }

    fn params(&self) -> &ProviderParams;
}
pub struct ProviderParams {
    pub user_id: Uuid,
    pub server_id: Uuid,
    pub url: Url,
}

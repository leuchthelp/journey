mod jellyfin;
pub use jellyfin::helpers;
pub use jellyfin::jellyfin_provider;
use std::any::type_name_of_val;

use url::Url;
use uuid::Uuid;

pub trait Provider<E> {
    fn new(params: ProviderParams) -> Self;
    fn user_id(&self) -> Result<Uuid, E>;
    fn server_id(&self) -> Result<Uuid, E>;
    fn url(&self) -> &Url;
    fn type_(&self) -> String {
        return type_name_of_val(self).to_string();
    }
    fn authenticated(&self) -> bool;

    fn authenticate_with_pw(
        &mut self,
        uname: String,
        psw: String,
    ) -> impl std::future::Future<Output = Result<(), E>> + Send;
}
pub struct ProviderParams {
    pub user_id: Option<Uuid>,
    pub server_id: Option<Uuid>,
    pub url: Url,
}

use journey_db::entity::ProviderDTO;

#[taurpc::procedures]
pub trait ProviderApiImpl {
    async fn get_providers() -> ProviderDTO;
}

#[derive(Clone, Debug)]
pub struct ProviderApi;

#[taurpc::resolvers]
impl ProviderApiImpl for ProviderApi {
    async fn get_providers(self) -> ProviderDTO {
        return ProviderDTO::default();
    }
}

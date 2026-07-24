use journey_db::entity::ProviderDTO;

#[taurpc::procedures(path = "provider")]
pub trait ProviderApi {
    async fn get_providers() -> ProviderDTO;
}

#[derive(Clone, Debug)]
pub struct ProviderApiImpl;

#[taurpc::resolvers]
impl ProviderApi for ProviderApiImpl {
    async fn get_providers(self) -> ProviderDTO {
        return ProviderDTO::default();
    }
}

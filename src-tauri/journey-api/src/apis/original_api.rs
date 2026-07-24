use journey_db::entity::OriginalDTO;

#[taurpc::procedures]
pub trait OriginalApiImpl {
    async fn get_original() -> OriginalDTO;
}

#[derive(Clone, Debug)]
pub struct OriginalApi;

#[taurpc::resolvers]
impl OriginalApiImpl for OriginalApi {
    async fn get_original(self) -> OriginalDTO {
        return OriginalDTO::default();
    }
}

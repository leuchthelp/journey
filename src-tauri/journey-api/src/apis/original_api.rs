use journey_db::entity::OriginalDTO;

#[taurpc::procedures(path = "original")]
pub trait OriginalApi {
    async fn get_original() -> OriginalDTO;
}

#[derive(Clone, Debug)]
pub struct OriginalApiImpl;

#[taurpc::resolvers]
impl OriginalApi for OriginalApiImpl {
    async fn get_original(self) -> OriginalDTO {
        return OriginalDTO::default();
    }
}

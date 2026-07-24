use journey_db::entity::ContentDTO;

#[taurpc::procedures]
pub trait ContentApiImpl {
    async fn get_content() -> ContentDTO;
}

#[derive(Clone, Debug)]
pub struct ContentApi;

#[taurpc::resolvers]
impl ContentApiImpl for ContentApi {
    async fn get_content(self) -> ContentDTO {
        return ContentDTO::default();
    }
}

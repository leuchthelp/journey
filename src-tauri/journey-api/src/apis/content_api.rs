use journey_db::entity::ContentDTO;

#[taurpc::procedures(path = "content")]
pub trait ContentApi {
    async fn get_content() -> ContentDTO;
}

#[derive(Clone, Debug)]
pub struct ContentApiImpl;

#[taurpc::resolvers]
impl ContentApi for ContentApiImpl {
    async fn get_content(self) -> ContentDTO {
        return ContentDTO::default();
    }
}

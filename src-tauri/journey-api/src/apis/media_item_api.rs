use journey_db::entity::MediaItemDTO;
use taurpc;

#[taurpc::procedures]
pub trait MediaItemApiImpl {
    async fn get_media_items() -> MediaItemDTO;
}

#[derive(Clone, Debug)]
pub struct MediaItemApi;

#[taurpc::resolvers]
impl MediaItemApiImpl for MediaItemApi {
    async fn get_media_items(self) -> MediaItemDTO {
        return MediaItemDTO::default();
    }
}

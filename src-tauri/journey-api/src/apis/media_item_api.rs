use journey_db::entity::MediaItemDTO;
use taurpc;

#[taurpc::procedures(path = "mediaItem")]
pub trait MediaItemApi {
    async fn get_media_items() -> MediaItemDTO;
}

#[derive(Clone, Debug)]
pub struct MediaItemApiImpl;

#[taurpc::resolvers]
impl MediaItemApi for MediaItemApiImpl {
    async fn get_media_items(self) -> MediaItemDTO {
        return MediaItemDTO::default();
    }
}

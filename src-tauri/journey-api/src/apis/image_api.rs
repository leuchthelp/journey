use journey_db::entity::ImageDTO;

#[taurpc::procedures]
pub trait ImageApiImpl {
    async fn get_images() -> ImageDTO;
}

#[derive(Clone, Debug)]
pub struct ImageApi;

#[taurpc::resolvers]
impl ImageApiImpl for ImageApi {
    async fn get_images(self) -> ImageDTO {
        return ImageDTO::default();
    }
}

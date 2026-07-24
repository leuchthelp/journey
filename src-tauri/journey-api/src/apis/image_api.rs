use journey_db::entity::ImageDTO;

#[taurpc::procedures(path = "image")]
pub trait ImageApi {
    async fn get_images() -> ImageDTO;
}

#[derive(Clone, Debug)]
pub struct ImageApiImpl;

#[taurpc::resolvers]
impl ImageApi for ImageApiImpl {
    async fn get_images(self) -> ImageDTO {
        return ImageDTO::default();
    }
}

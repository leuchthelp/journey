use crate::entity::{
    MediaItemsDTO, ProviderDTO, media_items::ConvertableMediaItems, providers::ConvertableProvider,
};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "images")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    #[sea_orm(unique)]
    pub url: String,
    pub server_id: Option<Uuid>,
    #[sea_orm(belongs_to, from = "server_id", to = "server_id")]
    pub provider: BelongsTo<Option<super::providers::Entity>>,
    pub kind: String,
    #[sea_orm(has_many, via = "jt_media_item_to_image")]
    pub media_items: HasMany<super::media_items::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

pub trait ConvertableImage {
    fn from_model(item: ModelEx) -> ImagesDTO;
}

#[taurpc::ipc_type]
#[derive(Debug)]
pub struct ImagesDTO {
    pub url: String,
    pub server_id: Option<Uuid>,
    pub provider: Option<ProviderDTO>,
    pub kind: String,
    pub media_items: Option<Vec<MediaItemsDTO>>,
}

impl ConvertableImage for ImagesDTO {
    fn from_model(item: ModelEx) -> ImagesDTO {
        let provider = ProviderDTO::from_model(item.provider.unwrap().clone());
        let media_items = item
            .media_items
            .iter()
            .map(|f| MediaItemsDTO::from_model(f.clone()))
            .collect::<Vec<_>>();

        return ImagesDTO {
            url: item.url,
            server_id: item.server_id,
            provider: Some(provider),
            kind: item.kind,
            media_items: Some(media_items),
        };
    }
}

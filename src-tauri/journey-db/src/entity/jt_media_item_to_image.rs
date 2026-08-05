use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "jt_media_item_to_image")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub media_item_id: Uuid,
    #[sea_orm(primary_key, auto_increment = false)]
    pub image_url: String,
    #[sea_orm(belongs_to, from = "media_item_id", to = "uuid")]
    pub media_item: BelongsTo<super::media_items::Entity>,
    #[sea_orm(belongs_to, from = "image_url", to = "url")]
    pub image: BelongsTo<super::images::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

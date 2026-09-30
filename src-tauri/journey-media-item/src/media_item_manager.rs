use anyhow::Result;
use async_trait::async_trait;
use inherent::inherent;
use journey_db::{
    ConversionError, JourneyDbError,
    entity::{
        MediaItemDTO, content, images, jt_parent_to_child,
        media_items::{self, MediaItemType},
        providers, sources,
    },
    get_conn,
    sea_orm::{ColumnTrait, Condition, EntityLoaderTrait, QueryFilter},
};
use serde::Serialize;
use specta::Type;
use thiserror::Error;

#[derive(Debug, Error, Serialize, Type)]
pub enum MediaItemManagerError {
    #[error("Failed to retrieve MediaItems: {0}")]
    FailedItemRetrievalError(String),
    #[error(transparent)]
    JourneyDbError(#[from] JourneyDbError),
    #[error(transparent)]
    ConversionError(#[from] ConversionError),
}

pub type MediaItemManagerResult<T> = Result<T, MediaItemManagerError>;

#[async_trait]
pub trait RequiredForMediaItemManager {}

#[async_trait]
pub trait MediaItemManagerFn: RequiredForMediaItemManager + Sync {
    async fn get_items(
        &self,
        ty: MediaItemType,
        amount: u64,
    ) -> MediaItemManagerResult<Vec<MediaItemDTO>> {
        let conn = get_conn().await?;

        let filter_item_ty = media_items::Column::Ty.eq(ty);

        let filter = Condition::all().add(filter_item_ty);

        let mut paginator = media_items::Entity::load()
            .filter(filter)
            .with(providers::Entity)
            .with(content::Entity)
            .with(sources::Entity)
            .with(images::Entity)
            .with(jt_parent_to_child::Entity::REVERSE)
            .paginate(&conn, amount);

        let mut items = vec![];

        if let Ok(Some(models)) = paginator.fetch_and_next().await {
            for model in models {
                let dto = MediaItemDTO::from_model(model)?;
                items.push(dto);
            }
        }

        Ok(items)
    }
}

#[derive(Default, Debug)]
pub struct MediaItemManager {}

#[async_trait]
#[inherent]
impl RequiredForMediaItemManager for MediaItemManager {}

impl MediaItemManagerFn for MediaItemManager {}

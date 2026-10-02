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
    sea_orm::{ColumnTrait, EntityLoaderTrait, QueryFilter},
};
use serde::Serialize;
use specta::Type;
use thiserror::Error;
use uuid::Uuid;

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

        let mut paginator = media_items::Entity::load()
            .filter(media_items::Column::Ty.eq(ty))
            .with(content::Entity)
            .with(sources::Entity)
            .with(images::Entity)
            .paginate(&conn, amount);

        let mut items = vec![];

        match paginator.fetch_and_next().await {
            Ok(Some(models)) => {
                for model in models {
                    let dto = MediaItemDTO::from_model(model)?;
                    items.push(dto);
                }
            }
            Ok(None) => return Err(JourneyDbError::RecordNotFound(ty.to_string()).into()),
            Err(err) => return Err(JourneyDbError::ConnectionError(err.to_string()).into()),
        };

        Ok(items)
    }
    async fn get_item(
        &self,
        ty: MediaItemType,
        uuid: Uuid,
    ) -> MediaItemManagerResult<MediaItemDTO> {
        match media_items::Entity::load()
            .filter(media_items::Column::Ty.eq(ty))
            .with(providers::Entity)
            .with(content::Entity)
            .with(sources::Entity)
            .with(images::Entity)
            .with(jt_parent_to_child::Entity)
            .with(jt_parent_to_child::Entity::REVERSE)
            .one(&get_conn().await?)
            .await
        {
            Ok(Some(model)) => Ok(MediaItemDTO::from_model(model)?),
            Ok(None) => Err(JourneyDbError::RecordNotFound(uuid.to_string()).into()),
            Err(err) => Err(JourneyDbError::ConnectionError(err.to_string()).into()),
        }
    }
}

#[derive(Default, Debug)]
pub struct MediaItemManager {}

#[async_trait]
#[inherent]
impl RequiredForMediaItemManager for MediaItemManager {}

impl MediaItemManagerFn for MediaItemManager {}

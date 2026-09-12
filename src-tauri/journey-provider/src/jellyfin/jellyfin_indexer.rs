use std::fmt::Debug;

use async_trait::async_trait;
use futures::future::try_join_all;
use jellyfin_sdk_rs::{
    apis::{configuration::Configuration, image_api::get_item_image_infos},
    models::{BaseItemDto, BaseItemKind, ImageType, ItemFields},
};
use journey_db::{
    JourneyDbError,
    entity::{
        content::{self},
        images::{self},
        media_items::{self, MediaItemType},
        providers,
    },
    sea_orm::{DatabaseConnection, DatabaseTransaction, TransactionTrait},
};
use serde::Serialize;
use specta::Type;
use thiserror::Error;
use tokio::sync::mpsc::UnboundedSender;
use tracing::warn;
use uuid::Uuid;

use crate::{
    indexer::{Indexer, IndexerError, IndexerMsg, IndexerResult, NewIndexer, RequiredForIndexer},
    jellyfin::helpers::get_items_request,
};

#[derive(Debug, Error, Serialize, Type)]
//#[serde(tag = "error", content = "data")]
pub enum JellyfinIndexerError {
    #[error("Failed to retrieve Jellyfin API response entry.")]
    ApiEntryRetrievalError(Option<String>),
}

#[derive(Debug, Clone)]
pub struct JellyfinIndexer {
    pub model: providers::ActiveModelEx,
    pub config: Option<Configuration>,
}

impl NewIndexer for JellyfinIndexer {
    type Indexer = JellyfinIndexer;
    type Config = Configuration;

    fn new(model: providers::ActiveModelEx, config: Option<Self::Config>) -> Box<Self::Indexer> {
        Box::new(JellyfinIndexer {
            model: model,
            config: config,
        })
    }
}

#[async_trait]
impl RequiredForIndexer for JellyfinIndexer {
    fn get_model(&self) -> &providers::ActiveModelEx {
        &self.model
    }
    async fn index(
        &self,
        conn: &DatabaseConnection,
        comm: UnboundedSender<IndexerMsg>,
    ) -> IndexerResult<Vec<Option<IndexerError>>> {
        let user_id = self.user_id()?.to_string();
        let mut final_res: Vec<Option<IndexerError>> = vec![];

        {
            let txn = match conn.begin().await {
                Ok(txn) => Ok(txn),
                Err(err) => Err(IndexerError::FailedTransactionError(err.to_string())),
            }?;
            let mut res = self
                .index_by_type(&txn, &comm, &user_id, vec![BaseItemKind::MusicArtist])
                .await?;

            final_res.append(&mut res);
            match txn.commit().await {
                Ok(()) => Ok(()),
                Err(err) => Err(JourneyDbError::FailedTransactionError(err.to_string())),
            }?;
        }

        {
            let txn = match conn.begin().await {
                Ok(txn) => Ok(txn),
                Err(err) => Err(JourneyDbError::FailedTransactionError(err.to_string())),
            }?;
            let mut res = self
                .index_by_type(&txn, &comm, &user_id, vec![BaseItemKind::MusicAlbum])
                .await?;

            final_res.append(&mut res);
            match txn.commit().await {
                Ok(()) => Ok(()),
                Err(err) => Err(JourneyDbError::FailedTransactionError(err.to_string())),
            }?;
        }

        {
            let txn = match conn.begin().await {
                Ok(txn) => Ok(txn),
                Err(err) => Err(JourneyDbError::FailedTransactionError(err.to_string())),
            }?;
            let mut res = self
                .index_by_type(&txn, &comm, &user_id, vec![BaseItemKind::Audio])
                .await?;

            final_res.append(&mut res);
            match txn.commit().await {
                Ok(()) => Ok(()),
                Err(err) => Err(JourneyDbError::FailedTransactionError(err.to_string())),
            }?;
        }

        Ok(final_res)
    }
}

impl Indexer for JellyfinIndexer {}

impl JellyfinIndexer {
    fn get_config(&self) -> IndexerResult<&Configuration> {
        match &self.config {
            Some(config) => Ok(config),
            None => Err(JellyfinIndexerError::ApiEntryRetrievalError(None).into()),
        }
    }
    fn check_entry<T>(&self, entry: Option<T>) -> IndexerResult<T> {
        match entry {
            Some(entry) => Ok(entry),
            _ => Err(JellyfinIndexerError::ApiEntryRetrievalError(None).into()),
        }
    }
    fn match_item_type(&self, kind: BaseItemKind) -> IndexerResult<media_items::MediaItemType> {
        Ok(match kind {
            BaseItemKind::Audio => MediaItemType::Audio,
            BaseItemKind::MusicGenre => MediaItemType::Genre,
            BaseItemKind::MusicAlbum => MediaItemType::Album,
            BaseItemKind::MusicArtist => MediaItemType::Artist,
            BaseItemKind::Playlist => MediaItemType::Playlist,
            _ => MediaItemType::Unknown,
        })
    }
    async fn get_items(
        &self,
        user_id: &str,
        kind: Vec<BaseItemKind>,
    ) -> IndexerResult<Vec<BaseItemDto>> {
        warn!("Getting BaseItemDto's for user: {}", user_id);

        let response = get_items_request()
            .configuration(self.get_config()?)
            .user_id(user_id)
            .recursive(true)
            .include_item_types(kind)
            .fields(vec![ItemFields::ProviderIds])
            .call()
            .await?;

        Ok(self.check_entry(response.items)?)
    }
    async fn index_by_type(
        &self,
        txn: &DatabaseTransaction,
        comm: &UnboundedSender<IndexerMsg>,
        user_id: &str,
        kind: Vec<BaseItemKind>,
    ) -> IndexerResult<Vec<Option<IndexerError>>> {
        let items = self.get_items(user_id, kind).await?;

        let tasks = items
            .iter()
            .map(|item| self.assemble_media_item(txn, comm, item));

        Ok(try_join_all(tasks).await?)
    }
    async fn assemble_media_item(
        &self,
        txn: &DatabaseTransaction,
        comm: &UnboundedSender<IndexerMsg>,
        item: &BaseItemDto,
    ) -> IndexerResult<Option<IndexerError>> {
        let source_id = self.check_entry(item.id)?;
        let task_images_metadata = self.get_images(source_id);

        let weak_id = self.check_entry(item.name.clone().flatten())?;
        let ty = self.match_item_type(self.check_entry(item.r#type)?)?;
        let music_brainz_id = self.get_music_brainz_id(item, ty)?;

        let res = self
            .index_media_item(
                txn,
                comm,
                music_brainz_id,
                weak_id,
                source_id,
                ty,
                self.get_content(item)?,
                self.get_parent_source_ids(item),
                task_images_metadata.await?,
            )
            .await?;

        Ok(res)
    }
    fn get_music_brainz_id(
        &self,
        item: &BaseItemDto,
        ty: MediaItemType,
    ) -> IndexerResult<Option<Uuid>> {
        let known = self.check_entry(item.provider_ids.clone().flatten())?;

        let potential_id = match ty {
            MediaItemType::Artist => known.get("MusicBrainzArtist"),
            MediaItemType::Album => known.get("MusicBrainzReleaseGroup"),
            MediaItemType::Audio => known.get("MusicBrainzReleaseGroup"),
            _ => None,
        };

        match potential_id {
            Some(id) => match Uuid::parse_str(id) {
                Ok(uuid) => Ok(Some(uuid)),
                Err(err) => {
                    warn!(
                        "ReleaseGroup id was not a Uuid: {}, generated a custom one to continue.",
                        err
                    );
                    Ok(None)
                }
            },
            None => Ok(None),
        }
    }
    fn get_parent_source_ids(&self, item: &BaseItemDto) -> Vec<Uuid> {
        let mut parent_source_ids: Vec<Uuid> = vec![];
        match item.album_id.flatten() {
            Some(id) => parent_source_ids.push(id),
            _ => warn!(
                "No albums for: {:#?} with id: {:#?} -> skipping",
                item.id,
                item.name.clone().flatten()
            ),
        }

        match item.album_artists.clone().flatten() {
            Some(artists) => {
                for artist in artists {
                    match artist.id {
                        Some(id) => parent_source_ids.push(id),
                        _ => warn!(
                            "Artist: {:#?} somehow contained no id -> skipping",
                            artist.name.flatten()
                        ),
                    }
                }
            }
            _ => warn!(
                "No album artists for: {:#?} with id: {:#?} -> skipping",
                item.id, item.name
            ),
        }

        parent_source_ids
    }
    fn get_content(
        &self,
        item: &BaseItemDto,
    ) -> IndexerResult<Vec<(content::ContentType, Option<String>)>> {
        warn!(
            "Getting content for source_id: {:#?} - item: {:#?}",
            item.id,
            item.name.clone().flatten()
        );

        let album = (content::ContentType::Album, item.album.clone().flatten());

        let artists = (
            content::ContentType::Artists,
            item.album_artist.clone().flatten(),
        );

        let container = (
            content::ContentType::Container,
            item.container.clone().flatten(),
        );

        let date = match self.check_entry(item.premiere_date.clone().flatten()) {
            Ok(date) => Some(date.to_string()),
            Err(_) => None,
        };
        let release_date = (content::ContentType::ReleaseDate, date);

        Ok(vec![album, artists, container, release_date])
    }
    #[allow(unreachable_patterns)]
    fn match_image_type(&self, kind: ImageType) -> IndexerResult<images::ImageType> {
        Ok(match kind {
            ImageType::Art => images::ImageType::Art,
            ImageType::Backdrop => images::ImageType::Backdrop,
            ImageType::Banner => images::ImageType::Banner,
            ImageType::Box => images::ImageType::Box,
            ImageType::BoxRear => images::ImageType::BoxRear,
            ImageType::Chapter => images::ImageType::Chapter,
            ImageType::Disc => images::ImageType::Disc,
            ImageType::Logo => images::ImageType::Logo,
            ImageType::Menu => images::ImageType::Menu,
            ImageType::Primary => images::ImageType::Primary,
            ImageType::Profile => images::ImageType::Profile,
            ImageType::Screenshot => images::ImageType::Screenshot,
            ImageType::Thumb => images::ImageType::Thumb,
            _ => images::ImageType::Unknown,
        })
    }
    async fn get_images(&self, source_id: Uuid) -> IndexerResult<Vec<(String, images::ImageType)>> {
        warn!("Getting images for: {}", source_id);

        let images_req =
            match get_item_image_infos(self.get_config()?, &source_id.to_string()).await {
                Ok(images) => Ok(images),
                Err(err) => Err(JellyfinIndexerError::ApiEntryRetrievalError(Some(
                    err.to_string(),
                ))),
            }?;

        let base_url = self.url()?;
        let mut image_metadata: Vec<(String, images::ImageType)> = vec![];
        for image_info in images_req {
            let ty = self.match_image_type(self.check_entry(image_info.image_type)?)?;
            let init_url = format!("{}Items/{}/Images/{}", base_url, source_id, ty);

            image_metadata.push((init_url, ty));
        }

        Ok(image_metadata)
    }
}

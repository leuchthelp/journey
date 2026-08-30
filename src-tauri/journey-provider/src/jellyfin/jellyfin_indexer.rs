use std::fmt::Debug;

use async_trait::async_trait;
use futures::future::try_join_all;
use inherent::inherent;
use jellyfin_sdk_rs::{
    apis::{configuration::Configuration, image_api::get_item_image_infos},
    models::{BaseItemDto, BaseItemKind, ImageType},
};
use serde::Serialize;
use specta::Type;
use thiserror::Error;
use tokio::sync::mpsc::Sender;
use tracing::warn;
use url::Url;
use uuid::Uuid;

use crate::{
    helpers::get_items_request,
    indexer::{Indexer, IndexerError, IndexerMsg, IndexerResult, NewIndexer, RequiredForIndexer},
};
use journey_db::{
    JourneyDbError,
    entity::{
        MediaItems,
        content::{self, ContentType},
        images::{self},
        media_items::{self, MediaItemType},
        original, providers,
    },
    get_conn,
    sea_orm::{DatabaseTransaction, IntoActiveModel},
};

#[derive(Debug, Error, Serialize, Type)]
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
#[inherent]
impl RequiredForIndexer for JellyfinIndexer {
    fn get_model(&self) -> &providers::ActiveModelEx {
        &self.model
    }
    async fn index(
        &self,
        txn: &DatabaseTransaction,
        comm: Sender<IndexerMsg>,
    ) -> IndexerResult<()> {
        let user_id = self.user_id()?.to_string();
        let model = self.get_model().clone();

        self.index_by_type(&comm, model, &user_id, vec![BaseItemKind::MusicArtist])
            .await?;

        Ok(())
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
    fn match_item_type(&self, kind: BaseItemKind) -> IndexerResult<MediaItemType> {
        Ok(match kind {
            BaseItemKind::Audio => MediaItemType::Audio,
            BaseItemKind::MusicGenre => MediaItemType::Genre,
            BaseItemKind::MusicAlbum => MediaItemType::Album,
            BaseItemKind::MusicArtist => MediaItemType::Artist,
            BaseItemKind::Playlist => MediaItemType::Playlist,
            _ => MediaItemType::Unknown,
        })
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
    fn check_entry<T>(&self, entry: Option<T>) -> IndexerResult<T> {
        match entry {
            Some(entry) => Ok(entry),
            _ => Err(JellyfinIndexerError::ApiEntryRetrievalError(None).into()),
        }
    }
    fn wrap_content(
        &self,
        description: Option<String>,
        id: &Uuid,
        ty: ContentType,
    ) -> Option<content::ActiveModelEx> {
        match description {
            Some(description) => Some(
                content::ActiveModel::builder()
                    .set_description(description)
                    .set_parent_id(*id)
                    .set_ty(ty),
            ),
            _ => {
                warn!("Skipping: {} for {} because it does not exist", ty, id);
                None
            }
        }
    }
    async fn index_by_type(
        &self,
        comm_indexer: &Sender<IndexerMsg>,
        mut model: providers::ActiveModelEx,
        user_id: &str,
        kind: Vec<BaseItemKind>,
    ) -> IndexerResult<()> {
        let items = self.get_items(user_id, kind).await?;
        let tasks = items
            .iter()
            .map(|item| self.build_media_item(comm_indexer, &model, item));
        let media_items = try_join_all(tasks).await?;

        for item in media_items {
            model.media_items.push(item);
        }

        match model.save(&get_conn().await?).await {
            Ok(_) => Ok(()),
            Err(err) => Err(IndexerError::FailedDbInsertError(format!(
                "{} with specific: {:#?}",
                err,
                err.sql_err()
            ))),
        }
    }
    async fn build_media_item(
        &self,
        comm_indexer: &Sender<IndexerMsg>,
        model: &providers::ActiveModelEx,
        item: &BaseItemDto,
    ) -> IndexerResult<media_items::ActiveModelEx> {
        let item_id = self.check_entry(item.id)?;
        let ty = self.match_item_type(self.check_entry(item.r#type)?)?;
        let images = self.get_images(model, &item_id);
        let content = self.get_content(&item_id, item);
        let parents = self.get_item_parents(item);
        let music_brainz_id = self.get_music_brainz_id(item).await?;

        let original = original::ActiveModelEx::new()
            .set_uuid(item_id)
            .set_parent_id(music_brainz_id)
            .set_server_id(*model.server_id.as_ref());

        let mut media_item = media_items::ActiveModelEx::new()
            .set_ty(ty)
            .set_uuid(music_brainz_id)
            .set_outline_gradient("#ff000000")
            .set_loaded(false)
            .set_local(None)
            .add_original(original);

        for image in images.await? {
            media_item.images.push(image);
        }

        for parent in parents.await? {
            media_item.parents.push(parent);
        }

        for entry in content.await? {
            media_item.content.push(entry);
        }

        let msg = IndexerMsg {
            item: item.name.clone().flatten(),
            success: true,
        };
        match comm_indexer.try_send(msg) {
            Ok(_) => Ok(media_item),
            Err(err) => Err(IndexerError::FailedDbInsertError(err.to_string())),
        }
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
            .call()
            .await?;

        Ok(self.check_entry(response.items)?)
    }
    async fn get_music_brainz_id(&self, item: &BaseItemDto) -> IndexerResult<Option<Uuid>> {
        // let external_info_res = get_metadata_editor_info(self.get_config()?, &id.to_string()).await;
        // warn!("{:#?}", external_info_res);

        let user_data = self.check_entry(item.user_data.clone().flatten())?;
        match self.check_entry(user_data.item_id) {
            Ok(id) => Ok(Some(id)),
            Err(_) => {
                warn!(
                    "Could not find a musicbrainz id for item: {:#?}, generating tmp one",
                    item.name.clone().flatten()
                );
                Ok(None)
            }
        }
    }
    async fn get_images(
        &self,
        model: &providers::ActiveModelEx,
        id: &Uuid,
    ) -> IndexerResult<Vec<images::ActiveModelEx>> {
        warn!("Getting images for: {}", id);

        let images_req = match get_item_image_infos(self.get_config()?, &id.to_string()).await {
            Ok(images) => images,
            Err(err) => {
                return Err(
                    JellyfinIndexerError::ApiEntryRetrievalError(Some(err.to_string())).into(),
                );
            }
        };

        let base_url = model.url.as_ref();
        let mut images: Vec<images::ActiveModelEx> = vec![];
        for image_info in images_req {
            let ty = match image_info.image_type {
                Some(ty) => self.match_image_type(ty),
                _ => return Err(JellyfinIndexerError::ApiEntryRetrievalError(None).into()),
            }?;

            let tag = self.check_entry(image_info.image_tag.flatten())?;

            let url = match Url::parse(&format!("{}{}/{}", base_url, tag, ty)) {
                Ok(url) => url,
                Err(err) => {
                    return Err(IndexerError::FailedParseUrlError(format!(
                        "failed with: {} for base: {}/{}",
                        err, base_url, ty
                    )));
                }
            };

            let image_model = images::ActiveModelEx::new()
                .set_url(url)
                .set_ty(ty)
                .set_server_id(*model.server_id.as_ref())
                .set_provider(model.clone());

            images.push(image_model);
        }

        Ok(images)
    }
    async fn get_item_parents(
        &self,
        item: &BaseItemDto,
    ) -> IndexerResult<Vec<media_items::ActiveModel>> {
        warn!("Getting parents for: {:#?}", item.name.clone().flatten());

        let mut parent_ids: Vec<Uuid> = vec![];
        match item.album_id.flatten() {
            Some(id) => parent_ids.push(id),
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
                        Some(id) => parent_ids.push(id),
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

        let conn = &get_conn().await?;
        let mut tasks = vec![];
        for id in parent_ids {
            tasks.push(MediaItems::find_by_uuid(id).one(conn));
        }

        let parent_models = match try_join_all(tasks).await {
            Ok(parents) => parents,
            Err(err) => return Err(JourneyDbError::ConnectionError(err.to_string()).into()),
        };

        let mut parents: Vec<media_items::ActiveModel> = vec![];
        for model in parent_models {
            match model {
                Some(model) => parents.push(model.into_active_model()),
                _ => warn!("Somehow got no model back from database even though select succeeded"),
            }
        }
        Ok(parents)
    }
    async fn get_content(
        &self,
        id: &Uuid,
        item: &BaseItemDto,
    ) -> IndexerResult<Vec<content::ActiveModelEx>> {
        warn!(
            "Getting content for: {} - item: {:#?}",
            id,
            item.name.clone().flatten()
        );

        let album = self.wrap_content(item.album.clone().flatten(), id, ContentType::Album);

        let artists = self.wrap_content(
            item.album_artist.clone().flatten(),
            id,
            ContentType::Artists,
        );

        let container =
            self.wrap_content(item.container.clone().flatten(), id, ContentType::Container);

        let date = match self.check_entry(item.premiere_date.clone().flatten()) {
            Ok(date) => Some(date.to_string()),
            Err(_) => None,
        };
        let release_date = self.wrap_content(date, id, ContentType::ReleaseDate);

        let mut res: Vec<content::ActiveModelEx> = vec![];
        for potential in vec![album, artists, container, release_date] {
            match potential {
                Some(model) => res.push(model),
                _ => (),
            }
        }

        Ok(res)
    }
}

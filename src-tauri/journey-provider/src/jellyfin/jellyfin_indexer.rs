use std::fmt::Debug;

use async_trait::async_trait;
use futures::future::try_join_all;
use inherent::inherent;
use jellyfin_sdk_rs::{
    apis::{configuration::Configuration, image_api::get_item_image_infos},
    models::{BaseItemDto, BaseItemKind, ImageType, ItemFields},
};
use rapidhash::RapidHashSet;
use serde::Serialize;
use similar::TextDiff;
use specta::Type;
use thiserror::Error;
use tokio::sync::mpsc::UnboundedSender;
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
    sea_orm::{
        ColumnTrait, Condition, DatabaseConnection, DatabaseTransaction, DbErr, IntoActiveModel,
        QueryFilter,
    },
    sea_query::Expr,
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
    pub fn get_model(&self) -> &providers::ActiveModelEx {
        &self.model
    }
    pub async fn index(
        &self,
        txn: &DatabaseTransaction,
        comm: UnboundedSender<IndexerMsg>,
    ) -> IndexerResult<()> {
        let user_id = self.user_id()?.to_string();

        self.index_by_type(txn, &comm, &user_id, vec![BaseItemKind::MusicArtist])
            .await?;
        self.index_by_type(txn, &comm, &user_id, vec![BaseItemKind::MusicAlbum])
            .await?;
        self.index_by_type(txn, &comm, &user_id, vec![BaseItemKind::Audio])
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
    fn check_entry<T>(&self, entry: Option<T>) -> IndexerResult<T> {
        match entry {
            Some(entry) => Ok(entry),
            _ => Err(JellyfinIndexerError::ApiEntryRetrievalError(None).into()),
        }
    }
    fn check_exists<T: PartialEq>(&self, current: &T, slice: &[T]) -> bool {
        slice.contains(current)
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
            //.limit(7)
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
    ) -> IndexerResult<()> {
        let items = self.get_items(user_id, kind).await?;
        let tasks = items
            .iter()
            .map(|item| self.add_media_item(txn, comm, item));

        // for task in tasks {
        //     task.await?
        // }
        try_join_all(tasks).await?;
        Ok(())
    }
    fn get_music_brainz_id(
        &self,
        item: &BaseItemDto,
        ty: MediaItemType,
    ) -> IndexerResult<(Uuid, bool)> {
        let known = self.check_entry(item.provider_ids.clone().flatten())?;
        warn!("provider_ids: {:#?}", known);

        let potential_id = match ty {
            MediaItemType::Artist => known.get("MusicBrainzArtist"),
            MediaItemType::Album => known.get("MusicBrainzReleaseGroup"),
            MediaItemType::Audio => known.get("MusicBrainzReleaseGroup"),
            _ => None,
        };

        match potential_id {
            Some(id) => match Uuid::parse_str(id) {
                Ok(uuid) => Ok((uuid, false)),
                Err(err) => {
                    warn!("ReleaseGroup id was not a Uuid: {}", err);
                    Ok((Uuid::now_v7(), true))
                }
            },
            None => Ok((Uuid::now_v7(), true)),
        }
    }
    async fn filtered_call(
        &self,
        conn: &DatabaseConnection,
        filter: Expr,
    ) -> IndexerResult<Option<Vec<media_items::ModelEx>>> {
        match media_items::Entity::load()
            .filter(filter)
            .with(providers::Entity)
            .with(content::Entity)
            .with(original::Entity)
            .with(images::Entity)
            .all(conn)
            .await
        {
            Ok(items) => match items.is_empty() {
                false => Ok(Some(items)),
                true => Ok(None),
            },
            Err(DbErr::RecordNotFound(err)) => {
                warn!("Did find existing MediaItem: {}", err);
                Ok(None)
            }
            Err(err) => Err(JourneyDbError::RecordNotFound(err.to_string()).into()),
        }
    }
    async fn existing_media_item(
        &self,
        conn: &DatabaseConnection,
        music_brainz_id: &Uuid,
        weak_id: &String,
        ty: &MediaItemType,
    ) -> IndexerResult<(Option<media_items::ModelEx>, uuid::Uuid)> {
        let filter_music_brainz_id = media_items::Column::Uuid.eq(*music_brainz_id);
        let filter_item_ty = media_items::Column::Ty.eq(*ty);

        let combined_filter = Condition::all()
            .add(filter_music_brainz_id)
            .add(filter_item_ty.clone());

        match self.filtered_call(conn, combined_filter.into()).await? {
            Some(mut models) => Ok((models.pop(), *music_brainz_id)),
            None => {
                warn!("Could not find item with matching MediaBrainzId, falling back to WeakID");
                let filter_weak_id = media_items::Column::WeakId.like(weak_id.clone());

                let combined_filter = Condition::all().add(filter_weak_id).add(filter_item_ty);

                match self.filtered_call(conn, combined_filter.into()).await? {
                    Some(mut models) => {
                        let mut index: usize = 0;
                        let mut max_prob: f32 = f32::MIN;
                        for (i, model) in models.iter().enumerate() {
                            let diff = TextDiff::from_chars(model.weak_id.clone(), weak_id);

                            match diff.ratio() > max_prob {
                                true => {
                                    index = i;
                                    max_prob = diff.ratio()
                                }
                                false => (),
                            }
                        }

                        let model = models.remove(index);
                        let real_id = model.uuid;
                        Ok((Some(model), real_id))
                    }
                    None => {
                        warn!("Still couldn't find matching Model, does probably not exists yet.");
                        Ok((None, *music_brainz_id))
                    }
                }
            }
        }
    }
    async fn add_media_item(
        &self,
        txn: &DatabaseTransaction,
        comm: &UnboundedSender<IndexerMsg>,
        item: &BaseItemDto,
    ) -> IndexerResult<()> {
        let conn = get_conn().await?;

        let item_id = self.check_entry(item.id)?;
        let weak_id = self.check_entry(item.name.clone().flatten())?;

        let ty = self.match_item_type(self.check_entry(item.r#type)?)?;
        let (music_brainz_id, is_tmp) = self.get_music_brainz_id(item, ty)?;

        /*
            music_brainz_id could become a temporary one at which point we fall back to weak_id to perform
            one last shot at finding an existing item to avoid duplicates.
            However, in that case music_brainz_id becomes misaligned. Therefore it needs to be set back to
            the actual one found in the database.
        */
        let (mut media_item, music_brainz_id, already_exists) = match self
            .existing_media_item(&conn, &music_brainz_id, &weak_id, &ty)
            .await?
        {
            (Some(item), music_brainz_id) => (item.into_active_model(), music_brainz_id, true),
            (None, music_brainz_id) => (
                media_items::ActiveModelEx::default()
                    .set_ty(ty)
                    .set_uuid(music_brainz_id)
                    .set_weak_id(&weak_id)
                    .set_is_tmp(is_tmp)
                    .set_outline_gradient("#ff000000"),
                music_brainz_id,
                false,
            ),
        };

        let task_images = self.add_images(media_item.images.as_slice(), &item_id);
        let task_parents = self.add_item_parents(media_item.parents.as_slice(), item);

        match self.check_exists(self.get_model(), media_item.providers.as_slice()) {
            true => (),
            false => _ = media_item.providers.push(self.get_model().clone()),
        }

        let original = self.add_original(&music_brainz_id, item)?;
        match self.check_exists(&original, media_item.original.as_slice()) {
            true => (),
            false => _ = media_item.providers.push(self.get_model().clone()),
        }

        for content in self.add_content(media_item.content.as_slice(), &music_brainz_id, item)? {
            media_item.content.push(content);
        }

        for image in task_images.await? {
            media_item.images.push(image);
        }

        for parent in task_parents.await? {
            media_item.parents.push(parent);
        }

        /*
           Should roll it's own nested transaction & therefore provide savepoint & rollback functionality according to:
           - https://www.sea-ql.org/SeaORM/docs/advanced-query/transaction/#nested-transaction
           - https://www.sea-ql.org/SeaORM/docs/advanced-query/nested-active-model/
        */
        let success = match media_item.save(txn).await {
            Ok(_) => Ok(true),
            Err(err) => Err(journey_db::JourneyDbError::Unknown(err.to_string())),
        }?;

        let msg = IndexerMsg {
            item: Some(weak_id),
            success: success,
            already_exists: already_exists,
        };
        match comm.send(msg) {
            Ok(_) => Ok(()),
            Err(err) => Err(IndexerError::FailedMsgSendError(err.to_string())),
        }
    }
    fn add_original(
        &self,
        music_brainz_id: &Uuid,
        item: &BaseItemDto,
    ) -> IndexerResult<original::ActiveModelEx> {
        let item_id = self.check_entry(item.id)?;
        let original = original::ActiveModelEx::new()
            .set_uuid(item_id)
            .set_parent_id(*music_brainz_id)
            .set_server_id(self.server_id()?);

        Ok(original)
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
    fn add_content(
        &self,
        existing: &[content::ActiveModelEx],
        id: &Uuid,
        item: &BaseItemDto,
    ) -> IndexerResult<Vec<content::ActiveModelEx>> {
        warn!(
            "Getting content for: {} - item: {:#?}",
            id,
            item.name.clone().flatten()
        );

        let mut known_content = RapidHashSet::default();
        for content in existing {
            match (
                content.parent_id.try_as_ref(),
                content.ty.try_as_ref(),
                content.description.try_as_ref(),
            ) {
                (Some(Some(parent_id)), Some(ty), Some(Some(description))) => {
                    _ = known_content.insert((parent_id, ty, description))
                }
                _ => (),
            }
        }

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
                Some(model) => match (
                    model.parent_id.try_as_ref(),
                    model.ty.try_as_ref(),
                    model.description.try_as_ref(),
                ) {
                    (Some(Some(parent_id)), Some(ty), Some(Some(description))) => {
                        match known_content.contains(&(parent_id, ty, description)) {
                            false => res.push(model),
                            true => (),
                        }
                    }
                    _ => (),
                },
                _ => (),
            }
        }

        Ok(res)
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
    async fn add_images(
        &self,
        existing: &[images::ActiveModelEx],
        id: &Uuid,
    ) -> IndexerResult<Vec<images::ActiveModelEx>> {
        warn!("Getting images for: {}", id);

        let images_req = match get_item_image_infos(self.get_config()?, &id.to_string()).await {
            Ok(images) => Ok(images),
            Err(err) => Err(JellyfinIndexerError::ApiEntryRetrievalError(Some(
                err.to_string(),
            ))),
        }?;

        let mut known_images = RapidHashSet::default();
        for image in existing {
            if let Some(url) = image.url.try_as_ref() {
                known_images.insert(url);
            }
        }

        warn!("known images: {:#?}", known_images);
        warn!("images req: {:#?}", images_req);

        let base_url = self.url()?;
        let mut images: Vec<images::ActiveModelEx> = vec![];
        for image_info in images_req {
            let ty = self.match_image_type(self.check_entry(image_info.image_type)?)?;
            let init_url = &format!("{}Items/{}/Images/{}", base_url, id, ty);

            match known_images.contains(init_url) {
                false => {
                    let url = match Url::parse(init_url) {
                        Ok(url) => url,
                        Err(err) => {
                            return Err(IndexerError::FailedParseUrlError(format!(
                                "failed with: {} for base: {}/{}",
                                err, base_url, ty
                            )));
                        }
                    };
                    warn!("new_url: {}", url);

                    let image_model = images::ActiveModelEx::new()
                        .set_url(url)
                        .set_ty(ty)
                        .set_server_id(self.server_id()?)
                        .set_provider(self.get_model().clone());

                    images.push(image_model)
                }
                true => (),
            }
        }

        Ok(images)
    }
    async fn add_item_parents(
        &self,
        existing: &[media_items::ActiveModelEx],
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

        let mut known_parents = RapidHashSet::default();
        for parent in existing {
            if let Some(music_brainz_id) = parent.uuid.try_as_ref() {
                known_parents.insert(music_brainz_id);
            }
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
}

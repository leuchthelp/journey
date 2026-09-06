use std::fmt::Debug;

use anyhow::Result;
use async_trait::async_trait;
use dyn_clone::{DynClone, clone_trait_object};
use futures::future::try_join_all;
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
    helpers::check_exists, indexer_manager::IndexerKey,
    jellyfin::jellyfin_indexer::JellyfinIndexerError,
};
use journey_db::{
    JourneyDbError,
    entity::{
        ProviderVariant,
        content::{self},
        images, jt_parent_to_child,
        media_items::{self, MediaItemType},
        providers, sources,
    },
    sea_orm::{
        ColumnTrait, Condition, DatabaseConnection, DatabaseTransaction, DbErr, EntityTrait,
        IntoActiveModel, QueryFilter,
    },
    sea_query::Expr,
};

#[derive(Debug)]
pub struct IndexerMsg {
    pub item: Option<String>,
    pub success: bool,
    pub already_exists: bool,
}

#[derive(Debug, Error, Serialize, Type)]
pub enum IndexerError {
    #[error("Failed to parse the given String to an Url: {0}")]
    FailedParseUrlError(String),
    #[error("Failed to retrieve Jellyfin API response entry.")]
    ApiEntryRetrievalError(Option<String>),
    #[error("Failed to insert into database: {0}")]
    FailedDbInsertError(String),
    #[error("Failed to send update message over channel: {0}")]
    FailedMsgSendError(String),
    #[error("Failed to run transaction: {0}")]
    FailedTransactionError(String),
    #[error(
        "ProviderVariant has not been set. This cannot be done here. Check the original Provider implementation"
    )]
    MissingVariantError,
    #[error("server_id has not been set yet, try authenticating first.")]
    MissingServerIdError,
    #[error("user_id has not been set yet, try authenticating first.")]
    MissingUserIdError,
    #[error("Url has not been set yet, provide one first.")]
    MissingUrlError,
    #[error(transparent)]
    JellyfinIndexerError(#[from] JellyfinIndexerError),
    #[error(transparent)]
    JourneyDbError(#[from] journey_db::JourneyDbError),
}

pub type IndexerResult<T> = Result<T, IndexerError>;

pub trait NewIndexer {
    type Indexer;
    type Config;

    fn new(model: providers::ActiveModelEx, config: Option<Self::Config>) -> Box<Self::Indexer>;
}

#[async_trait]
pub trait RequiredForIndexer {
    fn get_model(&self) -> &providers::ActiveModelEx;
    async fn index(
        &self,
        conn: &DatabaseConnection,
        comm: UnboundedSender<IndexerMsg>,
    ) -> IndexerResult<Vec<Option<IndexerError>>>;
}

#[async_trait]
pub trait Indexer: RequiredForIndexer + DynClone + Debug + Send {
    fn ty(&self) -> IndexerResult<ProviderVariant> {
        match self.get_model().ty.try_as_ref() {
            Some(variant) => Ok(*variant),
            _ => Err(IndexerError::MissingVariantError),
        }
    }
    fn user_id(&self) -> IndexerResult<Uuid> {
        match self.get_model().user_id.try_as_ref() {
            Some(user_id) if *user_id != Uuid::nil() => Ok(*user_id),
            _ => Err(IndexerError::MissingServerIdError),
        }
    }
    fn provider_id(&self) -> IndexerResult<Uuid> {
        match self.get_model().provider_id.try_as_ref() {
            Some(server_id) if *server_id != Uuid::nil() => Ok(*server_id),
            _ => Err(IndexerError::MissingServerIdError),
        }
    }
    fn url(&self) -> IndexerResult<Url> {
        match self.get_model().url.try_as_ref() {
            Some(url) => Ok(match Url::parse(url) {
                Ok(url) => Ok(url),
                Err(err) => Err(IndexerError::FailedParseUrlError(err.to_string())),
            }?),
            _ => Err(IndexerError::MissingUrlError),
        }
    }
    fn key(&self) -> IndexerResult<IndexerKey> {
        Ok(IndexerKey {
            variant: self.ty()?,
            provider_id: self.provider_id()?,
        })
    }
    async fn filtered_call(
        &self,
        txn: &DatabaseTransaction,
        filter: Expr,
    ) -> IndexerResult<Option<Vec<media_items::ModelEx>>> {
        match media_items::Entity::load()
            .filter(filter)
            .with(providers::Entity)
            .with(content::Entity)
            .with(sources::Entity)
            .with(images::Entity)
            .with(jt_parent_to_child::Entity::REVERSE)
            .all(txn)
            .await
        {
            Ok(items) => match items.is_empty() {
                false => Ok(Some(items)),
                true => Ok(None),
            },
            Err(DbErr::RecordNotFound(err)) => {
                warn!("Did not find existing MediaItem: {}", err);
                Ok(None)
            }
            Err(err) => Err(JourneyDbError::RecordNotFound(err.to_string()).into()),
        }
    }
    async fn existing_media_item(
        &self,
        txn: &DatabaseTransaction,
        music_brainz_id: Uuid,
        weak_id: &String,
        ty: MediaItemType,
    ) -> IndexerResult<(Option<media_items::ModelEx>, uuid::Uuid)> {
        let filter_music_brainz_id = media_items::Column::MusicBrainzId.eq(music_brainz_id);
        let filter_item_ty = media_items::Column::Ty.eq(ty);

        let combined_filter = Condition::all()
            .add(filter_music_brainz_id)
            .add(filter_item_ty.clone());

        match self.filtered_call(txn, combined_filter.into()).await? {
            Some(mut models) => Ok((models.pop(), music_brainz_id)),
            None => {
                warn!("Could not find item with matching MediaBrainzId, falling back to WeakID");
                let filter_weak_id = media_items::Column::WeakId.like(weak_id);

                let combined_filter = Condition::all().add(filter_weak_id).add(filter_item_ty);

                match self.filtered_call(txn, combined_filter.into()).await? {
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
                        let real_id = model.music_brainz_id;
                        Ok((Some(model), real_id))
                    }
                    None => {
                        warn!("Still couldn't find matching Model, probably does not exists yet.");
                        Ok((None, music_brainz_id))
                    }
                }
            }
        }
    }
    async fn build_media_item(
        &self,
        txn: &DatabaseTransaction,
        music_brainz_id: Option<Uuid>,
        weak_id: &String,
        ty: MediaItemType,
    ) -> IndexerResult<(media_items::ActiveModelEx, Uuid, bool)> {
        let (music_brainz_id, is_tmp) = match music_brainz_id {
            Some(id) => (id, false),
            None => (Uuid::now_v7(), true),
        };

        match self
            .existing_media_item(txn, music_brainz_id, weak_id, ty)
            .await?
        {
            (Some(item), music_brainz_id) => Ok((item.into_active_model(), music_brainz_id, true)),
            (None, music_brainz_id) => Ok((
                media_items::ActiveModelEx::default()
                    .set_ty(ty)
                    .set_music_brainz_id(music_brainz_id)
                    .set_weak_id(weak_id)
                    .set_is_tmp(is_tmp)
                    .set_outline_gradient("#ff000000"),
                music_brainz_id,
                false,
            )),
        }
    }
    async fn index_media_item(
        &self,
        txn: &DatabaseTransaction,
        comm: &UnboundedSender<IndexerMsg>,
        music_brainz_id: Option<Uuid>,
        weak_id: String,
        source_id: Uuid,
        ty: MediaItemType,
        potential_new_content: Vec<(content::ContentType, Option<String>)>,
        potential_parent_source_ids: Vec<Uuid>,
        potential_new_image: Vec<(String, images::ImageType)>,
    ) -> IndexerResult<Option<IndexerError>> {
        let (mut media_item, music_brainz_id, already_exists) = self
            .build_media_item(txn, music_brainz_id, &weak_id, ty)
            .await?;

        let task_parents = self.validate_parents(
            &txn,
            media_item.parents.as_slice(),
            potential_parent_source_ids,
        );

        match check_exists(self.get_model(), media_item.providers.as_slice()) {
            true => (),
            false => _ = media_item.providers.push(self.get_model().clone()),
        }

        if let Some(source) =
            self.validate_source(media_item.sources.as_slice(), music_brainz_id, source_id)?
        {
            media_item.sources.push(source);
        }

        for entry in self.validate_content(
            media_item.content.as_slice(),
            music_brainz_id,
            potential_new_content,
        )? {
            media_item.content.push(entry);
        }

        for image in self.validate_images(media_item.images.as_slice(), potential_new_image)? {
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
        let (success, error) = match media_item.save(txn).await {
            Ok(_) => (true, None),
            Err(err) => (
                false,
                Some(journey_db::JourneyDbError::Unknown(err.to_string()).into()),
            ),
        };

        let msg = IndexerMsg {
            item: Some(weak_id),
            success: success,
            already_exists: already_exists,
        };

        match comm.send(msg) {
            Ok(_) => Ok(error),
            Err(err) => Err(IndexerError::FailedMsgSendError(err.to_string())),
        }
    }
    fn validate_source(
        &self,
        existing: &[sources::ActiveModelEx],
        music_brainz_id: Uuid,
        source_id: Uuid,
    ) -> IndexerResult<Option<sources::ActiveModelEx>> {
        /*
           Due to this sea-orm issue: https://github.com/SeaQL/sea-orm/issues/3183
           it is currently not possible to used the above approach for a clean solution.
           Will have to wait for improvements upstream before this can be fixed.
        */
        let source = sources::ActiveModelEx::new()
            .set_source_id(source_id)
            .set_parent_id(music_brainz_id)
            .set_provider_id(self.provider_id()?);

        let mut source_exists = false;
        for known in existing {
            if known.source_id.try_as_ref() == source.source_id.try_as_ref() {
                source_exists = true
            }
        }

        match source_exists {
            false => Ok(Some(source)),
            true => Ok(None),
        }
    }
    fn validate_images(
        &self,
        existing: &[images::ActiveModelEx],
        image_metadata: Vec<(String, images::ImageType)>,
    ) -> IndexerResult<Vec<images::ActiveModelEx>> {
        let mut known_images = RapidHashSet::default();
        for image in existing {
            if let Some(url) = image.url.try_as_ref() {
                known_images.insert(url);
            }
        }

        let mut images: Vec<images::ActiveModelEx> = vec![];
        for (init_url, ty) in image_metadata {
            match known_images.contains(&init_url) {
                false => {
                    let url = match Url::parse(&init_url) {
                        Ok(url) => Ok(url),
                        Err(err) => Err(IndexerError::FailedParseUrlError(format!(
                            "failed with: {} for base: {}/{}",
                            err, init_url, ty
                        ))),
                    }?;

                    let image_model = images::ActiveModelEx::new()
                        .set_url(url)
                        .set_ty(ty)
                        .set_provider_id(self.provider_id()?)
                        .set_provider(self.get_model().clone());

                    images.push(image_model)
                }
                true => (),
            }
        }

        Ok(images)
    }
    fn validate_content(
        &self,
        existing: &[content::ActiveModelEx],
        parent_id: Uuid,
        potential_content: Vec<(content::ContentType, Option<String>)>,
    ) -> IndexerResult<Vec<content::ActiveModelEx>> {
        let mut known_content = RapidHashSet::default();
        for content in existing {
            match (
                content.parent_id.try_as_ref(),
                content.ty.try_as_ref(),
                content.description.try_as_ref(),
            ) {
                (Some(parent_id), Some(ty), Some(description)) => {
                    _ = known_content.insert((parent_id, ty, description))
                }
                _ => (),
            }
        }

        let mut new_content: Vec<content::ActiveModelEx> = vec![];
        for (ty, description) in potential_content {
            match description {
                Some(description) => {
                    match known_content.contains(&(&parent_id, &ty, &description)) {
                        false => {
                            let content = content::ActiveModel::builder()
                                .set_description(description)
                                .set_parent_id(parent_id)
                                .set_ty(ty);
                            new_content.push(content)
                        }
                        true => (),
                    }
                }
                _ => warn!(
                    "Skipping: {} for {:#?} because it does not exist",
                    ty, parent_id
                ),
            }
        }

        Ok(new_content)
    }
    async fn validate_parents(
        &self,
        txn: &DatabaseTransaction,
        existing: &[media_items::ActiveModelEx],
        parent_source_ids: Vec<Uuid>,
    ) -> IndexerResult<Vec<media_items::ActiveModel>> {
        let mut known_parents = RapidHashSet::default();
        for parent in existing {
            for known_source in parent.sources.as_slice() {
                if let Some(source_id) = known_source.source_id.try_as_ref() {
                    known_parents.insert(source_id);
                }
            }
        }

        let mut tasks = vec![];
        for source_id in parent_source_ids {
            tasks.push(
                media_items::Entity::find()
                    .has_related(sources::Entity, sources::Column::SourceId.eq(source_id))
                    .one(txn),
            );
        }

        let parent_models = match try_join_all(tasks).await {
            Ok(parents) => Ok(parents),
            Err(err) => Err(JourneyDbError::ConnectionError(err.to_string())),
        }?;

        let mut parents: Vec<media_items::ActiveModel> = vec![];
        for model in parent_models {
            match model {
                Some(model) => parents.push(model.into_active_model()),
                _ => warn!(
                    "Got no models for tiered ids. Might have not been added to database successfully yet."
                ),
            }
        }
        Ok(parents)
    }
}

clone_trait_object!(Indexer);

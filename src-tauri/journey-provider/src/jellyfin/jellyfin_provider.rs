use std::fmt::Debug;

use async_trait::async_trait;
use futures::future::try_join_all;
use inherent::inherent;
use jellyfin_sdk_rs::{
    apis::{
        authentication_api::authenticate_user_by_name, configuration::Configuration,
        image_api::get_item_image_infos,
    },
    configure,
    models::{AuthenticateUserByName, BaseItemDto, BaseItemKind, ImageType, UserDto},
    required::{ClientInfo, DeviceInfo},
};
use serde::Serialize;
use specta::Type;
use thiserror::Error;
use tokio::sync::mpsc::Sender;
use tracing::warn;
use url::Url;
use uuid::Uuid;

use crate::{
    ProviderError,
    helpers::get_items_request,
    provider::{NewProvider, Provider, ProviderResult, RequiredForProvider},
    provider_manager::IndexerMsg,
};
use journey_db::{
    JourneyDbError,
    entity::{
        MediaItems, ProviderVariant,
        content::{self, ContentType},
        images::{self},
        media_items::{self, MediaItemType},
        original, providers,
    },
    get_conn,
    sea_orm::{ActiveValue::Set, IntoActiveModel},
};
use journey_utils::constants::{PRODUCT_NAME, PRODUCT_VERSION};

#[derive(Debug, Error, Serialize, Type)]
pub enum JellyfinProviderError {
    #[error("Failed to retrieve Jellyfin API response entry.")]
    ApiEntryRetrievalError(Option<String>),
    #[error("server_id hasn't been set yet, try authenticating first.")]
    MissingServerIdError,
    #[error("user_id hasn't been set yet, try authenticating first.")]
    MissingUserIdError,
    #[error("Url hasn't been set yet, provide one first.")]
    MissingUrlError,
    #[error("Failed to parse given String to Uuid.")]
    FailedUuidParseError,
    #[error("Failed to authenticate with username & password.")]
    FailedPasswordAuthError,
    #[error("Failed to build client config for SDK client.")]
    FailedBuildConfigError,
}

#[derive(Debug, Clone)]
pub struct JellyfinProvider {
    pub(crate) model: providers::ActiveModelEx,
    pub(crate) config: Option<Configuration>,
    pub(crate) client_info: ClientInfo,
    pub(crate) device_info: DeviceInfo,
}

impl NewProvider for JellyfinProvider {
    type Provider = JellyfinProvider;

    fn new(model: providers::ActiveModelEx) -> ProviderResult<Box<Self>> {
        let client_info = ClientInfo {
            name: PRODUCT_NAME,
            version: PRODUCT_VERSION,
        };

        let device_info = DeviceInfo {
            id: Uuid::now_v7(),
            name: format!(
                "{}-{}-{}-{}",
                tauri_plugin_os::hostname(),
                tauri_plugin_os::platform(),
                tauri_plugin_os::arch(),
                tauri_plugin_os::version()
            ),
            languages: None,
        };

        Ok(Box::new(JellyfinProvider {
            model: model,
            config: None,
            client_info,
            device_info,
        }))
    }
}

#[async_trait]
#[inherent]
impl RequiredForProvider for JellyfinProvider {
    pub fn ty(&self) -> ProviderVariant {
        ProviderVariant::JellyfinProvider
    }
    pub fn get_model(&self) -> &providers::ActiveModelEx {
        &self.model
    }
    pub fn set_model(&mut self, new: providers::ActiveModelEx) {
        self.model = new
    }
    pub fn invalidate(&mut self) -> ProviderResult<()> {
        self.model = providers::ActiveModelEx::default();
        self.config = None;
        Ok(())
    }
    pub async fn password_auth(&mut self, uname: String, psw: String) -> ProviderResult<String> {
        let mut client_config = match configure()
            .base_url(&self.url()?)
            .client_info(&self.client_info)
            .device_info(&self.device_info)
            .call()
        {
            Ok(config) => config,
            Err(_) => return Err(JellyfinProviderError::FailedBuildConfigError.into()),
        };

        let auth_by_name = AuthenticateUserByName {
            username: Some(Some(uname)),
            pw: Some(Some(psw)),
        };
        let auth_res = match authenticate_user_by_name(&client_config, auth_by_name).await {
            Ok(res) => res,
            Err(_) => return Err(JellyfinProviderError::FailedPasswordAuthError.into()),
        };

        let access_token = match auth_res.access_token.flatten() {
            Some(token) => Ok(token),
            None => Err(JellyfinProviderError::ApiEntryRetrievalError(None)),
        }?;

        self.set_server_id(auth_res.server_id)?;
        self.set_user_id(auth_res.user)?;

        client_config = match configure()
            .base_url(&self.url()?)
            .client_info(&self.client_info)
            .device_info(&self.device_info)
            .access_token(&access_token)
            .call()
        {
            Ok(config) => config,
            Err(_) => return Err(JellyfinProviderError::FailedBuildConfigError.into()),
        };

        self.config = Some(client_config);
        Ok(access_token)
    }
    pub async fn index(&self, comm_indexer: Sender<IndexerMsg>) -> ProviderResult<()> {
        let user_id = &self.user_id()?.to_string();

        self.index_by_type(user_id, &comm_indexer, vec![BaseItemKind::MusicAlbum])
            .await?;
        // self.index_by_type(user_id, &indexer, vec![BaseItemKind::MusicArtist])
        //     .await?;
        // self.index_by_type(user_id, &indexer, vec![BaseItemKind::Audio])
        //     .await?;

        Ok(())
    }
}

impl JellyfinProvider {
    fn set_server_id(&mut self, server_id: Option<Option<String>>) -> ProviderResult<()> {
        let server_id = match server_id.flatten() {
            Some(token) => Ok(token),
            None => Err(JellyfinProviderError::ApiEntryRetrievalError(None)),
        }?;

        self.model.server_id = Set(match Uuid::parse_str(&server_id) {
            Ok(uuid) => uuid,
            Err(_) => return Err(JellyfinProviderError::FailedUuidParseError.into()),
        });
        Ok(())
    }

    /*
       EFFECTIVELY: WE DON'T TRUST THE JELLYFIN API AT ALL

       When we authenticate we are required to check if the value provided by Jellyfin
       actually exists. Due to the way Jellyfins current API is structured there are a
       lot of unnecessary Option<>.

       We could pass it through in one line but we risk an error for a missing user_id
       if anything in the Jellyfin API gets funky.
    */
    fn set_user_id(&mut self, user_dto: Option<Option<Box<UserDto>>>) -> ProviderResult<()> {
        let user_dto = match user_dto.flatten() {
            Some(dto) => Ok(dto),
            None => Err(JellyfinProviderError::ApiEntryRetrievalError(None)),
        }?;

        let user_id = match user_dto.id {
            Some(id) => Ok(id),
            None => Err(JellyfinProviderError::MissingUserIdError),
        }?;

        self.model.user_id = Set(user_id);
        Ok(())
    }
    fn get_config(&self) -> ProviderResult<&Configuration> {
        match &self.config {
            Some(config) => Ok(config),
            None => Err(JellyfinProviderError::ApiEntryRetrievalError(None).into()),
        }
    }
    fn match_item_type(&self, kind: BaseItemKind) -> ProviderResult<MediaItemType> {
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
    fn match_image_type(&self, kind: ImageType) -> ProviderResult<images::ImageType> {
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
    fn check_entry<T: Debug>(&self, entry: Option<T>) -> ProviderResult<T> {
        match entry {
            Some(entry) => Ok(entry),
            _ => Err(JellyfinProviderError::ApiEntryRetrievalError(Some(format!(
                "for entry: {:#?}",
                entry
            )))
            .into()),
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
        user_id: &str,
        comm_indexer: &Sender<IndexerMsg>,
        kind: Vec<BaseItemKind>,
    ) -> ProviderResult<()> {
        let mut model = self.get_model().clone();

        let items = self.get_items(user_id, kind).await?;
        let tasks = items
            .iter()
            .map(|item| self.build_media_item(comm_indexer, item));
        let media_items = try_join_all(tasks).await?;

        for item in media_items {
            model.media_items.push(item);
        }

        match model.save(&get_conn().await?).await {
            Ok(_) => Ok(()),
            Err(err) => Err(ProviderError::FailedDbInsertError(format!(
                "{} with specific: {:#?}",
                err,
                err.sql_err()
            ))),
        }
    }
    async fn build_media_item(
        &self,
        comm_indexer: &Sender<IndexerMsg>,
        item: &BaseItemDto,
    ) -> ProviderResult<media_items::ActiveModelEx> {
        let item_id = self.check_entry(item.id)?;
        let ty = self.match_item_type(self.check_entry(item.r#type)?)?;
        let images = self.get_images(&item_id);
        let content = self.get_content(&item_id, item);
        let parents = self.get_item_parents(item);
        let music_brainz_id = self.get_music_brainz_id(item).await?;

        let original = original::ActiveModelEx::new()
            .set_uuid(item_id)
            .set_parent_id(music_brainz_id)
            .set_server_id(self.server_id()?);

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
            Err(err) => Err(ProviderError::FailedDbInsertError(err.to_string())),
        }
    }
    async fn get_items(
        &self,
        user_id: &str,
        kind: Vec<BaseItemKind>,
    ) -> ProviderResult<Vec<BaseItemDto>> {
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
    async fn get_music_brainz_id(&self, item: &BaseItemDto) -> ProviderResult<Option<Uuid>> {
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
    async fn get_images(&self, id: &Uuid) -> ProviderResult<Vec<images::ActiveModelEx>> {
        warn!("Getting images for: {}", id);

        let images_req = match get_item_image_infos(self.get_config()?, &id.to_string()).await {
            Ok(images) => images,
            Err(err) => {
                return Err(
                    JellyfinProviderError::ApiEntryRetrievalError(Some(err.to_string())).into(),
                );
            }
        };

        let base_url = self.url()?;
        let mut images: Vec<images::ActiveModelEx> = vec![];
        for image_info in images_req {
            let ty = match image_info.image_type {
                Some(ty) => self.match_image_type(ty),
                _ => return Err(JellyfinProviderError::ApiEntryRetrievalError(None).into()),
            }?;

            let tag = self.check_entry(image_info.image_tag.flatten())?;

            let url = match Url::parse(&format!("{}{}/{}", base_url, tag, ty)) {
                Ok(url) => url,
                Err(err) => {
                    return Err(ProviderError::FailedParseUrlError(format!(
                        "failed with: {} for base: {}/{}",
                        err, base_url, ty
                    )));
                }
            };

            let image_model = images::ActiveModelEx::new()
                .set_url(url)
                .set_ty(ty)
                .set_server_id(self.server_id()?)
                .set_provider(self.get_model().clone());

            images.push(image_model);
        }

        Ok(images)
    }
    async fn get_item_parents(
        &self,
        item: &BaseItemDto,
    ) -> ProviderResult<Vec<media_items::ActiveModel>> {
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
    ) -> ProviderResult<Vec<content::ActiveModelEx>> {
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

#[async_trait]
#[inherent]
impl Provider for JellyfinProvider {}

#[cfg(test)]
mod variant_jellyfin {
    use std::collections::HashMap;

    use serial_test::serial;
    use test_log::test;
    use tokio::sync::mpsc::{self, Receiver, Sender};
    use tracing::warn;
    use url::Url;

    use crate::{
        jellyfin_provider::JellyfinProvider,
        provider::{NewProvider, Provider},
        provider_manager::IndexerMsg,
    };
    use journey_db::entity::{ProviderVariant, providers::ActiveModelEx};
    use journey_keyring::Entry;
    use journey_utils::get_env_local;

    #[test]
    fn matching_name() {
        let model = ActiveModelEx::new().set_url(Url::parse("http://smth.example.com").unwrap());

        assert!(matches!(
            JellyfinProvider::new(model).unwrap().ty(),
            ProviderVariant::JellyfinProvider
        ));
    }

    #[tokio::test]
    //#[ignore]
    #[serial]
    async fn try_auth_flow() {
        let env_map = get_env_local();

        let env_map = env_map.unwrap();
        journey_keyring::use_native_store().unwrap();

        warn!("{}", env_map.var("TEST_JELLYFIN_URL").unwrap());
        let url = env_map.var("TEST_JELLYFIN_URL").unwrap();

        let model = ActiveModelEx::new()
            .set_url(Url::parse(&url).unwrap())
            .set_ty(ProviderVariant::JellyfinProvider);
        let mut provider = JellyfinProvider::new(model).unwrap();

        assert!(provider.authenticated().is_err());
        assert!(provider.server_id().is_err());
        assert!(provider.user_id().is_err());
        assert!(provider.url().is_ok());

        let token = provider
            .password_auth(
                env_map.var("TEST_JELLYFIN_USER").unwrap(),
                env_map.var("TEST_JELLYFIN_PW").unwrap(),
            )
            .await
            .unwrap();
        provider.save_token(&token).unwrap();
        provider.add_to_db().await.unwrap();

        assert!(provider.authenticated().unwrap() == true);
        assert!(provider.server_id().is_ok());
        assert!(provider.user_id().is_ok());
        assert!(provider.url().is_ok());

        let test = Entry::search(&HashMap::from([("service", "journey")])).unwrap();
        test.iter().for_each(|f| warn!("{:#?}", f.get_password()));

        let (tx, mut _rx): (Sender<IndexerMsg>, Receiver<IndexerMsg>) = mpsc::channel(100);
        provider.index(tx).await.unwrap();

        provider.remove_token().unwrap();
        provider.remove_from_db().await.unwrap();
        provider.invalidate().unwrap();

        assert!(provider.authenticated().is_err());
        assert!(provider.server_id().is_err());
        assert!(provider.user_id().is_err());
        assert!(provider.url().is_err());

        journey_keyring::release_store();
    }
}

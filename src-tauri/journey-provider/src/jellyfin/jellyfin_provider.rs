use crate::{
    ProviderError,
    helpers::get_items_request,
    provider::{NewProvider, Provider, ProviderResult, RequiredForProvider},
};
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
use journey_db::{
    JourneyDbError,
    entity::{
        MediaItems, ProviderVariant,
        content::{self, ActiveModel, ContentType},
        images::{self},
        media_items::{self, MediaItemType},
        original, providers,
    },
    get_conn,
    sea_orm::{ActiveValue::Set, IntoActiveModel, QuerySelect},
};
use journey_utils::constants::{PRODUCT_NAME, PRODUCT_VERSION};
use serde::Serialize;
use specta::Type;
use std::{fmt::Debug, sync::Arc};
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::warn;
use url::Url;
use uuid::Uuid;

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
    pub(crate) model: Arc<RwLock<providers::ActiveModelEx>>,
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
            model: Arc::new(RwLock::new(model)),
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
    pub async fn get_model(&self) -> providers::ActiveModelEx {
        self.model.read().await.clone()
    }
    pub async fn set_model(&mut self, new: providers::ActiveModelEx) {
        let mut lock = self.model.write().await;
        *lock = new
    }
    pub async fn password_auth(&mut self, uname: String, psw: String) -> ProviderResult<String> {
        let mut client_config = match configure()
            .base_url(&self.url().await?)
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

        self.set_server_id(auth_res.server_id).await?;
        self.set_user_id(auth_res.user).await?;

        client_config = match configure()
            .base_url(&self.url().await?)
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
    pub async fn invalidate(&mut self) -> ProviderResult<()> {
        let mut lock = self.model.write().await;
        *lock = providers::ActiveModelEx::default();
        self.config = None;
        Ok(())
    }
    pub async fn index(&mut self) -> ProviderResult<()> {
        let user_id = &self.user_id().await?.to_string();
        self.index_by_type(user_id, vec![BaseItemKind::MusicAlbum])
            .await?;
        // self.index_by_type(user_id, vec![BaseItemKind::MusicArtist])
        //     .await?;
        // self.index_by_type(user_id, vec![BaseItemKind::Audio])
        //     .await?;
        Ok(())
    }
}

impl JellyfinProvider {
    async fn set_server_id(&mut self, server_id: Option<Option<String>>) -> ProviderResult<()> {
        let server_id = match server_id.flatten() {
            Some(token) => Ok(token),
            None => Err(JellyfinProviderError::ApiEntryRetrievalError(None)),
        }?;

        self.model.write().await.server_id = Set(match Uuid::parse_str(&server_id) {
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
    async fn set_user_id(&mut self, user_dto: Option<Option<Box<UserDto>>>) -> ProviderResult<()> {
        let user_dto = match user_dto.flatten() {
            Some(dto) => Ok(dto),
            None => Err(JellyfinProviderError::ApiEntryRetrievalError(None)),
        }?;

        let user_id = match user_dto.id {
            Some(id) => Ok(id),
            None => Err(JellyfinProviderError::MissingUserIdError),
        }?;

        self.model.write().await.user_id = Set(user_id);
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
    async fn index_by_type(
        &mut self,
        user_id: &str,
        kind: Vec<BaseItemKind>,
    ) -> ProviderResult<()> {
        let items = self.get_items(user_id, kind).await?;

        let tasks = items.iter().map(|item| self.build_media_item(item));
        let media_items = try_join_all(tasks).await?;

        let mut model = self.get_model().await;
        for item in media_items {
            model = model.add_media_item(item);
        }

        let model = match model.save(&get_conn().await?).await {
            Ok(model) => model.into_active_model(),
            Err(err) => {
                return Err(ProviderError::FailedDbInsertError(format!(
                    "{} with specific: {:#?}",
                    err,
                    err.sql_err()
                )));
            }
        };

        self.set_model(model).await;
        Ok(())
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
    async fn get_images(&self, id: Uuid) -> ProviderResult<Vec<images::ActiveModelEx>> {
        warn!("Getting images for: {}", id);

        let mut images: Vec<images::ActiveModelEx> = vec![];

        let images_req = match get_item_image_infos(self.get_config()?, &id.to_string()).await {
            Ok(images) => images,
            Err(err) => {
                return Err(
                    JellyfinProviderError::ApiEntryRetrievalError(Some(err.to_string())).into(),
                );
            }
        };

        let base_url = self.url().await?;
        for image_info in images_req {
            warn!("{:#?}", image_info);

            let ty = match image_info.image_type {
                Some(ty) => self.match_image_type(ty),
                _ => return Err(JellyfinProviderError::ApiEntryRetrievalError(None).into()),
            }?;

            let tag = self.check_entry(image_info.image_tag.flatten())?;

            let url = match Url::parse(&format!("{}/{}/{}", base_url, tag, ty)) {
                Ok(url) => url,
                Err(err) => {
                    return Err(ProviderError::FailedParseUrlError(format!(
                        "failed with: {} for base: {}/{}",
                        err, base_url, ty
                    )));
                }
            };

            let image_model = images::ActiveModel::builder()
                .set_url(url)
                .set_ty(ty)
                .set_server_id(self.server_id().await?);

            images.push(image_model);
        }

        Ok(images)
    }
    fn wrap_content(
        &self,
        description: Option<String>,
        id: &Uuid,
        ty: ContentType,
    ) -> ProviderResult<content::ActiveModelEx> {
        let description = match description {
            Some(description) => Some(description),
            _ => {
                warn!("Skipping: {} for {} because it does not exist", ty, id);
                None
            }
        };

        let smth = ActiveModel::builder()
            .set_description(description)
            .set_parent_id(*id)
            .set_ty(ty);

        Ok(smth)
    }
    async fn get_content(
        &self,
        id: &Uuid,
        item: &BaseItemDto,
    ) -> ProviderResult<Vec<content::ActiveModelEx>> {
        warn!("Getting content for: {} - full item: {:#?}", id, item);

        let album = self.wrap_content(item.album.clone().flatten(), id, ContentType::Album)?;

        let artists = self.wrap_content(
            item.album_artist.clone().flatten(),
            id,
            ContentType::Artists,
        )?;

        let container =
            self.wrap_content(item.container.clone().flatten(), id, ContentType::Container)?;

        let release_date = self.wrap_content(
            Some(
                self.check_entry(item.premiere_date.clone().flatten())?
                    .to_string(),
            ),
            id,
            ContentType::ReleaseDate,
        )?;

        Ok(vec![album, artists, container, release_date])
    }
    async fn get_item_parents(
        &self,
        item: &BaseItemDto,
    ) -> ProviderResult<Vec<media_items::ActiveModel>> {
        warn!("Getting parents for: {:#?}", item);

        let mut parent_ids: Vec<Uuid> = vec![];

        match item.album_id.flatten() {
            Some(id) => parent_ids.push(id),
            _ => warn!(
                "No albums for: {:#?} with id: {:#?} -> skipping",
                item.id, item.name
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
            tasks.push(
                MediaItems::find_by_uuid(id)
                    .select_only()
                    .column(media_items::Column::Uuid)
                    .one(conn),
            );
        }

        let parent_models = match try_join_all(tasks).await {
            Ok(parents) => parents,
            Err(_) => return Err(JourneyDbError::ConnectionError.into()),
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
    async fn build_media_item(
        &self,
        item: &BaseItemDto,
    ) -> ProviderResult<media_items::ActiveModelEx> {
        let item_id = self.check_entry(item.id)?;
        let ty = self.match_item_type(self.check_entry(item.r#type)?)?;
        let images = self.get_images(item_id).await?;
        let content = self.get_content(&item_id, item).await?;
        let parents = self.get_item_parents(item).await?;

        let user_data = self.check_entry(item.user_data.clone().flatten())?;
        let music_brainz_id = self.check_entry(user_data.item_id)?;

        let original = original::ActiveModelEx::new()
            .set_url(self.url().await?)
            .set_uuid(item_id)
            .set_parent_id(music_brainz_id)
            .set_server_id(self.server_id().await?);

        let mut media_item = media_items::ActiveModelEx::new()
            .set_ty(ty)
            .set_uuid(music_brainz_id)
            .add_original(original);

        for image in images {
            media_item = media_item.add_image(image);
        }

        for parent in parents {
            media_item = media_item.add_parent(parent);
        }

        for entry in content {
            media_item = media_item.add_content(entry);
        }

        Ok(media_item)
    }
}

#[async_trait]
#[inherent]
impl Provider for JellyfinProvider {}

#[cfg(test)]
mod variant_jellyfin {
    use std::collections::HashMap;

    use crate::{
        jellyfin_provider::JellyfinProvider,
        provider::{NewProvider, Provider},
    };
    use journey_db::entity::{ProviderVariant, providers::ActiveModelEx};
    use journey_keyring::Entry;
    use journey_utils::get_env_local;
    use serial_test::serial;
    use test_log::test;
    use tracing::warn;
    use url::Url;

    #[test]
    fn matching_name() {
        let model = ActiveModelEx::new().set_url(Url::parse("http://smth.example.com").unwrap());

        assert!(matches!(
            JellyfinProvider::new(model).unwrap().ty(),
            ProviderVariant::JellyfinProvider
        ));
    }

    #[tokio::test]
    #[ignore]
    #[serial]
    async fn try_auth_flow() {
        let env_map = get_env_local();

        let env_map = env_map.unwrap();
        journey_keyring::use_native_store().unwrap();

        warn!("{}", env_map.var("TEST_JELLYFIN_URL").unwrap());
        let url = env_map.var("TEST_JELLYFIN_URL").unwrap();

        let model = ActiveModelEx::new().set_url(Url::parse(&url).unwrap());
        let mut provider = JellyfinProvider::new(model).unwrap();

        assert!(provider.authenticated().await.is_err());
        assert!(provider.server_id().await.is_err());
        assert!(provider.user_id().await.is_err());
        assert!(provider.url().await.is_ok());

        let token = provider
            .password_auth(
                env_map.var("TEST_JELLYFIN_USER").unwrap(),
                env_map.var("TEST_JELLYFIN_PW").unwrap(),
            )
            .await
            .unwrap();
        provider.save_token(&token).await.unwrap();
        provider.add_to_db().await.unwrap();

        assert!(provider.authenticated().await.unwrap() == true);
        assert!(provider.server_id().await.is_ok());
        assert!(provider.user_id().await.is_ok());
        assert!(provider.url().await.is_ok());

        let test = Entry::search(&HashMap::from([("service", "journey")])).unwrap();
        test.iter().for_each(|f| warn!("{:#?}", f.get_password()));

        provider.index().await.unwrap();

        provider.remove_token().await.unwrap();
        provider.remove_from_db().await.unwrap();
        provider.invalidate().await.unwrap();

        assert!(provider.authenticated().await.is_err());
        assert!(provider.server_id().await.is_err());
        assert!(provider.user_id().await.is_err());
        assert!(provider.url().await.is_err());

        journey_keyring::release_store();
    }
}

use crate::{
    ProviderError,
    helpers::get_items_request,
    provider::{NewProvider, Provider, ProviderResult, RequiredForProvider},
};
use async_trait::async_trait;
use futures::future::try_join_all;
use inherent::inherent;
use jellyfin_sdk_rs::{
    apis::{authentication_api::authenticate_user_by_name, configuration::Configuration},
    configure,
    models::{AuthenticateUserByName, BaseItemDto, BaseItemKind, UserDto},
    required::{ClientInfo, DeviceInfo},
};
use journey_db::{
    JourneyDbError,
    entity::{
        MediaItems, ProviderVariant, images,
        media_items::{self, ActiveModelEx, MediaItemType, Model},
        providers,
    },
    get_conn,
    sea_orm::{
        ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, QuerySelect, TryIntoModel,
    },
};
use journey_utils::constants::{PRODUCT_NAME, PRODUCT_VERSION};
use serde::Serialize;
use specta::Type;
use thiserror::Error;
use tracing::warn;
use url::Url;
use uuid::Uuid;

#[derive(Debug, Error, Serialize, Type)]
pub enum JellyfinProviderError {
    #[error("Failed to retrieve Jellyfin API response entry.")]
    ApiEntryRetrievalError,
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
    pub(crate) params: providers::ActiveModel,
    pub(crate) config: Option<Configuration>,
    pub(crate) client_info: ClientInfo,
    pub(crate) device_info: DeviceInfo,
}

impl NewProvider for JellyfinProvider {
    type Provider = JellyfinProvider;

    fn new(params: providers::ActiveModel) -> ProviderResult<Box<Self>> {
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
            params: params,
            config: None,
            client_info,
            device_info,
        }))
    }
}

#[async_trait]
#[inherent]
impl RequiredForProvider for JellyfinProvider {
    pub fn user_id(&self) -> ProviderResult<Uuid> {
        let model = match self.params.clone().try_into_model() {
            Ok(model) => model,
            Err(_) => return Err(ProviderError::FailedConvModelError),
        };

        match model.user_id {
            user_id if user_id != Uuid::nil() => Ok(user_id),
            _ => Err(JellyfinProviderError::MissingServerIdError.into()),
        }
    }
    pub fn server_id(&self) -> ProviderResult<Uuid> {
        let model = match self.params.clone().try_into_model() {
            Ok(model) => model,
            Err(_) => return Err(ProviderError::FailedConvModelError),
        };

        match model.server_id {
            server_id if server_id != Uuid::nil() => Ok(server_id),
            _ => Err(JellyfinProviderError::MissingServerIdError.into()),
        }
    }
    pub fn url(&self) -> ProviderResult<Url> {
        let model = self.params.clone().try_into_model();
        match model {
            Ok(model) => Ok(match Url::parse(&model.url) {
                Ok(model) => model,
                Err(_) => return Err(ProviderError::FailedParseUrlError),
            }),
            Err(_) => Err(JellyfinProviderError::MissingUrlError.into()),
        }
    }
    pub fn ty(&self) -> ProviderVariant {
        ProviderVariant::JellyfinProvider
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
            None => Err(JellyfinProviderError::ApiEntryRetrievalError),
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
    pub async fn invalidate(&mut self) -> ProviderResult<()> {
        self.remove_from_db().await?;
        self.remove_token()?;

        self.params = providers::ActiveModel::default();
        self.config = None;
        Ok(())
    }
    pub async fn index(&self) -> ProviderResult<()> {
        let user_id = &self.user_id()?.to_string();
        self.index_by_type(user_id, vec![BaseItemKind::MusicAlbum])
            .await?;
        self.index_by_type(user_id, vec![BaseItemKind::MusicArtist])
            .await?;
        self.index_by_type(user_id, vec![BaseItemKind::Audio])
            .await?;
        Ok(())
    }
}

impl JellyfinProvider {
    fn set_server_id(&mut self, server_id: Option<Option<String>>) -> ProviderResult<()> {
        let server_id = match server_id.flatten() {
            Some(token) => Ok(token),
            None => Err(JellyfinProviderError::ApiEntryRetrievalError),
        }?;

        self.params.server_id = Set(match Uuid::parse_str(&server_id) {
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
            None => Err(JellyfinProviderError::ApiEntryRetrievalError),
        }?;

        let user_id = match user_dto.id {
            Some(id) => Ok(id),
            None => Err(JellyfinProviderError::MissingUserIdError),
        }?;

        self.params.user_id = Set(user_id);
        Ok(())
    }
    fn get_config(&self) -> ProviderResult<&Configuration> {
        match &self.config {
            Some(config) => Ok(config),
            None => Err(JellyfinProviderError::ApiEntryRetrievalError.into()),
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
    fn check_entry<T>(&self, entry: Option<T>) -> ProviderResult<T> {
        match entry {
            Some(entry) => Ok(entry),
            _ => Err(JellyfinProviderError::ApiEntryRetrievalError.into()),
        }
    }
    async fn index_by_type(&self, user_id: &str, kind: Vec<BaseItemKind>) -> ProviderResult<()> {
        let items = self.get_items(user_id, kind).await?;

        let tasks = items.iter().map(|item| self.build_media_item(item));
        match MediaItems::insert_many(try_join_all(tasks).await?)
            .exec(&get_conn().await?)
            .await
        {
            Ok(_) => Ok(()),
            Err(_) => Err(ProviderError::FailedDbInsertError),
        }
    }
    async fn get_items(
        &self,
        user_id: &str,
        kind: Vec<BaseItemKind>,
    ) -> ProviderResult<Vec<BaseItemDto>> {
        let response = get_items_request()
            .configuration(self.get_config()?)
            .user_id(user_id)
            .recursive(true)
            .include_item_types(kind)
            .call()
            .await?;

        match response.items {
            Some(items) => Ok(items),
            _ => Err(JellyfinProviderError::ApiEntryRetrievalError.into()),
        }
    }
    async fn get_images(&self) -> ProviderResult<Vec<images::ActiveModelEx>> {
        let images: Vec<images::ActiveModelEx> = vec![];

        Ok(images)
    }
    async fn build_media_item(
        &self,
        item: &BaseItemDto,
    ) -> ProviderResult<media_items::ActiveModelEx> {
        let uuid = self.check_entry(item.id)?;
        let ty = self.match_item_type(self.check_entry(item.r#type)?)?;
        let images = self.get_images().await?;
        let parents = self.get_item_parents(&uuid, item).await?;

        let mut media_item = media_items::ActiveModelEx::new().set_uuid(uuid).set_ty(ty);
        for image in images {
            media_item = media_item.add_image(image);
        }

        for parent in parents {
            media_item = media_item.add_parent(parent);
        }

        Ok(media_item)
    }
    async fn get_item_parents(
        &self,
        parent_id: &Uuid,
        item: &BaseItemDto,
    ) -> ProviderResult<Vec<media_items::ActiveModel>> {
        let mut parent_ids: Vec<Uuid> = vec![];

        match item.album_id.flatten() {
            Some(id) => parent_ids.push(id),
            _ => warn!("No albums for: {:#?} with id: {:#?}", parent_id, item.name),
        }

        match item.album_artists.clone().flatten() {
            Some(artists) => {
                for artist in artists {
                    match artist.id {
                        Some(id) => parent_ids.push(id),
                        _ => (),
                    }
                }
            }
            _ => warn!(
                "No album artists for: {:#?} with id: {:#?}",
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
                _ => warn!("Somehow got now model back from Db even though select succeeded"),
            }
        }
        Ok(parents)
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
    use journey_db::{
        entity::{ProviderVariant, providers::ActiveModel},
        sea_orm::{ActiveModelTrait, ActiveValue::Set},
    };
    use journey_keyring::Entry;
    use journey_utils::get_env_local;
    use serial_test::serial;
    use test_log::test;
    use tracing::warn;
    use url::Url;

    #[test]
    fn matching_name() {
        let params = ActiveModel {
            url: Set(Url::parse("http://smth.example.com").unwrap().into()),
            ..Default::default()
        };

        assert!(matches!(
            JellyfinProvider::new(params).unwrap().ty(),
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

        let params = ActiveModel {
            url: Set(Url::parse(&url).unwrap().into()),
            ..ActiveModel::default_values()
        };

        let mut provider = JellyfinProvider::new(params).unwrap();

        provider.authenticated().unwrap();
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

        assert!(provider.authenticated().unwrap() == true);
        assert!(provider.server_id().is_ok());
        assert!(provider.user_id().is_ok());
        assert!(provider.url().is_ok());

        let test = Entry::search(&HashMap::from([("service", "journey")])).unwrap();
        test.iter().for_each(|f| warn!("{:#?}", f.get_password()));

        provider.invalidate().await.unwrap();

        assert!(provider.authenticated().unwrap() == false);
        assert!(provider.server_id().is_err());
        assert!(provider.user_id().is_err());
        assert!(provider.url().is_err());

        journey_keyring::release_store();
    }
}

use std::fmt::Debug;

use async_trait::async_trait;
use inherent::inherent;
use jellyfin_sdk_rs::{
    apis::{authentication_api::authenticate_user_by_name, configuration::Configuration},
    configure,
    models::{AuthenticateUserByName, UserDto},
    required::{ClientInfo, DeviceInfo},
};
use serde::Serialize;
use specta::Type;
use thiserror::Error;
use uuid::Uuid;

use crate::{
    indexer::{Indexer, NewIndexer},
    jellyfin::jellyfin_indexer::JellyfinIndexer,
    provider::{NewProvider, Provider, ProviderResult, RequiredForProvider},
};
use journey_db::entity::{ProviderVariant, providers};
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

    fn new(model: providers::ActiveModelEx) -> Box<Self> {
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

        Box::new(JellyfinProvider {
            model: model,
            config: None,
            client_info,
            device_info,
        })
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
    pub fn get_indexer(&self) -> ProviderResult<Box<dyn Indexer>> {
        Ok(JellyfinIndexer::new(
            self.get_model().clone(),
            Some(self.get_config()?.clone()),
        ))
    }
}

impl JellyfinProvider {
    fn set_server_id(&mut self, server_id: Option<Option<String>>) -> ProviderResult<()> {
        let server_id = match server_id.flatten() {
            Some(token) => Ok(token),
            None => Err(JellyfinProviderError::ApiEntryRetrievalError(None)),
        }?;

        let id = match Uuid::parse_str(&server_id) {
            Ok(uuid) => Ok(uuid),
            Err(_) => Err(JellyfinProviderError::FailedUuidParseError),
        }?;

        Ok(self.model.server_id.set_ne(id))
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

        Ok(self.model.user_id.set_ne(user_id))
    }
    fn get_config(&self) -> ProviderResult<&Configuration> {
        match &self.config {
            Some(config) => Ok(config),
            None => Err(JellyfinProviderError::ApiEntryRetrievalError(None).into()),
        }
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
    use journey_db::entity::{ProviderVariant, providers};
    use journey_keyring::Entry;
    use journey_utils::get_env_local;

    #[test]
    fn matching_name() {
        let model =
            providers::ActiveModelEx::new().set_url(Url::parse("http://smth.example.com").unwrap());

        assert!(matches!(
            JellyfinProvider::new(model).ty(),
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

        let model = providers::ActiveModelEx::new().set_url(Url::parse(&url).unwrap());
        let mut provider = JellyfinProvider::new(model);

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

        let (_tx, mut _rx): (Sender<IndexerMsg>, Receiver<IndexerMsg>) = mpsc::channel(100);
        let _indexer = provider.get_indexer().unwrap();

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

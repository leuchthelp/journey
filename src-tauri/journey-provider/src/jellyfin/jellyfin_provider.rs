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
    ProviderError,
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
    #[error("Failed to build client config for SDK client.")]
    FailedBuildConfigError(String),
}

#[derive(Debug, Clone)]
pub struct JellyfinProvider {
    model: providers::ActiveModelEx,
    config: Option<Configuration>,
    client_info: ClientInfo,
    device_info: DeviceInfo,
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
    pub fn get_indexer(&self) -> ProviderResult<Box<dyn Indexer + Send + Sync>> {
        Ok(JellyfinIndexer::new(
            self.get_model().clone(),
            Some(self.get_config()?.clone()),
        ))
    }
    pub async fn password_auth(&mut self, uname: String, psw: String) -> ProviderResult<String> {
        let client_config = match configure()
            .base_url(&self.url()?)
            .client_info(&self.client_info)
            .device_info(&self.device_info)
            .call()
        {
            Ok(config) => Ok(config),
            Err(err) => Err(JellyfinProviderError::FailedBuildConfigError(
                err.to_string(),
            )),
        }?;

        let auth_by_name = AuthenticateUserByName {
            username: Some(Some(uname)),
            pw: Some(Some(psw)),
        };
        let auth_res = match authenticate_user_by_name(&client_config, auth_by_name).await {
            Ok(res) => res,
            Err(_) => return Err(ProviderError::FailedPasswordAuthError),
        };

        let access_token = match auth_res.access_token.flatten() {
            Some(token) => Ok(token),
            None => Err(JellyfinProviderError::ApiEntryRetrievalError(None)),
        }?;

        self.set_server_id(auth_res.server_id)?;
        self.set_user_id(auth_res.user)?;

        let client_config = match configure()
            .base_url(&self.url()?)
            .client_info(&self.client_info)
            .device_info(&self.device_info)
            .access_token(&access_token)
            .call()
        {
            Ok(config) => Ok(Some(config)),
            Err(err) => Err(JellyfinProviderError::FailedBuildConfigError(
                err.to_string(),
            )),
        }?;

        self.config = client_config;
        Ok(access_token)
    }
}

impl JellyfinProvider {
    fn set_server_id(&mut self, server_id: Option<Option<String>>) -> ProviderResult<()> {
        let server_id = match server_id.flatten() {
            Some(token) => Ok(token),
            None => Err(JellyfinProviderError::ApiEntryRetrievalError(None)),
        }?;

        let id = match Uuid::parse_str(&server_id) {
            Ok(uuid) => uuid,
            Err(_) => return Err(ProviderError::FailedUuidParseError),
        };

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
            Some(id) => id,
            None => return Err(ProviderError::MissingUserIdError),
        };

        Ok(self.model.user_id.set_ne(user_id))
    }
    fn get_config(&self) -> ProviderResult<&Configuration> {
        match &self.config {
            Some(config) => Ok(config),
            None => Err(JellyfinProviderError::ApiEntryRetrievalError(None).into()),
        }
    }
}

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
        indexer::IndexerMsg,
        jellyfin_provider::JellyfinProvider,
        provider::{NewProvider, Provider},
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

        let model = providers::ActiveModelEx::new()
            .set_url(Url::parse(&url).unwrap())
            .set_ty(ProviderVariant::JellyfinProvider);
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

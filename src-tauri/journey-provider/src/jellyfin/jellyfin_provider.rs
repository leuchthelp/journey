use async_trait::async_trait;
use inherent::inherent;
use jellyfin_sdk_rs::{
    apis::{authentication_api::authenticate_user_by_name, configuration::Configuration},
    configure,
    models::{AuthenticateUserByName, UserDto},
    required::{ClientInfo, DeviceInfo},
};
use journey_db::{
    entity::{ProviderVariant, providers::ActiveModel},
    sea_orm::{ActiveValue::Set, TryIntoModel},
};
use journey_utils::constants::{PRODUCT_NAME, PRODUCT_VERSION};
use thiserror::Error;
use url::Url;
use uuid::Uuid;

use crate::provider::{Provider, ProviderNew, ProviderResult};

use serde::Serialize;
use specta::Type;

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
}

#[derive(Debug, Clone)]
pub struct JellyfinProvider {
    pub(crate) params: ActiveModel,
    pub(crate) config: Option<Configuration>,
    pub(crate) client_info: ClientInfo,
    pub(crate) device_info: DeviceInfo,
}

impl ProviderNew for JellyfinProvider {
    type Provider = JellyfinProvider;

    fn new(params: ActiveModel) -> ProviderResult<Box<Self>> {
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
impl Provider for JellyfinProvider {
    pub fn user_id(&self) -> ProviderResult<Uuid> {
        let model = self.params.clone().try_into_model()?;

        match model.user_id {
            user_id if user_id != Uuid::nil() => Ok(user_id),
            _ => Err(JellyfinProviderError::MissingServerIdError.into()),
        }
    }

    pub fn server_id(&self) -> ProviderResult<Uuid> {
        let model = self.params.clone().try_into_model()?;

        match model.server_id {
            server_id if server_id != Uuid::nil() => Ok(server_id),
            _ => Err(JellyfinProviderError::MissingServerIdError.into()),
        }
    }

    pub fn url(&self) -> ProviderResult<Url> {
        let model = self.params.clone().try_into_model();
        match model {
            Ok(model) => Ok(Url::parse(&model.url)?),
            Err(_) => Err(JellyfinProviderError::MissingUrlError.into()),
        }
    }

    pub fn ty(&self) -> ProviderVariant {
        ProviderVariant::JellyfinProvider
    }

    pub async fn password_auth(
        &mut self,
        uname: String,
        psw: String,
    ) -> ProviderResult<(Uuid, Uuid)> {
        if self.authenticated()? {
            return Ok(self.key()?);
        }
        let mut client_config = configure()
            .base_url(&self.url()?)
            .client_info(&self.client_info)
            .device_info(&self.device_info)
            .call()?;

        let auth_by_name = AuthenticateUserByName {
            username: Some(Some(uname)),
            pw: Some(Some(psw)),
        };
        let auth_res = authenticate_user_by_name(&client_config, auth_by_name).await?;

        let access_token = match auth_res.access_token.flatten() {
            Some(token) => Ok(token),
            None => Err(JellyfinProviderError::ApiEntryRetrievalError),
        }?;

        self.set_server_id(auth_res.server_id)?;
        self.set_user_id(auth_res.user)?;
        let add_db_task = self.add_to_db();

        client_config = configure()
            .base_url(&self.url()?)
            .client_info(&self.client_info)
            .device_info(&self.device_info)
            .access_token(&access_token)
            .call()?;

        add_db_task.await?;
        self.save_token(&access_token)?;
        self.config = Some(client_config);
        Ok(self.key()?)
    }

    pub async fn invalidate(&mut self) -> ProviderResult<()> {
        let remove_db_task = self.remove_from_db();
        self.remove_token()?;
        remove_db_task.await?;

        self.params = ActiveModel::default();
        self.config = None;
        Ok(())
    }
}

impl JellyfinProvider {
    fn set_server_id(&mut self, server_id: Option<Option<String>>) -> ProviderResult<()> {
        let server_id = match server_id.flatten() {
            Some(token) => Ok(token),
            None => Err(JellyfinProviderError::ApiEntryRetrievalError),
        }?;

        self.params.server_id = Set(Uuid::parse_str(&server_id)?);
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
}

#[cfg(test)]
mod variant_jellyfin {
    use std::collections::HashMap;

    use crate::{
        jellyfin_provider::JellyfinProvider,
        provider::{Provider, ProviderNew},
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

        provider
            .password_auth(
                env_map.var("TEST_JELLYFIN_USER").unwrap(),
                env_map.var("TEST_JELLYFIN_PW").unwrap(),
            )
            .await
            .unwrap();

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

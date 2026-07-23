use async_trait::async_trait;
use jellyfin_sdk_rs::{
    apis::{authentication_api::authenticate_user_by_name, configuration::Configuration},
    configure,
    models::{AuthenticateUserByName, UserDto},
    required::{ClientInfo, DeviceInfo},
};
use journey_utils::get_env_prod;
use thiserror::Error;
use url::Url;
use uuid::Uuid;

use crate::provider::{Provider, ProviderNew, ProviderParams, ProviderResult};

#[derive(Error, Debug)]
pub enum JellyfinProviderError {
    #[error("Failed to retrieve Jellyfin API response entry.")]
    ApiEntryRetrievalError,
    #[error("server_id hasn't been set yet, try authenticating first.")]
    MissingServerIdError,
    #[error("user_id hasn't been set yet, try authenticating first.")]
    MissingUserIdError,
}

#[derive(Debug)]
pub struct JellyfinProvider {
    params: ProviderParams,
    config: Option<Configuration>,
    client_info: ClientInfo,
    device_info: DeviceInfo,
    authenticated: bool,
}

#[async_trait]
impl ProviderNew<JellyfinProvider> for JellyfinProvider {
    fn new(params: ProviderParams) -> ProviderResult<Self> {
        let client_info = ClientInfo {
            name: get_env_prod()?.var("VITE_JOURNEY_NAME")?,
            version: get_env_prod()?.var("VITE_JOURNEY_VERSION")?.to_string(),
        };

        let device_info = DeviceInfo {
            id: Uuid::now_v7().to_string(),
            name: format!(
                "{}-{}-{}-{}",
                tauri_plugin_os::hostname(),
                tauri_plugin_os::platform(),
                tauri_plugin_os::arch(),
                tauri_plugin_os::version()
            ),
            languages: None,
        };

        Ok(JellyfinProvider {
            params,
            config: None,
            client_info,
            device_info,
            authenticated: false,
        })
    }

    async fn authenticate_with_pw(&mut self, uname: String, psw: String) -> ProviderResult<()> {
        if self.authenticated {
            Ok(())
        } else {
            let mut client_config = configure()
                .base_url(self.url())
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

            client_config = configure()
                .base_url(self.url())
                .client_info(&self.client_info)
                .device_info(&self.device_info)
                .access_token(&access_token)
                .call()?;

            self.save_token(&access_token)?;
            self.config = Some(client_config);
            self.authenticated = true;
            Ok(())
        }
    }
}

impl Provider for JellyfinProvider {
    fn user_id(&self) -> ProviderResult<Uuid> {
        let res = match self.params.user_id {
            Some(server_id) => Ok(server_id),
            None => Err(JellyfinProviderError::MissingUserIdError),
        };

        Ok(res?)
    }

    fn server_id(&self) -> ProviderResult<Uuid> {
        let res = match self.params.server_id {
            Some(server_id) => Ok(server_id),
            None => Err(JellyfinProviderError::MissingServerIdError),
        };

        Ok(res?)
    }

    fn url(&self) -> &Url {
        &self.params.url
    }

    fn authenticated(&self) -> &bool {
        &self.authenticated
    }

    fn invalidate(&mut self) -> ProviderResult<()> {
        self.remove_token()?;

        self.params = ProviderParams {
            user_id: None,
            server_id: None,
            url: self.url().clone(),
        };
        self.config = None;
        self.authenticated = false;
        Ok(())
    }
}

impl JellyfinProvider {
    fn set_server_id(&mut self, server_id: Option<Option<String>>) -> ProviderResult<()> {
        let server_id = match server_id.flatten() {
            Some(token) => Ok(token),
            None => Err(JellyfinProviderError::ApiEntryRetrievalError),
        }?;

        self.params.server_id = Some(Uuid::parse_str(&server_id)?);
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
    fn set_user_id(&mut self, user_id: Option<Option<Box<UserDto>>>) -> ProviderResult<()> {
        let user_id = match user_id.flatten() {
            Some(dto) => Ok(dto),
            None => Err(JellyfinProviderError::ApiEntryRetrievalError),
        }?;

        self.params.user_id = Some(match user_id.id {
            Some(id) => Ok(id),
            None => Err(JellyfinProviderError::MissingUserIdError),
        }?);

        Ok(())
    }
}

#[cfg(test)]
mod variant_jellyfin {
    use std::collections::HashMap;

    use crate::{
        jellyfin_provider::JellyfinProvider,
        provider::{Provider, ProviderNew, ProviderParams},
    };
    use journey_keyring::Entry;
    use journey_utils::get_env_local;
    use serial_test::serial;
    use test_log::test;
    use tracing::warn;
    use url::Url;

    #[test]
    fn matching_name() {
        assert!(
            JellyfinProvider::new(ProviderParams {
                url: Url::parse("http://example.net").unwrap(),
                user_id: None,
                server_id: None
            })
            .unwrap()
            .type_()
            .contains("JellyfinProvider")
        );
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
        let mut provider = JellyfinProvider::new(ProviderParams {
            url: Url::parse(&url).unwrap(),
            user_id: None,
            server_id: None,
        })
        .unwrap();

        assert!(*provider.authenticated() == false);
        assert!(provider.server_id().is_err());
        assert!(provider.user_id().is_err());

        provider
            .authenticate_with_pw(
                env_map.var("TEST_JELLYFIN_USER").unwrap(),
                env_map.var("TEST_JELLYFIN_PW").unwrap(),
            )
            .await
            .unwrap();

        assert!(*provider.authenticated() == true);
        assert!(provider.server_id().is_ok());
        assert!(provider.user_id().is_ok());

        let test = Entry::search(&HashMap::from([("service", "journey")])).unwrap();
        test.iter().for_each(|f| warn!("{:#?}", f.get_password()));

        provider.invalidate().unwrap();

        assert!(*provider.authenticated() == false);
        assert!(provider.server_id().is_err());
        assert!(provider.user_id().is_err());

        journey_keyring::release_store();
    }
}

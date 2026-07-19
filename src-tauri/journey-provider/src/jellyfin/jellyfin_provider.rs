use dotenvy::{EnvLoader, EnvMap};
use jellyfin_sdk_rs::{
    apis::{
        authentication_api::{AuthenticateUserByNameError, authenticate_user_by_name},
        configuration::Configuration,
    },
    configure,
    models::AuthenticateUserByName,
    required::{ClientInfo, DeviceInfo},
};
use journey_keyring::Entry;
use thiserror::Error;
use url::Url;
use uuid::Uuid;

use crate::{Provider, ProviderParams};

fn get_env() -> EnvMap {
    return EnvLoader::with_path("../../.env.production")
        .load()
        .unwrap();
}

#[derive(Error, Debug)]
pub enum JellyfinProviderError {
    #[error("Failed to save access token to native keyring: {0}")]
    SetTokenFailureError(#[from] journey_keyring::keyring_core::Error),
    #[error(transparent)]
    AuthenticationError(#[from] jellyfin_sdk_rs::apis::Error<AuthenticateUserByNameError>),
    #[error(transparent)]
    UuidParserError(#[from] uuid::Error),
    #[error(transparent)]
    ConfigurationError(#[from] Box<dyn std::error::Error>),
    #[error("Failed to retrieve Jellyfin API response entry.")]
    ApiEntryRetrievalError,
    #[error("server_id hasn't been set yet, try authenticating first.")]
    MissingServerIdError,
    #[error("user_id hasn't been set yet, try authenticating first.")]
    MissingUserIdError,
}

pub struct JellyfinProvider {
    params: ProviderParams,
    config: Option<Configuration>,
    client_info: ClientInfo,
    device_info: DeviceInfo,
    authenticated: bool,
}

impl Provider<JellyfinProviderError> for JellyfinProvider {
    fn new(params: ProviderParams) -> Self {
        let client_info = ClientInfo {
            name: get_env().var("VITE_JOURNEY_NAME").unwrap(),
            version: get_env().var("VITE_JOURNEY_VERSION").unwrap().to_string(),
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

        JellyfinProvider {
            params,
            config: None,
            client_info,
            device_info,
            authenticated: false,
        }
    }

    fn user_id(&self) -> Result<Uuid, JellyfinProviderError> {
        let res = match self.params.user_id {
            Some(server_id) => Ok(server_id),
            None => Err(JellyfinProviderError::MissingUserIdError),
        };

        Ok(res?)
    }

    fn server_id(&self) -> Result<Uuid, JellyfinProviderError> {
        let res = match self.params.server_id {
            Some(server_id) => Ok(server_id),
            None => Err(JellyfinProviderError::MissingServerIdError),
        };

        Ok(res?)
    }

    fn url(&self) -> &Url {
        &self.params.url
    }

    fn authenticated(&self) -> bool {
        self.authenticated
    }

    async fn authenticate_with_pw(
        &mut self,
        uname: String,
        psw: String,
    ) -> Result<(), JellyfinProviderError> {
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
            self.params.user_id = Some(auth_res.user.flatten().unwrap().id.unwrap());

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

impl JellyfinProvider {
    fn save_token(&self, access_token: &String) -> Result<(), JellyfinProviderError> {
        let token_entry = Entry::new(
            &get_env().var("VITE_JOURNEY_NAME").unwrap(),
            format!("{}-{}", self.server_id()?, self.user_id()?).as_str(),
        )?;
        token_entry.set_password(&access_token)?;

        Ok(())
    }

    fn set_server_id(
        &mut self,
        server_id: Option<Option<String>>,
    ) -> Result<(), JellyfinProviderError> {
        let server_id = match server_id.flatten() {
            Some(token) => Ok(token),
            None => Err(JellyfinProviderError::ApiEntryRetrievalError),
        }?;

        self.params.server_id = Some(Uuid::parse_str(&server_id)?);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::jellyfin_provider::JellyfinProvider;
    use crate::{Provider, ProviderParams};
    use dotenvy::EnvLoader;
    use journey_keyring::Entry;
    use url::Url;

    #[test]
    fn matching_name() {
        assert!(
            JellyfinProvider::new(ProviderParams {
                url: Url::parse("http://example.net").unwrap(),
                user_id: None,
                server_id: None
            })
            .type_()
            .contains("JellyfinProvider")
        );
    }

    #[tokio::test]
    #[ignore]
    async fn try_auth() {
        let env_map = EnvLoader::with_path("../../.env.local").load();
        if env_map.is_err() {
            assert!(true)
        } else {
            let env_map = env_map.unwrap();
            journey_keyring::use_native_store().unwrap();

            println!("{}", env_map.var("TEST_JELLYFIN_URL").unwrap());
            let url = env_map.var("TEST_JELLYFIN_URL").unwrap();
            let mut provider = JellyfinProvider::new(ProviderParams {
                url: Url::parse(&url).unwrap(),
                user_id: None,
                server_id: None,
            });

            assert!(provider.authenticated() == false);
            assert!(provider.server_id().is_err());
            assert!(provider.user_id().is_err());

            provider
                .authenticate_with_pw(
                    env_map.var("TEST_JELLYFIN_USER").unwrap(),
                    env_map.var("TEST_JELLYFIN_PW").unwrap(),
                )
                .await
                .unwrap();

            assert!(provider.authenticated() == true);
            assert!(provider.server_id().is_ok());
            assert!(provider.user_id().is_ok());

            let test = Entry::search(&HashMap::from([("service", "journey")])).unwrap();
            test.iter()
                .for_each(|f| println!("{:#?}", f.get_password()));
            journey_keyring::release_store();
        }
    }
}

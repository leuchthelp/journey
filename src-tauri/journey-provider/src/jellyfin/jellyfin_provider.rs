use jellyfin_sdk_rs::{
    apis::{authentication_api::authenticate_user_by_name, configuration::Configuration},
    configure,
    models::AuthenticateUserByName,
    required::{ClientInfo, DeviceInfo},
};
use journey_db::uuid::Uuid;
use std::error::Error;
use url::Url;

use crate::{Provider, ProviderParams};

pub struct JellyfinProvider {
    params: ProviderParams,
    config: Option<Configuration>,
    client_info: ClientInfo,
    device_info: DeviceInfo,
    authenticated: bool,
}

impl Provider for JellyfinProvider {
    fn new(params: ProviderParams) -> Self {
        let client_info = ClientInfo {
            name: "journey".to_string(),
            version: "0.1.0".to_string(),
        };

        let device_info = DeviceInfo {
            id: Uuid::now_v7().to_string(),
            name: "pc".to_string(),
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

    fn user_id(&self) -> Uuid {
        self.params.user_id
    }

    fn server_id(&self) -> Uuid {
        self.params.server_id
    }

    fn url(&self) -> &Url {
        &self.params.url
    }

    fn params(&self) -> &ProviderParams {
        &self.params
    }

    async fn authenticate_with_pw(
        mut self,
        uname: String,
        psw: String,
    ) -> Result<(), Box<dyn Error>> {
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

            let access_token = match auth_res.access_token.unwrap() {
                None => {
                    panic!("Result lacked access token supply even though we succeeded with auth.")
                }
                Some(token) => token,
            };

            client_config = configure()
                .base_url(self.url())
                .client_info(&self.client_info)
                .device_info(&self.device_info)
                .access_token(access_token)
                .call()?;

            self.config = Some(client_config);
            self.authenticated = true;
            Ok(())
        }
    }
}

impl JellyfinProvider {
    fn test(self) {
        self.server_id();
    }
}

#[cfg(test)]
mod tests {
    use crate::provider::jellyfin_provider::JellyfinProvider;
    use crate::provider::{Provider, ProviderParams};
    use journey_db::uuid::Uuid;
    use url::Url;

    #[test]
    fn matching_name() {
        assert!(
            JellyfinProvider::new(ProviderParams {
                url: Url::parse("http://example.net").unwrap(),
                user_id: Uuid::now_v7(),
                server_id: Uuid::now_v7()
            })
            .type_()
            .contains("JellyfinProvider")
        );
    }
}

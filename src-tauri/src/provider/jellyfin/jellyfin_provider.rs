use jellyfin_sdk_rs::apis::configuration::Configuration;
use url::Url;
use uuid::Uuid;

use crate::provider::{Provider, ProviderParams};

pub struct JellyfinProvider {
    params: ProviderParams,
    config: Configuration,
    authenticated: bool,
}

impl Provider for JellyfinProvider {
    fn new(params: ProviderParams) -> Self {
        JellyfinProvider {
            params: params,
            config: Configuration::default(),
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
    use url::Url;
    use uuid::Uuid;

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

use jellyfin_sdk_rs::apis::configuration::Configuration;
use uuid::Uuid;

use crate::provider::jellyfin::Provider;

#[derive(Default)]
pub struct JellyfinProvider {
    pub user_id: Uuid,
    pub server_id: Uuid,
    pub url: String,
    config: Configuration,
    authenticated: bool,
}

impl Provider for JellyfinProvider {
    fn new(user_id: Uuid, server_id: Uuid, url: String) -> Self {
        JellyfinProvider {
            user_id: user_id,
            server_id: server_id,
            url: url,
            ..Default::default()
        }
    }
}

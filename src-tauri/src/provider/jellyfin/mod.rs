pub mod jellyfin_provider;
pub mod helpers;

use uuid::Uuid;

pub trait Provider {
    fn new(user_id: Uuid, server_id: Uuid, url: String) -> Self;
}
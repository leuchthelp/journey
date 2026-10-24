pub mod provider_api;
pub mod simple;

use std::sync::Arc;

pub use journey_media_item::MediaItemManager;
pub use journey_playback::audio_player::AudioPlayer;
pub use journey_provider::{ProviderManager, ProviderManagerFn};
pub use kameo::actor::{ActorRef, Spawn};
pub use tokio::sync::RwLock;

pub use journey_db::entity::providers::*;

pub struct AppStateInner {
    pub provider_manager: Arc<RwLock<ProviderManager>>,
    pub media_item_manager: MediaItemManager,
    pub audio_player: ActorRef<AudioPlayer>,
}

pub type AppState = Arc<AppStateInner>;

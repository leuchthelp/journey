pub mod simple;
pub mod provider_api;

use std::sync::Arc;

use anyhow::Result;
use journey_media_item::MediaItemManager;
use journey_playback::audio_player::AudioPlayer;
use journey_provider::{ProviderManager, ProviderManagerFn};
use kameo::actor::{ActorRef, Spawn};
use tokio::sync::RwLock;

pub struct AppStateInner {
    pub provider_manager: RwLock<ProviderManager>,
    pub media_item_manager: MediaItemManager,
    pub audio_player: ActorRef<AudioPlayer>,
}

pub type AppState = Arc<AppStateInner>;

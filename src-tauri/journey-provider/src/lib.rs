mod jellyfin;
pub use jellyfin::jellyfin_provider;

mod provider;
mod provider_manager;

pub use provider::ProviderError;
pub use provider::ProviderResult;

pub use provider_manager::ProviderManager;
pub use provider_manager::ProviderManagerError;
pub use provider_manager::ProviderManagerFn;
pub use provider_manager::ProviderManagerResult;

mod indexer;
mod indexer_manager;
mod indexer_runner;
mod progress_tracker;

pub use indexer::IndexerMsg;

pub use indexer_manager::IndexerKey;
pub use indexer_manager::IndexerManagerError;
pub use indexer_runner::GetProgress;

pub use progress_tracker::ProgressTrackerError;
pub use progress_tracker::ProgressTrackerMsg;

mod helpers;

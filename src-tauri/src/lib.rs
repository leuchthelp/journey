use anyhow::Result;
use journey_provider::{ProviderManager, ProviderManagerFn};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[taurpc::procedures]
trait Api {}

#[derive(Clone, Debug)]
struct ApiImpl;

#[taurpc::resolvers]
impl Api for ApiImpl {}

struct AppData {
    provider_manager: ProviderManager,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() -> Result<()> {
    let router = taurpc::Router::new().merge(ApiImpl.into_handler());

    journey_db::init_db().await?;

    let mut provider_manager = ProviderManager::default();
    provider_manager.init().await?;

    let app_data = AppData { provider_manager };

    #[cfg(debug_assertions)]
    taurpc::Exporter::new().export(&router, "../src/bindings.ts")?;

    tauri::Builder::default()
        .manage(app_data)
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_persisted_scope::init())
        .invoke_handler(router.into_handler())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}

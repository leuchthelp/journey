use anyhow::Result;
use journey_api::get_router;
use journey_provider::{ProviderManager, ProviderManagerFn};

struct AppData {
    provider_manager: ProviderManager,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() -> Result<()> {
    let router = get_router()?;

    let mut provider_manager = ProviderManager::default();
    provider_manager.init().await?;

    let app_data = AppData { provider_manager };

    #[cfg(debug_assertions)]
    taurpc::Exporter::new().export(&router, "../src/lib/bindings.ts")?;

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

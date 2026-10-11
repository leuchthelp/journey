use anyhow::Result;
use journey_api::get_router;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() -> Result<()> {
    let router = get_router().await?;

    #[cfg(debug_assertions)]
    taurpc::Exporter::new()
        .error_handling(taurpc::ErrorHandlingMode::Result)
        .export(&router, "../src/lib/bindings.ts")?;

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_persisted_scope::init())
        .invoke_handler(router.into_handler())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}

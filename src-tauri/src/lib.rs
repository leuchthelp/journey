// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tracing::info;

#[taurpc::procedures]
trait Api {}

#[derive(Clone, Debug)]
struct ApiImpl;

use sea_orm::ActiveValue::Set;

#[taurpc::resolvers]
impl Api for ApiImpl {}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    let router = taurpc::Router::new().merge(ApiImpl.into_handler());

    journey_db::init_db().await.unwrap();

    #[cfg(debug_assertions)]
    taurpc::Exporter::new()
        .export(&router, "../src/bindings.ts")
        .unwrap();

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_persisted_scope::init())
        .invoke_handler(router.into_handler())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

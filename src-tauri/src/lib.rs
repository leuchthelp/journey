// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use journey_db::db as database;
use journey_db::entity::media_items::ConvertableMediaItems;
use journey_db::entity::{MediaItemsDTO, media_items};
use journey_provider::jellyfin_provider::JellyfinProvider;
use journey_provider::{Provider, ProviderParams};
use uuid::Uuid;

#[taurpc::procedures]
trait Api {
    async fn select() -> MediaItemsDTO;
    async fn insert() -> MediaItemsDTO;
}

#[derive(Clone)]
struct ApiImpl;

use sea_orm::ActiveValue::Set;

#[taurpc::resolvers]
impl Api for ApiImpl {
    async fn select(self) -> MediaItemsDTO {
        return database::select().await;
    }
    async fn insert(self) -> MediaItemsDTO {
        let amodel = media_items::ActiveModel {
            uuid: Set(Uuid::now_v7()),
            outline_gradient: Set("test".to_string()),
            kind: Set("SongItem".to_string()),
            loaded: Set(false),
            local: Set("".to_string()),
        };

        let tmp = database::insert(amodel).await;
        log::info!("{:#?}", tmp);
        return MediaItemsDTO::from_model(tmp.into_ex());
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    let router = taurpc::Router::new().merge(ApiImpl.into_handler());

    #[cfg(debug_assertions)]
    taurpc::Exporter::new()
        .export(&router, "../src/bindings.ts")
        .unwrap();

    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(tauri_plugin_log::log::LevelFilter::Info)
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::Webview,
                ))
                .build(),
        )
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_persisted_scope::init())
        .invoke_handler(router.into_handler())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

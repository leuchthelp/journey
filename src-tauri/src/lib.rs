use tauri_plugin_sql::{Migration, MigrationKind};
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use musicbrainz_rs::entity::artist::*;
use musicbrainz_rs::prelude::*;

#[tauri::command]
fn test2() {
    let nirvana = Artist::fetch()
        .id("5b11f4ce-a62d-471e-81fc-a69a8278c7da")
        .execute()
        .unwrap();

    let tmp = nirvana.name;
    println!("{tmp}");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let migrations = vec![
        // Define your migrations here
        Migration {
            version: 1,
            description: "create_initial_tables",
            sql: include_str!("../migrations/20260606211500_colossal_payback/migration.sql"),
            kind: MigrationKind::Up,
        }    
    ];

    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_persisted_scope::init())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:dev.db", migrations)
                .build(),
        )
        .invoke_handler(tauri::generate_handler![test2])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

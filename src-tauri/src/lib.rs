use tauri_plugin_sql::{Migration, MigrationKind};
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

//use musicbrainz_rs::entity::artist::*;
//use musicbrainz_rs::prelude::*;

// #[tauri::command]
// fn test() {
//     let nirvana = Artist::fetch()
//         .id("5b11f4ce-a62d-471e-81fc-a69a8278c7da")
//         .execute();

//     let tmp = nirvana?.name;
//     println!("{tmp}");
// }

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let migrations = vec![
        // Define your migrations here
        Migration {
            version: 1,
            description: "create_initial_tables",
            sql: include_str!("../migrations/20260516160712_married_switch.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 2,
            description: "add item type",
            sql: include_str!("../migrations/20260517092712_mushy_giant_man.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 3,
            description: "make all none null",
            sql: include_str!("../migrations/20260517102756_loose_pandemic.sql"),
            kind: MigrationKind::Up,
        },
    ];

    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_persisted_scope::init())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:dev.db", migrations)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        //.invoke_handler(tauri::generate_handler![test])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

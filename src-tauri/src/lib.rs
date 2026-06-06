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
            sql: include_str!("../migrations/20260516160712_married_switch/migration.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 2,
            description: "add item type",
            sql: include_str!("../migrations/20260517092712_mushy_giant_man/migration.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 3,
            description: "make all none null",
            sql: include_str!("../migrations/20260517102756_loose_pandemic/migration.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 4,
            description: "integrate betterauth",
            sql: include_str!("../migrations/20260527141800_last_black_tom/migration.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 5,
            description: "add server table",
            sql: include_str!("../migrations/20260531090213_dear_random/migration.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 6,
            description: "change type of server id",
            sql: include_str!("../migrations/20260531091022_dark_mentallo/migration.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 7,
            description: "add provider type",
            sql: include_str!("../migrations/20260531095232_gifted_mariko_yashida/migration.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 8,
            description: "rename",
            sql: include_str!("../migrations/20260531111218_thin_sersi/migration.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 9,
            description: "add provider user",
            sql: include_str!("../migrations/20260601151022_material_timeslip/migration.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 10,
            description: "separate serverId & userId",
            sql: include_str!("../migrations/20260601202016_tranquil_mephisto/migration.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 11,
            description: "confused",
            sql: include_str!("../migrations/20260601214120_harsh_marvel_apes/migration.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 12,
            description: "new internal datastructure",
            sql: include_str!("../migrations/20260604161431_fat_hemingway/migration.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 13,
            description: "fix some attributes",
            sql: include_str!("../migrations/20260606115628_many_blacklash/migration.sql"),
            kind: MigrationKind::Up,
        },
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

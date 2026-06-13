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
    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_persisted_scope::init())
        .invoke_handler(tauri::generate_handler![test2])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

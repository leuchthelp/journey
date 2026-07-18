// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tokio::main]
async fn main() {
    journey_keyring::use_native_store().unwrap();
    journey_lib::run().await;
    journey_keyring::release_store();
}

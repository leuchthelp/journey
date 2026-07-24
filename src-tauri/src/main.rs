// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use anyhow::Result;
use tracing::info;
use tracing_subscriber::{self};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        // filter spans/events with level TRACE or higher.
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .pretty()
        // build but do not install the subscriber.
        .init();

    info!("here we go");

    journey_keyring::use_native_store().unwrap();
    journey_lib::run().await?;
    journey_keyring::release_store();
    Ok(())
}

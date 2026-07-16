use jellyfin_sdk_rs::{self as sdk, models::BaseItemKind};
use url::Url;

use crate::provider::{jellyfin::helpers, jellyfin_provider::JellyfinProvider};

pub async fn test() {
    let device_info = sdk::required::DeviceInfo {
        id: uuid::Uuid::now_v7().to_string(),
        name: "pc".to_string(),
        languages: None,
    };

    let client_info = sdk::required::ClientInfo {
        name: "journey".to_string(),
        version: "0.1.0".to_string(),
    };

    let mut client_config = sdk::configure()
        .base_url(Url::parse("https://music.leuchtapp.com").unwrap())
        .client_info(&client_info)
        .device_info(&device_info)
        .access_token(&None)
        .call()
        .unwrap();

    println!("{:#?}", client_config);

    let auth_by_name = sdk::models::authenticate_user_by_name::AuthenticateUserByName {
        username: Some(Some("leucht".to_string())),
        pw: Some(Some("taAgfdP0NcVy915o4NJkhyM41qjoZ7ZW".to_string())),
    };
    let auth_res =
        sdk::apis::authentication_api::authenticate_user_by_name(&client_config, auth_by_name);

    let res = auth_res.await;

    client_config = sdk::configure()
        .base_url(Url::parse("https://music.leuchtapp.com").unwrap())
        .client_info(&client_info)
        .device_info(&device_info)
        .access_token(&res.unwrap().access_token.unwrap())
        .call()
        .unwrap();

    let items = helpers::get_items()
        .configuration(&client_config)
        .recursive(true)
        .include_item_types(vec![
            BaseItemKind::Audio,
            BaseItemKind::MusicAlbum,
            BaseItemKind::MusicArtist,
        ])
        .call();

    let res = items.await;
    log::info!("{:#?}", res);
}

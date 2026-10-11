pub mod constants;

use dotenvy::{EnvLoader, EnvMap};
pub fn get_env_prod() -> Result<EnvMap, dotenvy::Error> {
    return EnvLoader::with_path("../../.env.production").load();
}

pub fn get_env_local() -> Result<EnvMap, dotenvy::Error> {
    return EnvLoader::with_path("../../.env.local").load();
}

pub fn get_env_location(location: String) -> Result<EnvMap, dotenvy::Error> {
    return EnvLoader::with_path(&location).load();
}

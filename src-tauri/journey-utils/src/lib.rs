use dotenvy::{EnvLoader, EnvMap};
pub fn get_env() -> Result<EnvMap, dotenvy::Error> {
    return EnvLoader::with_path("../../.env.production").load();
}

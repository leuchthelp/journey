use std::sync::LazyLock;

use tauri_plugin_os::Version;

pub const PRODUCT_VERSION: &'static str = "0.1.0.alpha";
pub const PRODUCT_NAME: &'static str = "journey";
pub const OS_HOSTNAME: LazyLock<String> = LazyLock::new(|| tauri_plugin_os::hostname());
pub const OS_PLATFORM: &'static str = std::env::consts::OS;
pub const OS_ARCH: &'static str = std::env::consts::ARCH;
pub const OS_VERSION: LazyLock<Version> = LazyLock::new(|| tauri_plugin_os::version());

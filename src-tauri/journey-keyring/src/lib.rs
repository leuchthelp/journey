use keyring_core::{Error, Result, set_default_store, unset_default_store};
use std::collections::HashMap;
use std::format;

pub use keyring_core;
pub use keyring_core::Entry;

const NAMED_STORES: [&str; 5] = ["android", "keyutils", "protected", "windows", "sample"];

/// Set the default store to the platform's OS-provided credential store.
///
/// If the platform has no OS-provided credential store, the sample store is used.
///
/// On Linux (only), the kernel keyutils store is used unless
/// `prefer_secret_service` is true, in which case the Secret Service
/// store is used.
#[allow(unused_variables)]
pub fn use_native_store() -> Result<()> {
    if cfg!(debug_assertions) {
        use_named_store("sample")?;

        Ok(())
    } else {
        #[cfg(target_os = "android")]
        use_named_store("android")?;
        #[cfg(any(target_os = "ios", target_os = "macos"))]
        use_named_store("protected")?;
        #[cfg(target_os = "windows")]
        use_named_store("windows")?;
        #[cfg(target_os = "linux")]
        use_named_store("keyutils")?;

        Ok(())
    }
}

/// Set the default store to one of the known stores in its default configuration.
///
/// Gives an `Invalid` error if the store name is not known.
///
/// Returns any error returned from store creation.
fn use_named_store(name: &str) -> Result<()> {
    if name.to_lowercase().as_str() == "sample" {
        use_sample_store(&HashMap::from([("persist", "true")]))
    } else {
        use_named_store_with_modifiers(name, &HashMap::new())
    }
}

fn use_named_store_with_modifiers(name: &str, modifiers: &HashMap<&str, &str>) -> Result<()> {
    match name.to_lowercase().as_str() {
        "android" => use_android_native_store(modifiers),
        "keyutils" => use_linux_keyutils_store(modifiers),
        "protected" => use_apple_protected_store(modifiers),
        "sample" => use_sample_store(modifiers),
        "windows" => use_windows_native_store(modifiers),
        _ => {
            let ok = NAMED_STORES.join(", ");
            let err = Error::Invalid(name.to_string(), format!("must be one of: {ok}"));
            Err(err)
        }
    }
}

/// Set the default store to the `keyring-core::Sample` store.
///
/// This is available on all platforms.
fn use_sample_store(config: &HashMap<&str, &str>) -> Result<()> {
    use keyring_core::sample::Store;
    set_default_store(Store::new_with_configuration(config)?);
    Ok(())
}

/// Use the macOS Keychain Services store.
///
/// Fails with a `NotSupportedByStore` error on other platforms.
#[allow(unused_variables)]
#[allow(unused)]
fn use_apple_keychain_store(config: &HashMap<&str, &str>) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        use apple_native_keyring_store::keychain::Store;
        set_default_store(Store::new_with_configuration(config)?);
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err(Error::NotSupportedByStore(
            "The macOS keychain is only available on macOS".to_string(),
        ))
    }
}

/// Use the iOS/macOS Protected Data store.
///
/// NOTE: macOS apps without a provisioning profile
/// cannot use the protected store. Because an app cannot
/// check itself for a provisioning profile, we use
/// whether the app is sandboxed as a proxy for this.
/// (by checking whether the `APP_SANDBOX_CONTAINER_ID`
/// is defined as an environment variable).
///
/// Note that it is possible for apps to be sandboxed
/// without a provisioning profile, in which case this
/// function will instantiate a store successfully, but
/// all attempts to read or write credentials will fail.
///
/// Fails with a `NotSupportedByStore` error on other platforms.
#[allow(unused_variables)]
fn use_apple_protected_store(config: &HashMap<&str, &str>) -> Result<()> {
    #[cfg(target_os = "macos")]
    if std::env::var("APP_SANDBOX_CONTAINER_ID").is_ok() {
        use apple_native_keyring_store::protected::Store;
        set_default_store(Store::new_with_configuration(config)?);
        Ok(())
    } else {
        use_apple_keychain_store(config);
    }
    #[cfg(target_os = "ios")]
    {
        use apple_native_keyring_store::protected::Store;
        set_default_store(Store::new_with_configuration(config)?);
        Ok(())
    }
    #[cfg(not(any(target_os = "macos", target_os = "ios")))]
    {
        Err(Error::NotSupportedByStore(
            "The macOS keychain is only available on macOS".to_string(),
        ))
    }
}

/// Use the Linux Keyutils store.
///
/// Fails with a `NotSupportedByStore` error on other platforms.
#[allow(unused_variables)]
fn use_linux_keyutils_store(config: &HashMap<&str, &str>) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        use linux_keyutils_keyring_store::Store;
        set_default_store(Store::new_with_configuration(config)?);
        Ok(())
    }
    #[cfg(not(target_os = "linux"))]
    {
        Err(Error::NotSupportedByStore(
            "The keyutils store is only available on Linux".to_string(),
        ))
    }
}

/// Use the Windows Credential store.
///
/// Fails with a `NotSupportedByStore` error on other platforms.
#[allow(unused_variables)]
fn use_windows_native_store(config: &HashMap<&str, &str>) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        use windows_native_keyring_store::Store;
        set_default_store(Store::new_with_configuration(config)?);
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err(Error::NotSupportedByStore(
            "The Windows credential store is only available on Windows".to_string(),
        ))
    }
}

/// Use the Android Shared Preferences store.
///
/// Shared Preference data is encrypted using the Android keystore.
///
/// Fails with a `NotSupportedByStore` error on other platforms.
#[allow(unused_variables)]
fn use_android_native_store(config: &HashMap<&str, &str>) -> Result<()> {
    #[cfg(target_os = "android")]
    {
        use android_native_keyring_store::Store;
        set_default_store(Store::new_with_configuration(config)?);
        Ok(())
    }
    #[cfg(not(target_os = "android"))]
    {
        Err(Error::NotSupportedByStore(
            "The Android native store is only available on Android".to_string(),
        ))
    }
}

/// Release the current default store.
pub fn release_store() {
    unset_default_store();
}

//! Platform-specific persistence for the wallet UI.

#[cfg(target_arch = "wasm32")]
pub use csv_sdk::consumer_storage::browser_storage::BrowserStorageError as StorageError;
#[cfg(target_arch = "wasm32")]
pub use csv_sdk::consumer_storage::browser_storage::*;

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use serde::{Serialize, de::DeserializeOwned};
    use std::path::PathBuf;

    /// Error returned by the desktop file-backed store.
    #[derive(Debug, thiserror::Error)]
    pub enum StorageError {
        #[error("storage key not found: {0}")]
        NotFound(String),
        #[error("storage I/O error: {0}")]
        Io(String),
        #[error("storage serialization error: {0}")]
        Serialization(String),
    }

    /// A namespaced JSON-file store used by the native desktop application.
    #[derive(Clone)]
    pub struct LocalStorageManager {
        root: PathBuf,
        prefix: String,
    }

    impl LocalStorageManager {
        pub fn new(prefix: &str) -> Result<Self, StorageError> {
            let root = dirs::data_local_dir()
                .or_else(dirs::home_dir)
                .unwrap_or_else(std::env::temp_dir)
                .join("hemion");
            std::fs::create_dir_all(&root).map_err(|error| StorageError::Io(error.to_string()))?;
            Ok(Self {
                root,
                prefix: prefix.to_string(),
            })
        }

        fn path_for(&self, key: &str) -> PathBuf {
            let safe_key: String = key
                .chars()
                .map(|character| {
                    if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                        character
                    } else {
                        '_'
                    }
                })
                .collect();
            self.root.join(format!("{}-{}.json", self.prefix, safe_key))
        }

        pub fn save<T: Serialize>(&self, key: &str, value: &T) -> Result<(), StorageError> {
            let json = serde_json::to_string(value)
                .map_err(|error| StorageError::Serialization(error.to_string()))?;
            self.set_raw(key, &json)
        }

        pub fn load<T: DeserializeOwned>(&self, key: &str) -> Result<T, StorageError> {
            let json = self
                .get_raw(key)?
                .ok_or_else(|| StorageError::NotFound(key.to_string()))?;
            serde_json::from_str(&json)
                .map_err(|error| StorageError::Serialization(error.to_string()))
        }

        pub fn try_load<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
            self.load(key).ok()
        }

        pub fn delete(&self, key: &str) -> Result<(), StorageError> {
            let path = self.path_for(key);
            match std::fs::remove_file(path) {
                Ok(()) => Ok(()),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(error) => Err(StorageError::Io(error.to_string())),
            }
        }

        pub fn get_raw(&self, key: &str) -> Result<Option<String>, StorageError> {
            match std::fs::read_to_string(self.path_for(key)) {
                Ok(value) => Ok(Some(value)),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
                Err(error) => Err(StorageError::Io(error.to_string())),
            }
        }

        pub fn set_raw(&self, key: &str, value: &str) -> Result<(), StorageError> {
            let destination = self.path_for(key);
            let temporary = destination.with_extension("tmp");
            std::fs::write(&temporary, value)
                .map_err(|error| StorageError::Io(error.to_string()))?;
            std::fs::rename(temporary, destination)
                .map_err(|error| StorageError::Io(error.to_string()))
        }

        pub fn contains(&self, key: &str) -> bool {
            self.path_for(key).exists()
        }
    }

    pub const UNIFIED_STORAGE_KEY: &str = "unified_storage";
    pub const WALLET_MNEMONIC_KEY: &str = "mnemonic_encrypted";

    pub fn wallet_storage() -> Result<LocalStorageManager, StorageError> {
        LocalStorageManager::new("wallet")
    }

    pub fn seal_storage() -> Result<LocalStorageManager, StorageError> {
        LocalStorageManager::new("seals")
    }

    pub fn asset_storage() -> Result<LocalStorageManager, StorageError> {
        LocalStorageManager::new("assets")
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use native::*;

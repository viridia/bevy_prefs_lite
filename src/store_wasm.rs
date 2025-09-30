pub use crate::{prefs::PreferencesStore, PreferencesFile, PreferencesFileContent};
use bevy::tasks::IoTaskPool;
use web_sys::window;

/// Resource which represents the place where preferences files are stored. This can be either
/// a filesystem directory (when working on a desktop platform) or a virtual directory such
/// as web LocalStorage.
///
/// You can access individual preferences files using the `.get()` or `.get_mut()` method. These
/// methods load the preferences into memory if they are not already loaded.
#[derive(Resource)]
pub struct StoreWasm {
    app_name: String,
}

impl StoreWasm {
    /// Construct a new `StoreWasm` resource.
    ///
    /// # Arguments
    /// * `app_name` - The name of the application. This is used to uniquely identify the
    ///   preferences directory so as not to confuse it with other applications' preferences.
    ///   To ensure global uniqueness, it is recommended to use a reverse domain name, e.g.
    ///   "com.example.myapp".
    ///
    ///   This is only used on desktop platforms. On web platforms, the name is ignored.
    pub fn new(app_name: &str) -> Self {
        Self {
            app_name: app_name.to_owned(),
        }
    }

    /// Returns the storage key for a given filename. This consists of the app name combined
    /// with the filename.
    fn storage_key(&self, filename: &str) -> String {
        format!("{}-{}", self.app_name, filename)
    }
}

impl PreferencesStore for StoreWasm {
    /// Returns true if preferences path is valid.
    fn is_valid(&self) -> bool {
        window().unwrap().local_storage().is_ok()
    }

    /// Create a new, empty preferences file.
    fn create(&self) -> PreferencesFile {
        PreferencesFile::new()
    }

    /// Save all changed `PreferenceFile`s to disk
    ///
    /// # Arguments
    /// * `filename` - the name of the file to be saved
    /// * `contents` - the contents of the file
    fn save(&self, filename: &str, contents: &PreferencesFile) {
        if let Ok(Some(storage)) = window().unwrap().local_storage() {
            info!("Saving preferences file: {}", filename);
            let json_str = contents.encode();
            storage
                .set_item(&self.storage_key(filename).as_str(), &json_str)
                .unwrap();
        }
    }

    /// Save all changed `PreferenceFile`s to disk, in another thread
    ///
    /// # Arguments
    /// * `filename` - the name of the file to be saved
    /// * `contents` - the contents of the file
    fn save_async(&self, filename: &str, contents: PreferencesFileContent) {
        IoTaskPool::get().scope(|scope| {
            scope.spawn(async {
                if let Ok(Some(storage)) = window().unwrap().local_storage() {
                    info!("Saving preferences file (async): {}", filename);
                    let json_str = contents.encode();
                    storage
                        .set_item(&self.storage_key(filename).as_str(), &json_str)
                        .unwrap();
                }
            });
        });
    }

    /// Deserialize a preferences file from disk. If the file does not exist, `None` will
    /// be returned.
    ///
    /// # Arguments
    /// * `filename` - The name of the preferences file, without the file extension.
    fn load(&mut self, filename: &str) -> Option<PreferencesFile> {
        if let Ok(Some(storage)) = window().unwrap().local_storage() {
            let storage_key = self.storage_key(filename);
            let Ok(Some(json_str)) = storage.get_item(&storage_key) else {
                return None;
            };

            Some(PreferencesFile::from_string(&json_str, filename))
        } else {
            None
        }
    }
}

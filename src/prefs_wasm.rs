use bevy::{platform_support::collections::HashMap, prelude::*};
use web_sys::window;

pub use crate::prefs_json::PreferencesFile;

/// Resource which represents the place where preferences files are stored. This can be either
/// a filesystem directory (when working on a desktop platform) or a virtual directory such
/// as web LocalStorage.
///
/// You can access individual preferences files using the `.get()` or `.get_mut()` method. These
/// methods load the preferences into memory if they are not already loaded.
#[derive(Resource)]
pub struct Preferences {
    app_name: String,
    files: HashMap<String, PreferencesFile>,
}

impl Preferences {
    /// Construct a new `Preferences` resource.
    ///
    /// # Arguments
    /// * `app_name` - The name of the application. This is used to uniquely identify the
    ///   preferences directory so as not to confuse it with other applications' preferences.
    ///   To ensure global uniqueness, it is recommended to use a reverse domain name, e.g.
    ///   "com.example.myapp".
    ///
    ///   This is only used on desktop platforms. On web platforms, the name is ignored.
    pub fn new(app_name: &str) -> Self {
        // console::log_1("Preferences::new");
        Self {
            app_name: app_name.to_owned(),
            files: HashMap::default(),
        }
    }

    /// Returns true if preferences path is valid.
    pub fn is_valid(&self) -> bool {
        window().unwrap().local_storage().is_ok()
    }

    /// Save all changed `PreferenceFile`s to disk
    ///
    /// # Arguments
    /// * `force` - If true, all preferences will be saved, even if they have not changed.
    pub fn save(&self, force: bool) {
        if let Ok(Some(storage)) = window().unwrap().local_storage() {
            for (filename, file) in self.files.iter() {
                if file.is_changed() || force {
                    info!("Saving preferences file: {}", filename);
                    file.clear_changed();

                    let json_str = file.encode();
                    storage
                        .set_item(&self.storage_key(filename).as_str(), &json_str)
                        .unwrap();
                }
            }
        }
    }

    /// Lazily load a `PreferencesFile`. If the file is already loaded, it will be returned
    /// immediately. If the file exists but is not loaded, it will be loaded and returned.
    /// If the file does not exist, or the base preference path cannot be determined, `None` will
    /// be returned.
    ///
    /// Once loaded, a `PreferencesFile` will remain in memory.
    ///
    /// # Arguments
    /// * `filename` - The name of the preferences file, without the file extension.
    ///
    /// # Returns
    /// * `Some(&mut PreferencesFile)` if the file was loaded or created successfully.
    /// * `None` if the local storage is not available, or there was no data for that key.
    /// * If the preferences entry exists but could not be decoded, a warning is printed and
    ///   an empty `PreferencesFile` is returned.
    pub fn get<'a>(&'a mut self, filename: &str) -> Option<&'a mut PreferencesFile> {
        if let Ok(Some(storage)) = window().unwrap().local_storage() {
            let storage_key = self.storage_key(filename);
            if self.files.contains_key(&storage_key) {
                return self.files.get_mut(&storage_key);
            }

            let Ok(Some(json_str)) = storage.get_item(&storage_key) else {
                return None;
            };

            self.files.insert(
                filename.to_owned(),
                PreferencesFile::from_string(&json_str, filename),
            );
            self.files.get_mut(filename)
        } else {
            None
        }
    }

    /// Lazily load a preferences file, or create it if it does not exist. If the file is already
    /// loaded, it will be returned immediately. If the file exists but is not loaded, it will be
    /// loaded and returned. If the file does not exist, a new `PreferencesFile` will be created
    /// and returned (but not saved). If the base preference path cannot be determined, `None` will
    /// be returned.
    ///
    /// Once loaded, a `PreferencesFile` will remain in memory.
    ///
    /// # Arguments
    /// * `filename` - The name of the preferences file, without the file extension.
    ///
    /// # Returns
    /// * `Some(&mut PreferencesFile)` if the file was loaded or created successfully.
    /// * `None` if the local storage is not available.
    /// * If the preferences does not exist, or could not be decoded,
    ///   an empty `PreferencesFile` is returned.
    pub fn get_mut<'a>(&'a mut self, filename: &str) -> Option<&'a mut PreferencesFile> {
        if let Ok(Some(storage)) = window().unwrap().local_storage() {
            let storage_key = self.storage_key(filename);
            Some(self.files.entry(filename.to_owned()).or_insert_with(|| {
                if let Ok(Some(json_str)) = storage.get_item(storage_key.as_str()) {
                    PreferencesFile::from_string(&json_str, filename)
                } else {
                    PreferencesFile::default()
                }
            }))
        } else {
            None
        }
    }

    /// Returns the storage key for a given filename. This consists of the app name combined
    /// with the filename.
    fn storage_key(&self, filename: &str) -> String {
        format!("{}-{}", self.app_name, filename)
    }
}

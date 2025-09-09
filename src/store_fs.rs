use std::path::PathBuf;

use bevy::prelude::*;

use directories::BaseDirs;

use crate::{
    prefs::PreferencesStore,
    prefs_toml::{load_toml_file, serialize_table},
    PreferencesFile,
};

/// PreferencesStore which uses the local filesystem. Preferences will be located in the
/// OS-specific directory for user preferences.
pub struct StoreFs {
    base_path: Option<PathBuf>,
}

impl StoreFs {
    /// Construct a new filesystem preferences store.
    ///
    /// # Arguments
    /// * `app_name` - The name of the application. This is used to uniquely identify the
    ///   preferences directory so as not to confuse it with other applications' preferences.
    ///   To ensure global uniqueness, it is recommended to use a reverse domain name, e.g.
    ///   "com.example.myapp".
    pub(crate) fn new(app_name: &str) -> Self {
        Self {
            base_path: if let Some(base_dirs) = BaseDirs::new() {
                let prefs_path = base_dirs.preference_dir().join(app_name);
                info!("Preferences path: {:?}", prefs_path);
                Some(prefs_path)
            } else {
                warn!("Could not find user configuration directories");
                None
            },
        }
    }
}

impl PreferencesStore for StoreFs {
    /// Returns true if preferences path is valid.
    fn is_valid(&self) -> bool {
        self.base_path.is_some()
    }

    fn create(&self) -> PreferencesFile {
        PreferencesFile::new()
    }

    /// Save all changed `PreferenceFile`s to disk
    ///
    /// # Arguments
    /// * `force` - If true, all preferences will be saved, even if they have not changed.
    fn save(&self, filename: &str, file: &PreferencesFile) {
        if let Some(base_path) = &self.base_path {
            // Recursively create the preferences directory if it doesn't exist.
            let mut dir_builder = std::fs::DirBuilder::new();
            dir_builder.recursive(true);
            if let Err(e) = dir_builder.create(base_path.clone()) {
                warn!("Could not create preferences directory: {:?}", e);
                return;
            }

            // Save preferences to temp file
            let temp_path = base_path.join(format!("{filename}.toml.new"));
            if let Err(e) = std::fs::write(&temp_path, serialize_table(&file.table)) {
                error!("Error saving preferences file: {}", e);
            }

            // Replace old prefs file with new one.
            let file_path = base_path.join(format!("{filename}.toml"));
            if let Err(e) = std::fs::rename(&temp_path, file_path) {
                warn!("Could not save preferences file: {:?}", e);
            }
        }
    }

    /// Deserialize a preferences file from disk. If the file does not exist, `None` will
    /// be returned.
    ///
    /// # Arguments
    /// * `filename` - The name of the preferences file, without the file extension.
    fn load(&mut self, filename: &str) -> Option<PreferencesFile> {
        let Some(base_path) = &self.base_path else {
            return None;
        };

        let file_path = base_path.join(format!("{filename}.toml"));
        load_toml_file(&file_path).map(PreferencesFile::from_table)
    }
}

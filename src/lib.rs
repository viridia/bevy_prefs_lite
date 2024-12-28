use std::path::PathBuf;

use bevy::{prelude::*, utils::HashMap};
use directories::BaseDirs;

mod autosave;

pub use autosave::{AutosavePrefsPlugin, StartAutosaveTimer};

#[cfg(not(target_arch = "wasm32"))]
mod prefs_toml;

#[cfg(not(target_arch = "wasm32"))]
use prefs_toml::{load_table, serialize_table};

#[cfg(not(target_arch = "wasm32"))]
pub use prefs_toml::{PreferencesFile, PreferencesGroup, PreferencesGroupMut};

/// Resource which represents the place where preferences files are stored. This can be either
/// a filesystem directory (when working on a desktop platform) or a virtual directory such
/// as web LocalStorage.
///
/// You can access individual preferences files using the `.get()` or `.get_mut()` method. These
/// methods load the preferences into memory if they are not already loaded.
#[derive(Resource)]
pub struct Preferences {
    base_path: Option<PathBuf>,
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
    ///
    pub fn new(app_name: &str) -> Self {
        Self {
            base_path: if let Some(base_dirs) = BaseDirs::new() {
                let prefs_path = base_dirs.preference_dir().join(app_name);
                info!("Preferences path: {:?}", prefs_path);
                Some(prefs_path)
            } else {
                warn!("Could not find user configuration directories");
                None
            },
            files: HashMap::default(),
        }
    }

    /// Returns true if preferences path is valid.
    pub fn is_valid(&self) -> bool {
        self.base_path.is_some()
    }

    /// Save all changed `PreferenceFile`s to disk
    ///
    /// # Arguments
    /// * `force` - If true, all preferences will be saved, even if they have not changed.
    pub fn save(&self, force: bool) {
        if let Some(base_path) = &self.base_path {
            // Recursively create the preferences directory if it doesn't exist.
            let mut dir_builder = std::fs::DirBuilder::new();
            dir_builder.recursive(true);
            if let Err(e) = dir_builder.create(base_path.clone()) {
                warn!("Could not create preferences directory: {:?}", e);
                return;
            }

            for (filename, file) in self.files.iter() {
                if file.is_changed() || force {
                    info!("Saving preferences file: {}", filename);
                    file.clear_changed();

                    // Save preferences to temp file
                    let temp_path = base_path.join(format!("{}.toml.new", filename));
                    if let Err(e) = std::fs::write(&temp_path, serialize_table(&file.table)) {
                        error!("Error saving preferences file: {}", e);
                    }

                    // Replace old prefs file with new one.
                    let file_path = base_path.join(format!("{}.toml", filename));
                    if let Err(e) = std::fs::rename(&temp_path, file_path) {
                        warn!("Could not save preferences file: {:?}", e);
                    }
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
    pub fn get<'a>(&'a mut self, filename: &str) -> Option<&'a mut PreferencesFile> {
        let Some(base_path) = &self.base_path else {
            return None;
        };

        if !self.files.contains_key(filename) {
            let file_path = base_path.join(format!("{}.toml", filename));
            let table = load_table(&file_path);
            if let Some(table) = table {
                self.files
                    .insert(filename.to_owned(), PreferencesFile::from_table(table));
            };
        }

        self.files.get_mut(filename)
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
    pub fn get_mut<'a>(&'a mut self, filename: &str) -> Option<&'a mut PreferencesFile> {
        let Some(base_path) = &self.base_path else {
            return None;
        };

        if !self.files.contains_key(filename) {
            let file_path = base_path.join(format!("{}.toml", filename));
            let table = load_table(&file_path);
            let prefs_file = if let Some(table) = table {
                PreferencesFile::from_table(table)
            } else {
                // Create new file
                PreferencesFile::new()
            };
            self.files.insert(filename.to_owned(), prefs_file);
        }

        self.files.get_mut(filename)
    }
}

/// Observer event triggered when it's time to save preferences
#[derive(Clone, Debug, Event)]
pub struct CollectPreferences;

/// A Command which saves preferences to disk.
#[derive(Default, PartialEq)]
pub enum SavePreferences {
    /// Save preferences only if they have changed (based on [`PreferencesChanged` resource]).
    #[default]
    IfChanged,
    /// Save preferences unconditionally.
    Always,
}

impl Command for SavePreferences {
    fn apply(self, world: &mut World) {
        let prefs = world.get_resource::<Preferences>().unwrap();
        prefs.save(self == SavePreferences::Always);
    }
}

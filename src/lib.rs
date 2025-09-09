use bevy::prelude::*;

mod autosave;

pub use autosave::{AutosavePrefsPlugin, StartAutosaveTimer};

mod prefs;

mod prefs_json;
mod prefs_toml;

#[cfg(not(target_arch = "wasm32"))]
mod store_fs;

#[cfg(target_arch = "wasm32")]
mod store_wasm;

#[cfg(not(target_arch = "wasm32"))]
pub use store_fs::StoreFs;

#[cfg(target_arch = "wasm32")]
pub use prefs_wasm::Preferences;

pub use crate::prefs::Preferences;

#[cfg(target_arch = "wasm32")]
mod format {
    use crate::prefs_json;

    pub type PreferencesFile = prefs_json::JsonPreferencesFile;
    pub type PreferencesGroup<'a> = prefs_json::JsonPreferencesGroup<'a>;
    pub type PreferencesGroupMut<'a> = prefs_json::JsonPreferencesGroupMut<'a>;
}

#[cfg(not(target_arch = "wasm32"))]
mod format {
    use crate::prefs_toml;

    pub type PreferencesFile = prefs_toml::TomlPreferencesFile;
    pub type PreferencesGroup<'a> = prefs_toml::TomlPreferencesGroup<'a>;
    pub type PreferencesGroupMut<'a> = prefs_toml::TomlPreferencesGroupMut<'a>;
}

pub use self::format::*;

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

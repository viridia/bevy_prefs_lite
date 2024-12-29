use bevy::prelude::*;

mod autosave;

pub use autosave::{AutosavePrefsPlugin, StartAutosaveTimer};

#[cfg(not(target_arch = "wasm32"))]
mod prefs_toml;

#[cfg(target_arch = "wasm32")]
mod prefs_json;

#[cfg(not(target_arch = "wasm32"))]
mod prefs_fs;

#[cfg(target_arch = "wasm32")]
mod prefs_wasm;

#[cfg(not(target_arch = "wasm32"))]
pub use prefs_toml::{PreferencesFile, PreferencesGroup, PreferencesGroupMut};

#[cfg(target_arch = "wasm32")]
pub use prefs_json::PreferencesFile;

#[cfg(not(target_arch = "wasm32"))]
pub use prefs_fs::Preferences;

#[cfg(target_arch = "wasm32")]
pub use prefs_wasm::Preferences;

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

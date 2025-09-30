use bevy::{
    app::{App, Plugin, Update},
    ecs::{
        resource::Resource,
        system::{Command, Commands, Res, ResMut},
        world::World,
    },
    time::Time,
};

use crate::SavePreferences;

/// Resource which contains a countdown timer for debouncing preferences changes.
/// If this is non-zero, preferences will be saved after the timer reaches zero.
#[derive(Resource, Default)]
struct AutosaveTimer(f32);

/// Plugin which automatically saves preferences when they change. This uses a delay timer
/// to prevent saving preferences too frequently. Preferences will be automatically saved 1 second
/// after they have been marked as changed.
pub struct AutosavePrefsPlugin;

impl Plugin for AutosavePrefsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AutosaveTimer>();
    }

    fn finish(&self, app: &mut App) {
        // Only load preferences if we were able to locate the user configuration directories.
        // let prefs = app.world().get_resource::<Preferences>().unwrap();
        // if prefs.is_valid() {
        //     info!("Loading Preferences from: {:?}", prefs.base_path);
        //     app.world_mut().trigger(LoadPreferences);
        // }
        app.add_systems(Update, auto_save_preferences);
    }
}

fn auto_save_preferences(mut timer: ResMut<AutosaveTimer>, time: Res<Time>, mut cmd: Commands) {
    if timer.0 > 0.0 {
        timer.0 = (timer.0 - time.delta_secs()).max(0.0);
        if timer.0 <= 0.0 {
            cmd.queue(SavePreferences::IfChanged);
        }
    }
}

/// A Command which marks preferences as changed, and starts the countdown timer for saving them.
#[derive(Default)]
pub struct StartAutosaveTimer;

impl Command for StartAutosaveTimer {
    fn apply(self, world: &mut World) {
        let mut timer = world.get_resource_mut::<AutosaveTimer>().unwrap();
        timer.0 = 1.0;
    }
}

use bevy::{
    prelude::*,
    window::{PrimaryWindow, WindowMode, WindowResized, WindowResolution},
};
use bevy_prefs_lite::{AutosavePrefsPlugin, Preferences, PreferencesFile, StartAutosaveTimer};

/// Example that remembers window position and size.
fn main() {
    info!("Hello, world!");
    // Configure preferences directory
    let mut preferences = Preferences::new("org.viridia.windowpos");

    // Initialize the window with the saved settings
    let mut window = Window {
        title: "Bevy Window Size Example".into(),
        ..default()
    };
    load_window_settings(&mut preferences, &mut window);

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(window),
            ..default()
        }))
        .add_plugins(AutosavePrefsPlugin)
        .insert_resource(preferences)
        .add_systems(Startup, setup)
        .add_systems(Update, update_window_settings)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera::default(), Camera2d));
}

/// System which keeps the window settings up to date when the user resizes or moves the window.
pub fn update_window_settings(
    mut move_events: MessageReader<WindowMoved>,
    mut resize_events: MessageReader<WindowResized>,
    windows: Query<&mut Window, With<PrimaryWindow>>,
    mut preferences: ResMut<Preferences>,
    mut commands: Commands,
) {
    let Ok(window) = windows.single() else {
        return;
    };

    let mut window_changed = false;
    for _ in move_events.read() {
        window_changed = true;
    }

    for _ in resize_events.read() {
        window_changed = true;
    }

    if window_changed {
        if let Some(app_prefs) = preferences.get_mut("prefs") {
            store_window_settings(app_prefs, window, &mut commands);
        }
    }
}

fn load_window_settings(prefs: &mut Preferences, window: &mut Window) {
    if let Some(app_prefs) = prefs.get("prefs") {
        if let Some(window_prefs) = app_prefs.get_group("window") {
            if let Some(fullscreen) = window_prefs.get::<bool>("fullscreen") {
                window.mode = if fullscreen {
                    WindowMode::BorderlessFullscreen(MonitorSelection::Current)
                } else {
                    WindowMode::Windowed
                };
            }
            if let Some(pos) = window_prefs.get::<IVec2>("position") {
                window.position = WindowPosition::new(pos);
            }
            if let Some(size) = window_prefs.get::<UVec2>("size") {
                window.resolution = WindowResolution::new(size.x, size.y);
            }
        }
    }
}

fn store_window_settings(
    app_prefs: &mut PreferencesFile,
    window: &Window,
    commands: &mut Commands,
) {
    let mut window_prefs = app_prefs.get_group_mut("window").unwrap();

    // Window fullscreen mode
    window_prefs.set_if_changed("fullscreen", window.mode != WindowMode::Windowed);

    // Window position
    match window.position {
        WindowPosition::At(pos) => {
            window_prefs.set_if_changed("position", pos);
        }
        _ => {
            window_prefs.remove("position");
        }
    };

    // Window size
    window_prefs.set_if_changed(
        "size",
        UVec2::new(
            window.resolution.width() as u32,
            window.resolution.height() as u32,
        ),
    );

    commands.queue(StartAutosaveTimer);
}

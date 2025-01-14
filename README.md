# bevy_prefs_lite

This crate provides basic preferences support for Bevy applications. The word "preferences"
in this context is used to mean user settings that are (1) set while running the app, (2) persistent
across restarts, and (3) implicitly saved. It is not meant to be a general config file
serialization mechanism.

Preferences typically include things like:

- Current editing "mode" or tool.
- Keyboard or game controller bindings.
- Music and sound effects volume settings.
- The location of the last saved game.
- The user's login name for a network game (but not password!)
- "Do not show this dialog again" checkbox settings.

Preferences are _NOT_:

- **Saved games**. The user can have many saved games, wherease typically there is only one set of
  preferences, which is user global. Also, while many games require the user to explicitly perform
  a "save" action, preferences generally are saved automatically.
- **Assets**. Preferences live in the operating-system-specific folder for user settings,
  whereas assets are something that is shipped with the game.
- **Meant to be human-editable**. While it is possible to edit preference files, these files are
  located in a system folder that is "hidden" from non-technical users such as `~/.config` or
  `$HOME/Library/Preferences/`. That being said, the format of preference files is TOML or JSON,
  which can easily be edited in a text editor.
- **Meant to be editable by other applications** - this crate only supports "basic" preferences,
  which means that it intentionally does not support some of the more advanced use cases. This
  includes cases where a third-party tool writes out a config file which is read by the game.

## Supported Features

- Supports both Desktop and Web (WASM) platforms.
- Preferences are serialized to TOML or JSON format.
- Preferences are saved in standard OS locations. Config directories are created if they do
  not already exist. The settings directory name is configurable.
- File-corruption-resistant: the framework will save the settings to a temp file, close the file,
  and then use a filesystem operation to move the temporary file to the settings config. This means
  that if the game crashes while saving, the settings file won't be corrupted.
- Debouncing/throttling - often a user setting, such as an audio volume slider or window
  splitter bar, changes at high frequency when dragged. The library allows you to mark preferences
  as "changed", which will save out preferences after a delay of one second.
- Various configurable options for saving preferences:
  - Mark changed: you can explicitly mark the preferences as "changed", which will trigger a
    deferred save.
  - Explicit synchronous flush: you can issue a `Command` which immediately and synchronously
    writes out the settings file.

## Platform support

When compiling for WASM targets, preferences are stored in browser `LocalStorage` in serialized
JSON format.

When compiling for desktop, preferences are stored as TOML files in the standard OS locations
for user preferences.

## Non-goals

Because this library supports "simple" preferences, some things have been intentionally left out:

- Serialization of exotic types - we don't support serialization of every possible Rust type.
  Generally, if a type is serializable via `serde` to JSON or TOML it will work.
- Choice of config file formats. The reason JSON is used in WASM is because it is more idiomatic
  in web apps; similarly, TOML is a common choice for desktop configuration files, and is similar
  to older formats such as ".ini" files.
- Hot loading / settings file change detection. Because the only program that ever writes to the
  settings file is the game itself, there's no need to be notified when the file has changed
  (and it would significantly complicate the design).
- Local overrides such as project or workspace-specific preferences. The vast majority of apps
  don't need this functionality, which would make the API considerably more complex.

## Usage

### Preferences Structure

The `Preferences` object represents the container for preferences files. Within this container
you can create individual `PreferenceFile` objects, each one backed by a separate file such as
"prefs.toml" (on the web, each file is stored as a separate key item in `LocalStorage`).

Each preferences file contains one or more `PreferenceGroups` which represents a section within
the file. Groups can also contain other groups.

Finally, groups have individual properties which are accessed via `get` and `set` methods.

In the examples below, the `app.toml` file would have a structure like this:

```toml
[window]
size = [
    800,
    600
]
```

### Initializing the preferences store and loading preferences

Normally the `Preferences` object is initialized during app initialization. You create a new
`Preferences` object, passing it a unique string which identifies your application. This string
is used to ensure that your preferences don't overwrite those of other apps.

The "reverse domain name" convention is an easy way to ensure global uniqueness:

```rust
// Configure preferences directory
let mut preferences = Preferences::new("com.mydomain.coolgame");
```

In desktop targets, the app name is used to establish a preferences directory in the standard
OS location for preferences.

The preferences store will verify that the preferences directory exists, but won't load anything
yet. To actually load preferences, you'll need to load a `PreferencesFile`, which corresponds
to individual preference files in your config directory such as `app.toml`:

```rust
let app_prefs = preferences.get("app").unwrap();
if let Some(window_group) = app_prefs.get_group("window") {
    if let Some(window_size) = window_group.get::<UVec2>("size") {
        // Configure window size
    }
}
```

So for example on Mac, the above code would look for a file in the location
"$HOME/Library/Preferences/com.mydomain.coolgame/app.toml".

In WASM, it would look for a local storage key named "com.mydomain.coolgame-app".

The `Preferences` object is also an ECS Resource, so you can insert it into the game world. This
makes it easy for other parts of the game code to load their preference settings. For example,
startup systems can inject preferences like any other resource.

```rust
app.insert_resource(preferences);
```

### Saving Preferances

To save preferences, you can use the `mut` versions of the preference methods:

```rust
let mut app_prefs = preferences.get_mut("app").unwrap();
let window_group = app_prefs.get_group_mut("window").unwrap();
window_group.set("size", UVec2::new(10, 10));
```

The `mut` methods do several things:

- They automatically create new preferences files and groups if they don't already exist.
- They store the new property value.
- They will compare with the previous value, and mark the preference file as changed
  if the new value is different.

However, setting the value only changes the preferences setting in memory, it does not automatically
save the changes to disk. To trigger a save, you can issue a `SavePreferences` command:

```rust
commands.queue(SavePreferences::IfChanged);
```

This will cause any preference files to be saved if they are marked as changed. It's up to you
to decide when to save preferences, but they should be saved before the app exits.

### Autosaving

The `AutosavePrefsPlugin` implements a timer which can be used to save preferences. Once you
install this plugin, you can then start the timer by issuing a command:

```rust
commands.queue(StartAutosaveTimer);
```

This command sets the save timer to 1 second, which counts down and then saves any changed
preference files when the timer goes off. This is useful for settings that change at high
frequency (like dragging an audio volume slider), reducing the number of writes to disk.

use std::{fs, path::PathBuf, sync::atomic::AtomicBool};

use bevy::prelude::*;
use serde::{de::DeserializeOwned, Serialize};

/// Load a preferences file from disk in TOML format.
pub(crate) fn load_toml_file(file: &PathBuf) -> Option<toml::Table> {
    if file.exists() && file.is_file() {
        let prefs_str = match fs::read_to_string(file) {
            Ok(prefs_str) => prefs_str,
            Err(e) => {
                error!("Error reading preferences file: {}", e);
                return None;
            }
        };

        let table_value = match toml::from_str::<toml::Value>(&prefs_str) {
            Ok(table_value) => table_value,
            Err(e) => {
                error!("Error parsing preferences file: {}", e);
                return None;
            }
        };

        match table_value {
            toml::Value::Table(table) => Some(table),
            _ => {
                error!("Preferences file must be a table");
                None
            }
        }
    } else {
        // Preferences file does not exist yet.
        None
    }
}

/// Save a preferences file to disk in TOML format.
pub(crate) fn serialize_table(table: &toml::Table) -> String {
    toml::to_string_pretty(&table).unwrap()
}

/// Represents a single preferences file containing multiple groups of settings.
#[derive(Debug, Default)]
pub struct TomlPreferencesFile {
    pub(crate) table: toml::Table,
    changed: AtomicBool,
}

impl TomlPreferencesFile {
    /// Create a new, empty preferences file.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a preferences file from a TOML table.
    pub(crate) fn from_table(table: toml::Table) -> Self {
        Self {
            table,
            changed: AtomicBool::new(false),
        }
    }

    /// Get a preferences group from the file, or `None` if the group does not exist.
    pub fn get_group(&self, group: &str) -> Option<TomlPreferencesGroup> {
        self.table
            .get(group)
            .and_then(|v| v.as_table())
            .map(|table| TomlPreferencesGroup { table })
    }

    /// Get a mutable reference to a preferences group from the file, creating it if it does not
    /// exist.
    pub fn get_group_mut<'a>(&'a mut self, group: &str) -> Option<TomlPreferencesGroupMut<'a>> {
        let entry = self
            .table
            .entry(group.to_owned())
            .or_insert_with(|| toml::Value::Table(toml::Table::new()));
        entry.as_table_mut().map(|table| TomlPreferencesGroupMut {
            table,
            changed: &mut self.changed,
        })
    }

    /// Mark the preferences group as changed.
    pub fn set_changed(&self) {
        self.changed
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// Clear the changed flag for the preferences group.
    pub fn clear_changed(&self) {
        self.changed
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }

    /// Check if the preferences group has been changed.
    pub fn is_changed(&self) -> bool {
        self.changed.load(std::sync::atomic::Ordering::Relaxed)
    }
}

pub struct TomlPreferencesGroup<'a> {
    table: &'a toml::Table,
}

pub struct TomlPreferencesGroupMut<'a> {
    table: &'a mut toml::Table,
    changed: &'a AtomicBool,
}

impl TomlPreferencesGroup<'_> {
    /// Get a key from the preferences group as a deserializable value, or `None` if the key does
    /// not exist or is not deserializable.
    pub fn get<D>(&self, key: &str) -> Option<D>
    where
        D: DeserializeOwned,
    {
        let value = self.table.get(key)?.clone();
        toml::Value::try_into(value).ok()
    }

    /// Read a nested preferences group from the group, or `None` if the property does not exist or
    /// is not a table.
    pub fn get_group(&self, key: &str) -> Option<TomlPreferencesGroup> {
        self.table
            .get(key)
            .and_then(|v| v.as_table())
            .map(|table| TomlPreferencesGroup { table })
    }
}

impl TomlPreferencesGroupMut<'_> {
    /// Delete a key from the preferences group.
    pub fn remove(&mut self, key: &str) {
        if self.table.remove(key).is_some() {
            self.changed
                .store(true, std::sync::atomic::Ordering::Relaxed);
        }
    }

    /// Get a key from the preferences group as a deserializable value, or `None` if the key does
    /// not exist or is not deserializable.
    pub fn get<D>(&self, key: &str) -> Option<D>
    where
        D: DeserializeOwned,
    {
        let value = self.table.get(key)?.clone();
        toml::Value::try_into(value).ok()
    }

    /// Set a key in the preferences group to a serializable value, and mark the file as changed.
    pub fn set<S: Serialize>(&mut self, key: &str, value: S) {
        let value = toml::Value::try_from(value).unwrap();
        self.table.insert(key.to_owned(), value);
        self.changed
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// Convert `value` into a TOML value. If it is different than the current value, set the key
    /// in the preferences group to the new value, and mark the file as changed.
    pub fn set_if_changed<S: Serialize>(&mut self, key: &str, value: S) {
        let value = toml::Value::try_from(value).unwrap();
        match self.table.get(key) {
            Some(v) if v == &value => (),
            _ => {
                self.table.insert(key.to_owned(), value);
                self.changed
                    .store(true, std::sync::atomic::Ordering::Relaxed);
            }
        }
    }

    /// Read a nested preferences group from the group, or `None` if the property does not exist or
    /// is not a table.
    pub fn get_group(&self, key: &str) -> Option<TomlPreferencesGroup> {
        self.table
            .get(key)
            .and_then(|v| v.as_table())
            .map(|table| TomlPreferencesGroup { table })
    }

    /// Get a mutable reference to a nested preferences group from the group, creating it if it
    /// does not exist.
    pub fn get_group_mut<'a>(&'a mut self, key: &str) -> Option<TomlPreferencesGroupMut<'a>> {
        let entry = self.table.entry(key.to_owned()).or_insert_with(|| {
            self.changed
                .store(true, std::sync::atomic::Ordering::Relaxed);
            toml::Value::Table(toml::Table::new())
        });
        entry.as_table_mut().map(|table| TomlPreferencesGroupMut {
            table,
            changed: self.changed,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_table() {
        let mut table = toml::Table::new();
        table.insert("key".to_string(), toml::Value::String("value".to_string()));

        let serialized = serialize_table(&table);
        assert_eq!(serialized, "key = \"value\"\n");
    }

    #[test]
    fn test_preferences_file_get_group() {
        let mut table = toml::Table::new();
        let mut group = toml::Table::new();
        group.insert("key".to_string(), toml::Value::String("value".to_string()));
        table.insert("group".to_string(), toml::Value::Table(group));

        let prefs = TomlPreferencesFile::from_table(table);
        let group = prefs.get_group("group").unwrap();
        assert_eq!(group.get::<String>("key").unwrap(), "value");
    }

    #[test]
    fn test_preferences_file_get_group_mut() {
        let table = toml::Table::new();
        let mut prefs = TomlPreferencesFile::from_table(table);
        {
            let mut group = prefs.get_group_mut("group").unwrap();
            group.set("key", "value");
        }
        let group = prefs.get_group("group").unwrap();
        assert_eq!(group.get::<String>("key").unwrap(), "value");
    }

    #[test]
    fn test_preferences_group_get_bool() {
        let mut table = toml::Table::new();
        table.insert("key".to_string(), toml::Value::Boolean(true));
        let group = TomlPreferencesGroup { table: &table };
        assert!(group.get::<bool>("key").unwrap());
    }

    #[test]
    fn test_preferences_group_get_string() {
        let mut table = toml::Table::new();
        table.insert("key".to_string(), toml::Value::String("value".to_string()));
        let group = TomlPreferencesGroup { table: &table };
        assert_eq!(group.get::<String>("key").unwrap(), "value");
    }

    #[test]
    fn test_preferences_group_get_integer() {
        let mut table = toml::Table::new();
        table.insert("key".to_string(), toml::Value::Integer(42));
        let group = TomlPreferencesGroup { table: &table };
        assert_eq!(group.get::<i32>("key").unwrap(), 42);
    }

    #[test]
    fn test_preferences_group_get_float() {
        let mut table = toml::Table::new();
        table.insert("key".to_string(), toml::Value::Float(3.1));
        let group = TomlPreferencesGroup { table: &table };
        assert_eq!(group.get::<f32>("key").unwrap(), 3.1);
    }

    #[test]
    fn test_preferences_group_get_ivec2() {
        let mut table = toml::Table::new();
        table.insert(
            "key".to_string(),
            toml::Value::Array(vec![toml::Value::Integer(1), toml::Value::Integer(2)]),
        );
        let group = TomlPreferencesGroup { table: &table };
        assert_eq!(group.get::<IVec2>("key").unwrap(), IVec2::new(1, 2));
    }

    #[test]
    fn test_preferences_group_get_uvec2() {
        let mut table = toml::Table::new();
        table.insert(
            "key".to_string(),
            toml::Value::Array(vec![toml::Value::Integer(1), toml::Value::Integer(2)]),
        );
        let group = TomlPreferencesGroup { table: &table };
        assert_eq!(group.get::<UVec2>("key").unwrap(), UVec2::new(1, 2));
    }

    #[test]
    fn test_preferences_group_get_vec2() {
        let mut table = toml::Table::new();
        table.insert(
            "key".to_string(),
            toml::Value::Array(vec![toml::Value::Float(1.0), toml::Value::Float(2.0)]),
        );
        let group = TomlPreferencesGroup { table: &table };
        assert_eq!(group.get::<Vec2>("key").unwrap(), Vec2::new(1.0, 2.0));
    }

    #[test]
    fn test_preferences_group_get_ivec3() {
        let mut table = toml::Table::new();
        table.insert(
            "key".to_string(),
            toml::Value::Array(vec![
                toml::Value::Integer(1),
                toml::Value::Integer(2),
                toml::Value::Integer(3),
            ]),
        );
        let group = TomlPreferencesGroup { table: &table };
        assert_eq!(group.get::<IVec3>("key").unwrap(), IVec3::new(1, 2, 3));
    }

    #[test]
    fn test_preferences_group_get_uvec3() {
        let mut table = toml::Table::new();
        table.insert(
            "key".to_string(),
            toml::Value::Array(vec![
                toml::Value::Integer(1),
                toml::Value::Integer(2),
                toml::Value::Integer(3),
            ]),
        );
        let group = TomlPreferencesGroup { table: &table };
        assert_eq!(group.get::<UVec3>("key").unwrap(), UVec3::new(1, 2, 3));
    }

    #[test]
    fn test_preferences_group_get_vec3() {
        let mut table = toml::Table::new();
        table.insert(
            "key".to_string(),
            toml::Value::Array(vec![
                toml::Value::Float(1.0),
                toml::Value::Float(2.0),
                toml::Value::Float(3.0),
            ]),
        );
        let group = TomlPreferencesGroup { table: &table };
        assert_eq!(group.get::<Vec3>("key").unwrap(), Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_preferences_group_mut_set_bool() {
        let mut table = toml::Table::new();
        let changed = AtomicBool::new(false);
        let mut group = TomlPreferencesGroupMut {
            table: &mut table,
            changed: &changed,
        };
        group.set("key", true);
        assert!(group.get::<bool>("key").unwrap());
        assert!(changed.load(std::sync::atomic::Ordering::Relaxed));

        changed.store(false, std::sync::atomic::Ordering::Relaxed);
        group.set_if_changed("key", true);
        assert!(group.get::<bool>("key").unwrap());
        assert!(!changed.load(std::sync::atomic::Ordering::Relaxed));
    }

    #[test]
    fn test_preferences_group_mut_set_string() {
        let mut table = toml::Table::new();
        let changed = AtomicBool::new(false);
        let mut group = TomlPreferencesGroupMut {
            table: &mut table,
            changed: &changed,
        };
        group.set("key", "value");
        assert_eq!(group.get::<String>("key").unwrap(), "value");
        assert!(changed.load(std::sync::atomic::Ordering::Relaxed));
    }

    #[test]
    fn test_preferences_group_mut_set_integer() {
        let mut table = toml::Table::new();
        let changed = AtomicBool::new(false);
        let mut group = TomlPreferencesGroupMut {
            table: &mut table,
            changed: &changed,
        };
        group.set("key", 42);
        assert_eq!(group.get::<i32>("key").unwrap(), 42);
        assert!(changed.load(std::sync::atomic::Ordering::Relaxed));
    }

    #[test]
    fn test_preferences_group_mut_set_float() {
        let mut table = toml::Table::new();
        let changed = AtomicBool::new(false);
        let mut group = TomlPreferencesGroupMut {
            table: &mut table,
            changed: &changed,
        };
        group.set("key", 3.1);
        assert_eq!(group.get::<f64>("key").unwrap(), 3.1);
        assert!(changed.load(std::sync::atomic::Ordering::Relaxed));
    }

    #[test]
    fn test_preferences_group_mut_set_ivec2() {
        let mut table = toml::Table::new();
        let changed = AtomicBool::new(false);
        let mut group = TomlPreferencesGroupMut {
            table: &mut table,
            changed: &changed,
        };
        group.set("key", IVec2::new(1, 2));
        assert_eq!(group.get::<IVec2>("key").unwrap(), IVec2::new(1, 2));
        assert!(changed.load(std::sync::atomic::Ordering::Relaxed));
    }

    #[test]
    fn test_preferences_group_mut_set_uvec2() {
        let mut table = toml::Table::new();
        let changed = AtomicBool::new(false);
        let mut group = TomlPreferencesGroupMut {
            table: &mut table,
            changed: &changed,
        };
        group.set::<UVec2>("key", UVec2::new(1, 2));
        assert_eq!(group.get::<UVec2>("key").unwrap(), UVec2::new(1, 2));
        assert!(changed.load(std::sync::atomic::Ordering::Relaxed));
    }

    #[test]
    fn test_preferences_group_mut_set_vec2() {
        let mut table = toml::Table::new();
        let changed = AtomicBool::new(false);
        let mut group = TomlPreferencesGroupMut {
            table: &mut table,
            changed: &changed,
        };
        group.set("key", Vec2::new(1.0, 2.0));
        assert_eq!(group.get::<Vec2>("key").unwrap(), Vec2::new(1.0, 2.0));
        assert!(changed.load(std::sync::atomic::Ordering::Relaxed));
    }

    #[test]
    fn test_preferences_group_mut_set_ivec3() {
        let mut table = toml::Table::new();
        let changed = AtomicBool::new(false);
        let mut group = TomlPreferencesGroupMut {
            table: &mut table,
            changed: &changed,
        };
        group.set("key", IVec3::new(1, 2, 3));
        assert_eq!(group.get::<IVec3>("key").unwrap(), IVec3::new(1, 2, 3));
        assert!(changed.load(std::sync::atomic::Ordering::Relaxed));
    }

    #[test]
    fn test_preferences_group_mut_set_uvec3() {
        let mut table = toml::Table::new();
        let changed = AtomicBool::new(false);
        let mut group = TomlPreferencesGroupMut {
            table: &mut table,
            changed: &changed,
        };
        group.set("key", UVec3::new(1, 2, 3));
        assert_eq!(group.get::<UVec3>("key").unwrap(), UVec3::new(1, 2, 3));
        assert!(changed.load(std::sync::atomic::Ordering::Relaxed));
    }

    #[test]
    fn test_preferences_group_mut_set_vec3() {
        let mut table = toml::Table::new();
        let changed = AtomicBool::new(false);
        let mut group = TomlPreferencesGroupMut {
            table: &mut table,
            changed: &changed,
        };
        group.set("key", Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(group.get::<Vec3>("key").unwrap(), Vec3::new(1.0, 2.0, 3.0));
        assert!(changed.load(std::sync::atomic::Ordering::Relaxed));

        changed.store(false, std::sync::atomic::Ordering::Relaxed);
        group.set_if_changed("key", Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(group.get::<Vec3>("key").unwrap(), Vec3::new(1.0, 2.0, 3.0));
        assert!(!changed.load(std::sync::atomic::Ordering::Relaxed));

        group.set_if_changed("key", Vec3::new(3.0, 2.0, 1.0));
        assert_eq!(group.get::<Vec3>("key").unwrap(), Vec3::new(3.0, 2.0, 1.0));
        assert!(changed.load(std::sync::atomic::Ordering::Relaxed));
    }
}

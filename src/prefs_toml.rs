use std::{fs, path::PathBuf, sync::atomic::AtomicBool};

use bevy::prelude::*;

/// Load a preferences file from disk in TOML format.
pub(crate) fn load_table(file: &PathBuf) -> Option<toml::Table> {
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
pub struct PreferencesFile {
    pub(crate) table: toml::Table,
    changed: AtomicBool,
}

impl PreferencesFile {
    /// Create a new, empty preferences file.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a preferences file from a TOML table.
    pub fn from_table(table: toml::Table) -> Self {
        Self {
            table,
            changed: AtomicBool::new(false),
        }
    }

    /// Get a preferences group from the file, or `None` if the group does not exist.
    pub fn get_group(&self, group: &str) -> Option<PreferencesGroup> {
        self.table
            .get(group)
            .and_then(|v| v.as_table())
            .map(|table| PreferencesGroup { table })
    }

    /// Get a mutable reference to a preferences group from the file, creating it if it does not
    /// exist.
    pub fn get_group_mut<'a>(&'a mut self, group: &str) -> Option<PreferencesGroupMut<'a>> {
        let entry = self
            .table
            .entry(group.to_owned())
            .or_insert_with(|| toml::Value::Table(toml::Table::new()));
        entry.as_table_mut().map(|table| PreferencesGroupMut {
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

pub struct PreferencesGroup<'a> {
    table: &'a toml::Table,
}

pub struct PreferencesGroupMut<'a> {
    table: &'a mut toml::Table,
    changed: &'a AtomicBool,
}

impl PreferencesGroup<'_> {
    /// Read a boolean property from the group, or `None` if the property does not exist or is not
    /// a boolean.
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.table.get(key).and_then(|v| v.as_bool())
    }

    /// Read a string property from the group, or `None` if the property does not exist or is not
    /// a string.
    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.table.get(key).and_then(|v| v.as_str())
    }

    /// Read an integer property from the group, or `None` if the property does not exist or is not
    /// an integer.
    pub fn get_integer(&self, key: &str) -> Option<i64> {
        self.table.get(key).and_then(|v| v.as_integer())
    }

    /// Read a float property from the group, or `None` if the property does not exist or is not
    /// a float.
    pub fn get_float(&self, key: &str) -> Option<f64> {
        self.table.get(key).and_then(|v| v.as_float())
    }

    /// Read an `IVec2` property from the group, or `None` if the property does not exist or
    /// is not a valid `IVec2`.
    pub fn get_ivec2(&self, key: &str) -> Option<IVec2> {
        self.table.get(key).and_then(value_to_ivec2)
    }

    /// Read a `UVec2` property from the group, or `None` if the property does not exist or
    /// is not a valid `UVec2`.
    pub fn get_uvec2(&self, key: &str) -> Option<UVec2> {
        self.table.get(key).and_then(value_to_uvec2)
    }

    /// Read a `Vec2` property from the group, or `None` if the property does not exist or
    /// is not a valid `Vec2`.
    pub fn get_vec2(&self, key: &str) -> Option<Vec2> {
        self.table.get(key).and_then(value_to_vec2)
    }

    /// Read an `IVec3` property from the group, or `None` if the property does not exist or
    /// is not a valid `IVec3`.
    pub fn get_ivec3(&self, key: &str) -> Option<IVec3> {
        self.table.get(key).and_then(value_to_ivec3)
    }

    /// Read a `UVec3` property from the group, or `None` if the property does not exist or
    /// is not a valid `UVec3`.
    pub fn get_uvec3(&self, key: &str) -> Option<UVec3> {
        self.table.get(key).and_then(value_to_uvec3)
    }

    /// Read a `Vec3` property from the group, or `None` if the property does not exist or
    /// is not a valid `Vec3`.
    pub fn get_vec3(&self, key: &str) -> Option<Vec3> {
        self.table.get(key).and_then(value_to_vec3)
    }

    /// Read a nested preferences group from the group, or `None` if the property does not exist or
    /// is not a table.
    pub fn get_group(&self, key: &str) -> Option<PreferencesGroup> {
        self.table
            .get(key)
            .and_then(|v| v.as_table())
            .map(|table| PreferencesGroup { table })
    }
}

impl PreferencesGroupMut<'_> {
    /// Delete a key from the preferences group.
    pub fn remove(&mut self, key: &str) {
        if self.table.remove(key).is_some() {
            self.changed
                .store(true, std::sync::atomic::Ordering::Relaxed);
        }
    }

    /// Read a boolean property from the group, or `None` if the property does not exist or is not
    /// a boolean.
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.table.get(key).and_then(|v| v.as_bool())
    }

    /// Set a boolean property in the group.
    pub fn set_bool(&mut self, key: &str, value: bool) -> &mut Self {
        match self.table.get(key) {
            Some(v) if v.as_bool() == Some(value) => return self,
            _ => {
                self.table
                    .insert(key.to_owned(), toml::Value::Boolean(value));
                self.changed
                    .store(true, std::sync::atomic::Ordering::Relaxed);
            }
        }
        self
    }

    /// Read a string property from the group, or `None` if the property does not exist or is not
    /// a string.
    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.table.get(key).and_then(|v| v.as_str())
    }

    /// Set a string property in the group.
    pub fn set_string(&mut self, key: &str, value: &str) -> &mut Self {
        match self.table.get(key) {
            Some(v) if v.as_str() == Some(value) => return self,
            _ => {
                self.table
                    .insert(key.to_owned(), toml::Value::String(value.to_owned()));
                self.changed
                    .store(true, std::sync::atomic::Ordering::Relaxed);
            }
        }
        self
    }

    /// Read an integer property from the group, or `None` if the property does not exist or is not
    /// an integer.
    pub fn get_integer(&self, key: &str) -> Option<i64> {
        self.table.get(key).and_then(|v| v.as_integer())
    }

    /// Set an integer property in the group.
    pub fn set_integer(&mut self, key: &str, value: i64) -> &mut Self {
        match self.table.get(key) {
            Some(v) if v.as_integer() == Some(value) => return self,
            _ => {
                self.table
                    .insert(key.to_owned(), toml::Value::Integer(value));
                self.changed
                    .store(true, std::sync::atomic::Ordering::Relaxed);
            }
        }
        self
    }

    /// Read a float property from the group, or `None` if the property does not exist or is not
    /// a float.
    pub fn get_float(&self, key: &str) -> Option<f64> {
        self.table.get(key).and_then(|v| v.as_float())
    }

    /// Set a float property in the group.
    pub fn set_float(&mut self, key: &str, value: f64) -> &mut Self {
        match self.table.get(key) {
            Some(v) if v.as_float() == Some(value) => return self,
            _ => {
                self.table.insert(key.to_owned(), toml::Value::Float(value));
                self.changed
                    .store(true, std::sync::atomic::Ordering::Relaxed);
            }
        }
        self
    }

    /// Read an `IVec2` property from the group, or `None` if the property does not exist or
    /// is not a valid `IVec2`.
    pub fn get_ivec2(&self, key: &str) -> Option<IVec2> {
        self.table.get(key).and_then(value_to_ivec2)
    }

    /// Set an `IVec2` property in the group.
    pub fn set_ivec2(&mut self, key: &str, value: IVec2) -> &mut Self {
        match self.table.get(key) {
            Some(v) if value_to_ivec2(v) == Some(value) => return self,
            _ => {
                self.table.insert(
                    key.to_owned(),
                    toml::Value::Array(vec![
                        toml::Value::Integer(value.x as i64),
                        toml::Value::Integer(value.y as i64),
                    ]),
                );
                self.changed
                    .store(true, std::sync::atomic::Ordering::Relaxed);
            }
        }
        self
    }

    /// Read a `UVec2` property from the group, or `None` if the property does not exist or
    /// is not a valid `UVec2`.
    pub fn get_uvec2(&self, key: &str) -> Option<UVec2> {
        self.table.get(key).and_then(value_to_uvec2)
    }

    /// Set a `UVec2` property in the group.
    pub fn set_uvec2(&mut self, key: &str, value: UVec2) -> &mut Self {
        match self.table.get(key) {
            Some(v) if value_to_uvec2(v) == Some(value) => return self,
            _ => {
                self.table.insert(
                    key.to_owned(),
                    toml::Value::Array(vec![
                        toml::Value::Integer(value.x as i64),
                        toml::Value::Integer(value.y as i64),
                    ]),
                );
                self.changed
                    .store(true, std::sync::atomic::Ordering::Relaxed);
            }
        }
        self
    }

    /// Read a `Vec2` property from the group, or `None` if the property does not exist or
    /// is not a valid `Vec2`.
    pub fn get_vec2(&self, key: &str) -> Option<Vec2> {
        self.table.get(key).and_then(value_to_vec2)
    }

    /// Set a `Vec2` property in the group.
    pub fn set_vec2(&mut self, key: &str, value: Vec2) -> &mut Self {
        match self.table.get(key) {
            Some(v) if value_to_vec2(v) == Some(value) => return self,
            _ => {
                self.table.insert(
                    key.to_owned(),
                    toml::Value::Array(vec![
                        toml::Value::Float(value.x as f64),
                        toml::Value::Float(value.y as f64),
                    ]),
                );
                self.changed
                    .store(true, std::sync::atomic::Ordering::Relaxed);
            }
        }
        self
    }

    /// Read an `IVec3` property from the group, or `None` if the property does not exist or
    /// is not a valid `IVec3`.
    pub fn get_ivec3(&self, key: &str) -> Option<IVec3> {
        self.table.get(key).and_then(value_to_ivec3)
    }

    /// Set an `IVec3` property in the group.
    pub fn set_ivec3(&mut self, key: &str, value: IVec3) -> &mut Self {
        match self.table.get(key) {
            Some(v) if value_to_ivec3(v) == Some(value) => return self,
            _ => {
                self.table.insert(
                    key.to_owned(),
                    toml::Value::Array(vec![
                        toml::Value::Integer(value.x as i64),
                        toml::Value::Integer(value.y as i64),
                        toml::Value::Integer(value.z as i64),
                    ]),
                );
                self.changed
                    .store(true, std::sync::atomic::Ordering::Relaxed);
            }
        }
        self
    }

    /// Read a `UVec3` property from the group, or `None` if the property does not exist or
    /// is not a valid `UVec3`.
    pub fn get_uvec3(&self, key: &str) -> Option<UVec3> {
        self.table.get(key).and_then(value_to_uvec3)
    }

    /// Set a `UVec3` property in the group.
    pub fn set_uvec3(&mut self, key: &str, value: UVec3) -> &mut Self {
        match self.table.get(key) {
            Some(v) if value_to_uvec3(v) == Some(value) => return self,
            _ => {
                self.table.insert(
                    key.to_owned(),
                    toml::Value::Array(vec![
                        toml::Value::Integer(value.x as i64),
                        toml::Value::Integer(value.y as i64),
                        toml::Value::Integer(value.z as i64),
                    ]),
                );
                self.changed
                    .store(true, std::sync::atomic::Ordering::Relaxed);
            }
        }
        self
    }

    /// Read a `Vec3` property from the group, or `None` if the property does not exist or
    /// is not a valid `Vec3`.
    pub fn get_vec3(&self, key: &str) -> Option<Vec3> {
        self.table.get(key).and_then(value_to_vec3)
    }

    /// Set a `Vec3` property in the group.
    pub fn set_vec3(&mut self, key: &str, value: Vec3) -> &mut Self {
        match self.table.get(key) {
            Some(v) if value_to_vec3(v) == Some(value) => return self,
            _ => {
                self.table.insert(
                    key.to_owned(),
                    toml::Value::Array(vec![
                        toml::Value::Float(value.x as f64),
                        toml::Value::Float(value.y as f64),
                        toml::Value::Float(value.z as f64),
                    ]),
                );
                self.changed
                    .store(true, std::sync::atomic::Ordering::Relaxed);
            }
        }
        self
    }

    /// Read a nested preferences group from the group, or `None` if the property does not exist or
    /// is not a table.
    pub fn get_group(&self, key: &str) -> Option<PreferencesGroup> {
        self.table
            .get(key)
            .and_then(|v| v.as_table())
            .map(|table| PreferencesGroup { table })
    }

    /// Get a mutable reference to a nested preferences group from the group, creating it if it
    /// does not exist.
    pub fn get_group_mut<'a>(&'a mut self, key: &str) -> Option<PreferencesGroupMut<'a>> {
        let entry = self
            .table
            .entry(key.to_owned())
            .or_insert_with(|| toml::Value::Table(toml::Table::new()));
        entry.as_table_mut().map(|table| PreferencesGroupMut {
            table,
            changed: self.changed,
        })
    }
}

fn value_to_ivec2(value: &toml::Value) -> Option<IVec2> {
    if let toml::Value::Array(a) = value {
        if a.len() == 2 {
            if let (Some(a0), Some(a1)) = (a[0].as_integer(), a[1].as_integer()) {
                return Some(IVec2::new(a0 as i32, a1 as i32));
            }
        }
    }
    None
}

fn value_to_uvec2(value: &toml::Value) -> Option<UVec2> {
    if let toml::Value::Array(a) = value {
        if a.len() == 2 {
            if let (Some(a0), Some(a1)) = (a[0].as_integer(), a[1].as_integer()) {
                if a0 >= 0 && a1 >= 0 {
                    return Some(UVec2::new(a0 as u32, a1 as u32));
                }
            }
        }
    }
    None
}

fn value_to_vec2(value: &toml::Value) -> Option<Vec2> {
    if let toml::Value::Array(a) = value {
        if a.len() == 2 {
            if let (Some(a0), Some(a1)) = (a[0].as_float(), a[1].as_float()) {
                return Some(Vec2::new(a0 as f32, a1 as f32));
            }
        }
    }
    None
}

fn value_to_ivec3(value: &toml::Value) -> Option<IVec3> {
    if let toml::Value::Array(a) = value {
        if a.len() == 3 {
            if let (Some(a0), Some(a1), Some(a2)) =
                (a[0].as_integer(), a[1].as_integer(), a[2].as_integer())
            {
                return Some(IVec3::new(a0 as i32, a1 as i32, a2 as i32));
            }
        }
    }
    None
}

fn value_to_uvec3(value: &toml::Value) -> Option<UVec3> {
    if let toml::Value::Array(a) = value {
        if a.len() == 3 {
            if let (Some(a0), Some(a1), Some(a2)) =
                (a[0].as_integer(), a[1].as_integer(), a[2].as_integer())
            {
                if a0 >= 0 && a1 >= 0 {
                    return Some(UVec3::new(a0 as u32, a1 as u32, a2 as u32));
                }
            }
        }
    }
    None
}

fn value_to_vec3(value: &toml::Value) -> Option<Vec3> {
    if let toml::Value::Array(a) = value {
        if a.len() == 3 {
            if let (Some(a0), Some(a1), Some(a2)) =
                (a[0].as_float(), a[1].as_float(), a[2].as_float())
            {
                return Some(Vec3::new(a0 as f32, a1 as f32, a2 as f32));
            }
        }
    }
    None
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

        let prefs = PreferencesFile::from_table(table);
        let group = prefs.get_group("group").unwrap();
        assert_eq!(group.get_string("key").unwrap(), "value");
    }

    #[test]
    fn test_preferences_file_get_group_mut() {
        let table = toml::Table::new();
        let mut prefs = PreferencesFile::from_table(table);
        {
            let mut group = prefs.get_group_mut("group").unwrap();
            group.set_string("key", "value");
        }
        let group = prefs.get_group("group").unwrap();
        assert_eq!(group.get_string("key").unwrap(), "value");
    }

    #[test]
    fn test_preferences_group_get_bool() {
        let mut table = toml::Table::new();
        table.insert("key".to_string(), toml::Value::Boolean(true));
        let group = PreferencesGroup { table: &table };
        assert!(group.get_bool("key").unwrap());
    }

    #[test]
    fn test_preferences_group_get_string() {
        let mut table = toml::Table::new();
        table.insert("key".to_string(), toml::Value::String("value".to_string()));
        let group = PreferencesGroup { table: &table };
        assert_eq!(group.get_string("key").unwrap(), "value");
    }

    #[test]
    fn test_preferences_group_get_integer() {
        let mut table = toml::Table::new();
        table.insert("key".to_string(), toml::Value::Integer(42));
        let group = PreferencesGroup { table: &table };
        assert_eq!(group.get_integer("key").unwrap(), 42);
    }

    #[test]
    fn test_preferences_group_get_float() {
        let mut table = toml::Table::new();
        table.insert("key".to_string(), toml::Value::Float(3.1));
        let group = PreferencesGroup { table: &table };
        assert_eq!(group.get_float("key").unwrap(), 3.1);
    }

    #[test]
    fn test_preferences_group_get_ivec2() {
        let mut table = toml::Table::new();
        table.insert(
            "key".to_string(),
            toml::Value::Array(vec![toml::Value::Integer(1), toml::Value::Integer(2)]),
        );
        let group = PreferencesGroup { table: &table };
        assert_eq!(group.get_ivec2("key").unwrap(), IVec2::new(1, 2));
    }

    #[test]
    fn test_preferences_group_get_uvec2() {
        let mut table = toml::Table::new();
        table.insert(
            "key".to_string(),
            toml::Value::Array(vec![toml::Value::Integer(1), toml::Value::Integer(2)]),
        );
        let group = PreferencesGroup { table: &table };
        assert_eq!(group.get_uvec2("key").unwrap(), UVec2::new(1, 2));
    }

    #[test]
    fn test_preferences_group_get_vec2() {
        let mut table = toml::Table::new();
        table.insert(
            "key".to_string(),
            toml::Value::Array(vec![toml::Value::Float(1.0), toml::Value::Float(2.0)]),
        );
        let group = PreferencesGroup { table: &table };
        assert_eq!(group.get_vec2("key").unwrap(), Vec2::new(1.0, 2.0));
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
        let group = PreferencesGroup { table: &table };
        assert_eq!(group.get_ivec3("key").unwrap(), IVec3::new(1, 2, 3));
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
        let group = PreferencesGroup { table: &table };
        assert_eq!(group.get_uvec3("key").unwrap(), UVec3::new(1, 2, 3));
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
        let group = PreferencesGroup { table: &table };
        assert_eq!(group.get_vec3("key").unwrap(), Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_preferences_group_mut_set_bool() {
        let mut table = toml::Table::new();
        let changed = AtomicBool::new(false);
        let mut group = PreferencesGroupMut {
            table: &mut table,
            changed: &changed,
        };
        group.set_bool("key", true);
        assert!(group.get_bool("key").unwrap());
        assert!(changed.load(std::sync::atomic::Ordering::Relaxed));

        changed.store(false, std::sync::atomic::Ordering::Relaxed);
        group.set_bool("key", true);
        assert!(group.get_bool("key").unwrap());
        assert!(!changed.load(std::sync::atomic::Ordering::Relaxed));
    }

    #[test]
    fn test_preferences_group_mut_set_string() {
        let mut table = toml::Table::new();
        let changed = AtomicBool::new(false);
        let mut group = PreferencesGroupMut {
            table: &mut table,
            changed: &changed,
        };
        group.set_string("key", "value");
        assert_eq!(group.get_string("key").unwrap(), "value");
        assert!(changed.load(std::sync::atomic::Ordering::Relaxed));
    }

    #[test]
    fn test_preferences_group_mut_set_integer() {
        let mut table = toml::Table::new();
        let changed = AtomicBool::new(false);
        let mut group = PreferencesGroupMut {
            table: &mut table,
            changed: &changed,
        };
        group.set_integer("key", 42);
        assert_eq!(group.get_integer("key").unwrap(), 42);
        assert!(changed.load(std::sync::atomic::Ordering::Relaxed));
    }

    #[test]
    fn test_preferences_group_mut_set_float() {
        let mut table = toml::Table::new();
        let changed = AtomicBool::new(false);
        let mut group = PreferencesGroupMut {
            table: &mut table,
            changed: &changed,
        };
        group.set_float("key", 3.1);
        assert_eq!(group.get_float("key").unwrap(), 3.1);
        assert!(changed.load(std::sync::atomic::Ordering::Relaxed));
    }

    #[test]
    fn test_preferences_group_mut_set_ivec2() {
        let mut table = toml::Table::new();
        let changed = AtomicBool::new(false);
        let mut group = PreferencesGroupMut {
            table: &mut table,
            changed: &changed,
        };
        group.set_ivec2("key", IVec2::new(1, 2));
        assert_eq!(group.get_ivec2("key").unwrap(), IVec2::new(1, 2));
        assert!(changed.load(std::sync::atomic::Ordering::Relaxed));
    }

    #[test]
    fn test_preferences_group_mut_set_uvec2() {
        let mut table = toml::Table::new();
        let changed = AtomicBool::new(false);
        let mut group = PreferencesGroupMut {
            table: &mut table,
            changed: &changed,
        };
        group.set_uvec2("key", UVec2::new(1, 2));
        assert_eq!(group.get_uvec2("key").unwrap(), UVec2::new(1, 2));
        assert!(changed.load(std::sync::atomic::Ordering::Relaxed));
    }

    #[test]
    fn test_preferences_group_mut_set_vec2() {
        let mut table = toml::Table::new();
        let changed = AtomicBool::new(false);
        let mut group = PreferencesGroupMut {
            table: &mut table,
            changed: &changed,
        };
        group.set_vec2("key", Vec2::new(1.0, 2.0));
        assert_eq!(group.get_vec2("key").unwrap(), Vec2::new(1.0, 2.0));
        assert!(changed.load(std::sync::atomic::Ordering::Relaxed));
    }

    #[test]
    fn test_preferences_group_mut_set_ivec3() {
        let mut table = toml::Table::new();
        let changed = AtomicBool::new(false);
        let mut group = PreferencesGroupMut {
            table: &mut table,
            changed: &changed,
        };
        group.set_ivec3("key", IVec3::new(1, 2, 3));
        assert_eq!(group.get_ivec3("key").unwrap(), IVec3::new(1, 2, 3));
        assert!(changed.load(std::sync::atomic::Ordering::Relaxed));
    }

    #[test]
    fn test_preferences_group_mut_set_uvec3() {
        let mut table = toml::Table::new();
        let changed = AtomicBool::new(false);
        let mut group = PreferencesGroupMut {
            table: &mut table,
            changed: &changed,
        };
        group.set_uvec3("key", UVec3::new(1, 2, 3));
        assert_eq!(group.get_uvec3("key").unwrap(), UVec3::new(1, 2, 3));
        assert!(changed.load(std::sync::atomic::Ordering::Relaxed));
    }

    #[test]
    fn test_preferences_group_mut_set_vec3() {
        let mut table = toml::Table::new();
        let changed = AtomicBool::new(false);
        let mut group = PreferencesGroupMut {
            table: &mut table,
            changed: &changed,
        };
        group.set_vec3("key", Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(group.get_vec3("key").unwrap(), Vec3::new(1.0, 2.0, 3.0));
        assert!(changed.load(std::sync::atomic::Ordering::Relaxed));

        changed.store(false, std::sync::atomic::Ordering::Relaxed);
        group.set_vec3("key", Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(group.get_vec3("key").unwrap(), Vec3::new(1.0, 2.0, 3.0));
        assert!(!changed.load(std::sync::atomic::Ordering::Relaxed));

        group.set_vec3("key", Vec3::new(3.0, 2.0, 1.0));
        assert_eq!(group.get_vec3("key").unwrap(), Vec3::new(3.0, 2.0, 1.0));
        assert!(changed.load(std::sync::atomic::Ordering::Relaxed));
    }
}

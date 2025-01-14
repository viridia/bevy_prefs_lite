use bevy::prelude::*;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{Map, Value as JsonValue};
use std::sync::atomic::{AtomicBool, Ordering};

/// Represents a single preferences file containing multiple groups of settings.
#[derive(Debug, Default)]
pub struct PreferencesFile {
    root: Map<String, JsonValue>,
    changed: AtomicBool,
}

impl PreferencesFile {
    /// Create a preferences file from a JSON table.
    pub(crate) fn from_string(json_str: &str, storage_key: &str) -> Self {
        let Ok(root) = serde_json::from_str::<Map<String, JsonValue>>(json_str) else {
            warn!(
                "Could not parse JSON from LocalStorage key: {}",
                storage_key
            );
            return Self::default();
        };
        Self {
            root,
            changed: AtomicBool::new(false),
        }
    }

    /// Get a preferences group from the file, or `None` if the group does not exist.
    pub fn get_group(&self, group: &str) -> Option<PreferencesGroup> {
        self.root
            .get(group)
            .and_then(|v| v.as_object())
            .map(|json| PreferencesGroup { json })
    }

    /// Get a mutable reference to a preferences group from the file, creating it if it does not
    /// exist.
    pub fn get_group_mut<'a>(&'a mut self, group: &str) -> Option<PreferencesGroupMut<'a>> {
        let entry = self
            .root
            .entry(group.to_owned())
            .or_insert_with(|| JsonValue::Object(Map::new()));
        entry.as_object_mut().map(|json| PreferencesGroupMut {
            json,
            changed: &mut self.changed,
        })
    }

    pub fn is_changed(&self) -> bool {
        self.changed.load(Ordering::Relaxed)
    }

    pub fn set_changed(&self) {
        self.changed.store(true, Ordering::Relaxed);
    }

    pub fn clear_changed(&self) {
        self.changed.store(false, Ordering::Relaxed);
    }

    pub(crate) fn encode(&self) -> String {
        serde_json::to_string(&self.root).unwrap()
    }
}

pub struct PreferencesGroup<'a> {
    json: &'a Map<String, JsonValue>,
}

pub struct PreferencesGroupMut<'a> {
    json: &'a mut Map<String, JsonValue>,
    changed: &'a AtomicBool,
}

impl PreferencesGroup<'_> {
    /// Get a key from the preferences group as a deserializable value, or `None` if the key does
    /// not exist or is not deserializable.
    pub fn get<D>(&self, key: &str) -> Option<D>
    where
        D: DeserializeOwned,
    {
        let value = self.json.get(key)?.clone();
        serde_json::from_value::<D>(value).ok()
    }

    /// Read a nested preferences group from the group, or `None` if the property does not exist or
    /// is not a table.
    pub fn get_group(&self, key: &str) -> Option<PreferencesGroup> {
        self.json
            .get(key)
            .and_then(|v| v.as_object())
            .map(|json| PreferencesGroup { json })
    }
}

impl PreferencesGroupMut<'_> {
    /// Delete a key from the preferences group.
    pub fn remove(&mut self, key: &str) {
        if self.json.remove(key).is_some() {
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
        let value = self.json.get(key)?.clone();
        serde_json::from_value::<D>(value).ok()
    }

    /// Set a key in the preferences group to a serializable value, and mark the file as changed.
    pub fn set<S: Serialize>(&mut self, key: &str, value: S) {
        let value = serde_json::to_value(value).unwrap();
        self.json.insert(key.to_owned(), value);
        self.changed
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// Convert `value` into a JSON value. If it is different than the current value, set the key
    /// in the preferences group to the new value, and mark the file as changed.
    pub fn set_if_changed<S: Serialize>(&mut self, key: &str, value: S) {
        let value = serde_json::to_value(value).unwrap();
        match self.json.get(key) {
            Some(v) if v == &value => (),
            _ => {
                self.json.insert(key.to_owned(), value);
                self.changed
                    .store(true, std::sync::atomic::Ordering::Relaxed);
            }
        }
    }

    /// Read a nested preferences group from the group, or `None` if the property does not exist or
    /// is not a table.
    pub fn get_group(&self, key: &str) -> Option<PreferencesGroup> {
        self.json
            .get(key)
            .and_then(|v| v.as_object())
            .map(|json| PreferencesGroup { json })
    }

    /// Get a mutable reference to a nested preferences group from the group, creating it if it
    /// does not exist.
    pub fn get_group_mut<'a>(&'a mut self, key: &str) -> Option<PreferencesGroupMut<'a>> {
        let entry = self.json.entry(key.to_owned()).or_insert_with(|| {
            self.changed
                .store(true, std::sync::atomic::Ordering::Relaxed);
            JsonValue::Object(Map::new())
        });
        entry.as_object_mut().map(|json| PreferencesGroupMut {
            json,
            changed: self.changed,
        })
    }
}

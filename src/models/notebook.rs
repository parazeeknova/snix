use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notebook {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
    pub color: String,
    pub icon: String,
    pub parent_id: Option<Uuid>,
    pub children: Vec<Uuid>,
    pub snippet_count: usize,
    pub metadata: HashMap<String, String>,
}

impl Notebook {
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description: None,
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
            color: String::from("#f38ba8"), // Rose Pine love color
            icon: String::from("○"),        // Unicode circle symbol instead of emoji
            parent_id: None,
            children: Vec::new(),
            snippet_count: 0,
            metadata: HashMap::new(),
        }
    }

    pub fn new_with_parent(name: String, parent_id: Uuid) -> Self {
        let mut notebook = Self::new(name);
        notebook.parent_id = Some(parent_id);
        notebook.icon = String::from("◉"); // Filled circle symbol instead of emoji
        notebook
    }

    pub fn add_child(&mut self, child_id: Uuid) {
        if !self.children.contains(&child_id) {
            self.children.push(child_id);
            self.updated_at = Utc::now();
        }
    }

    pub fn remove_child(&mut self, child_id: &Uuid) {
        self.children.retain(|id| id != child_id);
        self.updated_at = Utc::now();
    }

    pub fn update_snippet_count(&mut self, count: usize) {
        self.snippet_count = count;
        self.updated_at = Utc::now();
    }

    pub fn _is_root(&self) -> bool {
        self.parent_id.is_none()
    }

    pub fn _has_children(&self) -> bool {
        !self.children.is_empty()
    }

    /// Add a tag to the notebook
    pub fn _add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.updated_at = Utc::now();
        }
    }

    /// Remove a tag from the notebook
    pub fn _remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
        self.updated_at = Utc::now();
    }

    /// Set a metadata value
    pub fn _set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
        self.updated_at = Utc::now();
    }

    /// Get a metadata value
    pub fn _get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

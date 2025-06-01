use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Represents a tag that can be applied to snippets
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Tag {
    /// Unique identifier for the tag
    pub id: Uuid,

    /// Name of the tag (without the # prefix)
    pub name: String,

    /// Optional color for the tag (RGB hex code without #)
    pub color: Option<String>,

    /// When the tag was created
    pub created_at: DateTime<Utc>,

    /// When the tag was last used
    pub last_used_at: DateTime<Utc>,

    /// Number of times this tag has been used
    pub usage_count: usize,
}

impl Tag {
    /// Creates a new tag with the given name
    pub fn new(name: String) -> Self {
        // Remove leading # if present
        let clean_name = if name.starts_with('#') {
            name[1..].to_string()
        } else {
            name
        };

        Self {
            id: Uuid::new_v4(),
            name: clean_name,
            color: None,
            created_at: Utc::now(),
            last_used_at: Utc::now(),
            usage_count: 0,
        }
    }

    /// Returns the tag with a # prefix for display
    pub fn display_name(&self) -> String {
        format!("#{}", self.name)
    }

    /// Updates the usage count and last used time
    pub fn mark_used(&mut self) {
        self.usage_count += 1;
        self.last_used_at = Utc::now();
    }
}

/// Manages all tags and their associations with snippets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagManager {
    /// All known tags
    pub tags: HashMap<Uuid, Tag>,

    /// Maps snippet IDs to the set of tag IDs they have
    pub snippet_tags: HashMap<Uuid, HashSet<Uuid>>,

    /// Maps tag IDs to the set of snippet IDs that have that tag
    pub tag_snippets: HashMap<Uuid, HashSet<Uuid>>,
}

impl Default for TagManager {
    fn default() -> Self {
        Self {
            tags: HashMap::new(),
            snippet_tags: HashMap::new(),
            tag_snippets: HashMap::new(),
        }
    }
}

impl TagManager {
    /// Creates a new tag manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new tag or returns existing one with the same name
    pub fn create_tag(&mut self, name: String) -> Uuid {
        // Remove leading # if present
        let clean_name = if name.starts_with('#') {
            name[1..].to_string()
        } else {
            name
        };

        // Check if a tag with this name already exists
        for tag in self.tags.values() {
            if tag.name.to_lowercase() == clean_name.to_lowercase() {
                return tag.id;
            }
        }

        // Create a new tag
        let tag = Tag::new(clean_name);
        let tag_id = tag.id;
        self.tags.insert(tag_id, tag);
        self.tag_snippets.insert(tag_id, HashSet::new());

        tag_id
    }

    /// Add a tag to a snippet
    pub fn add_tag_to_snippet(&mut self, snippet_id: Uuid, tag_name: String) -> Uuid {
        let tag_id = self.create_tag(tag_name);

        // Update the tag's usage
        if let Some(tag) = self.tags.get_mut(&tag_id) {
            tag.mark_used();
        }

        // Update the snippet_tags mapping
        self.snippet_tags
            .entry(snippet_id)
            .or_insert_with(HashSet::new)
            .insert(tag_id);

        // Update the tag_snippets mapping
        self.tag_snippets
            .entry(tag_id)
            .or_insert_with(HashSet::new)
            .insert(snippet_id);

        tag_id
    }

    /// Get all snippets with a specific tag
    pub fn get_snippets_with_tag(&self, tag_id: &Uuid) -> Option<&HashSet<Uuid>> {
        self.tag_snippets.get(tag_id)
    }

    /// Find tags that match a query string
    pub fn find_tags_by_name(&self, query: &str) -> Vec<&Tag> {
        let query = query.to_lowercase();
        self.tags
            .values()
            .filter(|tag| tag.name.to_lowercase().contains(&query))
            .collect()
    }

    /// Handle when a snippet is deleted
    pub fn handle_snippet_deleted(&mut self, snippet_id: &Uuid) {
        // Get all tags associated with this snippet
        if let Some(tag_ids) = self.snippet_tags.remove(snippet_id) {
            // For each tag, remove this snippet from its set
            for tag_id in tag_ids {
                if let Some(snippets) = self.tag_snippets.get_mut(&tag_id) {
                    snippets.remove(snippet_id);

                    // If the tag is no longer used, remove it
                    if snippets.is_empty() {
                        self.tag_snippets.remove(&tag_id);
                        self.tags.remove(&tag_id);
                    }
                }
            }
        }
    }
}

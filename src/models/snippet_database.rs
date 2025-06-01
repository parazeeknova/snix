use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::models::{CodeSnippet, Notebook, TagManager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnippetDatabase {
    pub notebooks: HashMap<Uuid, Notebook>,
    pub root_notebooks: Vec<Uuid>,
    pub snippets: HashMap<Uuid, CodeSnippet>,
    pub tag_manager: TagManager,
}

impl Default for SnippetDatabase {
    fn default() -> Self {
        Self {
            notebooks: HashMap::new(),
            root_notebooks: Vec::new(),
            snippets: HashMap::new(),
            tag_manager: TagManager::new(),
        }
    }
}

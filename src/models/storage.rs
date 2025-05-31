use crate::models::{CodeSnippet, Notebook};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnippetDatabase {
    pub notebooks: HashMap<Uuid, Notebook>,
    pub snippets: HashMap<Uuid, CodeSnippet>,
    pub root_notebooks: Vec<Uuid>,
}

impl Default for SnippetDatabase {
    fn default() -> Self {
        Self {
            notebooks: HashMap::new(),
            snippets: HashMap::new(),
            root_notebooks: Vec::new(),
        }
    }
}

/// Storage Manager for disk operations
#[derive(Debug)]
pub struct StorageManager {
    _data_dir: PathBuf,
    snippets_dir: PathBuf,
    _notebooks_dir: PathBuf,
    database_file: PathBuf,
}

impl StorageManager {
    pub fn new() -> Result<Self> {
        let data_dir = dirs::data_dir()
            .context("Failed to get data directory")?
            .join("snix");

        let db_file = data_dir.join("database.json");
        let snippets_dir = data_dir.join("snippets");

        // Create directories if they don't exist
        fs::create_dir_all(&data_dir)?;
        fs::create_dir_all(&snippets_dir)?;

        Ok(Self {
            _data_dir: data_dir.clone(),
            snippets_dir,
            _notebooks_dir: data_dir,
            database_file: db_file,
        })
    }

    pub fn load_database(&self) -> Result<SnippetDatabase> {
        if !self.database_file.exists() {
            return Ok(SnippetDatabase::default());
        }

        let content =
            fs::read_to_string(&self.database_file).context("Failed to read database file")?;

        serde_json::from_str(&content).context("Failed to parse database JSON")
    }

    pub fn save_database(&self, db: &SnippetDatabase) -> Result<()> {
        let content = serde_json::to_string_pretty(db).context("Failed to serialize database")?;

        fs::write(&self.database_file, content).context("Failed to write database file")
    }

    pub fn save_snippet_content(&self, snippet: &CodeSnippet) -> Result<()> {
        let notebook_dir = self.snippets_dir.join(snippet.notebook_id.to_string());
        fs::create_dir_all(&notebook_dir)?;

        let filename = format!("{}.{}", snippet.id, snippet.file_extension);
        let file_path = notebook_dir.join(filename);

        fs::write(file_path, &snippet.content).context("Failed to write snippet content")
    }

    pub fn load_snippet_content(
        &self,
        snippet_id: Uuid,
        notebook_id: Uuid,
        extension: &str,
    ) -> Result<String> {
        let filename = format!("{}.{}", snippet_id, extension);
        let file_path = self
            .snippets_dir
            .join(notebook_id.to_string())
            .join(filename);

        if !file_path.exists() {
            return Ok(String::new());
        }

        fs::read_to_string(file_path).context("Failed to read snippet content")
    }

    pub fn delete_snippet_file(&self, snippet: &CodeSnippet) -> Result<()> {
        let filename = format!("{}.{}", snippet.id, snippet.file_extension);
        let file_path = self
            .snippets_dir
            .join(snippet.notebook_id.to_string())
            .join(filename);

        if file_path.exists() {
            fs::remove_file(file_path).context("Failed to delete snippet file")?;
        }

        Ok(())
    }

    pub fn delete_notebook_directory(&self, notebook_id: Uuid) -> Result<()> {
        let notebook_dir = self.snippets_dir.join(notebook_id.to_string());

        if notebook_dir.exists() {
            fs::remove_dir_all(notebook_dir).context("Failed to delete notebook directory")?;
        }

        Ok(())
    }

    pub fn get_snippet_file_path(&self, snippet: &CodeSnippet) -> PathBuf {
        let filename = format!("{}.{}", snippet.id, snippet.file_extension);
        self.snippets_dir
            .join(snippet.notebook_id.to_string())
            .join(filename)
    }
}

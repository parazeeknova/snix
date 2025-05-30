use crate::models::{CodeSnippet, Notebook};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
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
    _data_dir: PathBuf, // Prefix with underscore to mark as intentionally unused
    snippets_dir: PathBuf,
    _notebooks_dir: PathBuf, // Prefix with underscore to mark as intentionally unused
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

    /// Export a notebook to a file
    pub fn _export_notebook(
        &self,
        notebook_id: Uuid,
        db: &SnippetDatabase,
        export_path: &Path,
    ) -> Result<PathBuf> {
        let notebook = db
            .notebooks
            .get(&notebook_id)
            .context("Notebook not found")?;

        let export_dir = export_path.join(&notebook.name);
        fs::create_dir_all(&export_dir)?;

        // Export notebook metadata
        let metadata = serde_json::to_string_pretty(notebook)?;
        fs::write(export_dir.join("notebook.json"), metadata)?;

        // Export all snippets in this notebook
        for snippet in db.snippets.values() {
            if snippet.notebook_id == notebook_id {
                let content = self.load_snippet_content(
                    snippet.id,
                    snippet.notebook_id,
                    &snippet.file_extension,
                )?;

                let filename = format!(
                    "{}.{}",
                    snippet.title.replace(' ', "_"),
                    snippet.file_extension
                );
                fs::write(export_dir.join(filename), content)?;
            }
        }

        Ok(export_dir)
    }

    /// Import a notebook from a file
    pub fn _import_notebook(&self, import_path: &Path, db: &mut SnippetDatabase) -> Result<Uuid> {
        let metadata_file = import_path.join("notebook.json");

        if !metadata_file.exists() {
            return Err(anyhow::anyhow!(
                "Invalid notebook export: missing notebook.json"
            ));
        }

        let metadata_content = fs::read_to_string(metadata_file)?;
        let mut notebook: Notebook = serde_json::from_str(&metadata_content)?;

        // Generate new ID to avoid conflicts
        let _old_id = notebook.id;
        notebook.id = Uuid::new_v4();
        let new_notebook_id = notebook.id; // Store the ID before moving

        // Import all snippet files
        for entry in fs::read_dir(import_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.file_name() != Some(std::ffi::OsStr::new("notebook.json")) {
                // Try to extract filename and extension
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    if let Some((title, _extension)) = filename.rsplit_once('.') {
                        let content = fs::read_to_string(&path)?;
                        let extension = if let Some(ext) = Path::new(&filename).extension() {
                            ext.to_string_lossy().to_string()
                        } else {
                            "txt".to_string()
                        };

                        let language = crate::models::SnippetLanguage::_from_extension(&extension);

                        let mut snippet = CodeSnippet::new(
                            title.replace('_', " ").to_string(),
                            language,
                            new_notebook_id,
                        );
                        snippet.update_content(content);

                        self.save_snippet_content(&snippet)?;
                        db.snippets.insert(snippet.id, snippet);
                    }
                }
            }
        }

        db.notebooks.insert(new_notebook_id, notebook);
        db.root_notebooks.push(new_notebook_id);

        Ok(new_notebook_id)
    }

    /// Backup the database to a timestamped file
    pub fn _backup_database(&self) -> Result<PathBuf> {
        let backup_dir = self._data_dir.join("backups");
        fs::create_dir_all(&backup_dir)?;

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_file = backup_dir.join(format!("backup_{}.json", timestamp));

        fs::copy(&self.database_file, &backup_file)?;

        Ok(backup_file)
    }

    /// Get the data directory path
    pub fn _get_data_directory(&self) -> &Path {
        &self._data_dir
    }

    /// Get the snippets directory path
    pub fn _get_snippets_directory(&self) -> &Path {
        &self.snippets_dir
    }
}

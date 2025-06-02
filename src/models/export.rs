use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::storage::SnippetDatabase;
use crate::models::{CodeSnippet, Notebook, TagManager};

/// Export format options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExportFormat {
    JSON,
}

/// Export options for customizing what to export
#[derive(Debug, Clone)]
pub struct ExportOptions {
    pub format: ExportFormat,
    pub include_content: bool,
    pub notebook_ids: Option<Vec<Uuid>>,
    pub include_favorites_only: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            format: ExportFormat::JSON,
            include_content: true,
            notebook_ids: None,
            include_favorites_only: false,
        }
    }
}

/// Export file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportData {
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub notebooks: HashMap<Uuid, Notebook>,
    pub snippets: HashMap<Uuid, CodeSnippet>,
    pub root_notebooks: Vec<Uuid>,
    pub tags: HashMap<String, Vec<Uuid>>,
}

impl ExportData {
    /// Create a new export data object from the database
    pub fn from_database(db: &SnippetDatabase, options: &ExportOptions) -> Self {
        let mut notebooks = db.notebooks.clone();
        let mut snippets = HashMap::new();
        let mut root_notebooks = db.root_notebooks.clone();
        let tags = HashMap::new();

        // Filter notebooks if specific IDs were requested
        if let Some(notebook_ids) = &options.notebook_ids {
            notebooks.retain(|id, _| notebook_ids.contains(id));
            root_notebooks.retain(|id| notebook_ids.contains(id));
        }

        // Get all snippets, applying filters if needed
        for (id, snippet) in &db.snippets {
            let mut include = true;

            // Filter by notebook if needed
            if let Some(notebook_ids) = &options.notebook_ids {
                include = notebook_ids.contains(&snippet.notebook_id);
            }

            // Filter by favorites if needed
            if options.include_favorites_only {
                include = include && snippet.is_favorite;
            }

            if include {
                let mut snippet_clone = snippet.clone();

                // Optionally strip content to reduce export size
                if !options.include_content {
                    snippet_clone.content = String::new();
                }

                snippets.insert(*id, snippet_clone);
            }
        }

        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            created_at: Utc::now(),
            notebooks,
            snippets,
            root_notebooks,
            tags,
        }
    }

    /// Create a new export data object from the database including tag information
    pub fn from_database_with_tags(
        db: &SnippetDatabase,
        tag_manager: &TagManager,
        options: &ExportOptions,
    ) -> Self {
        let mut data = Self::from_database(db, options);

        // Initialize tags map
        let mut tags_map: HashMap<String, Vec<Uuid>> = HashMap::new();

        // Convert tag_manager structure to the expected format
        for (tag_id, tag) in &tag_manager.tags {
            if let Some(snippets) = tag_manager.tag_snippets.get(tag_id) {
                // Only include snippets that are in our export
                let included_snippets: Vec<Uuid> = snippets
                    .iter()
                    .filter(|snippet_id| data.snippets.contains_key(snippet_id))
                    .copied()
                    .collect();

                if !included_snippets.is_empty() {
                    tags_map.insert(tag.name.clone(), included_snippets);
                }
            }
        }

        data.tags = tags_map;
        data
    }
}

/// Export database to a file including tag information
pub fn export_database_with_tags(
    db: &SnippetDatabase,
    tag_manager: &TagManager,
    path: &Path,
    options: &ExportOptions,
) -> Result<()> {
    let export_data = ExportData::from_database_with_tags(db, tag_manager, options);

    // Export as JSON
    let json = serde_json::to_string_pretty(&export_data)
        .context("Failed to serialize database to JSON")?;
    fs::write(path, json).context("Failed to write JSON export file")?;

    Ok(())
}

/// Import database from a file
pub fn import_database(path: &Path) -> Result<ExportData> {
    let mut file = File::open(path).context("Failed to open import file")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .context("Failed to read import file")?;

    // Try to determine format from file extension
    if path.extension().map_or(false, |ext| ext == "json") {
        let data = serde_json::from_str(&contents).context("Failed to parse JSON import file")?;
        Ok(data)
    } else {
        // Try JSON
        serde_json::from_str(&contents).context("Failed to parse import file as JSON")
    }
}

/// Merge imported data into existing database
pub fn merge_import_into_database(
    db: &mut SnippetDatabase,
    import_data: ExportData,
    overwrite_existing: bool,
) -> Result<(usize, usize)> {
    // Returns (notebooks_added, snippets_added)
    let mut notebooks_added = 0;
    let mut snippets_added = 0;

    // Import notebooks
    for (id, notebook) in import_data.notebooks {
        if !db.notebooks.contains_key(&id) || overwrite_existing {
            db.notebooks.insert(id, notebook);
            notebooks_added += 1;
        }
    }

    // Import root notebooks
    for id in import_data.root_notebooks {
        if !db.root_notebooks.contains(&id) && db.notebooks.contains_key(&id) {
            db.root_notebooks.push(id);
        }
    }

    // Import snippets
    for (id, snippet) in import_data.snippets {
        if !db.snippets.contains_key(&id) || overwrite_existing {
            // Make sure the notebook exists
            if db.notebooks.contains_key(&snippet.notebook_id) {
                db.snippets.insert(id, snippet);
                snippets_added += 1;
            }
        }
    }

    Ok((notebooks_added, snippets_added))
}

/// Merge imported data including tags into existing database
pub fn merge_import_into_database_with_tags(
    db: &mut SnippetDatabase,
    tag_manager: &mut TagManager,
    import_data: ExportData,
    overwrite_existing: bool,
) -> Result<(usize, usize)> {
    // First merge the database content
    let (notebooks_added, snippets_added) =
        merge_import_into_database(db, import_data.clone(), overwrite_existing)?;

    // Then process tags
    for (tag_name, snippet_ids) in import_data.tags {
        for id in snippet_ids {
            if db.snippets.contains_key(&id) {
                tag_manager.add_tag_to_snippet(id, tag_name.clone());
            }
        }
    }

    Ok((notebooks_added, snippets_added))
}

/// Import from clipboard
pub fn import_from_clipboard() -> Result<Option<ExportData>> {
    #[cfg(not(target_os = "linux"))]
    {
        return Err(anyhow::anyhow!(
            "Clipboard import is only supported on Linux"
        ));
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;

        // Try to get clipboard content using xclip or wl-paste
        let output = Command::new("xclip")
            .args(["-selection", "clipboard", "-o"])
            .output()
            .or_else(|_| Command::new("wl-paste").output());

        match output {
            Ok(output) if output.status.success() => {
                let content = String::from_utf8(output.stdout)
                    .context("Clipboard content is not valid UTF-8")?;

                if content.is_empty() {
                    return Ok(None);
                }

                // Try parsing as JSON first
                let json_result = serde_json::from_str(&content);
                if let Ok(data) = json_result {
                    return Ok(Some(data));
                }

                // Then try YAML
                let yaml_result = serde_yaml::from_str(&content);
                if let Ok(data) = yaml_result {
                    return Ok(Some(data));
                }

                // Neither format worked
                Err(anyhow::anyhow!("Clipboard content is not a valid export"))
            }
            Ok(_) => Err(anyhow::anyhow!("Failed to read clipboard content")),
            Err(_) => Err(anyhow::anyhow!(
                "Clipboard tools not available (xclip or wl-paste)"
            )),
        }
    }
}

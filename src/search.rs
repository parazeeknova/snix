use crate::app::{App, SearchResult, SearchResultType};
use uuid::Uuid;

/// Performs a search across all notebooks, snippets, and content
/// Returns the number of results found
pub fn perform_search(app: &mut App, query: &str) -> usize {
    app.search_results.clear();
    app.selected_search_result = 0;

    if query.trim().is_empty() {
        return 0;
    }

    let query = query.to_lowercase();

    // Search in notebooks
    for (id, notebook) in &app.snippet_database.notebooks {
        if notebook.name.to_lowercase().contains(&query) {
            app.search_results.push(SearchResult {
                id: *id,
                name: notebook.name.clone(),
                result_type: SearchResultType::Notebook,
                match_context: format!("Notebook name match: {}", notebook.name),
                parent_id: notebook.parent_id,
            });
        }

        // Search in notebook descriptions
        if let Some(desc) = &notebook.description {
            if desc.to_lowercase().contains(&query) {
                app.search_results.push(SearchResult {
                    id: *id,
                    name: notebook.name.clone(),
                    result_type: SearchResultType::Notebook,
                    match_context: format!("Description: {}", desc),
                    parent_id: notebook.parent_id,
                });
            }
        }
    }

    // Search in snippets
    for (id, snippet) in &app.snippet_database.snippets {
        // Search in snippet titles
        if snippet.title.to_lowercase().contains(&query) {
            app.search_results.push(SearchResult {
                id: *id,
                name: snippet.title.clone(),
                result_type: SearchResultType::Snippet,
                match_context: format!("Snippet title match: {}", snippet.title),
                parent_id: Some(snippet.notebook_id),
            });
        }

        // Search in snippet descriptions
        if let Some(desc) = &snippet.description {
            if desc.to_lowercase().contains(&query) {
                app.search_results.push(SearchResult {
                    id: *id,
                    name: snippet.title.clone(),
                    result_type: SearchResultType::Snippet,
                    match_context: format!("Description: {}", desc),
                    parent_id: Some(snippet.notebook_id),
                });
            }
        }

        // Search in snippet content
        if snippet.content.to_lowercase().contains(&query) {
            // Find the matching line(s) for context
            let mut match_context = String::new();
            for (i, line) in snippet.content.lines().enumerate() {
                if line.to_lowercase().contains(&query) {
                    let line_num = i + 1;
                    let trimmed_line = line.trim();
                    match_context = format!("Line {}: {}", line_num, trimmed_line);
                    break;
                }
            }

            app.search_results.push(SearchResult {
                id: *id,
                name: snippet.title.clone(),
                result_type: SearchResultType::CodeContent,
                match_context,
                parent_id: Some(snippet.notebook_id),
            });
        }
    }

    app.search_results.len()
}

/// Gets the parent path for a notebook or snippet result
pub fn get_parent_path(app: &App, parent_id: Option<Uuid>) -> String {
    let mut path = Vec::new();
    let mut current_id = parent_id;

    // Walk up the parent chain
    while let Some(id) = current_id {
        if let Some(notebook) = app.snippet_database.notebooks.get(&id) {
            path.push(notebook.name.clone());
            current_id = notebook.parent_id;
        } else {
            break;
        }
    }

    // Reverse the path and join with ">"
    path.reverse();
    path.join(" > ")
}

/// Opens the selected search result
/// Returns true if the result was successfully opened
pub fn open_selected_search_result(app: &mut App) -> bool {
    if app.search_results.is_empty() {
        return false;
    }

    let result = &app.search_results[app.selected_search_result];

    match result.result_type {
        SearchResultType::Notebook => {
            app.current_notebook_id = Some(result.id);
            app.code_snippets_state = crate::app::CodeSnippetsState::NotebookDetails {
                notebook_id: result.id,
            };

            // Find the notebook in the tree view and select it
            if let Some(index) = app.tree_items.iter().position(|item| {
                if let crate::app::TreeItem::Notebook(id, _) = item {
                    *id == result.id
                } else {
                    false
                }
            }) {
                app.selected_tree_item = index;
            }

            true
        }
        SearchResultType::Snippet | SearchResultType::CodeContent => {
            // Find the snippet and mark it as accessed
            if let Some(snippet) = app.snippet_database.snippets.get_mut(&result.id) {
                snippet.mark_accessed();
                let _ = app.save_database();
            }

            // Find the snippet in the tree view and select it
            if let Some(index) = app.tree_items.iter().position(|item| {
                if let crate::app::TreeItem::Snippet(id, _) = item {
                    *id == result.id
                } else {
                    false
                }
            }) {
                app.selected_tree_item = index;
            }

            // Launch the external editor for this snippet
            crate::handlers::keys::launch_external_editor(app, result.id);
            true
        }
    }
}

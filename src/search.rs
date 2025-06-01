use crate::app::{App, RecentSearchEntry, SearchResult, SearchResultType};
use uuid::Uuid;

const MAX_RECENT_SEARCHES: usize = 20;

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

    let result_count = app.search_results.len();
    save_to_recent_searches(app, query.clone(), result_count);

    result_count
}

/// Saves a search query to the recent searches list & Don't save empty queries
fn save_to_recent_searches(app: &mut App, query: String, result_count: usize) {
    if query.trim().is_empty() {
        return;
    }

    // Remove this query if it already exists (to avoid duplicates)
    app.recent_searches.retain(|entry| entry.query != query);

    // Create a new search entry and add it to the beginning
    let entry = RecentSearchEntry::new(query, result_count);
    app.recent_searches.insert(0, entry);

    // Trim the list if it exceeds the maximum number of recent searches
    if app.recent_searches.len() > MAX_RECENT_SEARCHES {
        app.recent_searches.truncate(MAX_RECENT_SEARCHES);
    }

    // Reset the selected index
    app.selected_recent_search = 0;
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

    // Clone the necessary data from the result before modifying app
    let result_index = app.selected_search_result;
    let result_id = app.search_results[result_index].id;
    let result_type = app.search_results[result_index].result_type.clone();
    let parent_id = app.search_results[result_index].parent_id;

    // Update the last selected item in recent searches
    if let Some(entry) = app.recent_searches.first_mut() {
        if entry.query == app.search_query {
            entry.last_selected_type = Some(result_type.clone());
            entry.last_selected_id = Some(result_id);
        }
    }

    match result_type {
        SearchResultType::Notebook => {
            app.refresh_tree_items();

            // Find the index of this notebook in the tree
            if let Some(index) = app.tree_items.iter().position(
                |item| matches!(item, crate::app::TreeItem::Notebook(id, _) if *id == result_id),
            ) {
                // Set the selected tree item to this notebook
                app.selected_tree_item = index;

                // If the notebook is collapsed, expand it
                app.expand_notebook(result_id);

                // Set the code snippets state to NotebookView
                app.code_snippets_state = crate::app::CodeSnippetsState::NotebookView {
                    notebook_id: result_id,
                };

                return true;
            }
        }
        SearchResultType::Snippet | SearchResultType::CodeContent => {
            // For snippets or code content, we need to:
            // 1. Find the notebook this snippet belongs to
            // 2. Make sure the notebook is expanded
            // 3. Set the selected tree item to this snippet

            app.refresh_tree_items();
            if let Some(index) = app.tree_items.iter().position(
                |item| matches!(item, crate::app::TreeItem::Snippet(id, _) if *id == result_id),
            ) {
                app.selected_tree_item = index;

                if let Some(notebook_id) = parent_id {
                    app.expand_notebook(notebook_id);
                }

                return true;
            }
        }
    }

    false
}

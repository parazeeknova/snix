use crate::models::StorageManager;
use colored::Colorize;
use std::error::Error;
use uuid::Uuid;

/// Shows the content of a specific snippet by ID or name
pub fn show_snippet(name_or_id: &str) -> Result<(), Box<dyn Error>> {
    let storage = StorageManager::new()?;
    let database = storage.load_database()?;

    // First try parsing as UUID
    let snippet_id = match Uuid::parse_str(name_or_id) {
        Ok(id) => Some(id),
        Err(_) => {
            // If not a valid UUID, try to find by name
            let name = name_or_id.to_lowercase();

            // Try exact match first
            let exact_match = database
                .snippets
                .values()
                .find(|s| s.title.to_lowercase() == name);

            if let Some(snippet) = exact_match {
                Some(snippet.id)
            } else {
                // Then try partial match
                let partial_match = database
                    .snippets
                    .values()
                    .find(|s| s.title.to_lowercase().contains(&name));

                partial_match.map(|s| s.id)
            }
        }
    };

    match snippet_id {
        Some(id) => {
            if let Some(snippet) = database.snippets.get(&id) {
                display_snippet_content(snippet, &database);
            } else {
                println!(
                    "{}  Snippet not found with ID: {}",
                    "┃".bright_magenta(),
                    id
                );
            }
        }
        None => {
            println!(
                "{}  No snippet found with name: {}",
                "┃".bright_magenta(),
                name_or_id
            );
            println!("{}  Available snippets:", "┃".bright_magenta());
            println!("{}", "─".repeat(60).bright_magenta());

            // List available snippets to help the user
            for (idx, snippet) in database.snippets.values().enumerate().take(10) {
                println!(
                    "{}  {}. {}",
                    "┃".bright_magenta(),
                    (idx + 1).to_string().yellow(),
                    snippet.title.bright_white()
                );
            }

            if database.snippets.len() > 10 {
                println!(
                    "{}  ... and {} more",
                    "┃".bright_magenta(),
                    database.snippets.len() - 10
                );
            }
        }
    }

    Ok(())
}

/// Helper function to display snippet content
fn display_snippet_content(
    snippet: &crate::models::CodeSnippet,
    database: &crate::models::storage::SnippetDatabase,
) {
    // Find the notebook name
    let notebook_name = database
        .notebooks
        .get(&snippet.notebook_id)
        .map(|n| n.name.as_str())
        .unwrap_or("Unknown");

    // Find the full path
    let path = get_snippet_path(snippet, database);

    println!(
        "{}  {} {}",
        "┃".bright_magenta(),
        "SNIPPET".bright_green().bold(),
        snippet.title.bold()
    );
    println!("{}", "─".repeat(60).bright_magenta());

    println!(
        "{}  {}: {}",
        "┃".bright_magenta(),
        "Notebook".bright_blue(),
        notebook_name
    );
    println!(
        "{}  {}: {}",
        "┃".bright_magenta(),
        "Path".bright_magenta(),
        path
    );
    println!(
        "{}  {}: {}",
        "┃".bright_magenta(),
        "Language".bright_yellow(),
        snippet.language.display_name()
    );
    if let Some(desc) = &snippet.description {
        println!(
            "{}  {}: {}",
            "┃".bright_magenta(),
            "Description".bright_cyan(),
            desc
        );
    }
    println!(
        "{}  {}: {}",
        "┃".bright_magenta(),
        "ID".bright_black(),
        snippet.id
    );
    println!("{}", "─".repeat(60).bright_magenta());

    // Content with basic formatting
    // Split by lines and add the margin to each line
    for line in snippet.content.lines() {
        println!("{}  {}", "┃".bright_magenta(), line);
    }
}

/// Get the full path of a snippet (notebook/subnotebook/snippet)
fn get_snippet_path(
    snippet: &crate::models::CodeSnippet,
    database: &crate::models::storage::SnippetDatabase,
) -> String {
    let mut path_components = Vec::new();
    path_components.push(snippet.title.clone());

    let mut current_id = snippet.notebook_id;
    while let Some(notebook) = database.notebooks.get(&current_id) {
        path_components.push(notebook.name.clone());

        if let Some(parent_id) = notebook.parent_id {
            current_id = parent_id;
        } else {
            break;
        }
    }

    path_components.reverse();
    path_components.join("/")
}

/// Searches for snippets matching a query string
pub fn search_snippets(query: &str) -> Result<(), Box<dyn Error>> {
    let storage = StorageManager::new()?;
    let database = storage.load_database()?;

    let mut results = Vec::new();

    // Search in titles
    for snippet in database.snippets.values() {
        if snippet.title.to_lowercase().contains(&query.to_lowercase()) {
            results.push((snippet.id, "title", snippet));
            continue;
        }

        // Search in content
        if snippet
            .content
            .to_lowercase()
            .contains(&query.to_lowercase())
        {
            results.push((snippet.id, "content", snippet));
            continue;
        }

        // Search in description
        if let Some(desc) = &snippet.description {
            if desc.to_lowercase().contains(&query.to_lowercase()) {
                results.push((snippet.id, "description", snippet));
            }
        }
    }

    println!(
        "{}  {} '{}'",
        "┃".bright_magenta(),
        "SEARCH RESULTS FOR".bold(),
        query.bright_white()
    );

    if results.is_empty() {
        println!(
            "{}  No snippets found matching query: {}",
            "┃".bright_magenta(),
            query
        );

        return Ok(());
    }

    println!(
        "{}  Found {} snippets matching '{}':",
        "┃".bright_magenta(),
        results.len(),
        query
    );
    println!("{}", "─".repeat(60).bright_magenta());

    for (idx, (id, match_type, snippet)) in results.iter().enumerate() {
        let path = get_snippet_path(snippet, &database);

        println!(
            "{}  {}. {} (match in: {})",
            "┃".bright_magenta(),
            (idx + 1).to_string().bright_yellow(),
            snippet.title.bright_white().bold(),
            match_type.bright_green()
        );
        println!(
            "{}     {}: {}",
            "┃".bright_magenta(),
            "Path".bright_blue(),
            path
        );
        println!(
            "{}     {}: {}",
            "┃".bright_magenta(),
            "ID".bright_black(),
            id
        );

        if idx < results.len() - 1 {
            println!(
                "{}  {}",
                "┃".bright_magenta(),
                "─".repeat(40).bright_black()
            );
        }
    }

    Ok(())
}

/// Lists all favorite snippets
pub fn list_favorites() -> Result<(), Box<dyn Error>> {
    let storage = StorageManager::new()?;
    let database = storage.load_database()?;

    let favorites: Vec<_> = database
        .snippets
        .values()
        .filter(|s| s.is_favorited())
        .collect();

    if favorites.is_empty() {
        println!("{}  No favorite snippets found.", "┃".bright_magenta());
        return Ok(());
    }

    println!(
        "{}  {} favorite snippets:",
        "┃".bright_magenta(),
        favorites.len()
    );

    for (idx, snippet) in favorites.iter().enumerate() {
        let path = get_snippet_path(snippet, &database);

        println!(
            "{}  {}. {} {}",
            "┃".bright_magenta(),
            (idx + 1).to_string().bright_yellow(),
            "".yellow(),
            snippet.title.bright_white().bold()
        );
        println!(
            "{}     {}: {}",
            "┃".bright_magenta(),
            "Path".bright_blue(),
            path
        );
        println!(
            "{}     {}: {}",
            "┃".bright_magenta(),
            "Language".bright_green(),
            snippet.language.display_name()
        );
        println!(
            "{}     {}: {}",
            "┃".bright_magenta(),
            "ID".bright_black(),
            snippet.id
        );

        if idx < favorites.len() - 1 {
            println!(
                "{}  {}",
                "┃".bright_magenta(),
                "─".repeat(40).bright_black()
            );
        }
    }
    Ok(())
}

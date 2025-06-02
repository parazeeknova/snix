use crate::models::storage::SnippetDatabase;
use colored::Colorize;
use std::error::Error;
use uuid::Uuid;

/// Displays the database content in a tree-like structure
pub fn display_tree(
    database: &SnippetDatabase,
    root_id: Option<Uuid>,
) -> Result<(), Box<dyn Error>> {
    if database.notebooks.is_empty() {
        println!("No notebooks found in database.");
        return Ok(());
    }

    match root_id {
        Some(id) => {
            if let Some(notebook) = database.notebooks.get(&id) {
                println!(
                    "{}  {} {}",
                    "┃".bright_magenta(),
                    "󰠮".bright_blue(),
                    notebook.name.bold()
                );
                print_notebook_contents(database, id, 1, &notebook.name, vec![]);
            } else {
                println!(
                    "{}  Notebook with ID {} not found",
                    "┃".bright_magenta(),
                    id
                );
            }
        }
        None => {
            let count = database.root_notebooks.len();

            for (idx, notebook_id) in database.root_notebooks.iter().enumerate() {
                if let Some(notebook) = database.notebooks.get(notebook_id) {
                    let is_last = idx == count - 1;
                    println!(
                        "{}  {} {}",
                        "┃".bright_magenta(),
                        "󰠮".bright_blue(),
                        notebook.name.bold()
                    );

                    // Create guide vector - true means draw line, false means space
                    let mut guides = Vec::new();

                    // Add guide for this level
                    if is_last {
                        guides.push(false); // Last item doesn't need a line below it
                    } else {
                        guides.push(true); // Not last, so draw line for following siblings
                    }

                    print_notebook_contents(database, *notebook_id, 1, &notebook.name, guides);
                }
            }
        }
    }
    Ok(())
}

/// Recursively prints the contents of a notebook
fn print_notebook_contents(
    database: &SnippetDatabase,
    notebook_id: Uuid,
    depth: usize,
    path: &str,
    guides: Vec<bool>,
) {
    // Get all snippets in this notebook
    let snippets: Vec<_> = database
        .snippets
        .values()
        .filter(|s| s.notebook_id == notebook_id)
        .collect();

    // Get all child notebooks
    let children: Vec<_> = if let Some(notebook) = database.notebooks.get(&notebook_id) {
        notebook
            .children
            .iter()
            .filter_map(|id| database.notebooks.get(id).map(|n| (*id, n)))
            .collect()
    } else {
        Vec::new()
    };

    // Display snippets first
    for (i, snippet) in snippets.iter().enumerate() {
        let is_last_snippet = i == snippets.len() - 1;
        let is_last_item = is_last_snippet && children.is_empty();

        let language_icon = snippet.language.icon();
        let star = if snippet.is_favorited() {
            " ".yellow()
        } else {
            "".normal()
        };

        let full_path = format!("{}/{}", path, snippet.title);

        // Print left margin with indentation guides
        print!("{}  ", "┃".bright_magenta());

        // Print the tree indentation guides
        for guide in &guides {
            if *guide {
                print!("┃  ");
            } else {
                print!("   ");
            }
        }

        // Print the item connector
        if is_last_item {
            print!("└── ");
        } else {
            print!("├── ");
        }

        // Print the actual snippet content
        println!(
            "{}{} {} [{}] {}",
            star,
            language_icon,
            snippet.title.bright_white(),
            snippet.language.short_name().bright_black(),
            full_path.bright_black().italic()
        );
    }

    // Display child notebooks
    for (i, (child_id, child)) in children.iter().enumerate() {
        let is_last = i == children.len() - 1;
        let child_path = format!("{}/{}", path, child.name);

        // Print left margin with indentation guides
        print!("{}  ", "┃".bright_magenta());

        // Print the tree indentation guides
        for guide in &guides {
            if *guide {
                print!("┃  ");
            } else {
                print!("   ");
            }
        }

        if is_last {
            print!("└── ");
        } else {
            print!("├── ");
        }

        println!(
            "{} {} {}",
            "󰠮".bright_blue(),
            child.name.bold(),
            child_path.bright_black().italic()
        );

        // Create guide vector for the next level
        let mut next_guides = guides.clone();

        // Add guide for this level
        if is_last {
            next_guides.push(false); // Last item doesn't need a line below it
        } else {
            next_guides.push(true); // Not last, so draw line for following siblings
        }

        // Recursively print children
        print_notebook_contents(database, *child_id, depth + 1, &child_path, next_guides);
    }
}

/// Find notebook ID by name (case insensitive, partial match)
pub fn find_notebook_by_name(database: &SnippetDatabase, name: &str) -> Option<Uuid> {
    // Try exact match first
    for (id, notebook) in &database.notebooks {
        if notebook.name.to_lowercase() == name.to_lowercase() {
            return Some(*id);
        }
    }

    // Try partial match if exact match failed
    for (id, notebook) in &database.notebooks {
        if notebook.name.to_lowercase().contains(&name.to_lowercase()) {
            return Some(*id);
        }
    }

    None
}

pub fn list_all_notebooks(database: &SnippetDatabase) -> Result<(), Box<dyn Error>> {
    for (idx, (id, notebook)) in database.notebooks.iter().enumerate() {
        let parent_name = if let Some(parent_id) = notebook.parent_id {
            database
                .notebooks
                .get(&parent_id)
                .map(|n| format!(" (in {})", n.name))
                .unwrap_or_default()
        } else {
            " (root)".to_string()
        };

        println!(
            "{}  {}. {} {}{}",
            "┃".bright_magenta(),
            (idx + 1).to_string().bright_yellow(),
            notebook.name.bright_white().bold(),
            parent_name.bright_black(),
            format!(" [{}]", id).bright_black().italic()
        );
    }
    Ok(())
}

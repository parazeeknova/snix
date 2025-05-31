//! Keyboard Input Handling Module
//!
//! This module provides comprehensive keyboard input handling for the RustUI application.
//! It processes all user keyboard interactions and translates them into appropriate
//! application state changes, navigation actions, and menu interactions.

use crate::app::{App, AppState, CodeSnippetsState, InputMode, TreeItem};
use crate::models::SnippetLanguage;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Main keyboard event handler and dispatcher
/// This is the primary entry point for all keyboard input processing. It receives
/// key events from the terminal and routes them to appropriate specialized handlers
/// based on the current application state.
pub fn handle_key_events(key: KeyEvent, app: &mut App) -> bool {
    // Handle input mode first for code snippets
    if app.state == AppState::CodeSnippets && app.input_mode != InputMode::Normal {
        return handle_input_mode_keys(key, app);
    }

    match key.code {
        // Global quit command - works from any page
        KeyCode::Char('q') | KeyCode::Char('Q') => {
            if app.state == AppState::StartPage || app.state != AppState::CodeSnippets {
                return true;
            }
            false
        }

        // Help menu toggle (works from any page)
        KeyCode::Char('?') => {
            app.clear_messages();
            if app.input_mode == InputMode::HelpMenu {
                app.input_mode = InputMode::Normal;
            } else {
                app.input_mode = InputMode::HelpMenu;
            }
            false
        }

        // Global back navigation - only works if there's history to go back to
        KeyCode::Backspace => {
            if app.can_go_back() {
                app.go_back();
            }
            false
        }

        // Route to state-specific key handlers
        _ => match app.state {
            AppState::StartPage => handle_start_page_keys(key, app),
            AppState::CodeSnippets => handle_code_snippets_keys(key, app),
            _ => handle_other_page_keys(key, app),
        },
    }
}

/// Handles keyboard input for input mode in code snippets
fn handle_input_mode_keys(key: KeyEvent, app: &mut App) -> bool {
    match key.code {
        KeyCode::Esc => {
            // Close any input mode including help menu
            app.input_mode = InputMode::Normal;
            app.input_buffer.clear();
            app.pending_snippet_title.clear();
            app.clear_messages();
            false
        }
        KeyCode::Enter => {
            let input = app.input_buffer.trim().to_string();
            app.input_buffer.clear();

            match app.input_mode.clone() {
                InputMode::CreateNotebook => {
                    if !input.is_empty() {
                        // Clear current_notebook_id to create a root notebook
                        app.current_notebook_id = None;
                        match app.create_notebook(input) {
                            Ok(_) => {
                                app.set_success_message(
                                    "Notebook created successfully!".to_string(),
                                );
                            }
                            Err(e) => {
                                app.set_error_message(e);
                            }
                        }
                    }
                    app.input_mode = InputMode::Normal;
                }
                InputMode::CreateNestedNotebook => {
                    if !input.is_empty() {
                        // current_notebook_id or selected notebook will be used as parent
                        if app.current_notebook_id.is_none() {
                            if let Some(TreeItem::Notebook(id, _)) = app.get_selected_item() {
                                app.current_notebook_id = Some(*id);
                            }
                        }

                        match app.create_notebook(input) {
                            Ok(_) => {
                                app.set_success_message(
                                    "Nested notebook created successfully!".to_string(),
                                );
                            }
                            Err(e) => {
                                app.set_error_message(e);
                            }
                        }
                    }
                    app.input_mode = InputMode::Normal;
                }
                InputMode::CreateSnippet => {
                    if !input.is_empty() {
                        // Check if user provided a file extension
                        let (title, language) = if input.contains('.') {
                            let parts: Vec<&str> = input.rsplitn(2, '.').collect();
                            let extension = parts[0].to_lowercase();
                            let title = parts[1].to_string();

                            // Get language from extension
                            let language = match extension.as_str() {
                                "rs" => SnippetLanguage::Rust,
                                "js" => SnippetLanguage::JavaScript,
                                "ts" => SnippetLanguage::TypeScript,
                                "py" => SnippetLanguage::Python,
                                "go" => SnippetLanguage::Go,
                                "java" => SnippetLanguage::Java,
                                "c" => SnippetLanguage::C,
                                "cpp" | "cc" | "cxx" => SnippetLanguage::Cpp,
                                "cs" => SnippetLanguage::CSharp,
                                "php" => SnippetLanguage::PHP,
                                "rb" => SnippetLanguage::Ruby,
                                "swift" => SnippetLanguage::Swift,
                                "kt" => SnippetLanguage::Kotlin,
                                "dart" => SnippetLanguage::Dart,
                                "html" => SnippetLanguage::HTML,
                                "css" => SnippetLanguage::CSS,
                                "scss" => SnippetLanguage::SCSS,
                                "sql" => SnippetLanguage::SQL,
                                "sh" | "bash" => SnippetLanguage::Bash,
                                "ps1" => SnippetLanguage::PowerShell,
                                "yml" | "yaml" => SnippetLanguage::Yaml,
                                "json" => SnippetLanguage::Json,
                                "xml" => SnippetLanguage::Xml,
                                "md" => SnippetLanguage::Markdown,
                                "dockerfile" => SnippetLanguage::Dockerfile,
                                "toml" => SnippetLanguage::Toml,
                                "ini" => SnippetLanguage::Ini,
                                "conf" | "config" => SnippetLanguage::Config,
                                _ => SnippetLanguage::Text,
                            };

                            (title, language)
                        } else {
                            // No extension provided, use plain text
                            (input, SnippetLanguage::Text)
                        };

                        // Get notebook_id for the snippet
                        if let Some(notebook_id) = get_current_notebook_id(app) {
                            match app.create_snippet(title, language, notebook_id) {
                                Ok(_snippet_id) => {
                                    app.set_success_message(
                                        "Snippet created successfully!".to_string(),
                                    );
                                    // Set the code snippets state back to NotebookList to show the new snippet
                                    app.code_snippets_state = CodeSnippetsState::NotebookList;
                                    app.refresh_tree_items();
                                }
                                Err(e) => {
                                    app.set_error_message(e);
                                }
                            }
                        } else {
                            app.set_error_message("No notebook selected".to_string());
                        }

                        app.input_mode = InputMode::Normal;
                    } else {
                        app.input_mode = InputMode::Normal;
                        app.code_snippets_state = CodeSnippetsState::NotebookList;
                        app.clear_messages();
                    }
                }
                InputMode::SelectLanguage => {
                    // This shouldn't happen with Enter, language selection uses different keys
                    app.input_mode = InputMode::Normal;
                    app.pending_snippet_title.clear();
                    app.clear_messages();

                    // Ensure we go back to notebook list view
                    app.code_snippets_state = CodeSnippetsState::NotebookList;
                }
                InputMode::Search => {
                    // TODO: Implement search
                    app.search_query = input;
                    app.input_mode = InputMode::Normal;
                    app.code_snippets_state = CodeSnippetsState::SearchSnippets;
                    app.clear_messages();
                }
                _ => {
                    app.input_mode = InputMode::Normal;
                    app.clear_messages();
                }
            }
            false
        }
        KeyCode::Backspace => {
            if !app.input_buffer.is_empty() {
                app.input_buffer.pop();
            }
            false
        }
        KeyCode::Up | KeyCode::Char('k') if app.input_mode == InputMode::SelectLanguage => {
            app.selected_language = if app.selected_language == 0 {
                get_available_languages().len() - 1
            } else {
                app.selected_language - 1
            };
            false
        }
        KeyCode::Down | KeyCode::Char('j') if app.input_mode == InputMode::SelectLanguage => {
            app.selected_language = (app.selected_language + 1) % get_available_languages().len();
            false
        }
        KeyCode::Char(c) => {
            if app.input_mode != InputMode::SelectLanguage {
                app.input_buffer.push(c);
            }
            false
        }
        _ => false,
    }
}

/// Get list of available languages for snippet creation
fn get_available_languages() -> Vec<SnippetLanguage> {
    vec![
        SnippetLanguage::Rust,
        SnippetLanguage::JavaScript,
        SnippetLanguage::TypeScript,
        SnippetLanguage::Python,
        SnippetLanguage::Go,
        SnippetLanguage::Java,
        SnippetLanguage::C,
        SnippetLanguage::Cpp,
        SnippetLanguage::CSharp,
        SnippetLanguage::PHP,
        SnippetLanguage::Ruby,
        SnippetLanguage::Swift,
        SnippetLanguage::Kotlin,
        SnippetLanguage::Dart,
        SnippetLanguage::HTML,
        SnippetLanguage::CSS,
        SnippetLanguage::SCSS,
        SnippetLanguage::SQL,
        SnippetLanguage::Bash,
        SnippetLanguage::PowerShell,
        SnippetLanguage::Yaml,
        SnippetLanguage::Json,
        SnippetLanguage::Xml,
        SnippetLanguage::Markdown,
        SnippetLanguage::Dockerfile,
        SnippetLanguage::Toml,
        SnippetLanguage::Ini,
        SnippetLanguage::Config,
        SnippetLanguage::Text,
    ]
}

/// Handles keyboard input specifically for the code snippets page
fn handle_code_snippets_keys(key: KeyEvent, app: &mut App) -> bool {
    match app.code_snippets_state {
        CodeSnippetsState::NotebookList => handle_notebook_list_keys(key, app),
        CodeSnippetsState::NotebookView { notebook_id } => {
            handle_notebook_view_keys(key, app, notebook_id)
        }
        CodeSnippetsState::_SnippetEditor { snippet_id } => {
            handle_snippet_editor_keys(key, app, snippet_id)
        }
        CodeSnippetsState::SearchSnippets => handle_search_keys(key, app),
        _ => handle_other_snippets_keys(key, app),
    }
}

/// Handles keys for the main notebook list view
fn handle_notebook_list_keys(key: KeyEvent, app: &mut App) -> bool {
    if key.code == KeyCode::Enter && (app.error_message.is_some() || app.success_message.is_some())
    {
        app.clear_messages();
        return false;
    }

    if app.input_mode == InputMode::Normal {
        match key.code {
            KeyCode::Esc
            | KeyCode::Up
            | KeyCode::Down
            | KeyCode::Char('j')
            | KeyCode::Char('k')
            | KeyCode::Char('h')
            | KeyCode::Char('H') => {
                app.clear_messages();
            }
            _ => {}
        }
    }

    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            app.previous_tree_item();
            false
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.next_tree_item();
            false
        }

        // Enter selected item (notebook or snippet)
        KeyCode::Enter => {
            // If there's a message showing, Enter should just dismiss it
            if app.error_message.is_some() || app.success_message.is_some() {
                app.clear_messages();
                return false;
            }

            if let Some(selected_item) = app.get_selected_item().cloned() {
                match selected_item {
                    TreeItem::Notebook(notebook_id, _) => {
                        app.current_notebook_id = Some(notebook_id);
                        app.code_snippets_state = CodeSnippetsState::NotebookView { notebook_id };
                    }
                    TreeItem::Snippet(snippet_id, _) => {
                        if let Some(snippet) = app.snippet_database.snippets.get_mut(&snippet_id) {
                            snippet.mark_accessed();
                        }
                        let _ = app.save_database();
                        launch_external_editor(app, snippet_id);
                    }
                }
            }
            false
        }

        KeyCode::Char('n') | KeyCode::Char('N') => {
            app.clear_messages();
            // Create a root notebook (no parent)
            app.input_mode = InputMode::CreateNotebook;
            app.input_buffer.clear();
            false
        }

        KeyCode::Char('b') | KeyCode::Char('B') => {
            app.clear_messages();
            // Create a nested notebook inside the currently selected notebook
            if let Some(TreeItem::Notebook(_, _)) = app.get_selected_item() {
                app.input_mode = InputMode::CreateNestedNotebook;
                app.input_buffer.clear();
            } else if let Some(TreeItem::Snippet(snippet_id, _)) = app.get_selected_item() {
                // If a snippet is selected, use its notebook as parent
                if let Some(snippet) = app.snippet_database.snippets.get(snippet_id) {
                    app.current_notebook_id = Some(snippet.notebook_id);
                    app.input_mode = InputMode::CreateNestedNotebook;
                    app.input_buffer.clear();
                }
            } else {
                app.set_error_message("Select a notebook first".to_string());
            }
            false
        }

        // Create new snippet (in current notebook or first available)
        KeyCode::Char('s') | KeyCode::Char('S') => {
            app.clear_messages();
            if app.snippet_database.notebooks.is_empty() {
                app.set_error_message("Create a notebook first".to_string());
            } else {
                app.input_mode = InputMode::CreateSnippet;
                app.input_buffer.clear();

                // Set notebook_id for snippet creation
                let notebook_id = get_current_notebook_id(app)
                    .unwrap_or_else(|| app.snippet_database.root_notebooks[0]);

                // Set the code_snippets_state to CreateSnippet with the proper notebook_id
                app.code_snippets_state = CodeSnippetsState::CreateSnippet { notebook_id };
            }
            false
        }

        // Delete selected item
        KeyCode::Delete | KeyCode::Char('d') | KeyCode::Char('D') => {
            app.clear_messages();
            if let Some(selected_item) = app.get_selected_item().cloned() {
                match selected_item {
                    TreeItem::Notebook(notebook_id, _) => {
                        // Show confirmation for notebook deletion
                        if let Some(notebook) = app.snippet_database.notebooks.get(&notebook_id) {
                            if notebook.snippet_count > 0 {
                                app.set_error_message(format!("Cannot delete '{}': contains {} snippets. Delete snippets first.", notebook.name, notebook.snippet_count));
                            } else {
                                match app.delete_notebook(notebook_id) {
                                    Ok(_) => {
                                        app.set_success_message(
                                            "Notebook deleted successfully!".to_string(),
                                        );
                                    }
                                    Err(e) => {
                                        app.set_error_message(e);
                                    }
                                }
                            }
                        }
                    }
                    TreeItem::Snippet(snippet_id, _) => {
                        if let Some(snippet) = app.snippet_database.snippets.get(&snippet_id) {
                            let snippet_name = snippet.title.clone();
                            match app.delete_snippet(snippet_id) {
                                Ok(_) => {
                                    app.set_success_message(format!(
                                        "Snippet '{}' deleted successfully!",
                                        snippet_name
                                    ));
                                }
                                Err(e) => {
                                    app.set_error_message(e);
                                }
                            }
                        }
                    }
                }
            } else {
                app.set_error_message("No item selected to delete".to_string());
            }
            false
        }

        // Search snippets
        KeyCode::Char('/') => {
            app.clear_messages();
            app.input_mode = InputMode::Search;
            app.input_buffer.clear();
            false
        }

        // Refresh tree view
        KeyCode::Char('r') | KeyCode::Char('R') => {
            app.clear_messages();
            app.refresh_tree_items();
            app.set_success_message("Tree view refreshed".to_string());
            false
        }

        // Toggle favorites filter
        KeyCode::Char('f') => {
            app.clear_messages();
            app.show_favorites_only = !app.show_favorites_only;
            app.refresh_tree_items();
            let status = if app.show_favorites_only { "on" } else { "off" };
            app.set_success_message(format!("Favorites filter: {}", status));
            false
        }

        // Settings
        KeyCode::Char(',') => {
            app.clear_messages();
            app.code_snippets_state = CodeSnippetsState::Settings;
            false
        }

        // Back/Escape
        KeyCode::Esc => {
            app.clear_messages();
            if app.can_go_back() {
                app.go_back();
            }
            false
        }

        // Home
        KeyCode::Char('h') | KeyCode::Char('H') => {
            app.clear_messages();
            app.page_history.clear();
            app.state = AppState::StartPage;
            false
        }

        // Clear messages manually
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.clear_messages();
            false
        }

        _ => false,
    }
}

/// Handles keys for notebook view
fn handle_notebook_view_keys(key: KeyEvent, app: &mut App, _notebook_id: uuid::Uuid) -> bool {
    match key.code {
        KeyCode::Esc => {
            app.code_snippets_state = CodeSnippetsState::NotebookList;
            app.current_notebook_id = None;
            false
        }
        // Add more notebook-specific keys here
        _ => false,
    }
}

/// Handles keys for snippet editor
fn handle_snippet_editor_keys(key: KeyEvent, app: &mut App, _snippet_id: uuid::Uuid) -> bool {
    match key.code {
        KeyCode::Esc => {
            app.code_snippets_state = CodeSnippetsState::NotebookList;
            false
        }
        // The actual editing happens in external editor
        _ => false,
    }
}

/// Handles keys for search view
fn handle_search_keys(key: KeyEvent, app: &mut App) -> bool {
    match key.code {
        KeyCode::Esc => {
            app.code_snippets_state = CodeSnippetsState::NotebookList;
            false
        }
        _ => false,
    }
}

/// Handles keys for other snippet states
fn handle_other_snippets_keys(key: KeyEvent, app: &mut App) -> bool {
    match key.code {
        KeyCode::Esc => {
            app.code_snippets_state = CodeSnippetsState::NotebookList;
            false
        }
        _ => false,
    }
}

/// Get the current notebook ID for creating snippets
fn get_current_notebook_id(app: &App) -> Option<uuid::Uuid> {
    // If we have a current notebook selected, use that
    if let Some(id) = app.current_notebook_id {
        return Some(id);
    }

    // Try to get notebook from selected tree item
    if let Some(TreeItem::Notebook(id, _)) = app.get_selected_item() {
        return Some(*id);
    }

    // If selected item is a snippet, get its notebook
    if let Some(TreeItem::Snippet(snippet_id, _)) = app.get_selected_item() {
        if let Some(snippet) = app.snippet_database.snippets.get(snippet_id) {
            return Some(snippet.notebook_id);
        }
    }

    // Fall back to first available notebook
    app.snippet_database.root_notebooks.first().copied()
}

/// Launch external editor for snippet editing
fn launch_external_editor(app: &mut App, snippet_id: uuid::Uuid) {
    // Set flag to indicate a full UI redraw will be needed after editor use
    app.needs_redraw = true;

    if let Some(snippet) = app.snippet_database.snippets.get(&snippet_id) {
        if let Some(ref storage) = app.storage_manager {
            let file_path = storage.get_snippet_file_path(snippet);

            // Ensure the file exists with current content
            if let Err(e) = storage.save_snippet_content(snippet) {
                app.set_error_message(format!("Failed to prepare file for editing: {}", e));
                return;
            }

            // Properly suspend the current TUI application
            if let Err(e) = suspend_tui_for_editor(&file_path) {
                app.set_error_message(format!("Failed to launch editor: {}", e));
                return;
            }

            // Reload snippet content after editing
            if let Ok(content) = storage.load_snippet_content(
                snippet.id,
                snippet.notebook_id,
                &snippet.file_extension,
            ) {
                if let Some(snippet) = app.snippet_database.snippets.get_mut(&snippet_id) {
                    snippet.update_content(content);

                    // Save changes
                    if let Err(e) = storage.save_snippet_content(snippet) {
                        app.set_error_message(format!("Failed to save snippet: {}", e));
                    } else {
                        if let Err(e) = app.save_database() {
                            app.set_error_message(format!("Failed to save database: {}", e));
                        } else {
                            app.set_success_message("Snippet saved successfully!".to_string());

                            // Ensure we're back in the snippets list view
                            app.code_snippets_state = CodeSnippetsState::NotebookList;
                            app.refresh_tree_items();
                        }
                    }
                }
            }
        }
    }
}

/// Properly suspend TUI and launch external editor
fn suspend_tui_for_editor(file_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    use ratatui::crossterm::{
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    };
    use std::io::{Write, stdout};
    use std::process::Command;

    // First completely exit the terminal UI
    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen)?;

    // Clear the terminal completely using the most thorough approach
    print!("\x1B[!p"); // Soft reset (DEC)
    print!("\x1B[3J"); // Clear scrollback buffer
    print!("\x1B[2J"); // Clear entire screen
    print!("\x1B[H"); // Move cursor to home position
    print!("\x1B[?25h"); // Show cursor
    stdout().flush()?;

    // Try to launch editors in order of preference
    let editors = ["nvim", "vim", "nano"];
    let mut editor_launched = false;

    for editor in &editors {
        if let Ok(mut child) = Command::new(editor).arg(file_path).spawn() {
            // Wait for editor to close
            if let Ok(_) = child.wait() {
                editor_launched = true;
                break;
            }
        }
    }

    if !editor_launched {
        println!("Could not launch any editor (nvim, vim, nano)");
        println!("Press Enter to continue...");
        let mut buffer = String::new();
        std::io::stdin().read_line(&mut buffer)?;
        return Err("Could not launch any editor".into());
    }

    // Give a visual signal that we're returning to the application
    println!("\nReturning to snix...");
    stdout().flush()?;
    std::thread::sleep(std::time::Duration::from_millis(300));

    // Execute a full reset sequence for the terminal
    print!("\x1Bc"); // Full terminal reset
    print!("\x1B[!p"); // Soft reset (DEC)
    print!("\x1B[3J"); // Clear scrollback buffer
    print!("\x1B[2J"); // Clear entire screen
    print!("\x1B[H"); // Move cursor to home position
    stdout().flush()?;

    // Restore the terminal UI state
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;

    // Final screen initialization
    print!("\x1B[?1049h"); // Ensure alternate screen buffer is active
    print!("\x1B[?25l"); // Hide cursor
    print!("\x1B[2J"); // Clear screen
    print!("\x1B[H"); // Move cursor to home
    stdout().flush()?;

    Ok(())
}

/// Handles keyboard input specifically for the start page (main menu)
///
/// This function processes all keyboard interactions when the user is on the main
/// start page. It handles menu navigation (up/down movement), item selection,
/// and provides convenient single-letter shortcuts for quick navigation to
/// specific sections of the application.
fn handle_start_page_keys(key: KeyEvent, app: &mut App) -> bool {
    // Clear any previous messages
    app.clear_messages();

    match key.code {
        // Menu navigation - move selection up (vi-style 'k' and arrow key)
        KeyCode::Up | KeyCode::Char('k') => {
            app.previous_menu_item();
            false
        }

        // Menu navigation - move selection down (vi-style 'j' and arrow key)
        KeyCode::Down | KeyCode::Char('j') => {
            app.next_menu_item();
            false
        }

        // Activate the currently selected menu item
        KeyCode::Enter => {
            match app.selected_menu_item {
                // Navigate to Boilerplates page
                0 => {
                    app.navigate_to(AppState::Boilerplates);
                    false
                }

                // Navigate to Marketplace page
                1 => {
                    app.navigate_to(AppState::Marketplace);
                    false
                }

                // Navigate to Code Snippets page
                2 => {
                    app.navigate_to(AppState::CodeSnippets);
                    false
                }

                // Navigate to Info/About page
                3 => {
                    app.navigate_to(AppState::InfoPage);
                    false
                }

                // Navigate to Settings page
                4 => {
                    app.navigate_to(AppState::Settings);
                    false
                }

                // Exit application (last menu item)
                5 => true,

                // Safety fallback for any invalid menu indices
                _ => false,
            }
        }

        // Direct navigation shortcuts - Boilerplates (both cases supported)
        KeyCode::Char('b') | KeyCode::Char('B') => {
            app.navigate_to(AppState::Boilerplates);
            false
        }

        // Direct navigation shortcuts - Marketplace (both cases supported)
        KeyCode::Char('m') | KeyCode::Char('M') => {
            app.navigate_to(AppState::Marketplace);
            false
        }

        // Direct navigation shortcuts - Code Snippets (both cases supported)
        KeyCode::Char('s') | KeyCode::Char('S') => {
            app.navigate_to(AppState::CodeSnippets);
            false
        }

        // Direct navigation shortcuts - Info/About page (both cases supported)
        KeyCode::Char('i') | KeyCode::Char('I') => {
            app.navigate_to(AppState::InfoPage);
            false
        }

        // Direct navigation shortcuts - Settings (both cases supported)
        KeyCode::Char('c') | KeyCode::Char('C') => {
            app.navigate_to(AppState::Settings);
            false
        }

        // Ignore all other key presses on the start page
        _ => false,
    }
}

/// Handles keyboard input for all non-start pages (WIP dialogs and future pages)
///
/// This function processes keyboard interactions when the user is on any page other
/// than the start page. Currently, all non-start pages show work-in-progress dialogs,
/// so this handler primarily focuses on navigation commands to return to previous
/// pages or the home page.
fn handle_other_page_keys(key: KeyEvent, app: &mut App) -> bool {
    // Dismiss any messages with Enter key
    if key.code == KeyCode::Enter && (app.error_message.is_some() || app.success_message.is_some())
    {
        app.clear_messages();
        return false;
    }

    match key.code {
        // Standard back navigation - uses the navigation history stack
        KeyCode::Esc => {
            // Clear messages if any
            if app.error_message.is_some() || app.success_message.is_some() {
                app.clear_messages();
            } else if app.can_go_back() {
                app.go_back();
            }
            false
        }

        // Emergency home navigation - clears all history and returns to start page
        // This provides a quick way to get back to the main menu from any nested page
        KeyCode::Char('h') | KeyCode::Char('H') => {
            app.page_history.clear();
            app.state = AppState::StartPage;
            false
        }

        // Ignore all other key presses on non-start pages
        _ => false,
    }
}

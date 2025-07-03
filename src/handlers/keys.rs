use crate::app::{App, AppState, CodeSnippetsState, InputMode, RecentSearchEntry, TreeItem};
use crate::handlers::ollama;
use crate::models::SnippetLanguage;
use crate::models::export::ExportFormat;
use crate::ui::backup_restore;
use crate::ui::colors::RosePine;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

struct NavigationHandler;

impl NavigationHandler {
    /// Handle tab navigation (left/right cycling)
    fn handle_tab_navigation(key: KeyEvent, current_tab: usize, max_tab: usize) -> Option<usize> {
        match key.code {
            KeyCode::Left | KeyCode::BackTab => Some(if current_tab == 0 {
                max_tab
            } else {
                current_tab - 1
            }),
            KeyCode::Right | KeyCode::Tab => Some((current_tab + 1) % (max_tab + 1)),
            _ => None,
        }
    }
}

/// Common search functionality
struct SearchHandler;

impl SearchHandler {
    /// Activate search mode with common setup
    fn activate_search_mode(app: &mut App) {
        app.clear_messages();
        app.input_mode = InputMode::Search;
        app.search_query.clear();
        app.input_buffer.clear();
        app.search_results.clear();
        app.selected_search_result = 0;
        app.selected_recent_search = 0;
        app.needs_redraw = true;
        app.set_success_message("Search mode activated. Type to search...".to_string());
    }

    /// Handle search result navigation
    fn handle_search_navigation(key: KeyEvent, app: &mut App) -> bool {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if !app.search_results.is_empty() {
                    app.selected_search_result = if app.selected_search_result > 0 {
                        app.selected_search_result - 1
                    } else {
                        app.search_results.len() - 1
                    };
                    app.set_success_message(format!(
                        "Selected result {}/{}",
                        app.selected_search_result + 1,
                        app.search_results.len()
                    ));
                } else if !app.recent_searches.is_empty() && app.search_query.is_empty() {
                    app.selected_recent_search = if app.selected_recent_search > 0 {
                        app.selected_recent_search - 1
                    } else {
                        app.recent_searches.len() - 1
                    };
                }
                app.needs_redraw = true;
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !app.search_results.is_empty() {
                    app.selected_search_result =
                        (app.selected_search_result + 1) % app.search_results.len();
                    app.set_success_message(format!(
                        "Selected result {}/{}",
                        app.selected_search_result + 1,
                        app.search_results.len()
                    ));
                } else if !app.recent_searches.is_empty() && app.search_query.is_empty() {
                    app.selected_recent_search =
                        (app.selected_recent_search + 1) % app.recent_searches.len();
                }
                app.needs_redraw = true;
                true
            }
            _ => false,
        }
    }

    /// Handle search query input and execution
    fn handle_search_input(key: KeyEvent, app: &mut App) -> bool {
        match key.code {
            KeyCode::Char(c) => {
                app.search_query.push(c);
                let query = app.search_query.clone();
                let count = app.perform_search(&query);
                app.set_success_message(format!("Found {} results for '{}'", count, query));
                app.needs_redraw = true;
                true
            }
            KeyCode::Backspace => {
                if !app.search_query.is_empty() {
                    app.search_query.pop();
                    if app.search_query.is_empty() {
                        app.search_results.clear();
                        app.selected_search_result = 0;
                        app.set_success_message("Type to search".to_string());
                    } else {
                        let query = app.search_query.clone();
                        let count = app.perform_search(&query);
                        app.set_success_message(format!("Found {} results for '{}'", count, query));
                    }
                }
                app.needs_redraw = true;
                true
            }
            _ => false,
        }
    }

    /// Close search mode and save recent search if applicable
    fn close_search_mode(app: &mut App) {
        app.input_mode = InputMode::Normal;

        // Save to recent searches only if not empty
        if !app.search_query.is_empty() {
            let query = app.search_query.clone();
            let result_count = app.search_results.len();
            if !app.recent_searches.iter().any(|entry| entry.query == query) {
                let entry = RecentSearchEntry::new(query, result_count);
                app.recent_searches.insert(0, entry);
                // Limit to 10 recent searches
                if app.recent_searches.len() > 10 {
                    app.recent_searches.pop();
                }
            }
        }

        app.search_query.clear();
        app.search_results.clear();
        app.selected_search_result = 0;
        app.selected_recent_search = 0;
        app.clear_messages();
    }
}

/// Common Ollama chat functionality
struct OllamaHandler;

impl OllamaHandler {
    /// Initialize and open Ollama chat for a snippet
    fn open_snippet_chat(app: &mut App, snippet_id: uuid::Uuid) -> bool {
        if let Some(snippet) = app.snippet_database.snippets.get(&snippet_id) {
            // Initialize Ollama state if needed
            if app.ollama_state.is_none() {
                app.ollama_state = Some(crate::ui::ollama::OllamaState::new());
            }

            if let Some(ollama_state) = &mut app.ollama_state {
                // Generate snippet hash to check for existing sessions
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};

                let mut hasher = DefaultHasher::new();
                snippet.content.hash(&mut hasher);
                let snippet_hash = format!("{:x}", hasher.finish());

                // Check for existing sessions with this snippet
                let mut existing_sessions: Vec<_> = ollama_state
                    .saved_sessions
                    .iter()
                    .filter(|session| session.snippet_hash.as_ref() == Some(&snippet_hash))
                    .collect();

                // Sort by updated_at (most recent first)
                existing_sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

                // Set current snippet for context
                ollama_state.current_snippet = Some(snippet.content.clone());

                if let Some(most_recent_session) = existing_sessions.first() {
                    let session_to_load = (*most_recent_session).clone();

                    ollama_state.conversation = session_to_load.conversation.clone();
                    ollama_state.current_session = Some(session_to_load.clone());
                    ollama_state.system_prompt = session_to_load.system_prompt.clone();
                    ollama_state.system_prompt_buffer = session_to_load.system_prompt.clone();
                    ollama_state.unsaved_changes = false;

                    // Set the last assistant response for copy functionality
                    ollama_state.last_assistant_response = ollama_state
                        .conversation
                        .iter()
                        .rev()
                        .find(|msg| msg.role == crate::ui::ollama::ChatRole::Assistant)
                        .map(|msg| msg.content.clone());

                    // Switch to chat panel and scroll to bottom
                    ollama_state.active_panel = crate::ui::ollama::ActivePanel::CurrentChat;
                    ollama_state.scroll_position = usize::MAX;

                    ollama_state.add_success_toast(format!(
                        "Loaded existing chat: {} ({} messages)",
                        session_to_load.title,
                        session_to_load.get_message_count()
                    ));
                } else {
                    // No existing sessions - create new chat
                    ollama_state.conversation.clear();
                    ollama_state.scroll_position = 0;
                    ollama_state.unsaved_changes = false;
                    ollama_state.last_assistant_response = None;

                    // Create enhanced system prompt with snippet context
                    let enhanced_system_prompt = ollama::create_snippet_system_prompt(
                        &snippet.language.to_string(),
                        &snippet.title,
                        &snippet.content,
                    );

                    ollama_state.system_prompt = enhanced_system_prompt;

                    let snippet_info = format!(
                        " Working with {} snippet: '{}'",
                        snippet.language.to_string(),
                        snippet.title
                    );
                    ollama_state.add_message(crate::ui::ollama::ChatRole::System, snippet_info);

                    ollama_state.add_success_toast(format!(
                        "Started new chat for snippet: {}",
                        snippet.title
                    ));
                }

                // Common setup for both new and existing chats
                ollama_state.show_popup = true;
                ollama_state.loading_models = true;
                ollama_state.error_message = None;

                // Clear models and reload them (but don't clear sessions)
                ollama_state.models.clear();
                ollama_state.selected_model_index = 0;

                if let Err(e) = ollama::fetch_ollama_models(app) {
                    if let Some(ollama_state) = &mut app.ollama_state {
                        ollama_state
                            .add_error_toast(format!("Failed to fetch Ollama models: {}", e));
                    }
                } else {
                    // Ensure sessions are always refreshed after model loading
                    if let Some(ollama_state) = &mut app.ollama_state {
                        if let Some(storage) = &ollama_state.chat_storage {
                            if let Ok(sessions) = storage.load_all_sessions() {
                                ollama_state.saved_sessions = sessions;
                            }
                        }
                    }
                    return true;
                }
            }
            return true;
        }
        app.set_error_message("Select a snippet first".to_string());
        false
    }
}

/// Common input handling patterns
struct InputHandler;

impl InputHandler {
    /// Handle common escape behavior across input modes
    fn handle_escape(app: &mut App, clear_input: bool) {
        app.input_mode = InputMode::Normal;
        if clear_input {
            app.input_buffer.clear();
            app.pending_snippet_title.clear();
        }
        app.clear_messages();
    }

    /// Handle selection navigation for language/color picking
    fn handle_selection_navigation(key: KeyEvent, current: &mut usize, max_items: usize) -> bool {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                *current = if *current == 0 {
                    max_items - 1
                } else {
                    *current - 1
                };
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                *current = (*current + 1) % max_items;
                true
            }
            _ => false,
        }
    }
}

/// Common clipboard operations
struct ClipboardHandler;

impl ClipboardHandler {
    /// Copy text to clipboard using available utilities
    fn copy_to_clipboard(content: &str) -> bool {
        let commands = [
            ("xclip", vec!["-selection", "clipboard"]),
            ("wl-copy", vec![]),
            ("termux-clipboard-set", vec![]),
        ];

        for (cmd, args) in &commands {
            if let Ok(mut process) = Command::new(cmd).args(args).stdin(Stdio::piped()).spawn() {
                if let Some(stdin) = process.stdin.as_mut() {
                    if stdin.write_all(content.as_bytes()).is_ok() {
                        return true;
                    }
                }
            }
        }
        false
    }
}

/// Language detection and parsing utilities
struct LanguageDetector;

impl LanguageDetector {
    /// Parse title and language from input string with file extension
    fn parse_title_and_language(input: &str) -> (String, SnippetLanguage) {
        if input.contains('.') {
            let parts: Vec<&str> = input.rsplitn(2, '.').collect();
            let extension = parts[0].to_lowercase();
            let title = parts[1].to_string();
            let language = Self::language_from_extension(&extension);
            (title, language)
        } else {
            (input.to_string(), SnippetLanguage::Text)
        }
    }

    /// Map file extension to SnippetLanguage
    fn language_from_extension(extension: &str) -> SnippetLanguage {
        match extension {
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
        }
    }
}

/// Main keyboard event handler and dispatcher
pub fn handle_key_events(key: KeyEvent, app: &mut App) -> bool {
    // Handle Ollama popup if it's active
    if let Some(ollama_state) = &app.ollama_state {
        if ollama_state.show_popup {
            match ollama::handle_ollama_input(app, key) {
                Ok(_) => return false,
                Err(_) => return false,
            }
        }
    }

    // Handle special input modes first
    if app.input_mode == InputMode::SelectNotebookColor {
        return handle_notebook_color_selection(key, app);
    }

    // Handle other input modes
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
            app.input_mode = if app.input_mode == InputMode::HelpMenu {
                InputMode::Normal
            } else {
                InputMode::HelpMenu
            };
            false
        }

        // Global back navigation
        KeyCode::Backspace => {
            // Don't trigger back navigation when in import path popup
            if let (AppState::ExportImport, Some(export_state)) =
                (&app.state, &app.export_import_state)
            {
                if export_state.mode == crate::ui::export_import::ExportImportMode::ImportPathPopup
                {
                    return handle_export_import_keys(key, app);
                }
            }

            if app.can_go_back() {
                app.go_back();
            }
            false
        }

        // Route to state-specific key handlers
        _ => match app.state {
            AppState::StartPage => handle_start_page_keys(key, app),
            AppState::CodeSnippets => handle_code_snippets_keys(key, app),
            AppState::ExportImport => handle_export_import_keys(key, app),
            _ => handle_other_page_keys(key, app),
        },
    }
}

/// Handle notebook color selection input mode
fn handle_notebook_color_selection(key: KeyEvent, app: &mut App) -> bool {
    match key.code {
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
            app.clear_messages();
            false
        }
        KeyCode::Up | KeyCode::Down | KeyCode::Char('k') | KeyCode::Char('j') => {
            let colors = get_available_colors();
            if InputHandler::handle_selection_navigation(
                key,
                &mut app.selected_language,
                colors.len(),
            ) {
                // Color selection changed, no message needed
            }
            false
        }
        KeyCode::Enter => {
            if let Some(notebook_id) = app.current_notebook_id {
                match app.update_notebook_color(notebook_id, app.selected_language) {
                    Ok(_) => {
                        app.set_success_message("Notebook color updated successfully".to_string());
                    }
                    Err(e) => {
                        app.set_error_message(e);
                    }
                }
            } else {
                app.set_error_message("No notebook selected".to_string());
            }
            app.input_mode = InputMode::Normal;
            false
        }
        _ => false,
    }
}

/// Handles keyboard input for input mode in code snippets
fn handle_input_mode_keys(key: KeyEvent, app: &mut App) -> bool {
    // Special case for search mode - direct character input to search query
    if app.input_mode == InputMode::Search {
        match key.code {
            KeyCode::Esc => {
                SearchHandler::close_search_mode(app);
                app.set_success_message("Search closed".to_string());
                false
            }
            KeyCode::Enter => {
                // In search mode, Enter activates the selected result
                if !app.search_results.is_empty() {
                    app.open_selected_search_result();
                    app.input_mode = InputMode::Normal;
                    app.search_query.clear();
                } else {
                    app.set_success_message("No results to select".to_string());
                }
                false
            }
            _ => {
                // Try navigation first, then input
                if SearchHandler::handle_search_navigation(key, app) {
                    false
                } else if SearchHandler::handle_search_input(key, app) {
                    false
                } else {
                    false
                }
            }
        }
    } else {
        // Regular input mode handling for other modes
        match key.code {
            KeyCode::Esc => {
                InputHandler::handle_escape(app, true);
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
                            // Capture the parent notebook ID temporarily for this operation
                            let parent_id = if app.current_notebook_id.is_none() {
                                if let Some(TreeItem::Notebook(id, _)) = app.get_selected_item() {
                                    Some(*id)
                                } else if let Some(TreeItem::Snippet(snippet_id, _)) =
                                    app.get_selected_item()
                                {
                                    // If a snippet is selected, use its notebook as parent
                                    if let Some(snippet) =
                                        app.snippet_database.snippets.get(snippet_id)
                                    {
                                        Some(snippet.notebook_id)
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                app.current_notebook_id
                            };

                            app.current_notebook_id = parent_id;
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

                            app.current_notebook_id = None;
                        }
                        app.input_mode = InputMode::Normal;
                    }
                    InputMode::CreateSnippet => {
                        if !input.is_empty() {
                            let (title, language) =
                                LanguageDetector::parse_title_and_language(&input);

                            if let Some(notebook_id) = get_current_notebook_id(app) {
                                match app.create_snippet(title, language, notebook_id) {
                                    Ok(_snippet_id) => {
                                        app.set_success_message(
                                            "Snippet created successfully!".to_string(),
                                        );
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

                        app.code_snippets_state = CodeSnippetsState::NotebookList;
                    }
                    InputMode::Search => {
                        // When Enter is pressed in search input mode, treat it as confirmation
                        if input.is_empty() {
                            // Empty search - go back to normal mode
                            app.input_mode = InputMode::Normal;
                            app.search_query.clear();
                            app.search_results.clear();
                            app.selected_search_result = 0;
                            app.set_success_message("Search canceled".to_string());
                            return false;
                        }

                        // Set the search query from input buffer
                        app.search_query = input;

                        // Perform the search with the entered query
                        let query = app.search_query.clone();
                        let count = app.perform_search(&query);

                        if count == 0 {
                            app.set_success_message(format!(
                                "No results found for '{}'",
                                app.search_query
                            ));
                        } else {
                            app.set_success_message(format!(
                                "Found {} results for '{}'. Use ↑/↓ to navigate.",
                                count, app.search_query
                            ));
                        }

                        // Redirect to handle_search_keys for future key presses
                        // but stay in Search mode to allow navigation
                    }
                    InputMode::EditSnippetDescription => {
                        if let Some(TreeItem::Snippet(snippet_id, _)) = app.get_selected_item() {
                            match app.update_snippet_description(*snippet_id, input) {
                                Ok(_) => {
                                    app.set_success_message(
                                        "Description updated successfully".to_string(),
                                    );
                                }
                                Err(e) => {
                                    app.set_error_message(e);
                                }
                            }
                        } else {
                            app.set_error_message("Snippet selection lost".to_string());
                        }
                        app.input_mode = InputMode::Normal;
                        app.pending_snippet_title.clear();
                    }
                    InputMode::EditNotebookDescription => {
                        if let Some(notebook_id) = app.current_notebook_id {
                            match app.update_notebook_description(notebook_id, input) {
                                Ok(_) => {
                                    app.set_success_message(
                                        "Notebook description updated successfully".to_string(),
                                    );
                                }
                                Err(e) => {
                                    app.set_error_message(e);
                                }
                            }
                        } else {
                            app.set_error_message("No notebook selected".to_string());
                        }
                        app.input_mode = InputMode::Normal;
                    }
                    InputMode::SelectNotebookColor => {
                        app.input_mode = InputMode::Normal;
                    }
                    InputMode::EditTags => {
                        // Handle Enter to save tags
                        let input_text = input.clone();

                        // First get the snippet ID without borrowing
                        let snippet_id_opt =
                            if let Some(TreeItem::Snippet(id, _)) = app.get_selected_item() {
                                Some(*id)
                            } else {
                                None
                            };

                        // Now process with the ID without holding a borrow across function calls
                        if let Some(snippet_id) = snippet_id_opt {
                            // First update the snippet
                            if let Some(snippet) =
                                app.snippet_database.snippets.get_mut(&snippet_id)
                            {
                                // Clear existing tags before setting new ones
                                snippet.tags.clear();
                                snippet.set_tags_from_text(&input_text);

                                // Get a copy of the tags to avoid borrowing issues
                                let tags: Vec<String> = snippet.tags.clone();

                                // Now update the tag manager with the cloned tags
                                for tag_name in tags {
                                    app.tag_manager.add_tag_to_snippet(snippet_id, tag_name);
                                }

                                app.set_success_message("Tags updated".to_string());
                            } else {
                                app.set_error_message("Snippet not found".to_string());
                            }
                        }

                        // Always return to normal mode even if no snippet was found
                        app.input_mode = InputMode::Normal;
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
            KeyCode::Up | KeyCode::Down | KeyCode::Char('k') | KeyCode::Char('j')
                if app.input_mode == InputMode::SelectLanguage =>
            {
                let languages = get_available_languages();
                InputHandler::handle_selection_navigation(
                    key,
                    &mut app.selected_language,
                    languages.len(),
                );
                false
            }
            KeyCode::Char(c) => {
                if app.input_mode != InputMode::SelectLanguage
                    && app.input_mode != InputMode::SelectNotebookColor
                {
                    app.input_buffer.push(c);
                }
                false
            }
            _ => false,
        }
    }
}

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

// Handles keyboard input specifically for the code snippets page
fn handle_code_snippets_keys(key: KeyEvent, app: &mut App) -> bool {
    match app.code_snippets_state {
        CodeSnippetsState::NotebookList => handle_notebook_list_keys(key, app),
        CodeSnippetsState::NotebookView { notebook_id } => {
            handle_notebook_view_keys(key, app, notebook_id)
        }
        CodeSnippetsState::NotebookDetails { notebook_id } => {
            handle_notebook_details_keys(key, app, notebook_id)
        }
        CodeSnippetsState::_SnippetEditor { snippet_id } => {
            handle_snippet_editor_keys(key, app, snippet_id)
        }
        CodeSnippetsState::SearchSnippets => handle_search_keys(key, app),
        _ => handle_other_snippets_keys(key, app),
    }
}

// Handles keys for the main notebook list view
fn handle_notebook_list_keys(key: KeyEvent, app: &mut App) -> bool {
    if app.input_mode == InputMode::Search {
        return handle_search_keys(key, app);
    }

    if app.show_favorites_popup && key.code == KeyCode::Esc {
        app.show_favorites_popup = false;
        return false;
    }

    if app.has_pending_action() {
        match key.code {
            KeyCode::Enter => {
                app.confirm_pending_action();
                return false;
            }
            KeyCode::Esc => {
                app.cancel_pending_action();
                return false;
            }
            _ => return false,
        }
    }

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
        // Handle Shift + Up for moving notebook up in hierarchy
        KeyCode::Up if key.modifiers.contains(KeyModifiers::SHIFT) => {
            app.clear_messages();
            if app.move_notebook_up() {
                app.needs_redraw = true;
            }
            false
        }

        // Handle Shift + Down for moving notebook down in hierarchy
        KeyCode::Down if key.modifiers.contains(KeyModifiers::SHIFT) => {
            app.clear_messages();
            if app.move_notebook_down() {
                app.needs_redraw = true;
            }
            false
        }

        // Normal navigation
        KeyCode::Up | KeyCode::Char('k') => {
            app.previous_tree_item();
            app.reset_scroll_position();
            false
        }

        KeyCode::Down | KeyCode::Char('j') => {
            app.next_tree_item();
            app.reset_scroll_position();
            false
        }

        // Add Page Up and Page Down for scrolling content
        KeyCode::PageUp => {
            app.content_scroll_position = app.content_scroll_position.saturating_sub(5);
            app.needs_redraw = true;
            false
        }

        KeyCode::PageDown => {
            app.content_scroll_position = app.content_scroll_position.saturating_add(5);
            app.needs_redraw = true;
            false
        }

        // Enter selected item (notebook or snippet)
        KeyCode::Enter => {
            if app.error_message.is_some() || app.success_message.is_some() {
                app.clear_messages();
                return false;
            }

            if key.modifiers.contains(KeyModifiers::SHIFT) {
                // Special handler for Shift+Enter
                if let Some(TreeItem::Notebook(notebook_id, _)) = app.get_selected_item().cloned() {
                    app.current_notebook_id = Some(notebook_id);
                    // Use NotebookView when Shift+Enter is pressed, for classic view
                    app.code_snippets_state = CodeSnippetsState::NotebookView { notebook_id };
                    return false;
                }
            }

            if let Some(selected_item) = app.get_selected_item().cloned() {
                match selected_item {
                    TreeItem::Notebook(notebook_id, _) => {
                        app.current_notebook_id = Some(notebook_id);

                        app.code_snippets_state =
                            CodeSnippetsState::NotebookDetails { notebook_id };
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

        // Activate search mode with '/'
        KeyCode::Char('/') => {
            SearchHandler::activate_search_mode(app);
            false
        }

        KeyCode::Char('n') | KeyCode::Char('N') => {
            app.clear_messages();
            // Create a root notebook (no parent)
            app.current_notebook_id = None;
            // Temporarily store the current hovered state
            let prev_hovered = app.hovered_tree_item;
            // Reset hover state to avoid it influencing notebook creation
            app.hovered_tree_item = None;

            app.input_mode = InputMode::CreateNotebook;
            app.input_buffer.clear();

            // Restore hovered state after setting up the notebook creation
            app.hovered_tree_item = prev_hovered;
            false
        }

        KeyCode::Char('b') | KeyCode::Char('B') => {
            app.clear_messages();
            // Create a nested notebook inside the currently selected notebook
            let has_parent = if let Some(TreeItem::Notebook(id, _)) = app.get_selected_item() {
                // Temporarily store the parent notebook ID
                app.current_notebook_id = Some(*id);
                true
            } else if let Some(TreeItem::Snippet(snippet_id, _)) = app.get_selected_item() {
                // If a snippet is selected, use its notebook as parent
                if let Some(snippet) = app.snippet_database.snippets.get(snippet_id) {
                    app.current_notebook_id = Some(snippet.notebook_id);
                    true
                } else {
                    false
                }
            } else {
                false
            };

            if has_parent {
                app.input_mode = InputMode::CreateNestedNotebook;
                app.input_buffer.clear();
            } else {
                app.set_error_message("Select a notebook first".to_string());
            }
            false
        }

        // Toggle collapse/expand notebook with space key
        KeyCode::Char(' ') => {
            app.clear_messages();
            if app.toggle_notebook_collapse() {
                app.needs_redraw = true;
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

        // Delete selected item (notebook or snippet)
        KeyCode::Char('x') | KeyCode::Char('X') => {
            app.clear_messages();
            if let Some(selected_item) = app.get_selected_item().cloned() {
                match selected_item {
                    TreeItem::Notebook(notebook_id, _) => {
                        // Check if this notebook has snippets or nested notebooks
                        let has_snippets = app
                            .snippet_database
                            .snippets
                            .values()
                            .any(|s| s.notebook_id == notebook_id);

                        let has_children = app
                            .snippet_database
                            .notebooks
                            .values()
                            .any(|n| n.parent_id == Some(notebook_id));

                        if has_snippets || has_children {
                            app.set_error_message(
                                "Cannot delete a notebook that contains snippets or other notebooks"
                                    .to_string(),
                            );
                            return false;
                        }

                        // Request confirmation for deletion
                        app.request_delete_confirmation(notebook_id, true);
                    }
                    TreeItem::Snippet(snippet_id, _) => {
                        // Request confirmation for deletion
                        app.request_delete_confirmation(snippet_id, false);
                    }
                }
            } else {
                app.set_error_message("No item selected".to_string());
            }
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
        KeyCode::Char('f')
            if !app.has_pending_action()
                && app.input_mode == InputMode::Normal
                && app.get_selected_item().is_none() =>
        {
            app.clear_messages();
            app.show_favorites_only = !app.show_favorites_only;
            app.refresh_tree_items();
            let status = if app.show_favorites_only { "on" } else { "off" };
            app.set_success_message(format!("Favorites filter: {}", status));
            false
        }

        // Show floating favorites window with Shift+F
        KeyCode::Char('F') | KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::SHIFT) => {
            app.show_favorites_popup = !app.show_favorites_popup;
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

        // Copy snippet to clipboard
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            app.clear_messages();
            if let Some(TreeItem::Snippet(snippet_id, _)) = app.get_selected_item() {
                if let Some(snippet) = app.snippet_database.snippets.get(snippet_id) {
                    if ClipboardHandler::copy_to_clipboard(&snippet.content) {
                        app.set_success_message(format!("'{}' copied to clipboard", snippet.title));
                    } else {
                        app.set_error_message("Failed to copy to clipboard (xclip, wl-copy, or termux-clipboard-set required)".to_string());
                    }
                }
            } else {
                app.set_error_message("No snippet selected".to_string());
            }
            false
        }

        // Edit snippet description
        KeyCode::Char('d') | KeyCode::Char('D') => {
            app.clear_messages();
            if let Some(TreeItem::Snippet(snippet_id, _)) = app.get_selected_item() {
                if let Some(snippet) = app.snippet_database.snippets.get(snippet_id) {
                    app.input_mode = InputMode::EditSnippetDescription;
                    app.current_notebook_id = Some(snippet.notebook_id);
                    app.input_buffer = snippet.description.clone().unwrap_or_default();
                    app.pending_snippet_title = snippet.title.clone();
                } else {
                    app.set_error_message("Snippet not found".to_string());
                }
            } else {
                app.set_error_message("Select a snippet first".to_string());
            }
            false
        }

        // View notebook details with 'v' key
        KeyCode::Char('v') | KeyCode::Char('V') => {
            app.clear_messages();
            if let Some(TreeItem::Notebook(notebook_id, _)) = app.get_selected_item().cloned() {
                app.current_notebook_id = Some(notebook_id);
                app.code_snippets_state = CodeSnippetsState::NotebookDetails { notebook_id };
                app.selected_details_tab = 0; // Reset to overview tab
            } else {
                app.set_error_message("Select a notebook first".to_string());
            }
            false
        }

        // Move item to next sibling (Shift+Right)
        KeyCode::Right if key.modifiers.contains(KeyModifiers::SHIFT) => {
            app.clear_messages();
            if app.move_item_to_next_sibling() {
                app.needs_redraw = true;
            }
            false
        }

        // Move item to previous sibling (Shift+Left)
        KeyCode::Left if key.modifiers.contains(KeyModifiers::SHIFT) => {
            app.clear_messages();
            if app.move_item_to_prev_sibling() {
                app.needs_redraw = true;
            }
            false
        }

        // Set tag editing mode
        KeyCode::Char('t') => {
            app.clear_messages();

            if let Some(TreeItem::Snippet(snippet_id, _)) = app.get_selected_item() {
                if let Some(snippet) = app.snippet_database.snippets.get(snippet_id) {
                    // Set input buffer to current tags
                    app.input_buffer = snippet.get_tags_display_string();
                    app.input_mode = InputMode::EditTags;
                    // Clear any messages to ensure the full tag editing UI is visible
                    app.clear_messages();
                } else {
                    app.set_error_message("Snippet not found".to_string());
                }
            } else {
                app.set_error_message("Select a snippet first".to_string());
            }
            false
        }

        // Toggle favorite status for the current snippet
        KeyCode::Char('f') => {
            if app.input_mode == InputMode::Normal {
                if let Some(TreeItem::Snippet(snippet_id, _)) = app.get_selected_item() {
                    if let Err(e) = app.toggle_favorite_snippet(*snippet_id) {
                        app.set_error_message(e);
                    }
                } else {
                    app.set_error_message("Select a snippet to mark as favorite".to_string());
                }
            }
            false
        }

        // Open snippet in Ollama chat
        KeyCode::Char('l') => {
            app.clear_messages();
            if let Some(TreeItem::Snippet(snippet_id, _)) = app.get_selected_item() {
                OllamaHandler::open_snippet_chat(app, *snippet_id);
            } else {
                app.set_error_message("Select a snippet first".to_string());
            }
            false
        }

        _ => false,
    }
}

/// Handles keys for notebook view
fn handle_notebook_view_keys(key: KeyEvent, app: &mut App, _notebook_id: uuid::Uuid) -> bool {
    // If search mode is active, handle search keys
    if app.input_mode == InputMode::Search {
        return handle_search_keys(key, app);
    }

    match key.code {
        KeyCode::Esc => {
            app.code_snippets_state = CodeSnippetsState::NotebookList;
            false
        }

        // Activate search mode with '/'
        KeyCode::Char('/') => {
            SearchHandler::activate_search_mode(app);
            false
        }

        // Open snippet in Ollama chat
        KeyCode::Char('l') => {
            if let Some(TreeItem::Snippet(snippet_id, _)) = app.get_selected_item() {
                OllamaHandler::open_snippet_chat(app, *snippet_id);
            } else {
                app.set_error_message("Select a snippet first".to_string());
            }
            false
        }

        _ => handle_notebook_list_keys(key, app),
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
            SearchHandler::close_search_mode(app);
            false
        }

        // Navigation of search results - simplified to always work
        KeyCode::Up | KeyCode::Down | KeyCode::Char('k') | KeyCode::Char('j') => {
            SearchHandler::handle_search_navigation(key, app);
            false
        }

        // Open selected result or use selected recent search
        KeyCode::Enter => {
            if !app.search_results.is_empty() {
                if app.open_selected_search_result() {
                    // If successful, the application state may have changed
                    app.input_mode = InputMode::Normal;

                    // Clear search state
                    app.search_query.clear();
                    app.search_results.clear();
                    app.selected_search_result = 0;
                    app.selected_recent_search = 0;
                } else {
                    app.set_success_message("Failed to open selected result".to_string());
                }
            } else if !app.recent_searches.is_empty() && app.search_query.is_empty() {
                // Use selected recent search
                if let Some(entry) = app.recent_searches.get(app.selected_recent_search) {
                    app.search_query = entry.query.clone();
                    let query = app.search_query.clone();
                    let count = app.perform_search(&query);
                    app.set_success_message(format!(
                        "Re-running search '{}' - found {} results",
                        query, count
                    ));
                }
            } else {
                app.set_success_message("No search results to select".to_string());
            }
            app.needs_redraw = true;
            false
        }

        // Handle input for search
        KeyCode::Char(_) | KeyCode::Backspace => {
            SearchHandler::handle_search_input(key, app);
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
pub fn launch_external_editor(app: &mut App, snippet_id: uuid::Uuid) {
    // Set flag to indicate a full UI redraw will be needed after editor use
    app.needs_redraw = true;

    if let Some(snippet) = app.snippet_database.snippets.get(&snippet_id) {
        if let Some(ref storage) = app.storage_manager {
            let file_path = storage.get_snippet_file_path(snippet);

            if let Err(e) = storage.save_snippet_content(snippet) {
                app.set_error_message(format!("Failed to prepare file for editing: {}", e));
                return;
            }

            if let Err(e) = suspend_tui_for_editor(&file_path) {
                app.set_error_message(format!("Failed to launch editor: {}", e));
                return;
            }

            if let Ok(content) = storage.load_snippet_content(
                snippet.id,
                snippet.notebook_id,
                &snippet.file_extension,
            ) {
                if let Some(snippet) = app.snippet_database.snippets.get_mut(&snippet_id) {
                    snippet.update_content(content);

                    if let Err(e) = storage.save_snippet_content(snippet) {
                        app.set_error_message(format!("Failed to save snippet: {}", e));
                    } else {
                        if let Err(e) = app.save_database() {
                            app.set_error_message(format!("Failed to save database: {}", e));
                        } else {
                            app.set_success_message("Snippet saved successfully!".to_string());

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
    print!("\x1B[H"); // Move cursor to home position
    stdout().flush()?;

    Ok(())
}

/// Handles keyboard input specifically for the start page (main menu)
fn handle_start_page_keys(key: KeyEvent, app: &mut App) -> bool {
    // If backup/restore overlay is open, handle its keys first
    if app.show_backup_restore_overlay {
        return backup_restore::handle_backup_restore_keys(key, app);
    }

    // If About popup is open, handle its keys first
    if app.show_about_popup {
        return handle_about_popup_keys(key, app);
    }

    // Dismiss any messages with Enter key
    if key.code == KeyCode::Enter && (app.error_message.is_some() || app.success_message.is_some())
    {
        app.clear_messages();
        return false;
    }

    match key.code {
        // Menu navigation - move selection up/down (vi-style 'k'/'j' and arrow keys)
        KeyCode::Up | KeyCode::Down | KeyCode::Char('k') | KeyCode::Char('j') => {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => app.previous_menu_item(),
                KeyCode::Down | KeyCode::Char('j') => app.next_menu_item(),
                _ => {}
            }
            false
        }

        // Activate the currently selected menu item
        KeyCode::Enter => {
            match app.selected_menu_item {
                0 => app.navigate_to(AppState::Boilerplates),
                1 => app.navigate_to(AppState::Marketplace),
                2 => app.navigate_to(AppState::CodeSnippets),
                3 => {
                    app.navigate_to(AppState::ExportImport);
                    app.export_import_state =
                        Some(crate::ui::export_import::ExportImportState::default());
                }
                4 => {
                    app.show_backup_restore_overlay = true;
                    if app.backup_restore_state.is_none() {
                        app.backup_restore_state =
                            Some(crate::ui::backup_restore::BackupRestoreState::default());
                    }
                }
                5 => app.navigate_to(AppState::InfoPage),
                6 => app.navigate_to(AppState::Settings),
                7 => return true, // Exit
                _ => {}
            }
            false
        }
        KeyCode::Char('u') => {
            app.show_backup_restore_overlay = true;
            if app.backup_restore_state.is_none() {
                app.backup_restore_state =
                    Some(crate::ui::backup_restore::BackupRestoreState::default());
            }
            false
        }
        KeyCode::Esc => {
            if app.show_backup_restore_overlay {
                app.show_backup_restore_overlay = false;
                false
            } else {
                app.clear_messages();
                if app.can_go_back() {
                    app.go_back();
                }
                false
            }
        }
        // Quick navigation shortcuts
        KeyCode::Char('q') => {
            return true;
        }

        KeyCode::Char('a') | KeyCode::Char('A') => {
            app.show_about_popup = true;
            false
        }

        KeyCode::Char('b') => {
            app.navigate_to(AppState::Boilerplates);
            false
        }

        KeyCode::Char('m') => {
            app.navigate_to(AppState::Marketplace);
            false
        }

        KeyCode::Char('s') => {
            app.navigate_to(AppState::CodeSnippets);
            false
        }

        KeyCode::Char('e') => {
            app.navigate_to(AppState::ExportImport);
            app.export_import_state = Some(crate::ui::export_import::ExportImportState::default());
            false
        }

        KeyCode::Char('i') => {
            app.navigate_to(AppState::InfoPage);
            false
        }

        KeyCode::Char('c') => {
            app.navigate_to(AppState::Settings);
            false
        }

        // Quick search functionality from start page
        KeyCode::Char('/') => {
            app.navigate_to(AppState::CodeSnippets);
            app.code_snippets_state = CodeSnippetsState::SearchSnippets;
            app.input_mode = InputMode::Search;
            app.input_buffer.clear();
            app.search_query.clear();
            app.search_results.clear();
            false
        }

        KeyCode::Char('1')
        | KeyCode::Char('2')
        | KeyCode::Char('3')
        | KeyCode::Char('4')
        | KeyCode::Char('5')
        | KeyCode::Char('6')
        | KeyCode::Char('7')
        | KeyCode::Char('8')
        | KeyCode::Char('9')
        | KeyCode::Char('0') => {
            let index = match key.code {
                KeyCode::Char('1') => 0,
                KeyCode::Char('2') => 1,
                KeyCode::Char('3') => 2,
                KeyCode::Char('4') => 3,
                KeyCode::Char('5') => 4,
                KeyCode::Char('6') => 5,
                KeyCode::Char('7') => 6,
                KeyCode::Char('8') => 7,
                KeyCode::Char('9') => 8,
                KeyCode::Char('0') => 9,
                _ => unreachable!(),
            };

            let mut recent_snippets: Vec<_> = app.snippet_database.snippets.values().collect();
            recent_snippets.sort_by(|a, b| b.accessed_at.cmp(&a.accessed_at));

            if index < recent_snippets.len() {
                let snippet = &recent_snippets[index];
                let snippet_id = snippet.id;

                app.navigate_to(AppState::CodeSnippets);
                if let Some(snippet) = app.snippet_database.snippets.get_mut(&snippet_id) {
                    snippet.mark_accessed();
                }
                let _ = app.save_database();
                launch_external_editor(app, snippet_id);
            }

            false
        }
        _ => false,
    }
}

/// Handle keyboard input for the About popup
fn handle_about_popup_keys(key: KeyEvent, app: &mut App) -> bool {
    match key.code {
        KeyCode::Esc => {
            app.show_about_popup = false;
            false
        }
        KeyCode::Tab | KeyCode::Right | KeyCode::BackTab | KeyCode::Left => {
            if let Some(new_tab) =
                NavigationHandler::handle_tab_navigation(key, app.selected_about_tab, 4)
            {
                app.selected_about_tab = new_tab;
            }
            false
        }
        _ => false,
    }
}

/// Handles keyboard input for all non-start pages (WIP dialogs and future pages)
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

fn handle_notebook_details_keys(key: KeyEvent, app: &mut App, notebook_id: uuid::Uuid) -> bool {
    // If search mode is active, handle search keys
    if app.input_mode == InputMode::Search {
        return handle_search_keys(key, app);
    }

    // Check if we have a pending confirmation
    if app.has_pending_action() {
        match key.code {
            KeyCode::Enter => {
                app.confirm_pending_action();
                return false;
            }
            KeyCode::Esc => {
                app.cancel_pending_action();
                return false;
            }
            _ => return false,
        }
    }

    match key.code {
        KeyCode::Esc => {
            app.code_snippets_state = CodeSnippetsState::NotebookList;
            app.clear_messages();
            false
        }

        KeyCode::Char('e') | KeyCode::Char('E') => {
            // Edit notebook name
            if let Some(notebook) = app.snippet_database.notebooks.get(&notebook_id) {
                app.input_buffer = notebook.name.clone();
                app.input_mode = InputMode::EditNotebookName;
                app.current_notebook_id = Some(notebook_id);
            }
            false
        }

        KeyCode::Char('d') | KeyCode::Char('D') => {
            // Edit notebook description
            if let Some(notebook) = app.snippet_database.notebooks.get(&notebook_id) {
                app.input_buffer = notebook.description.clone().unwrap_or_default();
                app.input_mode = InputMode::EditNotebookDescription;
                app.current_notebook_id = Some(notebook_id);
            }
            false
        }

        KeyCode::Char('c') | KeyCode::Char('C') => {
            // Change notebook color
            app.input_mode = InputMode::SelectNotebookColor;
            app.current_notebook_id = Some(notebook_id);
            false
        }

        KeyCode::Char('s') | KeyCode::Char('S') => {
            // Create snippet in this notebook
            app.code_snippets_state = CodeSnippetsState::CreateSnippet { notebook_id };
            app.input_mode = InputMode::CreateSnippet;
            app.input_buffer.clear();
            false
        }

        KeyCode::Char('x') | KeyCode::Char('X') => {
            // Delete notebook with confirmation
            let notebook_name = app
                .snippet_database
                .notebooks
                .get(&notebook_id)
                .map(|n| n.name.clone())
                .unwrap_or_else(|| "Unknown".to_string());

            app.set_pending_action(
                format!(
                    "Delete notebook \"{}\" and all its snippets?",
                    notebook_name
                ),
                Box::new(move |app: &mut App| {
                    if let Err(e) = app.delete_notebook(notebook_id) {
                        app.set_error_message(e);
                    } else {
                        app.set_success_message(format!("Notebook \"{}\" deleted", notebook_name));
                        app.code_snippets_state = CodeSnippetsState::NotebookList;
                    }
                }),
            );
            false
        }

        // Activate search mode with '/'
        KeyCode::Char('/') => {
            SearchHandler::activate_search_mode(app);
            false
        }

        // Open snippet in Ollama chat
        KeyCode::Char('l') => {
            if let Some(TreeItem::Snippet(snippet_id, _)) = app.get_selected_item() {
                OllamaHandler::open_snippet_chat(app, *snippet_id);
            } else {
                app.set_error_message("Select a snippet first".to_string());
            }
            false
        }

        _ => false,
    }
}

#[allow(dead_code)]
fn get_available_colors() -> Vec<(&'static str, ratatui::style::Color)> {
    vec![
        ("Default", RosePine::TEXT),
        ("Red", RosePine::LOVE),
        ("Orange", RosePine::GOLD),
        ("Green", RosePine::FOAM),
        ("Blue", RosePine::IRIS),
        ("Purple", RosePine::IRIS),
        ("Pink", RosePine::ROSE),
        ("White", ratatui::style::Color::White),
    ]
}

/// Handles keyboard input for the export/import page
fn handle_export_import_keys(key: KeyEvent, app: &mut App) -> bool {
    use crate::models::{
        import_database, import_from_clipboard, merge_import_into_database_with_tags,
    };
    use crate::ui::export_import::{ExportImportMode, ExportImportState};
    use std::path::Path;

    // Get mutable reference to export/import state
    if app.export_import_state.is_none() {
        app.export_import_state = Some(ExportImportState::default());
    }

    let state = app.export_import_state.as_mut().unwrap();

    // If we have a status message showing, any key dismisses it
    if state.status_message.is_some() {
        state.status_message = None;
        return false;
    }

    match state.mode {
        ExportImportMode::MainMenu => {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    state.selected_option = state.selected_option.saturating_sub(1);
                    false
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    state.selected_option = (state.selected_option + 1).min(2);
                    false
                }
                KeyCode::Char('e') | KeyCode::Char('E') => {
                    // Export to JSON
                    state.mode = ExportImportMode::ExportOptions;
                    state.selected_option = 0;
                    false
                }
                KeyCode::Char('f') | KeyCode::Char('F') => {
                    // Import from file
                    state.mode = ExportImportMode::ImportOptions;
                    state.selected_option = 0;
                    false
                }
                KeyCode::Char('c') | KeyCode::Char('C') => {
                    // Import from clipboard
                    state.mode = ExportImportMode::ImportClipboard;
                    false
                }
                KeyCode::Enter => {
                    match state.selected_option {
                        0 => {
                            // Export to JSON
                            state.mode = ExportImportMode::ExportOptions;
                            state.selected_option = 0;
                            false
                        }
                        1 => {
                            // Import from file
                            state.mode = ExportImportMode::ImportOptions;
                            state.selected_option = 0;
                            false
                        }
                        2 => {
                            // Import from clipboard
                            state.mode = ExportImportMode::ImportClipboard;
                            false
                        }
                        _ => false,
                    }
                }
                KeyCode::Esc => {
                    if app.can_go_back() {
                        app.go_back();
                    }
                    false
                }
                _ => false,
            }
        }
        ExportImportMode::ExportOptions => {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    state.selected_option = state.selected_option.saturating_sub(1);
                    false
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    state.selected_option = (state.selected_option + 1).min(3);
                    false
                }
                KeyCode::Enter => {
                    match state.selected_option {
                        0 => {
                            // Toggle include content
                            state.include_content = !state.include_content;
                            false
                        }
                        1 => {
                            // Toggle favorites only
                            state.favorites_only = !state.favorites_only;
                            false
                        }
                        2 => {
                            // Cycle through formats
                            state.export_format = match state.export_format {
                                ExportFormat::JSON => ExportFormat::YAML,
                                ExportFormat::YAML => ExportFormat::TOML,
                                ExportFormat::TOML => ExportFormat::JSON,
                            };
                            false
                        }
                        3 => {
                            // Continue to path selection
                            state.mode = ExportImportMode::ExportPath;

                            // Set the default extension based on the selected format
                            let extension = match state.export_format {
                                ExportFormat::JSON => "json",
                                ExportFormat::YAML => "yaml",
                                ExportFormat::TOML => "toml",
                            };

                            // Update the export path with the correct extension
                            let path = state.export_path.clone();
                            if let Some(file_stem) = path.file_stem() {
                                let file_stem_str = file_stem.to_string_lossy();
                                if let Some(parent) = path.parent() {
                                    state.export_path =
                                        parent.join(format!("{}.{}", file_stem_str, extension));
                                } else {
                                    state.export_path =
                                        PathBuf::from(format!("{}.{}", file_stem_str, extension));
                                }
                            }

                            app.input_buffer = state.export_path.to_string_lossy().to_string();
                            false
                        }
                        _ => false,
                    }
                }
                KeyCode::Esc => {
                    state.mode = ExportImportMode::MainMenu;
                    state.selected_option = 0;
                    false
                }
                _ => false,
            }
        }
        ExportImportMode::ExportPath => {
            match key.code {
                KeyCode::Enter => {
                    let path = PathBuf::from(app.input_buffer.trim());
                    state.export_path = path.to_path_buf();
                    app.input_buffer.clear();

                    state.mode = ExportImportMode::Exporting;

                    // Create options
                    let options = crate::models::export::ExportOptions {
                        _format: state.export_format,
                        include_content: state.include_content,
                        notebook_ids: None,
                        include_favorites_only: state.favorites_only,
                    };

                    // Export
                    if let Err(e) = crate::models::export::export_database_with_tags(
                        &app.snippet_database,
                        &app.tag_manager,
                        &state.export_path,
                        &options,
                    ) {
                        state.status_message = Some(format!("Export failed: {}", e));
                        state.is_error = true;
                    } else {
                        state.status_message = Some(format!(
                            "Export successful! Saved to {}",
                            state.export_path.display()
                        ));
                        state.is_error = false;
                    }

                    state.mode = ExportImportMode::MainMenu;
                    false
                }
                KeyCode::Esc => {
                    app.input_buffer.clear();
                    state.mode = ExportImportMode::ExportOptions;
                    false
                }
                KeyCode::Tab => {
                    // Cycle through formats: JSON -> YAML -> TOML -> JSON
                    state.export_format = match state.export_format {
                        ExportFormat::JSON => ExportFormat::YAML,
                        ExportFormat::YAML => ExportFormat::TOML,
                        ExportFormat::TOML => ExportFormat::JSON,
                    };

                    // Update file extension based on format
                    if !app.input_buffer.is_empty() {
                        let path = PathBuf::from(app.input_buffer.trim());
                        let file_stem = path
                            .file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();

                        let extension = match state.export_format {
                            ExportFormat::JSON => "json",
                            ExportFormat::YAML => "yaml",
                            ExportFormat::TOML => "toml",
                        };

                        app.input_buffer = if let Some(parent) = path.parent() {
                            if parent.as_os_str().is_empty() {
                                format!("{}.{}", file_stem, extension)
                            } else {
                                format!("{}/{}.{}", parent.display(), file_stem, extension)
                            }
                        } else {
                            format!("{}.{}", file_stem, extension)
                        };
                    } else {
                        // If input buffer is empty, create a default filename with correct extension
                        let filename = match state.export_format {
                            ExportFormat::JSON => "snippets_export.json",
                            ExportFormat::YAML => "snippets_export.yaml",
                            ExportFormat::TOML => "snippets_export.toml",
                        };
                        app.input_buffer = filename.to_string();
                    }
                    false
                }
                KeyCode::Char(c) => {
                    app.input_buffer.push(c);
                    false
                }
                KeyCode::Backspace => {
                    if !app.input_buffer.is_empty() {
                        app.input_buffer.pop();
                    }
                    false
                }
                _ => false,
            }
        }
        ExportImportMode::ImportOptions => {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    state.selected_option = state.selected_option.saturating_sub(1);
                    false
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    state.selected_option = (state.selected_option + 1).min(1);
                    false
                }
                KeyCode::Enter => {
                    match state.selected_option {
                        0 => {
                            // Toggle overwrite existing
                            state.overwrite_existing = !state.overwrite_existing;
                            false
                        }
                        1 => {
                            // Continue to file selection
                            state.mode = ExportImportMode::ImportPathPopup;
                            app.input_buffer.clear();
                            false
                        }
                        _ => false,
                    }
                }
                KeyCode::Esc => {
                    state.mode = ExportImportMode::MainMenu;
                    state.selected_option = 1;
                    false
                }
                _ => false,
            }
        }
        ExportImportMode::ImportPathPopup => {
            match key.code {
                KeyCode::Enter => {
                    if !app.input_buffer.is_empty() {
                        let path = Path::new(&app.input_buffer);
                        state.import_path = path.to_path_buf();

                        // Set path and mode
                        {
                            let state = app.export_import_state.as_mut().unwrap();
                            state.import_path = path.to_path_buf();
                            state.mode = ExportImportMode::Importing;
                        }

                        // Store the overwrite value
                        let overwrite =
                            app.export_import_state.as_ref().unwrap().overwrite_existing;
                        let import_path = app
                            .export_import_state
                            .as_ref()
                            .unwrap()
                            .import_path
                            .clone();

                        // Take ownership of the tag manager to avoid borrow issues
                        let mut tag_manager_clone = app.tag_manager.clone();

                        match import_database(&import_path) {
                            Ok(import_data) => {
                                // Use the function that handles tags
                                match merge_import_into_database_with_tags(
                                    &mut app.snippet_database,
                                    &mut tag_manager_clone,
                                    import_data,
                                    overwrite,
                                ) {
                                    Ok((notebooks, snippets)) => {
                                        // Update the app's tag manager with the merged one
                                        app.tag_manager = tag_manager_clone;
                                        app.refresh_tree_items();

                                        let save_result = app.save_database();

                                        // Update the status message and mode
                                        let state = app.export_import_state.as_mut().unwrap();
                                        if let Err(e) = save_result {
                                            state.status_message = Some(format!(
                                                "Import succeeded but failed to save database: {}",
                                                e
                                            ));
                                            state.is_error = true;
                                        } else {
                                            state.status_message = Some(format!(
                                                "Successfully imported {} notebooks and {} snippets",
                                                notebooks, snippets
                                            ));
                                            state.is_error = false;
                                        }

                                        state.mode = ExportImportMode::MainMenu;
                                    }
                                    Err(e) => {
                                        let state = app.export_import_state.as_mut().unwrap();
                                        state.status_message =
                                            Some(format!("Failed to merge import data: {}", e));
                                        state.is_error = true;
                                        state.mode = ExportImportMode::MainMenu;
                                    }
                                }
                            }
                            Err(e) => {
                                let state = app.export_import_state.as_mut().unwrap();
                                state.status_message = Some(format!("Import failed: {}", e));
                                state.is_error = true;
                                state.mode = ExportImportMode::MainMenu;
                            }
                        }
                    }
                    false
                }
                KeyCode::Esc => {
                    state.mode = ExportImportMode::ImportOptions;
                    app.input_buffer.clear();
                    false
                }
                KeyCode::Char(c) => {
                    app.input_buffer.push(c);
                    false
                }
                KeyCode::Backspace => {
                    // Delete character if the buffer is not empty
                    // If empty, do nothing but don't exit the popup
                    if !app.input_buffer.is_empty() {
                        app.input_buffer.pop();
                    }
                    // Always return false to prevent the backspace from
                    // propagating and potentially triggering another handler
                    false
                }
                KeyCode::Tab => {
                    // Implement Tab completion
                    complete_path(&mut app.input_buffer);
                    false
                }
                _ => false,
            }
        }
        ExportImportMode::ImportClipboard => {
            match key.code {
                KeyCode::Enter => {
                    {
                        let state = app.export_import_state.as_mut().unwrap();
                        state.mode = ExportImportMode::Importing;
                    }

                    // Store the overwrite value
                    let overwrite = app.export_import_state.as_ref().unwrap().overwrite_existing;

                    // Take ownership of the tag manager to avoid borrow issues
                    let mut tag_manager_clone = app.tag_manager.clone();

                    match import_from_clipboard() {
                        Ok(Some(import_data)) => {
                            // Use the function that handles tags
                            match merge_import_into_database_with_tags(
                                &mut app.snippet_database,
                                &mut tag_manager_clone,
                                import_data,
                                overwrite,
                            ) {
                                Ok((notebooks, snippets)) => {
                                    // Update the app's tag manager with the merged one
                                    app.tag_manager = tag_manager_clone;
                                    app.refresh_tree_items();
                                    let save_result = app.save_database();

                                    // Update the status message and mode
                                    let state = app.export_import_state.as_mut().unwrap();
                                    if let Err(e) = save_result {
                                        state.status_message = Some(format!(
                                            "Import succeeded but failed to save database: {}",
                                            e
                                        ));
                                        state.is_error = true;
                                    } else {
                                        state.status_message = Some(format!(
                                            "Successfully imported {} notebooks and {} snippets from clipboard",
                                            notebooks, snippets
                                        ));
                                        state.is_error = false;
                                    }

                                    state.mode = ExportImportMode::MainMenu;
                                }
                                Err(e) => {
                                    let state = app.export_import_state.as_mut().unwrap();
                                    state.status_message =
                                        Some(format!("Failed to merge import data: {}", e));
                                    state.is_error = true;
                                    state.mode = ExportImportMode::MainMenu;
                                }
                            }
                        }
                        Ok(None) => {
                            // Handle empty clipboard
                            let state = app.export_import_state.as_mut().unwrap();
                            state.status_message = Some("Clipboard is empty".to_string());
                            state.is_error = true;
                            state.mode = ExportImportMode::MainMenu;
                        }
                        Err(e) => {
                            // Handle error
                            let state = app.export_import_state.as_mut().unwrap();
                            state.status_message = Some(format!("Clipboard import failed: {}", e));
                            state.is_error = true;
                            state.mode = ExportImportMode::MainMenu;
                        }
                    }
                    false
                }
                KeyCode::Esc => {
                    state.mode = ExportImportMode::MainMenu;
                    state.selected_option = 2;
                    false
                }
                _ => false,
            }
        }
        ExportImportMode::Exporting | ExportImportMode::Importing => {
            // We shouldn't normally reach here as these are transitional states
            // But if we do, just go back to the main menu
            state.mode = ExportImportMode::MainMenu;
            false
        }
        // This mode is deprecated in favor of ImportPathPopup, but we need to handle it
        ExportImportMode::_ImportPath => {
            // Just redirect to the popup version
            state.mode = ExportImportMode::ImportPathPopup;
            false
        }
    }
}

/// Function to handle path autocompletion
fn complete_path(input_buffer: &mut String) {
    let path_str = input_buffer.trim();

    // If input is empty, use a default path
    if path_str.is_empty() {
        *input_buffer = "snippets_export.json".to_string();
        return;
    }

    // Expand tilde to home directory
    let expanded_path = if path_str.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            home.join(path_str.trim_start_matches("~/"))
                .display()
                .to_string()
        } else {
            path_str.to_string()
        }
    } else {
        path_str.to_string()
    };

    // Get directory portion and filename portion
    let (dir_path, file_prefix) = match expanded_path.rfind('/') {
        Some(pos) => {
            let (dir, file) = expanded_path.split_at(pos + 1);
            (dir.to_string(), file.to_string())
        }
        None => {
            // No slash, assume current directory
            ("./".to_string(), expanded_path.to_string())
        }
    };

    // Try to read the directory and find matching files
    if let Ok(entries) = std::fs::read_dir(&dir_path) {
        let mut matches = Vec::new();

        // Collect all matching entries
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(&file_prefix) {
                let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
                let full_path = format!("{}{}", dir_path, name);

                // Add slash to directories
                let path_with_suffix = if is_dir {
                    format!("{}/", full_path)
                } else {
                    full_path
                };

                matches.push((name, path_with_suffix, is_dir));
            }
        }

        // Sort directories first, then files
        matches.sort_by(|a, b| {
            if a.2 && !b.2 {
                std::cmp::Ordering::Less
            } else if !a.2 && b.2 {
                std::cmp::Ordering::Greater
            } else {
                a.0.cmp(&b.0)
            }
        });

        // Complete with the first match if there's only one,
        // or complete to the common prefix if there are multiple
        if matches.len() == 1 {
            *input_buffer = matches[0].1.clone();
        } else if matches.len() > 1 {
            // Find common prefix
            let mut common_prefix = String::new();
            if let Some(first_name) = matches.first().map(|m| &m.0) {
                common_prefix = first_name.clone();

                for (name, _, _) in &matches[1..] {
                    // Find common characters between common_prefix and name
                    let mut new_prefix = String::new();
                    for (c1, c2) in common_prefix.chars().zip(name.chars()) {
                        if c1 == c2 {
                            new_prefix.push(c1);
                        } else {
                            break;
                        }
                    }
                    common_prefix = new_prefix;

                    if common_prefix.is_empty() {
                        break;
                    }
                }
            }

            // Apply the common prefix if it's longer than the current prefix
            if common_prefix.len() > file_prefix.len() {
                *input_buffer = format!("{}{}", dir_path, common_prefix);
            }
        }
    }
}

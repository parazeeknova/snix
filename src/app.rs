use crate::models::{CodeSnippet, Notebook, SnippetDatabase, SnippetLanguage, StorageManager};
use crate::ui::{code_snippets, components, start_page};
use chrono::Utc;
use ratatui::Frame;
use uuid::Uuid;

/// Application State Enumeration
///
/// Represents all possible states (pages) that the application can be in.
/// Each variant corresponds to a different screen or page in the user interface.
///
/// This enum is used to track which page is currently active and to handle
/// navigation between different sections of the application. The state determines
/// which rendering function is called and what content is displayed to the user.
#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    StartPage,
    Boilerplates,
    Marketplace,
    CodeSnippets,
    InfoPage,
    Settings,
}

impl Default for AppState {
    /// Returns the default application state
    ///
    /// The application always starts on the StartPage, which serves as the main
    /// navigation hub for accessing all other features and pages.
    fn default() -> Self {
        AppState::StartPage
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CodeSnippetsState {
    NotebookList,
    _NotebookView { notebook_id: Uuid },
    NotebookDetails { notebook_id: Uuid },
    _SnippetEditor { snippet_id: Uuid },
    _CreateNotebook,
    CreateSnippet { notebook_id: Uuid },
    SearchSnippets,
    Settings,
}

/// Tree view item types for navigation
#[derive(Debug, Clone, PartialEq)]
pub enum TreeItem {
    Notebook(Uuid, usize), // Added depth parameter for indentation
    Snippet(Uuid, usize),  // Added depth parameter for indentation
}

/// Main Application State Container
///
/// This struct holds all the state information needed to run the application.
/// It tracks the current page, menu selection, and navigation history to provide
/// a smooth user experience with proper back navigation.
///
/// The App struct is the central hub for all state management and is passed
/// to rendering functions to determine what content to display and how to
/// style interactive elements based on the current state.

#[derive(Debug)]
pub struct App {
    pub state: AppState,
    pub selected_menu_item: usize,
    pub page_history: Vec<AppState>,

    // Code Snippets Manager State
    pub code_snippets_state: CodeSnippetsState,
    pub snippet_database: SnippetDatabase,
    pub storage_manager: Option<StorageManager>,
    pub selected_tree_item: usize,
    pub hovered_tree_item: Option<usize>,
    pub tree_items: Vec<TreeItem>,
    pub current_notebook_id: Option<Uuid>,
    pub search_query: String,
    pub show_favorites_only: bool,
    pub error_message: Option<String>,
    pub success_message: Option<String>,
    pub input_buffer: String,
    pub input_mode: InputMode,
    pub selected_language: usize,
    pub pending_snippet_title: String,
    pub needs_redraw: bool,
    pub content_scroll_position: usize,
    pub selected_details_tab: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortBy {
    _Updated,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    CreateNotebook,
    CreateNestedNotebook,
    CreateSnippet,
    SelectLanguage,
    Search,
    HelpMenu,
    _RenameNotebook,
    _RenameSnippet,
    EditSnippetDescription,
}

impl App {
    /// Creates a new instance of the application with default initial state
    ///
    /// Initializes the application in the StartPage state with the first menu item
    /// selected and an empty navigation history. This provides a clean starting
    /// point for the user interface.
    pub fn new() -> Self {
        let storage_manager = StorageManager::new().ok();
        let snippet_database = if let Some(ref manager) = storage_manager {
            manager.load_database().unwrap_or_default()
        } else {
            SnippetDatabase::default()
        };

        let mut app = Self {
            state: AppState::StartPage,
            selected_menu_item: 0,
            page_history: vec![AppState::StartPage],

            code_snippets_state: CodeSnippetsState::NotebookList,
            snippet_database,
            storage_manager,
            selected_tree_item: 0,
            hovered_tree_item: None,
            tree_items: Vec::new(),
            current_notebook_id: None,
            search_query: String::new(),
            show_favorites_only: false,
            error_message: None,
            success_message: None,
            input_buffer: String::new(),
            input_mode: InputMode::Normal,
            selected_language: 0,
            pending_snippet_title: String::new(),
            needs_redraw: true,
            content_scroll_position: 0,
            selected_details_tab: 0,
        };

        app.refresh_tree_items();
        app
    }

    /// Moves the menu selection to the next item in a circular fashion
    ///
    /// Increments the selected menu item index, wrapping around to 0 when it
    /// reaches the maximum number of menu items. This allows users to navigate
    /// through menu options using the down arrow or 'j' key.
    ///
    /// The total number of menu items is currently 6 (indices 0-5), so the
    /// selection will cycle through all available options.
    pub fn next_menu_item(&mut self) {
        self.selected_menu_item = (self.selected_menu_item + 1) % 6;
    }

    /// Moves the menu selection to the previous item in a circular fashion
    ///
    /// Decrements the selected menu item index, wrapping around to the last item
    /// (index 5) when it goes below 0. This allows users to navigate through menu
    /// options using the up arrow or 'k' key in reverse order.
    pub fn previous_menu_item(&mut self) {
        self.selected_menu_item = if self.selected_menu_item == 0 {
            5
        } else {
            self.selected_menu_item - 1
        };
    }

    /// Navigates to a new application state and updates the page history
    ///
    /// Changes the current application state to the specified new state, but only
    /// if it's different from the current state. The current state is saved to
    /// the page history stack before transitioning, enabling back navigation.
    ///
    /// This method is the primary way to move between different pages in the
    /// application and ensures that navigation history is properly maintained.
    pub fn navigate_to(&mut self, new_state: AppState) {
        if self.state != new_state {
            self.page_history.push(self.state.clone());
            self.state = new_state;

            // Reset code snippets state when entering
            if self.state == AppState::CodeSnippets {
                self.code_snippets_state = CodeSnippetsState::NotebookList;
                self.refresh_tree_items();
            }
        }
    }

    /// Navigates back to the previous page in the history stack
    ///
    /// Pops the most recent state from the page history and sets it as the current
    /// state. This provides a standard "back" navigation experience similar to web
    /// browsers or mobile applications.
    ///
    /// If there's no history to go back to (empty history stack), this method
    /// does nothing, preventing the application from getting into an invalid state.
    pub fn go_back(&mut self) {
        if let Some(previous_state) = self.page_history.pop() {
            self.state = previous_state;
        }
    }

    /// Checks whether back navigation is possible
    ///
    /// Returns true if there are states in the page history stack that the user
    /// can navigate back to. This is useful for conditionally showing back buttons
    /// or enabling back navigation shortcuts.
    pub fn can_go_back(&self) -> bool {
        !self.page_history.is_empty()
    }

    // Code Snippets Manager Methods
    pub fn refresh_tree_items(&mut self) {
        self.tree_items.clear();

        let root_notebooks = self.snippet_database.root_notebooks.clone();
        for notebook_id in root_notebooks {
            self.add_notebook_to_tree(notebook_id, 0);
        }

        if self.tree_items.is_empty() {
            self.selected_tree_item = 0;
        } else {
            self.selected_tree_item = self.selected_tree_item.min(self.tree_items.len() - 1);
        }
    }

    fn add_notebook_to_tree(&mut self, notebook_id: Uuid, depth: usize) {
        self.tree_items.push(TreeItem::Notebook(notebook_id, depth));

        let snippets: Vec<_> = self
            .snippet_database
            .snippets
            .values()
            .filter(|s| s.notebook_id == notebook_id)
            .map(|s| s.id)
            .collect();

        for snippet_id in snippets {
            self.tree_items
                .push(TreeItem::Snippet(snippet_id, depth + 1));
        }

        if let Some(notebook) = self.snippet_database.notebooks.get(&notebook_id) {
            let children = notebook.children.clone();
            for child_id in children {
                self.add_notebook_to_tree(child_id, depth + 1);
            }
        }
    }

    pub fn next_tree_item(&mut self) {
        if !self.tree_items.is_empty() {
            self.selected_tree_item = (self.selected_tree_item + 1) % self.tree_items.len();
            // Update the hovered item to match the selected item
            self.hovered_tree_item = Some(self.selected_tree_item);
            self.needs_redraw = true;
        }
    }

    pub fn previous_tree_item(&mut self) {
        if !self.tree_items.is_empty() {
            self.selected_tree_item = if self.selected_tree_item > 0 {
                self.selected_tree_item - 1
            } else {
                self.tree_items.len() - 1
            };
            self.hovered_tree_item = Some(self.selected_tree_item);
            self.needs_redraw = true;
        }
    }

    pub fn create_notebook(&mut self, name: String) -> Result<Uuid, String> {
        if name.trim().is_empty() {
            return Err("Notebook name cannot be empty".to_string());
        }

        // Determine parent notebook ID from either current_notebook_id or selected tree item
        let parent_id = if let Some(id) = self.current_notebook_id {
            Some(id)
        } else if let Some(TreeItem::Notebook(id, _)) = self.get_selected_item() {
            Some(*id)
        } else if let Some(TreeItem::Snippet(snippet_id, _)) = self.get_selected_item() {
            // If a snippet is selected, use its notebook as parent
            if let Some(snippet) = self.snippet_database.snippets.get(snippet_id) {
                Some(snippet.notebook_id)
            } else {
                None
            }
        } else {
            None
        };

        let notebook = if let Some(parent_id) = parent_id {
            Notebook::new_with_parent(name, parent_id)
        } else {
            Notebook::new(name)
        };

        let notebook_id = notebook.id;

        if let Some(parent_id) = notebook.parent_id {
            if let Some(parent) = self.snippet_database.notebooks.get_mut(&parent_id) {
                parent.add_child(notebook_id);
            }
        } else {
            self.snippet_database.root_notebooks.push(notebook_id);
        }

        self.snippet_database
            .notebooks
            .insert(notebook_id, notebook);

        if let Err(e) = self.save_database() {
            return Err(format!("Failed to save notebook: {}", e));
        }

        self.refresh_tree_items();
        Ok(notebook_id)
    }

    pub fn create_snippet(
        &mut self,
        title: String,
        language: SnippetLanguage,
        notebook_id: Uuid,
    ) -> Result<Uuid, String> {
        if title.trim().is_empty() {
            return Err("Snippet title cannot be empty".to_string());
        }

        if !self.snippet_database.notebooks.contains_key(&notebook_id) {
            return Err("Notebook not found".to_string());
        }

        let snippet = CodeSnippet::new(title, language, notebook_id);
        let snippet_id = snippet.id;

        self.snippet_database.snippets.insert(snippet_id, snippet);

        if let Some(notebook) = self.snippet_database.notebooks.get_mut(&notebook_id) {
            notebook.update_snippet_count(
                self.snippet_database
                    .snippets
                    .values()
                    .filter(|s| s.notebook_id == notebook_id)
                    .count(),
            );
        }

        if let Err(e) = self.save_database() {
            return Err(format!("Failed to save snippet: {}", e));
        }

        self.refresh_tree_items();
        Ok(snippet_id)
    }

    pub fn delete_notebook(&mut self, notebook_id: Uuid) -> Result<(), String> {
        // Check if notebook exists
        if !self.snippet_database.notebooks.contains_key(&notebook_id) {
            return Err("Notebook not found".to_string());
        }

        // Delete all snippets in this notebook
        let snippet_ids: Vec<_> = self
            .snippet_database
            .snippets
            .values()
            .filter(|s| s.notebook_id == notebook_id)
            .map(|s| s.id)
            .collect();

        for snippet_id in snippet_ids {
            self.delete_snippet(snippet_id)?;
        }

        // Remove from parent's children or root list
        if let Some(notebook) = self.snippet_database.notebooks.get(&notebook_id) {
            if let Some(parent_id) = notebook.parent_id {
                if let Some(parent) = self.snippet_database.notebooks.get_mut(&parent_id) {
                    parent.remove_child(&notebook_id);
                }
            } else {
                self.snippet_database
                    .root_notebooks
                    .retain(|&id| id != notebook_id);
            }
        }

        self.snippet_database.notebooks.remove(&notebook_id);
        if let Some(ref storage) = self.storage_manager {
            if let Err(e) = storage.delete_notebook_directory(notebook_id) {
                eprintln!("Warning: Failed to delete notebook directory: {}", e);
            }
        }

        if let Err(e) = self.save_database() {
            return Err(format!("Failed to save changes: {}", e));
        }

        self.refresh_tree_items();
        Ok(())
    }

    pub fn delete_snippet(&mut self, snippet_id: Uuid) -> Result<(), String> {
        if let Some(snippet) = self.snippet_database.snippets.remove(&snippet_id) {
            if let Some(ref storage) = self.storage_manager {
                if let Err(e) = storage.delete_snippet_file(&snippet) {
                    eprintln!("Warning: Failed to delete snippet file: {}", e);
                }
            }

            if let Some(notebook) = self
                .snippet_database
                .notebooks
                .get_mut(&snippet.notebook_id)
            {
                notebook.update_snippet_count(
                    self.snippet_database
                        .snippets
                        .values()
                        .filter(|s| s.notebook_id == snippet.notebook_id)
                        .count(),
                );
            }

            if let Err(e) = self.save_database() {
                return Err(format!("Failed to save changes: {}", e));
            }

            self.refresh_tree_items();
            Ok(())
        } else {
            Err("Snippet not found".to_string())
        }
    }

    pub fn get_selected_item(&self) -> Option<&TreeItem> {
        self.tree_items.get(self.selected_tree_item)
    }

    pub fn save_database(&self) -> Result<(), String> {
        if let Some(ref storage) = self.storage_manager {
            storage
                .save_database(&self.snippet_database)
                .map_err(|e| e.to_string())
        } else {
            Err("Storage manager not available".to_string())
        }
    }

    pub fn set_error_message(&mut self, message: String) {
        self.error_message = Some(message);
        self.success_message = None;
    }

    pub fn set_success_message(&mut self, message: String) {
        self.success_message = Some(message);
        self.error_message = None;
    }

    pub fn clear_messages(&mut self) {
        self.error_message = None;
        self.success_message = None;
    }

    /// Call this periodically to auto-clear messages after a timeout
    pub fn _tick(&mut self) {
        // Messages will be cleared by user interaction or manual clearing
        // This is a placeholder for future auto-clear functionality
    }

    /// Renders the current application state to the terminal frame
    ///
    /// This is the main rendering dispatch method that determines which UI rendering
    /// function to call based on the current application state. It acts as a router,
    /// directing the rendering process to the appropriate page implementation.
    ///
    /// For the StartPage, it calls the dedicated start page renderer. For all other
    /// states, it displays a work-in-progress dialog with appropriate page titles
    /// and icons, maintaining consistent navigation while indicating that those
    /// features are under development.
    pub fn render(&mut self, frame: &mut Frame) {
        match self.state {
            AppState::StartPage => start_page::render(frame, self),
            AppState::Boilerplates => {
                components::render_wip_dialog(frame, frame.area(), "󰘦 Boilerplates", self)
            }
            AppState::Marketplace => {
                components::render_wip_dialog(frame, frame.area(), "󰓜 Marketplace", self)
            }
            AppState::CodeSnippets => code_snippets::render(frame, self),
            AppState::InfoPage => {
                components::render_wip_dialog(frame, frame.area(), "  About", self)
            }
            AppState::Settings => {
                components::render_wip_dialog(frame, frame.area(), " Settings", self)
            }
        }
    }

    pub fn update_snippet_description(
        &mut self,
        snippet_id: Uuid,
        description: String,
    ) -> Result<(), String> {
        if let Some(snippet) = self.snippet_database.snippets.get_mut(&snippet_id) {
            snippet.description = if description.is_empty() {
                None
            } else {
                Some(description)
            };
            snippet.updated_at = Utc::now();

            if let Err(e) = self.save_database() {
                return Err(format!("Failed to save description: {}", e));
            }

            Ok(())
        } else {
            Err("Snippet not found".to_string())
        }
    }

    pub fn reset_scroll_position(&mut self) {
        self.content_scroll_position = 0;
        self.needs_redraw = true;
    }
}

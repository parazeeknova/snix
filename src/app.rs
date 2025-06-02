use crate::models::storage::SnippetDatabase;
use crate::models::{CodeSnippet, Notebook, SnippetLanguage, StorageManager, TagManager};
use crate::ui::backup_restore::BackupRestoreState;
use crate::ui::export_import::ExportImportState;
use crate::ui::{code_snippets, components, export_import, start_page};
use chrono::{DateTime, Utc};
use ratatui::Frame;
use uuid::Uuid;

/// Application State Enumeration
/// Represents all possible states (pages) that the application can be in.
/// Each variant corresponds to a different screen or page in the user interface.
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
    ExportImport,
}

impl Default for AppState {
    /// Returns the default application state
    /// The application always starts on the StartPage, which serves as the main
    /// navigation hub for accessing all other features and pages.
    fn default() -> Self {
        AppState::StartPage
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CodeSnippetsState {
    NotebookList,
    NotebookView { notebook_id: Uuid },
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
    Notebook(Uuid, usize),
    Snippet(Uuid, usize),
}

/// Main Application State Container
/// This struct holds all the state information needed to run the application.
/// It tracks the current page, menu selection, and navigation history to provide
/// a smooth user experience with proper back navigation.
/// The App struct is the central hub for all state management and is passed
/// to rendering functions to determine what content to display and how to
/// style interactive elements based on the current state.

pub enum ConfirmationState {
    None,
    DeleteItem {
        item_id: Uuid,
        is_notebook: bool,
    },
    _MoveItem {
        item_id: Uuid,
        is_notebook: bool,
        target_id: Uuid,
    },
    Custom {
        #[allow(dead_code)]
        action: Box<dyn FnOnce(&mut App) + 'static>,
    },
}

// Custom debug implementation since FnOnce doesn't implement Debug
impl std::fmt::Debug for ConfirmationState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfirmationState::None => write!(f, "ConfirmationState::None"),
            ConfirmationState::DeleteItem {
                item_id,
                is_notebook,
            } => {
                write!(
                    f,
                    "ConfirmationState::DeleteItem {{ item_id: {:?}, is_notebook: {:?} }}",
                    item_id, is_notebook
                )
            }
            ConfirmationState::_MoveItem {
                item_id,
                is_notebook,
                target_id,
            } => {
                write!(
                    f,
                    "ConfirmationState::_MoveItem {{ item_id: {:?}, is_notebook: {:?}, target_id: {:?} }}",
                    item_id, is_notebook, target_id
                )
            }
            ConfirmationState::Custom { .. } => {
                write!(f, "ConfirmationState::Custom {{ .. }}")
            }
        }
    }
}

// Add the following enum to track different search result types
#[derive(Debug, Clone, PartialEq)]
pub enum SearchResultType {
    Notebook,
    Snippet,
    CodeContent,
}

// Add a struct to represent a search result
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: Uuid,
    pub name: String,
    pub result_type: SearchResultType,
    pub match_context: String,
    pub parent_id: Option<Uuid>,
}

/// Struct to represent a detailed recent search entry
#[derive(Debug, Clone)]
pub struct RecentSearchEntry {
    pub query: String,
    pub timestamp: DateTime<Utc>,
    pub result_count: usize,
    pub last_selected_type: Option<SearchResultType>,
    pub last_selected_id: Option<Uuid>,
}

impl RecentSearchEntry {
    pub fn new(query: String, result_count: usize) -> Self {
        Self {
            query,
            timestamp: Utc::now(),
            result_count,
            last_selected_type: None,
            last_selected_id: None,
        }
    }

    pub fn formatted_time(&self) -> String {
        self.timestamp.format("%Y-%m-%d %H:%M").to_string()
    }
}

/// Main Application State Container
/// This struct holds all the state information needed to run the application.
/// It tracks the current page, menu selection, and navigation history to provide
/// a smooth user experience with proper back navigation.
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
    pub search_results: Vec<SearchResult>,
    pub selected_search_result: usize,
    pub show_favorites_only: bool,
    pub show_favorites_popup: bool,
    pub error_message: Option<String>,
    pub success_message: Option<String>,
    pub input_buffer: String,
    pub input_mode: InputMode,
    pub selected_language: usize,
    pub pending_snippet_title: String,
    pub needs_redraw: bool,
    pub content_scroll_position: usize,
    pub selected_details_tab: usize,
    #[allow(dead_code)]
    pub notebook_color: Option<Uuid>,
    pub collapsed_notebooks: std::collections::HashSet<Uuid>,
    pub confirmation_state: ConfirmationState,
    pub recent_searches: Vec<RecentSearchEntry>,
    pub selected_recent_search: usize,
    pub tag_manager: TagManager,
    pub export_import_state: Option<ExportImportState>,
    pub backup_restore_state: Option<BackupRestoreState>,
    pub show_backup_restore_overlay: bool,
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
    _RenameNotebook,
    _RenameSnippet,
    EditSnippetDescription,
    SelectLanguage,
    Search,
    HelpMenu,
    EditNotebookDescription,
    SelectNotebookColor,
    EditNotebookName,
    EditTags,
}

impl App {
    /// Creates a new instance of the application with default initial state
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

        // Initialize and load the tag manager
        let tag_manager = if let Some(ref manager) = storage_manager {
            manager.load_tag_manager().unwrap_or_default()
        } else {
            TagManager::new()
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
            search_results: Vec::new(),
            selected_search_result: 0,
            show_favorites_only: false,
            show_favorites_popup: false,
            error_message: None,
            success_message: None,
            input_buffer: String::new(),
            input_mode: InputMode::Normal,
            selected_language: 0,
            pending_snippet_title: String::new(),
            needs_redraw: true,
            content_scroll_position: 0,
            selected_details_tab: 0,
            notebook_color: None,
            collapsed_notebooks: std::collections::HashSet::new(),
            confirmation_state: ConfirmationState::None,
            recent_searches: Vec::new(),
            selected_recent_search: 0,
            tag_manager,
            export_import_state: None,
            backup_restore_state: None,
            show_backup_restore_overlay: false,
        };

        app.refresh_tree_items();
        app
    }

    /// Moves the menu selection to the next item in a circular fashion
    /// Increments the selected menu item index, wrapping around to 0 when it
    /// reaches the maximum number of menu items. This allows users to navigate
    /// through menu options using the down arrow or 'j' key.
    /// The total number of menu items is currently 8 (indices 0-7), so the
    /// selection will cycle through all available options.
    pub fn next_menu_item(&mut self) {
        self.selected_menu_item = (self.selected_menu_item + 1) % 8;
    }

    /// Moves the menu selection to the previous item in a circular fashion
    /// Decrements the selected menu item index, wrapping around to the last item
    /// when it reaches 0. This allows users to navigate through menu options
    /// using the up arrow or 'k' key.
    /// The total number of menu items is currently 8 (indices 0-7), so the
    /// selection will cycle through all available options.
    pub fn previous_menu_item(&mut self) {
        self.selected_menu_item = (self.selected_menu_item + 8 - 1) % 8;
    }

    /// Navigates to a new application state and updates the page history
    /// Changes the current application state to the specified new state, but only
    /// if it's different from the current state. The current state is saved to
    /// the page history stack before transitioning, enabling back navigation.
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
    /// Pops the most recent state from the page history and sets it as the current
    /// state. This provides a standard "back" navigation experience similar to web
    /// browsers or mobile applications.
    /// If there's no history to go back to (empty history stack), this method
    /// does nothing, preventing the application from getting into an invalid state.
    pub fn go_back(&mut self) {
        if let Some(previous_state) = self.page_history.pop() {
            self.state = previous_state;
        }
    }

    /// Checks whether back navigation is possible
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

        // Skip children if this notebook is collapsed
        if self.collapsed_notebooks.contains(&notebook_id) {
            return;
        }

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

        // Determine parent notebook ID based on the input mode and current state
        let parent_id = match self.input_mode {
            // For normal notebook creation, always create a root notebook
            InputMode::CreateNotebook => None,

            // For nested notebook creation, use the current_notebook_id
            InputMode::CreateNestedNotebook => {
                if let Some(id) = self.current_notebook_id {
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
                }
            }

            _ => self.current_notebook_id,
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
        // Check if the snippet exists
        if !self.snippet_database.snippets.contains_key(&snippet_id) {
            return Err("Snippet not found".to_string());
        }

        // Get the notebook ID before we remove the snippet
        let notebook_id = self
            .snippet_database
            .snippets
            .get(&snippet_id)
            .map(|s| s.notebook_id);

        // Delete the snippet file (if storage is available)
        if let Some(ref storage) = self.storage_manager {
            if let Some(snippet) = self.snippet_database.snippets.get(&snippet_id) {
                if let Err(e) = storage.delete_snippet_file(snippet) {
                    return Err(format!("Failed to delete snippet file: {}", e));
                }
            }
        }

        // Clean up any tag associations for this snippet
        self.tag_manager.handle_snippet_deleted(&snippet_id);

        // Remove the snippet from the database
        self.snippet_database.snippets.remove(&snippet_id);

        // Decrease the snippet count in the parent notebook
        if let Some(id) = notebook_id {
            if let Some(notebook) = self.snippet_database.notebooks.get_mut(&id) {
                notebook.snippet_count = notebook.snippet_count.saturating_sub(1);
                notebook.updated_at = chrono::Utc::now();
            }
        }

        // Save the updated database
        if let Err(e) = self.save_database() {
            return Err(format!(
                "Failed to save database after snippet deletion: {}",
                e
            ));
        }

        // Refresh tree items to reflect the change
        self.refresh_tree_items();
        self.selected_tree_item = self
            .selected_tree_item
            .min(self.tree_items.len().saturating_sub(1));

        Ok(())
    }

    pub fn get_selected_item(&self) -> Option<&TreeItem> {
        self.tree_items.get(self.selected_tree_item)
    }

    pub fn get_hovered_item(&self) -> Option<&TreeItem> {
        if let Some(hovered_index) = self.hovered_tree_item {
            self.tree_items.get(hovered_index)
        } else {
            self.get_selected_item()
        }
    }

    pub fn save_database(&self) -> Result<(), String> {
        if let Some(ref storage) = self.storage_manager {
            if let Err(e) = storage.save_database(&self.snippet_database) {
                return Err(format!("Failed to save database: {}", e));
            }

            // Also save the tag manager as a separate file
            if let Err(e) = storage.save_tag_manager(&self.tag_manager) {
                return Err(format!("Failed to save tags: {}", e));
            }
        } else {
            return Err("No storage manager available".to_string());
        }
        Ok(())
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
    /// This is the main entry point for all rendering in the application. It uses
    /// the current application state to determine which page-specific rendering
    /// function to call.
    /// For the StartPage, it calls the dedicated start page renderer. For all other
    /// states, it displays a work-in-progress dialog with appropriate page titles
    /// and icons, maintaining consistent navigation while indicating that those
    /// features are under development.
    pub fn render(&mut self, frame: &mut Frame) {
        match self.state {
            AppState::StartPage => {
                start_page::render(frame, self);
                if self.show_backup_restore_overlay {
                    crate::ui::backup_restore::render(frame, self);
                }
                if self.show_favorites_popup {
                    crate::ui::favorites::render_floating_favorites(frame, self);
                }
            }
            AppState::Boilerplates => {
                components::render_wip_dialog(frame, frame.area(), "󰘦 Boilerplates", self)
            }
            AppState::Marketplace => {
                components::render_wip_dialog(frame, frame.area(), "󰓜 Marketplace", self)
            }
            AppState::CodeSnippets => code_snippets::render(frame, self),
            AppState::InfoPage => {
                components::render_wip_dialog(frame, frame.area(), " About", self)
            }
            AppState::Settings => {
                components::render_wip_dialog(frame, frame.area(), " Settings", self)
            }
            AppState::ExportImport => export_import::render(frame, self),
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

    pub fn update_notebook_description(
        &mut self,
        notebook_id: Uuid,
        description: String,
    ) -> Result<(), String> {
        if let Some(notebook) = self.snippet_database.notebooks.get_mut(&notebook_id) {
            notebook.description = Some(description);
            notebook.updated_at = chrono::Utc::now();
            self.save_database()?;
            Ok(())
        } else {
            Err("Notebook not found".to_string())
        }
    }

    #[allow(dead_code)]
    pub fn update_notebook_color(
        &mut self,
        notebook_id: Uuid,
        color_index: usize,
    ) -> Result<(), String> {
        if let Some(notebook) = self.snippet_database.notebooks.get_mut(&notebook_id) {
            // Store color index in a custom field or metadata
            // For now, we'll use the description with a prefix to store the color
            let desc = notebook.description.clone().unwrap_or_default();

            // Extract description without color prefix if it exists
            let desc_without_color = if desc.starts_with("[COLOR:") {
                if let Some(end_idx) = desc.find(']') {
                    desc[end_idx + 1..].trim().to_string()
                } else {
                    desc
                }
            } else {
                desc
            };

            // Add color prefix to description
            notebook.description = Some(format!("[COLOR:{}] {}", color_index, desc_without_color));
            notebook.updated_at = chrono::Utc::now();
            self.save_database()?;
            Ok(())
        } else {
            Err("Notebook not found".to_string())
        }
    }

    pub fn get_notebook_color(&self, notebook_id: &Uuid) -> usize {
        if let Some(notebook) = self.snippet_database.notebooks.get(notebook_id) {
            if let Some(desc) = &notebook.description {
                if desc.starts_with("[COLOR:") {
                    if let Some(end_idx) = desc.find(']') {
                        if let Ok(color_idx) = desc[7..end_idx].parse::<usize>() {
                            return color_idx;
                        }
                    }
                }
            }
        }
        0 // Default color index
    }

    pub fn toggle_notebook_collapse(&mut self) -> bool {
        if let Some(TreeItem::Notebook(notebook_id, _)) = self.get_selected_item() {
            let id = *notebook_id;
            if self.collapsed_notebooks.contains(&id) {
                self.expand_notebook(id);
            } else {
                self.collapse_notebook(id);
            }
            true
        } else {
            false
        }
    }

    pub fn collapse_notebook(&mut self, notebook_id: Uuid) {
        self.collapsed_notebooks.insert(notebook_id);
        self.refresh_tree_items();
        self.needs_redraw = true;
    }

    pub fn expand_notebook(&mut self, notebook_id: Uuid) {
        self.collapsed_notebooks.remove(&notebook_id);
        self.refresh_tree_items();
        self.needs_redraw = true;
    }

    pub fn is_notebook_collapsed(&self, notebook_id: &Uuid) -> bool {
        self.collapsed_notebooks.contains(notebook_id)
    }

    // Methods to move notebooks in the hierarchy
    pub fn move_notebook_up(&mut self) -> bool {
        if let Some(TreeItem::Notebook(notebook_id, _)) = self.get_selected_item().cloned() {
            if let Some(notebook) = self.snippet_database.notebooks.get(&notebook_id).cloned() {
                // If already at root level, nothing to do
                if notebook.parent_id.is_none() {
                    self.set_error_message("Notebook is already at root level".to_string());
                    return false;
                }

                // Get parent notebook
                if let Some(parent_id) = notebook.parent_id {
                    if let Some(parent) = self.snippet_database.notebooks.get(&parent_id).cloned() {
                        // Get grandparent ID
                        let grandparent_id = parent.parent_id;

                        // Update notebook parent to grandparent (move up one level)
                        if let Some(notebook_to_update) =
                            self.snippet_database.notebooks.get_mut(&notebook_id)
                        {
                            notebook_to_update.parent_id = grandparent_id;
                            notebook_to_update.updated_at = chrono::Utc::now();
                        }

                        // Remove this notebook from parent's children
                        if let Some(parent_to_update) =
                            self.snippet_database.notebooks.get_mut(&parent_id)
                        {
                            parent_to_update.children.retain(|id| *id != notebook_id);
                            parent_to_update.updated_at = chrono::Utc::now();
                        }

                        // Add this notebook to grandparent's children if it exists
                        if let Some(grandparent_id) = grandparent_id {
                            if let Some(grandparent) =
                                self.snippet_database.notebooks.get_mut(&grandparent_id)
                            {
                                grandparent.children.push(notebook_id);
                                grandparent.updated_at = chrono::Utc::now();
                            }
                        } else {
                            // No grandparent, add to root notebooks
                            if !self.snippet_database.root_notebooks.contains(&notebook_id) {
                                self.snippet_database.root_notebooks.push(notebook_id);
                            }
                        }

                        // Save to make persistent
                        let _ = self.save_database();

                        // Update the tree view
                        self.refresh_tree_items();
                        self.needs_redraw = true;
                        self.set_success_message("Notebook moved up one level".to_string());
                        return true;
                    }
                }
            }
        } else if let Some(TreeItem::Snippet(snippet_id, _)) = self.get_selected_item().cloned() {
            if let Some(snippet) = self.snippet_database.snippets.get(&snippet_id).cloned() {
                let notebook_id = snippet.notebook_id;

                // Get current notebook
                if let Some(notebook) = self.snippet_database.notebooks.get(&notebook_id).cloned() {
                    // If notebook is at root level, snippet can't move up
                    if notebook.parent_id.is_none() {
                        self.set_error_message("Snippet is already in a root notebook".to_string());
                        return false;
                    }

                    // Get parent notebook ID
                    if let Some(parent_id) = notebook.parent_id {
                        // Move snippet to parent notebook
                        if let Some(snippet_to_update) =
                            self.snippet_database.snippets.get_mut(&snippet_id)
                        {
                            snippet_to_update.notebook_id = parent_id;
                            snippet_to_update.updated_at = chrono::Utc::now();
                        }

                        // Save to make persistent
                        let _ = self.save_database();

                        // Update the tree view
                        self.refresh_tree_items();
                        self.needs_redraw = true;
                        self.set_success_message("Snippet moved up one level".to_string());
                        return true;
                    }
                }
            }
        }

        self.set_error_message("Unable to move item up".to_string());
        false
    }

    pub fn move_notebook_down(&mut self) -> bool {
        if let Some(TreeItem::Notebook(notebook_id, _)) = self.get_selected_item().cloned() {
            // To move down, we need to select a sibling or another notebook as the new parent
            if let Some(hovered_item) = self.get_hovered_item().cloned() {
                match hovered_item {
                    TreeItem::Notebook(target_id, _) => {
                        // Can't move to itself
                        if target_id == notebook_id {
                            self.set_error_message("Cannot move notebook into itself".to_string());
                            return false;
                        }

                        // Check if target is a descendant of the notebook - can't move to own child
                        if self.is_descendant_of(&target_id, &notebook_id) {
                            self.set_error_message(
                                "Cannot move notebook into its own descendant".to_string(),
                            );
                            return false;
                        }

                        // Get current notebook
                        if let Some(notebook) =
                            self.snippet_database.notebooks.get(&notebook_id).cloned()
                        {
                            // Remove from current parent's children list
                            if let Some(parent_id) = notebook.parent_id {
                                if let Some(parent) =
                                    self.snippet_database.notebooks.get_mut(&parent_id)
                                {
                                    parent.children.retain(|id| *id != notebook_id);
                                    parent.updated_at = chrono::Utc::now();
                                }
                            } else {
                                // If it's a root notebook, remove from root list
                                self.snippet_database
                                    .root_notebooks
                                    .retain(|id| *id != notebook_id);
                            }

                            // Update notebook's parent
                            if let Some(notebook_to_update) =
                                self.snippet_database.notebooks.get_mut(&notebook_id)
                            {
                                notebook_to_update.parent_id = Some(target_id);
                                notebook_to_update.updated_at = chrono::Utc::now();
                            }

                            // Add to new parent's children list
                            if let Some(new_parent) =
                                self.snippet_database.notebooks.get_mut(&target_id)
                            {
                                if !new_parent.children.contains(&notebook_id) {
                                    new_parent.children.push(notebook_id);
                                    new_parent.updated_at = chrono::Utc::now();
                                }
                            }

                            // Save to make persistent
                            let _ = self.save_database();
                            self.refresh_tree_items();
                            self.needs_redraw = true;
                            self.set_success_message("Notebook moved down one level".to_string());
                            return true;
                        }
                    }
                    _ => {
                        self.set_error_message("Hover over a notebook to move into it".to_string());
                        return false;
                    }
                }
            } else {
                // If no item is hovered, try to find the first visible notebook in the tree
                // that isn't the selected notebook or its parent
                for (idx, item) in self.tree_items.iter().enumerate() {
                    if let TreeItem::Notebook(target_id, _) = item {
                        // Skip self
                        if *target_id == notebook_id {
                            continue;
                        }

                        // Can't move into descendants
                        if self.is_descendant_of(target_id, &notebook_id) {
                            continue;
                        }

                        // Set this as hovered and try again
                        self.hovered_tree_item = Some(idx);
                        return self.move_notebook_down();
                    }
                }

                self.set_error_message("No suitable notebook found to move into".to_string());
                return false;
            }
        } else if let Some(TreeItem::Snippet(snippet_id, _)) = self.get_selected_item().cloned() {
            // First check if a notebook is hovered - this takes priority
            if let Some(hovered_item) = self.get_hovered_item().cloned() {
                match hovered_item {
                    TreeItem::Notebook(target_id, _) => {
                        // Verify we're not trying to move to the same notebook
                        if let Some(snippet) = self.snippet_database.snippets.get(&snippet_id) {
                            if snippet.notebook_id == target_id {
                                self.set_error_message(
                                    "Snippet is already in this notebook".to_string(),
                                );
                                return false;
                            }

                            // Store the current notebook for potential future reference
                            let current_notebook_id = snippet.notebook_id;

                            // Check for moving to a child of current notebook
                            // This is specifically for returning a snippet to a nested folder it came from
                            let is_nested_move = if let Some(target_notebook) =
                                self.snippet_database.notebooks.get(&target_id)
                            {
                                // Is the target a descendant of the current notebook?
                                if target_notebook.parent_id == Some(current_notebook_id) {
                                    true
                                } else {
                                    false
                                }
                            } else {
                                false
                            };

                            // Move snippet to this notebook
                            if let Some(snippet_to_update) =
                                self.snippet_database.snippets.get_mut(&snippet_id)
                            {
                                snippet_to_update.notebook_id = target_id;
                                snippet_to_update.updated_at = chrono::Utc::now();

                                // Save to make persistent
                                let _ = self.save_database();
                                self.refresh_tree_items();
                                self.needs_redraw = true;

                                if is_nested_move {
                                    self.set_success_message(
                                        "Snippet moved back to nested notebook".to_string(),
                                    );
                                } else {
                                    self.set_success_message(
                                        "Snippet moved to selected notebook".to_string(),
                                    );
                                }
                                return true;
                            }
                        }
                    }
                    _ => {
                        // Nothing to do - we'll fall back to checking for child notebooks
                    }
                }
            }

            // No notebook hovered - try to find child notebooks of the current notebook
            if let Some(snippet) = self.snippet_database.snippets.get(&snippet_id) {
                let current_notebook_id = snippet.notebook_id;

                // Find child notebooks of the current notebook
                if let Some(current_notebook) =
                    self.snippet_database.notebooks.get(&current_notebook_id)
                {
                    // If current notebook has children, move to first child
                    if !current_notebook.children.is_empty() {
                        let first_child_id = current_notebook.children[0];

                        // Move snippet to this child notebook
                        if let Some(snippet_to_update) =
                            self.snippet_database.snippets.get_mut(&snippet_id)
                        {
                            snippet_to_update.notebook_id = first_child_id;
                            snippet_to_update.updated_at = chrono::Utc::now();

                            // Save to make persistent
                            let _ = self.save_database();

                            // Set hovered state to show destination
                            if let Some(index) = self.tree_items.iter().position(|item| {
                                if let TreeItem::Notebook(id, _) = item {
                                    *id == first_child_id
                                } else {
                                    false
                                }
                            }) {
                                self.hovered_tree_item = Some(index);
                            }

                            // Update the tree view
                            self.refresh_tree_items();
                            self.needs_redraw = true;

                            self.set_success_message("Snippet moved to child notebook".to_string());
                            return true;
                        }
                    } else {
                        // Try to find any other notebook to move to
                        for (idx, item) in self.tree_items.iter().enumerate() {
                            if let TreeItem::Notebook(target_id, _) = item {
                                // Skip the current notebook
                                if *target_id == current_notebook_id {
                                    continue;
                                }

                                // Set this as hovered and try again
                                self.hovered_tree_item = Some(idx);
                                return self.move_notebook_down();
                            }
                        }

                        self.set_error_message(
                            "Current notebook has no child notebooks".to_string(),
                        );
                        return false;
                    }
                }
            }
        }

        self.set_error_message("Unable to move item down".to_string());
        false
    }

    // Helper to check if a notebook is a descendant of another
    fn is_descendant_of(&self, potential_descendant: &Uuid, ancestor: &Uuid) -> bool {
        if let Some(notebook) = self.snippet_database.notebooks.get(potential_descendant) {
            if let Some(parent_id) = notebook.parent_id {
                if parent_id == *ancestor {
                    return true;
                }
                return self.is_descendant_of(&parent_id, ancestor);
            }
        }
        false
    }

    // Move an item to the next sibling notebook (right)
    pub fn move_item_to_next_sibling(&mut self) -> bool {
        if let Some(TreeItem::Snippet(snippet_id, _)) = self.get_selected_item().cloned() {
            // First, find the current parent notebook
            if let Some(snippet) = self.snippet_database.snippets.get(&snippet_id).cloned() {
                let current_parent_id = snippet.notebook_id;

                // Find the parent of the parent (grandparent)
                if let Some(parent) = self
                    .snippet_database
                    .notebooks
                    .get(&current_parent_id)
                    .cloned()
                {
                    if let Some(grandparent_id) = parent.parent_id {
                        // Find the grandparent to get list of siblings
                        if let Some(grandparent) =
                            self.snippet_database.notebooks.get(&grandparent_id)
                        {
                            // Get siblings (children of grandparent)
                            let siblings = &grandparent.children;

                            // Find index of current parent in siblings
                            if let Some(index) =
                                siblings.iter().position(|id| *id == current_parent_id)
                            {
                                // Get next sibling (or wrap around to first)
                                let next_index = (index + 1) % siblings.len();
                                let next_sibling_id = siblings[next_index];

                                // Move snippet to the next sibling
                                if let Some(snippet_to_update) =
                                    self.snippet_database.snippets.get_mut(&snippet_id)
                                {
                                    snippet_to_update.notebook_id = next_sibling_id;
                                    snippet_to_update.updated_at = chrono::Utc::now();

                                    // Save to make persistent
                                    let _ = self.save_database();

                                    // Update the tree view
                                    self.refresh_tree_items();
                                    self.needs_redraw = true;

                                    // Set hovered state to show destination
                                    if let Some(index) = self.tree_items.iter().position(|item| {
                                        if let TreeItem::Notebook(id, _) = item {
                                            *id == next_sibling_id
                                        } else {
                                            false
                                        }
                                    }) {
                                        self.hovered_tree_item = Some(index);
                                    }

                                    self.set_success_message(format!(
                                        "Moved snippet to next sibling notebook"
                                    ));
                                    return true;
                                }
                            }
                        }
                    } else {
                        // Parent is a root notebook, find the next root notebook
                        let root_notebooks = &self.snippet_database.root_notebooks;
                        if !root_notebooks.is_empty() {
                            if let Some(index) = root_notebooks
                                .iter()
                                .position(|id| *id == current_parent_id)
                            {
                                // Get next root notebook (or wrap around to first)
                                let next_index = (index + 1) % root_notebooks.len();
                                let next_root_id = root_notebooks[next_index];

                                // Move snippet to the next root notebook
                                if let Some(snippet_to_update) =
                                    self.snippet_database.snippets.get_mut(&snippet_id)
                                {
                                    snippet_to_update.notebook_id = next_root_id;
                                    snippet_to_update.updated_at = chrono::Utc::now();

                                    // Save to make persistent
                                    let _ = self.save_database();

                                    // Update the tree view
                                    self.refresh_tree_items();
                                    self.needs_redraw = true;

                                    // Set hovered state to show destination
                                    if let Some(index) = self.tree_items.iter().position(|item| {
                                        if let TreeItem::Notebook(id, _) = item {
                                            *id == next_root_id
                                        } else {
                                            false
                                        }
                                    }) {
                                        self.hovered_tree_item = Some(index);
                                    }

                                    self.set_success_message(format!(
                                        "Moved snippet to next root notebook"
                                    ));
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
            self.set_error_message(
                "Cannot find a suitable sibling notebook to move to".to_string(),
            );
            return false;
        } else if let Some(TreeItem::Notebook(notebook_id, _)) = self.get_selected_item().cloned() {
            // Similar logic for notebooks
            if let Some(notebook) = self.snippet_database.notebooks.get(&notebook_id).cloned() {
                if let Some(parent_id) = notebook.parent_id {
                    // Find the parent to get list of siblings
                    if let Some(parent) = self.snippet_database.notebooks.get(&parent_id) {
                        // Get siblings (children of parent)
                        let siblings = &parent.children;

                        // Find index of current notebook in siblings
                        if let Some(index) = siblings.iter().position(|id| *id == notebook_id) {
                            // Get next sibling (or wrap around to first)
                            let next_index = (index + 1) % siblings.len();
                            let next_sibling_id = siblings[next_index];

                            // Check that we're not trying to move to self
                            if next_sibling_id == notebook_id {
                                self.set_error_message("No other siblings available".to_string());
                                return false;
                            }

                            // Move this notebook after the next sibling in the parent's children array
                            if let Some(parent_update) =
                                self.snippet_database.notebooks.get_mut(&parent_id)
                            {
                                parent_update.children.retain(|id| *id != notebook_id);

                                // Find the new index of the sibling (after removal)
                                if let Some(new_pos) = parent_update
                                    .children
                                    .iter()
                                    .position(|id| *id == next_sibling_id)
                                {
                                    parent_update.children.insert(new_pos + 1, notebook_id);
                                } else {
                                    parent_update.children.push(notebook_id);
                                }

                                parent_update.updated_at = chrono::Utc::now();

                                // Save to make persistent
                                let _ = self.save_database();

                                // Update the tree view
                                self.refresh_tree_items();
                                self.needs_redraw = true;

                                self.set_success_message(format!(
                                    "Moved notebook to next position"
                                ));
                                return true;
                            }
                        }
                    }
                } else {
                    // This is a root notebook, move to next position in root list
                    let root_notebooks = &self.snippet_database.root_notebooks;
                    if !root_notebooks.is_empty() {
                        if let Some(index) = root_notebooks.iter().position(|id| *id == notebook_id)
                        {
                            // Get next root position (or wrap around)
                            let next_index = (index + 1) % root_notebooks.len();

                            // Check if there's only one root notebook
                            if next_index == index {
                                self.set_error_message(
                                    "No other root notebooks available".to_string(),
                                );
                                return false;
                            }

                            // Move this notebook in the root notebooks array
                            let mut new_roots = self.snippet_database.root_notebooks.clone();
                            new_roots.remove(index);
                            new_roots.insert(next_index, notebook_id);
                            self.snippet_database.root_notebooks = new_roots;

                            // Save to make persistent
                            let _ = self.save_database();

                            // Update the tree view
                            self.refresh_tree_items();
                            self.needs_redraw = true;

                            self.set_success_message(format!(
                                "Moved notebook to next root position"
                            ));
                            return true;
                        }
                    }
                }
            }
            self.set_error_message("Cannot find a suitable position to move to".to_string());
            return false;
        }

        self.set_error_message("Select a notebook or snippet to move".to_string());
        false
    }

    // Move an item to the previous sibling notebook (left)
    pub fn move_item_to_prev_sibling(&mut self) -> bool {
        if let Some(TreeItem::Snippet(snippet_id, _)) = self.get_selected_item().cloned() {
            // First, find the current parent notebook
            if let Some(snippet) = self.snippet_database.snippets.get(&snippet_id).cloned() {
                let current_parent_id = snippet.notebook_id;

                // Find the parent of the parent (grandparent)
                if let Some(parent) = self
                    .snippet_database
                    .notebooks
                    .get(&current_parent_id)
                    .cloned()
                {
                    if let Some(grandparent_id) = parent.parent_id {
                        // Find the grandparent to get list of siblings
                        if let Some(grandparent) =
                            self.snippet_database.notebooks.get(&grandparent_id)
                        {
                            // Get siblings (children of grandparent)
                            let siblings = &grandparent.children;

                            // Find index of current parent in siblings
                            if let Some(index) =
                                siblings.iter().position(|id| *id == current_parent_id)
                            {
                                // Get previous sibling (or wrap around to last)
                                let prev_index = if index == 0 {
                                    siblings.len() - 1
                                } else {
                                    index - 1
                                };
                                let prev_sibling_id = siblings[prev_index];

                                // Move snippet to the previous sibling
                                if let Some(snippet_to_update) =
                                    self.snippet_database.snippets.get_mut(&snippet_id)
                                {
                                    snippet_to_update.notebook_id = prev_sibling_id;
                                    snippet_to_update.updated_at = chrono::Utc::now();

                                    // Save to make persistent
                                    let _ = self.save_database();
                                    self.refresh_tree_items();
                                    self.needs_redraw = true;

                                    // Set hovered state to show destination
                                    if let Some(index) = self.tree_items.iter().position(|item| {
                                        if let TreeItem::Notebook(id, _) = item {
                                            *id == prev_sibling_id
                                        } else {
                                            false
                                        }
                                    }) {
                                        self.hovered_tree_item = Some(index);
                                    }

                                    self.set_success_message(format!(
                                        "Moved snippet to previous sibling notebook"
                                    ));
                                    return true;
                                }
                            }
                        }
                    } else {
                        // Parent is a root notebook, find the previous root notebook
                        let root_notebooks = &self.snippet_database.root_notebooks;
                        if !root_notebooks.is_empty() {
                            if let Some(index) = root_notebooks
                                .iter()
                                .position(|id| *id == current_parent_id)
                            {
                                // Get previous root notebook (or wrap around to last)
                                let prev_index = if index == 0 {
                                    root_notebooks.len() - 1
                                } else {
                                    index - 1
                                };
                                let prev_root_id = root_notebooks[prev_index];

                                // Move snippet to the previous root notebook
                                if let Some(snippet_to_update) =
                                    self.snippet_database.snippets.get_mut(&snippet_id)
                                {
                                    snippet_to_update.notebook_id = prev_root_id;
                                    snippet_to_update.updated_at = chrono::Utc::now();

                                    // Save to make persistent
                                    let _ = self.save_database();

                                    // Update the tree view
                                    self.refresh_tree_items();
                                    self.needs_redraw = true;

                                    // Set hovered state to show destination
                                    if let Some(index) = self.tree_items.iter().position(|item| {
                                        if let TreeItem::Notebook(id, _) = item {
                                            *id == prev_root_id
                                        } else {
                                            false
                                        }
                                    }) {
                                        self.hovered_tree_item = Some(index);
                                    }

                                    self.set_success_message(format!(
                                        "Moved snippet to previous root notebook"
                                    ));
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
            self.set_error_message(
                "Cannot find a suitable sibling notebook to move to".to_string(),
            );
            return false;
        } else if let Some(TreeItem::Notebook(notebook_id, _)) = self.get_selected_item().cloned() {
            // Similar logic for notebooks
            if let Some(notebook) = self.snippet_database.notebooks.get(&notebook_id).cloned() {
                if let Some(parent_id) = notebook.parent_id {
                    // Find the parent to get list of siblings
                    if let Some(parent) = self.snippet_database.notebooks.get(&parent_id) {
                        // Get siblings (children of parent)
                        let siblings = &parent.children;

                        // Find index of current notebook in siblings
                        if let Some(index) = siblings.iter().position(|id| *id == notebook_id) {
                            // Get previous sibling (or wrap around to last)
                            let prev_index = if index == 0 {
                                siblings.len() - 1
                            } else {
                                index - 1
                            };
                            let prev_sibling_id = siblings[prev_index];

                            // Check that we're not trying to move to self
                            if prev_sibling_id == notebook_id {
                                self.set_error_message("No other siblings available".to_string());
                                return false;
                            }

                            // Move this notebook before the previous sibling in the parent's children array
                            if let Some(parent_update) =
                                self.snippet_database.notebooks.get_mut(&parent_id)
                            {
                                parent_update.children.retain(|id| *id != notebook_id);

                                // Find the new index of the sibling (after removal)
                                if let Some(new_pos) = parent_update
                                    .children
                                    .iter()
                                    .position(|id| *id == prev_sibling_id)
                                {
                                    parent_update.children.insert(new_pos, notebook_id);
                                } else {
                                    parent_update.children.insert(0, notebook_id);
                                }

                                parent_update.updated_at = chrono::Utc::now();

                                // Save to make persistent
                                let _ = self.save_database();
                                self.refresh_tree_items();
                                self.needs_redraw = true;

                                self.set_success_message(format!(
                                    "Moved notebook to previous position"
                                ));
                                return true;
                            }
                        }
                    }
                } else {
                    // This is a root notebook, move to previous position in root list
                    let root_notebooks = &self.snippet_database.root_notebooks;
                    if !root_notebooks.is_empty() {
                        if let Some(index) = root_notebooks.iter().position(|id| *id == notebook_id)
                        {
                            // Get previous root position (or wrap around)
                            let prev_index = if index == 0 {
                                root_notebooks.len() - 1
                            } else {
                                index - 1
                            };

                            if prev_index == index {
                                self.set_error_message(
                                    "No other root notebooks available".to_string(),
                                );
                                return false;
                            }

                            // Move this notebook in the root notebooks array
                            let mut new_roots = self.snippet_database.root_notebooks.clone();
                            new_roots.remove(index);
                            new_roots.insert(prev_index, notebook_id);
                            self.snippet_database.root_notebooks = new_roots;

                            let _ = self.save_database();
                            self.refresh_tree_items();
                            self.needs_redraw = true;

                            self.set_success_message(format!(
                                "Moved notebook to previous root position"
                            ));
                            return true;
                        }
                    }
                }
            }
            self.set_error_message("Cannot find a suitable position to move to".to_string());
            return false;
        }

        self.set_error_message("Select a notebook or snippet to move".to_string());
        false
    }

    // Add these methods to handle confirmation states
    /// Request confirmation for deleting an item
    pub fn request_delete_confirmation(&mut self, item_id: Uuid, is_notebook: bool) {
        self.confirmation_state = ConfirmationState::DeleteItem {
            item_id,
            is_notebook,
        };
        self.clear_messages();

        // Set message based on item type
        if is_notebook {
            if let Some(notebook) = self.snippet_database.notebooks.get(&item_id) {
                self.set_success_message(format!(
                    "Are you sure you want to delete notebook '{}'?",
                    notebook.name
                ));
            }
        } else {
            if let Some(snippet) = self.snippet_database.snippets.get(&item_id) {
                self.set_success_message(format!(
                    "Are you sure you want to delete snippet '{}'?",
                    snippet.title
                ));
            }
        }
    }

    /// Confirms the pending action and executes it
    pub fn confirm_pending_action(&mut self) -> bool {
        // Take ownership of the confirmation state
        let current_state =
            std::mem::replace(&mut self.confirmation_state, ConfirmationState::None);

        match current_state {
            ConfirmationState::DeleteItem {
                item_id,
                is_notebook,
            } => {
                self.clear_messages();

                if is_notebook {
                    if let Err(e) = self.delete_notebook(item_id) {
                        self.set_error_message(e);
                    } else {
                        self.set_success_message("Notebook deleted successfully".to_string());
                        self.code_snippets_state = CodeSnippetsState::NotebookList;
                    }
                } else {
                    if let Err(e) = self.delete_snippet(item_id) {
                        self.set_error_message(e);
                    } else {
                        self.set_success_message("Snippet deleted successfully".to_string());
                    }
                }

                self.refresh_tree_items();
                true
            }
            ConfirmationState::_MoveItem {
                item_id,
                is_notebook,
                target_id,
            } => {
                if is_notebook {
                    // Logic for moving notebooks
                    // TODO: Implement this someday
                    // For now, just set an error message
                    self.set_error_message("Moving notebooks not implemented yet".to_string());
                } else {
                    if let Some(snippet) = self.snippet_database.snippets.get_mut(&item_id) {
                        snippet.notebook_id = target_id;

                        if let Err(e) = self.save_database() {
                            self.set_error_message(e);
                        } else {
                            self.set_success_message("Snippet moved successfully".to_string());
                        }
                    }
                }

                self.refresh_tree_items();
                true
            }
            ConfirmationState::Custom { action } => {
                action(self);
                true
            }
            ConfirmationState::None => false,
        }
    }

    pub fn cancel_pending_action(&mut self) {
        self.confirmation_state = ConfirmationState::None;
        self.clear_messages();
    }

    pub fn has_pending_action(&self) -> bool {
        !matches!(self.confirmation_state, ConfirmationState::None)
    }

    pub fn perform_search(&mut self, query: &str) -> usize {
        crate::search::perform_search(self, query)
    }

    pub fn open_selected_search_result(&mut self) -> bool {
        crate::search::open_selected_search_result(self)
    }

    pub fn toggle_favorite_snippet(&mut self, snippet_id: Uuid) -> Result<(), String> {
        let is_favorited = {
            if let Some(snippet) = self.snippet_database.snippets.get_mut(&snippet_id) {
                snippet.toggle_favorite();
                snippet.is_favorited()
            } else {
                return Err("Snippet not found".to_string());
            }
        };

        self.save_database()?;

        self.set_success_message(format!(
            "Snippet {} as favorite",
            if is_favorited { "marked" } else { "unmarked" }
        ));

        Ok(())
    }

    pub fn set_pending_action<F>(&mut self, message: String, action: Box<F>)
    where
        F: FnOnce(&mut App) + 'static,
    {
        self.error_message = None;
        self.success_message = None;

        self.error_message = Some(format!("{} (Enter: Confirm, Esc: Cancel)", message));
        self.confirmation_state = ConfirmationState::Custom { action };
    }
}

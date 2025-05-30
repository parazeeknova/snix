//! Application State Management Module
//!
//! This module contains the core application state management logic for RustUI.
//! It defines the main application states, handles navigation between different
//! pages, manages user interface state, and controls the overall flow of the
//! application.
//!
//! The module provides a clean separation between UI rendering and state management,
//! making it easy to add new pages and features while maintaining a consistent
//! navigation experience.

use crate::ui::{components, start_page};
use ratatui::Frame;

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

/// Main Application State Container
///
/// This struct holds all the state information needed to run the application.
/// It tracks the current page, menu selection, and navigation history to provide
/// a smooth user experience with proper back navigation.
///
/// The App struct is the central hub for all state management and is passed
/// to rendering functions to determine what content to display and how to
/// style interactive elements based on the current state.
///
/// # Fields
///
/// - `state`: Current page/screen being displayed
/// - `selected_menu_item`: Index of currently highlighted menu item (0-based)
/// - `page_history`: Stack of previously visited pages for back navigation
#[derive(Debug)]
pub struct App {
    pub state: AppState,
    pub selected_menu_item: usize,
    pub page_history: Vec<AppState>,
}

impl App {
    /// Creates a new instance of the application with default initial state
    ///
    /// Initializes the application in the StartPage state with the first menu item
    /// selected and an empty navigation history. This provides a clean starting
    /// point for the user interface.
    ///
    /// # Returns
    ///
    /// A new `App` instance ready to be used for rendering and event handling.
    pub fn new() -> Self {
        Self {
            state: AppState::StartPage,
            selected_menu_item: 0,
            page_history: vec![AppState::StartPage],
        }
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
    ///
    /// # Parameters
    ///
    /// - `new_state`: The application state to navigate to
    pub fn navigate_to(&mut self, new_state: AppState) {
        if self.state != new_state {
            self.page_history.push(self.state.clone());
            self.state = new_state;
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
    ///
    /// # Returns
    ///
    /// - `true` if back navigation is available
    /// - `false` if the user is at the root of the navigation stack
    pub fn can_go_back(&self) -> bool {
        !self.page_history.is_empty()
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
    ///
    /// # Parameters
    ///
    /// - `frame`: The ratatui Frame object to render to
    pub fn render(&self, frame: &mut Frame) {
        match self.state {
            AppState::StartPage => start_page::render(frame, self),
            AppState::Boilerplates => {
                components::render_wip_dialog(frame, frame.area(), "📦 Boilerplates", self)
            }
            AppState::Marketplace => {
                components::render_wip_dialog(frame, frame.area(), "🛒 Marketplace", self)
            }
            AppState::CodeSnippets => {
                components::render_wip_dialog(frame, frame.area(), "📝 Code Snippets", self)
            }
            AppState::InfoPage => {
                components::render_wip_dialog(frame, frame.area(), "ℹ️ About", self)
            }
            AppState::Settings => {
                components::render_wip_dialog(frame, frame.area(), "⚙️ Settings", self)
            }
        }
    }
}

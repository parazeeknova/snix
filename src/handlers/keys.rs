//! Keyboard Input Handling Module
//!
//! This module provides comprehensive keyboard input handling for the RustUI application.
//! It processes all user keyboard interactions and translates them into appropriate
//! application state changes, navigation actions, and menu interactions.
//!
//! The module supports both standard navigation keys (arrows, Enter, Esc) and convenient
//! single-letter shortcuts for quick access to different application sections. It also
//! handles global actions like quitting the application and back navigation.

use crate::app::{App, AppState};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

/// Main keyboard event handler and dispatcher
///
/// This is the primary entry point for all keyboard input processing. It receives
/// key events from the terminal and routes them to appropriate specialized handlers
/// based on the current application state.
///
/// The function implements a hierarchical key handling system:
/// 1. First, it checks for global keys that work on any page (quit, back)
/// 2. Then, it dispatches to state-specific handlers based on current page
///
/// # Parameters
///
/// - `key`: The keyboard event captured from the terminal
/// - `app`: Mutable reference to the application state for modifications
///
/// # Returns
///
/// - `true` if the application should exit
/// - `false` if the application should continue running

pub fn handle_key_events(key: KeyEvent, app: &mut App) -> bool {
    match key.code {
        // Global quit command - works from any page
        KeyCode::Char('q') | KeyCode::Char('Q') => return true,

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
            _ => handle_other_page_keys(key, app),
        },
    }
}

/// Handles keyboard input specifically for the start page (main menu)
///
/// This function processes all keyboard interactions when the user is on the main
/// start page. It handles menu navigation (up/down movement), item selection,
/// and provides convenient single-letter shortcuts for quick navigation to
/// specific sections of the application.
///
/// The start page has a 6-item menu (indices 0-5):
/// 0. Boilerplates (shortcut: 'b')
/// 1. Marketplace (shortcut: 'm')
/// 2. Code Snippets (shortcut: 's')
/// 3. Info/About (shortcut: 'i')
/// 4. Settings (shortcut: 'c')
/// 5. Exit (Enter to quit)
///
/// # Parameters
///
/// - `key`: The keyboard event to process
/// - `app`: Mutable reference to application state for menu and navigation updates
///
/// # Returns
///
/// - `true` if the user selected "Exit" (menu item 5)
/// - `false` for all other actions (navigation, menu selection, etc.)
///
/// # Key Mappings
///
/// - Arrow keys and vi-style navigation (j/k) for menu movement
/// - Enter to activate the currently selected menu item
/// - Single letter shortcuts for direct navigation to specific pages
/// - All shortcuts work with both lowercase and uppercase letters
fn handle_start_page_keys(key: KeyEvent, app: &mut App) -> bool {
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
///
/// The function provides two ways to navigate away from the current page:
/// 1. Standard back navigation (Esc) - returns to the previous page in history
/// 2. Direct home navigation (h/H) - jumps directly to the start page, clearing history
///
/// # Parameters
///
/// - `key`: The keyboard event to process
/// - `app`: Mutable reference to application state for navigation updates
///
/// # Returns
///
/// Always returns `false` since these pages don't have exit functionality.
/// Users must return to the start page to access the exit option.
///
/// # Navigation Behavior
///
/// - `Esc`: Standard back navigation using the page history stack
/// - `h/H`: Emergency home navigation that clears all history and returns to start page
/// - All other keys are ignored on these pages
fn handle_other_page_keys(key: KeyEvent, app: &mut App) -> bool {
    match key.code {
        // Standard back navigation - uses the navigation history stack
        KeyCode::Esc => {
            if app.can_go_back() {
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

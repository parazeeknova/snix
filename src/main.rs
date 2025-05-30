//! snix - Template & Boilerplate Manager
//!
//! A terminal-based user interface application for managing development boilerplates,
//! project templates, and code snippets. Built with Rust and ratatui for a fast,
//! efficient, and beautiful terminal experience.
//!
//! snix provides developers with a centralized tool to:
//! - Manage project templates and boilerplates for various tech stacks
//! - Browse and download community-created templates from a marketplace
//! - Store and organize frequently used code snippets
//! - Configure development workflow preferences

use color_eyre::eyre::Result;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event},
};

mod app;
mod handlers;
mod ui;

use app::App;
use handlers::keys::handle_key_events;

/// Application entry point and initialization
///
/// This function serves as the main entry point for the RustUI application. It handles
/// the complete application lifecycle from startup to shutdown.
///
/// The function uses color-eyre for enhanced error reporting, which provides beautiful
/// stack traces and helpful debugging information in case of panics or errors.
fn main() -> Result<()> {
    println!("🔨 Starting snix - Template & Boilerplate Manager!");
    println!("Created by parazeeknova");

    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();

    result
}

/// Main application event loop and terminal management
///
/// This function contains the core application logic that drives the entire user interface.
/// It manages the application state, handles the continuous render-update cycle, and
/// processes user input events in a responsive manner.
fn run(mut terminal: DefaultTerminal) -> Result<()> {
    let mut app = App::new();
    // Main event loop - continues until user requests exit
    loop {
        terminal.draw(|frame| app.render(frame))?;

        if let Event::Key(key) = event::read()? {
            if handle_key_events(key, &mut app) {
                break;
            }
        }
    }
    Ok(())
}

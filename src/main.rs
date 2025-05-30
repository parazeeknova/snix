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

use crate::app::App;
use color_eyre::Result;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    crossterm::{
        event::{self, Event},
        event::{DisableMouseCapture, EnableMouseCapture},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
};
use std::error::Error;
use std::io::{self};
use std::time::Duration;

mod app;
mod handlers;
mod models;
mod ui;

/// Application entry point and initialization
/// This function initializes the terminal, sets up event handling, and
/// runs the main application loop. It ensures proper terminal cleanup
/// even if the application panics.
fn main() -> Result<(), Box<dyn Error>> {
    println!("Starting snix - Template & Boilerplate Manager");
    println!("Created by parazeeknova");

    color_eyre::install()?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let mut should_quit = false;

    while !should_quit {
        if app.needs_redraw {
            force_redraw(&mut terminal, &app)?;
            app.needs_redraw = false;
        } else {
            terminal.draw(|frame| app.render(frame))?;
        }
        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                should_quit = handlers::keys::handle_key_events(key, &mut app);
                if app.needs_redraw {
                    force_redraw(&mut terminal, &app)?;
                    app.needs_redraw = false;
                }
            }
        }
        app._tick();
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    println!("Thanks for using snix! Goodbye!");

    Ok(())
}

/// Forces a complete redraw of the terminal UI
/// Used after suspending for editor to ensure a clean UI state
fn force_redraw<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &App,
) -> Result<(), Box<dyn Error>> {
    terminal.clear()?;
    use ratatui::crossterm::{
        execute,
        terminal::{Clear, ClearType},
    };
    use std::io::stdout;

    execute!(stdout(), Clear(ClearType::All))?;
    terminal.draw(|frame| app.render(frame))?;
    terminal.draw(|frame| app.render(frame))?;

    Ok(())
}

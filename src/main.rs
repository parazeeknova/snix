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
        cursor::Show,
        event::{self, Event},
        event::{DisableMouseCapture, EnableMouseCapture},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
};
use std::error::Error;
use std::io::{self};
use std::panic;
use std::time::Duration;

mod app;
mod handlers;
mod models;
mod search;
mod ui;

use handlers::keys::handle_key_events;

/// Main entry point for the application
/// Sets up the terminal, runs the application loop, and ensures clean exit
/// even if the application panics.
fn main() -> Result<(), Box<dyn Error>> {
    // Set up panic hook
    panic::set_hook(Box::new(|info| {
        let _ = cleanup_terminal();
        eprintln!("Panic occurred: {:?}", info);
    }));

    // Set up the terminal
    let mut terminal = setup_terminal()?;

    // Run the application
    let result = run_app(&mut terminal);

    // Clean up the terminal
    cleanup_terminal()?;

    // Handle the result
    if let Err(err) = result {
        eprintln!("Error: {}", err);
        return Err(err);
    }

    Ok(())
}

/// Sets up the terminal for the TUI application
fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>, Box<dyn Error>> {
    println!("Starting snix - Template & Boilerplate Manager");
    println!("Created by parazeeknova");

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    Ok(terminal)
}

/// Runs the main application loop
fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<(), Box<dyn Error>> {
    let mut app = App::new();
    let mut should_quit = false;

    while !should_quit {
        if app.needs_redraw {
            force_redraw(terminal, &mut app)?;
            app.needs_redraw = false;
        } else {
            terminal.draw(|frame| app.render(frame))?;
        }
        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                should_quit = handle_key_events(key, &mut app);

                if app.needs_redraw {
                    force_redraw(terminal, &mut app)?;
                    app.needs_redraw = false;
                }
            }
        }
        app._tick();
    }

    Ok(())
}

/// Forces a complete redraw of the terminal UI
/// Used after suspending for editor to ensure a clean UI state
fn force_redraw<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
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

/// Cleans up the terminal state when the application exits
fn cleanup_terminal() -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;
    execute!(stdout, Show)?;

    println!("Thanks for using snix! Goodbye!");

    Ok(())
}

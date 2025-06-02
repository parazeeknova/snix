//! CLI Module for Snix
//! This module provides command-line interface functionality for Snix,
//! allowing users to interact with their snippet database directly from
//! the terminal without launching the full TUI application.

pub mod commands;
pub mod tree;

use crate::models::StorageManager;
use colored::Colorize;
use std::error::Error;

/// Executes CLI commands based on the provided arguments
pub fn execute_cli(args: &[String]) -> Result<(), Box<dyn Error>> {
    if args.is_empty() {
        // No arguments provided, show help
        print_help();
        return Ok(());
    }

    match args[0].as_str() {
        "list" | "ls" => {
            let storage = StorageManager::new()?;
            let database = storage.load_database()?;

            if args.len() == 1 {
                tree::display_tree(&database, None)?;
                return Ok(());
            }

            // Try to find notebook by name or ID
            let notebook_id = if let Ok(id) = uuid::Uuid::parse_str(&args[1]) {
                // Valid UUID format, use directly
                Some(id)
            } else {
                // Try to find by name
                match tree::find_notebook_by_name(&database, &args[1]) {
                    Some(id) => Some(id),
                    None => {
                        println!(
                            "{}  No notebook found with name: {}",
                            "┃".bright_magenta(),
                            args[1]
                        );

                        // Show available notebooks
                        tree::list_all_notebooks(&database)?;
                        return Ok(());
                    }
                }
            };

            // Display tree view of notebooks and snippets
            tree::display_tree(&database, notebook_id)?;
        }
        "notebooks" => {
            // List all available notebooks with their IDs
            let storage = StorageManager::new()?;
            let database = storage.load_database()?;
            tree::list_all_notebooks(&database)?;
        }
        "favorites" | "fav" => {
            commands::list_favorites()?;
        }
        "show" | "view" | "cat" => {
            if args.len() < 2 {
                println!(
                    "{}  Error: Missing snippet name or ID",
                    "┃".bright_magenta()
                );
                println!(
                    "{}  Usage: snix show <SNIPPET_NAME_OR_ID>",
                    "┃".bright_magenta()
                );
                return Ok(());
            }

            commands::show_snippet(&args[1])?;
        }
        "search" | "find" => {
            if args.len() < 2 {
                println!("{}  Error: Missing search query", "┃".bright_magenta());
                println!("{}  Usage: snix search <QUERY>", "┃".bright_magenta());
                return Ok(());
            }

            commands::search_snippets(&args[1])?;
        }
        "help" => {
            print_help();
        }
        _ => {
            println!("{}  Unknown command: {}", "┃".bright_magenta(), args[0]);

            print_help();
        }
    }

    Ok(())
}

/// Prints the help message with available commands
fn print_help() {
    println!(
        "{}  {}",
        "┃".bright_magenta(),
        "SNIX CLI - SNIPPET MANAGER".bold()
    );

    println!("{}  {}", "┃".bright_magenta(), "USAGE:".bright_yellow());
    println!("{}  snix [COMMAND] [ARGS]", "┃".bright_magenta());
    println!("{}  {}", "┃".bright_magenta(), "COMMANDS:".bright_yellow());
    println!(
        "{}  {:<27} {}",
        "┃".bright_magenta(),
        "list, ls".bright_white(),
        "List all notebooks and snippets in tree format"
    );
    println!(
        "{}  {:<27} {}",
        "┃".bright_magenta(),
        "list <NOTEBOOK_NAME>".bright_white(),
        "List snippets in the specified notebook"
    );
    println!(
        "{}  {:<27} {}",
        "┃".bright_magenta(),
        "notebooks".bright_white(),
        "List all notebooks with their IDs"
    );
    println!(
        "{}  {:<27} {}",
        "┃".bright_magenta(),
        "show, view <NAME>".bright_white(),
        "Display a snippet by name (partial name works)"
    );
    println!(
        "{}  {:<27} {}",
        "┃".bright_magenta(),
        "search, find <QUERY>".bright_white(),
        "Search for snippets matching the query"
    );
    println!(
        "{}  {:<27} {}",
        "┃".bright_magenta(),
        "favorites, fav".bright_white(),
        "List all favorite snippets"
    );
    println!(
        "{}  {:<27} {}",
        "┃".bright_magenta(),
        "help".bright_white(),
        "Display this help message"
    );

    println!("{}  {}", "┃".bright_magenta(), "TIP:".bright_green());
    println!(
        "{}  Run with no arguments to launch the full TUI (Terminal User Interface) mode",
        "┃".bright_magenta()
    );
}

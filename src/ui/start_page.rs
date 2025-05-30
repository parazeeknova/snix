//! Start Page UI Module
//!
//! This module handles the rendering of the main start page (home screen) for RustUI.
//! The start page serves as the primary navigation hub and landing screen, featuring:

use crate::app::App;
use crate::ui::colors::RosePine;
use crate::ui::components::render_bottom_bar;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, BorderType, List, ListItem, ListState, Paragraph, Widget},
};

/// Main rendering function for the start page
///
/// This is the primary entry point for rendering the start page UI. It orchestrates
/// the layout of all visual elements and creates a cohesive, welcoming interface
/// that serves as the application's main navigation hub.
pub fn render(frame: &mut Frame, app: &App) {
    let main_area = frame.area();

    let block = Block::bordered()
        .title(" BoilerForge - Template & Boilerplate Manager ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(RosePine::HIGHLIGHT_HIGH));

    let inner_area = block.inner(main_area);
    block.render(main_area, frame.buffer_mut());

    let main_chunks = Layout::vertical([
        Constraint::Fill(1),   // Main content area (title + disclaimer + menu)
        Constraint::Length(3), // Description area
        Constraint::Length(3), // Bottom navigation bar
    ])
    .split(inner_area);

    let content_area = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(60),
        Constraint::Fill(1),
    ])
    .split(main_chunks[0])[1];

    let content_chunks = Layout::vertical([
        Constraint::Fill(1),    // Top padding
        Constraint::Length(8),  // Title area
        Constraint::Length(2),  // Disclaimer area
        Constraint::Length(16), // Menu area (increased for padding)
        Constraint::Fill(1),    // Bottom padding
    ])
    .split(content_area);

    render_title(frame, content_chunks[1]);
    render_disclaimer(frame, content_chunks[2]);
    render_menu(frame, content_chunks[3], app);
    render_description(frame, main_chunks[1], app);
    render_bottom_bar(frame, main_chunks[2], app);
}

/// Renders the ASCII art title with elegant typography
fn render_title(frame: &mut Frame, area: Rect) {
    let title_text = create_rustui_ascii_title();

    let title = Paragraph::new(title_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(RosePine::ROSE).bold());

    title.render(area, frame.buffer_mut());
}

/// Generates the ASCII art representation of the application name
fn create_rustui_ascii_title() -> String {
    r#"
██████╗ ██╗   ██╗███████╗████████╗██╗   ██╗██╗
██╔══██╗██║   ██║██╔════╝╚══██╔══╝██║   ██║██║
██████╔╝██║   ██║███████╗   ██║   ██║   ██║██║
██╔══██╗██║   ██║╚════██║   ██║   ██║   ██║██║
██║  ██║╚██████╔╝███████║   ██║   ╚██████╔╝██║
╚═╝  ╚═╝ ╚═════╝ ╚══════╝   ╚═╝    ╚═════╝ ╚═╝
"#
    .to_string()
}

/// Renders the application tagline and disclaimer
fn render_disclaimer(frame: &mut Frame, area: Rect) {
    let disclaimer = "Boilerplate & Code snippets manager ";

    let disclaimer_paragraph = Paragraph::new(disclaimer)
        .alignment(Alignment::Center)
        .style(Style::default().fg(RosePine::SUBTLE).italic());

    disclaimer_paragraph.render(area, frame.buffer_mut());
}

/// Renders the interactive navigation menu with selection highlighting
fn render_menu(frame: &mut Frame, area: Rect, app: &App) {
    let menu_area = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(45),
        Constraint::Fill(1),
    ])
    .split(area)[1];

    let padded_area = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(menu_area)[1];

    let menu_items = vec![
        ("󰈮", "Boilerplates", "b"),
        ("󱣒", "Marketplace", "m"),
        ("", "Code Snippets", "s"),
        ("", "About", "i"),
        ("", "Settings", "c"),
        ("󰈆", "Exit", "q"),
    ];

    let list_items: Vec<ListItem> = menu_items
        .iter()
        .enumerate()
        .map(|(i, (icon, name, key))| {
            let style = if i == app.selected_menu_item {
                Style::default().fg(RosePine::LOVE).bold()
            } else {
                Style::default().fg(RosePine::TEXT)
            };

            let prefix = if i == app.selected_menu_item {
                "▶"
            } else {
                " "
            };

            let content = format!("{} {} {}", prefix, icon, name);
            let shortcut = format!("[{}]", key);

            let line = format!("{:<20} {:>20}", content, shortcut);
            let centered_line = format!("{:^43}", line);

            ListItem::new(centered_line).style(style)
        })
        .collect();

    let mut spaced_items = Vec::new();
    for (i, item) in list_items.into_iter().enumerate() {
        spaced_items.push(item);
        if i < menu_items.len() - 1 {
            spaced_items.push(ListItem::new(""));
        }
    }

    let list = List::new(spaced_items)
        .style(Style::default().fg(RosePine::TEXT))
        .highlight_style(
            Style::default()
                .fg(RosePine::LOVE)
                .bg(RosePine::HIGHLIGHT_LOW)
                .bold(),
        )
        .highlight_symbol("");

    let mut list_state = ListState::default();
    list_state.select(Some(app.selected_menu_item * 2));

    frame.render_stateful_widget(list, padded_area, &mut list_state);
}

/// Renders contextual descriptions for the currently selected menu item
fn render_description(frame: &mut Frame, area: Rect, app: &App) {
    let descriptions = vec![
        "Create, manage and deploy boilerplates for React, Vue, Angular, and more",
        "Discover community templates, frameworks, and starter projects",
        "Quick access to reusable code snippets and development patterns",
        "Learn about RustUI's powerful boilerplate management features",
        "Customize your development workflow and preferences",
        "Save your work and exit the application",
    ];

    let description = descriptions.get(app.selected_menu_item).unwrap_or(&"");

    let description_paragraph = Paragraph::new(*description)
        .alignment(Alignment::Center)
        .style(Style::default().fg(RosePine::MUTED));

    description_paragraph.render(area, frame.buffer_mut());
}

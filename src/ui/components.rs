//! UI Components and Layout Module
//!
//! This module contains all the reusable UI components and layout logic for the RustUI
//! application. It provides functions for rendering complex interface elements like
//! navigation bars, dialogs, and breadcrumb systems with consistent styling and behavior.
//!
//! # Components
//!
//! - **Bottom Navigation Bar**: Breadcrumb navigation and keyboard shortcuts
//! - **Work-in-Progress Dialog**: Centered modal for pages under development
//! - **Breadcrumb System**: Hierarchical navigation showing current location

use crate::app::{App, TreeItem};
use crate::ui::colors::RosePine;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Clear, Paragraph, Widget},
};

/// Renders the bottom navigation bar with breadcrumbs and keyboard shortcuts
///
/// This function creates a comprehensive navigation bar at the bottom of the screen
/// that serves two main purposes:
/// 1. Shows breadcrumb navigation indicating the user's current location in the app
/// 2. Displays available keyboard shortcuts relevant to the current context

pub fn render_bottom_bar(frame: &mut Frame, area: Rect, app: &mut App) {
    let navbar_chunks = Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).split(area);

    let breadcrumbs = get_breadcrumbs_with_symbols(app);

    let left_content = Paragraph::new(breadcrumbs)
        .alignment(Alignment::Left)
        .style(Style::default().fg(RosePine::SUBTLE))
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(RosePine::HIGHLIGHT_HIGH)),
        );

    // Get context-aware shortcuts
    let shortcuts = get_context_shortcuts(app);

    let right_content = Paragraph::new(shortcuts)
        .alignment(Alignment::Right)
        .style(Style::default().fg(RosePine::MUTED))
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(RosePine::HIGHLIGHT_HIGH)),
        );

    left_content.render(navbar_chunks[0], frame.buffer_mut());
    right_content.render(navbar_chunks[1], frame.buffer_mut());
}

fn get_context_shortcuts(app: &mut App) -> String {
    use crate::app::{AppState, InputMode};

    let back_hint = if app.can_go_back() {
        " [←] Back │ "
    } else {
        ""
    };

    match (&app.state, &app.input_mode) {
        (_, InputMode::CreateNotebook | InputMode::CreateSnippet | InputMode::Search) => {
            format!(" [⏎] Confirm │ [Esc] Cancel ")
        }
        (_, InputMode::SelectLanguage) => {
            format!(" [↑↓] Navigate │ [⏎] Select │ [Esc] Cancel ")
        }

        (AppState::StartPage, InputMode::Normal) => {
            format!(
                "{} [↑↓] Navigate │ [⏎] Select │ [b] Boilerplates │ [s] Snippets │ [q] Quit ",
                back_hint
            )
        }

        (AppState::CodeSnippets, InputMode::Normal) => {
            if app.snippet_database.notebooks.is_empty() {
                format!("{} [n] New Notebook │ [h] Home │ [q] Quit ", back_hint)
            } else {
                format!(
                    "{} [n] Root Notebook │ [b] Nested Notebook │ [s] Snippet │ [d] Delete │ [/] Search │ [?] Help",
                    back_hint
                )
            }
        }

        // Other pages
        _ => {
            format!(
                "{} [↑↓] Navigate │ [⏎] Select │ [h] Home │ [q] Quit ",
                back_hint
            )
        }
    }
}

/// Constructs the breadcrumb navigation trail with appropriate styling and symbols
///
/// This function builds a visually rich breadcrumb navigation system that shows the user's
/// current location within the application hierarchy. It uses a combination of symbols,
/// colors, and background highlights to create an intuitive navigation experience.

fn get_breadcrumbs_with_symbols(app: &mut App) -> Line<'static> {
    let mut spans = Vec::new();

    // Always start with Home
    if app.state == crate::app::AppState::StartPage {
        spans.push(Span::styled(
            " 󰋜 Home ",
            Style::default().fg(RosePine::BASE).bg(RosePine::IRIS),
        ));
    } else {
        spans.push(Span::styled(
            " 󰋜 Home ",
            Style::default().fg(RosePine::SUBTLE),
        ));
    }

    // Add the current section (Snippets, Boilerplates, etc.)
    if app.state != crate::app::AppState::StartPage {
        spans.push(Span::styled(" ❯ ", Style::default().fg(RosePine::MUTED)));

        match app.state {
            crate::app::AppState::Boilerplates => {
                spans.push(Span::styled(
                    " Boilerplates ",
                    Style::default().fg(RosePine::BASE).bg(RosePine::IRIS),
                ));
            }
            crate::app::AppState::Marketplace => {
                spans.push(Span::styled(
                    " Marketplace ",
                    Style::default().fg(RosePine::BASE).bg(RosePine::IRIS),
                ));
            }
            crate::app::AppState::CodeSnippets => {
                spans.push(Span::styled(
                    " Snippets ",
                    Style::default().fg(RosePine::BASE).bg(RosePine::IRIS),
                ));

                // Add the full path for the selected item in the tree view
                if let Some(selected_item) = app.get_selected_item() {
                    match selected_item {
                        TreeItem::Notebook(notebook_id, _) => {
                            // Build the full path for the notebook
                            let mut path = Vec::new();
                            let mut current_id = Some(*notebook_id);

                            // Collect all parent notebooks up to the root
                            while let Some(id) = current_id {
                                if let Some(notebook) = app.snippet_database.notebooks.get(&id) {
                                    path.push((id, notebook.name.clone()));
                                    current_id = notebook.parent_id;
                                } else {
                                    break;
                                }
                            }

                            // Display the path in reverse order (from root to current)
                            for (_i, (id, name)) in path.iter().rev().enumerate() {
                                spans.push(Span::styled(
                                    " ❯ ",
                                    Style::default().fg(RosePine::MUTED),
                                ));
                                let style = if *id == *notebook_id {
                                    Style::default().fg(RosePine::BASE).bg(RosePine::LOVE)
                                } else {
                                    Style::default().fg(RosePine::SUBTLE)
                                };

                                spans.push(Span::styled(format!("   {} ", name), style));
                            }
                        }
                        TreeItem::Snippet(snippet_id, _) => {
                            if let Some(snippet) = app.snippet_database.snippets.get(snippet_id) {
                                let notebook_id = snippet.notebook_id;

                                // Build the full path for the notebook containing the snippet
                                let mut path = Vec::new();
                                let mut current_id = Some(notebook_id);

                                // Collect all parent notebooks up to the root
                                while let Some(id) = current_id {
                                    if let Some(notebook) = app.snippet_database.notebooks.get(&id)
                                    {
                                        path.push((id, notebook.name.clone()));
                                        current_id = notebook.parent_id;
                                    } else {
                                        break;
                                    }
                                }

                                // Display the path in reverse order (from root to current)
                                for (_id, name) in path.iter().rev() {
                                    spans.push(Span::styled(
                                        " ❯ ",
                                        Style::default().fg(RosePine::MUTED),
                                    ));
                                    spans.push(Span::styled(
                                        format!("   {} ", name),
                                        Style::default().fg(RosePine::SUBTLE),
                                    ));
                                }

                                // Add the snippet to the breadcrumbs
                                spans.push(Span::styled(
                                    " ❯ ",
                                    Style::default().fg(RosePine::MUTED),
                                ));
                                spans.push(Span::styled(
                                    format!(" {} {} ", snippet.language.icon(), snippet.title),
                                    Style::default().fg(RosePine::BASE).bg(RosePine::GOLD),
                                ));
                            }
                        }
                    }
                }
            }
            crate::app::AppState::InfoPage => {
                spans.push(Span::styled(
                    " Info ",
                    Style::default().fg(RosePine::BASE).bg(RosePine::IRIS),
                ));
            }
            crate::app::AppState::Settings => {
                spans.push(Span::styled(
                    " Settings ",
                    Style::default().fg(RosePine::BASE).bg(RosePine::IRIS),
                ));
            }
            _ => {}
        }
    }

    Line::from(spans)
}

/// Renders a centered work-in-progress dialog for pages under development
pub fn render_wip_dialog(frame: &mut Frame, area: Rect, page_title: &str, app: &mut App) {
    let block = Block::bordered()
        .title(format!(" {} ", page_title))
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(RosePine::HIGHLIGHT_HIGH));

    let inner_area = block.inner(area);
    block.render(area, frame.buffer_mut());

    let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).split(inner_area);

    let dialog_area = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(60),
        Constraint::Fill(1),
    ])
    .split(chunks[0])[1];

    let dialog_vertical = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(12),
        Constraint::Fill(1),
    ])
    .split(dialog_area)[1];

    let wip_content = vec![
        Line::from(vec![Span::styled("", Style::default())]),
        Line::from(vec![Span::styled(
            "          WORK IN PROGRESS  ⚠️",
            Style::default().fg(RosePine::GOLD).bold(),
        )]),
        Line::from(vec![Span::styled("", Style::default())]),
        Line::from(vec![Span::styled(
            "    This page is currently under development.",
            Style::default().fg(RosePine::TEXT),
        )]),
        Line::from(vec![Span::styled(
            "      Please check back later for updates!",
            Style::default().fg(RosePine::SUBTLE),
        )]),
        Line::from(vec![Span::styled("", Style::default())]),
        Line::from(vec![Span::styled(
            "        Press [←] or [Esc] to go back",
            Style::default().fg(RosePine::FOAM),
        )]),
        Line::from(vec![Span::styled("", Style::default())]),
    ];

    let dialog = Paragraph::new(wip_content)
        .alignment(Alignment::Center)
        .block(
            Block::bordered()
                .title(" Under Construction 🚧 ")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Double)
                .style(Style::default().fg(RosePine::LOVE)),
        )
        .style(Style::default().bg(RosePine::SURFACE));

    Clear.render(dialog_vertical, frame.buffer_mut());
    dialog.render(dialog_vertical, frame.buffer_mut());

    render_bottom_bar(frame, chunks[1], app);
}

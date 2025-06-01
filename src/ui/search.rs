use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::app::{App, InputMode, SearchResultType};

pub fn render_floating_search(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let dialog_width = area.width.saturating_sub(10).max(80);
    let dialog_height = area.height.saturating_sub(6).max(24);

    let dialog_x = (area.width.saturating_sub(dialog_width)) / 2;
    let dialog_y = (area.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect {
        x: area.x + dialog_x,
        y: area.y + dialog_y,
        width: dialog_width,
        height: dialog_height,
    };

    frame.render_widget(Clear, dialog_area);

    let dialog_block = Block::default()
        .title("  Telescope Search ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(crate::ui::colors::RosePine::IRIS))
        .border_type(ratatui::widgets::BorderType::Rounded)
        .style(Style::default().bg(crate::ui::colors::RosePine::BASE));

    frame.render_widget(dialog_block, dialog_area);

    let inner_area = Rect {
        x: dialog_area.x + 1,
        y: dialog_area.y + 1,
        width: dialog_area.width.saturating_sub(2),
        height: dialog_area.height.saturating_sub(2),
    };

    let vertical_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(inner_area);

    let input_area = vertical_layout[0];
    let content_area = vertical_layout[1];

    // Split the content area horizontally for results and preview
    let horizontal_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40), // Results list
            Constraint::Percentage(60), // Preview
        ])
        .split(content_area);

    let results_area = horizontal_layout[0];
    let preview_area = horizontal_layout[1];

    // Render search query with cursor - improved debug mode
    let input_block = Block::bordered()
        .title(" Search ")
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .style(
            Style::default()
                .fg(crate::ui::colors::RosePine::IRIS)
                .bg(crate::ui::colors::RosePine::SURFACE),
        );

    // Format the search query with a visible cursor indicator
    let display_text = if app.search_query.is_empty() {
        "Type to search... ↑/↓: Navigate  ⏎: Select  Esc: Close".to_string()
    } else {
        format!(" {}", app.search_query)
    };

    let search_paragraph = Paragraph::new(display_text)
        .style(
            Style::default()
                .fg(crate::ui::colors::RosePine::GOLD)
                .add_modifier(Modifier::BOLD)
                .bg(crate::ui::colors::RosePine::SURFACE),
        )
        .block(input_block);

    frame.render_widget(search_paragraph, input_area);

    // Show cursor when in search mode
    if app.input_mode == InputMode::Search {
        // Calculate cursor position based on search query
        frame.set_cursor_position(ratatui::layout::Position {
            x: input_area.x + 3 + app.search_query.len() as u16,
            y: input_area.y + 1,
        });
    }

    // Show recent searches when search query is empty
    if app.input_mode == InputMode::Search
        && app.search_query.is_empty()
        && !app.recent_searches.is_empty()
    {
        render_recent_searches(frame, content_area, app);
    } else if !app.search_results.is_empty() {
        // Show search results if there are any
        // Create a results block with no background
        let results_block = Block::default()
            .title(format!(" Results ({}) ", app.search_results.len()))
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .style(
                Style::default()
                    .fg(crate::ui::colors::RosePine::SUBTLE)
                    .bg(crate::ui::colors::RosePine::SURFACE),
            );

        frame.render_widget(results_block, results_area);

        // Create inner area with 1 cell of padding
        let results_inner = Rect {
            x: results_area.x + 1,
            y: results_area.y + 1,
            width: results_area.width.saturating_sub(2),
            height: results_area.height.saturating_sub(2),
        };

        // Create the list items for search results - completely new approach
        let items: Vec<ListItem> = app
            .search_results
            .iter()
            .enumerate()
            .map(|(i, result)| {
                let icon = match result.result_type {
                    SearchResultType::Notebook => "󰠮 ",
                    SearchResultType::Snippet => "󰈮 ",
                    SearchResultType::CodeContent => "󰧮 ",
                };

                let is_selected = i == app.selected_search_result;

                // Get parent path for context
                let parent_path = crate::search::get_parent_path(app, result.parent_id);
                let path_display = if !parent_path.is_empty() {
                    format!(" [{}]", parent_path)
                } else {
                    String::new()
                };

                // Add language info for snippet results
                let language_info = if let SearchResultType::Snippet
                | SearchResultType::CodeContent = result.result_type
                {
                    if let Some(snippet) = app.snippet_database.snippets.get(&result.id) {
                        format!(" ({}) ", snippet.language.display_name())
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };

                // Format the line - first the name/title then the path and context
                let name_span = Span::styled(
                    format!("{}{}{}", icon, result.name, language_info),
                    if is_selected {
                        Style::default()
                            .fg(crate::ui::colors::RosePine::LOVE)
                            .bold()
                    } else {
                        Style::default().fg(crate::ui::colors::RosePine::TEXT)
                    },
                );

                let path_span = Span::styled(
                    path_display,
                    if is_selected {
                        Style::default()
                            .fg(crate::ui::colors::RosePine::FOAM)
                            .bold()
                    } else {
                        Style::default().fg(crate::ui::colors::RosePine::SUBTLE)
                    },
                );

                let context_span = Span::styled(
                    format!(" {}", result.match_context),
                    if is_selected {
                        Style::default()
                            .fg(crate::ui::colors::RosePine::IRIS)
                            .bold()
                    } else {
                        Style::default().fg(crate::ui::colors::RosePine::MUTED)
                    },
                );

                let line = Line::from(vec![
                    Span::styled(
                        if is_selected { "→ " } else { "  " },
                        Style::default().fg(crate::ui::colors::RosePine::GOLD),
                    ),
                    name_span,
                    path_span,
                    context_span,
                ]);

                // Create list item with the line
                ListItem::new(line).style(if is_selected {
                    Style::default().bg(crate::ui::colors::RosePine::OVERLAY)
                } else {
                    Style::default().bg(crate::ui::colors::RosePine::SURFACE)
                })
            })
            .collect();

        // Create a very simple list with minimal styling and transparent background
        let results_list = List::new(items)
            .style(Style::default().bg(crate::ui::colors::RosePine::SURFACE))
            .block(
                Block::default().style(Style::default().bg(crate::ui::colors::RosePine::SURFACE)),
            );

        frame.render_stateful_widget(
            results_list,
            results_inner,
            &mut ListState::default().with_selected(Some(app.selected_search_result)),
        );

        // Render preview of selected result if available
        if let Some(result) = app.search_results.get(app.selected_search_result) {
            // Create a simple border around the preview area
            let preview_block = Block::default()
                .title(" Preview ")
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .style(
                    Style::default()
                        .fg(crate::ui::colors::RosePine::SUBTLE)
                        .bg(crate::ui::colors::RosePine::SURFACE),
                );

            frame.render_widget(preview_block, preview_area);

            // Create inner area with 1 cell of padding
            let preview_inner = Rect {
                x: preview_area.x + 1,
                y: preview_area.y + 1,
                width: preview_area.width.saturating_sub(2),
                height: preview_area.height.saturating_sub(2),
            };

            // Direct render preview content without additional borders
            render_preview_content(frame, preview_inner, app, result);
        }
    } else if app.search_query.is_empty() {
        // Show help text when search is empty
        render_search_help(frame, results_area, preview_area);
    } else {
        let no_results_text = Paragraph::new("No results found. Try a different search query.")
            .style(
                Style::default()
                    .fg(crate::ui::colors::RosePine::GOLD)
                    .bg(crate::ui::colors::RosePine::SURFACE),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .title(" Results ")
                    .style(
                        Style::default()
                            .fg(crate::ui::colors::RosePine::SUBTLE)
                            .bg(crate::ui::colors::RosePine::SURFACE),
                    ),
            );
        frame.render_widget(no_results_text, results_area);
    }
}

/// Renders help text for the search dialog when search query is empty
fn render_search_help(frame: &mut Frame, results_area: Rect, preview_area: Rect) {
    let help_block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Reset));

    frame.render_widget(help_block, results_area);

    let help_inner = Rect {
        x: results_area.x + 1,
        y: results_area.y + 1,
        width: results_area.width.saturating_sub(2),
        height: results_area.height.saturating_sub(2),
    };

    // Clear background - Fixed BUG for whitebg
    frame.render_widget(
        Block::default().style(Style::default().bg(Color::Reset)),
        help_inner,
    );

    let help_text = Text::from(vec![
        Line::from(vec![Span::styled(
            " Telescope Search:",
            Style::default()
                .fg(Color::LightBlue)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("Type to search across notebooks, snippets, and content"),
        Line::from("Results update in real-time as you type"),
        Line::from(""),
        Line::from(vec![
            Span::styled("↑/↓", Style::default().fg(Color::Yellow)),
            Span::raw(" Navigate results"),
        ]),
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" Open selected item"),
        ]),
        Line::from(vec![
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" Close search"),
        ]),
    ]);

    let help_paragraph = Paragraph::new(help_text)
        .style(Style::default().bg(Color::Reset))
        .wrap(Wrap { trim: true });

    frame.render_widget(help_paragraph, help_inner);

    // Also render something in the preview area
    let preview_block = Block::default()
        .title(" Preview ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Reset));

    frame.render_widget(preview_block, preview_area);

    let preview_inner = Rect {
        x: preview_area.x + 1,
        y: preview_area.y + 1,
        width: preview_area.width.saturating_sub(2),
        height: preview_area.height.saturating_sub(2),
    };

    let preview_text = vec![
        Line::from("Start typing to search..."),
        Line::from(""),
        Line::from("Search will find matches in:"),
        Line::from("- Notebook names"),
        Line::from("- Notebook descriptions"),
        Line::from("- Snippet titles"),
        Line::from("- Snippet descriptions"),
        Line::from("- Snippet content"),
    ];

    frame.render_widget(
        Paragraph::new(preview_text)
            .style(Style::default().bg(Color::Reset))
            .wrap(Wrap { trim: true }),
        preview_inner,
    );
}

fn render_preview_content(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    result: &crate::app::SearchResult,
) {
    frame.render_widget(
        Block::default()
            .style(Style::default().bg(Color::Reset))
            .borders(Borders::NONE),
        area,
    );

    match result.result_type {
        SearchResultType::Notebook => {
            if let Some(notebook) = app.snippet_database.notebooks.get(&result.id) {
                let mut notebook_info = Vec::new();
                notebook_info.push(Line::from(vec![
                    Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(&notebook.name),
                ]));

                if let Some(desc) = &notebook.description {
                    notebook_info.push(Line::from(vec![
                        Span::styled(
                            "Description: ",
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(desc),
                    ]));
                }

                // Count snippets in this notebook
                let snippet_count = app
                    .snippet_database
                    .snippets
                    .values()
                    .filter(|s| s.notebook_id == result.id)
                    .count();

                notebook_info.push(Line::from(vec![
                    Span::styled("Snippets: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(snippet_count.to_string()),
                ]));

                let notebook_paragraph = Paragraph::new(notebook_info)
                    .wrap(Wrap { trim: true })
                    .style(Style::default().bg(Color::Reset));

                frame.render_widget(notebook_paragraph, area);
            }
        }
        SearchResultType::Snippet | SearchResultType::CodeContent => {
            // Preview snippet content with syntax highlighting
            if let Some(snippet) = app.snippet_database.snippets.get(&result.id) {
                // Create a simplified header + content layout
                let chunks = Layout::vertical([
                    Constraint::Length(1), // Minimal header (just one line)
                    Constraint::Min(1),    // Content
                ])
                .split(area);

                // Render simple header
                let header = Line::from(vec![Span::styled(
                    format!(
                        "{} {} ({})",
                        snippet.language.icon(),
                        snippet.title,
                        snippet.language.display_name()
                    ),
                    Style::default()
                        .fg(crate::ui::colors::RosePine::TEXT)
                        .add_modifier(Modifier::BOLD),
                )]);

                frame.render_widget(
                    Paragraph::new(header).style(Style::default().bg(Color::Reset)),
                    chunks[0],
                );

                // Content area
                if !snippet.content.is_empty() {
                    // Use the original syntax highlighting function but make sure we
                    // reset the background first to avoid any artifacts
                    frame.render_widget(
                        Block::default()
                            .style(Style::default().bg(Color::Reset))
                            .borders(Borders::NONE),
                        chunks[1],
                    );

                    // Call the original highlighting function but in the content area
                    display_syntax_highlighted_content(
                        frame,
                        chunks[1],
                        &snippet.content,
                        snippet,
                        app,
                    );
                } else {
                    frame.render_widget(
                        Paragraph::new("(Empty snippet)")
                            .style(Style::default().fg(Color::Gray).bg(Color::Reset)),
                        chunks[1],
                    );
                }
            }
        }
    }
}

/// Render the recent searches in a detailed view
fn render_recent_searches(frame: &mut Frame, content_area: Rect, app: &mut App) {
    // First divide the content area vertically to add space for recent files below
    let vertical_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(60), // Recent searches section
            Constraint::Percentage(40), // Recent files section
        ])
        .split(content_area);

    let recent_searches_area = vertical_layout[0];
    let recent_files_area = vertical_layout[1];

    // Split the recent searches area into list and details
    let horizontal_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40), // Recent searches list
            Constraint::Percentage(60), // Preview details
        ])
        .split(recent_searches_area);

    let recents_area = horizontal_layout[0];
    let details_area = horizontal_layout[1];

    // Create a block for the recent searches
    let recents_block = Block::default()
        .title(" Recent Searches ")
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .style(
            Style::default()
                .fg(crate::ui::colors::RosePine::SUBTLE)
                .bg(crate::ui::colors::RosePine::SURFACE),
        );

    frame.render_widget(recents_block, recents_area);

    let recents_inner = Rect {
        x: recents_area.x + 1,
        y: recents_area.y + 1,
        width: recents_area.width.saturating_sub(2),
        height: recents_area.height.saturating_sub(2),
    };

    // Create the list items for recent searches - now with much more detail
    let items: Vec<ListItem> = app
        .recent_searches
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let is_selected = i == app.selected_recent_search;

            // Main query with result count
            let query_span = Span::styled(
                format!("{}", entry.query),
                if is_selected {
                    Style::default()
                        .fg(crate::ui::colors::RosePine::LOVE)
                        .bold()
                } else {
                    Style::default().fg(crate::ui::colors::RosePine::TEXT)
                },
            );

            let count_span = Span::styled(
                format!(" ({} results)", entry.result_count),
                if is_selected {
                    Style::default()
                        .fg(crate::ui::colors::RosePine::GOLD)
                        .bold()
                } else {
                    Style::default().fg(crate::ui::colors::RosePine::SUBTLE)
                },
            );

            // Create line with timestamp
            let time_span = Span::styled(
                format!(" - {}", entry.formatted_time()),
                if is_selected {
                    Style::default()
                        .fg(crate::ui::colors::RosePine::FOAM)
                        .italic()
                } else {
                    Style::default()
                        .fg(crate::ui::colors::RosePine::MUTED)
                        .italic()
                },
            );

            // Create a multi-span line for each search
            let line = Line::from(vec![
                Span::styled(
                    if is_selected { "→ " } else { "  " },
                    Style::default().fg(crate::ui::colors::RosePine::GOLD),
                ),
                Span::styled(
                    "🔍 ",
                    Style::default().fg(crate::ui::colors::RosePine::IRIS),
                ),
                query_span,
                count_span,
                time_span,
            ]);

            // Create list item with the line
            ListItem::new(line).style(if is_selected {
                Style::default().bg(crate::ui::colors::RosePine::OVERLAY)
            } else {
                Style::default().bg(crate::ui::colors::RosePine::SURFACE)
            })
        })
        .collect();

    // Create the list with rose pine styling
    let recents_list = List::new(items)
        .style(Style::default().bg(crate::ui::colors::RosePine::SURFACE))
        .block(Block::default().style(Style::default().bg(crate::ui::colors::RosePine::SURFACE)));

    frame.render_stateful_widget(
        recents_list,
        recents_inner,
        &mut ListState::default().with_selected(Some(app.selected_recent_search)),
    );

    // Render details block for the selected recent search
    let details_block = Block::default()
        .title(" Search Details ")
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .style(
            Style::default()
                .fg(crate::ui::colors::RosePine::SUBTLE)
                .bg(crate::ui::colors::RosePine::SURFACE),
        );

    frame.render_widget(details_block, details_area);

    // Create inner area for details
    let details_inner = Rect {
        x: details_area.x + 1,
        y: details_area.y + 1,
        width: details_area.width.saturating_sub(2),
        height: details_area.height.saturating_sub(2),
    };

    // If a recent search is selected, show its details
    if let Some(entry) = app.recent_searches.get(app.selected_recent_search) {
        // Show detailed info about the selected search
        let mut detail_lines = Vec::new();

        // Query info
        detail_lines.push(Line::from(vec![
            Span::styled(
                " Search Query: ",
                Style::default()
                    .fg(crate::ui::colors::RosePine::MUTED)
                    .bold(),
            ),
            Span::styled(
                &entry.query,
                Style::default().fg(crate::ui::colors::RosePine::LOVE),
            ),
        ]));

        detail_lines.push(Line::from(""));

        // Search stats
        detail_lines.push(Line::from(vec![
            Span::styled(
                "󰡦 Results: ",
                Style::default()
                    .fg(crate::ui::colors::RosePine::MUTED)
                    .bold(),
            ),
            Span::styled(
                entry.result_count.to_string(),
                Style::default().fg(crate::ui::colors::RosePine::GOLD),
            ),
        ]));

        detail_lines.push(Line::from(vec![
            Span::styled(
                "󱑇 Searched at: ",
                Style::default()
                    .fg(crate::ui::colors::RosePine::MUTED)
                    .bold(),
            ),
            Span::styled(
                entry.formatted_time(),
                Style::default().fg(crate::ui::colors::RosePine::FOAM),
            ),
        ]));

        detail_lines.push(Line::from(""));

        // Last selected item info if available
        if let (Some(result_type), Some(result_id)) =
            (&entry.last_selected_type, &entry.last_selected_id)
        {
            detail_lines.push(Line::from(vec![Span::styled(
                "Last Selected: ",
                Style::default()
                    .fg(crate::ui::colors::RosePine::MUTED)
                    .bold(),
            )]));

            match result_type {
                SearchResultType::Notebook => {
                    if let Some(notebook) = app.snippet_database.notebooks.get(result_id) {
                        detail_lines.push(Line::from(vec![
                            Span::styled(
                                "  󰺄 Notebook: ",
                                Style::default().fg(crate::ui::colors::RosePine::SUBTLE),
                            ),
                            Span::styled(
                                &notebook.name,
                                Style::default().fg(crate::ui::colors::RosePine::TEXT),
                            ),
                        ]));

                        if let Some(desc) = &notebook.description {
                            if !desc.is_empty() {
                                detail_lines.push(Line::from(vec![
                                    Span::styled(
                                        "   Description: ",
                                        Style::default().fg(crate::ui::colors::RosePine::SUBTLE),
                                    ),
                                    Span::styled(
                                        desc,
                                        Style::default().fg(crate::ui::colors::RosePine::MUTED),
                                    ),
                                ]));
                            }
                        }

                        detail_lines.push(Line::from(vec![
                            Span::styled(
                                "   Snippets: ",
                                Style::default().fg(crate::ui::colors::RosePine::SUBTLE),
                            ),
                            Span::styled(
                                notebook.snippet_count.to_string(),
                                Style::default().fg(crate::ui::colors::RosePine::IRIS),
                            ),
                        ]));
                    }
                }
                SearchResultType::Snippet | SearchResultType::CodeContent => {
                    if let Some(snippet) = app.snippet_database.snippets.get(result_id) {
                        detail_lines.push(Line::from(vec![
                            Span::styled(
                                "   Snippet: ",
                                Style::default().fg(crate::ui::colors::RosePine::SUBTLE),
                            ),
                            Span::styled(
                                &snippet.title,
                                Style::default().fg(crate::ui::colors::RosePine::TEXT),
                            ),
                        ]));

                        detail_lines.push(Line::from(vec![
                            Span::styled(
                                "  󰘦 Language: ",
                                Style::default().fg(crate::ui::colors::RosePine::SUBTLE),
                            ),
                            Span::styled(
                                format!(
                                    "{} {}",
                                    snippet.language.icon(),
                                    snippet.language.display_name()
                                ),
                                Style::default().fg(crate::ui::colors::RosePine::FOAM),
                            ),
                        ]));

                        detail_lines.push(Line::from(vec![
                            Span::styled(
                                "  󰉻 Lines: ",
                                Style::default().fg(crate::ui::colors::RosePine::SUBTLE),
                            ),
                            Span::styled(
                                snippet.get_line_count().to_string(),
                                Style::default().fg(crate::ui::colors::RosePine::GOLD),
                            ),
                        ]));

                        if let Some(desc) = &snippet.description {
                            if !desc.is_empty() {
                                detail_lines.push(Line::from(vec![
                                    Span::styled(
                                        "   Description: ",
                                        Style::default().fg(crate::ui::colors::RosePine::SUBTLE),
                                    ),
                                    Span::styled(
                                        desc,
                                        Style::default().fg(crate::ui::colors::RosePine::MUTED),
                                    ),
                                ]));
                            }
                        }
                    }
                }
            }
        } else {
            detail_lines.push(Line::from(vec![Span::styled(
                " No item selected from this search yet",
                Style::default()
                    .fg(crate::ui::colors::RosePine::MUTED)
                    .italic(),
            )]));
        }

        detail_lines.push(Line::from(""));
        detail_lines.push(Line::from(vec![Span::styled(
            "Press Enter to run this search again",
            Style::default()
                .fg(crate::ui::colors::RosePine::IRIS)
                .italic(),
        )]));

        // Render the details paragraph
        let details_para = Paragraph::new(detail_lines)
            .style(Style::default().bg(crate::ui::colors::RosePine::SURFACE))
            .wrap(Wrap { trim: true });

        frame.render_widget(details_para, details_inner);
    }

    // Create a block for the recent files
    let recent_files_block = Block::default()
        .title(" 󰥔 Recent Files ")
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .style(
            Style::default()
                .fg(crate::ui::colors::RosePine::SUBTLE)
                .bg(crate::ui::colors::RosePine::SURFACE),
        );

    frame.render_widget(recent_files_block, recent_files_area);

    let recent_files_inner = Rect {
        x: recent_files_area.x + 1,
        y: recent_files_area.y + 1,
        width: recent_files_area.width.saturating_sub(2),
        height: recent_files_area.height.saturating_sub(2),
    };

    // Get most recently accessed snippets
    let mut recent_snippets: Vec<_> = app.snippet_database.snippets.values().collect();
    recent_snippets.sort_by(|a, b| b.accessed_at.cmp(&a.accessed_at));
    recent_snippets.truncate(20);

    if recent_snippets.is_empty() {
        let empty_text = Paragraph::new("No snippets accessed yet. Press 's' to create some!")
            .alignment(ratatui::layout::Alignment::Center)
            .style(Style::default().fg(crate::ui::colors::RosePine::MUTED));
        frame.render_widget(empty_text, recent_files_inner);
    } else {
        let items: Vec<ListItem> = recent_snippets
            .iter()
            .enumerate()
            .map(|(i, snippet)| {
                let icon = snippet.language.icon();
                let notebook_name = app
                    .snippet_database
                    .notebooks
                    .get(&snippet.notebook_id)
                    .map(|n| n.name.as_str())
                    .unwrap_or("Unknown");

                let color_index = app.get_notebook_color(&snippet.notebook_id);
                let colors = crate::ui::code_snippets::get_available_colors();
                let notebook_color = colors
                    .get(color_index % colors.len())
                    .unwrap_or(&colors[0])
                    .1;

                // Get time ago string
                let now = chrono::Utc::now();
                let duration = now.signed_duration_since(snippet.accessed_at);
                let ago = if duration.num_days() > 0 {
                    format!("{} days ago", duration.num_days())
                } else if duration.num_hours() > 0 {
                    format!("{} hours ago", duration.num_hours())
                } else if duration.num_minutes() > 0 {
                    format!("{} min ago", duration.num_minutes())
                } else {
                    "Just now".to_string()
                };

                let lines = snippet.get_line_count();
                let language = snippet.language.display_name();

                // Format the title to truncate if necessary
                let title = if snippet.title.len() > 20 {
                    format!("{}...", &snippet.title[..17])
                } else {
                    snippet.title.clone()
                };

                // Create a notebook span with its color
                let notebook_span = Span::styled(
                    format!("{}", notebook_name),
                    Style::default().fg(notebook_color),
                );

                let spans = vec![
                    Span::styled(
                        format!("{} ", icon),
                        Style::default().fg(crate::ui::colors::RosePine::GOLD),
                    ),
                    Span::styled(
                        title,
                        Style::default()
                            .fg(crate::ui::colors::RosePine::TEXT)
                            .bold(),
                    ),
                    Span::raw(" • "),
                    notebook_span,
                    Span::raw(format!(" • {} • {} lines • {}", language, lines, ago)),
                ];

                ListItem::new(Line::from(spans)).style(
                    Style::default()
                        .fg(if i == 0 {
                            crate::ui::colors::RosePine::LOVE
                        } else {
                            crate::ui::colors::RosePine::TEXT
                        })
                        .bg(if i % 2 == 0 {
                            crate::ui::colors::RosePine::HIGHLIGHT_LOW
                        } else {
                            crate::ui::colors::RosePine::SURFACE
                        }),
                )
            })
            .collect();

        let recent_files_list = List::new(items)
            .style(Style::default().fg(crate::ui::colors::RosePine::TEXT))
            .highlight_style(Style::default().fg(crate::ui::colors::RosePine::GOLD));

        frame.render_widget(recent_files_list, recent_files_inner);
    }
}

/// Use a wrapper around the syntax highlighting function to avoid borders and background issues
fn display_syntax_highlighted_content(
    frame: &mut Frame,
    area: Rect,
    content: &str,
    snippet: &crate::models::CodeSnippet,
    app: &App,
) {
    // Instead of directly calling the original function, we'll modify its behavior

    // The only change needed is to use the codebase's syntax highlighting
    // but with our own background reset beforehand
    frame.render_widget(
        Block::default()
            .style(Style::default().bg(Color::Reset))
            .borders(Borders::NONE),
        area,
    );

    // Now call the original function
    crate::ui::code_snippets::display_highlighted_content(frame, area, content, snippet, app);
}

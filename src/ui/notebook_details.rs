use crate::app::{App, CodeSnippetsState, InputMode};
use crate::models::SnippetLanguage;
use crate::ui::colors::RosePine;
use ratatui::widgets::Widget;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{
        BarChart, Block, BorderType, Cell, List, ListItem, ListState, Paragraph, Row, Table, Wrap,
    },
};
use std::collections::HashMap;
use uuid::Uuid;

/// Render detailed notebook view
pub fn render(frame: &mut Frame, app: &mut App, notebook_id: Uuid) {
    let notebook = match app.snippet_database.notebooks.get(&notebook_id) {
        Some(notebook) => notebook,
        None => {
            app.code_snippets_state = CodeSnippetsState::NotebookList;
            app.set_error_message("Notebook not found".to_string());
            return;
        }
    };

    // Get all snippets for this notebook
    let snippets: Vec<_> = app
        .snippet_database
        .snippets
        .values()
        .filter(|s| s.notebook_id == notebook_id)
        .collect();

    let main_area = frame.area();

    // Main block
    let block = Block::bordered()
        .title(format!(" {} Notebook Details: {} ", "", notebook.name))
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(RosePine::HIGHLIGHT_HIGH));

    let inner_area = block.inner(main_area);
    block.render(main_area, frame.buffer_mut());

    // Get notebook color
    let color_index = app.get_notebook_color(&notebook_id);
    let colors = get_available_colors();
    let notebook_color = colors
        .get(color_index % colors.len())
        .unwrap_or(&colors[0])
        .1;

    // Create a colored title block
    let title_block = Block::default()
        .title(format!(" {} Notebook Details: {} ", "", notebook.name))
        .title_alignment(Alignment::Center)
        .title_style(Style::default().fg(notebook_color).bold())
        .border_type(BorderType::Rounded)
        .borders(ratatui::widgets::Borders::TOP)
        .style(Style::default().fg(notebook_color));

    title_block.render(main_area, frame.buffer_mut());

    // Create a layout for the different sections
    let chunks = Layout::vertical([
        Constraint::Length(3),  // Navigation bar
        Constraint::Length(10), // Overview information
        Constraint::Length(12), // Language distribution
        Constraint::Length(8),  // Tags section
        Constraint::Min(5),     // Snippets list
        Constraint::Length(1),  // Status line
    ])
    .split(inner_area);

    // Render navigation bar
    let nav_block = Block::bordered()
        .title(" Actions ")
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(RosePine::HIGHLIGHT_HIGH));

    let nav_area = nav_block.inner(chunks[0]);
    nav_block.render(chunks[0], frame.buffer_mut());

    let nav_buttons = vec![
        ("e", "Edit Description"),
        ("c", "Change Color"),
        ("s", "New Snippet"),
    ];

    let button_width = nav_area.width / nav_buttons.len() as u16;
    let button_areas = Layout::horizontal(
        nav_buttons
            .iter()
            .map(|_| Constraint::Length(button_width))
            .collect::<Vec<_>>(),
    )
    .split(nav_area);

    for (i, (key, label)) in nav_buttons.iter().enumerate() {
        let button_text = format!(" {} {} ", key, label);
        let button = Paragraph::new(button_text)
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .fg(RosePine::TEXT)
                    .bg(RosePine::HIGHLIGHT_LOW),
            );

        button.render(button_areas[i], frame.buffer_mut());
    }

    // Render status line
    let status_text =
        "← Back (Esc) • 's' to Create Snippet • 'e' to Edit Description • 'c' to Change Color";
    let status = Paragraph::new(status_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(RosePine::MUTED));
    status.render(chunks[5], frame.buffer_mut());

    // Calculate statistics
    let total_lines: usize = snippets.iter().map(|s| s.get_line_count()).sum();

    let avg_use_count = if !snippets.is_empty() {
        snippets.iter().map(|s| s.use_count as usize).sum::<usize>() as f64 / snippets.len() as f64
    } else {
        0.0
    };

    // Count languages
    let mut languages = HashMap::new();
    let mut lang_lines = HashMap::new();
    for snippet in &snippets {
        let lines = snippet.get_line_count();
        *languages.entry(snippet.language.clone()).or_insert(0) += 1;
        *lang_lines.entry(snippet.language.clone()).or_insert(0) += lines;
    }

    // Collect all unique tags and count their occurrences
    let mut tag_counts = HashMap::new();
    let mut total_tags = 0;
    for snippet in &snippets {
        for tag in &snippet.tags {
            *tag_counts.entry(tag.clone()).or_insert(0) += 1;
            total_tags += 1;
        }
    }

    // Sort tags by frequency
    let mut sorted_tags: Vec<(String, usize)> = tag_counts.into_iter().collect();
    sorted_tags.sort_by(|a, b| b.1.cmp(&a.1));

    // Find most used language
    let _most_used_language = languages
        .iter()
        .max_by_key(|(_, count)| *count)
        .map(|(lang, _)| lang)
        .unwrap_or(&SnippetLanguage::Text);

    // Sort by frequency
    let mut lang_counts: Vec<(SnippetLanguage, usize)> = languages.into_iter().collect();
    lang_counts.sort_by(|a, b| b.1.cmp(&a.1));

    // Sort by lines
    let mut line_counts: Vec<(SnippetLanguage, usize)> = lang_lines.into_iter().collect();
    line_counts.sort_by(|a, b| b.1.cmp(&a.1));

    // Calculate age metrics
    let now = chrono::Utc::now();
    let notebook_age = (now - notebook.created_at).num_days();
    let last_update = (now - notebook.updated_at).num_days();
    let snippets_per_day = if notebook_age > 0 {
        snippets.len() as f64 / notebook_age as f64
    } else {
        snippets.len() as f64
    };

    // 1. OVERVIEW SECTION
    let overview_block = Block::bordered()
        .title(" Overview ")
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(RosePine::SUBTLE));

    let overview_area = overview_block.inner(chunks[1]);
    overview_block.render(chunks[1], frame.buffer_mut());

    let overview_chunks = Layout::horizontal([
        Constraint::Percentage(60), // Basic info
        Constraint::Percentage(40), // Stats
    ])
    .split(overview_area);

    // Left side - basic info
    let info_lines = vec![
        Line::from(vec![
            Span::styled("Name: ", Style::default().fg(RosePine::MUTED)),
            Span::styled(&notebook.name, Style::default().fg(RosePine::TEXT).bold()),
        ]),
        Line::from(vec![
            Span::styled("Created: ", Style::default().fg(RosePine::MUTED)),
            Span::styled(
                notebook.created_at.format("%Y-%m-%d %H:%M").to_string(),
                Style::default().fg(RosePine::TEXT),
            ),
        ]),
        Line::from(vec![
            Span::styled("Updated: ", Style::default().fg(RosePine::MUTED)),
            Span::styled(
                notebook.updated_at.format("%Y-%m-%d %H:%M").to_string(),
                Style::default().fg(RosePine::TEXT),
            ),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Description: ",
            Style::default().fg(RosePine::MUTED),
        )]),
        Line::from(vec![Span::styled(
            notebook
                .description
                .clone()
                .unwrap_or_else(|| "No description".to_string()),
            Style::default().fg(RosePine::TEXT),
        )]),
    ];

    let info_paragraph = Paragraph::new(info_lines)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    info_paragraph.render(overview_chunks[0], frame.buffer_mut());

    // Right side - Key stats
    let stats_lines = vec![
        Line::from(vec![
            Span::styled("Snippets: ", Style::default().fg(RosePine::MUTED)),
            Span::styled(
                snippets.len().to_string(),
                Style::default().fg(RosePine::LOVE).bold(),
            ),
        ]),
        Line::from(vec![
            Span::styled("Total Lines: ", Style::default().fg(RosePine::MUTED)),
            Span::styled(
                total_lines.to_string(),
                Style::default().fg(RosePine::GOLD).bold(),
            ),
        ]),
        Line::from(vec![
            Span::styled("Total Tags: ", Style::default().fg(RosePine::MUTED)),
            Span::styled(
                total_tags.to_string(),
                Style::default().fg(RosePine::IRIS).bold(),
            ),
        ]),
        Line::from(vec![
            Span::styled("Unique Tags: ", Style::default().fg(RosePine::MUTED)),
            Span::styled(
                sorted_tags.len().to_string(),
                Style::default().fg(RosePine::FOAM).bold(),
            ),
        ]),
        Line::from(vec![
            Span::styled("Avg. Usage: ", Style::default().fg(RosePine::MUTED)),
            Span::styled(
                format!("{:.1}", avg_use_count),
                Style::default().fg(RosePine::FOAM).bold(),
            ),
        ]),
        Line::from(vec![
            Span::styled("Notebook Age: ", Style::default().fg(RosePine::MUTED)),
            Span::styled(
                format!("{} days", notebook_age),
                Style::default().fg(RosePine::TEXT),
            ),
        ]),
        Line::from(vec![
            Span::styled("Last Updated: ", Style::default().fg(RosePine::MUTED)),
            Span::styled(
                format!("{} days ago", last_update),
                Style::default().fg(RosePine::TEXT),
            ),
        ]),
        Line::from(vec![
            Span::styled("Activity Rate: ", Style::default().fg(RosePine::MUTED)),
            Span::styled(
                format!("{:.2} snippets/day", snippets_per_day),
                Style::default().fg(RosePine::TEXT),
            ),
        ]),
    ];

    let stats_paragraph = Paragraph::new(stats_lines)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    stats_paragraph.render(overview_chunks[1], frame.buffer_mut());

    // 2. LANGUAGE DISTRIBUTION SECTION
    let lang_block = Block::bordered()
        .title(" Language Distribution ")
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(RosePine::SUBTLE));

    let lang_area = lang_block.inner(chunks[2]);
    lang_block.render(chunks[2], frame.buffer_mut());

    if snippets.is_empty() {
        let no_data = Paragraph::new("No snippets in this notebook")
            .alignment(Alignment::Center)
            .style(Style::default().fg(RosePine::MUTED));
        no_data.render(lang_area, frame.buffer_mut());
    } else {
        let lang_chunks = Layout::horizontal([
            Constraint::Percentage(60), // Chart
            Constraint::Percentage(40), // Table
        ])
        .split(lang_area);

        // Left side - Bar chart
        let data: Vec<(&str, u64)> = lang_counts
            .iter()
            .take(6) // Limit to top 6 languages
            .map(|(lang, count)| (lang.short_name(), *count as u64))
            .collect();

        if !data.is_empty() {
            let barchart = BarChart::default()
                .bar_width(5)
                .bar_gap(1)
                .bar_style(Style::default().fg(RosePine::FOAM))
                .value_style(Style::default().fg(RosePine::TEXT))
                .data(&data)
                .max(
                    lang_counts
                        .iter()
                        .map(|(_, count)| *count)
                        .max()
                        .unwrap_or(1) as u64,
                );

            barchart.render(lang_chunks[0], frame.buffer_mut());
        }

        // Right side - Language stats table
        if !lang_counts.is_empty() {
            let lang_rows: Vec<Row> = lang_counts
                .iter()
                .take(6) // Limit to top 6 languages
                .map(|(lang, count)| {
                    let _percentage = (*count as f64 / snippets.len() as f64) * 100.0;
                    let lines = line_counts
                        .iter()
                        .find(|(l, _)| l == lang)
                        .map(|(_, lines)| *lines)
                        .unwrap_or(0);
                    let line_percentage = (lines as f64 / total_lines as f64) * 100.0;

                    Row::new(vec![
                        Cell::from(format!("{}", lang.short_name()))
                            .style(Style::default().fg(RosePine::FOAM)),
                        Cell::from(count.to_string()).style(Style::default().fg(RosePine::TEXT)),
                        Cell::from(format!("{:.1}%", line_percentage))
                            .style(Style::default().fg(RosePine::LOVE)),
                    ])
                })
                .collect();

            let header = Row::new(vec![
                Cell::from("Lang").style(Style::default().fg(RosePine::IRIS).bold()),
                Cell::from("Count").style(Style::default().fg(RosePine::IRIS).bold()),
                Cell::from("Lines %").style(Style::default().fg(RosePine::IRIS).bold()),
            ]);

            let lang_table = Table::new(
                lang_rows,
                &[
                    Constraint::Percentage(40),
                    Constraint::Percentage(30),
                    Constraint::Percentage(30),
                ],
            )
            .header(header)
            .block(Block::default())
            .column_spacing(1);

            lang_table.render(lang_chunks[1], frame.buffer_mut());
        }
    }

    // TAGS SECTION
    let tags_block = Block::bordered()
        .title(" Tags ")
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(RosePine::SUBTLE));

    let tags_area = tags_block.inner(chunks[3]);
    tags_block.render(chunks[3], frame.buffer_mut());

    if sorted_tags.is_empty() {
        let no_tags = Paragraph::new("No tags found in this notebook")
            .alignment(Alignment::Center)
            .style(Style::default().fg(RosePine::MUTED));
        no_tags.render(tags_area, frame.buffer_mut());
    } else {
        let tag_columns =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(tags_area);
        let mut left_tags = Vec::new();
        let mut right_tags = Vec::new();

        for (idx, (tag, count)) in sorted_tags.iter().enumerate() {
            let tag_line = Line::from(vec![
                Span::styled(
                    format!("#{}", tag),
                    Style::default().fg(RosePine::IRIS).bold(),
                ),
                Span::raw(" "),
                Span::styled(
                    format!("({})", count),
                    Style::default().fg(RosePine::SUBTLE),
                ),
            ]);

            if idx % 2 == 0 {
                left_tags.push(tag_line);
            } else {
                right_tags.push(tag_line);
            }
        }

        // Render tag columns
        let left_paragraph = Paragraph::new(left_tags)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });

        let right_paragraph = Paragraph::new(right_tags)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });

        left_paragraph.render(tag_columns[0], frame.buffer_mut());
        right_paragraph.render(tag_columns[1], frame.buffer_mut());
    }

    // 3. SNIPPETS LIST SECTION
    let snippets_block = Block::bordered()
        .title(" Snippets ")
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(RosePine::SUBTLE));

    let snippets_area = snippets_block.inner(chunks[4]);
    snippets_block.render(chunks[4], frame.buffer_mut());

    if snippets.is_empty() {
        let no_snippets =
            Paragraph::new("No snippets in this notebook\nPress 's' to create a new snippet")
                .alignment(Alignment::Center)
                .style(Style::default().fg(RosePine::MUTED));
        no_snippets.render(snippets_area, frame.buffer_mut());
    } else {
        let header = Row::new(vec![
            Cell::from("Title").style(Style::default().fg(RosePine::LOVE).bold()),
            Cell::from("Language").style(Style::default().fg(RosePine::LOVE).bold()),
            Cell::from("Lines").style(Style::default().fg(RosePine::LOVE).bold()),
            Cell::from("Used").style(Style::default().fg(RosePine::LOVE).bold()),
            Cell::from("Updated").style(Style::default().fg(RosePine::LOVE).bold()),
        ]);

        let rows: Vec<Row> = snippets
            .iter()
            .map(|snippet| {
                let line_count = snippet.get_line_count();
                let updated = snippet.updated_at.format("%Y-%m-%d").to_string();

                Row::new(vec![
                    Cell::from(snippet.title.clone()).style(Style::default().fg(RosePine::TEXT)),
                    Cell::from(format!(
                        "{} {}",
                        snippet.language.icon(),
                        snippet.language.short_name()
                    ))
                    .style(Style::default().fg(RosePine::FOAM)),
                    Cell::from(line_count.to_string()).style(Style::default().fg(RosePine::GOLD)),
                    Cell::from(snippet.use_count.to_string())
                        .style(Style::default().fg(RosePine::IRIS)),
                    Cell::from(updated).style(Style::default().fg(RosePine::SUBTLE)),
                ])
            })
            .collect();

        let table = Table::new(
            rows,
            &[
                Constraint::Percentage(40),
                Constraint::Percentage(20),
                Constraint::Percentage(10),
                Constraint::Percentage(10),
                Constraint::Percentage(20),
            ],
        )
        .header(header)
        .block(Block::default())
        .column_spacing(1);

        table.render(snippets_area, frame.buffer_mut());
    }

    // Note: DO NOT return from this function early if in edit mode,
    // as we need to render overlays on top

    // Render overlays on top of everything else
    match app.input_mode {
        InputMode::EditNotebookDescription => {
            render_edit_description_overlay(frame, main_area, app);
        }
        InputMode::SelectNotebookColor => {
            render_color_selection_overlay(frame, main_area, app);
        }
        InputMode::Normal => {
            if let Some(ref message) = app.error_message {
                render_message_overlay(frame, main_area, message, true);
            } else if let Some(ref message) = app.success_message {
                render_message_overlay(frame, main_area, message, false);
            }
        }
        _ => {}
    }
}

fn get_available_colors() -> Vec<(&'static str, ratatui::style::Color)> {
    vec![
        ("Default", RosePine::TEXT),
        ("Red", RosePine::LOVE),
        ("Orange", RosePine::GOLD),
        ("Green", RosePine::FOAM),
        ("Blue", RosePine::IRIS),
        ("Purple", RosePine::IRIS),
        ("Pink", RosePine::ROSE),
        ("White", ratatui::style::Color::White),
    ]
}

fn render_edit_description_overlay(frame: &mut Frame, area: Rect, app: &mut App) {
    let popup_area = spotlight_bar(70, area);

    ratatui::widgets::Clear.render(popup_area, frame.buffer_mut());

    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(RosePine::SUBTLE).bg(RosePine::SURFACE));

    let inner_area = block.inner(popup_area);
    block.render(popup_area, frame.buffer_mut());

    let title = "Edit Notebook Description";
    let chunks = Layout::horizontal([
        Constraint::Length(title.len() as u16 + 4),
        Constraint::Min(10),
        Constraint::Length(24),
    ])
    .split(inner_area);

    let title_paragraph = Paragraph::new(title)
        .alignment(Alignment::Left)
        .style(Style::default().fg(RosePine::IRIS).bold());
    title_paragraph.render(chunks[0], frame.buffer_mut());

    let input_text = format!("{}", app.input_buffer);
    let input_paragraph = Paragraph::new(input_text)
        .style(Style::default().fg(RosePine::TEXT))
        .alignment(Alignment::Left);
    input_paragraph.render(chunks[1], frame.buffer_mut());

    let help_text = "⎋ Cancel • ⏎ Confirm";
    let help_paragraph = Paragraph::new(help_text)
        .alignment(Alignment::Right)
        .style(Style::default().fg(RosePine::MUTED));
    help_paragraph.render(chunks[2], frame.buffer_mut());
}

fn render_color_selection_overlay(frame: &mut Frame, area: Rect, app: &mut App) {
    let popup_area = spotlight_bar(70, area);

    ratatui::widgets::Clear.render(popup_area, frame.buffer_mut());

    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(RosePine::SUBTLE).bg(RosePine::SURFACE));

    let inner_area = block.inner(popup_area);
    block.render(popup_area, frame.buffer_mut());

    let title = "Select Notebook Color";
    let chunks = Layout::horizontal([
        Constraint::Length(title.len() as u16 + 4),
        Constraint::Min(10),
        Constraint::Length(24),
    ])
    .split(inner_area);

    let title_paragraph = Paragraph::new(title)
        .alignment(Alignment::Left)
        .style(Style::default().fg(RosePine::IRIS).bold());
    title_paragraph.render(chunks[0], frame.buffer_mut());

    let colors = get_available_colors();
    let selected_color = &colors[app.selected_language % colors.len()];
    let selected_text = format!("■ {}", selected_color.0);

    let dropdown_paragraph = Paragraph::new(selected_text)
        .alignment(Alignment::Left)
        .style(Style::default().fg(selected_color.1));
    dropdown_paragraph.render(chunks[1], frame.buffer_mut());

    let help_text = "↑↓ Navigate • ⏎ Select";
    let help_paragraph = Paragraph::new(help_text)
        .alignment(Alignment::Right)
        .style(Style::default().fg(RosePine::MUTED));
    help_paragraph.render(chunks[2], frame.buffer_mut());

    let list_area = Rect::new(
        area.x + area.width / 4,
        popup_area.y + popup_area.height + 1,
        area.width / 2,
        10,
    );

    ratatui::widgets::Clear.render(list_area, frame.buffer_mut());

    let list_block = Block::bordered()
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(RosePine::SUBTLE));

    let inner_list_area = list_block.inner(list_area);
    list_block.render(list_area, frame.buffer_mut());

    let color_items: Vec<ListItem> = colors
        .iter()
        .enumerate()
        .map(|(i, (name, color))| {
            let content = format!("■ {}", name);

            let style = if i == app.selected_language % colors.len() {
                Style::default().fg(*color).bold()
            } else {
                Style::default().fg(*color)
            };

            ListItem::new(content).style(style)
        })
        .collect();

    let color_list = List::new(color_items)
        .highlight_style(
            Style::default()
                .fg(RosePine::BASE)
                .bg(RosePine::LOVE)
                .bold(),
        )
        .highlight_symbol("▶ ");

    let mut list_state = ListState::default();
    list_state.select(Some(app.selected_language % colors.len()));

    frame.render_stateful_widget(color_list, inner_list_area, &mut list_state);
}

fn render_message_overlay(frame: &mut Frame, area: Rect, message: &str, is_error: bool) {
    let popup_area = spotlight_bar(70, area);

    ratatui::widgets::Clear.render(popup_area, frame.buffer_mut());

    let (icon, color) = if is_error {
        ("✗", RosePine::LOVE)
    } else {
        ("✓", RosePine::FOAM)
    };

    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(RosePine::SUBTLE).bg(RosePine::SURFACE));

    let inner_area = block.inner(popup_area);
    block.render(popup_area, frame.buffer_mut());

    let chunks = Layout::horizontal([
        Constraint::Length(3),
        Constraint::Min(10),
        Constraint::Length(24),
    ])
    .split(inner_area);

    let icon_paragraph = Paragraph::new(icon)
        .alignment(Alignment::Center)
        .style(Style::default().fg(color).bold());
    icon_paragraph.render(chunks[0], frame.buffer_mut());

    let message_paragraph = Paragraph::new(message)
        .alignment(Alignment::Left)
        .style(Style::default().fg(RosePine::TEXT));
    message_paragraph.render(chunks[1], frame.buffer_mut());

    let help_text = "Press any key to dismiss";
    let help_paragraph = Paragraph::new(help_text)
        .alignment(Alignment::Right)
        .style(Style::default().fg(RosePine::MUTED));
    help_paragraph.render(chunks[2], frame.buffer_mut());
}

// Helper function to create a centered bar for overlays
fn spotlight_bar(width_percent: u16, r: Rect) -> Rect {
    let layout = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Min(1),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - width_percent) / 2),
        Constraint::Percentage(width_percent),
        Constraint::Percentage((100 - width_percent) / 2),
    ])
    .split(layout[1])[1]
}

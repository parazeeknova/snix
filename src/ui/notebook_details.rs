use crate::app::{App, CodeSnippetsState};
use crate::models::SnippetLanguage;
use crate::ui::colors::RosePine;
use ratatui::widgets::Widget;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{BarChart, Block, BorderType, Cell, Paragraph, Row, Table, Wrap},
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

    // Create a layout for the different sections
    let chunks = Layout::vertical([
        Constraint::Length(10), // Overview information
        Constraint::Length(12), // Language distribution
        Constraint::Min(5),     // Snippets list
        Constraint::Length(1),  // Status line
    ])
    .split(inner_area);

    // Render status line
    let status_text = "← Back (Esc) • 's' to Create Snippet";
    let status = Paragraph::new(status_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(RosePine::MUTED));
    status.render(chunks[3], frame.buffer_mut());

    // Calculate statisticss
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

    let overview_area = overview_block.inner(chunks[0]);
    overview_block.render(chunks[0], frame.buffer_mut());

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

    let lang_area = lang_block.inner(chunks[1]);
    lang_block.render(chunks[1], frame.buffer_mut());

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

    // 3. SNIPPETS LIST SECTION
    let snippets_block = Block::bordered()
        .title(" Snippets ")
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(RosePine::SUBTLE));

    let snippets_area = snippets_block.inner(chunks[2]);
    snippets_block.render(chunks[2], frame.buffer_mut());

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
}

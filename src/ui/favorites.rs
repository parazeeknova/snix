use crate::app::App;
use crate::ui::colors::RosePine;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, BorderType, Cell, Clear, Paragraph, Row, Table, Widget},
};

/// Render the favorites window as a floating popup
pub fn render_floating_favorites(frame: &mut Frame, app: &App) {
    // Don't render if the popup shouldn't be shown
    if !app.show_favorites_popup {
        return;
    }

    let area = frame.area();
    let popup_width = 100;
    let popup_height = 30;

    let popup_area = Rect::new(
        (area.width.saturating_sub(popup_width)) / 2,
        (area.height.saturating_sub(popup_height)) / 2,
        popup_width.min(area.width),
        popup_height.min(area.height),
    );

    Clear.render(popup_area, frame.buffer_mut());

    let popup_block = Block::bordered()
        .title(" ★ Favorites ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(RosePine::LOVE));

    let inner_area = popup_block.inner(popup_area);
    popup_block.render(popup_area, frame.buffer_mut());

    // Get favorited snippets
    let favorite_snippets: Vec<_> = app
        .snippet_database
        .snippets
        .values()
        .filter(|s| s.is_favorited())
        .collect();

    if favorite_snippets.is_empty() {
        let no_favorites =
            Paragraph::new("No favorites yet. Press 'f' on a snippet to mark it as a favorite.")
                .alignment(Alignment::Center)
                .style(Style::default().fg(RosePine::MUTED));
        no_favorites.render(inner_area, frame.buffer_mut());

        // Show help at the bottom
        let help_text = "Press Esc to close";
        let help_paragraph = Paragraph::new(help_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(RosePine::MUTED));

        let chunks =
            Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).split(inner_area);

        help_paragraph.render(chunks[1], frame.buffer_mut());
        return;
    }

    // Create a table with favorites
    let header = Row::new(vec![
        Cell::from("Title").style(Style::default().fg(RosePine::IRIS).bold()),
        Cell::from("Language").style(Style::default().fg(RosePine::IRIS).bold()),
        Cell::from("Description").style(Style::default().fg(RosePine::IRIS).bold()),
        Cell::from("Tags").style(Style::default().fg(RosePine::IRIS).bold()),
    ]);

    let rows: Vec<Row> = favorite_snippets
        .iter()
        .map(|snippet| {
            let tags_display = if snippet.tags.is_empty() {
                "-".to_string()
            } else {
                snippet.get_tags_display_string()
            };

            let description = snippet.description.clone().unwrap_or_default();
            let truncated_desc = if description.len() > 40 {
                format!("{}...", &description[..37])
            } else {
                description
            };

            Row::new(vec![
                Cell::from(snippet.title.clone()).style(Style::default().fg(RosePine::TEXT)),
                Cell::from(format!(
                    "{} {}",
                    snippet.language.icon(),
                    snippet.language.short_name()
                ))
                .style(Style::default().fg(RosePine::FOAM)),
                Cell::from(truncated_desc).style(Style::default().fg(RosePine::PINE)),
                Cell::from(tags_display).style(Style::default().fg(RosePine::GOLD)),
            ])
        })
        .collect();

    let chunks = Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).split(inner_area);

    let table = Table::new(
        rows,
        &[
            Constraint::Percentage(30),
            Constraint::Percentage(15),
            Constraint::Percentage(35),
            Constraint::Percentage(20),
        ],
    )
    .header(header)
    .block(Block::default())
    .column_spacing(1);

    table.render(chunks[0], frame.buffer_mut());

    let help_text = "Press Esc to close";
    let help_paragraph = Paragraph::new(help_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(RosePine::MUTED));
    help_paragraph.render(chunks[1], frame.buffer_mut());
}

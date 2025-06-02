use crate::app::App;
use crate::ui::colors::RosePine;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, BorderType, Cell, Paragraph, Row, Table, Widget},
};

/// Render the favorites/pinned snippets section for the home screen
///
/// This will be used later on ignore this warning

pub fn render_favorites_section(frame: &mut Frame, area: Rect, app: &App) {
    let favorites_area = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(80),
        Constraint::Fill(1),
    ])
    .split(area)[1];

    let favorites_block = Block::bordered()
        .title("  Favorites [press 'f' to toggle] ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(RosePine::LOVE));

    let inner_area = favorites_block.inner(favorites_area);
    favorites_block.render(favorites_area, frame.buffer_mut());

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
        return;
    }

    // Create a table with favorites
    let header = Row::new(vec![
        Cell::from("Title").style(Style::default().fg(RosePine::IRIS).bold()),
        Cell::from("Language").style(Style::default().fg(RosePine::IRIS).bold()),
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

            Row::new(vec![
                Cell::from(snippet.title.clone()).style(Style::default().fg(RosePine::TEXT)),
                Cell::from(format!(
                    "{} {}",
                    snippet.language.icon(),
                    snippet.language.short_name()
                ))
                .style(Style::default().fg(RosePine::FOAM)),
                Cell::from(tags_display).style(Style::default().fg(RosePine::GOLD)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        &[
            Constraint::Percentage(40),
            Constraint::Percentage(20),
            Constraint::Percentage(40),
        ],
    )
    .header(header)
    .block(Block::default())
    .column_spacing(1);

    table.render(inner_area, frame.buffer_mut());
}

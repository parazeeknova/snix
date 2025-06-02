use crate::app::App;
use crate::ui::colors::RosePine;
use crate::ui::components::render_bottom_bar;
use chrono::{DateTime, Utc};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, List, ListItem, ListState, Paragraph, Widget},
};

/// Main rendering function for the start page
pub fn render(frame: &mut Frame, app: &mut App) {
    let main_area = frame.area();

    let block = Block::bordered()
        .title(" snix - Template & Boilerplate Manager ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(RosePine::HIGHLIGHT_HIGH));

    let inner_area = block.inner(main_area);
    block.render(main_area, frame.buffer_mut());
    let has_recent_snippets = !app.snippet_database.snippets.is_empty();

    let main_chunks = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(2),
        Constraint::Length(if has_recent_snippets { 12 } else { 0 }),
        Constraint::Length(3),
    ])
    .split(inner_area);

    let content_area = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(60),
        Constraint::Fill(1),
    ])
    .split(main_chunks[0])[1];

    let content_chunks = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(10),
        Constraint::Length(2),
        Constraint::Length(16),
        Constraint::Fill(1),
    ])
    .split(content_area);

    render_title(frame, content_chunks[1]);
    render_disclaimer(frame, content_chunks[2]);
    render_menu(frame, content_chunks[3], app);
    render_description(frame, main_chunks[1], app);

    if has_recent_snippets {
        render_recent_snippets(frame, main_chunks[2], app);
    }

    // Always render bottom bar at the bottom position
    render_bottom_bar(frame, main_chunks[3], app);
}

fn render_title(frame: &mut Frame, area: Rect) {
    let title_text = create_rustui_ascii_title();

    let title = Paragraph::new(title_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(RosePine::ROSE).bold());

    title.render(area, frame.buffer_mut());
}

fn create_rustui_ascii_title() -> String {
    r#"
███████╗███╗   ██╗██╗██╗  ██╗
██╔════╝████╗  ██║██║╚██╗██╔╝
███████╗██╔██╗ ██║██║ ╚███╔╝ 
╚════██║██║╚██╗██║██║ ██╔██╗
███████║██║ ╚████║██║██╔╝ ██╗
╚══════╝╚═╝  ╚═══╝╚═╝╚═╝  ╚═╝
"#
    .to_string()
}

fn render_disclaimer(frame: &mut Frame, area: Rect) {
    let disclaimer = "Fast ⚡ Boilerplate & Code snippets manager";

    let disclaimer_paragraph = Paragraph::new(disclaimer)
        .alignment(Alignment::Center)
        .style(Style::default().fg(RosePine::SUBTLE).italic());

    disclaimer_paragraph.render(area, frame.buffer_mut());
}

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
        ("󰘦", "Boilerplates", "b"),
        ("󰓜", "Marketplace", "m"),
        ("", "Code Snippets", "s"),
        ("", "Export/Import", "e"),
        ("󱞁", "Backup & Restore", "u"),
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
        "Import and export snippets/notebooks in JSON or YAML format",
        "Backup and restore your data, view backup history, and manage backups",
        "Customize your development workflow and preferences",
        "Save your work and exit the application",
    ];

    let description = descriptions.get(app.selected_menu_item).unwrap_or(&"");

    let description_paragraph = Paragraph::new(*description)
        .alignment(Alignment::Center)
        .style(Style::default().fg(RosePine::MUTED));

    description_paragraph.render(area, frame.buffer_mut());
}

/// Renders recent snippets section below the main content
fn render_recent_snippets(frame: &mut Frame, area: Rect, app: &App) {
    let snippets_area = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(80),
        Constraint::Fill(1),
    ])
    .split(area)[1];

    let block = Block::bordered()
        .title(" ⏱ Recent Snippets [1-10 to open] ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(RosePine::SUBTLE));

    let inner_area = block.inner(snippets_area);
    block.render(snippets_area, frame.buffer_mut());

    // Get most recently accessed snippets
    let mut recent_snippets: Vec<_> = app.snippet_database.snippets.values().collect();
    recent_snippets.sort_by(|a, b| b.accessed_at.cmp(&a.accessed_at));
    recent_snippets.truncate(10);

    if recent_snippets.is_empty() {
        let empty_text = Paragraph::new("No snippets accessed yet. Press 's' to create some!")
            .alignment(Alignment::Center)
            .style(Style::default().fg(RosePine::MUTED));
        empty_text.render(inner_area, frame.buffer_mut());
        return;
    }

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

            let ago = format_time_ago(&snippet.accessed_at);
            let lines = snippet.get_line_count();
            let language = snippet.language.display_name();

            // Format the title to truncate if necessary
            let title = if snippet.title.len() > 20 {
                format!("{}...", &snippet.title[..17])
            } else {
                snippet.title.clone()
            };

            // Create styled spans for the content
            let shortcut = format!("[{}]", i + 1);

            // Create a line with styled spans
            let notebook_span = Span::styled(
                format!("{}", notebook_name),
                Style::default().fg(notebook_color),
            );

            let spans = vec![
                Span::raw(format!("{} ", shortcut)),
                Span::styled(format!("{} ", icon), Style::default().fg(RosePine::GOLD)),
                Span::styled(title, Style::default().fg(RosePine::TEXT).bold()),
                Span::raw(" • "),
                notebook_span,
                Span::raw(format!(" • {} • {} lines • {}", language, lines, ago)),
            ];

            ListItem::new(Line::from(spans)).style(
                Style::default()
                    .fg(if i == 0 {
                        RosePine::LOVE
                    } else {
                        RosePine::TEXT
                    })
                    .bg(if i % 2 == 0 {
                        RosePine::HIGHLIGHT_LOW
                    } else {
                        RosePine::BASE
                    }),
            )
        })
        .collect();

    let list = List::new(items)
        .style(Style::default().fg(RosePine::TEXT))
        .highlight_style(Style::default().fg(RosePine::GOLD));

    list.render(inner_area, frame.buffer_mut());
}

/// Format time difference as human-readable string
fn format_time_ago(datetime: &DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(*datetime);

    if duration.num_days() > 0 {
        format!("{} days ago", duration.num_days())
    } else if duration.num_hours() > 0 {
        format!("{} hours ago", duration.num_hours())
    } else if duration.num_minutes() > 0 {
        format!("{} min ago", duration.num_minutes())
    } else {
        "Just now".to_string()
    }
}

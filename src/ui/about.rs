use crate::app::App;
use crate::ui::colors::RosePine;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Clear, Paragraph, Tabs, Widget},
};

/// Render the About window as a floating popup
pub fn render_about(frame: &mut Frame, app: &App) {
    // Don't render if the popup shouldn't be shown
    if !app.show_about_popup {
        return;
    }

    let area = frame.area();
    let popup_width = 100;
    let popup_height = 40;

    let popup_area = Rect::new(
        (area.width.saturating_sub(popup_width)) / 2,
        (area.height.saturating_sub(popup_height)) / 2,
        popup_width.min(area.width),
        popup_height.min(area.height),
    );

    Clear.render(popup_area, frame.buffer_mut());

    let popup_block = Block::bordered()
        .title(" ⓘ About Snix ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(RosePine::IRIS));

    let inner_area = popup_block.inner(popup_area);
    popup_block.render(popup_area, frame.buffer_mut());

    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .split(inner_area);

    let tab_titles = vec![
        "Overview",
        "Features",
        "Keybindings",
        "Analytics",
        "Credits",
    ];
    let tabs = Tabs::new(tab_titles)
        .select(app.selected_about_tab)
        .style(Style::default().fg(RosePine::SUBTLE))
        .highlight_style(Style::default().fg(RosePine::LOVE).bold())
        .divider("")
        .padding(" ", " ");

    frame.render_widget(tabs, chunks[0]);

    match app.selected_about_tab {
        0 => render_overview_tab(frame, chunks[1], app),
        1 => render_features_tab(frame, chunks[1], app),
        2 => render_keybindings_tab(frame, chunks[1], app),
        3 => render_analytics_tab(frame, chunks[1], app),
        4 => render_credits_tab(frame, chunks[1], app),
        _ => {}
    }

    let help_text = "Tab: Switch sections • ←/→: Navigate tabs • Esc: Close";
    let help_paragraph = Paragraph::new(help_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(RosePine::MUTED));
    help_paragraph.render(chunks[2], frame.buffer_mut());
}

fn render_overview_tab(frame: &mut Frame, area: Rect, app: &App) {
    let version = env!("CARGO_PKG_VERSION");

    let mut text = Vec::new();

    // Logo and version
    text.push(Line::from(vec![Span::styled(
        "  ███████╗███╗   ██╗██╗██╗  ██╗  ",
        Style::default().fg(RosePine::LOVE),
    )]));
    text.push(Line::from(vec![Span::styled(
        "  ██╔════╝████╗  ██║██║╚██╗██╔╝  ",
        Style::default().fg(RosePine::LOVE),
    )]));
    text.push(Line::from(vec![Span::styled(
        "  ███████╗██╔██╗ ██║██║ ╚███╔╝   ",
        Style::default().fg(RosePine::LOVE),
    )]));
    text.push(Line::from(vec![Span::styled(
        "  ╚════██║██║╚██╗██║██║ ██╔██╗   ",
        Style::default().fg(RosePine::LOVE),
    )]));
    text.push(Line::from(vec![Span::styled(
        "  ███████║██║ ╚████║██║██╔╝ ██╗  ",
        Style::default().fg(RosePine::LOVE),
    )]));
    text.push(Line::from(vec![Span::styled(
        "  ╚══════╝╚═╝  ╚═══╝╚═╝╚═╝  ╚═╝  ",
        Style::default().fg(RosePine::LOVE),
    )]));

    text.push(Line::from(""));
    text.push(Line::from(vec![Span::styled(
        format!("Version: {}", version),
        Style::default().fg(RosePine::GOLD).bold(),
    )]));
    text.push(Line::from(""));

    // App description
    text.push(Line::from(vec![
        Span::styled("Snix", Style::default().fg(RosePine::IRIS).bold()),
        Span::raw(" is a powerful snippet manager for developers that lets you organize, search,"),
    ]));
    text.push(Line::from(
        "and access your code snippets with ease. Built with Rust using the ratatui framework.",
    ));
    text.push(Line::from(""));

    // Stats summary
    text.push(Line::from(vec![Span::styled(
        "Your Snix Library",
        Style::default().fg(RosePine::FOAM).bold(),
    )]));
    text.push(Line::from(""));

    let num_notebooks = app.snippet_database.notebooks.len();
    let num_snippets = app.snippet_database.snippets.len();
    let num_favorites = app
        .snippet_database
        .snippets
        .values()
        .filter(|s| s.is_favorited())
        .count();

    text.push(Line::from(vec![
        Span::styled("  󰠮 ", Style::default().fg(RosePine::GOLD)),
        Span::styled(
            format!("{} Notebooks", num_notebooks),
            Style::default().fg(RosePine::TEXT),
        ),
    ]));
    text.push(Line::from(vec![
        Span::styled("  󰈮 ", Style::default().fg(RosePine::PINE)),
        Span::styled(
            format!("{} Snippets", num_snippets),
            Style::default().fg(RosePine::TEXT),
        ),
    ]));
    text.push(Line::from(vec![
        Span::styled("   ", Style::default().fg(RosePine::LOVE)),
        Span::styled(
            format!("{} Favorites", num_favorites),
            Style::default().fg(RosePine::TEXT),
        ),
    ]));

    text.push(Line::from(""));
    text.push(Line::from(vec![Span::styled(
        "Use Tab to navigate through the sections for more information.",
        Style::default().fg(RosePine::SUBTLE).italic(),
    )]));

    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(Block::default());

    frame.render_widget(paragraph, area);
}

fn render_features_tab(frame: &mut Frame, area: Rect, _app: &App) {
    let features = vec![
        (
            "󰠮 Notebooks Organization",
            "Create nested notebooks to organize your snippets.",
        ),
        (
            " Tagging System",
            "Tag your snippets for easier categorization and searching.",
        ),
        (
            "  Favorites",
            "Mark frequently used snippets as favorites for quick access.",
        ),
        (
            "  Search",
            "Powerful search functionality to find snippets by content or metadata.",
        ),
        (
            "  Analytics",
            "View statistics about your snippet collection.",
        ),
        (
            "󰆓  Import/Export",
            "Export and import your snippets in JSON, YAML, or TOML formats.",
        ),
        (
            "  Backup/Restore",
            "Create backups of your snippets and restore them when needed.",
        ),
        (
            "  Modern UI",
            "Terminal-based UI with a modern look and feel.",
        ),
        (
            "  Syntax Highlighting",
            "Beautiful syntax highlighting for various programming languages.",
        ),
        (
            "󰌌  Keyboard Shortcuts",
            "Efficient keyboard-driven interface.",
        ),
        (
            "  External Editor",
            "Edit your snippets in your favorite external editor.",
        ),
        (
            "󰥔  Recent Snippets",
            "Quick access to recently used snippets.",
        ),
    ];

    let chunks = Layout::vertical([Constraint::Length(2), Constraint::Min(1)]).split(area);

    let title = Paragraph::new("󱓞 Features")
        .style(Style::default().fg(RosePine::GOLD).bold())
        .alignment(Alignment::Center);
    frame.render_widget(title, chunks[0]);

    let features_area = chunks[1];
    let columns = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(features_area);

    let left_features = features[0..(features.len() / 2).min(features.len())].to_vec();
    let right_features = if features.len() > 1 {
        features[(features.len() / 2)..].to_vec()
    } else {
        vec![]
    };

    let left_text = left_features
        .iter()
        .map(|(title, desc)| {
            vec![
                Line::from(vec![Span::styled(
                    format!("{} ", title),
                    Style::default().fg(RosePine::IRIS).bold(),
                )]),
                Line::from(vec![Span::raw(format!("   {}", desc))]),
                Line::from(""),
            ]
        })
        .flatten()
        .collect::<Vec<Line>>();

    let left_paragraph = Paragraph::new(left_text)
        .alignment(Alignment::Left)
        .block(Block::default());
    frame.render_widget(left_paragraph, columns[0]);

    let right_text = right_features
        .iter()
        .map(|(title, desc)| {
            vec![
                Line::from(vec![Span::styled(
                    format!("{} ", title),
                    Style::default().fg(RosePine::IRIS).bold(),
                )]),
                Line::from(vec![Span::raw(format!("   {}", desc))]),
                Line::from(""),
            ]
        })
        .flatten()
        .collect::<Vec<Line>>();

    let right_paragraph = Paragraph::new(right_text)
        .alignment(Alignment::Left)
        .block(Block::default());
    frame.render_widget(right_paragraph, columns[1]);
}

fn render_keybindings_tab(frame: &mut Frame, area: Rect, _app: &App) {
    let global_keys = vec![
        ("Shift+?", "Toggle help menu"),
        ("q", "Quit (from start page)"),
        ("Esc", "Go back/close overlay"),
        ("h", "Go to home page"),
        ("Shift+F", "Show favorites popup"),
        ("Backspace", "Navigate back"),
    ];

    let navigation_keys = vec![
        ("↑/k", "Move up"),
        ("↓/j", "Move down"),
        ("Enter", "Select item"),
        ("/", "Search snippets"),
        ("Tab", "Cycle through options"),
    ];

    let snippet_keys = vec![
        ("s", "Create snippet"),
        ("f", "Toggle favorite"),
        ("y", "Copy to clipboard"),
        ("d", "Edit description"),
        ("t", "Edit tags"),
        ("x", "Delete item"),
    ];

    let notebook_keys = vec![
        ("n", "Create notebook"),
        ("b", "Create nested notebook"),
        ("Space", "Collapse/expand notebook"),
        ("v", "View notebook details"),
        ("Shift+↑", "Move item up"),
        ("Shift+↓", "Move item down"),
        ("Shift+←→", "Reorder siblings"),
    ];

    let chunks = Layout::vertical([Constraint::Length(2), Constraint::Min(1)]).split(area);

    let title = Paragraph::new("⌨️ Keyboard Shortcuts")
        .style(Style::default().fg(RosePine::GOLD).bold())
        .alignment(Alignment::Center);
    frame.render_widget(title, chunks[0]);

    let key_area = chunks[1];
    let rows =
        Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).split(key_area);

    let top_columns =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(rows[0]);

    let bottom_columns =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(rows[1]);

    let global_text = vec![
        Line::from(vec![Span::styled(
            "Global",
            Style::default().fg(RosePine::FOAM).bold(),
        )]),
        Line::from(""),
    ]
    .into_iter()
    .chain(global_keys.iter().map(|(key, desc)| {
        Line::from(vec![
            Span::styled(format!("{:8}", key), Style::default().fg(RosePine::GOLD)),
            Span::raw(format!("{}", desc)),
        ])
    }))
    .collect::<Vec<Line>>();

    let global_paragraph = Paragraph::new(global_text).block(Block::default());
    frame.render_widget(global_paragraph, top_columns[0]);

    let nav_text = vec![
        Line::from(vec![Span::styled(
            "Navigation",
            Style::default().fg(RosePine::FOAM).bold(),
        )]),
        Line::from(""),
    ]
    .into_iter()
    .chain(navigation_keys.iter().map(|(key, desc)| {
        Line::from(vec![
            Span::styled(format!("{:8}", key), Style::default().fg(RosePine::GOLD)),
            Span::raw(format!("{}", desc)),
        ])
    }))
    .collect::<Vec<Line>>();

    let nav_paragraph = Paragraph::new(nav_text).block(Block::default());
    frame.render_widget(nav_paragraph, top_columns[1]);

    let snippet_text = vec![
        Line::from(vec![Span::styled(
            "Snippets",
            Style::default().fg(RosePine::FOAM).bold(),
        )]),
        Line::from(""),
    ]
    .into_iter()
    .chain(snippet_keys.iter().map(|(key, desc)| {
        Line::from(vec![
            Span::styled(format!("{:8}", key), Style::default().fg(RosePine::GOLD)),
            Span::raw(format!("{}", desc)),
        ])
    }))
    .collect::<Vec<Line>>();

    let snippet_paragraph = Paragraph::new(snippet_text).block(Block::default());
    frame.render_widget(snippet_paragraph, bottom_columns[0]);

    let notebook_text = vec![
        Line::from(vec![Span::styled(
            "Notebooks",
            Style::default().fg(RosePine::FOAM).bold(),
        )]),
        Line::from(""),
    ]
    .into_iter()
    .chain(notebook_keys.iter().map(|(key, desc)| {
        Line::from(vec![
            Span::styled(format!("{:8}", key), Style::default().fg(RosePine::GOLD)),
            Span::raw(format!("{}", desc)),
        ])
    }))
    .collect::<Vec<Line>>();

    let notebook_paragraph = Paragraph::new(notebook_text).block(Block::default());
    frame.render_widget(notebook_paragraph, bottom_columns[1]);
}

fn render_analytics_tab(frame: &mut Frame, area: Rect, app: &App) {
    let snippets = &app.snippet_database.snippets;

    let mut language_counts = std::collections::HashMap::new();
    for snippet in snippets.values() {
        *language_counts
            .entry(snippet.language.short_name())
            .or_insert(0) += 1;
    }

    let mut languages: Vec<_> = language_counts.into_iter().collect();
    languages.sort_by(|a, b| b.1.cmp(&a.1));

    let total_snippets = snippets.len();
    let avg_size = if total_snippets > 0 {
        snippets.values().map(|s| s.content.len()).sum::<usize>() / total_snippets
    } else {
        0
    };

    let max_size = snippets
        .values()
        .map(|s| s.content.len())
        .max()
        .unwrap_or(0);

    // Calculate tag stats
    let tags_count = app.tag_manager.tags.len();
    let tagged_snippets = snippets.values().filter(|s| !s.tags.is_empty()).count();

    // Calculate time stats
    let now = chrono::Utc::now();
    let snippets_last_week = snippets
        .values()
        .filter(|s| now.signed_duration_since(s.created_at).num_days() < 7)
        .count();

    let snippets_last_month = snippets
        .values()
        .filter(|s| now.signed_duration_since(s.created_at).num_days() < 30)
        .count();

    let oldest_snippet = snippets
        .values()
        .min_by_key(|s| s.created_at)
        .map(|s| now.signed_duration_since(s.created_at).num_days())
        .unwrap_or(0);

    let chunks = Layout::vertical([Constraint::Length(2), Constraint::Min(1)]).split(area);

    let title = Paragraph::new(" Analytics")
        .style(Style::default().fg(RosePine::GOLD).bold())
        .alignment(Alignment::Center);
    frame.render_widget(title, chunks[0]);

    // Analytics in columns
    let analytics_area = chunks[1];
    let columns = Layout::horizontal([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(analytics_area);

    let left_area = columns[0];
    let left_chunks =
        Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).split(left_area);

    // General stats
    let stats_text = vec![
        Line::from(vec![Span::styled(
            "General Statistics",
            Style::default().fg(RosePine::FOAM).bold(),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "Total Notebooks: ",
                Style::default().fg(RosePine::TEXT).bold(),
            ),
            Span::raw(format!("{}", app.snippet_database.notebooks.len())),
        ]),
        Line::from(vec![
            Span::styled(
                "Total Snippets: ",
                Style::default().fg(RosePine::TEXT).bold(),
            ),
            Span::raw(format!("{}", total_snippets)),
        ]),
        Line::from(vec![
            Span::styled("Favorites: ", Style::default().fg(RosePine::TEXT).bold()),
            Span::raw(format!(
                "{}",
                app.snippet_database
                    .snippets
                    .values()
                    .filter(|s| s.is_favorited())
                    .count()
            )),
        ]),
        Line::from(vec![
            Span::styled(
                "Average Snippet Size: ",
                Style::default().fg(RosePine::TEXT).bold(),
            ),
            Span::raw(format!("{} characters", avg_size)),
        ]),
        Line::from(vec![
            Span::styled(
                "Largest Snippet: ",
                Style::default().fg(RosePine::TEXT).bold(),
            ),
            Span::raw(format!("{} characters", max_size)),
        ]),
        Line::from(vec![
            Span::styled("Tags Created: ", Style::default().fg(RosePine::TEXT).bold()),
            Span::raw(format!("{}", tags_count)),
        ]),
        Line::from(vec![
            Span::styled(
                "Tagged Snippets: ",
                Style::default().fg(RosePine::TEXT).bold(),
            ),
            Span::raw(format!(
                "{} ({}%)",
                tagged_snippets,
                if total_snippets > 0 {
                    tagged_snippets * 100 / total_snippets
                } else {
                    0
                }
            )),
        ]),
    ];

    let stats_paragraph = Paragraph::new(stats_text).block(Block::default());
    frame.render_widget(stats_paragraph, left_chunks[0]);

    // Recent activity
    let activity_text = vec![
        Line::from(vec![Span::styled(
            "Recent Activity",
            Style::default().fg(RosePine::FOAM).bold(),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "New snippets (last 7 days): ",
                Style::default().fg(RosePine::TEXT).bold(),
            ),
            Span::raw(format!("{}", snippets_last_week)),
        ]),
        Line::from(vec![
            Span::styled(
                "New snippets (last 30 days): ",
                Style::default().fg(RosePine::TEXT).bold(),
            ),
            Span::raw(format!("{}", snippets_last_month)),
        ]),
        Line::from(vec![
            Span::styled("Library age: ", Style::default().fg(RosePine::TEXT).bold()),
            Span::raw(format!("{} days since first snippet", oldest_snippet)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Most recent snippets:",
            Style::default().fg(RosePine::TEXT).bold(),
        )]),
    ];

    // Add most recent snippets (top 3)
    let mut recent_snippets: Vec<_> = snippets.values().collect();
    recent_snippets.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    let mut activity_lines = activity_text;
    for (i, snippet) in recent_snippets.iter().take(3).enumerate() {
        activity_lines.push(Line::from(vec![
            Span::raw(format!("{}. ", i + 1)),
            Span::styled(&snippet.title, Style::default().fg(RosePine::IRIS)),
            Span::raw(format!(" ({})", snippet.created_at.format("%Y-%m-%d"))),
        ]));
    }

    let activity_paragraph = Paragraph::new(activity_lines).block(Block::default());
    frame.render_widget(activity_paragraph, left_chunks[1]);

    // Right column - Language distribution
    let language_text = vec![
        Line::from(vec![Span::styled(
            "Language Distribution",
            Style::default().fg(RosePine::FOAM).bold(),
        )]),
        Line::from(""),
    ];

    let mut language_lines = language_text;
    for (lang, count) in languages.iter().take(10) {
        let percentage = if total_snippets > 0 {
            *count * 100 / total_snippets
        } else {
            0
        };
        language_lines.push(Line::from(vec![
            Span::styled(format!("{:8}", lang), Style::default().fg(RosePine::GOLD)),
            Span::raw(format!("{} snippets ({}%)", count, percentage)),
        ]));
    }

    let language_paragraph = Paragraph::new(language_lines).block(Block::default());
    frame.render_widget(language_paragraph, columns[1]);
}

fn render_credits_tab(frame: &mut Frame, area: Rect, _app: &App) {
    let text = vec![
        Line::from(vec![
            Span::styled("Snix", Style::default().fg(RosePine::LOVE).bold()),
            Span::styled(
                " is created with ♥  by parazeeknova",
                Style::default().fg(RosePine::TEXT),
            ),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Built with:",
            Style::default().fg(RosePine::FOAM).bold(),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Rust", Style::default().fg(RosePine::IRIS).bold()),
            Span::raw(
                " - A language empowering everyone to build reliable and efficient software.",
            ),
        ]),
        Line::from(vec![
            Span::styled("Ratatui", Style::default().fg(RosePine::IRIS).bold()),
            Span::raw(" - A Rust library to build rich terminal user interfaces."),
        ]),
        Line::from(vec![
            Span::styled("Syntect", Style::default().fg(RosePine::IRIS).bold()),
            Span::raw(" - Syntax highlighting for code snippets."),
        ]),
        Line::from(vec![
            Span::styled("Rose Pine", Style::default().fg(RosePine::IRIS).bold()),
            Span::raw(" - Color scheme inspired by nature."),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Special Thanks:",
            Style::default().fg(RosePine::FOAM).bold(),
        )]),
        Line::from(""),
        Line::from(
            "To the amazing Rust community for creating such wonderful tools and libraries.",
        ),
        Line::from(""),
        Line::from(vec![Span::styled(
            "License:",
            Style::default().fg(RosePine::FOAM).bold(),
        )]),
        Line::from(""),
        Line::from("Snix is open source software licensed under the MIT License."),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Source Code:",
            Style::default().fg(RosePine::FOAM).bold(),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "https://github.com/parazeeknova/snix",
            Style::default().fg(RosePine::IRIS),
        )]),
    ];

    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(Block::default());

    frame.render_widget(paragraph, area);
}

use crate::app::{App, CodeSnippetsState, InputMode, TreeItem};
use crate::ui::colors::RosePine;
use crate::ui::components::render_bottom_bar;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Clear, List, ListItem, ListState, Paragraph, Widget, Wrap},
};

pub fn render(frame: &mut Frame, app: &mut App) {
    let main_area = frame.area();

    if app.snippet_database.notebooks.is_empty() {
        render_welcome_screen(frame, main_area, app);
        return;
    }

    match app.code_snippets_state {
        CodeSnippetsState::NotebookList => render_main_view(frame, main_area, app),
        CodeSnippetsState::NotebookView { notebook_id } => {
            render_notebook_view(frame, main_area, app, notebook_id)
        }
        CodeSnippetsState::_SnippetEditor { snippet_id } => {
            render_snippet_editor(frame, main_area, app, snippet_id)
        }
        CodeSnippetsState::_CreateNotebook => render_create_notebook_dialog(frame, main_area, app),
        CodeSnippetsState::CreateSnippet { notebook_id } => {
            render_create_snippet_dialog(frame, main_area, app, notebook_id)
        }
        CodeSnippetsState::SearchSnippets => render_search_view(frame, main_area, app),
        CodeSnippetsState::Settings => render_settings_view(frame, main_area, app),
    }
}

fn render_welcome_screen(frame: &mut Frame, area: Rect, app: &mut App) {
    let block = Block::bordered()
        .title("  Code Snippets Manager ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(RosePine::HIGHLIGHT_HIGH));

    let inner_area = block.inner(area);
    block.render(area, frame.buffer_mut());

    let chunks = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(15),
        Constraint::Fill(1),
        Constraint::Length(3),
    ])
    .split(inner_area);

    let welcome_text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "★ Welcome to Code Snippets Manager!",
            Style::default().fg(RosePine::LOVE).bold(),
        )),
        Line::from(""),
        Line::from("You haven't created any notebooks yet."),
        Line::from("Notebooks are containers for organizing your code snippets."),
        Line::from(""),
        Line::from("Getting started:"),
        Line::from("• Press 'n' to create your first notebook"),
        Line::from("• Press 'b' to create a nested notebook inside another notebook"),
        Line::from("• Press 's' to add code snippets to your notebooks"),
        Line::from("• Notebooks are displayed with tree-sitter style indentation lines"),
        Line::from("• Use vim/nvim to edit your snippets with full LSP support"),
        Line::from(""),
        Line::from(Span::styled(
            "☀ Tips:",
            Style::default().fg(RosePine::GOLD).bold(),
        )),
        Line::from("• Use descriptive names for your notebooks"),
        Line::from("• Organize by project, language, or functionality"),
        Line::from("• Snippets support 20+ programming languages"),
    ];

    let welcome_paragraph = Paragraph::new(welcome_text)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(RosePine::TEXT));

    welcome_paragraph.render(chunks[1], frame.buffer_mut());
    render_bottom_bar(frame, chunks[3], app);
    render_overlays(frame, area, app);
}

fn render_main_view(frame: &mut Frame, area: Rect, app: &mut App) {
    let block = Block::bordered()
        .title("  Code Snippets Manager ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(RosePine::HIGHLIGHT_HIGH));

    let inner_area = block.inner(area);
    block.render(area, frame.buffer_mut());

    let main_chunks =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(3)]).split(inner_area);

    let content_chunks =
        Layout::horizontal([Constraint::Percentage(35), Constraint::Fill(1)]).split(main_chunks[0]);

    // Render the preview panel first (background)
    render_preview_panel(frame, content_chunks[1], app);

    // Then render the tree view (foreground)
    render_tree_view(frame, content_chunks[0], app);

    // Render the bottom bar
    render_bottom_bar(frame, main_chunks[1], app);

    // Render overlays last to ensure they appear on top
    render_overlays(frame, area, app);
}

/// Render all overlays (input dialogs, language selection, etc.)
/// This function should ALWAYS be called last to ensure overlays appear on top
fn render_overlays(frame: &mut Frame, area: Rect, app: &mut App) {
    match app.input_mode {
        InputMode::CreateNotebook
        | InputMode::CreateNestedNotebook
        | InputMode::CreateSnippet
        | InputMode::Search
        | InputMode::_RenameNotebook
        | InputMode::_RenameSnippet => {
            render_input_overlay(frame, area, app);
        }
        InputMode::SelectLanguage => {
            render_language_selection_overlay(frame, area, app);
        }
        InputMode::HelpMenu => {
            render_help_menu_overlay(frame, area, app);
        }
        InputMode::Normal => {
            if let Some(ref message) = app.error_message {
                render_message_overlay(frame, area, message, true);
            } else if let Some(ref message) = app.success_message {
                render_message_overlay(frame, area, message, false);
            }
        }
    }
}

/// Render language selection overlay
/// Renders a help menu overlay showing all available keyboard shortcuts
fn render_help_menu_overlay(frame: &mut Frame, area: Rect, _app: &mut App) {
    // Position the help menu in the bottom right corner
    let width = 60;
    let height = 24;
    let popup_area = Rect::new(
        area.width.saturating_sub(width + 2), // 2 cells padding from right edge
        area.height.saturating_sub(height + 2), // 2 cells padding from bottom edge
        width.min(area.width),
        height.min(area.height),
    );

    Clear.render(popup_area, frame.buffer_mut());

    let block = Block::bordered()
        .title(" 󰘳 Keyboard Shortcuts ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(RosePine::IRIS));

    let inner_area = block.inner(popup_area);
    block.render(popup_area, frame.buffer_mut());

    let shortcuts = vec![
        Line::from(Span::styled(
            "Navigation",
            Style::default().fg(RosePine::LOVE).bold(),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ↑/k ", Style::default().fg(RosePine::GOLD)),
            Span::raw("Move up"),
        ]),
        Line::from(vec![
            Span::styled("  ↓/j ", Style::default().fg(RosePine::GOLD)),
            Span::raw("Move down"),
        ]),
        Line::from(vec![
            Span::styled("  ⏎   ", Style::default().fg(RosePine::GOLD)),
            Span::raw("Select/Edit"),
        ]),
        Line::from(vec![
            Span::styled("  ←/h ", Style::default().fg(RosePine::GOLD)),
            Span::raw("Go back"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Notebooks",
            Style::default().fg(RosePine::LOVE).bold(),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  n   ", Style::default().fg(RosePine::GOLD)),
            Span::raw("Create root notebook"),
        ]),
        Line::from(vec![
            Span::styled("  b   ", Style::default().fg(RosePine::GOLD)),
            Span::raw("Create nested notebook inside selected notebook"),
        ]),
        Line::from(vec![
            Span::styled("  d   ", Style::default().fg(RosePine::GOLD)),
            Span::raw("Delete notebook/snippet"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Snippets",
            Style::default().fg(RosePine::LOVE).bold(),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  s   ", Style::default().fg(RosePine::GOLD)),
            Span::raw("Create snippet in current notebook"),
        ]),
        Line::from(vec![
            Span::styled("  /   ", Style::default().fg(RosePine::GOLD)),
            Span::raw("Search snippets"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "General",
            Style::default().fg(RosePine::LOVE).bold(),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ?   ", Style::default().fg(RosePine::GOLD)),
            Span::raw("Toggle this help menu"),
        ]),
        Line::from(vec![
            Span::styled("  h   ", Style::default().fg(RosePine::GOLD)),
            Span::raw("Go to home page"),
        ]),
        Line::from(vec![
            Span::styled("  q   ", Style::default().fg(RosePine::GOLD)),
            Span::raw("Quit application"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Press Esc or ? to close this menu",
            Style::default().fg(RosePine::SUBTLE).italic(),
        )),
    ];

    let help_paragraph = Paragraph::new(shortcuts)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(RosePine::TEXT));

    help_paragraph.render(inner_area, frame.buffer_mut());
}

fn render_language_selection_overlay(frame: &mut Frame, area: Rect, app: &mut App) {
    let popup_area = spotlight_bar(70, area);
    Clear.render(popup_area, frame.buffer_mut());
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(RosePine::SUBTLE).bg(RosePine::SURFACE));

    let inner_area = block.inner(popup_area);
    block.render(popup_area, frame.buffer_mut());

    let title = "Select Programming Language";
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

    let languages = get_available_languages();
    let selected_lang = &languages[app.selected_language];
    let selected_text = format!("{} {}", selected_lang.icon(), selected_lang.display_name());

    let dropdown_paragraph = Paragraph::new(selected_text)
        .alignment(Alignment::Left)
        .style(Style::default().fg(RosePine::TEXT));
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

    Clear.render(list_area, frame.buffer_mut());

    let list_block = Block::bordered()
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(RosePine::SUBTLE));

    let inner_list_area = list_block.inner(list_area);
    list_block.render(list_area, frame.buffer_mut());

    let language_items: Vec<ListItem> = languages
        .iter()
        .enumerate()
        .map(|(i, lang)| {
            let icon = lang.icon();
            let name = lang.display_name();
            let content = format!("{} {}", icon, name);

            let style = if i == app.selected_language {
                Style::default().fg(RosePine::LOVE).bold()
            } else {
                Style::default().fg(RosePine::TEXT)
            };

            ListItem::new(content).style(style)
        })
        .collect();

    let language_list = List::new(language_items)
        .highlight_style(
            Style::default()
                .fg(RosePine::BASE)
                .bg(RosePine::LOVE)
                .bold(),
        )
        .highlight_symbol("▶ ");

    let mut list_state = ListState::default();
    list_state.select(Some(app.selected_language));

    frame.render_stateful_widget(language_list, inner_list_area, &mut list_state);
}
fn render_message_overlay(frame: &mut Frame, area: Rect, message: &str, is_error: bool) {
    let popup_area = spotlight_bar(70, area);

    Clear.render(popup_area, frame.buffer_mut());

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

/// Get list of available languages for snippet creation
fn get_available_languages() -> Vec<crate::models::SnippetLanguage> {
    vec![
        crate::models::SnippetLanguage::Rust,
        crate::models::SnippetLanguage::JavaScript,
        crate::models::SnippetLanguage::TypeScript,
        crate::models::SnippetLanguage::Python,
        crate::models::SnippetLanguage::Go,
        crate::models::SnippetLanguage::Java,
        crate::models::SnippetLanguage::C,
        crate::models::SnippetLanguage::Cpp,
        crate::models::SnippetLanguage::CSharp,
        crate::models::SnippetLanguage::PHP,
        crate::models::SnippetLanguage::Ruby,
        crate::models::SnippetLanguage::Swift,
        crate::models::SnippetLanguage::Kotlin,
        crate::models::SnippetLanguage::Dart,
        crate::models::SnippetLanguage::HTML,
        crate::models::SnippetLanguage::CSS,
        crate::models::SnippetLanguage::SCSS,
        crate::models::SnippetLanguage::SQL,
        crate::models::SnippetLanguage::Bash,
        crate::models::SnippetLanguage::PowerShell,
        crate::models::SnippetLanguage::Yaml,
        crate::models::SnippetLanguage::Json,
        crate::models::SnippetLanguage::Xml,
        crate::models::SnippetLanguage::Markdown,
        crate::models::SnippetLanguage::Dockerfile,
        crate::models::SnippetLanguage::Toml,
        crate::models::SnippetLanguage::Ini,
        crate::models::SnippetLanguage::Config,
        crate::models::SnippetLanguage::Text,
    ]
}

fn render_tree_view(frame: &mut Frame, area: Rect, app: &mut App) {
    let block = Block::bordered()
        .title("  Notebooks & Snippets ")
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(RosePine::SUBTLE));

    let inner_area = block.inner(area);
    block.render(area, frame.buffer_mut());

    if app.tree_items.is_empty() {
        let empty_text = Paragraph::new("No notebooks found.\nPress 'n' to create one.")
            .alignment(Alignment::Center)
            .style(Style::default().fg(RosePine::MUTED));
        empty_text.render(inner_area, frame.buffer_mut());
        return;
    }

    // Before rendering, set the hovered item to the selected item if none is set
    // This ensures breadcrumbs always show something relevant
    if app.hovered_tree_item.is_none() {
        app.hovered_tree_item = Some(app.selected_tree_item);
    }

    let items: Vec<ListItem> = app
        .tree_items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let (name, style) = match item {
                TreeItem::Notebook(id, depth) => {
                    if let Some(notebook) = app.snippet_database.notebooks.get(id) {
                        let indent = create_tree_indent(*depth, false);
                        let icon = "";
                        let name = format!(
                            "{}{} {} ({})",
                            indent, icon, notebook.name, notebook.snippet_count
                        );
                        let style = if i == app.selected_tree_item {
                            Style::default().fg(RosePine::LOVE).bold()
                        } else {
                            Style::default().fg(RosePine::TEXT)
                        };
                        (name, style)
                    } else {
                        let indent = create_tree_indent(*depth, false);
                        let icon = "✗";
                        (
                            format!("{}{} Unknown Notebook", indent, icon),
                            Style::default().fg(RosePine::LOVE),
                        )
                    }
                }
                TreeItem::Snippet(id, depth) => {
                    if let Some(snippet) = app.snippet_database.snippets.get(id) {
                        let indent = create_tree_indent(*depth, true);
                        let icon = snippet.language.icon();
                        let name = format!("{}{} {}", indent, icon, snippet.title);
                        let style = if i == app.selected_tree_item {
                            Style::default().fg(RosePine::GOLD).bold()
                        } else {
                            Style::default().fg(RosePine::SUBTLE)
                        };
                        (name, style)
                    } else {
                        let indent = create_tree_indent(*depth, true);
                        let icon = "✗";
                        (
                            format!("{}{} Unknown Snippet", indent, icon),
                            Style::default().fg(RosePine::LOVE),
                        )
                    }
                }
            };

            ListItem::new(name).style(style)
        })
        .collect();

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .fg(RosePine::BASE)
                .bg(RosePine::LOVE)
                .bold(),
        )
        .highlight_symbol("▶ ");

    let mut list_state = ListState::default();
    list_state.select(Some(app.selected_tree_item));

    frame.render_stateful_widget(list, inner_area, &mut list_state);
}

/// Creates a tree-sitter style indentation with vertical lines
fn create_tree_indent(depth: usize, is_last_item: bool) -> String {
    if depth == 0 {
        return String::new();
    }

    let mut indent = String::new();

    for _ in 0..depth - 1 {
        indent.push_str("│ ");
    }

    if is_last_item {
        indent.push_str("└──");
    } else {
        indent.push_str("├──");
    }

    indent
}

fn render_preview_panel(frame: &mut Frame, area: Rect, app: &mut App) {
    let block = Block::bordered()
        .title("  Preview ")
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(RosePine::SUBTLE));

    let inner_area = block.inner(area);
    block.render(area, frame.buffer_mut());

    if let Some(selected_item) = app.get_selected_item() {
        match selected_item {
            TreeItem::Notebook(id, _) => {
                if let Some(notebook) = app.snippet_database.notebooks.get(id) {
                    render_notebook_preview(frame, inner_area, notebook, app);
                }
            }
            TreeItem::Snippet(id, _) => {
                if let Some(snippet) = app.snippet_database.snippets.get(id) {
                    render_snippet_preview(frame, inner_area, snippet);
                }
            }
        }
    } else {
        // Create a background block for the languages display
        let lang_block = Block::default()
            .style(Style::default().bg(RosePine::BASE))
            .borders(ratatui::widgets::Borders::NONE);
        lang_block.render(inner_area, frame.buffer_mut());

        // Display supported languages in an elegant grid layout
        let languages = get_available_languages();

        // Create a title for the languages display with a background
        let title_block = Block::default()
            .style(Style::default().bg(RosePine::SURFACE))
            .borders(ratatui::widgets::Borders::NONE);

        let title_area = Rect::new(inner_area.x, inner_area.y + 2, inner_area.width, 3);

        title_block.render(title_area, frame.buffer_mut());

        let title = Paragraph::new("Supported Languages")
            .alignment(Alignment::Center)
            .style(Style::default().fg(RosePine::IRIS).bold());

        title.render(title_area, frame.buffer_mut());

        let columns: usize = 3;
        let item_width = inner_area.width / columns as u16;
        let item_height = 3;

        // Create a grid of language items with background blocks
        for (i, lang) in languages.iter().enumerate() {
            let row = i / columns;
            let col = i % columns;

            let x = inner_area.x + (col as u16 * item_width);
            let y = inner_area.y + 7 + (row as u16 * item_height);
            let item_area = Rect::new(x, y, item_width, item_height);

            let item_block = Block::default()
                .style(Style::default().bg(if i % 2 == 0 {
                    RosePine::HIGHLIGHT_LOW
                } else {
                    RosePine::BASE
                }))
                .borders(ratatui::widgets::Borders::NONE);
            item_block.render(item_area, frame.buffer_mut());

            let icon = lang.icon();
            let name = lang.display_name();

            // Center the text vertically in the item
            let text_area = Rect::new(item_area.x + 2, item_area.y + 1, item_area.width - 4, 1);

            let lang_text = Paragraph::new(format!("  {} {}", icon, name))
                .alignment(Alignment::Left)
                .style(Style::default().fg(RosePine::TEXT));

            lang_text.render(text_area, frame.buffer_mut());
        }

        // Add a helpful message at the bottom with background
        let help_block = Block::default()
            .style(Style::default().bg(RosePine::SURFACE))
            .borders(ratatui::widgets::Borders::NONE);

        let help_area = Rect::new(
            inner_area.x,
            inner_area.y + inner_area.height - 4,
            inner_area.width,
            3,
        );

        help_block.render(help_area, frame.buffer_mut());

        let help_text = Paragraph::new(
            "Press 'n' to create a new notebook or select a notebook to add snippets",
        )
        .alignment(Alignment::Center)
        .style(Style::default().fg(RosePine::MUTED));

        let help_text_area = Rect::new(help_area.x, help_area.y + 1, help_area.width, 1);

        help_text.render(help_text_area, frame.buffer_mut());
    }
}

fn render_notebook_preview(
    frame: &mut Frame,
    area: Rect,
    notebook: &crate::models::Notebook,
    app: &App,
) {
    let chunks = Layout::vertical([Constraint::Length(8), Constraint::Fill(1)]).split(area);

    // Notebook info
    let info_lines = vec![
        Line::from(vec![
            Span::styled("󰠮 ", Style::default().fg(RosePine::GOLD)),
            Span::styled(&notebook.name, Style::default().fg(RosePine::TEXT).bold()),
        ]),
        Line::from(""),
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
        Line::from(vec![
            Span::styled("Snippets: ", Style::default().fg(RosePine::MUTED)),
            Span::styled(
                notebook.snippet_count.to_string(),
                Style::default().fg(RosePine::LOVE),
            ),
        ]),
        Line::from(""),
        Line::from(if let Some(desc) = &notebook.description {
            desc.clone()
        } else {
            "No description".to_string()
        })
        .style(Style::default().fg(RosePine::SUBTLE)),
    ];

    let info_paragraph = Paragraph::new(info_lines).wrap(Wrap { trim: true });

    info_paragraph.render(chunks[0], frame.buffer_mut());

    // Show snippets in this notebook
    let snippets: Vec<_> = app
        .snippet_database
        .snippets
        .values()
        .filter(|s| s.notebook_id == notebook.id)
        .collect();

    if !snippets.is_empty() {
        let snippet_items: Vec<ListItem> = snippets
            .iter()
            .map(|snippet| {
                let icon = snippet.language.icon();
                let name = format!(
                    "{} {} - {}",
                    icon,
                    snippet.title,
                    snippet.language.display_name()
                );
                ListItem::new(name).style(Style::default().fg(RosePine::TEXT))
            })
            .collect();

        let snippets_list = List::new(snippet_items)
            .block(
                Block::bordered()
                    .title(" Snippets ")
                    .border_type(BorderType::Rounded)
                    .style(Style::default().fg(RosePine::HIGHLIGHT_LOW)),
            )
            .style(Style::default().fg(RosePine::TEXT));

        snippets_list.render(chunks[1], frame.buffer_mut());
    }
}

fn render_snippet_preview(frame: &mut Frame, area: Rect, snippet: &crate::models::CodeSnippet) {
    let chunks = Layout::vertical([Constraint::Length(10), Constraint::Fill(1)]).split(area);

    // Snippet info
    let info_lines = vec![
        Line::from(vec![
            Span::styled(snippet.language.icon(), Style::default().fg(RosePine::GOLD)),
            Span::raw(" "),
            Span::styled(&snippet.title, Style::default().fg(RosePine::TEXT).bold()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Language: ", Style::default().fg(RosePine::MUTED)),
            Span::styled(
                snippet.language.display_name(),
                Style::default().fg(RosePine::FOAM),
            ),
        ]),
        Line::from(vec![
            Span::styled("Created: ", Style::default().fg(RosePine::MUTED)),
            Span::styled(
                snippet.created_at.format("%Y-%m-%d %H:%M").to_string(),
                Style::default().fg(RosePine::TEXT),
            ),
        ]),
        Line::from(vec![
            Span::styled("Updated: ", Style::default().fg(RosePine::MUTED)),
            Span::styled(
                snippet.updated_at.format("%Y-%m-%d %H:%M").to_string(),
                Style::default().fg(RosePine::TEXT),
            ),
        ]),
        Line::from(vec![
            Span::styled("Used: ", Style::default().fg(RosePine::MUTED)),
            Span::styled(
                format!("{} times", snippet.use_count),
                Style::default().fg(RosePine::GOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Lines: ", Style::default().fg(RosePine::MUTED)),
            Span::styled(
                snippet.get_line_count().to_string(),
                Style::default().fg(RosePine::TEXT),
            ),
        ]),
        Line::from(""),
        Line::from(if let Some(desc) = &snippet.description {
            desc.clone()
        } else {
            "No description".to_string()
        })
        .style(Style::default().fg(RosePine::SUBTLE)),
    ];

    let info_paragraph = Paragraph::new(info_lines).wrap(Wrap { trim: true });

    info_paragraph.render(chunks[0], frame.buffer_mut());

    // Show content preview with bat syntax highlighting when possible
    if !snippet.content.is_empty() {
        use std::io::Write;
        use std::process::{Command, Stdio};
        use std::str;

        let preview_content = snippet.get_preview(0);
        let title = format!(" Content Preview ({}) ", snippet.language.display_name());

        let content_block = Block::bordered()
            .title(title)
            .border_type(BorderType::Rounded)
            .style(Style::default().fg(RosePine::HIGHLIGHT_LOW));

        let inner_content_area = content_block.inner(chunks[1]);
        content_block.render(chunks[1], frame.buffer_mut());

        // Try to use bat for syntax highlighting if available
        let bat_available = Command::new("bat").arg("--version").output().is_ok();

        if bat_available {
            let language_arg = format!(
                "--language={}",
                snippet.language.display_name().to_lowercase()
            );

            // Use bat to generate highlighted output
            let mut bat_process = match Command::new("bat")
                .args(&[
                    "--color=always",
                    "--style=plain",
                    "--theme=ansi",
                    &language_arg,
                    "--paging=never",
                ])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
            {
                Ok(process) => process,
                Err(_) => {
                    display_regular_content(frame, inner_content_area, &preview_content, snippet);
                    return;
                }
            };

            if let Some(stdin) = bat_process.stdin.as_mut() {
                if stdin.write_all(preview_content.as_bytes()).is_err() {
                    display_regular_content(frame, inner_content_area, &preview_content, snippet);
                    return;
                }
            }

            // Read highlighted output
            match bat_process.wait_with_output() {
                Ok(output) if output.status.success() => {
                    if let Ok(highlighted) = str::from_utf8(&output.stdout) {
                        let content_paragraph = Paragraph::new(highlighted)
                            .wrap(Wrap { trim: false })
                            .scroll((0, 0))
                            .style(Style::default());

                        content_paragraph.render(inner_content_area, frame.buffer_mut());
                    } else {
                        display_regular_content(
                            frame,
                            inner_content_area,
                            &preview_content,
                            snippet,
                        );
                    }
                }
                _ => {
                    display_regular_content(frame, inner_content_area, &preview_content, snippet);
                }
            }
        } else {
            display_regular_content(frame, inner_content_area, &preview_content, snippet);
        }
    } else {
        let empty_text = Paragraph::new("Empty snippet\nPress Enter to edit")
            .alignment(Alignment::Center)
            .style(Style::default().fg(RosePine::MUTED));
        empty_text.render(chunks[1], frame.buffer_mut());
    }
}

fn display_regular_content(
    frame: &mut Frame,
    area: Rect,
    content: &str,
    snippet: &crate::models::CodeSnippet,
) {
    // Fallback to basic syntax highlighting
    let content_style = match snippet.language {
        crate::models::SnippetLanguage::Rust => Style::default().fg(RosePine::GOLD),
        crate::models::SnippetLanguage::Python => Style::default().fg(RosePine::FOAM),
        crate::models::SnippetLanguage::JavaScript => Style::default().fg(RosePine::ROSE),
        crate::models::SnippetLanguage::TypeScript => Style::default().fg(RosePine::IRIS),
        _ => Style::default().fg(RosePine::TEXT),
    };

    let content_paragraph = Paragraph::new(content)
        .wrap(Wrap { trim: false })
        .scroll((0, 0))
        .style(content_style);

    content_paragraph.render(area, frame.buffer_mut());
}

fn render_input_overlay(frame: &mut Frame, area: Rect, app: &mut App) {
    // Create a spotlight-style bar at the top of the screen
    let popup_area = spotlight_bar(70, area);

    // Clear the area where the spotlight bar will be rendered
    Clear.render(popup_area, frame.buffer_mut());

    let title = match app.input_mode {
        InputMode::CreateNotebook => "Create New Notebook",
        InputMode::CreateNestedNotebook => "Create Nested Notebook",
        InputMode::CreateSnippet => "Create New Snippet",
        InputMode::Search => "Search Snippets",
        InputMode::_RenameNotebook => "Rename Notebook",
        InputMode::_RenameSnippet => "Rename Snippet",
        _ => "Input",
    };

    // Create a sleek, modern block for the spotlight bar
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(RosePine::SUBTLE).bg(RosePine::SURFACE));
    let inner_area = block.inner(popup_area);
    block.render(popup_area, frame.buffer_mut());

    // Split the inner area into sections: title, input, and shortcuts
    let chunks = Layout::horizontal([
        Constraint::Length(title.len() as u16 + 4),
        Constraint::Min(10),
        Constraint::Length(24),
    ])
    .split(inner_area);

    // Render the title
    let title_paragraph = Paragraph::new(title)
        .alignment(Alignment::Left)
        .style(Style::default().fg(RosePine::IRIS).bold());
    title_paragraph.render(chunks[0], frame.buffer_mut());

    // Render the input field
    let input_text = format!("{}", app.input_buffer);
    let input_paragraph = Paragraph::new(input_text)
        .style(Style::default().fg(RosePine::TEXT))
        .alignment(Alignment::Left);
    input_paragraph.render(chunks[1], frame.buffer_mut());

    // Render the shortcuts
    let help_text = "⎋ Cancel • ⏎ Confirm";
    let help_paragraph = Paragraph::new(help_text)
        .alignment(Alignment::Right)
        .style(Style::default().fg(RosePine::MUTED));
    help_paragraph.render(chunks[2], frame.buffer_mut());
}

// Simplified placeholder views
fn render_notebook_view(frame: &mut Frame, area: Rect, _app: &App, _notebook_id: uuid::Uuid) {
    let paragraph = Paragraph::new("Detailed notebook view coming soon...")
        .alignment(Alignment::Center)
        .style(Style::default().fg(RosePine::TEXT));
    paragraph.render(area, frame.buffer_mut());
}

fn render_snippet_editor(frame: &mut Frame, area: Rect, _app: &App, _snippet_id: uuid::Uuid) {
    let paragraph = Paragraph::new("External editor integration active\nFile opened in vim/nvim")
        .alignment(Alignment::Center)
        .style(Style::default().fg(RosePine::TEXT));
    paragraph.render(area, frame.buffer_mut());
}

fn render_create_notebook_dialog(frame: &mut Frame, area: Rect, app: &mut App) {
    if app.input_mode == InputMode::CreateNotebook {
        render_input_overlay(frame, area, app);
    } else {
        let message = "Error: Not in notebook creation mode";
        render_message_overlay(frame, area, message, true);
    }
}

fn render_create_snippet_dialog(
    frame: &mut Frame,
    area: Rect,
    app: &mut App,
    notebook_id: uuid::Uuid,
) {
    match app.input_mode {
        InputMode::CreateSnippet => {
            // Check that we have a valid notebook
            // Show notebook name in the dialog
            if app.snippet_database.notebooks.contains_key(&notebook_id) {
                let notebook_name = app
                    .snippet_database
                    .notebooks
                    .get(&notebook_id)
                    .map(|n| n.name.as_str())
                    .unwrap_or("Unknown Notebook");

                let popup_area = spotlight_bar(70, area);
                Clear.render(popup_area, frame.buffer_mut());

                let block = Block::bordered()
                    .border_type(BorderType::Rounded)
                    .style(Style::default().fg(RosePine::SUBTLE).bg(RosePine::SURFACE));

                let inner_area = block.inner(popup_area);
                block.render(popup_area, frame.buffer_mut());

                let title = format!("Create Snippet in {}", notebook_name);
                let chunks = Layout::horizontal([
                    Constraint::Length(title.len() as u16 + 4), // Title section
                    Constraint::Min(10),                        // Input section
                    Constraint::Length(24),                     // Shortcuts section
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
            } else {
                let message = "Error: Selected notebook not found";
                render_message_overlay(frame, area, message, true);
            }
        }
        InputMode::SelectLanguage => {
            render_language_selection_overlay(frame, area, app);
        }
        _ => {
            let message = "Error: Not in snippet creation mode";
            render_message_overlay(frame, area, message, true);
        }
    }
}

fn render_search_view(frame: &mut Frame, area: Rect, _app: &mut App) {
    let paragraph = Paragraph::new("Search functionality coming soon...")
        .alignment(Alignment::Center)
        .style(Style::default().fg(RosePine::TEXT));
    paragraph.render(area, frame.buffer_mut());
}

fn render_settings_view(frame: &mut Frame, area: Rect, _app: &App) {
    let paragraph = Paragraph::new("Settings coming soon...")
        .alignment(Alignment::Center)
        .style(Style::default().fg(RosePine::TEXT));
    paragraph.render(area, frame.buffer_mut());
}

/// Creates a spotlight-style bar at the top of the screen with some margin
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

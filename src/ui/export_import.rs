use crate::app::App;
use crate::models::ExportFormat;
use crate::ui::colors::RosePine;
use crate::ui::components::render_bottom_bar;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Widget},
};
use std::path::PathBuf;

/// Export/Import state data
#[derive(Debug, Clone)]
pub struct ExportImportState {
    pub mode: ExportImportMode,
    pub export_format: ExportFormat,
    pub export_path: PathBuf,
    pub import_path: PathBuf,
    pub selected_option: usize,
    pub include_content: bool,
    pub favorites_only: bool,
    pub overwrite_existing: bool,
    pub status_message: Option<String>,
    pub is_error: bool,
}

impl Default for ExportImportState {
    fn default() -> Self {
        Self {
            mode: ExportImportMode::MainMenu,
            export_format: ExportFormat::JSON,
            export_path: PathBuf::from("snippets_export.json"),
            import_path: PathBuf::from(""),
            selected_option: 0,
            include_content: true,
            favorites_only: false,
            overwrite_existing: false,
            status_message: None,
            is_error: false,
        }
    }
}

/// Export/Import modes
#[derive(Debug, Clone, PartialEq)]
pub enum ExportImportMode {
    MainMenu,
    ExportOptions,
    ExportPath,
    ImportOptions,
    _ImportPath,
    ImportClipboard,
    Exporting,
    Importing,
    ImportPathPopup,
}

/// Main render function for the export/import page
pub fn render(frame: &mut Frame, app: &mut App) {
    // Get the export/import state
    let default_state = ExportImportState::default();
    let export_import_state = app.export_import_state.as_ref().unwrap_or(&default_state);

    // Clone the necessary parts to avoid borrow issues
    let current_mode = export_import_state.mode.clone();
    let status_message = export_import_state.status_message.clone();
    let is_error = export_import_state.is_error;

    let main_area = frame.area();

    let block = Block::bordered()
        .title(" Export & Import Manager ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(RosePine::HIGHLIGHT_HIGH));

    let inner_area = block.inner(main_area);
    block.render(main_area, frame.buffer_mut());

    let chunks = Layout::vertical([Constraint::Fill(1), Constraint::Length(3)]).split(inner_area);

    // Render the appropriate content based on the current mode
    match current_mode {
        ExportImportMode::MainMenu => render_main_menu(frame, chunks[0], app),
        ExportImportMode::ExportOptions => render_export_options(frame, chunks[0], app),
        ExportImportMode::ExportPath => render_export_path(frame, chunks[0], app),
        ExportImportMode::ImportOptions => render_import_options(frame, chunks[0], app),
        ExportImportMode::_ImportPath => render_import_path(frame, chunks[0], app),
        ExportImportMode::ImportClipboard => render_import_clipboard(frame, chunks[0], app),
        ExportImportMode::Exporting => render_exporting(frame, chunks[0], app),
        ExportImportMode::Importing => render_importing(frame, chunks[0], app),
        ExportImportMode::ImportPathPopup => {
            // First render the main menu as background
            render_main_menu(frame, chunks[0], app);
            // Then render the import path popup
            render_import_path_popup(frame, main_area, app);
        }
    }

    // Render the bottom bar
    render_bottom_bar(frame, chunks[1], app);

    // Render status message if present
    if let Some(message) = &status_message {
        render_status_message(frame, main_area, message, is_error);
    }
}

/// Render the main export/import menu
fn render_main_menu(frame: &mut Frame, area: Rect, app: &mut App) {
    let default_state = ExportImportState::default();
    let export_import_state = app.export_import_state.as_ref().unwrap_or(&default_state);

    let content_area = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(60),
        Constraint::Fill(1),
    ])
    .split(area)[1];

    let content_chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(10),
        Constraint::Fill(1),
    ])
    .split(content_area);

    // Title
    let title = Paragraph::new("Export & Import Snippets and Notebooks")
        .alignment(Alignment::Center)
        .style(Style::default().fg(RosePine::GOLD).bold());

    title.render(content_chunks[0], frame.buffer_mut());

    // Menu options with shortcut keys
    let menu_items = vec![
        (
            "  [E] Export to JSON",
            "Export snippets and notebooks to a JSON file",
        ),
        (
            "  [F] Import from File",
            "Import snippets and notebooks from a file",
        ),
        (
            "  [C] Import from Clipboard",
            "Import snippets and notebooks from clipboard",
        ),
    ];

    let list_items: Vec<ListItem> = menu_items
        .iter()
        .enumerate()
        .map(|(i, (name, desc))| {
            let style = if i == export_import_state.selected_option {
                Style::default().fg(RosePine::LOVE).bold()
            } else {
                Style::default().fg(RosePine::TEXT)
            };

            let prefix = if i == export_import_state.selected_option {
                "▶"
            } else {
                " "
            };

            let title_line = Line::from(vec![Span::styled(format!("{} {}", prefix, name), style)]);

            let desc_line = Line::from(vec![Span::styled(
                format!("   {}", desc),
                Style::default().fg(RosePine::SUBTLE),
            )]);

            ListItem::new(vec![title_line, desc_line, Line::from("")])
        })
        .collect();

    let menu_list = List::new(list_items)
        .block(Block::default())
        .highlight_style(
            Style::default()
                .fg(RosePine::LOVE)
                .bg(RosePine::HIGHLIGHT_LOW)
                .bold(),
        );

    let mut list_state = ListState::default();
    list_state.select(Some(export_import_state.selected_option));

    frame.render_stateful_widget(menu_list, content_chunks[1], &mut list_state);
}

/// Render the export options screen
fn render_export_options(frame: &mut Frame, area: Rect, app: &mut App) {
    let default_state = ExportImportState::default();
    let export_import_state = app.export_import_state.as_ref().unwrap_or(&default_state);

    let content_area = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(70),
        Constraint::Fill(1),
    ])
    .split(area)[1];

    let content_chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(12),
        Constraint::Fill(1),
    ])
    .split(content_area);

    // Title
    let title = Paragraph::new("Export Options (JSON)")
        .alignment(Alignment::Center)
        .style(Style::default().fg(RosePine::GOLD).bold());

    title.render(content_chunks[0], frame.buffer_mut());

    // Options
    let options = vec![
        (
            "Include snippet content",
            export_import_state.include_content,
            "Include the full content of snippets in the export",
        ),
        (
            "Export favorites only",
            export_import_state.favorites_only,
            "Only export snippets that are marked as favorites",
        ),
        (
            "Continue to select export path",
            true,
            "Proceed to select where to save the export file",
        ),
    ];

    let list_items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(i, (name, enabled, desc))| {
            let style = if i == export_import_state.selected_option {
                Style::default().fg(RosePine::LOVE).bold()
            } else {
                Style::default().fg(RosePine::TEXT)
            };

            let prefix = if i == export_import_state.selected_option {
                "▶"
            } else {
                " "
            };

            let checkbox = if i < 2 {
                if *enabled { "[✓]" } else { "[ ]" }
            } else {
                ""
            };

            let title_line = Line::from(vec![Span::styled(
                format!("{} {} {}", prefix, checkbox, name),
                style,
            )]);

            let desc_line = Line::from(vec![Span::styled(
                format!("   {}", desc),
                Style::default().fg(RosePine::SUBTLE),
            )]);

            ListItem::new(vec![title_line, desc_line, Line::from("")])
        })
        .collect();

    let options_list = List::new(list_items)
        .block(Block::default())
        .highlight_style(
            Style::default()
                .fg(RosePine::LOVE)
                .bg(RosePine::HIGHLIGHT_LOW)
                .bold(),
        );

    let mut list_state = ListState::default();
    list_state.select(Some(export_import_state.selected_option));

    frame.render_stateful_widget(options_list, content_chunks[1], &mut list_state);
}

/// Render the export path input screen
fn render_export_path(frame: &mut Frame, area: Rect, app: &mut App) {
    let content_area = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(70),
        Constraint::Fill(1),
    ])
    .split(area)[1];

    let content_chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(5),
        Constraint::Length(6),
        Constraint::Fill(1),
    ])
    .split(content_area);

    // Title
    let title = Paragraph::new("Export Path")
        .alignment(Alignment::Center)
        .style(Style::default().fg(RosePine::GOLD).bold());

    title.render(content_chunks[0], frame.buffer_mut());

    // Path input field
    let input_block = Block::bordered()
        .title(" Enter export file path ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(RosePine::SUBTLE));

    let inner_input_area = input_block.inner(content_chunks[1]);
    input_block.render(content_chunks[1], frame.buffer_mut());

    let input_text = Paragraph::new(&*app.input_buffer).style(Style::default().fg(RosePine::TEXT));

    input_text.render(inner_input_area, frame.buffer_mut());

    // Default location suggestions
    let suggestion_block = Block::bordered()
        .title(" Suggested locations ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(RosePine::SUBTLE));

    let inner_suggestion_area = suggestion_block.inner(content_chunks[2]);
    suggestion_block.render(content_chunks[2], frame.buffer_mut());

    let suggestions = [
        "snippets_export.json - Default export file",
        "~/Documents/snippets_export.json - Documents folder",
        "~/Downloads/snippets_export.json - Downloads folder",
    ];

    let suggestion_text =
        Paragraph::new(suggestions.join("\n")).style(Style::default().fg(RosePine::TEXT));

    suggestion_text.render(inner_suggestion_area, frame.buffer_mut());

    // Help text
    let help_text = Paragraph::new("Press Enter to confirm, Esc to cancel")
        .alignment(Alignment::Center)
        .style(Style::default().fg(RosePine::MUTED));

    help_text.render(content_chunks[3], frame.buffer_mut());
}

/// Render the import options screen
fn render_import_options(frame: &mut Frame, area: Rect, app: &mut App) {
    let default_state = ExportImportState::default();
    let export_import_state = app.export_import_state.as_ref().unwrap_or(&default_state);

    let content_area = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(70),
        Constraint::Fill(1),
    ])
    .split(area)[1];

    let content_chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(9),
        Constraint::Fill(1),
    ])
    .split(content_area);

    // Title
    let title = Paragraph::new("Import Options")
        .alignment(Alignment::Center)
        .style(Style::default().fg(RosePine::GOLD).bold());

    title.render(content_chunks[0], frame.buffer_mut());

    // Options
    let options = vec![
        (
            "Overwrite existing snippets and notebooks",
            export_import_state.overwrite_existing,
            "Replace snippets and notebooks with the same ID",
        ),
        (
            "Continue to select import file",
            true,
            "Proceed to select the file to import",
        ),
    ];

    let list_items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(i, (name, enabled, desc))| {
            let style = if i == export_import_state.selected_option {
                Style::default().fg(RosePine::LOVE).bold()
            } else {
                Style::default().fg(RosePine::TEXT)
            };

            let prefix = if i == export_import_state.selected_option {
                "▶"
            } else {
                " "
            };

            let checkbox = if i < 1 {
                if *enabled { "[✓]" } else { "[ ]" }
            } else {
                ""
            };

            let title_line = Line::from(vec![Span::styled(
                format!("{} {} {}", prefix, checkbox, name),
                style,
            )]);

            let desc_line = Line::from(vec![Span::styled(
                format!("   {}", desc),
                Style::default().fg(RosePine::SUBTLE),
            )]);

            ListItem::new(vec![title_line, desc_line, Line::from("")])
        })
        .collect();

    let options_list = List::new(list_items)
        .block(Block::default())
        .highlight_style(
            Style::default()
                .fg(RosePine::LOVE)
                .bg(RosePine::HIGHLIGHT_LOW)
                .bold(),
        );

    let mut list_state = ListState::default();
    list_state.select(Some(export_import_state.selected_option));

    frame.render_stateful_widget(options_list, content_chunks[1], &mut list_state);
}

/// Render the import path input screen
fn render_import_path(frame: &mut Frame, area: Rect, app: &mut App) {
    let content_area = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(70),
        Constraint::Fill(1),
    ])
    .split(area)[1];

    let content_chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(5),
        Constraint::Length(6),
        Constraint::Fill(1),
    ])
    .split(content_area);

    // Title
    let title = Paragraph::new("Import Path")
        .alignment(Alignment::Center)
        .style(Style::default().fg(RosePine::GOLD).bold());

    title.render(content_chunks[0], frame.buffer_mut());

    // Path input field
    let input_block = Block::bordered()
        .title(" Enter import file path ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(RosePine::SUBTLE));

    let inner_input_area = input_block.inner(content_chunks[1]);
    input_block.render(content_chunks[1], frame.buffer_mut());

    let input_text = Paragraph::new(&*app.input_buffer).style(Style::default().fg(RosePine::TEXT));

    input_text.render(inner_input_area, frame.buffer_mut());

    // Default location suggestions
    let suggestion_block = Block::bordered()
        .title(" Suggested files ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(RosePine::SUBTLE));

    let inner_suggestion_area = suggestion_block.inner(content_chunks[2]);
    suggestion_block.render(content_chunks[2], frame.buffer_mut());

    let suggestions = [
        "snippets_export.json - Default import file",
        "~/Documents/snippets_export.json - Documents folder",
        "~/Downloads/snippets_export.json - Downloads folder",
    ];

    let suggestion_text =
        Paragraph::new(suggestions.join("\n")).style(Style::default().fg(RosePine::TEXT));

    suggestion_text.render(inner_suggestion_area, frame.buffer_mut());

    // Help text
    let help_text = Paragraph::new("Press Enter to confirm, Esc to cancel")
        .alignment(Alignment::Center)
        .style(Style::default().fg(RosePine::MUTED));

    help_text.render(content_chunks[3], frame.buffer_mut());
}

/// Render the clipboard import screen
fn render_import_clipboard(frame: &mut Frame, area: Rect, _app: &mut App) {
    let content_area = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(70),
        Constraint::Fill(1),
    ])
    .split(area)[1];

    let content_chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(5),
        Constraint::Fill(1),
    ])
    .split(content_area);

    // Title
    let title = Paragraph::new("Import from Clipboard")
        .alignment(Alignment::Center)
        .style(Style::default().fg(RosePine::GOLD).bold());

    title.render(content_chunks[0], frame.buffer_mut());

    // Info message
    let info_block = Block::bordered()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(RosePine::SUBTLE));

    let inner_info_area = info_block.inner(content_chunks[1]);
    info_block.render(content_chunks[1], frame.buffer_mut());

    let info_text = Paragraph::new("Press Enter to import from clipboard, Esc to cancel")
        .alignment(Alignment::Center)
        .style(Style::default().fg(RosePine::TEXT));

    info_text.render(inner_info_area, frame.buffer_mut());

    // Help text
    let help_text = Paragraph::new("The clipboard should contain a valid JSON or YAML export")
        .alignment(Alignment::Center)
        .style(Style::default().fg(RosePine::MUTED));

    help_text.render(content_chunks[2], frame.buffer_mut());
}

/// Render the exporting status screen
fn render_exporting(frame: &mut Frame, area: Rect, _app: &mut App) {
    let content_area = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(70),
        Constraint::Fill(1),
    ])
    .split(area)[1];

    let content_chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(5),
        Constraint::Fill(1),
    ])
    .split(content_area);

    // Title
    let title = Paragraph::new("Exporting...")
        .alignment(Alignment::Center)
        .style(Style::default().fg(RosePine::GOLD).bold());

    title.render(content_chunks[0], frame.buffer_mut());

    // Status message
    let status_block = Block::bordered()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(RosePine::SUBTLE));

    let inner_status_area = status_block.inner(content_chunks[1]);
    status_block.render(content_chunks[1], frame.buffer_mut());

    let status_text = Paragraph::new("Exporting your snippets and notebooks...")
        .alignment(Alignment::Center)
        .style(Style::default().fg(RosePine::TEXT));

    status_text.render(inner_status_area, frame.buffer_mut());
}

/// Render the importing status screen
fn render_importing(frame: &mut Frame, area: Rect, _app: &mut App) {
    let content_area = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(70),
        Constraint::Fill(1),
    ])
    .split(area)[1];

    let content_chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(5),
        Constraint::Fill(1),
    ])
    .split(content_area);

    // Title
    let title = Paragraph::new("Importing...")
        .alignment(Alignment::Center)
        .style(Style::default().fg(RosePine::GOLD).bold());

    title.render(content_chunks[0], frame.buffer_mut());

    // Status message
    let status_block = Block::bordered()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(RosePine::SUBTLE));

    let inner_status_area = status_block.inner(content_chunks[1]);
    status_block.render(content_chunks[1], frame.buffer_mut());

    let status_text = Paragraph::new("Importing snippets and notebooks...")
        .alignment(Alignment::Center)
        .style(Style::default().fg(RosePine::TEXT));

    status_text.render(inner_status_area, frame.buffer_mut());
}

/// Render the import path as a popup overlay
fn render_import_path_popup(frame: &mut Frame, area: Rect, app: &mut App) {
    let popup_width = 70;
    let popup_height = 20;

    let popup_area = Rect::new(
        (area.width.saturating_sub(popup_width)) / 2,
        (area.height.saturating_sub(popup_height)) / 2,
        popup_width.min(area.width),
        popup_height.min(area.height),
    );

    // Clear the area where popup will be drawn
    Clear.render(popup_area, frame.buffer_mut());

    // Create a popup block
    let popup_block = Block::bordered()
        .title(" Import File ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(RosePine::HIGHLIGHT_HIGH));

    let inner_area = popup_block.inner(popup_area);
    popup_block.render(popup_area, frame.buffer_mut());

    // Split inner area into sections
    let chunks = Layout::vertical([
        Constraint::Length(2),  // Title
        Constraint::Length(3),  // Input field
        Constraint::Length(10), // Available files/autocomplete suggestions
        Constraint::Length(2),  // Help text
    ])
    .split(inner_area);

    // Title
    let title = Paragraph::new("Select file to import")
        .alignment(Alignment::Center)
        .style(Style::default().fg(RosePine::GOLD).bold());
    title.render(chunks[0], frame.buffer_mut());

    // Input field
    let input_block = Block::bordered()
        .title(" Enter import file path ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(RosePine::SUBTLE));

    let inner_input_area = input_block.inner(chunks[1]);
    input_block.render(chunks[1], frame.buffer_mut());

    let input_text = Paragraph::new(&*app.input_buffer).style(Style::default().fg(RosePine::TEXT));
    input_text.render(inner_input_area, frame.buffer_mut());

    // Autocompletion suggestions
    let suggestions_block = Block::bordered()
        .title(" Autocompletion ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(RosePine::SUBTLE));

    let inner_suggestions_area = suggestions_block.inner(chunks[2]);
    suggestions_block.render(chunks[2], frame.buffer_mut());

    // Try to get completions based on the current input
    let mut completion_suggestions = Vec::<String>::new();

    // Get potential path completions
    let path_str = app.input_buffer.trim();
    completion_suggestions.push("snippets_export.json - Default export file".to_string());
    completion_suggestions.push("~/Documents/snippets_export.json - Documents folder".to_string());
    completion_suggestions.push("~/Downloads/snippets_export.json - Downloads folder".to_string());

    // If input is not empty, try to get actual completions from filesystem
    if !path_str.is_empty() {
        // Expand tilde to home directory
        let expanded_path = if path_str.starts_with("~/") {
            if let Some(home) = dirs::home_dir() {
                home.join(path_str.trim_start_matches("~/"))
                    .display()
                    .to_string()
            } else {
                path_str.to_string()
            }
        } else {
            path_str.to_string()
        };

        // Get directory portion and filename portion
        let (dir_path, file_prefix) = match expanded_path.rfind('/') {
            Some(pos) => {
                let (dir, file) = expanded_path.split_at(pos + 1);
                (dir.to_string(), file.to_string())
            }
            None => {
                // No slash, assume current directory
                ("./".to_string(), expanded_path)
            }
        };

        // Try to read the directory and find matching files
        if let Ok(entries) = std::fs::read_dir(dir_path.clone()) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with(&file_prefix) {
                    let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
                    let path = format!("{}{}", dir_path, name);

                    if is_dir {
                        completion_suggestions.push(format!("{}/", path));
                    } else if name.ends_with(".json") {
                        completion_suggestions.push(path);
                    }
                }
            }
        }
    }

    let items: Vec<String> = completion_suggestions.into_iter().take(8).collect();

    let suggestions_text =
        Paragraph::new(items.join("\n")).style(Style::default().fg(RosePine::TEXT));

    suggestions_text.render(inner_suggestions_area, frame.buffer_mut());

    let help_text = Paragraph::new("Press Enter to import, Esc to cancel, Tab to autocomplete")
        .alignment(Alignment::Center)
        .style(Style::default().fg(RosePine::MUTED));
    help_text.render(chunks[3], frame.buffer_mut());
}

/// Render a status message as an overlay
fn render_status_message(frame: &mut Frame, area: Rect, message: &str, is_error: bool) {
    let popup_width = 60;
    let popup_height = 5;

    let popup_area = Rect::new(
        (area.width - popup_width) / 2,
        (area.height - popup_height) / 2,
        popup_width,
        popup_height,
    );

    Clear.render(popup_area, frame.buffer_mut());

    let (icon, color) = if is_error {
        ("✗", RosePine::LOVE)
    } else {
        ("✓", RosePine::FOAM)
    };

    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(color));

    let inner_area = block.inner(popup_area);
    block.render(popup_area, frame.buffer_mut());

    let title = if is_error { "Error" } else { "Success" };

    let content = vec![
        Line::from(vec![Span::styled(
            format!("{} {}", icon, title),
            Style::default().fg(color).bold(),
        )]),
        Line::from(vec![Span::styled(
            message,
            Style::default().fg(RosePine::TEXT),
        )]),
        Line::from(vec![Span::styled(
            "Press any key to continue",
            Style::default().fg(RosePine::MUTED),
        )]),
    ];

    let paragraph = Paragraph::new(content).alignment(Alignment::Center);

    paragraph.render(inner_area, frame.buffer_mut());
}

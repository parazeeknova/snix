use crate::app::{App, TreeItem};
use crate::ui::colors::RosePine;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Clear, Paragraph, Widget},
};

/// Renders the bottom navigation bar with breadcrumbs and keyboard shortcut
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
    use crate::app::{AppState, CodeSnippetsState, InputMode};
    use crate::ui::export_import::ExportImportMode;

    let back_hint = if app.can_go_back() { " [←] │ " } else { "" };

    match (&app.state, &app.input_mode) {
        (_, InputMode::CreateNotebook | InputMode::CreateSnippet | InputMode::Search) => {
            format!(" [⏎] Confirm │ [Esc] Cancel ")
        }
        (_, InputMode::SelectLanguage) => {
            format!(" [↑↓] Navigate │ [⏎] Select │ [Esc] Cancel ")
        }

        (AppState::StartPage, InputMode::Normal) => {
            format!(
                "{} [↑↓] Navigate │ [⏎] Select │ [/] Search │ [u] Backup │ [a] About │ [s] Snippets │ [q] Quit ",
                back_hint
            )
        }

        (AppState::CodeSnippets, InputMode::Normal) => {
            match &app.code_snippets_state {
                CodeSnippetsState::SearchSnippets => {
                    format!(
                        "{} [↑↓] Navigate │ [⏎] Open │ [Backspace] Edit Search │ [Esc] Back ",
                        back_hint
                    )
                }
                _ => {
                    if app.snippet_database.notebooks.is_empty() {
                        format!("{} [n 󰠮] │ [/ 󰭎] │ [h  ]│ [q 󰈆] ", back_hint)
                    } else {
                        // Check if a notebook is selected and can be collapsed/expanded
                        let collapse_text =
                            if let Some(TreeItem::Notebook(id, _)) = app.get_selected_item() {
                                if app.is_notebook_collapsed(id) {
                                    "[Space ]"
                                } else if app
                                    .snippet_database
                                    .notebooks
                                    .get(id)
                                    .map(|nb| !nb.children.is_empty() || nb.snippet_count > 0)
                                    .unwrap_or(false)
                                {
                                    "[Space ]"
                                } else {
                                    "[Space ]"
                                }
                            } else {
                                "[Space ]"
                            };

                        let move_hint =
                            if let Some(TreeItem::Notebook(_, _)) = app.get_selected_item() {
                                "[Shift+↑] Pr │ [Shift+↓] Cd │ [Shift+←→] Sb"
                            } else if let Some(TreeItem::Snippet(_, _)) = app.get_selected_item() {
                                "[Shift+↑] Pr │ [Shift+↓] Cd │ [Shift+←→] Sb"
                            } else {
                                ""
                            };

                        format!(
                            "{}[n/b 󰠮] │ [s 󰅨] │ [f / Shift+F ] │ {} │ {} │ [?] ",
                            back_hint, collapse_text, move_hint
                        )
                    }
                }
            }
        }

        (AppState::ExportImport, InputMode::Normal) => {
            if let Some(export_state) = &app.export_import_state {
                match export_state.mode {
                    ExportImportMode::MainMenu => {
                        format!(
                            "{} [↑↓] Navigate │ [⏎] Select │ [h] Home │ [Esc] Back ",
                            back_hint
                        )
                    }
                    ExportImportMode::ExportOptions => {
                        format!(
                            "{} [↑↓] Navigate │ [⏎] Toggle/Select │ [Esc] Back ",
                            back_hint
                        )
                    }
                    ExportImportMode::ImportOptions => {
                        format!(
                            "{} [↑↓] Navigate │ [⏎] Toggle/Select │ [Esc] Back ",
                            back_hint
                        )
                    }
                    ExportImportMode::ExportPath | ExportImportMode::_ImportPath => {
                        format!("{} [⏎] Confirm │ [Esc] Back ", back_hint)
                    }
                    ExportImportMode::ImportClipboard => {
                        format!("{} [⏎] Import │ [Esc] Back ", back_hint)
                    }
                    ExportImportMode::Importing => {
                        format!("{} [⏎] Import │ [Esc] Back ", back_hint)
                    }
                    ExportImportMode::ImportPathPopup => {
                        format!("{} [⏎] Import File │ [Esc] Back ", back_hint)
                    }
                    _ => format!("{} [Esc] Back ", back_hint),
                }
            } else {
                format!(
                    "{} [↑↓] Navigate │ [⏎] Select │ [h] Home │ [q] Quit ",
                    back_hint
                )
            }
        }

        // Other pages
        _ => {
            format!(
                "{} [↑↓] Navigate │ [⏎] Select │ [/] Search │ [h] Home │ [q] Quit ",
                back_hint
            )
        }
    }
}

/// Constructs the breadcrumb navigation trail with appropriate styling and symbols
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
                    "  Snippets ",
                    Style::default().fg(RosePine::BASE).bg(RosePine::IRIS),
                ));

                // Add the full path for the selected item in the tree view
                if let Some(selected_item) = app.get_selected_item() {
                    match selected_item {
                        TreeItem::Notebook(notebook_id, _) => {
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

                                // Add collapse/expand indicator if this is the selected notebook
                                let notebook_name = if *id == *notebook_id {
                                    let collapse_indicator = if app.is_notebook_collapsed(id) {
                                        "  "
                                    } else if app
                                        .snippet_database
                                        .notebooks
                                        .get(id)
                                        .map(|nb| !nb.children.is_empty())
                                        .unwrap_or(false)
                                    {
                                        "  "
                                    } else {
                                        "  "
                                    };

                                    format!(" {}{}", name, collapse_indicator)
                                } else {
                                    format!("   {} ", name)
                                };

                                let style = if *id == *notebook_id {
                                    Style::default().fg(RosePine::BASE).bg(RosePine::LOVE)
                                } else {
                                    Style::default().fg(RosePine::SUBTLE)
                                };

                                spans.push(Span::styled(notebook_name, style));
                            }
                        }
                        TreeItem::Snippet(snippet_id, _) => {
                            if let Some(snippet) = app.snippet_database.snippets.get(snippet_id) {
                                let notebook_id = snippet.notebook_id;
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
            crate::app::AppState::ExportImport => {
                spans.push(Span::styled(
                    " Export/Import ",
                    Style::default().fg(RosePine::BASE).bg(RosePine::IRIS),
                ));

                // Add mode-specific breadcrumb
                if let Some(export_state) = &app.export_import_state {
                    use crate::ui::export_import::ExportImportMode;

                    spans.push(Span::styled(" ❯ ", Style::default().fg(RosePine::MUTED)));

                    match export_state.mode {
                        ExportImportMode::MainMenu => {
                            spans.push(Span::styled(
                                " 󰍜 Menu ",
                                Style::default().fg(RosePine::BASE).bg(RosePine::LOVE),
                            ));
                        }
                        ExportImportMode::ExportOptions => {
                            spans.push(Span::styled(
                                " 󰥞 Export Options ",
                                Style::default().fg(RosePine::BASE).bg(RosePine::LOVE),
                            ));
                        }
                        ExportImportMode::ExportPath => {
                            spans.push(Span::styled(
                                "  Export Path ",
                                Style::default().fg(RosePine::BASE).bg(RosePine::LOVE),
                            ));
                        }
                        ExportImportMode::ImportOptions => {
                            spans.push(Span::styled(
                                " 󰥝 Import Options ",
                                Style::default().fg(RosePine::BASE).bg(RosePine::LOVE),
                            ));
                        }
                        ExportImportMode::_ImportPath => {
                            spans.push(Span::styled(
                                "  Import Path ",
                                Style::default().fg(RosePine::BASE).bg(RosePine::LOVE),
                            ));
                        }
                        ExportImportMode::ImportClipboard => {
                            spans.push(Span::styled(
                                "  Import from Clipboard ",
                                Style::default().fg(RosePine::BASE).bg(RosePine::LOVE),
                            ));
                        }
                        ExportImportMode::Exporting => {
                            spans.push(Span::styled(
                                "  Exporting... ",
                                Style::default().fg(RosePine::BASE).bg(RosePine::LOVE),
                            ));
                        }
                        ExportImportMode::Importing => {
                            spans.push(Span::styled(
                                " 󰋺 Importing... ",
                                Style::default().fg(RosePine::BASE).bg(RosePine::LOVE),
                            ));
                        }
                        ExportImportMode::ImportPathPopup => {
                            spans.push(Span::styled(
                                "  Import File ",
                                Style::default().fg(RosePine::BASE).bg(RosePine::LOVE),
                            ));
                        }
                    }
                }
            }
            crate::app::AppState::InfoPage => {
                spans.push(Span::styled(
                    " 󱁯 Info ",
                    Style::default().fg(RosePine::BASE).bg(RosePine::IRIS),
                ));
            }
            crate::app::AppState::Settings => {
                spans.push(Span::styled(
                    "  Settings ",
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

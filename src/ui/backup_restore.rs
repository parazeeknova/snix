use crate::app::App;
use crate::models::{
    ExportOptions, export_database_with_tags, import_database, merge_import_into_database_with_tags,
};
use crate::ui::colors::RosePine;
use chrono::{DateTime, TimeZone, Utc};
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::Style,
    widgets::{
        Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Widget, Wrap,
    },
};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[allow(dead_code)] // I'll refactor this soon for now just add this
#[derive(Debug, Clone, PartialEq)]
pub enum BackupRestoreMode {
    MainMenu,
    ConfirmDelete,
    StatusMessage,
    RestoreOptions,
}

#[allow(dead_code)] // I'll refactor this soon for now just add this
#[derive(Debug, Clone, Copy)]
pub enum RestoreStrategy {
    OverwriteAll,
    SkipExisting,
    MergeAndUpdate,
    SmartMerge,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BackupRestoreState {
    pub mode: BackupRestoreMode,
    pub selected_backup: Option<usize>,
    pub status_message: Option<String>,
    pub is_error: bool,
    pub backups: Vec<BackupInfo>,
    pub show_tree: bool,
    pub occasional_backup_enabled: bool,
    pub selected_option: usize,
    auto_backup_enabled: bool,
    pub last_auto_backup: Option<DateTime<Utc>>,
    pub auto_backup_interval: chrono::Duration,
    pub scroll_offset: usize,
    pub restore_strategy: RestoreStrategy,
    pub current_restore_backup: Option<usize>,
    pub restore_option_selected: usize,
}

impl Default for BackupRestoreState {
    fn default() -> Self {
        Self {
            mode: BackupRestoreMode::MainMenu,
            selected_backup: None,
            status_message: None,
            is_error: false,
            backups: Vec::new(),
            show_tree: false,
            occasional_backup_enabled: false,
            selected_option: 0,
            auto_backup_enabled: false,
            last_auto_backup: None,
            auto_backup_interval: chrono::Duration::minutes(60),
            scroll_offset: 0,
            restore_strategy: RestoreStrategy::SkipExisting,
            current_restore_backup: None,
            restore_option_selected: 0,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BackupInfo {
    pub file_path: PathBuf,
    pub created_at: DateTime<Utc>,
    pub tree_summary: String,
    pub file_size: u64,
    pub notebook_count: usize,
    pub snippet_count: usize,
    pub root_notebook_count: usize,
}

const BACKUP_DIR: &str = "backups";

/// Main render function for the backup/restore floating window
pub fn render(frame: &mut Frame, app: &mut App) {
    let mut state = app.backup_restore_state.clone().unwrap_or_default();
    state.backups = list_backups();
    let main_area = frame.area();

    if state.auto_backup_enabled {
        let now = Utc::now();
        let should_backup = match state.last_auto_backup {
            Some(last) => now.signed_duration_since(last) >= state.auto_backup_interval,
            None => true,
        };
        if should_backup {
            let fname = format!("backup-auto-{}.json", now.format("%Y%m%d-%H%M%S"));
            let path = std::path::Path::new(BACKUP_DIR).join(fname);
            let options = ExportOptions::default();
            let tag_manager_clone = app.tag_manager.clone();
            match export_database_with_tags(
                &app.snippet_database,
                &tag_manager_clone,
                &path,
                &options,
            ) {
                Ok(_) => {
                    state.status_message =
                        Some(format!("Automatic backup created: {}", path.display()));
                    state.is_error = false;
                    state.last_auto_backup = Some(now);
                    state.backups = list_backups();
                }
                Err(e) => {
                    state.status_message = Some(format!("Automatic backup failed: {}", e));
                    state.is_error = true;
                    state.last_auto_backup = Some(now);
                }
            }
        }
    }

    let popup_width = 140;
    let popup_height = 40;
    let popup_area = Rect::new(
        (main_area.width.saturating_sub(popup_width)) / 2,
        (main_area.height.saturating_sub(popup_height)) / 2,
        popup_width.min(main_area.width),
        popup_height.min(main_area.height),
    );

    Clear.render(popup_area, frame.buffer_mut());

    let block = Block::bordered()
        .title(" 󰅟 Backup & Restore ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(RosePine::HIGHLIGHT_HIGH));

    let inner_area = block.inner(popup_area);
    block.render(popup_area, frame.buffer_mut());

    let vertical_layout =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(2)]).split(inner_area);

    let main_area = vertical_layout[0];
    let keys_area = vertical_layout[1];

    let horizontal_layout =
        Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(main_area);

    let left = horizontal_layout[0];
    let right = horizontal_layout[1];

    match state.mode {
        BackupRestoreMode::MainMenu => render_main_menu(frame, left, &mut state),
        BackupRestoreMode::ConfirmDelete => render_confirm_delete(frame, left, &mut state),
        BackupRestoreMode::StatusMessage => render_status_message(frame, popup_area, &state),
        BackupRestoreMode::RestoreOptions => render_restore_options(frame, left, &mut state),
    }

    render_backup_preview(frame, right, &mut state);

    render_keybindings(frame, keys_area, &state);

    app.backup_restore_state = Some(state);
}

fn list_backups() -> Vec<BackupInfo> {
    let mut backups = Vec::new();
    let dir = PathBuf::from(BACKUP_DIR);
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                let metadata = entry.metadata().ok();
                let file_size = metadata.as_ref().map_or(0, |m| m.len());
                let created_at = metadata
                    .and_then(|m| m.modified().ok())
                    .and_then(|mtime| mtime.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| {
                        let seconds = d.as_secs() as i64;
                        match Utc.timestamp_opt(seconds, 0) {
                            chrono::LocalResult::Single(dt) => dt,
                            _ => Utc::now(),
                        }
                    })
                    .unwrap_or_else(|| Utc::now());

                let (tree_summary, notebook_count, snippet_count, root_notebook_count) =
                    match parse_backup_stats(&path) {
                        Ok((summary, notebooks, snippets, roots)) => {
                            (summary, notebooks, snippets, roots)
                        }
                        Err(_) => (format!("Tree for {} (unreadable)", path.display()), 0, 0, 0),
                    };

                backups.push(BackupInfo {
                    file_path: path.clone(),
                    created_at,
                    tree_summary,
                    file_size,
                    notebook_count,
                    snippet_count,
                    root_notebook_count,
                });
            }
        }
    }
    backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    backups
}

fn parse_backup_stats(path: &Path) -> Result<(String, usize, usize, usize), ()> {
    let content = std::fs::read_to_string(path).map_err(|_| ())?;
    let backup: BackupFile = serde_json::from_str(&content).map_err(|_| ())?;

    let notebook_count = backup.notebooks.len();
    let snippet_count = backup.snippets.len();
    let root_notebook_count = backup.root_notebooks.len();

    let mut lines = Vec::new();
    lines.push(format!(
        "Backup File: {}",
        path.file_name().unwrap_or_default().to_string_lossy()
    ));
    lines.push(format!("Total Notebooks: {}", notebook_count));
    lines.push(format!("Total Snippets: {}", snippet_count));
    lines.push(format!("Root Notebooks: {}", root_notebook_count));
    lines.push("".to_string());
    lines.push("Notebook Structure:".to_string());
    lines.push("".to_string());

    // Sort root notebooks by name for consistent display
    let mut root_notebooks = Vec::new();
    for root_id in &backup.root_notebooks {
        if let Some(notebook) = backup.notebooks.get(root_id) {
            root_notebooks.push(notebook);
        }
    }

    // Sort notebooks by name
    root_notebooks.sort_by(|a, b| a.name.cmp(&b.name));

    // Build tree for each root notebook
    for notebook in root_notebooks {
        build_notebook_tree(&backup, notebook, 0, &mut lines);
    }

    Ok((
        lines.join("\n"),
        notebook_count,
        snippet_count,
        root_notebook_count,
    ))
}

fn build_notebook_tree(
    backup: &BackupFile,
    notebook: &BackupNotebook,
    indent: usize,
    lines: &mut Vec<String>,
) {
    let prefix = if indent == 0 {
        "".to_string()
    } else {
        "│ ".repeat(indent - 1) + "├─ "
    };

    lines.push(format!("{}󰠮 {}", prefix, notebook.name));

    let mut snippets: Vec<&BackupSnippet> = backup
        .snippets
        .values()
        .filter(|s| s.notebook_id == notebook.id)
        .collect();

    snippets.sort_by(|a, b| a.title.cmp(&b.title));

    let mut children: Vec<&BackupNotebook> = backup
        .notebooks
        .values()
        .filter(|n| n.parent_id.as_deref() == Some(&notebook.id))
        .collect();

    children.sort_by(|a, b| a.name.cmp(&b.name));

    let snippet_count = snippets.len();
    for (i, snippet) in snippets.iter().enumerate() {
        let is_last = i == snippet_count - 1 && children.is_empty();
        let snippet_prefix = if indent == 0 {
            "  ".to_string()
        } else {
            "│ ".repeat(indent - 1) + (if is_last { "└─ " } else { "├─ " }) + "  "
        };

        lines.push(format!("{}󰗚 {}", snippet_prefix, snippet.title));
    }

    let child_count = children.len();
    for (i, child) in children.iter().enumerate() {
        let is_last = i == child_count - 1;
        if is_last {
            let mut temp_lines = Vec::new();
            build_notebook_tree(backup, child, indent + 1, &mut temp_lines);

            for line in temp_lines {
                if line.starts_with(&("│ ".repeat(indent))) {
                    let modified = "  ".to_string() + &line[3..];
                    lines.push(modified);
                } else {
                    lines.push(line);
                }
            }
        } else {
            build_notebook_tree(backup, child, indent + 1, lines);
        }
    }
}

fn render_main_menu(frame: &mut Frame, area: Rect, state: &mut BackupRestoreState) {
    let menu_block = Block::bordered()
        .title("  Actions & Backups ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().fg(RosePine::IRIS));

    let inner_area = menu_block.inner(area);
    menu_block.render(area, frame.buffer_mut());

    let menu_items = vec![
        (
            "󰁯  Create Backup".to_string(),
            "Create a new backup of all data".to_string(),
        ),
        (
            "󰔟  Occasional Backup".to_string(),
            "Trigger a manual backup".to_string(),
        ),
        (
            if state.auto_backup_enabled {
                "󰅠  Auto Backup: ON".to_string()
            } else {
                "󰅠  Auto Backup: OFF".to_string()
            },
            match (state.auto_backup_enabled, state.last_auto_backup) {
                (true, Some(dt)) => {
                    let ago = Utc::now().signed_duration_since(dt);
                    if ago.num_minutes() < 1 {
                        "Last auto backup: just now".to_string()
                    } else if ago.num_hours() < 1 {
                        "Last auto backup: less than 1 hour ago".to_string()
                    } else {
                        "Last auto backup: more than 1 hour ago".to_string()
                    }
                }
                (true, None) => "Auto backup enabled (no backup yet)".to_string(),
                (false, _) => "Auto backup is disabled".to_string(),
            },
        ),
    ];

    let backup_items: Vec<(String, String)> = state
        .backups
        .iter()
        .map(|backup| {
            let filename = backup
                .file_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();
            let date = backup.created_at.format("%Y-%m-%d %H:%M:%S").to_string();
            let size_kb = backup.file_size / 1024;
            let analytics = format!(
                "Created: {} UTC | Size: {}KB | Notebooks: {} | Snippets: {}",
                date, size_kb, backup.notebook_count, backup.snippet_count
            );
            (format!("󰆓  {}", filename), analytics)
        })
        .collect();

    let all_items: Vec<ListItem> = menu_items
        .iter()
        .map(|(title, desc)| {
            ListItem::new(format!("{}\n   {}", title, desc))
                .style(Style::default().fg(RosePine::TEXT))
        })
        .chain(std::iter::once(
            ListItem::new("────────────────────────────")
                .style(Style::default().fg(RosePine::HIGHLIGHT_LOW)),
        ))
        .chain(backup_items.iter().map(|(title, desc)| {
            ListItem::new(format!("{}\n   {}", title, desc))
                .style(Style::default().fg(RosePine::TEXT))
        }))
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(state.selected_option));

    let list = List::new(all_items)
        .highlight_style(
            Style::default()
                .fg(RosePine::LOVE)
                .bg(RosePine::HIGHLIGHT_LOW)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, inner_area, &mut list_state);

    if state.selected_option >= 4 {
        state.selected_backup = Some(state.selected_option);
    } else {
        state.selected_backup = None;
    }
}

fn render_backup_preview(frame: &mut Frame, area: Rect, state: &mut BackupRestoreState) {
    let selected = state.selected_option;
    let backup_idx = if selected >= 4 {
        Some(selected - 4)
    } else {
        None
    };

    if backup_idx.is_none() && state.mode == BackupRestoreMode::MainMenu {
        let block = Block::bordered()
            .title(" 󱏒 Preview ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .style(Style::default().fg(RosePine::FOAM));

        let inner = block.inner(area);
        block.render(area, frame.buffer_mut());

        let help_text = vec![
            "Select a backup to view its contents.",
            "",
            "Use the up/down keys to navigate.",
            "Press Enter on a backup to restore it.",
            "Press 'd' on a backup to delete it.",
            "",
            "You can create a new backup with the first option.",
            "Toggle auto-backup with the third option.",
        ]
        .join("\n");

        let para = Paragraph::new(help_text)
            .alignment(Alignment::Left)
            .style(Style::default().fg(RosePine::TEXT))
            .wrap(Wrap { trim: false });

        para.render(inner, frame.buffer_mut());
        return;
    }

    let empty = String::new();
    let (title, content): (String, &String) = if let Some(idx) = backup_idx {
        if let Some(backup) = state.backups.get(idx) {
            (
                format!(
                    " Preview: {} ",
                    backup
                        .file_path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                ),
                &backup.tree_summary,
            )
        } else {
            (" No backup selected ".to_string(), &empty)
        }
    } else {
        (" No backup selected ".to_string(), &empty)
    };

    let block = Block::bordered()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().fg(RosePine::FOAM));

    let inner = block.inner(area);
    block.render(area, frame.buffer_mut());

    let preview = if content.is_empty() {
        "Select a backup to preview its tree structure.".to_string()
    } else {
        content.to_string()
    };

    let lines: Vec<&str> = preview.lines().collect();
    let line_count = lines.len();

    if state.scroll_offset > line_count.saturating_sub(inner.height as usize) {
        state.scroll_offset = line_count.saturating_sub(inner.height as usize);
    }

    let visible_lines = if line_count <= inner.height as usize {
        preview.clone()
    } else {
        let start = state.scroll_offset;
        let end = start + inner.height as usize;
        let visible = lines[start..end.min(line_count)].join("\n");

        let mut result = String::new();
        if start > 0 {
            result.push_str("↑ Scroll up for more\n");
        }
        result.push_str(&visible);
        if end < line_count {
            result.push_str("\n↓ Scroll down for more");
        }
        result
    };

    let para = Paragraph::new(visible_lines)
        .alignment(Alignment::Left)
        .style(Style::default().fg(RosePine::TEXT))
        .wrap(Wrap { trim: false });

    para.render(inner, frame.buffer_mut());
}

fn render_confirm_delete(frame: &mut Frame, area: Rect, _state: &mut BackupRestoreState) {
    let msg = "Are you sure you want to delete this backup   ? [y/n]";
    let p = Paragraph::new(msg)
        .alignment(Alignment::Center)
        .style(Style::default().fg(RosePine::ROSE));
    p.render(area, frame.buffer_mut());
}

fn render_status_message(frame: &mut Frame, area: Rect, state: &BackupRestoreState) {
    let msg = state.status_message.as_deref().unwrap_or("");
    let color = if state.is_error {
        RosePine::LOVE
    } else {
        RosePine::FOAM
    };
    let p = Paragraph::new(msg)
        .alignment(Alignment::Center)
        .style(Style::default().fg(color));
    p.render(area, frame.buffer_mut());
}

fn render_restore_options(frame: &mut Frame, area: Rect, state: &mut BackupRestoreState) {
    let block = Block::bordered()
        .title(" Restore Options ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().fg(RosePine::LOVE));

    let inner_area = block.inner(area);
    block.render(area, frame.buffer_mut());

    let backup_name = if let Some(idx) = state.current_restore_backup {
        if let Some(backup) = state.backups.get(idx) {
            backup
                .file_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        } else {
            "Unknown backup".to_string()
        }
    } else {
        "Unknown backup".to_string()
    };

    let options = vec![
        (
            "Overwrite All",
            "Replace all existing notebooks and snippets with backup contents",
            RestoreStrategy::OverwriteAll,
        ),
        (
            "Skip Existing",
            "Only restore notebooks and snippets that don't exist in the current database",
            RestoreStrategy::SkipExisting,
        ),
        (
            "Merge and Update",
            "Add new items and update existing ones with backup versions",
            RestoreStrategy::MergeAndUpdate,
        ),
        (
            "Smart Merge (Recommended)",
            "Use timestamps to determine which version to keep (latest wins)",
            RestoreStrategy::SmartMerge,
        ),
    ];

    let header = format!("Choose how to restore backup: {}", backup_name);
    let header_para = Paragraph::new(header)
        .style(Style::default().fg(RosePine::FOAM))
        .alignment(Alignment::Center);

    let layout = Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]).split(inner_area);

    header_para.render(layout[0], frame.buffer_mut());

    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(i, (title, desc, _strategy))| {
            let style = if i == state.restore_option_selected {
                Style::default()
                    .fg(RosePine::LOVE)
                    .bg(RosePine::HIGHLIGHT_LOW)
            } else {
                Style::default().fg(RosePine::TEXT)
            };

            let prefix = if i == state.restore_option_selected {
                "▶ "
            } else {
                "  "
            };
            ListItem::new(format!("{}{}\n    {}", prefix, title, desc)).style(style)
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, layout[1]);

    if let Some(&(_, _, strategy)) = options.get(state.restore_option_selected) {
        state.restore_strategy = strategy;
    }
}

fn render_keybindings(frame: &mut Frame, area: Rect, state: &BackupRestoreState) {
    let keys = match state.mode {
        BackupRestoreMode::MainMenu => {
            "↑/↓: Navigate   Enter: Select/Restore   d: Delete   t: Tree   q/Esc: Close   PageUp/Down: Scroll"
        }
        BackupRestoreMode::ConfirmDelete => "y: Confirm Delete   n/Esc: Cancel",
        BackupRestoreMode::RestoreOptions => "↑/↓: Navigate   Enter: Confirm Restore   Esc: Cancel",
        BackupRestoreMode::StatusMessage => "Press any key to continue",
    };

    let p = Paragraph::new(keys)
        .alignment(Alignment::Center)
        .style(Style::default().fg(RosePine::HIGHLIGHT_HIGH));
    p.render(area, frame.buffer_mut());
}

pub fn handle_backup_restore_keys(key: KeyEvent, app: &mut App) -> bool {
    let mut state = app.backup_restore_state.clone().unwrap_or_default();
    let menu_len = 4 + state.backups.len();
    match state.mode {
        BackupRestoreMode::MainMenu => match key.code {
            KeyCode::Up => {
                if state.selected_option > 0 {
                    if state.selected_option == 4 {
                        state.selected_option = 2;
                    } else {
                        state.selected_option -= 1;
                    }
                }
                app.backup_restore_state = Some(state);
                return false;
            }
            KeyCode::Down => {
                if state.selected_option + 1 < menu_len {
                    if state.selected_option == 2 {
                        state.selected_option = 4;
                    } else {
                        state.selected_option += 1;
                    }
                }
                app.backup_restore_state = Some(state);
                return false;
            }
            KeyCode::PageUp => {
                state.scroll_offset = state.scroll_offset.saturating_sub(10);
                app.backup_restore_state = Some(state);
                return false;
            }
            KeyCode::PageDown => {
                state.scroll_offset += 10;
                app.backup_restore_state = Some(state);
                return false;
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                app.show_backup_restore_overlay = false;
                return false;
            }
            KeyCode::Enter => {
                match state.selected_option {
                    0 => {
                        let now = Utc::now();
                        let fname = format!("backup-{}.json", now.format("%Y%m%d-%H%M%S"));
                        let path = Path::new(BACKUP_DIR).join(fname);
                        let options = ExportOptions::default();
                        let tag_manager_clone = app.tag_manager.clone();
                        match export_database_with_tags(
                            &app.snippet_database,
                            &tag_manager_clone,
                            &path,
                            &options,
                        ) {
                            Ok(_) => {
                                state.status_message =
                                    Some(format!("Backup created: {}", path.display()));
                                state.is_error = false;
                                state.backups = list_backups();
                            }
                            Err(e) => {
                                state.status_message = Some(format!("Backup failed: {}", e));
                                state.is_error = true;
                            }
                        }
                        state.mode = BackupRestoreMode::StatusMessage;
                    }
                    1 => {
                        let now = Utc::now();
                        let fname = format!("backup-{}.json", now.format("%Y%m%d-%H%M%S"));
                        let path = Path::new(BACKUP_DIR).join(fname);
                        let options = ExportOptions::default();
                        let tag_manager_clone = app.tag_manager.clone();
                        match export_database_with_tags(
                            &app.snippet_database,
                            &tag_manager_clone,
                            &path,
                            &options,
                        ) {
                            Ok(_) => {
                                state.status_message =
                                    Some(format!("Occasional backup created: {}", path.display()));
                                state.is_error = false;
                                state.backups = list_backups();
                            }
                            Err(e) => {
                                state.status_message = Some(format!("Backup failed: {}", e));
                                state.is_error = true;
                            }
                        }
                        state.mode = BackupRestoreMode::StatusMessage;
                    }
                    2 => {
                        state.auto_backup_enabled = !state.auto_backup_enabled;
                        if state.auto_backup_enabled {
                            state.status_message = Some("Automatic backup enabled".to_string());
                        } else {
                            state.status_message = Some("Automatic backup disabled".to_string());
                        }
                        state.is_error = false;
                        state.mode = BackupRestoreMode::StatusMessage;
                    }
                    3 => {}
                    idx if idx >= 4 && idx < menu_len => {
                        let backup_idx = idx - 4;
                        if state.backups.get(backup_idx).is_some() {
                            state.current_restore_backup = Some(backup_idx);
                            state.restore_option_selected = 0;
                            state.mode = BackupRestoreMode::RestoreOptions;
                        }
                    }
                    _ => {}
                }
                app.backup_restore_state = Some(state);
                return false;
            }
            KeyCode::Char('d') => {
                if state.selected_option >= 4 && state.selected_option < menu_len {
                    state.mode = BackupRestoreMode::ConfirmDelete;
                }
                app.backup_restore_state = Some(state);
                return false;
            }
            KeyCode::Char('t') => {
                if state.selected_option >= 4 && state.selected_option < menu_len {
                    let backup_idx = state.selected_option - 4;
                    if let Some(backup) = state.backups.get(backup_idx) {
                        state.status_message = Some(backup.tree_summary.clone());
                        state.is_error = false;
                        state.mode = BackupRestoreMode::StatusMessage;
                    }
                }
                app.backup_restore_state = Some(state);
                return false;
            }
            _ => {
                app.backup_restore_state = Some(state);
                return false;
            }
        },
        BackupRestoreMode::ConfirmDelete => match key.code {
            KeyCode::Char('y') => {
                if state.selected_option >= 4 && state.selected_option < menu_len {
                    let backup_idx = state.selected_option - 4;
                    if let Some(backup) = state.backups.get(backup_idx) {
                        match fs::remove_file(&backup.file_path) {
                            Ok(_) => {
                                state.status_message = Some("Backup deleted".to_string());
                                state.is_error = false;
                                state.backups = list_backups();
                                if state.selected_option >= 4 + state.backups.len() {
                                    state.selected_option = state.backups.len() + 3;
                                }
                            }
                            Err(e) => {
                                state.status_message = Some(format!("Delete failed: {}", e));
                                state.is_error = true;
                            }
                        }
                        state.mode = BackupRestoreMode::StatusMessage;
                    }
                }
                app.backup_restore_state = Some(state);
                return false;
            }
            KeyCode::Char('n') | KeyCode::Esc | KeyCode::Char('q') => {
                state.mode = BackupRestoreMode::MainMenu;
                app.backup_restore_state = Some(state);
                return false;
            }
            _ => {
                app.backup_restore_state = Some(state);
                return false;
            }
        },
        BackupRestoreMode::RestoreOptions => match key.code {
            KeyCode::Up => {
                if state.restore_option_selected > 0 {
                    state.restore_option_selected -= 1;
                }
                app.backup_restore_state = Some(state);
                return false;
            }
            KeyCode::Down => {
                if state.restore_option_selected < 3 {
                    state.restore_option_selected += 1;
                }
                app.backup_restore_state = Some(state);
                return false;
            }
            KeyCode::Enter => {
                if let Some(backup_idx) = state.current_restore_backup {
                    if let Some(backup) = state.backups.get(backup_idx) {
                        match import_database(&backup.file_path) {
                            Ok(import_data) => {
                                let mut tag_manager_clone = app.tag_manager.clone();

                                let overwrite = match state.restore_strategy {
                                    RestoreStrategy::OverwriteAll => true,
                                    RestoreStrategy::SkipExisting => false,
                                    RestoreStrategy::MergeAndUpdate => true,
                                    RestoreStrategy::SmartMerge => true,
                                };

                                match merge_import_into_database_with_tags(
                                    &mut app.snippet_database,
                                    &mut tag_manager_clone,
                                    import_data,
                                    overwrite,
                                ) {
                                    Ok((notebooks, snippets)) => {
                                        app.tag_manager = tag_manager_clone;
                                        let _ = app.save_database();
                                        app.refresh_tree_items();

                                        let strategy_name = match state.restore_strategy {
                                            RestoreStrategy::OverwriteAll => "Overwrite All",
                                            RestoreStrategy::SkipExisting => "Skip Existing",
                                            RestoreStrategy::MergeAndUpdate => "Merge and Update",
                                            RestoreStrategy::SmartMerge => "Smart Merge",
                                        };

                                        state.status_message = Some(format!(
                                            "Backup restored successfully using {} strategy! Restored {} notebooks and {} snippets.",
                                            strategy_name, notebooks, snippets
                                        ));
                                        state.is_error = false;
                                    }
                                    Err(e) => {
                                        state.status_message =
                                            Some(format!("Restore failed: {}", e));
                                        state.is_error = true;
                                    }
                                }
                            }
                            Err(e) => {
                                state.status_message = Some(format!("Restore failed: {}", e));
                                state.is_error = true;
                            }
                        }
                        state.mode = BackupRestoreMode::StatusMessage;
                    }
                }
                app.backup_restore_state = Some(state);
                return false;
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                state.mode = BackupRestoreMode::MainMenu;
                app.backup_restore_state = Some(state);
                return false;
            }
            _ => {
                app.backup_restore_state = Some(state);
                return false;
            }
        },
        BackupRestoreMode::StatusMessage => {
            state.mode = BackupRestoreMode::MainMenu;
            state.status_message = None;
            app.backup_restore_state = Some(state);
            return false;
        }
    }
}

#[derive(Deserialize)]
struct BackupNotebook {
    id: String,
    name: String,
    parent_id: Option<String>,
}

#[derive(Deserialize)]
struct BackupSnippet {
    #[allow(dead_code)]
    id: String,
    title: String,
    notebook_id: String,
}

#[derive(Deserialize)]
struct BackupFile {
    notebooks: std::collections::HashMap<String, BackupNotebook>,
    root_notebooks: Vec<String>,
    snippets: std::collections::HashMap<String, BackupSnippet>,
}

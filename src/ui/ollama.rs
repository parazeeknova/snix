use crate::app::App;
use chrono::{DateTime, Utc};
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};
use serde::{Deserialize, Serialize};
use unicode_width::UnicodeWidthChar;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: Uuid,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub model_name: String,
    pub system_prompt: String,
    pub snippet_hash: Option<String>,
    pub snippet_title: Option<String>,
    pub conversation: Vec<ChatMessage>,
    pub is_favorited: bool,
    pub tags: Vec<String>,
}

impl ChatSession {
    pub fn new(model_name: String, system_prompt: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title: format!("Chat with {}", model_name),
            created_at: now,
            updated_at: now,
            model_name,
            system_prompt,
            snippet_hash: None,
            snippet_title: None,
            conversation: Vec::new(),
            is_favorited: false,
            tags: Vec::new(),
        }
    }

    pub fn with_snippet(mut self, snippet_content: &str, snippet_title: String) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        snippet_content.hash(&mut hasher);
        self.snippet_hash = Some(format!("{:x}", hasher.finish()));
        self.snippet_title = Some(snippet_title.clone());
        self.title = format!("Chat about {}", snippet_title);
        self
    }

    pub fn add_message(&mut self, role: ChatRole, content: String) {
        self.conversation.push(ChatMessage { role, content });
        self.updated_at = Utc::now();
    }

    pub fn get_preview(&self) -> String {
        if let Some(last_msg) = self.conversation.last() {
            let preview = last_msg.content.chars().take(40).collect::<String>();
            if last_msg.content.len() > 40 {
                format!("{}...", preview)
            } else {
                preview
            }
        } else {
            "New conversation".to_string()
        }
    }

    pub fn get_message_count(&self) -> usize {
        self.conversation.len()
    }

    pub fn get_relative_time(&self) -> String {
        let now = Utc::now();
        let duration = now.signed_duration_since(self.updated_at);

        if duration.num_days() > 7 {
            format!("{}w ago", duration.num_weeks())
        } else if duration.num_days() > 0 {
            format!("{}d ago", duration.num_days())
        } else if duration.num_hours() > 0 {
            format!("{}h ago", duration.num_hours())
        } else if duration.num_minutes() > 0 {
            format!("{}m ago", duration.num_minutes())
        } else {
            "now".to_string()
        }
    }
}

#[derive(Debug, Clone)]
pub struct ToastNotification {
    pub message: String,
    pub notification_type: NotificationType,
    pub created_at: std::time::Instant,
    pub duration_seconds: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NotificationType {
    Success,
    Error,
    Info,
}

impl ToastNotification {
    pub fn new(
        message: String,
        notification_type: NotificationType,
        duration_seconds: u64,
    ) -> Self {
        Self {
            message,
            notification_type,
            created_at: std::time::Instant::now(),
            duration_seconds,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed().as_secs() >= self.duration_seconds
    }
}

#[derive(Debug, Clone)]
pub struct OllamaState {
    pub show_popup: bool,
    pub models: Vec<String>,
    pub selected_model_index: usize,
    pub loading_models: bool,
    pub error_message: Option<String>,
    pub conversation: Vec<ChatMessage>,
    pub input_buffer: String,
    pub is_sending: bool,
    pub current_snippet: Option<String>,
    pub scroll_position: usize,
    pub scroll_speed: usize,
    pub loading_animation_frame: usize,
    pub pending_response_id: Option<u64>,
    pub typing_indicator: String,

    // Simplified sidebar state
    pub active_panel: ActivePanel,
    pub system_prompt: String,
    pub system_prompt_buffer: String,
    pub editing_system_prompt: bool,

    // Chat session management
    pub chat_storage: Option<ChatStorage>,
    pub current_session: Option<ChatSession>,
    pub saved_sessions: Vec<ChatSession>,
    pub selected_session_index: usize,
    pub history_filter: HistoryFilter,
    pub search_query: String,
    pub auto_save_enabled: bool,

    // Toast notification system
    pub toast_notifications: Vec<ToastNotification>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChatRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActivePanel {
    CurrentChat,
    ChatHistory,
    Settings,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HistoryFilter {
    All,
    Recent,
    Favorites,
    CurrentSnippet,
    Search,
}

#[derive(Debug, Clone)]
pub enum OllamaMessage {
    ModelsLoaded {
        models: Vec<String>,
    },
    ResponseChunk {
        request_id: u64,
        content: String,
        done: bool,
    },
    Error {
        request_id: u64,
        message: String,
    },
}

impl Default for OllamaState {
    fn default() -> Self {
        let chat_storage = ChatStorage::new().ok();
        Self {
            show_popup: false,
            models: Vec::new(),
            selected_model_index: 0,
            loading_models: false,
            error_message: None,
            conversation: Vec::new(),
            input_buffer: String::new(),
            is_sending: false,
            current_snippet: None,
            scroll_position: 0,
            scroll_speed: 3,
            loading_animation_frame: 0,
            pending_response_id: None,
            typing_indicator: String::new(),

            // Simplified sidebar state
            active_panel: ActivePanel::CurrentChat,
            system_prompt: "You are a helpful AI assistant. When working with code snippets, provide clear explanations and suggestions for improvement.".to_string(),
            system_prompt_buffer: String::new(),
            editing_system_prompt: false,

            // Chat session management
            chat_storage,
            current_session: None,
            saved_sessions: Vec::new(),
            selected_session_index: 0,
            history_filter: HistoryFilter::All,
            search_query: String::new(),
            auto_save_enabled: true,

            // Toast notification system
            toast_notifications: Vec::new(),
        }
    }
}

impl OllamaState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_message(&mut self, role: ChatRole, content: String) {
        self.conversation.push(ChatMessage { role, content });
        // Auto-scroll to the bottom when a new message is added
        // Use a large value that will be safely clamped in render
        self.scroll_position = 999999;
    }

    pub fn get_selected_model(&self) -> Option<&String> {
        if self.models.is_empty() || self.selected_model_index >= self.models.len() {
            None
        } else {
            Some(&self.models[self.selected_model_index])
        }
    }

    // Toast notification management
    pub fn add_toast(
        &mut self,
        message: String,
        notification_type: NotificationType,
        duration_seconds: u64,
    ) {
        let toast = ToastNotification::new(message, notification_type, duration_seconds);
        self.toast_notifications.push(toast);

        // Keep only the last 5 notifications to avoid cluttering
        if self.toast_notifications.len() > 5 {
            self.toast_notifications.remove(0);
        }
    }

    pub fn add_success_toast(&mut self, message: String) {
        self.add_toast(message, NotificationType::Success, 5);
    }

    pub fn add_error_toast(&mut self, message: String) {
        self.add_toast(message, NotificationType::Error, 8); // Error messages stay longer
    }

    pub fn add_info_toast(&mut self, message: String) {
        self.add_toast(message, NotificationType::Info, 5);
    }

    pub fn clean_expired_toasts(&mut self) {
        self.toast_notifications.retain(|toast| !toast.is_expired());
    }

    pub fn get_filtered_sessions(&self) -> Vec<&ChatSession> {
        match self.history_filter {
            HistoryFilter::All => self.saved_sessions.iter().collect(),
            HistoryFilter::Recent => {
                let mut sessions = self.saved_sessions.iter().collect::<Vec<_>>();
                sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
                sessions.into_iter().take(10).collect()
            }
            HistoryFilter::Favorites => self
                .saved_sessions
                .iter()
                .filter(|s| s.is_favorited)
                .collect(),
            HistoryFilter::CurrentSnippet => {
                if let Some(snippet) = &self.current_snippet {
                    use std::collections::hash_map::DefaultHasher;
                    use std::hash::{Hash, Hasher};

                    let mut hasher = DefaultHasher::new();
                    snippet.hash(&mut hasher);
                    let snippet_hash = format!("{:x}", hasher.finish());

                    self.saved_sessions
                        .iter()
                        .filter(|s| s.snippet_hash.as_ref() == Some(&snippet_hash))
                        .collect()
                } else {
                    Vec::new()
                }
            }
            HistoryFilter::Search => {
                if self.search_query.is_empty() {
                    self.saved_sessions.iter().collect()
                } else {
                    let query_lower = self.search_query.to_lowercase();
                    self.saved_sessions
                        .iter()
                        .filter(|session| {
                            session.title.to_lowercase().contains(&query_lower)
                                || session.model_name.to_lowercase().contains(&query_lower)
                                || session
                                    .conversation
                                    .iter()
                                    .any(|msg| msg.content.to_lowercase().contains(&query_lower))
                        })
                        .collect()
                }
            }
        }
    }
}

pub fn render_ollama_popup(f: &mut Frame, app: &App, area: Rect) {
    if let Some(ollama_state) = &app.ollama_state {
        // Always render toast notifications, even when popup is not showing
        render_toast_notifications(f, ollama_state, area);

        if !ollama_state.show_popup {
            return;
        }

        // Make the popup take up most of the screen for better UX
        let popup_width = area.width.min(160).max(120);
        let popup_height = area.height.min(60).max(40);
        let popup_x = (area.width - popup_width) / 2;
        let popup_y = (area.height - popup_height) / 2;
        let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

        f.render_widget(Clear, popup_area);

        // Main container with rounded corners
        let main_block = Block::default()
            .title("✨ Ollama AI Assistant")
            .title_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan));

        f.render_widget(main_block.clone(), popup_area);

        let inner_area = main_block.inner(popup_area);

        if ollama_state.loading_models {
            render_loading_screen(f, ollama_state, inner_area);
        } else if ollama_state.models.is_empty() {
            render_error_screen(f, ollama_state, inner_area);
        } else {
            render_main_interface(f, app, inner_area);
        }
    }
}

fn render_toast_notifications(f: &mut Frame, ollama_state: &OllamaState, area: Rect) {
    if ollama_state.toast_notifications.is_empty() {
        return;
    }

    let notification_width = 50;
    let notification_height = 3;
    let right_margin = 2;
    let bottom_margin = 2;

    for (index, toast) in ollama_state.toast_notifications.iter().enumerate() {
        let y_offset = (notification_height + 1) * index as u16;

        let notification_area = Rect {
            x: area.width.saturating_sub(notification_width + right_margin),
            y: area
                .height
                .saturating_sub(bottom_margin + y_offset + notification_height),
            width: notification_width,
            height: notification_height,
        };

        // Don't render if it would go off-screen
        if notification_area.y < area.y {
            break;
        }

        let (icon, color) = match toast.notification_type {
            NotificationType::Success => ("✅", Color::Green),
            NotificationType::Error => ("❌", Color::Red),
            NotificationType::Info => ("ℹ️", Color::Blue),
        };

        let notification_block = Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(color))
            .style(Style::default().bg(Color::Black));

        let notification_text = Paragraph::new(format!("{} {}", icon, toast.message))
            .block(notification_block)
            .style(Style::default().fg(color))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });

        f.render_widget(Clear, notification_area);
        f.render_widget(notification_text, notification_area);
    }
}

fn render_loading_screen(f: &mut Frame, ollama_state: &OllamaState, area: Rect) {
    let loading_chars = ["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];
    let animation_char = loading_chars[ollama_state.loading_animation_frame % loading_chars.len()];

    let loading_text = Paragraph::new(format!("🔄 Loading Ollama models... {}", animation_char))
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(loading_text, area);
}

fn render_error_screen(f: &mut Frame, ollama_state: &OllamaState, area: Rect) {
    let message = if let Some(error) = &ollama_state.error_message {
        format!(
            "❌ Error: {}\n\n💡 Make sure Ollama is running and try again.\n\n🔧 You can start Ollama with: ollama serve",
            error
        )
    } else {
        "❌ No Ollama models found.\n\n💡 Make sure Ollama is installed and running.\n\n🔧 Install models with: ollama pull llama2".to_string()
    };

    let error_text = Paragraph::new(message)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Red))
        .wrap(Wrap { trim: true });

    f.render_widget(error_text, area);
}

fn render_main_interface(f: &mut Frame, app: &App, area: Rect) {
    // Split into sidebar and main content with increased sidebar width
    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(35), Constraint::Min(50)])
        .split(area);

    render_sidebar(f, app, main_layout[0]);
    render_main_content(f, app, main_layout[1]);
}

fn render_sidebar(f: &mut Frame, app: &App, area: Rect) {
    if let Some(ollama_state) = &app.ollama_state {
        // Sidebar container with rounded corners
        let sidebar_block = Block::default()
            .title("🎛️ Options")
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(Color::Blue));

        f.render_widget(sidebar_block.clone(), area);

        let sidebar_inner = sidebar_block.inner(area);

        // Split sidebar into sections with more space for navigation
        let sidebar_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5), // Navigation (increased for all tabs to be visible)
                Constraint::Min(5),    // Content
                Constraint::Length(5), // Shortcuts (increased to 5 for 3 lines of shortcuts)
            ])
            .split(sidebar_inner);

        // Navigation tabs
        render_sidebar_navigation(f, ollama_state, sidebar_layout[0]);

        // Content based on selected option
        match ollama_state.active_panel {
            ActivePanel::CurrentChat => render_chat_options(f, ollama_state, sidebar_layout[1]),
            ActivePanel::ChatHistory => render_history_manager(f, ollama_state, sidebar_layout[1]),
            ActivePanel::Settings => render_settings_panel(f, ollama_state, sidebar_layout[1]),
        }

        // Render shortcuts at the bottom
        render_sidebar_shortcuts(f, ollama_state, sidebar_layout[2]);
    }
}

fn render_sidebar_navigation(f: &mut Frame, ollama_state: &OllamaState, area: Rect) {
    let panels = [
        ("💬", "Current Chat", ActivePanel::CurrentChat),
        ("📚", "Chat History", ActivePanel::ChatHistory),
        ("⚙️", "Settings", ActivePanel::Settings),
    ];

    let panel_items: Vec<ListItem> = panels
        .iter()
        .map(|(icon, label, panel)| {
            let is_selected = &ollama_state.active_panel == panel;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };

            let content = format!(" {} {}", icon, label);
            ListItem::new(content).style(style)
        })
        .collect();

    let navigation_block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" Navigation ");

    let navigation_list = List::new(panel_items)
        .block(navigation_block)
        .style(Style::default());

    f.render_widget(navigation_list, area);
}

fn render_sidebar_shortcuts(f: &mut Frame, ollama_state: &OllamaState, area: Rect) {
    let shortcuts = match ollama_state.active_panel {
        ActivePanel::CurrentChat => {
            vec![
                "Tab: Switch panel",
                "Ctrl+↑/↓: Select model",
                "Ctrl+R: Refresh models",
                "Ctrl+N: New chat",
                "Ctrl+S: Save session",
                "Ctrl+L: Clear chat",
                "Enter: Send message",
                "Esc: Close",
            ]
        }
        ActivePanel::ChatHistory => {
            vec![
                "Tab: Switch panel",
                "↑/↓: Navigate sessions",
                "M: Cycle filters",
                "/: Search mode",
                "F: Toggle favorite",
                "Delete: Delete session",
                "Enter: Load session",
                "Esc: Close",
            ]
        }
        ActivePanel::Settings => {
            vec![
                "Tab: Switch panel",
                "Enter: Edit system prompt",
                "Space: Toggle auto-save",
                "Ctrl+R: Refresh",
                "Esc: Close",
            ]
        }
    };

    // Create a nicely formatted shortcut text with 3 lines
    let shortcut_text = shortcuts
        .chunks(3) // Split into groups of 3 for 3 lines
        .map(|chunk| chunk.join(" • "))
        .collect::<Vec<_>>()
        .join("\n");

    let shortcuts_block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" Shortcuts ");

    let shortcuts_paragraph = Paragraph::new(shortcut_text)
        .block(shortcuts_block)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    f.render_widget(shortcuts_paragraph, area);
}

fn render_chat_options(f: &mut Frame, ollama_state: &OllamaState, area: Rect) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header
            Constraint::Length(4), // Model info
            Constraint::Length(6), // Available models section (new!)
            Constraint::Length(4), // Session info
            Constraint::Min(3),    // Status and controls
        ])
        .split(area);

    // Header
    let header = Paragraph::new("💬 Current Chat")
        .style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    f.render_widget(header, layout[0]);

    // Model information
    render_model_info(f, ollama_state, layout[1]);

    // Available models section (new!)
    render_available_models(f, ollama_state, layout[2]);

    // Session information
    render_session_info(f, ollama_state, layout[3]);

    // Status and controls
    render_chat_status(f, ollama_state, layout[4]);
}

fn render_model_info(f: &mut Frame, ollama_state: &OllamaState, area: Rect) {
    let model_name = ollama_state
        .get_selected_model()
        .unwrap_or(&"No model selected".to_string())
        .clone();

    let model_status = if ollama_state.models.is_empty() {
        "❌ No models available"
    } else if ollama_state.get_selected_model().is_some() {
        "✅ Model ready"
    } else {
        "⚠️ Select a model"
    };

    let model_info = format!("🤖 Model: {}\n{}", model_name, model_status);

    let model_block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(Color::Blue))
        .title(" Model ");

    let model_text = Paragraph::new(model_info)
        .block(model_block)
        .wrap(Wrap { trim: true })
        .style(Style::default());

    f.render_widget(model_text, area);
}

fn render_available_models(f: &mut Frame, ollama_state: &OllamaState, area: Rect) {
    let models_block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Available Models ");

    if ollama_state.loading_models {
        let loading_text = Paragraph::new("🔄 Loading models...")
            .block(models_block)
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center);
        f.render_widget(loading_text, area);
        return;
    }

    if ollama_state.models.is_empty() {
        let error_text = Paragraph::new("❌ No models found\nInstall with:\nollama pull llama2")
            .block(models_block)
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        f.render_widget(error_text, area);
        return;
    }

    // Create list items for available models
    let model_items: Vec<ListItem> = ollama_state
        .models
        .iter()
        .enumerate()
        .map(|(index, model)| {
            let is_selected = index == ollama_state.selected_model_index;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let icon = if is_selected { "🎯" } else { "🤖" };
            let content = format!(" {} {}", icon, model);
            ListItem::new(content).style(style)
        })
        .collect();

    let models_list = List::new(model_items)
        .block(models_block)
        .style(Style::default());

    f.render_widget(models_list, area);
}

fn render_session_info(f: &mut Frame, ollama_state: &OllamaState, area: Rect) {
    let session_info = if let Some(session) = &ollama_state.current_session {
        format!(
            "📁 {}\n💬 {} messages\n🕒 {}\n{}",
            if session.title.len() > 20 {
                format!("{}...", session.title.chars().take(17).collect::<String>())
            } else {
                session.title.clone()
            },
            session.get_message_count(),
            session.get_relative_time(),
            if session.is_favorited {
                "⭐ Favorited"
            } else {
                ""
            }
        )
    } else {
        "📁 New conversation\n💬 0 messages\n🕒 now".to_string()
    };

    let session_block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(Color::Magenta))
        .title(" Session ");

    let session_text = Paragraph::new(session_info)
        .block(session_block)
        .wrap(Wrap { trim: true })
        .style(Style::default());

    f.render_widget(session_text, area);
}

fn render_chat_status(f: &mut Frame, ollama_state: &OllamaState, area: Rect) {
    // Status only - no controls since they're now in the sidebar
    let status_text = if ollama_state.is_sending {
        format!(
            "🔄 Generating response...\n{}",
            ollama_state.typing_indicator
        )
    } else if ollama_state.loading_models {
        "📥 Loading models...".to_string()
    } else if ollama_state.get_selected_model().is_none() {
        "⚠️ Select a model to start chatting\nUse Ctrl+↑/↓ to select a model".to_string()
    } else {
        format!(
            "✅ Ready to chat\n💾 Auto-save: {}\n🤖 Current: {}",
            if ollama_state.auto_save_enabled {
                "On"
            } else {
                "Off"
            },
            ollama_state
                .get_selected_model()
                .unwrap_or(&"None".to_string())
        )
    };

    let status_block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(if ollama_state.is_sending {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Green)
        })
        .title(" Status ");

    let status_paragraph = Paragraph::new(status_text)
        .block(status_block)
        .wrap(Wrap { trim: true })
        .style(Style::default());

    f.render_widget(status_paragraph, area);
}

fn render_history_manager(f: &mut Frame, ollama_state: &OllamaState, area: Rect) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header
            Constraint::Length(3), // Filter selection
            Constraint::Length(if ollama_state.history_filter == HistoryFilter::Search {
                3
            } else {
                0
            }), // Search
            Constraint::Min(4),    // Sessions list
        ])
        .split(area);

    // Header with session count
    let sessions_count = ollama_state.get_filtered_sessions().len();
    let header = Paragraph::new(format!("📚 Chat History ({})", sessions_count))
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    f.render_widget(header, layout[0]);

    // Filter selection
    render_history_filters(f, ollama_state, layout[1]);

    // Search box (conditional) and sessions list
    if ollama_state.history_filter == HistoryFilter::Search {
        render_search_input(f, ollama_state, layout[2]);
        render_sessions_list(f, ollama_state, layout[3]);
    } else {
        render_sessions_list(f, ollama_state, layout[2]);
    };
}

fn render_history_filters(f: &mut Frame, ollama_state: &OllamaState, area: Rect) {
    let filters = [
        ("📄", "All", HistoryFilter::All),
        ("🕒", "Recent", HistoryFilter::Recent),
        ("⭐", "Favorites", HistoryFilter::Favorites),
        ("📋", "Snippet", HistoryFilter::CurrentSnippet),
        ("🔍", "Search", HistoryFilter::Search),
    ];

    let filter_items: Vec<ListItem> = filters
        .iter()
        .map(|(icon, label, filter)| {
            let is_selected = &ollama_state.history_filter == filter;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::DarkGray)
            } else {
                Style::default().fg(Color::Gray)
            };

            let content = format!("{} {}", icon, label);
            ListItem::new(content).style(style)
        })
        .collect();

    let filter_block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(Color::Blue))
        .title(" Filter ");

    let filter_list = List::new(filter_items)
        .block(filter_block)
        .style(Style::default());

    f.render_widget(filter_list, area);
}

fn render_search_input(f: &mut Frame, ollama_state: &OllamaState, area: Rect) {
    let search_block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(Color::Yellow))
        .title(" 🔍 Search ");

    let search_text = Paragraph::new(format!("{}_", ollama_state.search_query))
        .block(search_block)
        .style(Style::default().fg(Color::White));

    f.render_widget(search_text, area);
}

fn render_sessions_list(f: &mut Frame, ollama_state: &OllamaState, area: Rect) {
    let filtered_sessions = ollama_state.get_filtered_sessions();

    if filtered_sessions.is_empty() {
        let empty_message = match ollama_state.history_filter {
            HistoryFilter::All => {
                "No saved conversations yet\n\nStart chatting to create your first session!"
            }
            HistoryFilter::Recent => "No recent conversations",
            HistoryFilter::CurrentSnippet => {
                "No conversations for this snippet\n\nStart chatting about this code!"
            }
            HistoryFilter::Favorites => {
                "No favorite conversations yet\n\nPress 'F' to favorite a chat!"
            }
            HistoryFilter::Search => {
                if ollama_state.search_query.is_empty() {
                    "Enter search terms above..."
                } else {
                    "No conversations match your search"
                }
            }
        };

        let empty_block = Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" Sessions ");

        let empty_text = Paragraph::new(empty_message)
            .block(empty_block)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray))
            .wrap(Wrap { trim: true });

        f.render_widget(empty_text, area);
        return;
    }

    let session_items: Vec<ListItem> = filtered_sessions
        .iter()
        .enumerate()
        .map(|(i, session)| {
            let is_selected = i == ollama_state.selected_session_index
                && ollama_state.selected_session_index < filtered_sessions.len();
            let is_current = ollama_state
                .current_session
                .as_ref()
                .map_or(false, |current| current.id == session.id);

            let (_border_style, text_style) = if is_current {
                (
                    Color::Green,
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )
            } else if is_selected {
                (
                    Color::Yellow,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                (Color::Gray, Style::default().fg(Color::White))
            };

            let indicators = format!(
                "{}{}{}",
                if session.is_favorited { "⭐" } else { "" },
                if is_current { "▶ " } else { "" },
                if session.snippet_hash.is_some() {
                    "📋 "
                } else {
                    "💬 "
                }
            );

            let title = if session.title.len() > 22 {
                format!("{}...", session.title.chars().take(19).collect::<String>())
            } else {
                session.title.clone()
            };

            let preview = session.get_preview();
            let content = format!(
                "{}{}\n   {} • {} • {} msgs",
                indicators,
                title,
                preview,
                session.get_relative_time(),
                session.get_message_count()
            );

            ListItem::new(content).style(text_style)
        })
        .collect();

    let sessions_block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Sessions ");

    let sessions_list = List::new(session_items)
        .block(sessions_block)
        .style(Style::default());

    f.render_widget(sessions_list, area);
}

fn render_settings_panel(f: &mut Frame, ollama_state: &OllamaState, area: Rect) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header
            Constraint::Length(6), // Settings
            Constraint::Length(4), // System prompt
            Constraint::Min(4),    // Storage stats (expanded to take remaining space)
        ])
        .split(area);

    // Header
    let header = Paragraph::new("⚙️ Settings")
        .style(
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    f.render_widget(header, layout[0]);

    // Settings
    render_settings_options(f, ollama_state, layout[1]);

    // System prompt
    render_system_prompt_section(f, ollama_state, layout[2]);

    // Storage stats
    render_storage_stats(f, ollama_state, layout[3]);
}

fn render_settings_options(f: &mut Frame, ollama_state: &OllamaState, area: Rect) {
    let auto_save_status = if ollama_state.auto_save_enabled {
        "✅ Enabled"
    } else {
        "❌ Disabled"
    };

    let model_count = ollama_state.models.len();
    let current_model = ollama_state
        .get_selected_model()
        .unwrap_or(&"None".to_string())
        .clone();

    let settings_text = format!(
        "💾 Auto-save: {}\n🤖 Available models: {}\n📍 Current model: {}\n💬 Active conversations: {}\n📁 Storage: ~/.snix/ollama_chats/",
        auto_save_status,
        model_count,
        if current_model.len() > 15 {
            format!("{}...", current_model.chars().take(12).collect::<String>())
        } else {
            current_model
        },
        ollama_state.saved_sessions.len()
    );

    let settings_block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(Color::Blue))
        .title(" Configuration ");

    let settings_paragraph = Paragraph::new(settings_text)
        .block(settings_block)
        .wrap(Wrap { trim: true })
        .style(Style::default());

    f.render_widget(settings_paragraph, area);
}

fn render_system_prompt_section(f: &mut Frame, ollama_state: &OllamaState, area: Rect) {
    let prompt_preview = if ollama_state.system_prompt.len() > 50 {
        format!(
            "{}...",
            ollama_state
                .system_prompt
                .chars()
                .take(47)
                .collect::<String>()
        )
    } else {
        ollama_state.system_prompt.clone()
    };

    let content = if ollama_state.editing_system_prompt {
        &ollama_state.system_prompt_buffer
    } else {
        &prompt_preview
    };

    let prompt_block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(if ollama_state.editing_system_prompt {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Magenta)
        })
        .title(if ollama_state.editing_system_prompt {
            " ✏️ Editing System Prompt (Enter: Save, Esc: Cancel) "
        } else {
            " 🤖 System Prompt (Enter: Edit) "
        });

    let prompt_text = Paragraph::new(content.clone())
        .block(prompt_block)
        .wrap(Wrap { trim: true })
        .style(if ollama_state.editing_system_prompt {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::Gray)
        });

    f.render_widget(prompt_text, area);
}

fn render_storage_stats(f: &mut Frame, ollama_state: &OllamaState, area: Rect) {
    let stats_text = if let Some(storage) = &ollama_state.chat_storage {
        match storage.get_storage_stats() {
            Ok(stats) => {
                let size_kb = stats.storage_size_bytes as f64 / 1024.0;
                let models_count = stats.models.len();
                let _total_usage = stats.models.values().sum::<usize>();

                format!(
                    "💾 {:.1} KB storage used\n📊 {} sessions • {} models used\n💬 {} total messages",
                    size_kb, stats.total_sessions, models_count, stats.total_messages
                )
            }
            Err(_) => "❌ Unable to load statistics".to_string(),
        }
    } else {
        "❌ Storage not initialized".to_string()
    };

    let stats_block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(Color::Green))
        .title(" 📊 Storage Statistics ");

    let stats_paragraph = Paragraph::new(stats_text)
        .block(stats_block)
        .wrap(Wrap { trim: true })
        .style(Style::default());

    f.render_widget(stats_paragraph, area);
}

fn render_main_content(f: &mut Frame, app: &App, area: Rect) {
    if let Some(ollama_state) = &app.ollama_state {
        if ollama_state.get_selected_model().is_some() {
            render_chat_interface(f, app, area);
        } else {
            render_model_selection_prompt(f, area);
        }
    }
}

fn render_model_selection_prompt(f: &mut Frame, area: Rect) {
    let message = "Welcome to Snix Chat! 🚀\n\n👈 Please select a model from the sidebar\n\n✨ Features:\n• Smart conversations with your code\n• Persistent chat history\n• Advanced search & filtering\n• Snippet integration\n\n💡 Tip: Use Tab to navigate between panels";

    let prompt = Paragraph::new(message)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Cyan))
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .title(" 💬 Snix AI Chat ")
                .border_style(Style::default().fg(Color::Cyan)),
        );

    f.render_widget(prompt, area);
}

fn render_chat_interface(f: &mut Frame, app: &App, area: Rect) {
    if let Some(ollama_state) = &app.ollama_state {
        // Split main area to include space for scrollbar
        let main_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(40), Constraint::Length(1)])
            .split(area);

        let chat_area = main_layout[0];
        let scrollbar_area = main_layout[1];

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(5),    // Chat history
                Constraint::Length(4), // Input area
                Constraint::Length(1), // Footer
            ])
            .split(chat_area);

        // Header with model info
        render_chat_header(f, ollama_state, layout[0]);

        // Chat history with scrollbar
        render_chat_history(f, ollama_state, layout[1], scrollbar_area);

        // Input area
        render_chat_input(f, ollama_state, layout[2]);

        // Footer with shortcuts
        render_chat_footer(f, ollama_state, layout[3]);
    }
}

fn render_chat_header(f: &mut Frame, ollama_state: &OllamaState, area: Rect) {
    let model_name = ollama_state
        .get_selected_model()
        .unwrap_or(&"Unknown".to_string())
        .clone();

    let loading_chars = ["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];
    let animation_char = loading_chars[ollama_state.loading_animation_frame % loading_chars.len()];

    let header_content = if ollama_state.is_sending {
        format!(
            "🤖 {} {} Generating response...",
            model_name, animation_char
        )
    } else {
        let session_info = if let Some(session) = &ollama_state.current_session {
            format!(" • {} messages", session.get_message_count())
        } else {
            " • New conversation".to_string()
        };
        format!("🤖 {}{}", model_name, session_info)
    };

    let header_block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(if ollama_state.is_sending {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Cyan)
        })
        .title(" Current Chat ");

    let header_text = Paragraph::new(header_content)
        .block(header_block)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center);

    f.render_widget(header_text, area);
}

fn render_chat_history(
    f: &mut Frame,
    ollama_state: &OllamaState,
    area: Rect,
    scrollbar_area: Rect,
) {
    // Chat history container with rounded corners
    let chat_block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .title(" 💬 Conversation ")
        .title_style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(Color::Green));

    let chat_inner = chat_block.inner(area);
    f.render_widget(chat_block, area);

    // Calculate total height of all messages
    let mut total_height = 0;
    let message_heights: Vec<usize> = ollama_state
        .conversation
        .iter()
        .map(|msg| {
            let content_width = chat_inner.width.saturating_sub(4) as usize;
            let wrapped_height = calculate_wrapped_height(&msg.content, content_width);
            let height = wrapped_height + 2;
            total_height += height + 1;
            height
        })
        .collect();

    // Safely adjust scroll position
    let max_scroll = total_height.saturating_sub(chat_inner.height as usize);
    let scroll = ollama_state.scroll_position.min(max_scroll);

    // Calculate which messages to display based on scroll position
    let mut current_height = 0;
    let mut start_idx = 0;
    let mut start_offset = 0;

    for (idx, &height) in message_heights.iter().enumerate() {
        if current_height + height > scroll {
            start_idx = idx;
            start_offset = scroll - current_height;
            break;
        }
        current_height += height + 1;
    }

    // Render visible messages
    let mut y_offset: usize = 0;
    for idx in start_idx..ollama_state.conversation.len() {
        if y_offset >= chat_inner.height as usize {
            break;
        }

        let msg = &ollama_state.conversation[idx];
        let first_line_offset = if idx == start_idx { start_offset } else { 0 };

        // Determine message style based on role
        let (role_text, style, icon) = match msg.role {
            ChatRole::User => ("You", Style::default().fg(Color::Green), "👤"),
            ChatRole::Assistant => ("Assistant", Style::default().fg(Color::Blue), "🤖"),
            ChatRole::System => ("System", Style::default().fg(Color::Red), "⚙️"),
        };

        // Create message block
        let msg_block = Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(style)
            .title(Span::styled(
                format!("{} {}", icon, role_text),
                style.add_modifier(Modifier::BOLD),
            ));

        let msg_height = message_heights[idx];
        let visible_height = msg_height
            .saturating_sub(first_line_offset)
            .min(chat_inner.height as usize - y_offset);

        if visible_height == 0 {
            continue;
        }

        let msg_area = Rect::new(
            chat_inner.x,
            chat_inner.y + y_offset as u16,
            chat_inner.width,
            visible_height as u16,
        );

        f.render_widget(msg_block.clone(), msg_area);

        // Render message content
        let inner_msg_area = msg_block.inner(msg_area);
        if !inner_msg_area.is_empty() {
            let text = render_markdown(&msg.content, inner_msg_area.width as usize);
            let paragraph = Paragraph::new(text)
                .wrap(Wrap { trim: true })
                .scroll((first_line_offset as u16, 0));

            f.render_widget(paragraph, inner_msg_area);
        }

        y_offset += visible_height + 1;
    }

    // Show typing indicator
    if ollama_state.is_sending && !ollama_state.typing_indicator.is_empty() {
        let remaining_height = chat_inner.height as usize - y_offset;
        if remaining_height > 2 {
            let typing_area = Rect::new(
                chat_inner.x,
                chat_inner.y + y_offset as u16,
                chat_inner.width,
                3.min(remaining_height) as u16,
            );

            let typing_block = Block::default()
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .border_style(Style::default().fg(Color::Yellow))
                .title(Span::styled(
                    "🤖 Assistant",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ));

            f.render_widget(typing_block.clone(), typing_area);

            let typing_inner = typing_block.inner(typing_area);
            if !typing_inner.is_empty() {
                let typing_text = Paragraph::new(ollama_state.typing_indicator.clone()).style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::ITALIC),
                );
                f.render_widget(typing_text, typing_inner);
            }
        }
    }

    // Render scrollbar
    render_scrollbar(
        f,
        total_height,
        scroll,
        chat_inner.height as usize,
        scrollbar_area,
    );
}

fn render_scrollbar(
    f: &mut Frame,
    total_height: usize,
    scroll_position: usize,
    visible_height: usize,
    area: Rect,
) {
    if total_height <= visible_height {
        return;
    }

    let scrollbar_height = area.height as usize;
    let thumb_size = ((visible_height * scrollbar_height) / total_height).max(1);
    let thumb_position = (scroll_position * scrollbar_height) / total_height;

    // Draw scrollbar track
    for y in 0..scrollbar_height {
        let style = if y >= thumb_position && y < (thumb_position + thumb_size) {
            Style::default().bg(Color::Cyan).fg(Color::White)
        } else {
            Style::default().bg(Color::DarkGray).fg(Color::Gray)
        };

        f.render_widget(
            Block::default().style(style),
            Rect::new(area.x, area.y + y as u16, 1, 1),
        );
    }
}

fn render_chat_input(f: &mut Frame, ollama_state: &OllamaState, area: Rect) {
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .title(" 💭 Type your message ")
        .title_style(
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(if ollama_state.is_sending {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Blue)
        });

    f.render_widget(input_block.clone(), area);

    let input_inner = input_block.inner(area);

    let input_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(input_inner);

    let input_style = if ollama_state.is_sending {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    if ollama_state.is_sending {
        let input = Paragraph::new("🤖 Generating response...")
            .style(input_style)
            .wrap(Wrap { trim: false });
        f.render_widget(input, input_layout[0]);
    } else if ollama_state.input_buffer.is_empty() {
        let input = Paragraph::new("💭 Type your message here...")
            .style(input_style)
            .wrap(Wrap { trim: false });
        f.render_widget(input, input_layout[0]);
    } else {
        // Preserve spaces and add cursor - use a non-breaking approach
        use ratatui::text::{Line, Span, Text};

        let mut text_spans = Vec::new();

        // Process each character individually to prevent space collapsing
        for ch in ollama_state.input_buffer.chars() {
            let char_style = if ch == ' ' {
                // Make spaces slightly visible with background highlight
                input_style.bg(ratatui::style::Color::DarkGray)
            } else {
                input_style
            };
            text_spans.push(Span::styled(ch.to_string(), char_style));
        }

        // Add blinking cursor
        text_spans.push(Span::styled(
            "_",
            input_style.add_modifier(ratatui::style::Modifier::SLOW_BLINK),
        ));

        let text = Text::from(Line::from(text_spans));
        let input = Paragraph::new(text).wrap(Wrap { trim: false });

        f.render_widget(input, input_layout[0]);
    }

    // Show current snippet info if available
    if let Some(snippet) = &ollama_state.current_snippet {
        if !snippet.is_empty() {
            let snippet_text = format!("📄 Snippet: {} lines", snippet.lines().count());
            let snippet_info = Paragraph::new(snippet_text)
                .style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::ITALIC),
                )
                .alignment(Alignment::Right);

            f.render_widget(snippet_info, input_layout[1]);
        }
    }
}

fn render_chat_footer(f: &mut Frame, ollama_state: &OllamaState, area: Rect) {
    let shortcuts = if ollama_state.is_sending {
        "🔄 Generating... • ↑↓: Scroll • Tab: Switch panels • Esc: Cancel"
    } else {
        "↑↓: Scroll • PgUp/PgDn: Fast scroll • Tab: Switch panels • Ctrl+L: Clear • Enter: Send"
    };

    let footer = Paragraph::new(shortcuts)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(footer, area);
}

// Convert markdown to styled text for ratatui
fn render_markdown(markdown: &str, _width: usize) -> Text {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(markdown, options);

    let mut text = Text::default();
    let mut current_line = Line::default();
    let mut current_style = Style::default();
    let mut code_block = false;

    for event in parser {
        match event {
            Event::Start(Tag::Paragraph) => {
                if !current_line.spans.is_empty() {
                    text.lines.push(current_line);
                    current_line = Line::default();
                }
            }
            Event::End(TagEnd::Paragraph) => {
                if !current_line.spans.is_empty() {
                    text.lines.push(current_line);
                    current_line = Line::default();
                }
                text.lines.push(Line::default());
            }
            Event::Start(Tag::Heading { level, .. }) => {
                let level_style = match level {
                    HeadingLevel::H1 => Style::default()
                        .fg(Color::LightCyan)
                        .add_modifier(Modifier::BOLD),
                    HeadingLevel::H2 => Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                    _ => Style::default().add_modifier(Modifier::BOLD),
                };
                current_style = level_style;

                if !current_line.spans.is_empty() {
                    text.lines.push(current_line);
                    current_line = Line::default();
                }
            }
            Event::End(TagEnd::Heading(_)) => {
                if !current_line.spans.is_empty() {
                    text.lines.push(current_line);
                    current_line = Line::default();
                }
                current_style = Style::default();
                text.lines.push(Line::default());
            }
            Event::Start(Tag::CodeBlock(_)) => {
                if !current_line.spans.is_empty() {
                    text.lines.push(current_line);
                    current_line = Line::default();
                }
                code_block = true;
                text.lines.push(Line::from(vec![Span::styled(
                    "```",
                    Style::default().fg(Color::DarkGray),
                )]));
            }
            Event::End(TagEnd::CodeBlock) => {
                if !current_line.spans.is_empty() {
                    text.lines.push(current_line);
                    current_line = Line::default();
                }
                code_block = false;
                text.lines.push(Line::from(vec![Span::styled(
                    "```",
                    Style::default().fg(Color::DarkGray),
                )]));
                text.lines.push(Line::default());
            }
            Event::Start(Tag::Emphasis) => {
                current_style = current_style.add_modifier(Modifier::ITALIC);
            }
            Event::End(TagEnd::Emphasis) => {
                current_style = current_style.remove_modifier(Modifier::ITALIC);
            }
            Event::Start(Tag::Strong) => {
                current_style = current_style.add_modifier(Modifier::BOLD);
            }
            Event::End(TagEnd::Strong) => {
                current_style = current_style.remove_modifier(Modifier::BOLD);
            }
            Event::Code(text_str) => {
                current_line.spans.push(Span::styled(
                    format!("`{}`", text_str),
                    Style::default().fg(Color::LightMagenta),
                ));
            }
            Event::Text(text_str) => {
                let style = if code_block {
                    Style::default().fg(Color::LightYellow)
                } else {
                    current_style
                };

                let text_content = text_str.to_string();
                if text_content.contains('\n') {
                    for (i, line) in text_content.split('\n').enumerate() {
                        if i > 0 {
                            if !current_line.spans.is_empty() {
                                text.lines.push(current_line);
                                current_line = Line::default();
                            }
                        }
                        current_line
                            .spans
                            .push(Span::styled(line.to_string(), style));
                    }
                } else {
                    current_line.spans.push(Span::styled(text_content, style));
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if !current_line.spans.is_empty() {
                    text.lines.push(current_line);
                    current_line = Line::default();
                }
            }
            _ => {}
        }
    }

    if !current_line.spans.is_empty() {
        text.lines.push(current_line);
    }

    text
}

fn calculate_wrapped_height(text: &str, width: usize) -> usize {
    if width == 0 {
        return text.lines().count();
    }

    let mut height = 0;
    for line in text.lines() {
        if line.is_empty() {
            height += 1;
            continue;
        }

        let chars = line.chars().collect::<Vec<_>>();
        let mut line_width = 0;
        let mut line_count = 1;

        for c in chars {
            let char_width = UnicodeWidthChar::width(c).unwrap_or(1);
            if line_width + char_width > width {
                line_count += 1;
                line_width = char_width;
            } else {
                line_width += char_width;
            }
        }

        height += line_count;
    }

    height.max(1)
}

#[derive(Debug, Clone)]
pub struct ChatStorage {
    pub storage_dir: std::path::PathBuf,
}

impl ChatStorage {
    pub fn new() -> anyhow::Result<Self> {
        let storage_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
            .join(".snix")
            .join("ollama_chats");

        std::fs::create_dir_all(&storage_dir)?;

        Ok(Self { storage_dir })
    }

    pub fn save_session(&self, session: &ChatSession) -> anyhow::Result<()> {
        let file_path = self.storage_dir.join(format!("{}.json", session.id));
        let json = serde_json::to_string_pretty(session)?;
        std::fs::write(file_path, json)?;
        Ok(())
    }

    #[allow(dead_code)] // It's used upside
    pub fn load_session(&self, id: Uuid) -> anyhow::Result<ChatSession> {
        let file_path = self.storage_dir.join(format!("{}.json", id));
        let json = std::fs::read_to_string(file_path)?;
        let session = serde_json::from_str(&json)?;
        Ok(session)
    }

    pub fn delete_session(&self, id: Uuid) -> anyhow::Result<()> {
        let file_path = self.storage_dir.join(format!("{}.json", id));
        if file_path.exists() {
            std::fs::remove_file(file_path)?;
        }
        Ok(())
    }

    pub fn list_sessions(&self) -> anyhow::Result<Vec<ChatSession>> {
        let mut sessions = Vec::new();

        if !self.storage_dir.exists() {
            return Ok(sessions);
        }

        for entry in std::fs::read_dir(&self.storage_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match std::fs::read_to_string(&path) {
                    Ok(json) => match serde_json::from_str::<ChatSession>(&json) {
                        Ok(session) => sessions.push(session),
                        Err(e) => eprintln!("Failed to parse session file {:?}: {}", path, e),
                    },
                    Err(e) => eprintln!("Failed to read session file {:?}: {}", path, e),
                }
            }
        }

        // Sort by updated_at (most recent first)
        sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(sessions)
    }

    pub fn load_all_sessions(&self) -> anyhow::Result<Vec<ChatSession>> {
        self.list_sessions()
    }

    #[allow(dead_code)] // It's used as a pub fn
    pub fn export_session(
        &self,
        session: &ChatSession,
        format: ExportFormat,
    ) -> anyhow::Result<String> {
        match format {
            ExportFormat::Json => Ok(serde_json::to_string_pretty(session)?),
            ExportFormat::Markdown => {
                let mut output = String::new();
                output.push_str(&format!("# {}\n\n", session.title));
                output.push_str(&format!("**Model:** {}\n", session.model_name));
                output.push_str(&format!(
                    "**Created:** {}\n",
                    session.created_at.format("%Y-%m-%d %H:%M:%S UTC")
                ));
                output.push_str(&format!(
                    "**Updated:** {}\n",
                    session.updated_at.format("%Y-%m-%d %H:%M:%S UTC")
                ));

                if let Some(snippet_title) = &session.snippet_title {
                    output.push_str(&format!("**Code Snippet:** {}\n", snippet_title));
                }

                if !session.tags.is_empty() {
                    output.push_str(&format!("**Tags:** {}\n", session.tags.join(", ")));
                }

                output.push_str("\n---\n\n");

                for msg in &session.conversation {
                    let role = match msg.role {
                        ChatRole::User => "🧑 **User**",
                        ChatRole::Assistant => "🤖 **Assistant**",
                        ChatRole::System => "⚙️ **System**",
                    };
                    output.push_str(&format!("{}\n\n{}\n\n---\n\n", role, msg.content));
                }

                Ok(output)
            }
        }
    }

    pub fn get_storage_stats(&self) -> anyhow::Result<StorageStats> {
        let sessions = self.list_sessions()?;
        let total_sessions = sessions.len();
        let total_messages = sessions.iter().map(|s| s.conversation.len()).sum();

        let mut models = std::collections::HashMap::new();
        for session in &sessions {
            *models.entry(session.model_name.clone()).or_insert(0) += 1;
        }

        let storage_size = if self.storage_dir.exists() {
            calculate_dir_size(&self.storage_dir)?
        } else {
            0
        };

        Ok(StorageStats {
            total_sessions,
            total_messages,
            models,
            storage_size_bytes: storage_size,
        })
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ExportFormat {
    Json,
    Markdown,
}

#[derive(Debug, Clone)]
pub struct StorageStats {
    pub total_sessions: usize,
    pub total_messages: usize,
    pub models: std::collections::HashMap<String, usize>,
    pub storage_size_bytes: u64,
}

fn calculate_dir_size(dir: &std::path::Path) -> anyhow::Result<u64> {
    let mut size = 0;
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                size += entry.metadata()?.len();
            } else if path.is_dir() {
                size += calculate_dir_size(&path)?;
            }
        }
    }
    Ok(size)
}

use crate::app::App;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

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
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChatRole {
    User,
    Assistant,
    System,
}

impl Default for OllamaState {
    fn default() -> Self {
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
        }
    }
}

impl OllamaState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_message(&mut self, role: ChatRole, content: String) {
        self.conversation.push(ChatMessage { role, content });
    }

    pub fn get_selected_model(&self) -> Option<&String> {
        if self.models.is_empty() {
            None
        } else {
            Some(&self.models[self.selected_model_index])
        }
    }
}

pub fn render_ollama_popup(f: &mut Frame, app: &App, area: Rect) {
    if let Some(ollama_state) = &app.ollama_state {
        if !ollama_state.show_popup {
            return;
        }

        let popup_width = area.width.min(100).max(60);
        let popup_height = area.height.min(40).max(20);
        let popup_x = (area.width - popup_width) / 2;
        let popup_y = (area.height - popup_height) / 2;
        let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

        f.render_widget(Clear, popup_area);

        let popup_block = Block::default()
            .title(" Ollama Chat ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        f.render_widget(popup_block.clone(), popup_area);

        let inner_area = popup_block.inner(popup_area);

        if ollama_state.loading_models {
            let loading_text = Paragraph::new("Loading Ollama models...")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Yellow));

            f.render_widget(loading_text, inner_area);
        } else if ollama_state.models.is_empty() {
            let message = if let Some(error) = &ollama_state.error_message {
                format!(
                    "Error: {}\n\nMake sure Ollama is running and try again.",
                    error
                )
            } else {
                "No Ollama models found.\n\nMake sure Ollama is installed and running.".to_string()
            };

            let error_text = Paragraph::new(message)
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Red));

            f.render_widget(error_text, inner_area);
        } else if ollama_state.get_selected_model().is_some() {
            render_chat_interface(f, app, inner_area);
        } else {
            render_model_selection(f, app, inner_area);
        }
    }
}

fn render_model_selection(f: &mut Frame, app: &App, area: Rect) {
    if let Some(ollama_state) = &app.ollama_state {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Min(3),
                Constraint::Length(1),
            ])
            .split(area);

        let header = Paragraph::new("Select an Ollama model to chat with:")
            .style(Style::default().fg(Color::Cyan));
        f.render_widget(header, layout[0]);

        let models: Vec<ListItem> = ollama_state
            .models
            .iter()
            .enumerate()
            .map(|(i, model)| {
                let style = if i == ollama_state.selected_model_index {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                ListItem::new(model.clone()).style(style)
            })
            .collect();

        let models_list = List::new(models).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Available Models "),
        );

        f.render_widget(models_list, layout[1]);

        let footer = Paragraph::new("Press Enter to select, Esc to close")
            .style(Style::default().fg(Color::Gray));
        f.render_widget(footer, layout[2]);
    }
}

fn render_chat_interface(f: &mut Frame, app: &App, area: Rect) {
    if let Some(ollama_state) = &app.ollama_state {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(5),
                Constraint::Length(3),
            ])
            .split(area);

        let model_name = match ollama_state.get_selected_model() {
            Some(model) => model.clone(),
            None => "Unknown".to_string(),
        };
        let header = Paragraph::new(format!("Chatting with: {}", model_name))
            .style(Style::default().fg(Color::Cyan));
        f.render_widget(header, layout[0]);

        // Chat history
        let chat_block = Block::default()
            .borders(Borders::ALL)
            .title(" Conversation ");

        let chat_area = chat_block.inner(layout[1]);
        f.render_widget(chat_block, layout[1]);

        // Calculate total height of all messages
        let mut total_height = 0;
        let message_heights: Vec<usize> = ollama_state
            .conversation
            .iter()
            .map(|msg| {
                let mut text = Text::from(vec![Line::from("")]);
                for line in msg.content.lines() {
                    text.extend(Text::from(line));
                }
                let height = text.height() + 1;
                total_height += height;
                height
            })
            .collect();

        // Adjust scroll position if needed
        let max_scroll = total_height.saturating_sub(chat_area.height as usize);
        let scroll = ollama_state.scroll_position.min(max_scroll);

        // Calculate which messages to display based on scroll position
        let mut current_height = 0;
        let mut start_idx = 0;
        let mut start_offset = 0;

        // Find the first visible message
        for (idx, &height) in message_heights.iter().enumerate() {
            if current_height + height > scroll {
                start_idx = idx;
                start_offset = scroll - current_height;
                break;
            }
            current_height += height;
        }

        // Render visible messages
        let mut y_offset = 0;
        for idx in start_idx..ollama_state.conversation.len() {
            let msg = &ollama_state.conversation[idx];
            let (prefix, style) = match msg.role {
                ChatRole::User => ("You: ", Style::default().fg(Color::Green)),
                ChatRole::Assistant => ("Assistant: ", Style::default().fg(Color::Cyan)),
                ChatRole::System => ("System: ", Style::default().fg(Color::Yellow)),
            };

            // Create text with proper prefix styling
            let mut text = Text::from(vec![Line::from(vec![
                Span::styled(prefix, style.add_modifier(Modifier::BOLD)),
                Span::raw(""),
            ])]);

            // Add content with proper wrapping
            for line in msg.content.lines() {
                text.extend(Text::from(line));
            }

            // Calculate height needed for this message
            let text_height = text.height();

            // Skip the first message partially if needed
            let effective_y = if idx == start_idx {
                y_offset = text_height.saturating_sub(start_offset);
                0
            } else {
                y_offset
            };

            // Check if we have space to render this message
            if effective_y < chat_area.height as usize {
                let visible_height = (chat_area.height as usize - effective_y).min(text_height);
                let msg_area = Rect::new(
                    chat_area.x,
                    chat_area.y + effective_y as u16,
                    chat_area.width,
                    visible_height as u16,
                );

                let paragraph = Paragraph::new(text).wrap(Wrap { trim: true });

                f.render_widget(paragraph, msg_area);
                y_offset += text_height + 1;
            } else {
                // Not enough space, stop rendering
                break;
            }
        }

        // Input area
        let input_style = if ollama_state.is_sending {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default()
        };

        let input_placeholder = if ollama_state.is_sending {
            "Processing... Please wait for the response"
        } else {
            "Type your message and press Enter to send (Esc to close)"
        };

        let input_text = if ollama_state.input_buffer.is_empty() {
            input_placeholder.to_string()
        } else {
            ollama_state.input_buffer.clone()
        };

        let input = Paragraph::new(input_text)
            .style(input_style)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Your Message "),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(input, layout[2]);

        // Show scrolling hint if needed
        if total_height > chat_area.height as usize {
            let scroll_info = format!(
                "↑↓ to scroll ({}/{})",
                ollama_state.scroll_position.min(max_scroll),
                max_scroll
            );

            let scroll_hint = Paragraph::new(scroll_info)
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Right);

            let hint_area = Rect::new(layout[0].x, layout[0].y, layout[0].width, layout[0].height);

            f.render_widget(scroll_hint, hint_area);
        }
    }
}

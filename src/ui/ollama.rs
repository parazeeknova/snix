use crate::app::App;
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

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
            scroll_speed: 3,
            loading_animation_frame: 0,
            pending_response_id: None,
            typing_indicator: String::new(),
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
        if self.models.is_empty() {
            None
        } else {
            Some(&self.models[self.selected_model_index])
        }
    }

    pub fn update_typing_indicator(&mut self) {
        if self.is_sending {
            let dots = match (self.loading_animation_frame / 10) % 4 {
                0 => "",
                1 => ".",
                2 => "..",
                _ => "...",
            };
            self.typing_indicator = format!("Assistant is typing{}", dots);
        } else {
            self.typing_indicator.clear();
        }
    }

    pub fn auto_scroll_to_bottom(&mut self) {
        self.scroll_position = 999999;
    }
}

pub fn render_ollama_popup(f: &mut Frame, app: &App, area: Rect) {
    if let Some(ollama_state) = &app.ollama_state {
        if !ollama_state.show_popup {
            return;
        }

        let popup_width = area.width.min(120).max(90);
        let popup_height = area.height.min(50).max(35);
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
            let loading_chars = ["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];
            let animation_char =
                loading_chars[ollama_state.loading_animation_frame % loading_chars.len()];

            let loading_text =
                Paragraph::new(format!("Loading Ollama models... {}", animation_char))
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

        let footer = Paragraph::new("↑↓: Navigate • Enter: Select • Esc: Close")
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
                Constraint::Length(1),
            ])
            .split(area);

        let model_name = match ollama_state.get_selected_model() {
            Some(model) => model.clone(),
            None => "Unknown".to_string(),
        };

        // Show model name and loading indicator if processing
        let loading_chars = ["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];
        let animation_char =
            loading_chars[ollama_state.loading_animation_frame % loading_chars.len()];
        let header_text = if ollama_state.is_sending {
            format!(
                "Chatting with: {} {} Generating response...",
                model_name, animation_char
            )
        } else {
            format!("Chatting with: {}", model_name)
        };

        let header = Paragraph::new(header_text).style(Style::default().fg(Color::Cyan));
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
                let content_width = chat_area.width.saturating_sub(4) as usize; // Account for padding
                let wrapped_height = calculate_wrapped_height(&msg.content, content_width);
                let height = wrapped_height + 2; // Add padding for message box
                total_height += height + 1; // Add 1 for spacing between messages
                height
            })
            .collect();

        // Safely adjust scroll position if needed
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
            current_height += height + 1;
        }

        // Render visible messages
        let mut y_offset: usize = 0;
        for idx in start_idx..ollama_state.conversation.len() {
            if y_offset >= chat_area.height as usize {
                break;
            }

            let msg = &ollama_state.conversation[idx];
            let content_width = chat_area.width.saturating_sub(4) as usize;

            // Skip part of the first visible message if needed
            let first_line_offset = if idx == start_idx { start_offset } else { 0 };

            // Determine message style based on role
            let (role_text, style) = match msg.role {
                ChatRole::User => ("You:", Style::default().fg(Color::Green)),
                ChatRole::Assistant => ("Assistant:", Style::default().fg(Color::Blue)),
                ChatRole::System => ("System:", Style::default().fg(Color::Red)),
            };

            // Create message block
            let msg_block = Block::default()
                .borders(Borders::ALL)
                .border_style(style)
                .title(Span::styled(role_text, style));

            // Calculate message area
            let msg_height = message_heights[idx];
            let visible_height = msg_height
                .saturating_sub(first_line_offset)
                .min(chat_area.height as usize - y_offset);

            if visible_height == 0 {
                continue;
            }

            let msg_area = Rect::new(
                chat_area.x,
                chat_area.y + y_offset as u16,
                chat_area.width,
                visible_height as u16,
            );

            // Render message block
            f.render_widget(msg_block.clone(), msg_area);

            // Render message content with markdown parsing
            let inner_msg_area = msg_block.inner(msg_area);
            if !inner_msg_area.is_empty() {
                let text = render_markdown(&msg.content, content_width);
                let paragraph = Paragraph::new(text)
                    .wrap(Wrap { trim: true })
                    .scroll((first_line_offset as u16, 0));

                f.render_widget(paragraph, inner_msg_area);
            }

            y_offset += visible_height;

            // Add spacing between messages
            y_offset += 1;
        }

        // Show typing indicator if the assistant is responding
        if ollama_state.is_sending && !ollama_state.typing_indicator.is_empty() {
            let remaining_height = chat_area.height as usize - y_offset;
            if remaining_height > 2 {
                let typing_area = Rect::new(
                    chat_area.x,
                    chat_area.y + y_offset as u16,
                    chat_area.width,
                    3.min(remaining_height) as u16,
                );

                let typing_block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow))
                    .title(Span::styled(
                        "Assistant:",
                        Style::default().fg(Color::Yellow),
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

        // Input area
        let input_block = Block::default()
            .borders(Borders::ALL)
            .title(" Message ")
            .border_style(if ollama_state.is_sending {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Blue)
            });

        f.render_widget(input_block.clone(), layout[2]);

        let input_area = input_block.inner(layout[2]);

        let input_style = if ollama_state.is_sending {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default()
        };

        let input_text = if ollama_state.is_sending {
            "Generating response..."
        } else if ollama_state.input_buffer.is_empty() {
            "Type your message here..."
        } else {
            &ollama_state.input_buffer
        };

        let input = Paragraph::new(input_text)
            .style(input_style)
            .wrap(Wrap { trim: true });

        f.render_widget(input, input_area);

        // Show current snippet if available
        if let Some(snippet) = &ollama_state.current_snippet {
            if !snippet.is_empty() {
                let snippet_preview = format!("Current snippet: {} lines", snippet.lines().count());
                let snippet_info = Paragraph::new(snippet_preview.clone())
                    .style(Style::default().fg(Color::Yellow))
                    .alignment(Alignment::Right);

                // Create a small area for the snippet info at the bottom right
                let info_area = Rect::new(
                    layout[2].x + layout[2].width - snippet_preview.width() as u16 - 4,
                    layout[2].y + layout[2].height - 1,
                    snippet_preview.width() as u16 + 2,
                    1,
                );

                f.render_widget(snippet_info, info_area);
            }
        }

        let shortcuts = if ollama_state.is_sending {
            "Generating response... • ↑↓: Scroll • Esc: Close"
        } else {
            "↑↓: Scroll • PgUp/PgDn: Fast Scroll • Ctrl+L: Clear • Ctrl+End: Bottom • Esc: Close"
        };

        let footer = Paragraph::new(shortcuts)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(footer, layout[3]);
    }
}

// Convert markdown to styled text for ratatui
fn render_markdown(markdown: &str, width: usize) -> Text {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(markdown, options);

    let mut text = Text::default();
    let mut current_line = Line::default();
    let mut current_style = Style::default();
    let mut code_block = false;
    let mut list_indent: usize = 0;

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
                // Add empty line after paragraphs
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
                // Reset style and add empty line
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
            Event::Start(Tag::List(_)) => {
                list_indent += 2;
            }
            Event::End(TagEnd::List(_)) => {
                list_indent = list_indent.saturating_sub(2);
                if !current_line.spans.is_empty() {
                    text.lines.push(current_line);
                    current_line = Line::default();
                }
            }
            Event::Start(Tag::Item) => {
                if !current_line.spans.is_empty() {
                    text.lines.push(current_line);
                    current_line = Line::default();
                }

                // Add indentation and bullet
                let indent = " ".repeat(list_indent.saturating_sub(2));
                current_line.spans.push(Span::raw(indent));
                current_line
                    .spans
                    .push(Span::styled("• ", Style::default().fg(Color::Yellow)));
            }
            Event::End(TagEnd::Item) => {
                if !current_line.spans.is_empty() {
                    text.lines.push(current_line);
                    current_line = Line::default();
                }
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

                // Handle line wrapping for long text
                let text_content = text_str.to_string();
                if text_content.contains('\n') {
                    for (i, line) in text_content.split('\n').enumerate() {
                        if i > 0 {
                            if !current_line.spans.is_empty() {
                                text.lines.push(current_line);
                                current_line = Line::default();
                            }

                            if list_indent > 0 {
                                let indent = " ".repeat(list_indent);
                                current_line.spans.push(Span::raw(indent));
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
            Event::SoftBreak => {
                if !current_line.spans.is_empty() {
                    text.lines.push(current_line);
                    current_line = Line::default();
                }
            }
            Event::HardBreak => {
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

    // Apply width-based wrapping if needed
    if width > 0 && !text.lines.is_empty() {
        // The text will be wrapped by the Paragraph widget's Wrap option
        // so we don't need to do anything special here
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

    height.max(1) // Ensure at least one line
}

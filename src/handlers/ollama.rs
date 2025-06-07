use crate::app::App;
use crate::ui::ollama::ChatRole;
use anyhow::{Result, anyhow};
use flume::{Receiver, Sender};
use ollama_rs::{Ollama, generation::completion::request::GenerationRequest, models::ModelOptions};
use std::thread;
use tokio::runtime::Runtime;

// Global channel for async communication
static OLLAMA_CHANNEL: once_cell::sync::Lazy<(Sender<OllamaMessage>, Receiver<OllamaMessage>)> =
    once_cell::sync::Lazy::new(|| flume::unbounded());

#[derive(Debug, Clone)]
pub enum OllamaMessage {
    ModelsLoaded(Result<Vec<String>, String>),
    ResponseChunk {
        id: u64,
        content: String,
        is_final: bool,
    },
    Error {
        id: u64,
        error: String,
    },
}

pub fn fetch_ollama_models(app: &mut App) -> Result<()> {
    if let Some(ollama_state) = &mut app.ollama_state {
        ollama_state.loading_models = true;
        ollama_state.error_message = None;
        ollama_state.models.clear();

        let sender = OLLAMA_CHANNEL.0.clone();

        // Spawn a thread to run the async Ollama API call
        thread::spawn(move || {
            let rt = match Runtime::new() {
                Ok(rt) => rt,
                Err(e) => {
                    let _ = sender.send(OllamaMessage::ModelsLoaded(Err(format!(
                        "Runtime error: {}",
                        e
                    ))));
                    return;
                }
            };

            let result = rt.block_on(async {
                let ollama = Ollama::default();
                ollama.list_local_models().await
            });

            match result {
                Ok(models_list) => {
                    let model_names: Vec<String> =
                        models_list.iter().map(|model| model.name.clone()).collect();
                    let _ = sender.send(OllamaMessage::ModelsLoaded(Ok(model_names)));
                }
                Err(e) => {
                    let _ = sender.send(OllamaMessage::ModelsLoaded(Err(format!("Error: {}", e))));
                }
            }
        });

        Ok(())
    } else {
        Err(anyhow!("Ollama state not initialized"))
    }
}

pub fn send_message_to_ollama(app: &mut App, message: String) -> Result<()> {
    if let Some(ollama_state) = &mut app.ollama_state {
        if let Some(model) = ollama_state.get_selected_model().cloned() {
            // Add user message to conversation immediately
            ollama_state.add_message(ChatRole::User, message.clone());

            // Set sending state and clear input buffer
            ollama_state.is_sending = true;
            ollama_state.input_buffer.clear();

            // Generate unique ID for this request
            let request_id = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64;

            ollama_state.pending_response_id = Some(request_id);

            let sender = OLLAMA_CHANNEL.0.clone();
            let model_clone = model.clone();
            let snippet = ollama_state.current_snippet.clone().unwrap_or_default();

            // Prepare the prompt
            let prompt = if !snippet.is_empty() {
                format!(
                    "The following is a code snippet. Please help me understand or improve it:\n\n```\n{}\n```\n\n{}",
                    snippet, message
                )
            } else {
                message
            };

            // Spawn a thread to run the async Ollama API call
            thread::spawn(move || {
                let rt = match Runtime::new() {
                    Ok(rt) => rt,
                    Err(e) => {
                        let _ = sender.send(OllamaMessage::Error {
                            id: request_id,
                            error: format!("Runtime error: {}", e),
                        });
                        return;
                    }
                };

                let result = rt.block_on(async {
                    let ollama = Ollama::default();
                    let options = ModelOptions::default().temperature(0.7).num_predict(2048);
                    let request = GenerationRequest::new(model_clone, prompt).options(options);
                    ollama.generate(request).await
                });

                match result {
                    Ok(response) => {
                        let _ = sender.send(OllamaMessage::ResponseChunk {
                            id: request_id,
                            content: response.response,
                            is_final: true,
                        });
                    }
                    Err(e) => {
                        let _ = sender.send(OllamaMessage::Error {
                            id: request_id,
                            error: format!("Error: {}", e),
                        });
                    }
                }
            });
        }
    }

    Ok(())
}

pub fn update_loading_animation(app: &mut App) {
    // Process any pending Ollama messages first
    process_ollama_messages(app);

    // Check if ollama_state exists before trying to access it
    if let Some(ollama_state) = &mut app.ollama_state {
        if ollama_state.is_sending || ollama_state.loading_models {
            // Update the animation frame safely
            ollama_state.loading_animation_frame =
                ollama_state.loading_animation_frame.wrapping_add(1);

            // Update typing indicator
            ollama_state.update_typing_indicator();
        }
    }
}

pub fn process_ollama_messages(app: &mut App) {
    let receiver = &OLLAMA_CHANNEL.1;

    // Process all available messages without blocking
    while let Ok(message) = receiver.try_recv() {
        if let Some(ollama_state) = &mut app.ollama_state {
            match message {
                OllamaMessage::ModelsLoaded(result) => {
                    ollama_state.loading_models = false;
                    match result {
                        Ok(models) => {
                            if models.is_empty() {
                                ollama_state.error_message = Some("No models found".to_string());
                            } else {
                                ollama_state.models = models;
                                ollama_state.error_message = None;
                            }
                        }
                        Err(error) => {
                            ollama_state.error_message = Some(error);
                        }
                    }
                }
                OllamaMessage::ResponseChunk {
                    id,
                    content,
                    is_final,
                } => {
                    // Only process if this matches the current pending request
                    if ollama_state.pending_response_id == Some(id) {
                        if is_final {
                            ollama_state.add_message(ChatRole::Assistant, content);
                            ollama_state.is_sending = false;
                            ollama_state.pending_response_id = None;
                            ollama_state.typing_indicator.clear();
                        }
                        // For streaming responses, we could append chunks here
                    }
                }
                OllamaMessage::Error { id, error } => {
                    // Only process if this matches the current pending request
                    if ollama_state.pending_response_id == Some(id) {
                        ollama_state.add_message(ChatRole::System, error);
                        ollama_state.is_sending = false;
                        ollama_state.pending_response_id = None;
                        ollama_state.typing_indicator.clear();
                    }
                }
            }
        }
    }
}

pub fn handle_ollama_keys(app: &mut App, key: ratatui::crossterm::event::KeyEvent) -> bool {
    if let Some(ollama_state) = &mut app.ollama_state {
        if !ollama_state.show_popup {
            return false;
        }

        use ratatui::crossterm::event::{KeyCode, KeyModifiers};

        match key.code {
            KeyCode::Esc => {
                ollama_state.show_popup = false;
                false
            }

            // Model selection navigation
            KeyCode::Up | KeyCode::Char('k') => {
                if ollama_state.get_selected_model().is_none() && !ollama_state.models.is_empty() {
                    if ollama_state.selected_model_index > 0 {
                        ollama_state.selected_model_index -= 1;
                    } else {
                        ollama_state.selected_model_index = ollama_state.models.len() - 1;
                    }
                } else if ollama_state.get_selected_model().is_some() && !ollama_state.is_sending {
                    // Only allow scrolling when not sending (more responsive)
                    ollama_state.scroll_position = ollama_state
                        .scroll_position
                        .saturating_sub(ollama_state.scroll_speed);
                }
                false
            }

            KeyCode::Down | KeyCode::Char('j') => {
                if ollama_state.get_selected_model().is_none() && !ollama_state.models.is_empty() {
                    ollama_state.selected_model_index =
                        (ollama_state.selected_model_index + 1) % ollama_state.models.len();
                } else if ollama_state.get_selected_model().is_some() && !ollama_state.is_sending {
                    // Only allow scrolling when not sending (more responsive)
                    ollama_state.scroll_position += ollama_state.scroll_speed;
                    // The max scroll is handled in the render function
                }
                false
            }

            // Page up/down for faster scrolling
            KeyCode::PageUp => {
                if ollama_state.get_selected_model().is_some() && !ollama_state.is_sending {
                    // Scroll up by 10 lines * scroll speed
                    ollama_state.scroll_position = ollama_state
                        .scroll_position
                        .saturating_sub(10 * ollama_state.scroll_speed);
                }
                false
            }

            KeyCode::PageDown => {
                if ollama_state.get_selected_model().is_some() && !ollama_state.is_sending {
                    // Scroll down by 10 lines * scroll speed
                    ollama_state.scroll_position += 10 * ollama_state.scroll_speed;
                    // The max scroll is handled in the render function
                }
                false
            }

            // Home/End for jumping to top/bottom
            KeyCode::Home => {
                if ollama_state.get_selected_model().is_some() && !ollama_state.is_sending {
                    ollama_state.scroll_position = 0;
                }
                false
            }

            // Auto-scroll to bottom with Ctrl+End (must come before general End pattern)
            KeyCode::End if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if ollama_state.get_selected_model().is_some() {
                    ollama_state.auto_scroll_to_bottom();
                }
                false
            }

            KeyCode::End => {
                if ollama_state.get_selected_model().is_some() && !ollama_state.is_sending {
                    // Set to a large number that will be safely clamped in render
                    ollama_state.scroll_position = 999999;
                }
                false
            }

            // Model selection or sending a message
            KeyCode::Enter => {
                if ollama_state.is_sending {
                    // Don't allow new messages while processing
                    false
                } else if ollama_state.get_selected_model().is_none() {
                    // Select model if on model selection screen
                    if !ollama_state.models.is_empty() {
                        // Add a welcome message
                        ollama_state.conversation.clear();
                        ollama_state.add_message(
                            ChatRole::System,
                            format!(
                                "Selected model: {}. Type your message and press Enter to send.",
                                ollama_state.models[ollama_state.selected_model_index]
                            ),
                        );
                    }
                    false
                } else if !ollama_state.input_buffer.trim().is_empty() {
                    // Send message if input is not empty
                    let message = ollama_state.input_buffer.clone();
                    // This will be handled by the app's update function
                    let _ = send_message_to_ollama(app, message);
                    false
                } else {
                    false
                }
            }

            // Clear conversation with Ctrl+L (must come before general Char pattern)
            KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if ollama_state.get_selected_model().is_some() && !ollama_state.is_sending {
                    ollama_state.conversation.clear();
                    ollama_state.scroll_position = 0;
                    if let Some(snippet) = &ollama_state.current_snippet {
                        // Re-add system message if we have a snippet
                        ollama_state.add_message(
                            ChatRole::System,
                            format!(
                                "Conversation cleared. Working with snippet ({} lines)",
                                snippet.lines().count()
                            ),
                        );
                    } else {
                        ollama_state
                            .add_message(ChatRole::System, "Conversation cleared.".to_string());
                    }
                }
                false
            }

            // Input handling
            KeyCode::Char(c) => {
                if ollama_state.get_selected_model().is_some() && !ollama_state.is_sending {
                    ollama_state.input_buffer.push(c);
                }
                false
            }

            KeyCode::Backspace => {
                if ollama_state.get_selected_model().is_some() && !ollama_state.is_sending {
                    ollama_state.input_buffer.pop();
                }
                false
            }

            _ => false,
        }
    } else {
        false
    }
}

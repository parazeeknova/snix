use crate::app::App;
use crate::ui::ollama::ChatRole;
use anyhow::{Result, anyhow};
use ollama_rs::{Ollama, generation::completion::request::GenerationRequest, models::ModelOptions};
use std::sync::mpsc;
use std::thread;
use tokio::runtime::Runtime;

pub fn fetch_ollama_models(app: &mut App) -> Result<()> {
    if let Some(ollama_state) = &mut app.ollama_state {
        ollama_state.loading_models = true;
        ollama_state.error_message = None;
        ollama_state.models.clear();

        // Create a channel for communication between threads
        let (tx, rx) = mpsc::channel();

        // Spawn a thread to run the async Ollama API call
        thread::spawn(move || {
            // Create a new Tokio runtime for async operations
            let rt = match Runtime::new() {
                Ok(rt) => rt,
                Err(_) => {
                    // If we can't create a runtime, just ignore this thread
                    return;
                }
            };

            let result = rt.block_on(async {
                let ollama = Ollama::default();
                ollama.list_local_models().await
            });
            let _ = tx.send(result);
        });

        // Wait for the result with a timeout
        match rx.recv_timeout(std::time::Duration::from_secs(5)) {
            Ok(result) => match result {
                Ok(models_list) => {
                    if models_list.is_empty() {
                        ollama_state.error_message = Some("No models found".to_string());
                    } else {
                        // Extract model names from the response
                        let model_names: Vec<String> =
                            models_list.iter().map(|model| model.name.clone()).collect();

                        ollama_state.models = model_names;
                    }
                }
                Err(e) => {
                    ollama_state.error_message = Some(format!("Error: {}", e));
                }
            },
            Err(_) => {
                ollama_state.error_message =
                    Some("Timeout waiting for Ollama response".to_string());
            }
        }

        ollama_state.loading_models = false;
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

            // Create a channel for communication
            let (tx, rx) = mpsc::channel();

            // Clone necessary data for the thread
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
                // Create a new Tokio runtime for async operations
                let rt = match Runtime::new() {
                    Ok(rt) => rt,
                    Err(_) => {
                        // If we can't create a runtime, just ignore this thread
                        return;
                    }
                };

                let result = rt.block_on(async {
                    let ollama = Ollama::default();

                    // Create generation request with options using builder pattern
                    let options = ModelOptions::default().temperature(0.7).num_predict(2048);

                    let request = GenerationRequest::new(model_clone, prompt).options(options);

                    // Generate response
                    ollama.generate(request).await
                });
                let _ = tx.send(result);
            });

            // Wait for the result with a timeout (2 minutes should be enough for most responses)
            match rx.recv_timeout(std::time::Duration::from_secs(120)) {
                Ok(result) => match result {
                    Ok(response) => {
                        ollama_state.add_message(ChatRole::Assistant, response.response);
                    }
                    Err(e) => {
                        ollama_state.add_message(ChatRole::System, format!("Error: {}", e));
                    }
                },
                Err(_) => {
                    ollama_state.add_message(
                        ChatRole::System,
                        "Error: Timeout waiting for response".to_string(),
                    );
                }
            }

            ollama_state.is_sending = false;
        }
    }

    Ok(())
}

pub fn update_loading_animation(app: &mut App) {
    // Check if ollama_state exists before trying to access it
    if let Some(ollama_state) = &mut app.ollama_state {
        if ollama_state.is_sending || ollama_state.loading_models {
            // Update the animation frame safely
            ollama_state.loading_animation_frame =
                ollama_state.loading_animation_frame.wrapping_add(1);
        }
    }
}

pub fn handle_ollama_keys(app: &mut App, key: ratatui::crossterm::event::KeyEvent) -> bool {
    if let Some(ollama_state) = &mut app.ollama_state {
        if !ollama_state.show_popup {
            return false;
        }

        use ratatui::crossterm::event::KeyCode;

        match key.code {
            // Close popup
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
                } else if ollama_state.get_selected_model().is_some() {
                    // Scroll up in chat view with speed factor
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
                } else if ollama_state.get_selected_model().is_some() {
                    // Scroll down in chat view with speed factor
                    ollama_state.scroll_position += ollama_state.scroll_speed;
                    // The max scroll is handled in the render function
                }
                false
            }

            // Page up/down for faster scrolling
            KeyCode::PageUp => {
                if ollama_state.get_selected_model().is_some() {
                    // Scroll up by 10 lines * scroll speed
                    ollama_state.scroll_position = ollama_state
                        .scroll_position
                        .saturating_sub(10 * ollama_state.scroll_speed);
                }
                false
            }

            KeyCode::PageDown => {
                if ollama_state.get_selected_model().is_some() {
                    // Scroll down by 10 lines * scroll speed
                    ollama_state.scroll_position += 10 * ollama_state.scroll_speed;
                    // The max scroll is handled in the render function
                }
                false
            }

            // Home/End for jumping to top/bottom
            KeyCode::Home => {
                if ollama_state.get_selected_model().is_some() {
                    ollama_state.scroll_position = 0;
                }
                false
            }

            KeyCode::End => {
                if ollama_state.get_selected_model().is_some() {
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

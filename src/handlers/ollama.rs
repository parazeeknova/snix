use crate::app::App;
use crate::ui::ollama::ChatRole;
use anyhow::Result;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;

pub fn fetch_ollama_models(app: &mut App) -> Result<()> {
    if let Some(ollama_state) = &mut app.ollama_state {
        ollama_state.loading_models = true;
        ollama_state.error_message = None;
        ollama_state.models.clear();

        // Create a channel for communication between threads
        let (tx, rx) = mpsc::channel();

        // Spawn a thread to run the command
        thread::spawn(move || {
            let result = Command::new("ollama")
                .arg("list")
                .output()
                .map_err(|e| e.to_string());

            let _ = tx.send(result);
        });

        // Wait for the result with a timeout
        match rx.recv_timeout(std::time::Duration::from_secs(5)) {
            Ok(result) => match result {
                Ok(output) => {
                    if output.status.success() {
                        let output_str = String::from_utf8_lossy(&output.stdout).to_string();
                        let models = parse_ollama_models(&output_str);

                        if models.is_empty() {
                            ollama_state.error_message = Some("No models found".to_string());
                        } else {
                            ollama_state.models = models;
                        }
                    } else {
                        let error = String::from_utf8_lossy(&output.stderr).to_string();
                        ollama_state.error_message = Some(error);
                    }
                }
                Err(e) => {
                    ollama_state.error_message = Some(e);
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
        Err(anyhow::anyhow!("Ollama state not initialized"))
    }
}

fn parse_ollama_models(output: &str) -> Vec<String> {
    let mut models = Vec::new();

    // Skip the header line
    for line in output.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if !parts.is_empty() {
            models.push(parts[0].to_string());
        }
    }

    models
}

pub fn send_message_to_ollama(app: &mut App, message: String) -> Result<()> {
    if let Some(ollama_state) = &mut app.ollama_state {
        if let Some(model) = ollama_state.get_selected_model().cloned() {
            // Add user message to conversation immediately
            ollama_state.add_message(ChatRole::User, message.clone());

            // Add a temporary "thinking" message
            ollama_state.add_message(ChatRole::System, "Processing your request...".to_string());

            // Set sending state and clear input buffer
            ollama_state.is_sending = true;
            ollama_state.input_buffer.clear();

            // Create a channel for communication
            let (tx, rx) = mpsc::channel();

            // Clone necessary data for the thread
            let model_clone = model.clone();
            let snippet = ollama_state.current_snippet.clone().unwrap_or_default();

            // Spawn a thread to run the command
            thread::spawn(move || {
                let prompt = if !snippet.is_empty() {
                    format!(
                        "The following is a code snippet. Please help me understand or improve it:\n\n```\n{}\n```\n\n{}",
                        snippet, message
                    )
                } else {
                    message
                };

                let cmd = Command::new("ollama")
                    .arg("run")
                    .arg(model_clone)
                    .arg(prompt)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .map_err(|e| e.to_string());

                match cmd {
                    Ok(mut child) => {
                        let stdout = child.stdout.take().expect("Failed to capture stdout");
                        let reader = BufReader::new(stdout);
                        let mut response = String::new();

                        for line in reader.lines() {
                            match line {
                                Ok(line) => {
                                    response.push_str(&line);
                                    response.push('\n');
                                }
                                Err(e) => {
                                    let _ = tx.send(Err(e.to_string()));
                                    return;
                                }
                            }
                        }

                        let _ = tx.send(Ok(response));
                    }
                    Err(e) => {
                        let _ = tx.send(Err(e));
                    }
                }
            });

            // Wait for the result with a timeout
            match rx.recv_timeout(std::time::Duration::from_secs(120)) {
                Ok(result) => {
                    // Remove the temporary "thinking" message
                    if ollama_state.conversation.len() >= 2 {
                        if let Some(last_msg) = ollama_state.conversation.last() {
                            if last_msg.role == ChatRole::System
                                && last_msg.content == "Processing your request..."
                            {
                                ollama_state.conversation.pop();
                            }
                        }
                    }

                    match result {
                        Ok(response) => {
                            ollama_state.add_message(ChatRole::Assistant, response);
                        }
                        Err(e) => {
                            ollama_state.add_message(ChatRole::System, format!("Error: {}", e));
                        }
                    }
                }
                Err(_) => {
                    // Remove the temporary "thinking" message
                    if ollama_state.conversation.len() >= 2 {
                        if let Some(last_msg) = ollama_state.conversation.last() {
                            if last_msg.role == ChatRole::System
                                && last_msg.content == "Processing your request..."
                            {
                                ollama_state.conversation.pop();
                            }
                        }
                    }

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
                    // Scroll up in chat view
                    if ollama_state.scroll_position > 0 {
                        ollama_state.scroll_position -= 1;
                    }
                }
                false
            }

            KeyCode::Down | KeyCode::Char('j') => {
                if ollama_state.get_selected_model().is_none() && !ollama_state.models.is_empty() {
                    ollama_state.selected_model_index =
                        (ollama_state.selected_model_index + 1) % ollama_state.models.len();
                } else if ollama_state.get_selected_model().is_some() {
                    // Scroll down in chat view
                    ollama_state.scroll_position += 1;
                    // The max scroll is handled in the render function
                }
                false
            }

            // Page up/down for faster scrolling
            KeyCode::PageUp => {
                if ollama_state.get_selected_model().is_some() {
                    // Scroll up by 10 lines
                    ollama_state.scroll_position = ollama_state.scroll_position.saturating_sub(10);
                }
                false
            }

            KeyCode::PageDown => {
                if ollama_state.get_selected_model().is_some() {
                    // Scroll down by 10 lines
                    ollama_state.scroll_position += 10;
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
                    // Set to a large number, will be clamped in render
                    ollama_state.scroll_position = usize::MAX;
                }
                false
            }

            // Select model or send message
            KeyCode::Enter => {
                if ollama_state.get_selected_model().is_none() {
                    // We're in model selection mode
                    if !ollama_state.models.is_empty() {
                        // Initialize conversation with system message
                        ollama_state.conversation.clear();
                        ollama_state.add_message(
                            ChatRole::System,
                            "Chat started. You can now talk with the model.".to_string(),
                        );

                        // If we have a snippet, add it as context
                        if let Some(snippet) = &ollama_state.current_snippet {
                            ollama_state.add_message(
                                ChatRole::System,
                                format!("Current code snippet:\n\n```\n{}\n```", snippet),
                            );
                        }
                    }
                } else if !ollama_state.is_sending && !ollama_state.input_buffer.is_empty() {
                    // We're in chat mode, send the message
                    let message = ollama_state.input_buffer.clone();
                    let _ = send_message_to_ollama(app, message);
                }
                false
            }

            // Handle text input for chat
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

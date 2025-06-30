use crate::app::App;
use anyhow::{Result, anyhow};
use flume;
use ollama_rs::{Ollama, generation::completion::request::GenerationRequest, models::ModelOptions};
use once_cell::sync::Lazy;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use tokio::runtime::Runtime;

use crate::ui::ollama::{
    ActivePanel, ChatMessage, ChatRole, HistoryFilter, OllamaMessage, OllamaState,
};

const OLLAMA_HOST: &str = "http://localhost";
const OLLAMA_PORT: u16 = 11434;
const OLLAMA_TEMPERATURE: f32 = 0.7;
const OLLAMA_NUM_PREDICT: i32 = 2048;
const OLLAMA_TOP_K: u32 = 40;
const OLLAMA_TOP_P: f32 = 0.9;

const ERROR_CONNECTION_REFUSED: &str = "Cannot connect to Ollama. Please ensure Ollama is running:\n1. Install Ollama from https://ollama.ai\n2. Run 'ollama serve' in terminal\n3. Install a model: 'ollama pull llama2'";
const ERROR_NO_MODELS: &str = "No models found. Please install models using 'ollama pull <model_name>'. Example: 'ollama pull llama2'";

// Global static channel for communication using once_cell for thread safety
static OLLAMA_CHANNEL: Lazy<(flume::Sender<OllamaMessage>, flume::Receiver<OllamaMessage>)> =
    Lazy::new(|| flume::unbounded());

// Global tokio runtime for async operations
static GLOBAL_RUNTIME: Lazy<Runtime> =
    Lazy::new(|| Runtime::new().expect("Failed to create Tokio runtime"));

// Get sender for the global channel
pub fn get_ollama_sender() -> flume::Sender<OllamaMessage> {
    OLLAMA_CHANNEL.0.clone()
}

// Get receiver for the global channel
pub fn get_ollama_receiver() -> flume::Receiver<OllamaMessage> {
    OLLAMA_CHANNEL.1.clone()
}

// Helper functions for better code organization (huh!)

/// Creates a new Ollama client with default configuration
fn create_ollama_client() -> Ollama {
    Ollama::new(OLLAMA_HOST.to_string(), OLLAMA_PORT)
}

/// Creates default model options for Ollama requests
fn create_model_options() -> ModelOptions {
    ModelOptions::default()
        .temperature(OLLAMA_TEMPERATURE)
        .num_predict(OLLAMA_NUM_PREDICT)
        .top_k(OLLAMA_TOP_K)
        .top_p(OLLAMA_TOP_P)
}

/// Determines if an error is a connection-related error
fn is_connection_error(error: &str) -> bool {
    error.contains("Connection refused") || error.contains("No such host")
}

/// Creates enhanced system prompt for snippet analysis
pub fn create_snippet_system_prompt(
    snippet_language: &str,
    snippet_title: &str,
    snippet_content: &str,
) -> String {
    format!(
        "You are a helpful AI assistant specializing in code analysis and development. \
You are currently working with a {} code snippet titled '{}'. \
Here is the code snippet:
```{}
{}
```

Please provide helpful analysis, suggestions, explanations, or answer questions about this code. \
When discussing the code, be specific and reference particular parts when relevant.",
        snippet_language, snippet_title, snippet_language, snippet_content
    )
}

pub fn fetch_ollama_models(app: &mut App) -> Result<()> {
    if let Some(ollama_state) = &mut app.ollama_state {
        ollama_state.loading_models = true;
        ollama_state.error_message = None;
        ollama_state.models.clear();
        // Reset to safe index when clearing models
        ollama_state.selected_model_index = 0;

        let sender = get_ollama_sender();

        // Use the global runtime to spawn the async task
        GLOBAL_RUNTIME.spawn(async move {
            let result = async {
                let ollama = create_ollama_client();

                // Test connection first to avoid any issue (spoiler alert it did!)
                match ollama.list_local_models().await {
                    Ok(models_list) => {
                        let model_names: Vec<String> =
                            models_list.iter().map(|model| model.name.clone()).collect();

                        if model_names.is_empty() {
                            Err(anyhow!(ERROR_NO_MODELS))
                        } else {
                            Ok(model_names)
                        }
                    }
                    Err(e) => {
                        if is_connection_error(&e.to_string()) {
                            Err(anyhow!(ERROR_CONNECTION_REFUSED))
                        } else {
                            Err(anyhow!("Failed to load models: {}", e))
                        }
                    }
                }
            }
            .await;

            match result {
                Ok(model_names) => {
                    let _ = sender.send(OllamaMessage::ModelsLoaded {
                        models: model_names,
                    });
                }
                Err(e) => {
                    let _ = sender.send(OllamaMessage::Error {
                        request_id: 0,
                        message: e.to_string(),
                    });
                }
            }
        });

        Ok(())
    } else {
        Err(anyhow!("Ollama state not initialized"))
    }
}

pub async fn send_message_to_ollama(
    model: String,
    message: String,
    system_prompt: String,
    conversation_history: Vec<ChatMessage>,
    request_id: u64,
) -> Result<()> {
    let sender = get_ollama_sender();

    let ollama = create_ollama_client();

    // Build conversation context
    let mut full_prompt = if !system_prompt.trim().is_empty() {
        format!("{}\n\n", system_prompt)
    } else {
        String::new()
    };

    // Add conversation history
    for msg in &conversation_history {
        match msg.role {
            ChatRole::User => full_prompt.push_str(&format!("User: {}\n", msg.content)),
            ChatRole::Assistant => full_prompt.push_str(&format!("Assistant: {}\n", msg.content)),
            // System messages are already in the system prompt
            ChatRole::System => {}
        }
    }

    full_prompt.push_str(&format!("User: {}\nAssistant: ", message));

    let options = create_model_options();
    let request = GenerationRequest::new(model.clone(), full_prompt).options(options);

    match ollama.generate(request).await {
        Ok(res) => {
            let response = if res.response.trim().is_empty() {
                "I apologize, but I couldn't generate a response. Please try again.".to_string()
            } else {
                res.response
            };

            let chunk = OllamaMessage::ResponseChunk {
                request_id,
                content: response,
                done: true,
            };
            let _ = sender.send(chunk);
        }
        Err(e) => {
            let error_msg = if is_connection_error(&e.to_string()) {
                "Cannot connect to Ollama. Please ensure Ollama is running with 'ollama serve'"
                    .to_string()
            } else if e.to_string().contains("model") && e.to_string().contains("not found") {
                format!(
                    "Model '{}' not found. Please install it with 'ollama pull {}'",
                    model, model
                )
            } else {
                format!("Request failed: {}", e)
            };

            let error = OllamaMessage::Error {
                request_id,
                message: error_msg,
            };
            let _ = sender.send(error);
        }
    }

    Ok(())
}

pub fn update_loading_animation(app: &mut App) {
    process_ollama_messages(app);

    // Check if ollama_state exists before trying to access it
    if let Some(ollama_state) = &mut app.ollama_state {
        if ollama_state.is_sending || ollama_state.loading_models {
            // Update the animation frame safely
            ollama_state.loading_animation_frame =
                ollama_state.loading_animation_frame.wrapping_add(1);
        }

        ollama_state.clean_expired_toasts();
    }
}

pub fn process_ollama_messages(app: &mut App) {
    let receiver = &get_ollama_receiver();

    // Process all available messages without blocking
    while let Ok(message) = receiver.try_recv() {
        if let Some(ollama_state) = &mut app.ollama_state {
            match message {
                OllamaMessage::ModelsLoaded { models } => {
                    ollama_state.loading_models = false;
                    if models.is_empty() {
                        ollama_state.error_message = Some("❌ No models found!\n\nTo fix this:\n1. Install Ollama from https://ollama.ai\n2. Run 'ollama serve' in terminal\n3. Install a model: 'ollama pull llama2'\n4. Restart this application".to_string());
                    } else {
                        ollama_state.models = models;
                        // Auto-select first model if none selected
                        if ollama_state.selected_model_index >= ollama_state.models.len() {
                            ollama_state.selected_model_index = 0;
                        }
                        let selected_model = ollama_state
                            .get_selected_model()
                            .map(|s| s.clone())
                            .unwrap_or_else(|| "Unknown".to_string());
                        ollama_state.add_success_toast(format!(
                            "Loaded {} models! Selected: {}. Models visible in sidebar.",
                            ollama_state.models.len(),
                            selected_model
                        ));
                    }
                }
                OllamaMessage::ResponseChunk {
                    request_id,
                    content,
                    done,
                } => {
                    // Only process if this matches the current pending request
                    if ollama_state.pending_response_id == Some(request_id) {
                        if content.trim().is_empty() && !done {
                            continue;
                        }

                        // Add or update assistant response
                        if let Some(last_msg) = ollama_state.conversation.last_mut() {
                            if last_msg.role == ChatRole::Assistant && !done {
                                last_msg.content.push_str(&content);
                            } else if last_msg.role != ChatRole::Assistant || done {
                                ollama_state.conversation.push(ChatMessage {
                                    role: ChatRole::Assistant,
                                    content: content.clone(),
                                });
                            }
                        } else {
                            ollama_state.conversation.push(ChatMessage {
                                role: ChatRole::Assistant,
                                content: content.clone(),
                            });
                        }

                        // Update current session
                        if let Some(session) = &mut ollama_state.current_session {
                            if let Some(last_msg) = session.conversation.last_mut() {
                                if last_msg.role == ChatRole::Assistant {
                                    last_msg.content.push_str(&content);
                                } else {
                                    session.add_message(ChatRole::Assistant, content);
                                }
                            } else {
                                session.add_message(ChatRole::Assistant, content);
                            }
                        }

                        if done {
                            ollama_state.is_sending = false;
                            ollama_state.pending_response_id = None;
                            ollama_state.typing_indicator.clear();

                            // Auto-save if enabled
                            if ollama_state.auto_save_enabled {
                                if let Some(current_session) = &mut ollama_state.current_session {
                                    if let Some(storage) = &ollama_state.chat_storage {
                                        let _ = storage.save_session(current_session);
                                    }
                                } else if let Some(model) = ollama_state.get_selected_model() {
                                    // Create a new session if none exists
                                    let mut new_session = crate::ui::ollama::ChatSession::new(
                                        model.clone(),
                                        ollama_state.system_prompt.clone(),
                                    );

                                    // Associate with current snippet if available
                                    if let Some(snippet) = &ollama_state.current_snippet {
                                        new_session = new_session
                                            .with_snippet(snippet, "Current Snippet".to_string());
                                    }

                                    // Copy conversation
                                    new_session.conversation = ollama_state.conversation.clone();

                                    // Save the new session
                                    if let Some(storage) = &ollama_state.chat_storage {
                                        let _ = storage.save_session(&new_session);
                                    }

                                    ollama_state.current_session = Some(new_session);
                                }
                            }
                        }

                        // Auto-scroll to bottom
                        ollama_state.scroll_position = usize::MAX;
                    }
                }
                OllamaMessage::Error {
                    request_id,
                    message,
                } => {
                    // Process errors for model loading (request_id = 0) or current request
                    if ollama_state.pending_response_id == Some(request_id) || request_id == 0 {
                        if request_id != 0 {
                            // Only add error message to chat for actual chat requests
                            ollama_state.conversation.push(ChatMessage {
                                role: ChatRole::System,
                                content: format!("❌ Error: {}", message),
                            });
                        }
                        ollama_state.is_sending = false;
                        ollama_state.pending_response_id = None;
                        ollama_state.typing_indicator.clear();
                        ollama_state.error_message = Some(message);

                        // Auto-scroll to show error
                        ollama_state.scroll_position = usize::MAX;
                    }
                }
            }
        }
    }
}

pub fn handle_ollama_input(app: &mut App, key: KeyEvent) -> Result<()> {
    if let Some(ollama_state) = &mut app.ollama_state {
        match key.code {
            KeyCode::Esc => {
                if ollama_state.editing_system_prompt {
                    // Cancel editing system prompt
                    ollama_state.editing_system_prompt = false;
                    ollama_state.system_prompt_buffer.clear();
                } else {
                    // Close Ollama interface
                    app.ollama_state = None;
                }
            }
            KeyCode::Tab => {
                // Cycle through main panels
                ollama_state.active_panel = match ollama_state.active_panel {
                    ActivePanel::CurrentChat => ActivePanel::ChatHistory,
                    ActivePanel::ChatHistory => ActivePanel::Settings,
                    ActivePanel::Settings => ActivePanel::CurrentChat,
                };
            }
            KeyCode::Enter => {
                match ollama_state.active_panel {
                    ActivePanel::CurrentChat => {
                        if ollama_state.get_selected_model().is_some() {
                            send_chat_message(ollama_state)?;
                        }
                    }
                    ActivePanel::ChatHistory => {
                        load_selected_session(ollama_state)?;
                    }
                    ActivePanel::Settings => {
                        // Edit system prompt
                        if !ollama_state.editing_system_prompt {
                            ollama_state.editing_system_prompt = true;
                            ollama_state.system_prompt_buffer = ollama_state.system_prompt.clone();
                        } else {
                            // Save system prompt
                            ollama_state.system_prompt = ollama_state.system_prompt_buffer.clone();
                            ollama_state.editing_system_prompt = false;
                            ollama_state.system_prompt_buffer.clear();
                        }
                    }
                }
            }
            KeyCode::Up => {
                match ollama_state.active_panel {
                    ActivePanel::CurrentChat => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            // Navigate models up (Ctrl+Up)
                            if !ollama_state.models.is_empty()
                                && ollama_state.selected_model_index > 0
                            {
                                ollama_state.selected_model_index -= 1;
                                // Additional safety check to ensure index is still valid
                                if ollama_state.selected_model_index < ollama_state.models.len() {
                                    if let Some(model) = ollama_state.get_selected_model() {
                                        ollama_state
                                            .add_info_toast(format!("Selected model: {}", model));
                                    }
                                } else {
                                    // Reset to safe index - only reset if models exist
                                    if !ollama_state.models.is_empty() {
                                        ollama_state.selected_model_index = 0;
                                    }
                                }
                            } else if ollama_state.models.is_empty() {
                                ollama_state.add_error_toast(
                                    "No models available. Please install models first.".to_string(),
                                );
                            }
                        } else {
                            scroll_chat_up(ollama_state);
                        }
                    }
                    ActivePanel::ChatHistory => {
                        // Navigate sessions up
                        let filtered_sessions = ollama_state.get_filtered_sessions();
                        if !filtered_sessions.is_empty() && ollama_state.selected_session_index > 0
                        {
                            ollama_state.selected_session_index -= 1;
                        } else if filtered_sessions.is_empty() {
                            ollama_state.add_info_toast("No chat sessions available.".to_string());
                        }
                    }
                    ActivePanel::Settings => {
                        // No action for settings
                    }
                }
            }
            KeyCode::Down => {
                // Safety check: ensure we're in the expected panel for this operation
                match ollama_state.active_panel {
                    ActivePanel::CurrentChat => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            // Navigate models down (Ctrl+Down)
                            if !ollama_state.models.is_empty()
                                && ollama_state.selected_model_index
                                    < ollama_state.models.len().saturating_sub(1)
                            {
                                ollama_state.selected_model_index += 1;
                                // Additional safety check to ensure index is still valid
                                if ollama_state.selected_model_index < ollama_state.models.len() {
                                    if let Some(model) = ollama_state.get_selected_model() {
                                        ollama_state
                                            .add_info_toast(format!("Selected model: {}", model));
                                    }
                                } else {
                                    // Reset to safe index - only if models exist
                                    if !ollama_state.models.is_empty() {
                                        ollama_state.selected_model_index =
                                            ollama_state.models.len() - 1;
                                    } else {
                                        ollama_state.selected_model_index = 0;
                                    }
                                }
                            } else if ollama_state.models.is_empty() {
                                ollama_state.add_error_toast(
                                    "No models available. Please install models first.".to_string(),
                                );
                            }
                        } else {
                            scroll_chat_down(ollama_state);
                        }
                    }
                    ActivePanel::ChatHistory => {
                        // Navigate sessions down
                        let filtered_sessions = ollama_state.get_filtered_sessions();
                        if !filtered_sessions.is_empty()
                            && ollama_state.selected_session_index
                                < filtered_sessions.len().saturating_sub(1)
                        {
                            ollama_state.selected_session_index += 1;
                        } else if filtered_sessions.is_empty() {
                            ollama_state.add_info_toast("No chat sessions available.".to_string());
                        }
                    }
                    ActivePanel::Settings => {
                        // No action for settings
                    }
                }
            }
            KeyCode::Char('m') | KeyCode::Char('M') => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    // Show current model and model list
                    if !ollama_state.models.is_empty() {
                        ollama_state.add_info_toast(format!(
                            "Models ({}): {}",
                            ollama_state.models.len(),
                            ollama_state.models.join(", ")
                        ));
                    } else {
                        ollama_state.add_error_toast("No models available! Ensure Ollama is running and models are installed.".to_string());
                    }
                } else if ollama_state.active_panel == ActivePanel::ChatHistory {
                    ollama_state.history_filter = match ollama_state.history_filter {
                        HistoryFilter::All => HistoryFilter::Recent,
                        HistoryFilter::Recent => HistoryFilter::Favorites,
                        HistoryFilter::Favorites => HistoryFilter::CurrentSnippet,
                        HistoryFilter::CurrentSnippet => HistoryFilter::Search,
                        HistoryFilter::Search => HistoryFilter::All,
                    };
                    ollama_state.selected_session_index = 0;
                }
            }
            KeyCode::Char('/') => {
                if ollama_state.active_panel == ActivePanel::ChatHistory {
                    // Enter search mode
                    ollama_state.history_filter = HistoryFilter::Search;
                    ollama_state.search_query.clear();
                }
            }
            KeyCode::Char('f') | KeyCode::Char('F') => {
                if ollama_state.active_panel == ActivePanel::ChatHistory {
                    // Toggle favorite for selected session
                    toggle_session_favorite(ollama_state)?;
                }
            }
            KeyCode::Delete => {
                if ollama_state.active_panel == ActivePanel::ChatHistory {
                    // Delete selected session
                    delete_selected_session(ollama_state)?;
                }
            }
            KeyCode::Char(' ') => {
                if ollama_state.active_panel == ActivePanel::Settings {
                    // Toggle auto-save
                    ollama_state.auto_save_enabled = !ollama_state.auto_save_enabled;
                }
            }
            KeyCode::PageUp => {
                if ollama_state.active_panel == ActivePanel::CurrentChat {
                    // Fast scroll up
                    fast_scroll_chat_up(ollama_state);
                }
            }
            KeyCode::PageDown => {
                if ollama_state.active_panel == ActivePanel::CurrentChat {
                    // Fast scroll down
                    fast_scroll_chat_down(ollama_state);
                }
            }
            KeyCode::Home => {
                if ollama_state.active_panel == ActivePanel::CurrentChat {
                    // Jump to top
                    ollama_state.scroll_position = 0;
                }
            }
            KeyCode::End => {
                if ollama_state.active_panel == ActivePanel::CurrentChat {
                    // Jump to bottom
                    // Will be clamped in render
                    ollama_state.scroll_position = usize::MAX;
                }
            }
            KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                clear_conversation(ollama_state)?;
            }
            KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                create_new_session(ollama_state)?;
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                save_current_session(ollama_state)?;
            }
            KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if ollama_state.active_panel == ActivePanel::CurrentChat {
                    ollama_state.models.clear();
                    ollama_state.selected_model_index = 0;
                    ollama_state.loading_models = true;
                    ollama_state.error_message = Some("🔄 Refreshing models...".to_string());

                    // Trigger model refresh by directly using the global runtime
                    let sender = get_ollama_sender();
                    GLOBAL_RUNTIME.spawn(async move {
                        let result = async {
                            let ollama = create_ollama_client();

                            match ollama.list_local_models().await {
                                Ok(models_list) => {
                                    let model_names: Vec<String> = models_list
                                        .iter()
                                        .map(|model| model.name.clone())
                                        .collect();

                                    if model_names.is_empty() {
                                        Err(anyhow!(ERROR_NO_MODELS))
                                    } else {
                                        Ok(model_names)
                                    }
                                }
                                Err(e) => {
                                    if is_connection_error(&e.to_string()) {
                                        Err(anyhow!(ERROR_CONNECTION_REFUSED))
                                    } else {
                                        Err(anyhow!("Failed to load models: {}", e))
                                    }
                                }
                            }
                        }
                        .await;

                        match result {
                            Ok(model_names) => {
                                let _ = sender.send(OllamaMessage::ModelsLoaded {
                                    models: model_names,
                                });
                            }
                            Err(e) => {
                                let _ = sender.send(OllamaMessage::Error {
                                    request_id: 0,
                                    message: e.to_string(),
                                });
                            }
                        }
                    });
                } else {
                    refresh_sessions(ollama_state)?;
                }
            }
            KeyCode::Char(c) => {
                if ollama_state.editing_system_prompt {
                    // Edit system prompt
                    ollama_state.system_prompt_buffer.push(c);
                } else if ollama_state.history_filter == HistoryFilter::Search {
                    // Search input
                    ollama_state.search_query.push(c);
                } else if ollama_state.active_panel == ActivePanel::CurrentChat
                    && !ollama_state.is_sending
                {
                    // Chat input
                    ollama_state.input_buffer.push(c);
                }
            }
            KeyCode::Backspace => {
                if ollama_state.editing_system_prompt {
                    // Edit system prompt
                    ollama_state.system_prompt_buffer.pop();
                } else if ollama_state.history_filter == HistoryFilter::Search {
                    // Search input
                    ollama_state.search_query.pop();
                } else if ollama_state.active_panel == ActivePanel::CurrentChat
                    && !ollama_state.is_sending
                {
                    // Chat input
                    ollama_state.input_buffer.pop();
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn send_chat_message(ollama_state: &mut OllamaState) -> Result<()> {
    if ollama_state.input_buffer.trim().is_empty() || ollama_state.is_sending {
        return Ok(());
    }

    // Check if a model is selected
    let model = match ollama_state.get_selected_model() {
        Some(model) => model.clone(),
        None => {
            ollama_state.error_message =
                Some("No model selected. Please select a model first.".to_string());
            return Ok(());
        }
    };

    let message = ollama_state.input_buffer.trim().to_string();
    let system_prompt = ollama_state.system_prompt.clone();
    let conversation_history = ollama_state.conversation.clone();

    // Generate unique request ID - simplified to avoid potential issues
    let request_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);

    // Add user message to conversation
    ollama_state.conversation.push(ChatMessage {
        role: ChatRole::User,
        content: message.clone(),
    });

    // Clear input and set sending state
    ollama_state.input_buffer.clear();
    ollama_state.is_sending = true;
    ollama_state.pending_response_id = Some(request_id);
    ollama_state.typing_indicator = "Assistant is thinking...".to_string();

    // Update current session or create one if it doesn't exist
    if let Some(session) = &mut ollama_state.current_session {
        session.add_message(ChatRole::User, message.clone());
    } else {
        // Create a new session if none exists
        let model_name = model.clone();
        let mut new_session =
            crate::ui::ollama::ChatSession::new(model_name, system_prompt.clone());

        // Associate with current snippet if available
        if let Some(snippet_content) = &ollama_state.current_snippet {
            new_session = new_session.with_snippet(snippet_content, "Current Snippet".to_string());
        }

        new_session.add_message(ChatRole::User, message.clone());
        ollama_state.current_session = Some(new_session);
    }

    // Use the global runtime to spawn the async task instead of tokio::spawn
    GLOBAL_RUNTIME.spawn(async move {
        let _ = send_message_to_ollama(
            model,
            message,
            system_prompt,
            conversation_history,
            request_id,
        )
        .await;
    });

    Ok(())
}

fn load_selected_session(ollama_state: &mut OllamaState) -> Result<()> {
    let filtered_sessions = ollama_state.get_filtered_sessions();
    if ollama_state.selected_session_index < filtered_sessions.len() {
        if let Some(selected_session) = filtered_sessions.get(ollama_state.selected_session_index) {
            let selected_session = (*selected_session).clone();

            // Load conversation
            ollama_state.conversation = selected_session.conversation.clone();
            ollama_state.current_session = Some(selected_session.clone());
            ollama_state.system_prompt = selected_session.system_prompt.clone();

            // Switch to chat panel
            ollama_state.active_panel = ActivePanel::CurrentChat;

            // Scroll to bottom
            ollama_state.scroll_position = usize::MAX;
        }
    }
    Ok(())
}

fn toggle_session_favorite(ollama_state: &mut OllamaState) -> Result<()> {
    let filtered_sessions = ollama_state.get_filtered_sessions();
    if let Some(selected_session) = filtered_sessions.get(ollama_state.selected_session_index) {
        let session_id = selected_session.id;

        // Find and toggle in main sessions list
        if let Some(session) = ollama_state
            .saved_sessions
            .iter_mut()
            .find(|s| s.id == session_id)
        {
            session.is_favorited = !session.is_favorited;

            // Save to storage if available
            if let Some(storage) = &ollama_state.chat_storage {
                let _ = storage.save_session(session);
            }
        }
    }
    Ok(())
}

fn delete_selected_session(ollama_state: &mut OllamaState) -> Result<()> {
    let filtered_sessions = ollama_state.get_filtered_sessions();
    if let Some(selected_session) = filtered_sessions.get(ollama_state.selected_session_index) {
        let session_id = selected_session.id;

        // Remove from main sessions list
        ollama_state.saved_sessions.retain(|s| s.id != session_id);

        // Delete from storage if available
        if let Some(storage) = &ollama_state.chat_storage {
            let _ = storage.delete_session(session_id);
        }

        // Adjust selection index safely
        let filtered_count = ollama_state.get_filtered_sessions().len();
        if filtered_count > 0 && ollama_state.selected_session_index >= filtered_count {
            ollama_state.selected_session_index = filtered_count.saturating_sub(1);
        } else if filtered_count == 0 {
            ollama_state.selected_session_index = 0;
        }
    }
    Ok(())
}

fn clear_conversation(ollama_state: &mut OllamaState) -> Result<()> {
    ollama_state.conversation.clear();
    ollama_state.scroll_position = 0;

    if let Some(session) = &mut ollama_state.current_session {
        session.conversation.clear();
    }

    Ok(())
}

fn create_new_session(ollama_state: &mut OllamaState) -> Result<()> {
    let model_name = ollama_state
        .get_selected_model()
        .map(|m| m.clone())
        .unwrap_or_else(|| "unknown".to_string());

    // Create session with current system prompt (which should already include snippet context)
    let mut new_session =
        crate::ui::ollama::ChatSession::new(model_name, ollama_state.system_prompt.clone());

    // Associate with current snippet if available
    if let Some(snippet) = &ollama_state.current_snippet {
        new_session = new_session.with_snippet(snippet, "Current Snippet".to_string());
    }

    // Clear current conversation and set new session
    ollama_state.conversation.clear();
    ollama_state.current_session = Some(new_session);
    ollama_state.scroll_position = 0;

    // Switch to chat panel & Clear any error messages and show success
    ollama_state.active_panel = ActivePanel::CurrentChat;
    ollama_state.error_message = None;

    Ok(())
}

fn save_current_session(ollama_state: &mut OllamaState) -> Result<()> {
    if let Some(session) = &ollama_state.current_session {
        if let Some(storage) = &ollama_state.chat_storage {
            storage.save_session(session)?;

            // Update saved sessions list
            if let Some(existing) = ollama_state
                .saved_sessions
                .iter_mut()
                .find(|s| s.id == session.id)
            {
                *existing = session.clone();
            } else {
                ollama_state.saved_sessions.push(session.clone());
            }
        }
    }
    Ok(())
}

fn refresh_sessions(ollama_state: &mut OllamaState) -> Result<()> {
    if let Some(storage) = &ollama_state.chat_storage {
        ollama_state.saved_sessions = storage.load_all_sessions()?;
    }
    Ok(())
}

/// Safely scroll chat up with bounds checking
fn scroll_chat_up(ollama_state: &mut OllamaState) {
    ollama_state.scroll_position = ollama_state
        .scroll_position
        .saturating_sub(ollama_state.scroll_speed);
}

/// Safely scroll chat down with overflow protection
fn scroll_chat_down(ollama_state: &mut OllamaState) {
    // Calculate maximum scroll position based on content
    let max_scroll = calculate_max_scroll_position(ollama_state);

    // Only scroll if we haven't reached the bottom
    if ollama_state.scroll_position < max_scroll {
        ollama_state.scroll_position =
            (ollama_state.scroll_position + ollama_state.scroll_speed).min(max_scroll);
    }
}

/// Safely fast scroll chat up
fn fast_scroll_chat_up(ollama_state: &mut OllamaState) {
    ollama_state.scroll_position = ollama_state
        .scroll_position
        .saturating_sub(ollama_state.scroll_speed * 10);
}

/// Safely fast scroll chat down with overflow protection
fn fast_scroll_chat_down(ollama_state: &mut OllamaState) {
    // Calculate maximum scroll position based on content
    let max_scroll = calculate_max_scroll_position(ollama_state);

    // Only scroll if we haven't reached the bottom
    if ollama_state.scroll_position < max_scroll {
        ollama_state.scroll_position =
            (ollama_state.scroll_position + (ollama_state.scroll_speed * 10)).min(max_scroll);
    }
}

/// Calculate maximum scroll position based on chat content
fn calculate_max_scroll_position(ollama_state: &OllamaState) -> usize {
    // Default chat area height (this will be overridden by UI rendering bounds)
    let default_visible_height = 20; // Conservative estimate

    // Calculate total height of all messages
    let mut total_height = 0;

    for msg in &ollama_state.conversation {
        // Estimate message height (content height + borders + spacing)
        let content_width = 80; // Conservative estimate for content width
        let wrapped_height = calculate_estimated_wrapped_height(&msg.content, content_width);
        let message_height = wrapped_height + 2;
        total_height += message_height + 1;
    }

    // Add space for typing indicator if active
    if ollama_state.is_sending {
        total_height += 4;
    }

    // Return maximum scroll position (total height - visible height)
    total_height.saturating_sub(default_visible_height)
}

/// Estimate wrapped height for text content
fn calculate_estimated_wrapped_height(text: &str, width: usize) -> usize {
    if width == 0 {
        return 1;
    }

    let mut total_lines = 0;

    for line in text.split('\n') {
        if line.is_empty() {
            total_lines += 1;
        } else {
            let line_chars = line.chars().count();
            // Ceiling division
            let wrapped_lines = (line_chars + width - 1) / width;
            total_lines += wrapped_lines.max(1);
        }
    }

    total_lines.max(1)
}

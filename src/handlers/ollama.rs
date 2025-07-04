use crate::app::App;
use anyhow::{Result, anyhow};
use flume;
use ollama_rs::Ollama;
use once_cell::sync::Lazy;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use reqwest;
use serde_json;

use tokio::runtime::Runtime;

use crate::ui::ollama::{
    ActivePanel, ChatMessage, ChatRole, HistoryFilter, MessageMetrics, OllamaMessage, OllamaState,
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

/// Creates a new Ollama client with default configuration
fn create_ollama_client() -> Ollama {
    Ollama::new(OLLAMA_HOST.to_string(), OLLAMA_PORT)
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

    // Use direct HTTP streaming for real-time responses
    let client = reqwest::Client::new();

    let request_body = serde_json::json!({
        "model": model,
        "prompt": full_prompt,
        "stream": true,
        "options": {
            "temperature": OLLAMA_TEMPERATURE,
            "num_predict": OLLAMA_NUM_PREDICT,
            "top_k": OLLAMA_TOP_K,
            "top_p": OLLAMA_TOP_P,
        }
    });

    match client
        .post(&format!(
            "{}:{}{}",
            OLLAMA_HOST, OLLAMA_PORT, "/api/generate"
        ))
        .json(&request_body)
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                use futures::stream::StreamExt;

                let mut stream = response.bytes_stream();
                let mut buffer = String::new();

                while let Some(chunk_result) = stream.next().await {
                    match chunk_result {
                        Ok(chunk) => {
                            if let Ok(text) = std::str::from_utf8(&chunk) {
                                // Each line should be a JSON object
                                for line in text.lines() {
                                    if line.trim().is_empty() {
                                        continue;
                                    }

                                    if let Ok(json_response) =
                                        serde_json::from_str::<serde_json::Value>(line)
                                    {
                                        if let Some(response_text) =
                                            json_response.get("response").and_then(|r| r.as_str())
                                        {
                                            let is_done = json_response
                                                .get("done")
                                                .and_then(|d| d.as_bool())
                                                .unwrap_or(false);

                                            // Send each chunk immediately for real-time display
                                            let chunk = OllamaMessage::ResponseChunk {
                                                request_id,
                                                content: response_text.to_string(),
                                                done: is_done,
                                            };
                                            let _ = sender.send(chunk);

                                            buffer.push_str(response_text);

                                            if is_done {
                                                return Ok(());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            let error_msg = format!("Streaming error: {}", e);
                            let error = OllamaMessage::Error {
                                request_id,
                                message: error_msg,
                            };
                            let _ = sender.send(error);
                            return Err(e.into());
                        }
                    }
                }

                // If we reach here without a done=true response, send final marker
                let final_chunk = OllamaMessage::ResponseChunk {
                    request_id,
                    content: String::new(),
                    done: true,
                };
                let _ = sender.send(final_chunk);
            } else {
                let error_msg = format!("HTTP error: {}", response.status());
                let error = OllamaMessage::Error {
                    request_id,
                    message: error_msg,
                };
                let _ = sender.send(error);
            }
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
        ollama_state.update_copy_button_feedback();
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
                        ollama_state.error_message = Some("󰅙 No models found!\n\nTo fix this:\n1. Install Ollama from https://ollama.ai\n2. Run 'ollama serve' in terminal\n3. Install a model: 'ollama pull llama2'\n4. Restart this application".to_string());
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
                        // Skip empty content chunks unless it's the final marker
                        if content.trim().is_empty() && !done {
                            continue;
                        }

                        // Handle streaming chunks
                        if !content.is_empty() {
                            // Track performance metrics
                            ollama_state.add_response_chunk(&content);
                            ollama_state.typing_indicator = format!(
                                " 󱍢 Receiving... ({} tokens)",
                                ollama_state.current_message_token_count
                            );

                            // Update or create assistant message in conversation
                            if let Some(last_msg) = ollama_state.conversation.last_mut() {
                                if last_msg.role == ChatRole::Assistant {
                                    last_msg.content.push_str(&content);
                                } else {
                                    // Create new assistant message with default metrics (will be updated when done)
                                    ollama_state.conversation.push(ChatMessage {
                                        role: ChatRole::Assistant,
                                        content: content.clone(),
                                        metrics: MessageMetrics::default(),
                                        context_length: 0,
                                    });
                                }
                            } else {
                                // First message in conversation
                                ollama_state.conversation.push(ChatMessage {
                                    role: ChatRole::Assistant,
                                    content: content.clone(),
                                    metrics: MessageMetrics::default(),
                                    context_length: 0,
                                });
                            }

                            // Update current session
                            if let Some(session) = &mut ollama_state.current_session {
                                if let Some(last_msg) = session.conversation.last_mut() {
                                    if last_msg.role == ChatRole::Assistant {
                                        last_msg.content.push_str(&content);
                                    } else {
                                        session.add_message(ChatRole::Assistant, content.clone());
                                    }
                                } else {
                                    session.add_message(ChatRole::Assistant, content);
                                }
                            }
                        }

                        if done {
                            // Finalize performance metrics
                            let metrics = ollama_state.finish_message_timing();
                            let context_length = ollama_state.conversation.len() as u32;

                            // Update the last assistant message with final metrics and track for copying
                            if let Some(last_msg) = ollama_state.conversation.last_mut() {
                                if last_msg.role == ChatRole::Assistant {
                                    last_msg.metrics = metrics.clone();
                                    last_msg.context_length = context_length;
                                    // Track the complete response for copy functionality
                                    ollama_state.last_assistant_response =
                                        Some(last_msg.content.clone());
                                }
                            }

                            // Update current session with metrics
                            if let Some(session) = &mut ollama_state.current_session {
                                if let Some(last_msg) = session.conversation.last_mut() {
                                    if last_msg.role == ChatRole::Assistant {
                                        last_msg.metrics = metrics.clone();
                                        last_msg.context_length = context_length;
                                    }
                                }
                                // Update session stats
                                session.total_context_tokens = context_length;
                            }

                            ollama_state.is_sending = false;
                            ollama_state.pending_response_id = None;
                            ollama_state.typing_indicator.clear();
                            ollama_state.mark_unsaved_changes();

                            // Auto-save if enabled and we have content
                            if ollama_state.auto_save_enabled {
                                if let Some(current_session) = &mut ollama_state.current_session {
                                    if ollama_state.chat_storage.is_some() {
                                        let save_result = {
                                            let storage =
                                                ollama_state.chat_storage.as_ref().unwrap();
                                            storage.save_session(current_session)
                                        };

                                        match save_result {
                                            Ok(_) => {
                                                ollama_state
                                                    .add_success_toast("Chat saved! 󰭻".to_string());
                                                // Refresh the sessions list to show the updated session
                                                if let Some(storage) = &ollama_state.chat_storage {
                                                    if let Ok(sessions) =
                                                        storage.load_all_sessions()
                                                    {
                                                        ollama_state.saved_sessions = sessions;
                                                    }
                                                }
                                            }
                                            Err(_) => {
                                                ollama_state.add_error_toast(
                                                    "Failed to save chat".to_string(),
                                                );
                                            }
                                        }
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
                                    if ollama_state.chat_storage.is_some() {
                                        let save_result = {
                                            let storage =
                                                ollama_state.chat_storage.as_ref().unwrap();
                                            storage.save_session(&new_session)
                                        };

                                        match save_result {
                                            Ok(_) => {
                                                ollama_state.add_success_toast(
                                                    "New chat saved! 󰭻".to_string(),
                                                );
                                                // Refresh the sessions list to show the new session
                                                if let Some(storage) = &ollama_state.chat_storage {
                                                    if let Ok(sessions) =
                                                        storage.load_all_sessions()
                                                    {
                                                        ollama_state.saved_sessions = sessions;
                                                    }
                                                }
                                            }
                                            Err(_) => {
                                                ollama_state.add_error_toast(
                                                    "Failed to save new chat".to_string(),
                                                );
                                            }
                                        }
                                    }

                                    ollama_state.current_session = Some(new_session);
                                }
                            }
                        }

                        // Auto-scroll to bottom on any content update
                        ollama_state.scroll_to_bottom();
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
                                content: format!("󰅙 Error: {}", message),
                                metrics: MessageMetrics::default(),
                                context_length: 0,
                            });
                        }
                        ollama_state.is_sending = false;
                        ollama_state.pending_response_id = None;
                        ollama_state.typing_indicator.clear();
                        ollama_state.error_message = Some(message);
                        ollama_state.scroll_to_bottom();
                    }
                }
            }
        }
    }
}

pub fn handle_ollama_input(app: &mut App, key: KeyEvent) -> Result<()> {
    if let Some(ollama_state) = &mut app.ollama_state {
        // Handle save prompt first if it's shqwing
        if ollama_state.show_save_prompt {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    if let Err(_) = save_current_session(ollama_state) {
                        ollama_state.add_error_toast("Failed to save session".to_string());
                    } else {
                        ollama_state.add_success_toast("Session saved successfully!".to_string());
                        ollama_state.unsaved_changes = false;
                    }
                    ollama_state.show_save_prompt = false;
                    // Hide Ollama interface but preserve state
                    ollama_state.show_popup = false;
                    return Ok(());
                }
                KeyCode::Char('n') | KeyCode::Char('N') => {
                    ollama_state.show_save_prompt = false;
                    // Hide without saving but preserve state
                    ollama_state.show_popup = false;
                    return Ok(());
                }
                KeyCode::Esc => {
                    ollama_state.show_save_prompt = false;
                    return Ok(());
                }
                // Ignore other keys when save prompt is showing
                _ => return Ok(()),
            }
        }

        match key.code {
            KeyCode::Esc => {
                if ollama_state.editing_system_prompt {
                    // Cancel editing system prompt
                    ollama_state.editing_system_prompt = false;
                    ollama_state.system_prompt_buffer.clear();
                } else {
                    // Check for unsaved changes before exiting
                    let has_actual_unsaved_changes = ollama_state.has_unsaved_session();
                    let has_unsaved_conversation = !ollama_state.conversation.is_empty()
                        && ollama_state.current_session.is_none()
                        && ollama_state
                            .conversation
                            .iter()
                            .any(|msg| msg.role == ChatRole::User);

                    if has_actual_unsaved_changes || has_unsaved_conversation {
                        ollama_state.show_save_prompt = true;
                    } else {
                        // Hide Ollama interface but preserve state for associated chats
                        ollama_state.show_popup = false;
                    }
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
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    // Navigate models up (Ctrl+Up) - works from any panel
                    if !ollama_state.models.is_empty() && ollama_state.selected_model_index > 0 {
                        ollama_state.selected_model_index -= 1;
                        // Additional safety check to ensure index is still valid
                        if ollama_state.selected_model_index < ollama_state.models.len() {
                            if let Some(model) = ollama_state.get_selected_model() {
                                ollama_state.add_info_toast(format!("Selected model: {}", model));
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
                    match ollama_state.active_panel {
                        ActivePanel::CurrentChat => {
                            scroll_chat_up(ollama_state);
                        }
                        ActivePanel::ChatHistory => {
                            // Navigate sessions up
                            let filtered_sessions = ollama_state.get_filtered_sessions();
                            if !filtered_sessions.is_empty()
                                && ollama_state.selected_session_index > 0
                            {
                                ollama_state.selected_session_index -= 1;
                            } else if filtered_sessions.is_empty() {
                                ollama_state
                                    .add_info_toast("No chat sessions available.".to_string());
                            }
                        }
                        ActivePanel::Settings => {
                            // Allow scrolling in chat even when in settings panel
                            scroll_chat_up(ollama_state);
                        }
                    }
                }
            }
            KeyCode::Down => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    // Navigate models down (Ctrl+Down) - works from any panel
                    if !ollama_state.models.is_empty()
                        && ollama_state.selected_model_index
                            < ollama_state.models.len().saturating_sub(1)
                    {
                        ollama_state.selected_model_index += 1;
                        // Additional safety check to ensure index is still valid
                        if ollama_state.selected_model_index < ollama_state.models.len() {
                            if let Some(model) = ollama_state.get_selected_model() {
                                ollama_state.add_info_toast(format!("Selected model: {}", model));
                            }
                        } else {
                            // Reset to safe index - only if models exist
                            if !ollama_state.models.is_empty() {
                                ollama_state.selected_model_index = ollama_state.models.len() - 1;
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
                    match ollama_state.active_panel {
                        ActivePanel::CurrentChat => {
                            scroll_chat_down(ollama_state);
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
                                ollama_state
                                    .add_info_toast("No chat sessions available.".to_string());
                            }
                        }
                        ActivePanel::Settings => {
                            scroll_chat_down(ollama_state);
                        }
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
                }
            }
            KeyCode::Left => {
                if ollama_state.active_panel == ActivePanel::ChatHistory {
                    // Navigate filters left
                    ollama_state.history_filter = match ollama_state.history_filter {
                        HistoryFilter::All => HistoryFilter::CurrentSnippet,
                        HistoryFilter::Recent => HistoryFilter::All,
                        HistoryFilter::Favorites => HistoryFilter::Recent,
                        HistoryFilter::CurrentSnippet => HistoryFilter::Favorites,
                    };
                    ollama_state.selected_session_index = 0;
                }
            }
            KeyCode::Right => {
                if ollama_state.active_panel == ActivePanel::ChatHistory {
                    // Navigate filters right
                    ollama_state.history_filter = match ollama_state.history_filter {
                        HistoryFilter::All => HistoryFilter::Recent,
                        HistoryFilter::Recent => HistoryFilter::Favorites,
                        HistoryFilter::Favorites => HistoryFilter::CurrentSnippet,
                        HistoryFilter::CurrentSnippet => HistoryFilter::All,
                    };
                    ollama_state.selected_session_index = 0;
                }
            }
            KeyCode::Char('f') | KeyCode::Char('F') => {
                if ollama_state.active_panel == ActivePanel::ChatHistory {
                    // Toggle favorite for selected session
                    toggle_session_favorite(ollama_state)?;
                }
            }
            KeyCode::Char('N') => {
                if ollama_state.active_panel == ActivePanel::ChatHistory {
                    // Create new session from chat history panel
                    if let Err(e) = ollama_state.create_new_session() {
                        ollama_state
                            .add_error_toast(format!("Failed to create new session: {}", e));
                    } else {
                        ollama_state.add_success_toast("New session created! ".to_string());
                    }
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
                // Fast scroll up - works from any panel to improve UX
                fast_scroll_chat_up(ollama_state);
            }
            KeyCode::PageDown => {
                // Fast scroll down - works from any panel to improve UX
                fast_scroll_chat_down(ollama_state);
            }
            KeyCode::Home => {
                // Scroll to top of chat - works from any panel
                ollama_state.scroll_position = 0;
            }
            KeyCode::End => {
                // Scroll to bottom of chat - works from any panel
                ollama_state.scroll_to_bottom();
            }
            KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Err(e) = clear_conversation(ollama_state) {
                    ollama_state.add_error_toast(format!("Failed to clear conversation: {}", e));
                } else {
                    ollama_state.add_success_toast("Conversation cleared! 󰚃".to_string());
                }
            }
            KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Err(e) = ollama_state.create_new_session() {
                    ollama_state.add_error_toast(format!("Failed to create new session: {}", e));
                } else {
                    ollama_state.add_success_toast("New session created! ".to_string());
                }
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Err(e) = save_current_session(ollama_state) {
                    ollama_state.add_error_toast(format!("Failed to save session: {}", e));
                } else {
                    ollama_state.add_success_toast("Session saved! 󰭻".to_string());
                    // Refresh sessions list to show the updated session
                    if let Ok(_) = refresh_sessions(ollama_state) {
                        // Session list refreshed successfully
                    }
                }
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Copy last assistant response to clipboard
                if ollama_state.copy_last_response() {
                    ollama_state.add_success_toast("Response copied to clipboard! ".to_string());
                } else if ollama_state.last_assistant_response.is_none() {
                    ollama_state.add_info_toast("No response to copy yet".to_string());
                } else {
                    ollama_state.add_error_toast(
                        "Failed to copy to clipboard (clipboard tools required)".to_string(),
                    );
                }
            }
            KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if ollama_state.active_panel == ActivePanel::CurrentChat {
                    ollama_state.models.clear();
                    ollama_state.selected_model_index = 0;
                    ollama_state.loading_models = true;
                    ollama_state.error_message = Some(" Refreshing models...".to_string());

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
                    if let Err(e) = refresh_sessions(ollama_state) {
                        ollama_state.add_error_toast(format!("Failed to refresh sessions: {}", e));
                    } else {
                        ollama_state.add_success_toast("Sessions refreshed! ".to_string());
                    }
                }
            }
            KeyCode::Char(c) => {
                if ollama_state.editing_system_prompt {
                    // Edit system prompt
                    ollama_state.system_prompt_buffer.push(c);
                } else if ollama_state.active_panel == ActivePanel::ChatHistory {
                    // Search input - always allow typing in search when in chat history panel
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
                } else if ollama_state.active_panel == ActivePanel::ChatHistory {
                    // Search input - always allow backspace in search when in chat history panel
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
        metrics: MessageMetrics::default(),
        context_length: ollama_state.conversation.len() as u32,
    });

    // Clear input and set sending state
    ollama_state.input_buffer.clear();
    ollama_state.is_sending = true;
    ollama_state.pending_response_id = Some(request_id);
    ollama_state.typing_indicator = "Assistant is thinking...".to_string();

    // Start performance tracking for the response
    ollama_state.start_message_timing();
    ollama_state.mark_unsaved_changes();

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
            ollama_state.system_prompt_buffer = selected_session.system_prompt.clone();

            // Reset unsaved changes flag since we just loaded a saved session
            ollama_state.unsaved_changes = false;

            // Set the last assistant response for copy functionality
            ollama_state.last_assistant_response = ollama_state
                .conversation
                .iter()
                .rev()
                .find(|msg| msg.role == crate::ui::ollama::ChatRole::Assistant)
                .map(|msg| msg.content.clone());

            // Switch to chat panel
            ollama_state.active_panel = ActivePanel::CurrentChat;

            // Scroll to bottom to show the most recent messages
            ollama_state.scroll_to_bottom();
            ollama_state.add_success_toast(format!(
                "Loaded session: {} ({} messages)",
                selected_session.title,
                selected_session.get_message_count()
            ));
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
    ollama_state.last_assistant_response = None; // Clear copy functionality state

    if let Some(session) = &mut ollama_state.current_session {
        session.conversation.clear();
    }

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

            ollama_state.unsaved_changes = false;
        }
    } else if !ollama_state.conversation.is_empty() {
        // Create a new session from current conversation
        if let Some(model) = ollama_state.get_selected_model() {
            let mut new_session = crate::ui::ollama::ChatSession::new(
                model.clone(),
                ollama_state.system_prompt.clone(),
            );

            // Associate with current snippet if available
            if let Some(snippet) = &ollama_state.current_snippet {
                new_session = new_session.with_snippet(snippet, "Current Snippet".to_string());
            }

            // Copy conversation
            new_session.conversation = ollama_state.conversation.clone();

            // Save the new session
            if let Some(storage) = &ollama_state.chat_storage {
                storage.save_session(&new_session)?;
                ollama_state.saved_sessions.push(new_session.clone());
                ollama_state.current_session = Some(new_session);
                ollama_state.unsaved_changes = false;
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
    let scroll_amount = ollama_state.scroll_speed.max(1);
    ollama_state.scroll_position = ollama_state.scroll_position.saturating_sub(scroll_amount);
}

/// Safely scroll chat down with overflow protection
fn scroll_chat_down(ollama_state: &mut OllamaState) {
    // Calculate maximum scroll position based on content
    let max_scroll = calculate_max_scroll_position(ollama_state);
    let scroll_amount = ollama_state.scroll_speed.max(1);

    // Always allow scrolling up to the max position, but cap at reasonable limit
    ollama_state.scroll_position = (ollama_state.scroll_position + scroll_amount)
        .min(max_scroll)
        .min(2000);
}

/// Safely fast scroll chat up
fn fast_scroll_chat_up(ollama_state: &mut OllamaState) {
    let fast_scroll_amount = (ollama_state.scroll_speed * 5).max(5);
    ollama_state.scroll_position = ollama_state
        .scroll_position
        .saturating_sub(fast_scroll_amount);
}

/// Safely fast scroll chat down with overflow protection
fn fast_scroll_chat_down(ollama_state: &mut OllamaState) {
    // Calculate maximum scroll position based on content
    let max_scroll = calculate_max_scroll_position(ollama_state);
    let fast_scroll_amount = (ollama_state.scroll_speed * 5).max(5);

    // Always allow scrolling up to the max position, but cap at reasonable limit
    ollama_state.scroll_position = (ollama_state.scroll_position + fast_scroll_amount)
        .min(max_scroll)
        .min(2000);
}

/// Calculate maximum scroll position based on chat content
fn calculate_max_scroll_position(ollama_state: &OllamaState) -> usize {
    // Use a more reasonable estimate for visible height based on typical terminal sizes
    // Most terminals are at least 24 lines, chat area is typically 60-80% of that
    let estimated_visible_height = 12;

    // If no conversation, no scrolling needed
    if ollama_state.conversation.is_empty() {
        return 0;
    }

    // Calculate total height of all messages
    let mut total_height = 0;

    for msg in &ollama_state.conversation {
        // Estimate message height (content height + borders + spacing)
        let content_width = 60; // More conservative estimate for content width
        let wrapped_height = calculate_estimated_wrapped_height(&msg.content, content_width);
        let message_height = wrapped_height + 4; // Account for borders, title, and spacing
        total_height += message_height + 1; // Add spacing between messages
    }

    // Add space for typing indicator if active
    if ollama_state.is_sending {
        total_height += 4;
    }

    // Return maximum scroll position (total height - visible height)
    // Allow scrolling if content is larger than visible area, but cap at reasonable maximum
    if total_height > estimated_visible_height {
        (total_height - estimated_visible_height).min(1000)
    } else {
        // Even for short content, allow minimal scrolling to test functionality
        if ollama_state.conversation.len() > 1 {
            1
        } else {
            0
        }
    }
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

    // Add some padding for markdown formatting, code blocks, etc.
    let estimated_height = total_lines.max(1);
    (estimated_height as f64 * 1.2).ceil() as usize
}

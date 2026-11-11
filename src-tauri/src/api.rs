use crate::store::{
    effective_language, normalize_base_url, origin_from_base_url, Provider, Settings,
};
use crate::tools;
use crate::AppState;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio_util::sync::CancellationToken;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitialState {
    pub settings: Settings,
    pub conversations: Value,
    pub data_dir: String,
    pub system_language: String,
}

#[derive(Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatRequest {
    pub request_id: String,
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub system_prompt: Option<String>,
    pub thinking: bool,
    pub temperature: Option<f64>,
    pub base_url: String,
    pub api_key: String,
    pub language: String,
    pub tools_enabled: bool,
    pub allow_read: bool,
    pub allow_write: bool,
    pub allow_shell: bool,
    pub confirm_writes: bool,
    pub confirm_shell: bool,
    pub workspace: String,
    #[serde(default)]
    pub provider_kind: String,
    #[serde(default)]
    pub provider_name: String,
}

#[tauri::command]
pub fn get_initial_state(state: State<'_, AppState>) -> InitialState {
    InitialState {
        settings: state.store.load_settings(),
        conversations: state.store.load_conversations(),
        data_dir: state.store.dir().to_string_lossy().to_string(),
        system_language: crate::store::system_language().to_string(),
    }
}

#[tauri::command]
pub fn get_dev_prompt() -> Option<String> {
    std::env::var("LUMO_DESKTOP_DEV_PROMPT")
        .ok()
        .filter(|value| !value.trim().is_empty())
}

#[tauri::command]
pub fn get_dev_agent() -> bool {
    std::env::var("LUMO_DESKTOP_DEV_AGENT")
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DevAttachment {
    pub name: String,
    pub mime: String,
    pub data_base64: String,
}

#[tauri::command]
pub fn get_dev_open() -> Option<String> {
    std::env::var("LUMO_DESKTOP_DEV_OPEN")
        .ok()
        .filter(|value| !value.trim().is_empty())
}

#[tauri::command]
pub fn get_dev_attachment() -> Option<DevAttachment> {
    let path = std::env::var("LUMO_DESKTOP_DEV_ATTACH").ok()?;
    let path = PathBuf::from(path);
    let bytes = std::fs::read(&path).ok()?;
    let name = path.file_name()?.to_string_lossy().to_string();
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let mime = match extension.as_str() {
        "txt" | "md" | "csv" | "log" => "text/plain",
        "json" => "application/json",
        "pdf" => "application/pdf",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "odt" => "application/vnd.oasis.opendocument.text",
        _ => "application/octet-stream",
    };
    Some(DevAttachment {
        name,
        mime: mime.to_string(),
        data_base64: {
            use base64::Engine;
            base64::engine::general_purpose::STANDARD.encode(bytes)
        },
    })
}

#[tauri::command]
pub fn save_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    mut settings: Settings,
) -> Result<(), String> {
    state.store.save_settings(&mut settings)?;
    crate::tray::update_menu(&app, &settings);
    Ok(())
}

#[tauri::command]
pub fn save_conversations(state: State<'_, AppState>, conversations: Value) -> Result<(), String> {
    state.store.save_conversations(&conversations)
}

#[tauri::command]
pub async fn check_health(state: State<'_, AppState>, settings: Settings) -> Result<Value, String> {
    let lang = effective_language(&settings);
    let provider = settings.provider().cloned();
    let kind = provider
        .as_ref()
        .map(|provider| provider.kind.as_str())
        .unwrap_or("lumo");
    let provider_name = provider
        .as_ref()
        .map(|provider| provider.name.as_str())
        .unwrap_or("lumo-tamer");

    if kind != "lumo" {
        let base = provider
            .as_ref()
            .map(|provider| normalize_base_url(&provider.base_url))
            .unwrap_or_else(|| normalize_base_url(&settings.base_url));
        let url = format!("{base}/models");
        let api_key = provider
            .as_ref()
            .map(|provider| provider.api_key.as_str())
            .unwrap_or(settings.api_key.as_str());

        let mut request = state.client.get(&url).timeout(Duration::from_secs(8));
        if !api_key.trim().is_empty() {
            request = request.bearer_auth(api_key.trim());
        }

        let response = request
            .send()
            .await
            .map_err(|error| friendly_network_error(&url, &error, lang))?;
        let status = response.status();
        let body = response.text().await.map_err(|error| error.to_string())?;

        if !status.is_success() {
            return Err(http_error(lang, status.as_u16(), &body));
        }

        let parsed: Value = serde_json::from_str(&body).unwrap_or_else(|_| json!({}));
        let models = parsed
            .get("data")
            .and_then(Value::as_array)
            .map(|items| items.len())
            .unwrap_or(0);

        return Ok(json!({
            "status": "ok",
            "provider": provider_name,
            "kind": kind,
            "auth": { "valid": true },
            "models": models
        }));
    }

    let origin = origin_from_base_url(&normalize_base_url(&settings.base_url));
    let url = format!("{origin}/health");

    let response = state
        .client
        .get(&url)
        .timeout(Duration::from_secs(4))
        .send()
        .await
        .map_err(|error| friendly_network_error(&url, &error, lang))?;

    let status = response.status();
    let body = response.text().await.map_err(|e| e.to_string())?;

    if !status.is_success() {
        return Err(http_error(lang, status.as_u16(), &body));
    }

    let mut health = serde_json::from_str::<Value>(&body).map_err(|e| {
        format!(
            "{}: {e} — {}",
            tr(lang, "Réponse /health invalide", "Invalid /health response"),
            truncate(&body, 300)
        )
    })?;
    if let Some(object) = health.as_object_mut() {
        object.insert("provider".into(), json!(provider_name));
        object.insert("kind".into(), json!(kind));
    }
    Ok(health)
}

#[tauri::command]
pub async fn list_models(
    state: State<'_, AppState>,
    settings: Settings,
) -> Result<Vec<ModelInfo>, String> {
    let lang = effective_language(&settings);
    let base = normalize_base_url(&settings.base_url);
    let url = format!("{base}/models");

    let mut request = state.client.get(&url).timeout(Duration::from_secs(8));
    if !settings.api_key.trim().is_empty() {
        request = request.bearer_auth(settings.api_key.trim());
    }

    let response = request
        .send()
        .await
        .map_err(|error| friendly_network_error(&url, &error, lang))?;
    let status = response.status();
    let body = response.text().await.map_err(|e| e.to_string())?;

    if !status.is_success() {
        return Err(http_error(lang, status.as_u16(), &body));
    }

    let parsed: Value = serde_json::from_str(&body).map_err(|e| {
        format!(
            "{}: {e} — {}",
            tr(lang, "Réponse /models invalide", "Invalid /models response"),
            truncate(&body, 300)
        )
    })?;
    let models = parsed
        .get("data")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    let id = item.get("id")?.as_str()?.to_string();
                    let name = item
                        .get("name")
                        .and_then(Value::as_str)
                        .unwrap_or(&id)
                        .to_string();
                    Some(ModelInfo { id, name })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if models.is_empty() {
        return Err(tr(
            lang,
            "Aucun modèle renvoyé par le fournisseur",
            "The provider returned no models",
        )
        .to_string());
    }

    Ok(models)
}

#[tauri::command]
pub fn detect_opencode_providers() -> Vec<Provider> {
    crate::store::import_opencode_providers()
}

#[tauri::command]
pub async fn extract_memories(
    state: State<'_, AppState>,
    settings: Settings,
    text: String,
) -> Result<Vec<String>, String> {
    let lang = effective_language(&settings);
    let base = normalize_base_url(&settings.base_url);
    let url = format!("{base}/chat/completions");
    let snippet: String = text.chars().take(14_000).collect();
    if snippet.trim().is_empty() {
        return Ok(Vec::new());
    }

    let body = json!({
        "model": settings.model,
        "stream": false,
        "temperature": 0.1,
        "messages": [
            {
                "role": "system",
                "content": "Extract durable, user-specific facts from the conversation. \
                    Keep only stable preferences, identity details, ongoing projects, or \
                    important context. Ignore transient requests. Return ONLY a JSON array \
                    of short strings (max 5). If there is nothing durable, return []."
            },
            { "role": "user", "content": snippet }
        ]
    });

    let mut request = state.client.post(&url).json(&body);
    if !settings.api_key.trim().is_empty() {
        request = request.bearer_auth(settings.api_key.trim());
    }
    let response = request
        .send()
        .await
        .map_err(|error| friendly_network_error(&url, &error, lang))?;
    let status = response.status();
    let body = response.text().await.map_err(|error| error.to_string())?;
    if !status.is_success() {
        return Err(http_error(lang, status.as_u16(), &body));
    }

    let value: Value =
        serde_json::from_str(&body).map_err(|error| format!("Invalid memory response: {error}"))?;
    let content = value
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(Value::as_str)
        .unwrap_or("");

    let start = content.find('[');
    let end = content.rfind(']');
    let Some((start, end)) = start.zip(end) else {
        return Ok(Vec::new());
    };
    if end <= start {
        return Ok(Vec::new());
    }

    let parsed: Vec<String> = serde_json::from_str(&content[start..=end]).unwrap_or_default();
    let mut memories = Vec::new();
    for item in parsed {
        let item = item.trim();
        if item.is_empty() {
            continue;
        }
        let item: String = item.chars().take(400).collect();
        if !memories.iter().any(|existing: &String| existing == &item) {
            memories.push(item);
        }
        if memories.len() >= 5 {
            break;
        }
    }
    Ok(memories)
}

#[tauri::command]
pub fn start_chat(
    app: AppHandle,
    state: State<'_, AppState>,
    request: ChatRequest,
) -> Result<(), String> {
    let lang = if request.language == "fr" { "fr" } else { "en" };
    if request.request_id.trim().is_empty() {
        return Err(tr(lang, "requestId manquant", "Missing requestId").to_string());
    }
    if request.messages.is_empty() {
        return Err(tr(lang, "Aucun message à envoyer", "No message to send").to_string());
    }

    let request_id = request.request_id.clone();
    let token = CancellationToken::new();
    state
        .cancels
        .lock()
        .map_err(|_| {
            tr(
                lang,
                "verrou de requête empoisonné",
                "request lock poisoned",
            )
            .to_string()
        })?
        .insert(request_id, token.clone());

    let client = state.client.clone();
    tauri::async_runtime::spawn(async move {
        run_chat(app, client, request, token).await;
    });

    Ok(())
}

#[tauri::command]
pub fn stop_chat(state: State<'_, AppState>, request_id: String) -> bool {
    let token = state
        .cancels
        .lock()
        .ok()
        .and_then(|mut map| map.remove(&request_id));
    if let Some(token) = token {
        token.cancel();
        true
    } else {
        false
    }
}

#[tauri::command]
pub fn resolve_tool_permission(
    state: State<'_, AppState>,
    call_id: String,
    decision: String,
) -> bool {
    let sender = state
        .permissions
        .lock()
        .ok()
        .and_then(|mut map| map.remove(&call_id));
    if let Some(sender) = sender {
        sender.send(decision).is_ok()
    } else {
        false
    }
}

#[tauri::command]
pub async fn start_tamer(
    directory: Option<String>,
    language: Option<String>,
) -> Result<String, String> {
    let lang = if language.as_deref() == Some("fr") {
        "fr"
    } else {
        "en"
    };
    let directory = directory
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|home| home.join("Projects/lumo-tamer")))
        .ok_or_else(|| {
            tr(
                lang,
                "Impossible de déterminer le dossier de lumo-tamer",
                "Cannot determine the lumo-tamer directory",
            )
            .to_string()
        })?;

    if !directory.is_dir() {
        return Err(format!(
            "{}: {}",
            tr(lang, "Dossier introuvable", "Directory not found"),
            directory.display()
        ));
    }
    if !directory.join("docker-compose.yml").exists()
        && !directory.join("compose.yaml").exists()
        && !directory.join("compose.yml").exists()
    {
        return Err(format!(
            "{} {}",
            tr(
                lang,
                "Aucun fichier docker compose dans",
                "No docker compose file in"
            ),
            directory.display()
        ));
    }

    let output = tokio::process::Command::new("docker")
        .args(["compose", "up", "-d", "tamer"])
        .current_dir(&directory)
        .output()
        .await
        .map_err(|error| {
            format!(
                "{}: {error}",
                tr(lang, "Impossible de lancer docker", "Cannot run docker")
            )
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let details = match (stdout.is_empty(), stderr.is_empty()) {
        (false, false) => format!("{stdout}\n{stderr}"),
        (false, true) => stdout,
        (true, false) => stderr,
        (true, true) => String::new(),
    };

    if output.status.success() {
        Ok(details)
    } else {
        Err(if details.is_empty() {
            format!(
                "{} ({})",
                tr(lang, "docker compose a échoué", "docker compose failed"),
                output.status
            )
        } else {
            details
        })
    }
}

#[tauri::command]
pub fn open_external(url: String) -> Result<(), String> {
    let trimmed = url.trim();
    if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
        return Err("Only http(s) links can be opened".into());
    }
    open::that(trimmed).map_err(|error| format!("Cannot open link: {error}"))
}

enum Outcome {
    Done {
        finish_reason: Option<String>,
        usage: Option<Value>,
    },
    Cancelled,
    Error(String),
}

#[derive(Default)]
struct PendingToolCall {
    id: String,
    name: String,
    arguments: String,
}

async fn run_chat(
    app: AppHandle,
    client: reqwest::Client,
    request: ChatRequest,
    token: CancellationToken,
) {
    let request_id = request.request_id.clone();
    emit(&app, json!({ "requestId": request_id, "type": "started" }));

    let outcome = perform_chat(&app, &client, &request, &token).await;

    match outcome {
        Outcome::Done {
            finish_reason,
            usage,
        } => emit(
            &app,
            json!({
                "requestId": request_id,
                "type": "done",
                "finishReason": finish_reason,
                "usage": usage,
            }),
        ),
        Outcome::Cancelled => emit(
            &app,
            json!({ "requestId": request_id, "type": "done", "cancelled": true }),
        ),
        Outcome::Error(message) => {
            emit(
                &app,
                json!({ "requestId": request_id, "type": "error", "message": message }),
            );
            emit(
                &app,
                json!({ "requestId": request_id, "type": "done", "error": true }),
            );
        }
    }

    if let Some(state) = app.try_state::<AppState>() {
        if let Ok(mut cancels) = state.cancels.lock() {
            cancels.remove(&request_id);
        }
    }
}

async fn perform_chat(
    app: &AppHandle,
    client: &reqwest::Client,
    request: &ChatRequest,
    token: &CancellationToken,
) -> Outcome {
    let lang = if request.language == "fr" { "fr" } else { "en" };
    let provider_kind = request.provider_kind.trim();
    let provider_label = if request.provider_name.trim().is_empty() {
        "API"
    } else {
        request.provider_name.trim()
    };
    let base = normalize_base_url(&request.base_url);
    let url = format!("{base}/chat/completions");
    let workspace = tools::canonical_workspace(&request.workspace);
    let tool_defs = if request.tools_enabled {
        tools::definitions(request.allow_read, request.allow_write, request.allow_shell)
    } else {
        Vec::new()
    };

    let mut messages: Vec<Value> = Vec::new();
    let mut system_prompt = request
        .system_prompt
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_default();

    if !tool_defs.is_empty() {
        system_prompt.push_str(&format!(
            "\n\nYou can use local tools. The authorized workspace is `{}`. \
             Use absolute paths inside that workspace. Some write or shell actions may \
             require the user's confirmation; the application handles the prompt. \
             If a tool fails, report the error briefly and continue.",
            workspace.display()
        ));
    }

    if !system_prompt.is_empty() {
        messages.push(json!({ "role": "system", "content": system_prompt }));
    }
    for message in &request.messages {
        let role = match message.role.as_str() {
            "user" | "assistant" | "system" => message.role.as_str(),
            _ => continue,
        };
        if message.content.is_empty() {
            continue;
        }
        messages.push(json!({ "role": role, "content": message.content }));
    }

    if messages.iter().all(|message| message["role"] == "system") {
        return Outcome::Error(
            tr(
                lang,
                "Aucun message valide à envoyer",
                "No valid message to send",
            )
            .to_string(),
        );
    }

    let max_rounds = 8usize;
    let mut final_finish: Option<String> = None;
    let mut final_usage: Option<Value> = None;

    for round in 0..max_rounds {
        if token.is_cancelled() {
            return Outcome::Cancelled;
        }

        let mut body = json!({
            "model": request.model,
            "stream": true,
            "messages": messages,
        });
        if request.thinking && provider_kind == "lumo" {
            body["reasoning_effort"] = json!("high");
        }
        if let Some(temperature) = request.temperature {
            body["temperature"] = json!(temperature);
        }
        if !tool_defs.is_empty() {
            body["tools"] = Value::Array(tool_defs.clone());
            body["tool_choice"] = json!("auto");
        }

        let mut http_request = client.post(&url).json(&body);
        if !request.api_key.trim().is_empty() {
            http_request = http_request.bearer_auth(request.api_key.trim());
        }

        let response = tokio::select! {
            _ = token.cancelled() => return Outcome::Cancelled,
            result = http_request.send() => match result {
                Ok(response) => response,
                Err(error) => return Outcome::Error(friendly_network_error(&url, &error, lang)),
            },
        };

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Outcome::Error(http_error(lang, status.as_u16(), &body));
        }

        let mut stream = response.bytes_stream();
        let mut buffer: Vec<u8> = Vec::new();
        let mut tool_calls: BTreeMap<usize, PendingToolCall> = BTreeMap::new();
        let mut round_content = String::new();
        let mut round_finish: Option<String> = None;
        let mut round_usage: Option<Value> = None;
        let mut sse_done = false;
        let mut stream_error: Option<String> = None;

        while !sse_done {
            let chunk = tokio::select! {
                _ = token.cancelled() => return Outcome::Cancelled,
                chunk = stream.next() => chunk,
            };

            let Some(chunk) = chunk else {
                break;
            };

            let bytes = match chunk {
                Ok(bytes) => bytes,
                Err(error) => {
                    stream_error = Some(format!(
                        "{}: {error}",
                        tr(lang, "Flux interrompu", "Stream interrupted")
                    ));
                    break;
                }
            };

            buffer.extend_from_slice(&bytes);

            while let Some(position) = buffer.iter().position(|byte| *byte == b'\n') {
                let line_bytes: Vec<u8> = buffer.drain(..=position).collect();
                let line = String::from_utf8_lossy(&line_bytes)
                    .trim_end_matches(['\r', '\n'])
                    .to_string();

                if line.is_empty() || line.starts_with(':') {
                    continue;
                }

                let Some(data) = line.strip_prefix("data:") else {
                    continue;
                };
                let data = data.trim();
                if data.is_empty() {
                    continue;
                }
                if data == "[DONE]" {
                    sse_done = true;
                    break;
                }

                let value: Value = match serde_json::from_str(data) {
                    Ok(value) => value,
                    Err(_) => continue,
                };

                if let Some(error) = value.get("error") {
                    let message = error
                        .get("message")
                        .and_then(Value::as_str)
                        .map(str::to_string)
                        .unwrap_or_else(|| error.to_string());
                    return Outcome::Error(format!("{provider_label}: {message}"));
                }

                if let Some(usage) = value.get("usage").filter(|usage| !usage.is_null()) {
                    round_usage = Some(usage.clone());
                }

                let Some(choices) = value.get("choices").and_then(Value::as_array) else {
                    continue;
                };

                for choice in choices {
                    if let Some(reason) = choice.get("finish_reason").and_then(Value::as_str) {
                        round_finish = Some(reason.to_string());
                    }

                    let Some(delta) = choice.get("delta") else {
                        continue;
                    };

                    if let Some(content) = delta.get("content").and_then(Value::as_str) {
                        if !content.is_empty() {
                            round_content.push_str(content);
                            emit(
                                app,
                                json!({
                                    "requestId": request.request_id,
                                    "type": "delta",
                                    "content": content,
                                }),
                            );
                        }
                    }

                    let reasoning = delta
                        .get("reasoning_content")
                        .or_else(|| delta.get("reasoning"))
                        .and_then(Value::as_str);
                    if let Some(reasoning) = reasoning {
                        if !reasoning.is_empty() {
                            emit(
                                app,
                                json!({
                                    "requestId": request.request_id,
                                    "type": "reasoning",
                                    "content": reasoning,
                                }),
                            );
                        }
                    }

                    if let Some(calls) = delta.get("tool_calls").and_then(Value::as_array) {
                        for call in calls {
                            let index =
                                call.get("index").and_then(Value::as_u64).unwrap_or(0) as usize;
                            let entry = tool_calls.entry(index).or_default();
                            if let Some(id) = call.get("id").and_then(Value::as_str) {
                                if !id.is_empty() {
                                    entry.id = id.to_string();
                                }
                            }
                            if let Some(function) = call.get("function") {
                                if let Some(name) = function.get("name").and_then(Value::as_str) {
                                    entry.name.push_str(name);
                                }
                                if let Some(arguments) =
                                    function.get("arguments").and_then(Value::as_str)
                                {
                                    entry.arguments.push_str(arguments);
                                }
                            }
                        }
                    }
                }
            }
        }

        if let Some(message) = stream_error {
            return Outcome::Error(message);
        }
        if token.is_cancelled() {
            return Outcome::Cancelled;
        }

        final_finish = round_finish.clone().or(final_finish);
        if let Some(usage) = round_usage {
            final_usage = Some(usage);
        }

        let mut calls: Vec<PendingToolCall> = tool_calls.into_values().collect();
        if calls.is_empty() {
            return Outcome::Done {
                finish_reason: final_finish,
                usage: final_usage,
            };
        }

        if tool_defs.is_empty() {
            return Outcome::Error(
                tr(
                    lang,
                    "Le modèle a demandé un outil alors que les outils sont désactivés.",
                    "The model requested a tool while tools are disabled.",
                )
                .to_string(),
            );
        }

        for (index, call) in calls.iter_mut().enumerate() {
            if call.id.is_empty() {
                call.id = format!("call_{round}_{index}");
            }
            if call.name.is_empty() {
                call.name = "unknown".into();
            }
        }

        let tool_calls_json: Vec<Value> = calls
            .iter()
            .map(|call| {
                json!({
                    "id": call.id,
                    "type": "function",
                    "function": {
                        "name": call.name,
                        "arguments": call.arguments,
                    }
                })
            })
            .collect();
        messages.push(json!({
            "role": "assistant",
            "content": round_content,
            "tool_calls": tool_calls_json,
        }));

        for call in calls {
            let parsed_args: Value = serde_json::from_str(&call.arguments)
                .unwrap_or_else(|error| json!({ "_parse_error": error.to_string() }));
            let category = tools::category(&call.name).unwrap_or("unknown");
            let summary = tools::summarize(&call.name, &parsed_args);
            let preview = tools::preview(&parsed_args);

            emit(
                app,
                json!({
                    "requestId": request.request_id,
                    "type": "tool_start",
                    "callId": call.id,
                    "name": call.name,
                    "arguments": parsed_args,
                    "summary": summary,
                }),
            );

            let allowed_by_settings = match category {
                "read" => request.allow_read,
                "write" => request.allow_write,
                "shell" => request.allow_shell,
                _ => false,
            };

            let (result, denied) = if !allowed_by_settings {
                (
                    tools::ToolResult {
                        ok: false,
                        content: tr(
                            lang,
                            "Cet outil est désactivé dans les paramètres.",
                            "This tool is disabled in settings.",
                        )
                        .to_string(),
                    },
                    true,
                )
            } else {
                let needs_confirmation = match category {
                    "write" => request.confirm_writes,
                    "shell" => request.confirm_shell,
                    _ => false,
                };

                let allowed_by_user = if needs_confirmation {
                    let permission_id = format!("{}:{}", request.request_id, call.id);
                    let granted = request_tool_permission(
                        app,
                        token,
                        &request.request_id,
                        &permission_id,
                        category,
                        &call.name,
                        &summary,
                        &preview,
                    )
                    .await;
                    if token.is_cancelled() {
                        return Outcome::Cancelled;
                    }
                    granted
                } else {
                    true
                };

                if !allowed_by_user {
                    (
                        tools::ToolResult {
                            ok: false,
                            content: tr(
                                lang,
                                "Permission refusée par l'utilisateur.",
                                "Permission denied by the user.",
                            )
                            .to_string(),
                        },
                        true,
                    )
                } else {
                    (
                        tools::execute(&call.name, &parsed_args, &workspace, token).await,
                        false,
                    )
                }
            };

            let content_for_model = if result.ok {
                result.content.clone()
            } else {
                format!("ERROR: {}", result.content)
            };
            emit(
                app,
                json!({
                    "requestId": request.request_id,
                    "type": "tool_result",
                    "callId": call.id,
                    "name": call.name,
                    "ok": result.ok,
                    "denied": denied,
                    "content": truncate(&result.content, 8000),
                }),
            );

            messages.push(json!({
                "role": "tool",
                "tool_call_id": call.id,
                "content": truncate(&content_for_model, 40_000),
            }));
        }
    }

    Outcome::Error(
        tr(
            lang,
            "Limite d'itérations d'outils atteinte.",
            "Tool iteration limit reached.",
        )
        .to_string(),
    )
}

async fn request_tool_permission(
    app: &AppHandle,
    token: &CancellationToken,
    request_id: &str,
    permission_id: &str,
    category: &str,
    tool_name: &str,
    summary: &str,
    preview: &str,
) -> bool {
    if app
        .state::<AppState>()
        .auto_approve
        .lock()
        .map(|set| set.contains(category))
        .unwrap_or(false)
    {
        return true;
    }

    let (sender, receiver) = tokio::sync::oneshot::channel();
    let inserted = app
        .state::<AppState>()
        .permissions
        .lock()
        .map(|mut map| map.insert(permission_id.to_string(), sender))
        .is_ok();
    if !inserted {
        return false;
    }

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }

    emit(
        app,
        json!({
            "requestId": request_id,
            "type": "permission_request",
            "callId": permission_id,
            "toolCallId": permission_id.split_once(':').map(|(_, id)| id).unwrap_or(permission_id),
            "category": category,
            "tool": tool_name,
            "summary": summary,
            "preview": preview,
        }),
    );

    let decision = tokio::select! {
        _ = token.cancelled() => "deny".to_string(),
        _ = tokio::time::sleep(Duration::from_secs(120)) => "deny".to_string(),
        result = receiver => result.unwrap_or_else(|_| "deny".to_string()),
    };

    if let Ok(mut map) = app.state::<AppState>().permissions.lock() {
        map.remove(permission_id);
    }

    match decision.as_str() {
        "allow" => true,
        "always" => {
            if let Ok(mut set) = app.state::<AppState>().auto_approve.lock() {
                set.insert(category.to_string());
            }
            true
        }
        _ => false,
    }
}

fn emit(app: &AppHandle, payload: Value) {
    let _ = app.emit("chat-event", payload);
}

fn tr<'a>(lang: &str, french: &'a str, english: &'a str) -> &'a str {
    if lang == "fr" {
        french
    } else {
        english
    }
}

fn http_error(lang: &str, status: u16, body: &str) -> String {
    format!(
        "{} {status}: {}",
        tr(lang, "L'API a répondu HTTP", "API returned HTTP"),
        truncate(body, 800)
    )
}

fn friendly_network_error(url: &str, error: &reqwest::Error, lang: &str) -> String {
    let is_local_tamer = url.contains("3003");
    if error.is_connect() {
        if is_local_tamer {
            format!(
                "{} {url}. {}",
                tr(
                    lang,
                    "Impossible de joindre lumo-tamer sur",
                    "Cannot reach lumo-tamer at"
                ),
                tr(
                    lang,
                    "Vérifie que le conteneur tourne : `docker compose up -d tamer`.",
                    "Make sure the container is running: `docker compose up -d tamer`."
                )
            )
        } else {
            format!(
                "{} {url}.",
                tr(
                    lang,
                    "Impossible de joindre l'API sur",
                    "Cannot reach the API at"
                )
            )
        }
    } else if error.is_timeout() {
        format!(
            "{} {url}.",
            tr(
                lang,
                "L'API ne répond pas sur",
                "The API did not respond at"
            )
        )
    } else {
        format!(
            "{} {url}: {error}",
            tr(lang, "Erreur réseau vers", "Network error reaching")
        )
    }
}

fn truncate(value: &str, max: usize) -> String {
    let trimmed = value.trim();
    if trimmed.chars().count() <= max {
        return trimmed.to_string();
    }
    let shortened: String = trimmed.chars().take(max).collect();
    format!("{shortened}…")
}

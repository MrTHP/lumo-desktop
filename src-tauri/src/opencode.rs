use crate::api::ModelInfo;
use crate::store::Settings;
use crate::AppState;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio_util::sync::CancellationToken;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenCodeStatus {
    pub installed: bool,
    pub path: Option<String>,
    pub version: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRequest {
    pub request_id: String,
    pub prompt: String,
    pub workspace: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub agent: String,
    #[serde(default)]
    pub session_id: String,
    #[serde(default)]
    pub auto_approve: bool,
    #[serde(default)]
    pub binary: String,
    #[serde(default)]
    pub language: String,
}

fn tr<'a>(lang: &str, french: &'a str, english: &'a str) -> &'a str {
    if lang == "fr" {
        french
    } else {
        english
    }
}

fn which(binary: &str) -> Option<PathBuf> {
    if binary.contains('/') {
        let path = PathBuf::from(binary);
        return path.is_file().then_some(path);
    }

    if let Some(paths) = std::env::var_os("PATH") {
        for directory in std::env::split_paths(&paths) {
            let candidate = directory.join(binary);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

fn find_opencode(explicit: &str) -> Option<PathBuf> {
    let explicit = explicit.trim();
    if !explicit.is_empty() {
        if let Some(path) = which(explicit) {
            return Some(path);
        }
    }

    if let Ok(configured) = std::env::var("OPENCODE_BIN") {
        if let Some(path) = which(configured.trim()) {
            return Some(path);
        }
    }

    if let Some(path) = which("opencode") {
        return Some(path);
    }

    let mut candidates = Vec::new();
    if let Some(home) = dirs::home_dir() {
        candidates.push(home.join(".opencode/bin/opencode"));
        candidates.push(home.join(".local/bin/opencode"));
    }
    candidates.push(PathBuf::from("/usr/local/bin/opencode"));
    candidates.push(PathBuf::from("/usr/bin/opencode"));

    candidates.into_iter().find(|path| path.is_file())
}

#[tauri::command]
pub async fn opencode_status(settings: Settings) -> Result<OpenCodeStatus, String> {
    let Some(binary) = find_opencode(&settings.opencode_binary) else {
        return Ok(OpenCodeStatus {
            installed: false,
            path: None,
            version: None,
        });
    };

    let output = tokio::time::timeout(
        Duration::from_secs(8),
        Command::new(&binary).arg("--version").output(),
    )
    .await;

    let version = match output {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            if stdout.is_empty() {
                (!stderr.is_empty()).then_some(stderr)
            } else {
                Some(stdout)
            }
        }
        _ => None,
    };

    Ok(OpenCodeStatus {
        installed: true,
        path: Some(binary.to_string_lossy().to_string()),
        version,
    })
}

#[tauri::command]
pub async fn list_opencode_models(settings: Settings) -> Result<Vec<ModelInfo>, String> {
    let lang = if settings.language == "fr" {
        "fr"
    } else {
        "en"
    };
    let Some(binary) = find_opencode(&settings.opencode_binary) else {
        return Err(tr(
            lang,
            "OpenCode est introuvable. Indique son chemin dans les réglages.",
            "OpenCode was not found. Set its path in settings.",
        )
        .to_string());
    };

    let output = tokio::time::timeout(
        Duration::from_secs(30),
        Command::new(&binary)
            .arg("models")
            .env("NO_COLOR", "1")
            .output(),
    )
    .await
    .map_err(|_| {
        tr(
            lang,
            "OpenCode met trop de temps à répondre.",
            "OpenCode took too long to respond.",
        )
        .to_string()
    })?
    .map_err(|error| format!("OpenCode: {error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "{}: {}",
            tr(lang, "OpenCode a échoué", "OpenCode failed"),
            stderr.trim().chars().take(600).collect::<String>()
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut models: Vec<ModelInfo> = stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && line.contains('/'))
        .map(|line| ModelInfo {
            id: line.to_string(),
            name: line.to_string(),
        })
        .collect();

    models.sort_by(|left, right| left.id.cmp(&right.id));
    models.dedup_by(|left, right| left.id == right.id);

    if models.is_empty() {
        return Err(tr(
            lang,
            "OpenCode n'a renvoyé aucun modèle.",
            "OpenCode returned no models.",
        )
        .to_string());
    }

    Ok(models)
}

#[tauri::command]
pub async fn start_agent(
    app: AppHandle,
    state: State<'_, AppState>,
    request: AgentRequest,
) -> Result<(), String> {
    let lang = if request.language == "fr" { "fr" } else { "en" };
    if request.request_id.trim().is_empty() {
        return Err(tr(lang, "requestId manquant", "Missing requestId").to_string());
    }
    if request.prompt.trim().is_empty() {
        return Err(tr(lang, "Message vide", "Empty message").to_string());
    }

    let workspace = PathBuf::from(request.workspace.trim());
    let workspace = workspace.canonicalize().map_err(|_| {
        format!(
            "{}: {}",
            tr(lang, "Dossier de travail invalide", "Invalid workspace"),
            workspace.display()
        )
    })?;
    if !workspace.is_dir() {
        return Err(tr(
            lang,
            "Le dossier de travail n'est pas un dossier.",
            "The workspace is not a directory.",
        )
        .to_string());
    }

    let Some(binary) = find_opencode(&request.binary) else {
        return Err(tr(
            lang,
            "OpenCode est introuvable. Installe-le ou indique son chemin dans les réglages.",
            "OpenCode was not found. Install it or set its path in settings.",
        )
        .to_string());
    };

    let mut command = Command::new(&binary);
    command
        .arg("run")
        .arg("--standalone")
        .arg("--format")
        .arg("json")
        .current_dir(&workspace)
        // OpenCode resolves the project from PWD as well as the process cwd.
        .env("PWD", &workspace)
        .env("NO_COLOR", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    if !request.model.trim().is_empty() {
        command.arg("--model").arg(request.model.trim());
    }
    let agent = if request.agent.trim().is_empty() {
        "build"
    } else {
        request.agent.trim()
    };
    command.arg("--agent").arg(agent);
    if !request.session_id.trim().is_empty() {
        command.arg("--session").arg(request.session_id.trim());
    }
    if request.auto_approve {
        command.arg("--auto");
    }
    command.arg(request.prompt.trim());

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
        .insert(request_id.clone(), token.clone());

    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            if let Ok(mut cancels) = state.cancels.lock() {
                cancels.remove(&request_id);
            }
            return Err(format!(
                "{}: {error}",
                tr(
                    lang,
                    "Impossible de lancer OpenCode",
                    "Cannot launch OpenCode"
                )
            ));
        }
    };

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let stderr_tail: Arc<Mutex<VecDeque<String>>> = Arc::new(Mutex::new(VecDeque::new()));

    if let Some(stderr) = stderr {
        let tail = stderr_tail.clone();
        tauri::async_runtime::spawn(async move {
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let line = line.trim().to_string();
                if line.is_empty() {
                    continue;
                }
                if let Ok(mut tail) = tail.lock() {
                    tail.push_back(line.chars().take(800).collect());
                    while tail.len() > 24 {
                        tail.pop_front();
                    }
                }
            }
        });
    }

    tauri::async_runtime::spawn(async move {
        run_agent(
            app,
            child,
            stdout,
            request_id,
            token,
            lang.to_string(),
            stderr_tail,
        )
        .await;
    });

    Ok(())
}

async fn run_agent(
    app: AppHandle,
    mut child: tokio::process::Child,
    stdout: Option<tokio::process::ChildStdout>,
    request_id: String,
    token: CancellationToken,
    lang: String,
    stderr_tail: Arc<Mutex<VecDeque<String>>>,
) {
    let mut lines = BufReader::new(match stdout {
        Some(stdout) => stdout,
        None => {
            emit_agent(
                &app,
                json!({
                    "requestId": request_id,
                    "type": "error",
                    "message": tr(&lang, "OpenCode n'a pas de sortie.", "OpenCode has no output.")
                }),
            );
            emit_done(&app, &request_id, false, false);
            cleanup(&app, &request_id);
            return;
        }
    })
    .lines();

    let mut text_parts: HashMap<String, String> = HashMap::new();
    let mut emitted_error = false;

    loop {
        let next = tokio::select! {
            _ = token.cancelled() => {
                let _ = child.kill().await;
                emit_done(&app, &request_id, true, false);
                cleanup(&app, &request_id);
                return;
            }
            line = lines.next_line() => line,
        };

        match next {
            Ok(Some(line)) => {
                if handle_agent_line(&app, &request_id, &line, &mut text_parts) {
                    emitted_error = true;
                }
            }
            Ok(None) => break,
            Err(error) => {
                emit_agent(
                    &app,
                    json!({
                        "requestId": request_id,
                        "type": "error",
                        "message": format!("{}: {error}", tr(&lang, "Flux OpenCode interrompu", "OpenCode stream interrupted"))
                    }),
                );
                emitted_error = true;
                break;
            }
        }
    }

    let status = child.wait().await;
    let success = status
        .as_ref()
        .map(|status| status.success())
        .unwrap_or(false);

    if !success && !emitted_error {
        let tail = stderr_tail
            .lock()
            .ok()
            .map(|tail| tail.iter().cloned().collect::<Vec<_>>().join("\n"))
            .filter(|tail| !tail.trim().is_empty())
            .unwrap_or_else(|| {
                tr(
                    &lang,
                    "OpenCode s'est arrêté avec une erreur.",
                    "OpenCode exited with an error.",
                )
                .to_string()
            });

        emit_agent(
            &app,
            json!({
                "requestId": request_id,
                "type": "error",
                "message": tail
            }),
        );
    }

    emit_done(&app, &request_id, false, !success);
    cleanup(&app, &request_id);
}

fn handle_agent_line(
    app: &AppHandle,
    request_id: &str,
    line: &str,
    text_parts: &mut HashMap<String, String>,
) -> bool {
    let Ok(value) = serde_json::from_str::<Value>(line) else {
        return false;
    };

    if let Some(session_id) = value.get("sessionID").and_then(Value::as_str) {
        if !session_id.is_empty() {
            emit_agent(
                app,
                json!({
                    "requestId": request_id,
                    "type": "session",
                    "sessionId": session_id
                }),
            );
        }
    }

    let event_type = value
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let part = value.get("part");

    if event_type == "error" {
        let message = value
            .get("error")
            .and_then(|error| error.get("message").or(Some(error)))
            .and_then(Value::as_str)
            .or_else(|| value.get("message").and_then(Value::as_str))
            .unwrap_or("OpenCode error");
        emit_agent(
            app,
            json!({
                "requestId": request_id,
                "type": "error",
                "message": message
            }),
        );
        return true;
    }

    if event_type == "text"
        || part
            .and_then(|part| part.get("type"))
            .and_then(Value::as_str)
            == Some("text")
    {
        if let Some(part) = part {
            let part_id = part.get("id").and_then(Value::as_str).unwrap_or("text");
            let text = part.get("text").and_then(Value::as_str).unwrap_or_default();
            if !text.is_empty() {
                let previous = text_parts.get(part_id).cloned().unwrap_or_default();
                let delta = if text.starts_with(&previous) {
                    text[previous.len()..].to_string()
                } else {
                    text.to_string()
                };
                text_parts.insert(part_id.to_string(), text.to_string());
                if !delta.is_empty() {
                    emit_agent(
                        app,
                        json!({
                            "requestId": request_id,
                            "type": "delta",
                            "content": delta
                        }),
                    );
                }
            }
        }
        return false;
    }

    if event_type == "reasoning"
        || part
            .and_then(|part| part.get("type"))
            .and_then(Value::as_str)
            == Some("reasoning")
    {
        if let Some(part) = part {
            let part_id = part
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or("reasoning");
            let text = part
                .get("text")
                .or_else(|| part.get("reasoning"))
                .and_then(Value::as_str)
                .unwrap_or_default();
            if !text.is_empty() {
                let key = format!("reasoning:{part_id}");
                let previous = text_parts.get(&key).cloned().unwrap_or_default();
                let delta = if text.starts_with(&previous) {
                    text[previous.len()..].to_string()
                } else {
                    text.to_string()
                };
                text_parts.insert(key, text.to_string());
                if !delta.is_empty() {
                    emit_agent(
                        app,
                        json!({
                            "requestId": request_id,
                            "type": "reasoning",
                            "content": delta
                        }),
                    );
                }
            }
        }
        return false;
    }

    if event_type == "tool"
        || event_type == "tool_use"
        || part
            .and_then(|part| part.get("type"))
            .and_then(Value::as_str)
            == Some("tool")
    {
        if let Some(part) = part {
            let tool = part
                .get("tool")
                .and_then(Value::as_str)
                .or_else(|| part.get("name").and_then(Value::as_str))
                .unwrap_or("tool");
            let state = part.get("state").cloned().unwrap_or(Value::Null);
            let call_id = part
                .get("callID")
                .or_else(|| part.get("callId"))
                .or_else(|| part.get("id"))
                .and_then(Value::as_str)
                .unwrap_or(tool);
            let title = state
                .get("title")
                .and_then(Value::as_str)
                .or_else(|| part.get("title").and_then(Value::as_str))
                .unwrap_or(tool);
            let status = state
                .get("status")
                .and_then(Value::as_str)
                .unwrap_or("running");
            emit_agent(
                app,
                json!({
                    "requestId": request_id,
                    "type": "tool",
                    "callId": call_id,
                    "tool": tool,
                    "title": title,
                    "status": status,
                    "state": state
                }),
            );
        }
        return false;
    }

    if event_type == "step_finish" {
        if let Some(part) = part {
            emit_agent(
                app,
                json!({
                    "requestId": request_id,
                    "type": "usage",
                    "usage": part.get("tokens").cloned().unwrap_or(Value::Null),
                    "cost": part.get("cost").cloned().unwrap_or(Value::Null),
                    "reason": part.get("reason").cloned().unwrap_or(Value::Null)
                }),
            );
        }
    }

    false
}

fn emit_agent(app: &AppHandle, payload: Value) {
    let _ = app.emit("agent-event", payload);
}

fn emit_done(app: &AppHandle, request_id: &str, cancelled: bool, error: bool) {
    emit_agent(
        app,
        json!({
            "requestId": request_id,
            "type": "done",
            "cancelled": cancelled,
            "error": error
        }),
    );
}

fn cleanup(app: &AppHandle, request_id: &str) {
    if let Some(state) = app.try_state::<AppState>() {
        if let Ok(mut cancels) = state.cancels.lock() {
            cancels.remove(request_id);
        }
    }
}

use crate::store::Settings;
use crate::AppState;
use base64::Engine;
use serde::Serialize;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

const DEFAULT_REPO: &str = "https://github.com/ZeroTricks/lumo-tamer.git";
const AUTH_URL: &str = "https://localhost:3001";
const BASE_URL: &str = "http://127.0.0.1:3003/v1";
const HEALTH_URL: &str = "http://127.0.0.1:3003/health";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TamerStatus {
    pub installed: bool,
    pub directory: String,
    pub config_exists: bool,
    pub secrets_exist: bool,
    pub docker_available: bool,
    pub docker_running: bool,
    pub compose_available: bool,
    pub git_available: bool,
    pub browser_running: bool,
    pub tamer_running: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallResult {
    pub directory: String,
    pub api_key: String,
    pub base_url: String,
    pub auth_url: String,
}

pub fn resolve_tamer_directory(settings: &Settings) -> PathBuf {
    let configured = settings.tamer_dir.trim();
    if configured.is_empty() {
        default_install_directory()
    } else {
        PathBuf::from(configured)
    }
}

fn default_install_directory() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let projects = home.join("Projects/lumo-tamer");
    if compose_file(&projects).is_some() {
        projects
    } else {
        let data_root = dirs::data_local_dir().unwrap_or_else(|| home.join(".local/share"));
        let current = data_root.join("model-desk/lumo-tamer");
        let legacy = data_root.join("lumo-desktop/lumo-tamer");
        if current.exists() {
            current
        } else if legacy.exists() {
            legacy
        } else {
            current
        }
    }
}

fn compose_file(dir: &Path) -> Option<PathBuf> {
    for name in ["docker-compose.yml", "compose.yaml", "compose.yml"] {
        let path = dir.join(name);
        if path.is_file() {
            return Some(path);
        }
    }
    None
}

#[tauri::command]
pub async fn tamer_status(app: AppHandle) -> TamerStatus {
    let settings = app.state::<AppState>().store.load_settings();
    let directory = resolve_tamer_directory(&settings);
    let installed = compose_file(&directory).is_some();

    let (docker_available, docker_running, compose_available, git_available) = tokio::join!(
        command_ok("docker", &["--version"]),
        command_ok("docker", &["ps"]),
        command_ok("docker", &["compose", "version"]),
        command_ok("git", &["--version"]),
    );
    let (browser_running, tamer_running) = tokio::join!(
        docker_has_container("lumo-tamer-browser"),
        docker_has_container("lumo-tamer"),
    );

    TamerStatus {
        installed,
        directory: directory.to_string_lossy().to_string(),
        config_exists: directory.join("config.yaml").is_file(),
        secrets_exist: directory.join("secrets/lumo-vault-key").is_file(),
        docker_available,
        docker_running,
        compose_available,
        git_available,
        browser_running,
        tamer_running,
    }
}

#[tauri::command]
pub async fn install_tamer(
    app: AppHandle,
    directory: Option<String>,
    repo: Option<String>,
    language: Option<String>,
) -> Result<InstallResult, String> {
    let lang = if language.as_deref() == Some("fr") {
        "fr"
    } else {
        "en"
    };
    let settings = app.state::<AppState>().store.load_settings();
    let directory = directory
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| resolve_tamer_directory(&settings));
    let repo = repo
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| {
            if settings.tamer_repo.trim().is_empty() {
                DEFAULT_REPO.to_string()
            } else {
                settings.tamer_repo.clone()
            }
        });

    if !repo.starts_with("https://") && !repo.starts_with("http://") {
        return Err(tr(
            lang,
            "L'URL du dépôt lumo-tamer doit être en http(s).",
            "The lumo-tamer repository URL must use http(s).",
        )
        .to_string());
    }

    emit_step(&app, "docker", "running", "");
    if !command_ok("docker", &["--version"]).await {
        emit_step(&app, "docker", "error", "");
        return Err(tr(
            lang,
            "Docker n'est pas installé. Installe Docker Engine ou Docker Desktop, puis réessaie.",
            "Docker is not installed. Install Docker Engine or Docker Desktop, then try again.",
        )
        .to_string());
    }
    if !command_ok("docker", &["ps"]).await {
        emit_step(&app, "docker", "error", "");
        return Err(tr(
            lang,
            "Le démon Docker n'est pas démarré.",
            "The Docker daemon is not running.",
        )
        .to_string());
    }
    if !command_ok("docker", &["compose", "version"]).await {
        emit_step(&app, "docker", "error", "");
        return Err(tr(
            lang,
            "Docker Compose v2 est requis (commande `docker compose`).",
            "Docker Compose v2 is required (`docker compose` command).",
        )
        .to_string());
    }
    if !command_ok("git", &["--version"]).await {
        emit_step(&app, "git", "error", "");
        return Err(tr(
            lang,
            "Git n'est pas installé. Installe git, puis réessaie.",
            "Git is not installed. Install git, then try again.",
        )
        .to_string());
    }
    emit_step(&app, "docker", "done", "");

    if compose_file(&directory).is_none() {
        if directory.exists() {
            let is_empty = fs::read_dir(&directory)
                .map(|mut entries| entries.next().is_none())
                .unwrap_or(false);
            if !is_empty {
                return Err(format!(
                    "{}: {}",
                    tr(
                        lang,
                        "Le dossier existe mais ne contient pas lumo-tamer",
                        "The directory exists but does not contain lumo-tamer"
                    ),
                    directory.display()
                ));
            }
        }

        if let Some(parent) = directory.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "{} {}: {error}",
                    tr(lang, "Impossible de créer", "Cannot create"),
                    parent.display()
                )
            })?;
        }

        emit_step(&app, "clone", "running", "");
        let parent = directory
            .parent()
            .ok_or_else(|| format!("Invalid directory: {}", directory.display()))?;
        let args = vec![
            "clone".to_string(),
            "--depth".to_string(),
            "1".to_string(),
            repo.clone(),
            directory.to_string_lossy().to_string(),
        ];
        run_streaming(&app, parent, "git", &args, "clone", None, Some(Duration::from_secs(600))).await?;
        emit_step(&app, "clone", "done", "");
    }

    if compose_file(&directory).is_none() {
        return Err(format!(
            "{} {}",
            tr(
                lang,
                "Le dépôt cloné ne contient pas de fichier docker compose dans",
                "The cloned repository has no docker compose file in"
            ),
            directory.display()
        ));
    }

    if let Some(compose_path) = compose_file(&directory) {
        if let Ok(mut content) = fs::read_to_string(&compose_path) {
            let replacements = [
                ("\"3003:3003\"", "\"127.0.0.1:3003:3003\""),
                ("\"9222:9222\"", "\"127.0.0.1:9222:9222\""),
                ("\"3001:3001\"", "\"127.0.0.1:3001:3001\""),
            ];
            let mut changed = false;
            for (from, to) in replacements {
                if content.contains(from) {
                    content = content.replace(from, to);
                    changed = true;
                }
            }
            if changed {
                let _ = fs::write(&compose_path, content);
                log(
                    &app,
                    "config",
                    "Restricted lumo-tamer ports to 127.0.0.1 (localhost only).",
                );
            }
        }
    }

    emit_step(&app, "secrets", "running", "");
    ensure_vault_key(&directory)?;
    emit_step(&app, "secrets", "done", "");

    emit_step(&app, "config", "running", "");
    let api_key = ensure_config(&directory, lang)?;
    emit_step(&app, "config", "done", "");

    emit_step(&app, "build", "running", "");
    let build_args = vec![
        "compose".to_string(),
        "build".to_string(),
        "tamer".to_string(),
    ];
    run_streaming(
        &app,
        &directory,
        "docker",
        &build_args,
        "build",
        None,
        Some(Duration::from_secs(1800)),
    )
    .await?;
    emit_step(&app, "build", "done", "");

    emit_step(&app, "browser-build", "running", "");
    let browser_build_args = vec![
        "compose".to_string(),
        "--profile".to_string(),
        "optional".to_string(),
        "build".to_string(),
        "browser".to_string(),
    ];
    run_streaming(
        &app,
        &directory,
        "docker",
        &browser_build_args,
        "browser-build",
        None,
        Some(Duration::from_secs(1800)),
    )
    .await?;
    emit_step(&app, "browser-build", "done", "");

    emit_step(&app, "browser", "running", "");
    let browser_up_args = vec![
        "compose".to_string(),
        "--profile".to_string(),
        "optional".to_string(),
        "up".to_string(),
        "-d".to_string(),
        "browser".to_string(),
    ];
    run_streaming(
        &app,
        &directory,
        "docker",
        &browser_up_args,
        "browser",
        None,
        Some(Duration::from_secs(300)),
    )
    .await?;
    emit_step(&app, "browser", "done", "");

    Ok(InstallResult {
        directory: directory.to_string_lossy().to_string(),
        api_key,
        base_url: BASE_URL.to_string(),
        auth_url: AUTH_URL.to_string(),
    })
}

#[tauri::command]
pub async fn start_auth_browser(
    app: AppHandle,
    directory: Option<String>,
    language: Option<String>,
) -> Result<(), String> {
    let lang = if language.as_deref() == Some("fr") {
        "fr"
    } else {
        "en"
    };
    let settings = app.state::<AppState>().store.load_settings();
    let directory = directory
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| resolve_tamer_directory(&settings));

    if compose_file(&directory).is_none() {
        return Err(tr(
            lang,
            "lumo-tamer n'est pas installé dans ce dossier.",
            "lumo-tamer is not installed in this directory.",
        )
        .to_string());
    }

    emit_step(&app, "browser", "running", "");
    let browser_args = vec![
        "compose".to_string(),
        "--profile".to_string(),
        "optional".to_string(),
        "up".to_string(),
        "-d".to_string(),
        "browser".to_string(),
    ];
    run_streaming(
        &app,
        &directory,
        "docker",
        &browser_args,
        "browser",
        None,
        Some(Duration::from_secs(300)),
    )
    .await?;
    emit_step(&app, "browser", "done", "");
    Ok(())
}

#[tauri::command]
pub async fn authenticate_tamer(
    app: AppHandle,
    directory: Option<String>,
    language: Option<String>,
) -> Result<InstallResult, String> {
    let lang = if language.as_deref() == Some("fr") {
        "fr"
    } else {
        "en"
    };
    let settings = app.state::<AppState>().store.load_settings();
    let directory = directory
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| resolve_tamer_directory(&settings));

    if compose_file(&directory).is_none() {
        return Err(tr(
            lang,
            "lumo-tamer n'est pas installé dans ce dossier.",
            "lumo-tamer is not installed in this directory.",
        )
        .to_string());
    }

    emit_step(&app, "browser", "running", "");
    let browser_args = vec![
        "compose".to_string(),
        "--profile".to_string(),
        "optional".to_string(),
        "up".to_string(),
        "-d".to_string(),
        "browser".to_string(),
    ];
    run_streaming(
        &app,
        &directory,
        "docker",
        &browser_args,
        "browser",
        None,
        Some(Duration::from_secs(300)),
    )
    .await?;
    emit_step(&app, "browser", "done", "");

    emit_step(&app, "auth", "running", "");
    let auth_args = vec![
        "compose".to_string(),
        "run".to_string(),
        "--rm".to_string(),
        "-T".to_string(),
        "-i".to_string(),
        "tamer".to_string(),
        "auth".to_string(),
        "browser".to_string(),
    ];
    run_streaming(
        &app,
        &directory,
        "docker",
        &auth_args,
        "auth",
        Some("\n"),
        Some(Duration::from_secs(600)),
    )
    .await?;
    emit_step(&app, "auth", "done", "");

    emit_step(&app, "start", "running", "");
    let start_args = vec![
        "compose".to_string(),
        "up".to_string(),
        "-d".to_string(),
        "tamer".to_string(),
    ];
    run_streaming(
        &app,
        &directory,
        "docker",
        &start_args,
        "start",
        None,
        Some(Duration::from_secs(300)),
    )
    .await?;

    let client = reqwest::Client::new();
    let mut healthy = false;
    for _ in 0..30 {
        if health_ok(&client).await {
            healthy = true;
            break;
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
    if !healthy {
        emit_step(&app, "start", "error", "");
        return Err(tr(
            lang,
            "lumo-tamer a démarré mais /health ne répond pas encore. Vérifie les logs du conteneur.",
            "lumo-tamer started but /health is not responding yet. Check the container logs.",
        )
        .to_string());
    }
    emit_step(&app, "start", "done", "");

    let api_key = read_api_key(&directory.join("config.yaml")).ok_or_else(|| {
        tr(
            lang,
            "config.yaml ne contient pas server.apiKey.",
            "config.yaml does not contain server.apiKey.",
        )
        .to_string()
    })?;

    Ok(InstallResult {
        directory: directory.to_string_lossy().to_string(),
        api_key,
        base_url: BASE_URL.to_string(),
        auth_url: AUTH_URL.to_string(),
    })
}

#[tauri::command]
pub fn get_web_search(app: AppHandle) -> bool {
    let settings = app.state::<AppState>().store.load_settings();
    let directory = resolve_tamer_directory(&settings);
    let config = directory.join("config.yaml");
    let Ok(content) = fs::read_to_string(config) else {
        return false;
    };
    content.lines().any(|line| {
        line.trim_start()
            .strip_prefix("enableWebSearch:")
            .map(|value| value.trim().eq_ignore_ascii_case("true"))
            .unwrap_or(false)
    })
}

#[tauri::command]
pub async fn set_web_search(
    app: AppHandle,
    enabled: bool,
    language: Option<String>,
) -> Result<bool, String> {
    let lang = if language.as_deref() == Some("fr") {
        "fr"
    } else {
        "en"
    };
    let settings = app.state::<AppState>().store.load_settings();
    let directory = resolve_tamer_directory(&settings);
    if compose_file(&directory).is_none() {
        return Err(tr(
            lang,
            "lumo-tamer n'est pas installé dans ce dossier.",
            "lumo-tamer is not installed in this directory.",
        )
        .to_string());
    }

    let config_path = directory.join("config.yaml");
    let content = fs::read_to_string(&config_path).map_err(|error| {
        format!(
            "{} {}: {error}",
            tr(lang, "Impossible de lire", "Cannot read"),
            config_path.display()
        )
    })?;
    let mut lines: Vec<String> = content.lines().map(str::to_string).collect();
    let mut found = false;
    for line in &mut lines {
        let trimmed = line.trim_start();
        if trimmed.starts_with("enableWebSearch:") {
            let indent = &line[..line.len() - trimmed.len()];
            *line = format!("{indent}enableWebSearch: {enabled}");
            found = true;
        }
    }
    if !found {
        if let Some(position) = lines
            .iter()
            .position(|line| line.trim_start().starts_with("server:"))
        {
            lines.insert(position + 1, format!("  enableWebSearch: {enabled}"));
        } else {
            lines.push("server:".into());
            lines.push(format!("  enableWebSearch: {enabled}"));
        }
    }
    fs::write(&config_path, format!("{}\n", lines.join("\n"))).map_err(|error| {
        format!(
            "{} {}: {error}",
            tr(lang, "Impossible d'écrire", "Cannot write"),
            config_path.display()
        )
    })?;

    let restart_args = vec![
        "compose".to_string(),
        "restart".to_string(),
        "tamer".to_string(),
    ];
    run_streaming(
        &app,
        &directory,
        "docker",
        &restart_args,
        "websearch",
        None,
        Some(Duration::from_secs(180)),
    )
    .await?;

    let client = reqwest::Client::new();
    for _ in 0..30 {
        if health_ok(&client).await {
            return Ok(enabled);
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
    Err(tr(
        lang,
        "La recherche web a été modifiée mais lumo-tamer ne répond pas encore.",
        "Web search was updated but lumo-tamer is not responding yet.",
    )
    .to_string())
}

async fn run_streaming(
    app: &AppHandle,
    dir: &Path,
    program: &str,
    args: &[String],
    step: &str,
    stdin: Option<&str>,
    timeout: Option<Duration>,
) -> Result<(), String> {
    log(
        app,
        step,
        &format!("$ {program} {}", args.join(" ")),
    );

    let mut command = tokio::process::Command::new(program);
    command
        .args(args)
        .current_dir(dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if stdin.is_some() {
        command.stdin(Stdio::piped());
    }

    let mut child = command
        .spawn()
        .map_err(|error| format!("Cannot run {program}: {error}"))?;

    if let Some(input) = stdin {
        if let Some(mut pipe) = child.stdin.take() {
            pipe.write_all(input.as_bytes())
                .await
                .map_err(|error| format!("Cannot write to {program}: {error}"))?;
            drop(pipe);
        }
    }

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| format!("{program}: stdout unavailable"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| format!("{program}: stderr unavailable"))?;

    let app_out = app.clone();
    let step_out = step.to_string();
    let out_task = tauri::async_runtime::spawn(async move {
        let mut lines = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            log(&app_out, &step_out, &strip_ansi(&line));
        }
    });

    let app_err = app.clone();
    let step_err = step.to_string();
    let err_task = tauri::async_runtime::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            log(&app_err, &step_err, &strip_ansi(&line));
        }
    });

    let wait_result = if let Some(duration) = timeout {
        match tokio::time::timeout(duration, child.wait()).await {
            Ok(result) => result,
            Err(_) => {
                let _ = child.kill().await;
                let _ = out_task.await;
                let _ = err_task.await;
                emit_step(app, step, "error", "");
                return Err(format!("{step}: timeout"));
            }
        }
    } else {
        child.wait().await
    };

    let _ = out_task.await;
    let _ = err_task.await;

    let status = wait_result.map_err(|error| format!("{program}: {error}"))?;
    if !status.success() {
        emit_step(app, step, "error", "");
        return Err(format!("{step}: exited with {status}"));
    }
    Ok(())
}

fn ensure_vault_key(directory: &Path) -> Result<(), String> {
    let secrets = directory.join("secrets");
    fs::create_dir_all(&secrets)
        .map_err(|error| format!("Cannot create {}: {error}", secrets.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&secrets, fs::Permissions::from_mode(0o700));
    }

    let key_path = secrets.join("lumo-vault-key");
    if !key_path.exists() {
        let mut bytes = [0u8; 32];
        getrandom::getrandom(&mut bytes)
            .map_err(|error| format!("Cannot generate vault key: {error}"))?;
        let encoded = format!(
            "{}\n",
            base64::engine::general_purpose::STANDARD.encode(bytes)
        );
        fs::write(&key_path, encoded)
            .map_err(|error| format!("Cannot write {}: {error}", key_path.display()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&key_path, fs::Permissions::from_mode(0o600));
        }
    }

    Ok(())
}

fn ensure_config(directory: &Path, lang: &str) -> Result<String, String> {
    let config_path = directory.join("config.yaml");
    if config_path.is_file() {
        if let Some(key) = read_api_key(&config_path) {
            return Ok(key);
        }

        let existing = fs::read_to_string(&config_path)
            .map_err(|error| format!("Cannot read {}: {error}", config_path.display()))?;
        let key = random_api_key();
        let updated = if existing.lines().any(|line| line.trim_start().starts_with("server:")) {
            let mut output = String::new();
            let mut inserted = false;
            for line in existing.lines() {
                output.push_str(line);
                output.push('\n');
                if !inserted && line.trim_start().starts_with("server:") {
                    output.push_str(&format!("  apiKey: \"{key}\"\n"));
                    inserted = true;
                }
            }
            output
        } else {
            format!("{existing}\nserver:\n  apiKey: \"{key}\"\n")
        };
        fs::write(&config_path, updated)
            .map_err(|error| format!("Cannot write {}: {error}", config_path.display()))?;
        set_private_permissions(&config_path);
        return Ok(key);
    }

    let api_key = random_api_key();
    let config = format!(
        r#"auth:
  method: "browser"
  browser:
    cdpEndpoint: "http://browser:9222"

server:
  port: 3003
  apiKey: "{api_key}"
  bodyLimit: "1mb"
  customTools:
    enabled: true

  reasoning:
    default: "none"
    surfaceThinking: true

  log:
    level: "info"
"#
    );
    fs::write(&config_path, config)
        .map_err(|error| format!("Cannot write {}: {error}", config_path.display()))?;
    set_private_permissions(&config_path);
    let _ = lang;
    Ok(api_key)
}

fn read_api_key(path: &Path) -> Option<String> {
    let raw = fs::read_to_string(path).ok()?;
    for line in raw.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("apiKey:") {
            let value = rest.trim().trim_matches('"').trim_matches('\'').trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

fn random_api_key() -> String {
    let mut bytes = [0u8; 32];
    let _ = getrandom::getrandom(&mut bytes);
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn set_private_permissions(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
    }
}

async fn command_ok(program: &str, args: &[&str]) -> bool {
    let result = tokio::time::timeout(
        Duration::from_secs(8),
        tokio::process::Command::new(program)
            .args(args)
            .output(),
    )
    .await;

    matches!(result, Ok(Ok(output)) if output.status.success())
}

async fn docker_has_container(name: &str) -> bool {
    let result = tokio::time::timeout(
        Duration::from_secs(5),
        tokio::process::Command::new("docker")
            .args(["ps", "--format", "{{.Names}}"])
            .output(),
    )
    .await;

    match result {
        Ok(Ok(output)) if output.status.success() => String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line.trim() == name),
        _ => false,
    }
}

async fn health_ok(client: &reqwest::Client) -> bool {
    let Ok(response) = client
        .get(HEALTH_URL)
        .timeout(Duration::from_secs(3))
        .send()
        .await
    else {
        return false;
    };
    if !response.status().is_success() {
        return false;
    }
    let Ok(value) = response.json::<serde_json::Value>().await else {
        return false;
    };
    value.get("status").and_then(|value| value.as_str()) == Some("ok")
        && value
            .get("auth")
            .and_then(|auth| auth.get("valid"))
            .and_then(|valid| valid.as_bool())
            .unwrap_or(false)
}

fn emit_step(app: &AppHandle, step: &str, status: &str, line: &str) {
    let _ = app.emit(
        "tamer-install-event",
        json!({ "step": step, "status": status, "line": line }),
    );
}

fn log(app: &AppHandle, step: &str, line: &str) {
    let _ = app.emit(
        "tamer-install-event",
        json!({ "step": step, "status": "log", "line": line }),
    );
}

fn tr<'a>(lang: &str, french: &'a str, english: &'a str) -> &'a str {
    if lang == "fr" {
        french
    } else {
        english
    }
}

fn strip_ansi(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(character) = chars.next() {
        if character == '\u{1b}' && chars.peek() == Some(&'[') {
            chars.next();
            while let Some(&next) = chars.peek() {
                chars.next();
                if next.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            output.push(character);
        }
    }
    output
}

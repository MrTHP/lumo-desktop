use crate::AppState;
use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::State;
use tokio_util::sync::CancellationToken;

const DEFAULT_IMAGE_BASE_URL: &str = "http://127.0.0.1:8000";
const IMAGE_REQUEST_TIMEOUT: Duration = Duration::from_secs(900);
const MAX_HISTORY: usize = 200;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageStatus {
    pub ok: bool,
    pub base_url: String,
    pub kind: String,
    pub default_family: String,
    pub supported_families: Vec<String>,
    pub healthy: bool,
    pub reason: String,
    pub error: String,
    pub memory_supported: bool,
    pub model_loaded: bool,
    pub allocated_mb: Option<f64>,
    pub reserved_mb: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedImage {
    pub id: String,
    pub prompt: String,
    pub seed: i64,
    pub steps: u32,
    pub guidance: f64,
    pub width: u32,
    pub height: u32,
    #[serde(default)]
    pub backend: String,
    #[serde(default)]
    pub family: String,
    pub created_at: i64,
    pub file_name: String,
    #[serde(default)]
    pub wall_seconds: f64,
    #[serde(default)]
    pub peak_memory_mb: Option<f64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedImageView {
    #[serde(flatten)]
    pub image: GeneratedImage,
    pub thumbnail_url: String,
    pub data_url: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateImageRequest {
    pub request_id: String,
    pub prompt: String,
    #[serde(default)]
    pub seed: i64,
    #[serde(default = "default_steps")]
    pub steps: u32,
    #[serde(default = "default_guidance")]
    pub guidance: f64,
    pub width: u32,
    pub height: u32,
    #[serde(default)]
    pub backend: String,
    #[serde(default)]
    pub family: String,
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub language: String,
}

fn default_steps() -> u32 {
    4
}

fn default_guidance() -> f64 {
    1.0
}

#[tauri::command]
pub async fn image_status(
    state: State<'_, AppState>,
    base_url: String,
) -> Result<ImageStatus, String> {
    let base = normalized_base_url(&base_url);
    let url = format!("{base}/backends");
    let response = state
        .client
        .get(&url)
        .timeout(Duration::from_secs(6))
        .send()
        .await;

    let response = match response {
        Ok(response) => response,
        Err(error) => {
            return Ok(ImageStatus {
                ok: false,
                base_url: base,
                kind: String::new(),
                default_family: String::new(),
                supported_families: Vec::new(),
                healthy: false,
                reason: String::new(),
                error: friendly_network_error(&url, &error),
                memory_supported: false,
                model_loaded: false,
                allocated_mb: None,
                reserved_mb: None,
            });
        }
    };

    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    if !status.is_success() {
        return Ok(ImageStatus {
            ok: false,
            base_url: base,
            kind: String::new(),
            default_family: String::new(),
            supported_families: Vec::new(),
            healthy: false,
            reason: String::new(),
            error: format!("HTTP {}: {}", status.as_u16(), truncate(&body, 300)),
            memory_supported: false,
            model_loaded: false,
            allocated_mb: None,
            reserved_mb: None,
        });
    }

    let value: Value = serde_json::from_str(&body).unwrap_or_else(|_| json!({}));
    let kind = value
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    let default_arm = value
        .get("default")
        .and_then(Value::as_str)
        .or_else(|| value.get("default_family").and_then(Value::as_str))
        .unwrap_or_default();
    let default_family = family_from_arm(default_arm);

    let mut supported: Vec<String> = value
        .get("supported_families")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(|item| family_from_arm(item))
                .filter(|item| !item.is_empty())
                .collect()
        })
        .unwrap_or_default();
    if supported.is_empty() && !default_family.is_empty() {
        supported.push(default_family.clone());
    }
    supported.sort();
    supported.dedup();

    let healthy = value
        .get("healthy")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let reason = value
        .get("reason")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    let memory = match state
        .client
        .get(format!("{base}/memory"))
        .timeout(Duration::from_secs(3))
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => response.json::<Value>().await.ok(),
        _ => None,
    };
    let memory_supported = memory.is_some();
    let model_loaded = memory
        .as_ref()
        .and_then(|value| value.get("loaded"))
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let allocated_mb = memory
        .as_ref()
        .and_then(|value| value.get("allocatedMb"))
        .and_then(Value::as_f64);
    let reserved_mb = memory
        .as_ref()
        .and_then(|value| value.get("reservedMb"))
        .and_then(Value::as_f64);

    Ok(ImageStatus {
        ok: true,
        base_url: base,
        kind,
        default_family,
        supported_families: supported,
        healthy,
        reason,
        error: String::new(),
        memory_supported,
        model_loaded,
        allocated_mb,
        reserved_mb,
    })
}

#[tauri::command]
pub async fn generate_image(
    state: State<'_, AppState>,
    request: GenerateImageRequest,
) -> Result<GeneratedImageView, String> {
    let lang = if request.language == "fr" { "fr" } else { "en" };
    let prompt = request.prompt.trim().to_string();
    if prompt.is_empty() {
        return Err(tr(
            lang,
            "Décris l'image que tu veux générer.",
            "Describe the image you want to generate.",
        )
        .to_string());
    }
    if request.width < 16 || request.height < 16 {
        return Err(tr(
            lang,
            "La largeur et la hauteur doivent être d'au moins 16 px.",
            "Width and height must be at least 16 px.",
        )
        .to_string());
    }

    let request_id = if request.request_id.trim().is_empty() {
        format!("image-{}", now_millis())
    } else {
        request.request_id.trim().to_string()
    };
    let token = CancellationToken::new();
    {
        let mut cancels = state
            .image_cancels
            .lock()
            .map_err(|_| "image cancel lock poisoned".to_string())?;
        cancels.insert(request_id.clone(), token.clone());
    }

    let client = state.client.clone();
    let data_dir = state.store.dir().to_path_buf();
    let result = generate_inner(client, data_dir, &request, prompt, &token, lang).await;

    if let Ok(mut cancels) = state.image_cancels.lock() {
        cancels.remove(&request_id);
    }

    result
}

async fn generate_inner(
    client: reqwest::Client,
    data_dir: PathBuf,
    request: &GenerateImageRequest,
    prompt: String,
    token: &CancellationToken,
    lang: &str,
) -> Result<GeneratedImageView, String> {
    let base = normalized_base_url(&request.base_url);
    let url = format!("{base}/generate");

    let mut body = Map::new();
    body.insert("prompt".into(), json!(prompt));
    body.insert("seed".into(), json!(request.seed));
    body.insert("steps".into(), json!(request.steps.max(1)));
    body.insert("guidance".into(), json!(request.guidance.max(0.0)));
    body.insert("width".into(), json!(request.width));
    body.insert("height".into(), json!(request.height));
    if !request.backend.trim().is_empty() {
        body.insert("backend".into(), json!(request.backend.trim()));
    }

    let response = tokio::select! {
        _ = token.cancelled() => {
            return Err(tr(lang, "Génération annulée.", "Generation cancelled.").to_string());
        }
        result = client
            .post(&url)
            .json(&Value::Object(body))
            .timeout(IMAGE_REQUEST_TIMEOUT)
            .send() => {
            result.map_err(|error| friendly_network_error(&url, &error))?
        }
    };

    let status = response.status();
    let wall_seconds = response
        .headers()
        .get("x-wall-seconds")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(0.0);
    let peak_memory_mb = response
        .headers()
        .get("x-peak-memory-mb")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<f64>().ok());

    let bytes = tokio::select! {
        _ = token.cancelled() => {
            return Err(tr(lang, "Génération annulée.", "Generation cancelled.").to_string());
        }
        result = response.bytes() => {
            result.map_err(|error| format!("{}: {error}", tr(lang, "Lecture de l'image impossible", "Could not read the image")))?
        }
    };

    if !status.is_success() {
        let body = String::from_utf8_lossy(&bytes);
        return Err(format!(
            "{} {}: {}",
            tr(
                lang,
                "Le serveur Bonsai a répondu HTTP",
                "The Bonsai server returned HTTP"
            ),
            status.as_u16(),
            truncate(&body, 700)
        ));
    }
    if bytes.len() < 8 || &bytes[..8] != b"\x89PNG\r\n\x1a\n" {
        return Err(tr(
            lang,
            "Le serveur n'a pas renvoyé une image PNG valide.",
            "The server did not return a valid PNG image.",
        )
        .to_string());
    }

    let id = format!("img-{}-{:08x}", now_millis(), random_u32());
    let file_name = format!("{id}.png");
    let directory = data_dir.join("images");
    fs::create_dir_all(&directory).map_err(|error| {
        format!(
            "{}: {error}",
            tr(
                lang,
                "Création du dossier images impossible",
                "Could not create the images folder"
            )
        )
    })?;

    let image_path = directory.join(&file_name);
    fs::write(&image_path, &bytes).map_err(|error| {
        format!(
            "{}: {error}",
            tr(
                lang,
                "Écriture de l'image impossible",
                "Could not write the image"
            )
        )
    })?;
    set_private_permissions(&image_path);

    let thumbnail_url = write_thumbnail(&directory, &id, &bytes, lang).unwrap_or_default();

    let record = GeneratedImage {
        id,
        prompt,
        seed: request.seed,
        steps: request.steps.max(1),
        guidance: request.guidance.max(0.0),
        width: request.width,
        height: request.height,
        backend: request.backend.trim().to_string(),
        family: request.family.trim().to_string(),
        created_at: now_millis(),
        file_name,
        wall_seconds,
        peak_memory_mb,
    };

    let mut index = read_index(&directory);
    index.retain(|item| item.id != record.id);
    index.insert(0, record.clone());
    index.truncate(MAX_HISTORY);
    write_index(&directory, &index)?;

    let data_url = format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(&bytes)
    );

    Ok(GeneratedImageView {
        image: record,
        thumbnail_url,
        data_url: Some(data_url),
    })
}

#[tauri::command]
pub fn stop_image_generation(state: State<'_, AppState>, request_id: String) -> bool {
    let token = state
        .image_cancels
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
pub async fn unload_image_model(
    state: State<'_, AppState>,
    base_url: String,
    language: String,
) -> Result<Value, String> {
    let lang = if language == "fr" { "fr" } else { "en" };
    let base = normalized_base_url(&base_url);
    let url = format!("{base}/unload");
    let response = state
        .client
        .post(&url)
        .timeout(Duration::from_secs(20))
        .send()
        .await
        .map_err(|error| friendly_network_error(&url, &error))?;
    let status = response.status();
    let body = response.text().await.unwrap_or_default();

    if status == reqwest::StatusCode::NOT_FOUND {
        return Err(tr(
            lang,
            "Ce backend Bonsai ne fournit pas /unload. Mets à jour Bonsai-Image-Demo avec le patch de déchargement fourni par Model Desk, puis relance `./scripts/serve.sh`.",
            "This Bonsai backend does not expose /unload. Update Bonsai-Image-Demo with Model Desk's unload patch, then restart `./scripts/serve.sh`.",
        )
        .to_string());
    }
    if !status.is_success() {
        return Err(format!(
            "{} {}: {}",
            tr(
                lang,
                "Le backend Bonsai a répondu HTTP",
                "The Bonsai backend returned HTTP"
            ),
            status.as_u16(),
            truncate(&body, 700)
        ));
    }

    Ok(serde_json::from_str(&body).unwrap_or_else(|_| json!({ "ok": true })))
}

/// Free the Bonsai model when the desktop app really quits (not when it is
/// hidden in the tray). Uses a short blocking request in a worker thread so
/// the exit path is not blocked by a slow or unreachable backend.
pub fn unload_on_exit(app_handle: &tauri::AppHandle) {
    use tauri::Manager;

    let Some(state) = app_handle.try_state::<AppState>() else {
        return;
    };
    let settings = state.store.load_settings();
    if !settings.image_unload_on_quit {
        return;
    }
    let base = normalized_base_url(&settings.image_base_url);
    let url = format!("{base}/unload");
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        if let Ok(client) = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
        {
            let _ = client.post(&url).send();
        }
        let _ = sender.send(());
    });
    let _ = receiver.recv_timeout(Duration::from_secs(3));
}

#[tauri::command]
pub fn list_generated_images(state: State<'_, AppState>) -> Vec<GeneratedImageView> {
    let directory = state.store.dir().join("images");
    let mut index = read_index(&directory);
    index.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    index
        .into_iter()
        .filter_map(|record| read_view(&directory, record, false).ok())
        .collect()
}

#[tauri::command]
pub fn get_generated_image(
    state: State<'_, AppState>,
    id: String,
) -> Result<GeneratedImageView, String> {
    ensure_valid_id(&id)?;
    let directory = state.store.dir().join("images");
    let index = read_index(&directory);
    let record = index
        .into_iter()
        .find(|item| item.id == id)
        .ok_or_else(|| "Image introuvable".to_string())?;
    read_view(&directory, record, true)
}

#[tauri::command]
pub fn delete_generated_image(state: State<'_, AppState>, id: String) -> Result<(), String> {
    ensure_valid_id(&id)?;
    let directory = state.store.dir().join("images");
    let mut index = read_index(&directory);
    index.retain(|item| item.id != id);
    write_index(&directory, &index)?;
    let _ = fs::remove_file(directory.join(format!("{id}.png")));
    let _ = fs::remove_file(directory.join(format!("{id}.thumb.png")));
    Ok(())
}

#[tauri::command]
pub fn clear_generated_images(state: State<'_, AppState>) -> Result<(), String> {
    let directory = state.store.dir().join("images");
    let index = read_index(&directory);
    for record in &index {
        let _ = fs::remove_file(directory.join(&record.file_name));
        let _ = fs::remove_file(directory.join(format!("{}.thumb.png", record.id)));
    }
    write_index(&directory, &[])?;
    Ok(())
}

fn write_thumbnail(directory: &Path, id: &str, bytes: &[u8], lang: &str) -> Result<String, String> {
    let image = image::load_from_memory(bytes).map_err(|error| {
        format!(
            "{}: {error}",
            tr(lang, "Miniature impossible", "Could not build thumbnail")
        )
    })?;
    let thumbnail = image.thumbnail(360, 360);
    let mut buffer = Vec::new();
    thumbnail
        .write_to(&mut Cursor::new(&mut buffer), image::ImageFormat::Png)
        .map_err(|error| {
            format!(
                "{}: {error}",
                tr(
                    lang,
                    "Encodage de la miniature impossible",
                    "Could not encode the thumbnail"
                )
            )
        })?;
    fs::write(directory.join(format!("{id}.thumb.png")), &buffer).map_err(|error| {
        format!(
            "{}: {error}",
            tr(
                lang,
                "Écriture de la miniature impossible",
                "Could not write the thumbnail"
            )
        )
    })?;
    set_private_permissions(&directory.join(format!("{id}.thumb.png")));
    Ok(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(&buffer)
    ))
}

fn read_view(
    directory: &Path,
    record: GeneratedImage,
    include_data: bool,
) -> Result<GeneratedImageView, String> {
    let thumbnail = fs::read(directory.join(format!("{}.thumb.png", record.id))).ok();
    let thumbnail_url = thumbnail
        .map(|bytes| {
            format!(
                "data:image/png;base64,{}",
                base64::engine::general_purpose::STANDARD.encode(bytes)
            )
        })
        .unwrap_or_default();
    let data_url = if include_data {
        fs::read(directory.join(&record.file_name))
            .ok()
            .map(|bytes| {
                format!(
                    "data:image/png;base64,{}",
                    base64::engine::general_purpose::STANDARD.encode(bytes)
                )
            })
    } else {
        None
    };
    Ok(GeneratedImageView {
        image: record,
        thumbnail_url,
        data_url,
    })
}

fn read_index(directory: &Path) -> Vec<GeneratedImage> {
    fs::read_to_string(directory.join("index.json"))
        .ok()
        .and_then(|raw| serde_json::from_str::<Vec<GeneratedImage>>(&raw).ok())
        .unwrap_or_default()
}

fn write_index(directory: &Path, index: &[GeneratedImage]) -> Result<(), String> {
    fs::create_dir_all(directory).map_err(|error| error.to_string())?;
    let bytes = serde_json::to_vec_pretty(index).map_err(|error| error.to_string())?;
    let tmp = directory.join("index.json.tmp");
    fs::write(&tmp, bytes).map_err(|error| error.to_string())?;
    set_private_permissions(&tmp);
    fs::rename(&tmp, directory.join("index.json")).map_err(|error| error.to_string())
}

fn set_private_permissions(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
    }
}

fn ensure_valid_id(id: &str) -> Result<(), String> {
    let valid = !id.is_empty()
        && id.len() <= 96
        && id.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '-' || character == '_'
        });
    if valid {
        Ok(())
    } else {
        Err("Identifiant d'image invalide".into())
    }
}

fn normalized_base_url(value: &str) -> String {
    let trimmed = value.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        DEFAULT_IMAGE_BASE_URL.to_string()
    } else {
        trimmed.to_string()
    }
}

fn family_from_arm(arm: &str) -> String {
    let trimmed = arm.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    trimmed
        .strip_suffix("-gemlite")
        .or_else(|| trimmed.strip_suffix("-mlx"))
        .unwrap_or(trimmed)
        .to_string()
}

fn friendly_network_error(url: &str, error: &reqwest::Error) -> String {
    if error.is_connect() {
        format!(
            "Impossible de joindre le serveur Bonsai sur {url}. Lance `./scripts/serve.sh` dans le dossier Bonsai-Image-Demo (ou vérifie l'URL dans Paramètres → Images). / Cannot reach the Bonsai server at {url}. Run `./scripts/serve.sh` in Bonsai-Image-Demo (or check the URL in Settings → Images)."
        )
    } else if error.is_timeout() {
        format!(
            "Le serveur Bonsai ne répond pas sur {url}. / The Bonsai server did not respond at {url}."
        )
    } else {
        format!("Erreur réseau vers {url}: {error} / Network error reaching {url}: {error}")
    }
}

fn tr<'a>(lang: &str, french: &'a str, english: &'a str) -> &'a str {
    if lang == "fr" {
        french
    } else {
        english
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

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}

fn random_u32() -> u32 {
    let mut bytes = [0u8; 4];
    if getrandom::getrandom(&mut bytes).is_err() {
        return (now_millis() as u32) ^ 0x9e37_79b9;
    }
    u32::from_le_bytes(bytes)
}

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub instructions: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub created_at: i64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Memory {
    pub id: String,
    pub text: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub created_at: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub thinking: bool,
    pub system_prompt: String,
    pub theme: String,
    pub language: String,
    pub tamer_dir: String,
    pub tamer_repo: String,
    pub detected_from: Option<String>,
    pub tools_enabled: bool,
    pub allow_read: bool,
    pub allow_write: bool,
    pub allow_shell: bool,
    pub confirm_writes: bool,
    pub confirm_shell: bool,
    pub workspace: String,
    pub profile_name: String,
    pub profile_occupation: String,
    pub profile_context: String,
    pub profile_behavior: String,
    pub auto_memory: bool,
    pub projects: Vec<Project>,
    pub memories: Vec<Memory>,
    pub enable_web_search: bool,
    pub past_chats_access: String,
    pub project_files_access: String,
    pub creations_enabled: bool,
    pub visualizations_enabled: bool,
}

impl Default for Settings {
    fn default() -> Self {
        let mut settings = Settings {
            base_url: "http://127.0.0.1:3003/v1".into(),
            api_key: String::new(),
            model: "lumo".into(),
            thinking: false,
            system_prompt: String::new(),
            theme: "dark".into(),
            language: "auto".into(),
            tamer_dir: default_tamer_dir().to_string_lossy().to_string(),
            tamer_repo: "https://github.com/ZeroTricks/lumo-tamer.git".into(),
            detected_from: None,
            tools_enabled: false,
            allow_read: true,
            allow_write: true,
            allow_shell: false,
            confirm_writes: true,
            confirm_shell: true,
            workspace: dirs::home_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            profile_name: String::new(),
            profile_occupation: String::new(),
            profile_context: String::new(),
            profile_behavior: String::new(),
            auto_memory: false,
            projects: Vec::new(),
            memories: Vec::new(),
            enable_web_search: false,
            past_chats_access: "all".into(),
            project_files_access: "current".into(),
            creations_enabled: true,
            visualizations_enabled: true,
        };

        if let Some(path) = opencode_config_path() {
            if let Some((base_url, api_key)) = read_opencode_provider(&path) {
                settings.base_url = base_url;
                settings.api_key = api_key;
                settings.detected_from = Some(path.to_string_lossy().to_string());
            }
        }

        if let Ok(url) = std::env::var("LUMO_TAMER_URL") {
            if !url.trim().is_empty() {
                settings.base_url = url;
            }
        }
        if let Ok(key) = std::env::var("LUMO_TAMER_API_KEY") {
            if !key.trim().is_empty() {
                settings.api_key = key;
            }
        }

        settings.base_url = normalize_base_url(&settings.base_url);
        settings
    }
}

pub struct Store {
    dir: PathBuf,
}

impl Store {
    pub fn new(dir: PathBuf) -> Self {
        let _ = fs::create_dir_all(&dir);
        Self { dir }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    fn settings_path(&self) -> PathBuf {
        self.dir.join("settings.json")
    }

    fn conversations_path(&self) -> PathBuf {
        self.dir.join("conversations.json")
    }

    pub fn load_settings(&self) -> Settings {
        let path = self.settings_path();
        let mut settings = match fs::read_to_string(&path) {
            Ok(raw) => serde_json::from_str::<Settings>(&raw).unwrap_or_default(),
            Err(_) => Settings::default(),
        };

        // A settings file created before the OpenCode detection existed has an
        // empty key. Try to fill it in on first load so the app works out of
        // the box for lumo-tamer users.
        if settings.api_key.trim().is_empty() {
            if let Some((base_url, api_key)) = detect_from_opencode() {
                if settings.base_url.trim().is_empty()
                    || settings.base_url == "http://127.0.0.1:3003/v1"
                {
                    settings.base_url = base_url;
                }
                settings.api_key = api_key;
            }
        }

        settings.base_url = normalize_base_url(&settings.base_url);
        settings
    }

    pub fn save_settings(&self, settings: &Settings) -> Result<(), String> {
        write_private_json(&self.settings_path(), &serde_json::to_vec_pretty(settings).map_err(|e| e.to_string())?)
    }

    pub fn load_conversations(&self) -> Value {
        match fs::read_to_string(self.conversations_path()) {
            Ok(raw) => serde_json::from_str::<Value>(&raw).unwrap_or_else(|_| Value::Array(vec![])),
            Err(_) => Value::Array(vec![]),
        }
    }

    pub fn save_conversations(&self, conversations: &Value) -> Result<(), String> {
        if !conversations.is_array() {
            return Err("conversations must be a JSON array".into());
        }
        write_private_json(
            &self.conversations_path(),
            &serde_json::to_vec(conversations).map_err(|e| e.to_string())?,
        )
    }
}

fn write_private_json(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, bytes).map_err(|e| format!("écriture {}: {e}", tmp.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&tmp, fs::Permissions::from_mode(0o600));
    }

    fs::rename(&tmp, path).map_err(|e| format!("renommage {}: {e}", path.display()))
}

pub fn normalize_base_url(url: &str) -> String {
    let mut url = url.trim().trim_end_matches('/').to_string();
    if url.is_empty() {
        url = "http://127.0.0.1:3003/v1".into();
    }
    if !url.ends_with("/v1") && !url.contains("/v1/") {
        url.push_str("/v1");
    }
    url
}

fn default_tamer_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let projects = home.join("Projects/lumo-tamer");
    if projects.join("docker-compose.yml").exists() || projects.join("compose.yaml").exists() {
        projects
    } else {
        dirs::data_local_dir()
            .unwrap_or_else(|| home.join(".local/share"))
            .join("lumo-desktop/lumo-tamer")
    }
}

pub fn system_language() -> &'static str {
    match sys_locale::get_locale() {
        Some(locale) if locale.to_ascii_lowercase().starts_with("fr") => "fr",
        _ => "en",
    }
}

pub fn effective_language(settings: &Settings) -> &'static str {
    match settings.language.as_str() {
        "fr" => "fr",
        "en" => "en",
        _ => system_language(),
    }
}

pub fn origin_from_base_url(base_url: &str) -> String {
    base_url
        .trim_end_matches('/')
        .trim_end_matches("/v1")
        .trim_end_matches('/')
        .to_string()
}

fn opencode_config_path() -> Option<PathBuf> {
    let dir = dirs::config_dir()?.join("opencode");
    for name in ["opencode.jsonc", "opencode.json"] {
        let path = dir.join(name);
        if path.is_file() {
            return Some(path);
        }
    }
    None
}

fn detect_from_opencode() -> Option<(String, String)> {
    let path = opencode_config_path()?;
    read_opencode_provider(&path)
}

fn read_opencode_provider(path: &Path) -> Option<(String, String)> {
    let raw = fs::read_to_string(path).ok()?;
    let cleaned = strip_jsonc(&raw);
    let config: Value = serde_json::from_str(&cleaned).ok()?;
    let providers = config.get("providers")?.as_object()?;

    let mut fallback = None;
    for (key, provider) in providers {
        let settings = provider.get("settings");
        let base_url = settings
            .and_then(|s| s.get("baseURL").or_else(|| s.get("baseUrl")))
            .and_then(Value::as_str)
            .unwrap_or_default();
        let api_key = settings
            .and_then(|s| s.get("apiKey"))
            .and_then(Value::as_str)
            .unwrap_or_default();

        if base_url.is_empty() {
            continue;
        }

        let candidate = (normalize_base_url(base_url), api_key.to_string());
        let key_lower = key.to_ascii_lowercase();
        let name_lower = provider
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_ascii_lowercase();

        if key_lower.contains("lumo") || name_lower.contains("lumo") {
            return Some(candidate);
        }
        if fallback.is_none() && base_url.contains("3003") {
            fallback = Some(candidate);
        }
    }

    fallback
}

/// Minimal JSONC preprocessor: removes // and /* */ comments outside strings.
fn strip_jsonc(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    let mut in_string = false;
    let mut escaped = false;

    while i < bytes.len() {
        let c = bytes[i] as char;
        if in_string {
            out.push(c);
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
            i += 1;
            continue;
        }

        if c == '"' {
            in_string = true;
            out.push(c);
            i += 1;
            continue;
        }

        if c == '/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }

        if c == '/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(bytes.len());
            continue;
        }

        out.push(c);
        i += 1;
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_comments_but_not_urls() {
        let input = r#"
        {
          // comment
          "url": "http://127.0.0.1:3003/v1", /* block */
          "nested": { "a": 1 }
        }
        "#;
        let parsed: Value = serde_json::from_str(&strip_jsonc(input)).unwrap();
        assert_eq!(parsed["url"], "http://127.0.0.1:3003/v1");
    }

    #[test]
    fn normalizes_base_urls() {
        assert_eq!(
            normalize_base_url("http://127.0.0.1:3003"),
            "http://127.0.0.1:3003/v1"
        );
        assert_eq!(
            normalize_base_url("http://127.0.0.1:3003/v1/"),
            "http://127.0.0.1:3003/v1"
        );
        assert_eq!(
            origin_from_base_url("http://127.0.0.1:3003/v1"),
            "http://127.0.0.1:3003"
        );
    }
}

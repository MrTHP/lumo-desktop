use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
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
pub struct Provider {
    pub id: String,
    pub name: String,
    /// "lumo", "deepseek" or "openai".
    pub kind: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub models: Vec<String>,
    pub supports_thinking: bool,
    pub supports_tools: bool,
}

impl Default for Provider {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            kind: "openai".into(),
            base_url: String::new(),
            api_key: String::new(),
            model: String::new(),
            models: Vec::new(),
            supports_thinking: false,
            supports_tools: true,
        }
    }
}

pub const LUMO_BASE_URL: &str = "http://127.0.0.1:3003/v1";
pub const DEEPSEEK_BASE_URL: &str = "https://api.deepseek.com/v1";
pub const DEEPSEEK_MODELS: [&str; 2] = ["deepseek-flash", "deepseek-v4-pro"];

impl Provider {
    pub fn lumo(base_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            id: "lumo".into(),
            name: "Lumo (Proton local)".into(),
            kind: "lumo".into(),
            base_url: normalize_base_url(&base_url.into()),
            api_key: api_key.into(),
            model: "lumo".into(),
            models: vec!["lumo".into(), "lumo-lite".into(), "lumo-max".into()],
            supports_thinking: true,
            supports_tools: true,
        }
    }

    pub fn deepseek(api_key: impl Into<String>) -> Self {
        Self {
            id: "deepseek".into(),
            name: "DeepSeek".into(),
            kind: "deepseek".into(),
            base_url: DEEPSEEK_BASE_URL.into(),
            api_key: api_key.into(),
            model: "deepseek-flash".into(),
            models: DEEPSEEK_MODELS.iter().map(|id| (*id).to_string()).collect(),
            supports_thinking: true,
            supports_tools: true,
        }
    }
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
    pub providers: Vec<Provider>,
    pub active_provider: String,
    pub opencode_binary: String,
    pub opencode_workspace: String,
    pub opencode_model: String,
    pub opencode_agent: String,
    pub opencode_auto: bool,
}

impl Default for Settings {
    fn default() -> Self {
        let home = dirs::home_dir()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let detected = opencode_config_path()
            .and_then(|path| read_opencode_provider(&path).map(|pair| (path, pair)));
        let (detected_from, mut lumo_base, mut lumo_key) = match detected {
            Some((path, (base_url, api_key))) => {
                (Some(path.to_string_lossy().to_string()), base_url, api_key)
            }
            None => (None, LUMO_BASE_URL.to_string(), String::new()),
        };

        if let Ok(url) = std::env::var("LUMO_TAMER_URL") {
            if !url.trim().is_empty() {
                lumo_base = url;
            }
        }
        if let Ok(key) = std::env::var("LUMO_TAMER_API_KEY") {
            if !key.trim().is_empty() {
                lumo_key = key;
            }
        }

        let deepseek_key = std::env::var("DEEPSEEK_API_KEY")
            .ok()
            .filter(|key| !key.trim().is_empty())
            .or_else(|| read_opencode_auth_key("deepseek"))
            .unwrap_or_default();

        let mut providers = vec![
            Provider::lumo(lumo_base.clone(), lumo_key.clone()),
            Provider::deepseek(deepseek_key.clone()),
        ];
        for provider in import_opencode_providers() {
            merge_provider(&mut providers, provider);
        }

        let opencode_model = if deepseek_key.trim().is_empty() {
            "lumo-tamer/lumo".to_string()
        } else {
            "deepseek/deepseek-flash".to_string()
        };

        let mut settings = Settings {
            base_url: lumo_base,
            api_key: lumo_key,
            model: "lumo".into(),
            thinking: false,
            system_prompt: String::new(),
            theme: "dark".into(),
            language: "auto".into(),
            tamer_dir: default_tamer_dir().to_string_lossy().to_string(),
            tamer_repo: "https://github.com/ZeroTricks/lumo-tamer.git".into(),
            detected_from,
            tools_enabled: false,
            allow_read: true,
            allow_write: true,
            allow_shell: false,
            confirm_writes: true,
            confirm_shell: true,
            workspace: home.clone(),
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
            providers,
            active_provider: "lumo".into(),
            opencode_binary: "opencode".into(),
            opencode_workspace: home,
            opencode_model,
            opencode_agent: "build".into(),
            opencode_auto: true,
        };

        settings.normalize_providers();
        settings.sync_legacy_from_active();
        settings
    }
}

impl Settings {
    pub fn provider(&self) -> Option<&Provider> {
        self.providers
            .iter()
            .find(|provider| provider.id == self.active_provider)
            .or_else(|| self.providers.first())
    }

    pub fn sync_legacy_from_active(&mut self) {
        if let Some(provider) = self.provider().cloned() {
            self.base_url = provider.base_url;
            self.api_key = provider.api_key;
            self.model = provider.model;
        }
    }

    pub fn normalize_providers(&mut self) {
        if self.providers.is_empty() {
            let detected = opencode_config_path()
                .and_then(|path| read_opencode_provider(&path).map(|pair| (path, pair)));
            let (detected_from, lumo_base, lumo_key) = match detected {
                Some((path, (base_url, api_key))) => {
                    (Some(path.to_string_lossy().to_string()), base_url, api_key)
                }
                None => (None, LUMO_BASE_URL.to_string(), String::new()),
            };
            if self.detected_from.is_none() {
                self.detected_from = detected_from;
            }
            let base = if self.base_url.trim().is_empty() {
                lumo_base
            } else {
                self.base_url.clone()
            };
            let key = if self.api_key.trim().is_empty() {
                lumo_key
            } else {
                self.api_key.clone()
            };
            self.providers.push(Provider::lumo(base, key));
            let deepseek_key = std::env::var("DEEPSEEK_API_KEY")
                .ok()
                .filter(|value| !value.trim().is_empty())
                .or_else(|| read_opencode_auth_key("deepseek"))
                .unwrap_or_default();
            self.providers.push(Provider::deepseek(deepseek_key));
        }

        let mut seen = HashSet::new();
        for provider in &mut self.providers {
            if provider.id.trim().is_empty() {
                provider.id = sanitize_id(if provider.name.trim().is_empty() {
                    "provider"
                } else {
                    &provider.name
                });
            }
            let base_id = provider.id.clone();
            let mut suffix = 2;
            while !seen.insert(provider.id.clone()) {
                provider.id = format!("{base_id}-{suffix}");
                suffix += 1;
            }

            if provider.name.trim().is_empty() {
                provider.name = provider.id.clone();
            }
            provider.kind = provider.kind.trim().to_ascii_lowercase();
            if provider.kind.is_empty() {
                provider.kind = "openai".into();
            }
            if provider.base_url.trim().is_empty() {
                provider.base_url = match provider.kind.as_str() {
                    "lumo" => LUMO_BASE_URL.to_string(),
                    "deepseek" => DEEPSEEK_BASE_URL.to_string(),
                    _ => String::new(),
                };
            }
            if !provider.base_url.trim().is_empty() {
                provider.base_url = normalize_base_url(&provider.base_url);
            }

            match provider.kind.as_str() {
                "lumo" => {
                    provider.supports_thinking = true;
                    provider.supports_tools = true;
                    if provider.model.trim().is_empty() {
                        provider.model = "lumo".into();
                    }
                    if provider.models.is_empty() {
                        provider.models =
                            vec!["lumo".into(), "lumo-lite".into(), "lumo-max".into()];
                    }
                }
                "deepseek" => {
                    provider.supports_thinking = true;
                    provider.supports_tools = true;
                    if provider.model.trim().is_empty() {
                        provider.model = "deepseek-flash".into();
                    }
                    if provider.models.is_empty() {
                        provider.models =
                            DEEPSEEK_MODELS.iter().map(|id| (*id).to_string()).collect();
                    }
                }
                _ => {}
            }

            if provider.models.is_empty() && !provider.model.trim().is_empty() {
                provider.models = vec![provider.model.clone()];
            }
        }

        if let Some(deepseek) = self
            .providers
            .iter_mut()
            .find(|provider| provider.kind == "deepseek")
        {
            if deepseek.api_key.trim().is_empty() {
                if let Some(key) = std::env::var("DEEPSEEK_API_KEY")
                    .ok()
                    .filter(|value| !value.trim().is_empty())
                    .or_else(|| read_opencode_auth_key("deepseek"))
                {
                    deepseek.api_key = key;
                }
            }
        }

        if !self
            .providers
            .iter()
            .any(|provider| provider.id == self.active_provider)
        {
            self.active_provider = self
                .providers
                .iter()
                .find(|provider| provider.kind == "lumo")
                .or_else(|| self.providers.first())
                .map(|provider| provider.id.clone())
                .unwrap_or_default();
        }

        if self.opencode_binary.trim().is_empty() {
            self.opencode_binary = "opencode".into();
        }
        if self.opencode_agent.trim().is_empty() {
            self.opencode_agent = "build".into();
        }
        if self.opencode_workspace.trim().is_empty() {
            self.opencode_workspace = if self.workspace.trim().is_empty() {
                dirs::home_dir()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            } else {
                self.workspace.clone()
            };
        }
        if self.opencode_model.trim().is_empty() {
            self.opencode_model =
                if self.providers.iter().any(|provider| {
                    provider.kind == "deepseek" && !provider.api_key.trim().is_empty()
                }) {
                    "deepseek/deepseek-flash".into()
                } else {
                    "lumo-tamer/lumo".into()
                };
        }
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

        // Migrate settings written by older versions (one flat provider) and
        // fill in keys discovered in OpenCode when the user has not set one.
        settings.normalize_providers();
        settings.sync_legacy_from_active();
        settings
    }

    pub fn save_settings(&self, settings: &mut Settings) -> Result<(), String> {
        settings.normalize_providers();
        settings.sync_legacy_from_active();
        write_private_json(
            &self.settings_path(),
            &serde_json::to_vec_pretty(settings).map_err(|e| e.to_string())?,
        )
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

fn opencode_auth_path() -> Option<PathBuf> {
    dirs::data_local_dir().map(|dir| dir.join("opencode/auth.json"))
}

fn read_opencode_auth_key(provider_id: &str) -> Option<String> {
    let raw = fs::read_to_string(opencode_auth_path()?).ok()?;
    let value: Value = serde_json::from_str(&raw).ok()?;
    let entry = value.get(provider_id)?;
    let key = entry
        .get("key")
        .and_then(Value::as_str)
        .or_else(|| entry.get("apiKey").and_then(Value::as_str))?;
    if key.trim().is_empty() {
        None
    } else {
        Some(key.to_string())
    }
}

fn sanitize_id(raw: &str) -> String {
    let mut out = String::new();
    for character in raw.chars() {
        if character.is_ascii_alphanumeric() {
            out.push(character.to_ascii_lowercase());
        } else if character == '-' || character == '_' {
            out.push(character);
        }
    }
    if out.is_empty() {
        "provider".into()
    } else {
        out
    }
}

fn merge_provider(providers: &mut Vec<Provider>, candidate: Provider) {
    let existing = providers.iter_mut().find(|provider| {
        provider.id == candidate.id
            || (candidate.kind != "openai" && provider.kind == candidate.kind)
    });

    if let Some(existing) = existing {
        if existing.name.trim().is_empty() {
            existing.name = candidate.name;
        }
        if existing.base_url.trim().is_empty() {
            existing.base_url = candidate.base_url;
        }
        if existing.api_key.trim().is_empty() {
            existing.api_key = candidate.api_key;
        }
        if existing.model.trim().is_empty() {
            existing.model = candidate.model;
        }
        for model in candidate.models {
            if !existing.models.contains(&model) {
                existing.models.push(model);
            }
        }
        existing.supports_thinking |= candidate.supports_thinking;
        existing.supports_tools |= candidate.supports_tools;
    } else {
        providers.push(candidate);
    }
}

pub fn import_opencode_providers() -> Vec<Provider> {
    let mut imported = Vec::new();
    let providers = opencode_config_path()
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|raw| serde_json::from_str::<Value>(&strip_jsonc(&raw)).ok())
        .and_then(|config| {
            config
                .get("providers")
                .or_else(|| config.get("provider"))
                .cloned()
        })
        .and_then(|providers| providers.as_object().cloned());

    if let Some(providers) = providers {
        for (key, provider) in providers {
            let settings = provider.get("settings");
            let base_url = settings
                .and_then(|value| value.get("baseURL").or_else(|| value.get("baseUrl")))
                .and_then(Value::as_str)
                .unwrap_or_default();
            if base_url.trim().is_empty() {
                continue;
            }

            let api_key = settings
                .and_then(|value| value.get("apiKey"))
                .and_then(Value::as_str)
                .unwrap_or_default();
            let name = provider.get("name").and_then(Value::as_str).unwrap_or(&key);
            let identity = format!("{} {}", key.to_ascii_lowercase(), name.to_ascii_lowercase());
            let kind = if identity.contains("lumo") {
                "lumo"
            } else if identity.contains("deepseek") {
                "deepseek"
            } else {
                "openai"
            };
            let models = provider
                .get("models")
                .and_then(Value::as_object)
                .map(|models| models.keys().cloned().collect::<Vec<_>>())
                .unwrap_or_default();
            let model = models.first().cloned().unwrap_or_default();

            imported.push(Provider {
                id: sanitize_id(&key),
                name: name.to_string(),
                kind: kind.into(),
                base_url: normalize_base_url(base_url),
                api_key: api_key.to_string(),
                model,
                models,
                supports_thinking: true,
                supports_tools: true,
            });
        }
    }

    if !imported.iter().any(|provider| provider.kind == "deepseek") {
        if let Some(key) = read_opencode_auth_key("deepseek") {
            imported.push(Provider::deepseek(key));
        }
    }

    imported
}

fn read_opencode_provider(path: &Path) -> Option<(String, String)> {
    let raw = fs::read_to_string(path).ok()?;
    let cleaned = strip_jsonc(&raw);
    let config: Value = serde_json::from_str(&cleaned).ok()?;
    let providers = config
        .get("providers")
        .or_else(|| config.get("provider"))?
        .as_object()?;

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

    #[test]
    fn normalizes_deepseek_provider_and_active_fallback() {
        let mut settings = Settings::default();
        settings.providers = vec![Provider {
            id: "deepseek".into(),
            kind: "deepseek".into(),
            ..Default::default()
        }];
        settings.active_provider = "missing".into();

        settings.normalize_providers();

        assert_eq!(settings.active_provider, "deepseek");
        assert_eq!(settings.providers[0].base_url, DEEPSEEK_BASE_URL);
        assert_eq!(settings.providers[0].model, "deepseek-flash");
        assert_eq!(settings.providers[0].models.len(), 2);
    }

    #[test]
    fn syncs_legacy_fields_from_active_provider() {
        let mut settings = Settings::default();
        settings.providers = vec![Provider::deepseek("secret")];
        settings.active_provider = "deepseek".into();

        settings.sync_legacy_from_active();

        assert_eq!(settings.base_url, DEEPSEEK_BASE_URL);
        assert_eq!(settings.api_key, "secret");
        assert_eq!(settings.model, "deepseek-flash");
    }
}

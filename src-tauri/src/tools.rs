use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use walkdir::WalkDir;

const MAX_TOOL_CHARS: usize = 40_000;
const MAX_SEARCH_MATCHES: usize = 200;

pub struct ToolResult {
    pub ok: bool,
    pub content: String,
}

impl ToolResult {
    fn ok(content: impl Into<String>) -> Self {
        Self {
            ok: true,
            content: content.into(),
        }
    }

    fn error(content: impl Into<String>) -> Self {
        Self {
            ok: false,
            content: content.into(),
        }
    }
}

pub fn definitions(allow_read: bool, allow_write: bool, allow_shell: bool) -> Vec<Value> {
    let mut tools = Vec::new();

    if allow_read {
        tools.push(json!({
            "type": "function",
            "function": {
                "name": "read_file",
                "description": "Read a UTF-8 text file from the local filesystem. Returns numbered lines. Use absolute paths inside the authorized workspace.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Absolute path of the file to read" }
                    },
                    "required": ["path"]
                }
            }
        }));
        tools.push(json!({
            "type": "function",
            "function": {
                "name": "list_directory",
                "description": "List the entries of a local directory. Returns type, size and name for each entry.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Absolute path of the directory to list" }
                    },
                    "required": ["path"]
                }
            }
        }));
        tools.push(json!({
            "type": "function",
            "function": {
                "name": "search_files",
                "description": "Search for a case-insensitive text pattern inside files under a directory. Skips .git, node_modules, target and .cache.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Absolute directory to search in" },
                        "pattern": { "type": "string", "description": "Text pattern to look for" },
                        "glob": { "type": "string", "description": "Optional filename filter such as *.rs or *.md" }
                    },
                    "required": ["path", "pattern"]
                }
            }
        }));
    }

    if allow_write {
        tools.push(json!({
            "type": "function",
            "function": {
                "name": "write_file",
                "description": "Create or overwrite a UTF-8 text file. Parent directories are created when needed.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Absolute path of the file to write" },
                        "content": { "type": "string", "description": "Complete new content of the file" }
                    },
                    "required": ["path", "content"]
                }
            }
        }));
        tools.push(json!({
            "type": "function",
            "function": {
                "name": "edit_file",
                "description": "Replace an exact string inside a UTF-8 text file. By default only the first occurrence is replaced.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Absolute path of the file to edit" },
                        "old_string": { "type": "string", "description": "Exact text to replace" },
                        "new_string": { "type": "string", "description": "Replacement text" },
                        "replace_all": { "type": "boolean", "description": "Replace every occurrence instead of only the first" }
                    },
                    "required": ["path", "old_string", "new_string"]
                }
            }
        }));
    }

    if allow_shell {
        tools.push(json!({
            "type": "function",
            "function": {
                "name": "run_command",
                "description": "Run a shell command on the user's computer and return its output. Commands run with the user's permissions inside the authorized workspace.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "command": { "type": "string", "description": "Shell command to execute" },
                        "cwd": { "type": "string", "description": "Optional working directory inside the authorized workspace" }
                    },
                    "required": ["command"]
                }
            }
        }));
    }

    tools
}

pub fn category(name: &str) -> Option<&'static str> {
    match name {
        "read_file" | "list_directory" | "search_files" => Some("read"),
        "write_file" | "edit_file" => Some("write"),
        "run_command" => Some("shell"),
        _ => None,
    }
}

pub fn summarize(name: &str, args: &Value) -> String {
    match name {
        "read_file" | "write_file" | "edit_file" | "list_directory" => args
            .get("path")
            .and_then(Value::as_str)
            .unwrap_or("(missing path)")
            .to_string(),
        "search_files" => format!(
            "{}  ·  {}",
            args.get("pattern").and_then(Value::as_str).unwrap_or("?"),
            args.get("path").and_then(Value::as_str).unwrap_or("?")
        ),
        "run_command" => args
            .get("command")
            .and_then(Value::as_str)
            .unwrap_or("(missing command)")
            .to_string(),
        _ => name.to_string(),
    }
}

pub fn preview(args: &Value) -> String {
    let text = serde_json::to_string_pretty(args).unwrap_or_else(|_| args.to_string());
    truncate(&text, 4000)
}

pub fn canonical_workspace(raw: &str) -> PathBuf {
    let fallback = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
    let candidate = if raw.trim().is_empty() {
        fallback.clone()
    } else {
        PathBuf::from(raw.trim())
    };

    match candidate.canonicalize() {
        Ok(path) if path.is_dir() => path,
        _ => fallback.canonicalize().unwrap_or(fallback),
    }
}

pub async fn execute(
    name: &str,
    args: &Value,
    workspace: &Path,
    token: &CancellationToken,
) -> ToolResult {
    match name {
        "read_file" => read_file(args, workspace),
        "list_directory" => list_directory(args, workspace),
        "search_files" => search_files(args, workspace),
        "write_file" => write_file(args, workspace),
        "edit_file" => edit_file(args, workspace),
        "run_command" => run_command(args, workspace, token).await,
        _ => ToolResult::error(format!("Unknown tool: {name}")),
    }
}

fn read_file(args: &Value, workspace: &Path) -> ToolResult {
    let raw_path = match string_arg(args, "path") {
        Ok(value) => value,
        Err(error) => return ToolResult::error(error),
    };
    let path = match resolve_existing(workspace, &raw_path) {
        Ok(path) => path,
        Err(error) => return ToolResult::error(error),
    };
    if !path.is_file() {
        return ToolResult::error(format!("Not a regular file: {}", path.display()));
    }
    let metadata = match fs::metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) => return ToolResult::error(format!("Cannot stat {}: {error}", path.display())),
    };
    if metadata.len() > 2 * 1024 * 1024 {
        return ToolResult::error(format!(
            "File is too large ({} bytes). Read limit is 2 MiB.",
            metadata.len()
        ));
    }
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) => {
            return ToolResult::error(format!(
                "Cannot read {} as UTF-8 text: {error}",
                path.display()
            ))
        }
    };

    let lines: Vec<&str> = content.lines().collect();
    let max_lines = 2000usize;
    let mut out = String::new();
    out.push_str(&format!("File: {}\n", path.display()));
    for (index, line) in lines.iter().take(max_lines).enumerate() {
        out.push_str(&format!("{:>5}\t{}\n", index + 1, line));
    }
    if lines.len() > max_lines {
        out.push_str(&format!(
            "\n... [truncated: showing {max_lines} of {} lines]",
            lines.len()
        ));
    }
    ToolResult::ok(truncate(&out, MAX_TOOL_CHARS))
}

fn list_directory(args: &Value, workspace: &Path) -> ToolResult {
    let raw_path = match string_arg(args, "path") {
        Ok(value) => value,
        Err(error) => return ToolResult::error(error),
    };
    let path = match resolve_existing(workspace, &raw_path) {
        Ok(path) => path,
        Err(error) => return ToolResult::error(error),
    };
    if !path.is_dir() {
        return ToolResult::error(format!("Not a directory: {}", path.display()));
    }
    let entries = match fs::read_dir(&path) {
        Ok(entries) => entries,
        Err(error) => {
            return ToolResult::error(format!("Cannot list {}: {error}", path.display()))
        }
    };

    let mut rows: Vec<(bool, u64, String)> = Vec::new();
    for entry in entries.flatten().take(500) {
        let metadata = entry.metadata().ok();
        let is_dir = metadata.as_ref().map(|meta| meta.is_dir()).unwrap_or(false);
        let size = metadata.as_ref().map(|meta| meta.len()).unwrap_or(0);
        let name = entry.file_name().to_string_lossy().to_string();
        rows.push((is_dir, size, name));
    }
    rows.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.2.to_lowercase().cmp(&b.2.to_lowercase())));

    let mut out = format!("Directory: {}\n", path.display());
    for (is_dir, size, name) in &rows {
        if *is_dir {
            out.push_str(&format!("d  {:>10}  {name}/\n", "-"));
        } else {
            out.push_str(&format!("f  {size:>10}  {name}\n"));
        }
    }
    if rows.is_empty() {
        out.push_str("(empty)\n");
    }
    ToolResult::ok(truncate(&out, MAX_TOOL_CHARS))
}

fn search_files(args: &Value, workspace: &Path) -> ToolResult {
    let raw_path = match string_arg(args, "path") {
        Ok(value) => value,
        Err(error) => return ToolResult::error(error),
    };
    let pattern = match string_arg(args, "pattern") {
        Ok(value) => value,
        Err(error) => return ToolResult::error(error),
    };
    if pattern.trim().is_empty() {
        return ToolResult::error("The search pattern cannot be empty.");
    }
    let glob = args
        .get("glob")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let root = match resolve_existing(workspace, &raw_path) {
        Ok(path) => path,
        Err(error) => return ToolResult::error(error),
    };
    if !root.is_dir() {
        return ToolResult::error(format!("Not a directory: {}", root.display()));
    }

    let needle = pattern.to_lowercase();
    let mut matches = 0usize;
    let mut out = format!("Search: {pattern:?} in {}\n", root.display());

    'walk: for entry in WalkDir::new(&root)
        .follow_links(false)
        .max_depth(10)
        .into_iter()
        .filter_entry(|entry| !should_skip(entry))
        .flatten()
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if let Some(glob) = glob {
            let name = entry.file_name().to_string_lossy();
            if !glob_match(glob, &name) {
                continue;
            }
        }
        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(_) => continue,
        };
        if metadata.len() > 1024 * 1024 {
            continue;
        }
        let content = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(_) => continue,
        };

        for (line_number, line) in content.lines().enumerate() {
            if line.to_lowercase().contains(&needle) {
                let relative = path.strip_prefix(&root).unwrap_or(path);
                out.push_str(&format!(
                    "{}:{}: {}\n",
                    relative.display(),
                    line_number + 1,
                    truncate(line.trim(), 300)
                ));
                matches += 1;
                if matches >= MAX_SEARCH_MATCHES || out.len() >= MAX_TOOL_CHARS {
                    out.push_str(&format!(
                        "\n... [stopped after {matches} matches]\n"
                    ));
                    break 'walk;
                }
            }
        }
    }

    if matches == 0 {
        out.push_str("No matches.\n");
    }
    ToolResult::ok(truncate(&out, MAX_TOOL_CHARS))
}

fn write_file(args: &Value, workspace: &Path) -> ToolResult {
    let raw_path = match string_arg(args, "path") {
        Ok(value) => value,
        Err(error) => return ToolResult::error(error),
    };
    let content = match args.get("content").and_then(Value::as_str) {
        Some(content) => content,
        None => return ToolResult::error("Missing required argument: content"),
    };
    let path = match resolve_for_write(workspace, &raw_path) {
        Ok(path) => path,
        Err(error) => return ToolResult::error(error),
    };
    if let Some(parent) = path.parent() {
        if let Err(error) = fs::create_dir_all(parent) {
            return ToolResult::error(format!(
                "Cannot create parent directory {}: {error}",
                parent.display()
            ));
        }
    }
    match fs::write(&path, content) {
        Ok(()) => ToolResult::ok(format!(
            "Wrote {} bytes to {}",
            content.len(),
            path.display()
        )),
        Err(error) => ToolResult::error(format!("Cannot write {}: {error}", path.display())),
    }
}

fn edit_file(args: &Value, workspace: &Path) -> ToolResult {
    let raw_path = match string_arg(args, "path") {
        Ok(value) => value,
        Err(error) => return ToolResult::error(error),
    };
    let old_string = match string_arg(args, "old_string") {
        Ok(value) => value,
        Err(error) => return ToolResult::error(error),
    };
    let new_string = match string_arg(args, "new_string") {
        Ok(value) => value,
        Err(error) => return ToolResult::error(error),
    };
    if old_string.is_empty() {
        return ToolResult::error("old_string cannot be empty.");
    }
    let replace_all = args
        .get("replace_all")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let path = match resolve_for_write(workspace, &raw_path) {
        Ok(path) => path,
        Err(error) => return ToolResult::error(error),
    };
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) => {
            return ToolResult::error(format!("Cannot read {}: {error}", path.display()))
        }
    };
    let occurrences = content.matches(&old_string).count();
    if occurrences == 0 {
        return ToolResult::error(format!(
            "The exact old_string was not found in {}",
            path.display()
        ));
    }
    let updated = if replace_all {
        content.replace(&old_string, &new_string)
    } else {
        content.replacen(&old_string, &new_string, 1)
    };
    match fs::write(&path, updated) {
        Ok(()) => ToolResult::ok(format!(
            "Replaced {} occurrence(s) in {}",
            if replace_all { occurrences } else { 1 },
            path.display()
        )),
        Err(error) => ToolResult::error(format!("Cannot write {}: {error}", path.display())),
    }
}

async fn run_command(
    args: &Value,
    workspace: &Path,
    token: &CancellationToken,
) -> ToolResult {
    let command = match string_arg(args, "command") {
        Ok(value) => value,
        Err(error) => return ToolResult::error(error),
    };
    if command.trim().is_empty() {
        return ToolResult::error("The command cannot be empty.");
    }

    let cwd = match args
        .get("cwd")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(raw) => match resolve_existing(workspace, raw) {
            Ok(path) if path.is_dir() => path,
            Ok(path) => {
                return ToolResult::error(format!(
                    "Working directory is not a directory: {}",
                    path.display()
                ))
            }
            Err(error) => return ToolResult::error(error),
        },
        None => workspace.to_path_buf(),
    };

    let mut process = tokio::process::Command::new("sh");
    process
        .arg("-lc")
        .arg(&command)
        .current_dir(&cwd)
        .kill_on_drop(true);

    let output = tokio::select! {
        _ = token.cancelled() => {
            return ToolResult::error("Command cancelled.");
        }
        result = tokio::time::timeout(Duration::from_secs(120), process.output()) => {
            match result {
                Err(_) => return ToolResult::error("Command timed out after 120 seconds."),
                Ok(Err(error)) => return ToolResult::error(format!("Cannot run command: {error}")),
                Ok(Ok(output)) => output,
            }
        }
    };

    let stdout = strip_ansi(&String::from_utf8_lossy(&output.stdout));
    let stderr = strip_ansi(&String::from_utf8_lossy(&output.stderr));
    let mut text = format!(
        "$ {command}\n(exit code: {})\n",
        output.status.code().unwrap_or(-1)
    );
    if !stdout.trim().is_empty() {
        text.push_str("--- stdout ---\n");
        text.push_str(stdout.trim_end());
        text.push('\n');
    }
    if !stderr.trim().is_empty() {
        text.push_str("--- stderr ---\n");
        text.push_str(stderr.trim_end());
        text.push('\n');
    }
    if stdout.trim().is_empty() && stderr.trim().is_empty() {
        text.push_str("(no output)\n");
    }

    let result = ToolResult::ok(truncate(&text, MAX_TOOL_CHARS));
    if output.status.success() {
        result
    } else {
        ToolResult {
            ok: false,
            content: result.content,
        }
    }
}

fn string_arg(args: &Value, key: &str) -> Result<String, String> {
    args.get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("Missing required argument: {key}"))
}

fn make_absolute(workspace: &Path, raw: &str) -> PathBuf {
    let path = PathBuf::from(raw.trim());
    if path.is_absolute() {
        path
    } else {
        workspace.join(path)
    }
}

fn resolve_existing(workspace: &Path, raw: &str) -> Result<PathBuf, String> {
    let path = make_absolute(workspace, raw);
    let canonical = path
        .canonicalize()
        .map_err(|error| format!("Path does not exist or is inaccessible: {} ({error})", path.display()))?;
    ensure_inside(workspace, &canonical)?;
    Ok(canonical)
}

fn resolve_for_write(workspace: &Path, raw: &str) -> Result<PathBuf, String> {
    let path = make_absolute(workspace, raw);

    if path.exists() {
        let canonical = path
            .canonicalize()
            .map_err(|error| format!("Cannot resolve {}: {error}", path.display()))?;
        ensure_inside(workspace, &canonical)?;
        return Ok(canonical);
    }

    let mut existing = path.clone();
    let mut suffix: Vec<std::ffi::OsString> = Vec::new();
    while !existing.exists() {
        let name = existing
            .file_name()
            .ok_or_else(|| format!("Invalid path: {}", path.display()))?
            .to_os_string();
        suffix.push(name);
        existing = existing
            .parent()
            .ok_or_else(|| format!("Invalid path: {}", path.display()))?
            .to_path_buf();
    }

    let canonical_parent = existing
        .canonicalize()
        .map_err(|error| format!("Cannot resolve {}: {error}", existing.display()))?;
    ensure_inside(workspace, &canonical_parent)?;

    let mut resolved = canonical_parent;
    for component in suffix.iter().rev() {
        resolved.push(component);
    }
    Ok(resolved)
}

fn ensure_inside(workspace: &Path, path: &Path) -> Result<(), String> {
    if path.starts_with(workspace) {
        Ok(())
    } else {
        Err(format!(
            "Access denied: {} is outside the authorized workspace {}",
            path.display(),
            workspace.display()
        ))
    }
}

fn should_skip(entry: &walkdir::DirEntry) -> bool {
    if !entry.file_type().is_dir() {
        return true;
    }
    let name = entry.file_name().to_string_lossy();
    !matches!(
        name.as_ref(),
        ".git" | "node_modules" | "target" | ".cache" | ".local" | ".cargo"
    )
}

fn glob_match(pattern: &str, name: &str) -> bool {
    let pattern = pattern.as_bytes();
    let name = name.as_bytes();
    let (mut p, mut n) = (0usize, 0usize);
    let (mut star, mut mark) = (None, 0usize);

    while n < name.len() {
        if p < pattern.len() && (pattern[p] == b'?' || pattern[p] == name[n]) {
            p += 1;
            n += 1;
        } else if p < pattern.len() && pattern[p] == b'*' {
            star = Some(p);
            p += 1;
            mark = n;
        } else if let Some(star_index) = star {
            p = star_index + 1;
            mark += 1;
            n = mark;
        } else {
            return false;
        }
    }

    while p < pattern.len() && pattern[p] == b'*' {
        p += 1;
    }
    p == pattern.len()
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

fn truncate(value: &str, max: usize) -> String {
    if value.chars().count() <= max {
        return value.to_string();
    }
    let shortened: String = value.chars().take(max).collect();
    format!("{shortened}\n... [truncated]")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glob_matching() {
        assert!(glob_match("*.rs", "main.rs"));
        assert!(glob_match("test?.txt", "test1.txt"));
        assert!(!glob_match("*.rs", "main.py"));
    }

    #[test]
    fn paths_cannot_escape_workspace() {
        let workspace = std::env::temp_dir().canonicalize().unwrap();
        assert!(ensure_inside(&workspace, &workspace.join("ok.txt")).is_ok());
        assert!(ensure_inside(&workspace, Path::new("/etc/passwd")).is_err());
    }
}

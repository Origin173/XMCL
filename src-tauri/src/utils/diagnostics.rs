use crate::account::models::AccountInfo;
use crate::error::XMCLResult;
use crate::launcher_config::models::{JavaInfo, LauncherConfig};
use crate::utils::sys_info::{get_mapped_locale, get_memory_info};
use regex::{Captures, Regex};
use serde_json::Value;
use std::fmt::Write as _;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};
use std::time::SystemTime;
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};
use zip::write::{ExtendedFileOptions, FileOptions};
use zip::{CompressionMethod, ZipWriter};

const MAX_LAUNCHER_LOGS: usize = 5;
const MAX_GAME_LOGS: usize = 3;
const MAX_LOG_BYTES: usize = 2 * 1024 * 1024;
const REDACTED: &str = "<redacted>";

static JWT_RE: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"eyJ[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]*").unwrap());
// any text after "cookie...:" on the same line, covers cookie headers and old CAUC logs
static COOKIE_RE: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"(?i)(cookie[^:\r\n]{0,40}:\s*)[^\r\n]+").unwrap());
static BEARER_RE: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"(?i)(bearer\s+)[^\s\x22']+").unwrap());
static ACCESS_TOKEN_ARG_RE: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"(?i)(--accessToken\s+)\S+").unwrap());
static SECRET_KV_RE: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(
    r#"(?i)("?[\w-]*(?:token|session|password|secret|authorization)"?\s*[:=]\s*"?)([^"\s;,&]+)"#,
  )
  .unwrap()
});
static SECRET_KEY_RE: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"(?i)token|session|password|secret|authorization").unwrap());

/// Mask credentials (tokens, cookies, passwords) and the user's home directory in the text.
pub fn redact_text(text: &str, home_dir: Option<&Path>) -> String {
  let text = JWT_RE.replace_all(text, REDACTED);
  let text = COOKIE_RE.replace_all(&text, |c: &Captures| format!("{}{REDACTED}", &c[1]));
  let text = BEARER_RE.replace_all(&text, |c: &Captures| format!("{}{REDACTED}", &c[1]));
  let text = ACCESS_TOKEN_ARG_RE.replace_all(&text, |c: &Captures| format!("{}{REDACTED}", &c[1]));
  let mut text = SECRET_KV_RE
    .replace_all(&text, |c: &Captures| format!("{}{REDACTED}", &c[1]))
    .into_owned();

  if let Some(home) = home_dir.and_then(|h| h.to_str()).filter(|h| h.len() > 3) {
    for variant in [
      home.to_string(),
      home.replace('\\', "/"),
      home.replace('\\', "\\\\"),
    ] {
      if let Ok(re) = Regex::new(&format!("(?i){}", regex::escape(&variant))) {
        text = re.replace_all(&text, "~").into_owned();
      }
    }
  }
  text
}

fn redact_json(value: &mut Value) {
  match value {
    Value::Object(map) => {
      for (key, v) in map.iter_mut() {
        if SECRET_KEY_RE.is_match(key) && !v.is_null() {
          *v = Value::String(REDACTED.to_string());
        } else {
          redact_json(v);
        }
      }
    }
    Value::Array(arr) => arr.iter_mut().for_each(redact_json),
    _ => {}
  }
}

/// Newest files first, at most `limit` entries.
fn newest_files(dir: &Path, limit: usize) -> Vec<PathBuf> {
  let mut files: Vec<(SystemTime, PathBuf)> = fs::read_dir(dir)
    .map(|entries| {
      entries
        .flatten()
        .filter(|e| e.path().is_file())
        .map(|e| {
          let modified = e
            .metadata()
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);
          (modified, e.path())
        })
        .collect()
    })
    .unwrap_or_default();
  files.sort_by(|a, b| b.0.cmp(&a.0));
  files.into_iter().take(limit).map(|(_, p)| p).collect()
}

/// Read a log file, keeping only its tail when it is too large.
fn read_log_tail(path: &Path) -> Option<String> {
  let bytes = fs::read(path).ok()?;
  let start = bytes.len().saturating_sub(MAX_LOG_BYTES);
  Some(String::from_utf8_lossy(&bytes[start..]).into_owned())
}

fn build_report_text(app: &AppHandle, description: &str) -> XMCLResult<String> {
  let config = app.state::<Mutex<LauncherConfig>>().lock()?.clone();
  let accounts = app.state::<Mutex<AccountInfo>>().lock()?.clone();
  let javas = app.state::<Mutex<Vec<JavaInfo>>>().lock()?.clone();
  let memory = get_memory_info();
  let info = &config.basic_info;

  let mut report = String::new();
  let _ = writeln!(report, "# XMCL Diagnostic Report\n");
  let _ = writeln!(
    report,
    "Generated at: {}\n",
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S %:z")
  );
  let _ = writeln!(report, "## Description\n\n{}\n", description.trim());
  let _ = writeln!(report, "## Environment\n");
  let _ = writeln!(report, "- Launcher version: {}", info.launcher_version);
  let _ = writeln!(
    report,
    "- OS: {} {} ({}, {})",
    info.os_type, info.platform_version, info.platform, info.arch
  );
  let _ = writeln!(report, "- Portable: {}", info.is_portable);
  let _ = writeln!(report, "- Locale: {}", get_mapped_locale());
  let _ = writeln!(
    report,
    "- Memory: {} MB total, {} MB used",
    memory.total / 1024 / 1024,
    memory.used / 1024 / 1024
  );

  let _ = writeln!(report, "\n## Java\n");
  if javas.is_empty() {
    let _ = writeln!(report, "- (none detected)");
  }
  for java in &javas {
    let _ = writeln!(
      report,
      "- {} ({}) {}",
      java.name, java.vendor, java.exec_path
    );
  }

  let _ = writeln!(report, "\n## Accounts\n");
  for player in &accounts.players {
    let _ = writeln!(
      report,
      "- {:?} {}",
      player.player_type,
      player.auth_server_url.clone().unwrap_or_default()
    );
  }
  for server in &accounts.auth_servers {
    let _ = writeln!(
      report,
      "- auth server: {} (metadata cached: {})",
      server.auth_url,
      !server.metadata.is_null()
    );
  }

  Ok(report)
}

pub fn export_diagnostic_report(
  app: &AppHandle,
  description: &str,
  include_logs: bool,
  save_path: &Path,
) -> XMCLResult<()> {
  let home_dir = app.path().home_dir().ok();
  let home = home_dir.as_deref();

  let mut zip = ZipWriter::new(fs::File::create(save_path)?);
  let options =
    FileOptions::<ExtendedFileOptions>::default().compression_method(CompressionMethod::Deflated);

  let report = build_report_text(app, description)?;
  zip.start_file("report.md", options.clone())?;
  zip.write_all(redact_text(&report, home).as_bytes())?;

  let mut config_json = serde_json::to_value(&*app.state::<Mutex<LauncherConfig>>().lock()?)?;
  redact_json(&mut config_json);
  zip.start_file("launcher_config.json", options.clone())?;
  zip.write_all(redact_text(&serde_json::to_string_pretty(&config_json)?, home).as_bytes())?;

  if include_logs {
    let log_dirs = [
      ("logs/launcher", "LauncherLogs", MAX_LAUNCHER_LOGS),
      ("logs/game", "GameLogs", MAX_GAME_LOGS),
    ];
    for (zip_dir, cache_subdir, limit) in log_dirs {
      let dir = app
        .path()
        .resolve::<PathBuf>(cache_subdir.into(), BaseDirectory::AppCache)?;
      for path in newest_files(&dir, limit) {
        let (Some(name), Some(content)) = (
          path.file_name().and_then(|n| n.to_str()),
          read_log_tail(&path),
        ) else {
          continue;
        };
        zip.start_file(format!("{zip_dir}/{name}"), options.clone())?;
        zip.write_all(redact_text(&content, home).as_bytes())?;
      }
    }
  }

  zip.finish()?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn redacts_credentials() {
    let input = concat!(
      "Received cookie from login page: XSRF-TOKEN=abc123; path=/\n",
      "{\"accessToken\":\"deadbeef\",\"clientToken\":\"cafe\"}\n",
      "Authorization: Bearer eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIn0.sig\n",
      "java --accessToken 0123456789 --uuid 1\n",
      "Extracted CSRF token: zzz\n",
      "Extracted CSRF token (length: 40)\n",
    );
    let output = redact_text(input, None);
    for secret in [
      "abc123",
      "deadbeef",
      "cafe",
      "eyJhbGci",
      "0123456789",
      "zzz",
    ] {
      assert!(!output.contains(secret), "{secret} leaked in:\n{output}");
    }
    assert!(output.contains("Extracted CSRF token (length: 40)"));
    assert!(output.contains("--uuid 1"));
  }

  #[test]
  fn redacts_home_dir() {
    let output = redact_text(
      r"C:\Users\Alice\AppData and c:/users/alice/x",
      Some(Path::new(r"C:\Users\Alice")),
    );
    assert_eq!(output, r"~\AppData and ~/x");
  }

  #[test]
  fn redacts_json_keys() {
    let mut value = serde_json::json!({ "proxy": { "password": "p", "host": "h" } });
    redact_json(&mut value);
    assert_eq!(value["proxy"]["password"], REDACTED);
    assert_eq!(value["proxy"]["host"], "h");
  }
}

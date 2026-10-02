use crate::account::helpers::authlib_injector::cauc::CAUCAuthState;
use crate::account::helpers::authlib_injector::info::refresh_and_update_auth_servers;
use crate::account::helpers::authlib_injector::jar::get_jar_path;
use crate::account::models::{AccountInfo, PlayerType};
use crate::error::XMCLResult;
use crate::launch::models::LaunchingState;
use crate::launcher_config::models::{ClearCacheOptions, ClearCacheResult, LauncherConfig};
use crate::storage::Storage;
use crate::utils::logging::get_launcher_log_path;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

const LAUNCHER_LOGS_DIR: &str = "LauncherLogs";
const GAME_LOGS_DIR: &str = "GameLogs";

#[derive(Default)]
struct RemovalStats {
  freed_bytes: u64,
  failed_count: u32,
}

impl RemovalStats {
  fn merge(&mut self, other: RemovalStats) {
    self.freed_bytes += other.freed_bytes;
    self.failed_count += other.failed_count;
  }
}

/// Remove a file or a directory tree entry by entry, so that files locked by other
/// processes are skipped instead of aborting the whole removal.
fn remove_path(path: &Path) -> RemovalStats {
  let mut stats = RemovalStats::default();
  let Ok(meta) = fs::symlink_metadata(path) else {
    return stats;
  };

  if meta.is_dir() {
    if let Ok(entries) = fs::read_dir(path) {
      for entry in entries.flatten() {
        stats.merge(remove_path(&entry.path()));
      }
    }
    if fs::remove_dir(path).is_err() {
      stats.failed_count += 1;
    }
  } else if fs::remove_file(path).is_ok() {
    stats.freed_bytes += meta.len();
  } else {
    stats.failed_count += 1;
  }

  stats
}

/// Remove all entries inside `dir` (keeping `dir` itself) except those matched by `skip`.
fn remove_dir_contents(dir: &Path, skip: impl Fn(&Path) -> bool) -> RemovalStats {
  let mut stats = RemovalStats::default();
  if let Ok(entries) = fs::read_dir(dir) {
    for entry in entries.flatten() {
      let path = entry.path();
      if !skip(&path) {
        stats.merge(remove_path(&path));
      }
    }
  }
  stats
}

fn clear_download_cache_dir(dir: &Path) -> RemovalStats {
  // guard against misconfigured cache directories pointing to a filesystem root
  if dir.as_os_str().is_empty() || dir.parent().is_none() {
    return RemovalStats::default();
  }
  remove_dir_contents(dir, |_| false)
}

fn clear_log_files(app: &AppHandle, cache_dir: &Path) -> RemovalStats {
  let mut stats = RemovalStats::default();

  let current_log = get_launcher_log_path(app.clone());
  stats.merge(remove_dir_contents(
    &cache_dir.join(LAUNCHER_LOGS_DIR),
    |path| path == current_log,
  ));

  // keep logs of games that are still running or waiting for the crash report
  let active_logs: Vec<String> = {
    let launching_queue = app.state::<Mutex<Vec<LaunchingState>>>();
    let queue = launching_queue
      .lock()
      .map(|q| q.clone())
      .unwrap_or_default();
    queue
      .iter()
      .map(|l| format!("game_log_{}.log", l.id))
      .collect()
  };
  stats.merge(remove_dir_contents(
    &cache_dir.join(GAME_LOGS_DIR),
    |path| {
      path
        .file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| active_logs.iter().any(|l| l == n))
    },
  ));

  stats
}

async fn clear_login_cache(app: &AppHandle, clear_credentials: bool) -> XMCLResult<RemovalStats> {
  let mut stats = RemovalStats::default();

  // drop the temporary CAUC session (cookies and XSRF token)
  if let Ok(mut cauc_state) = app.state::<Mutex<Option<CAUCAuthState>>>().lock() {
    cauc_state.take();
  }

  {
    let account_binding = app.state::<Mutex<AccountInfo>>();
    let mut account_state = account_binding.lock()?;
    // a persisted `true` left by an interrupted OAuth flow blocks later logins
    account_state.is_oauth_processing = false;
    if clear_credentials {
      for player in account_state
        .players
        .iter_mut()
        .filter(|p| p.player_type != PlayerType::Offline)
      {
        player.access_token = None;
        player.refresh_token = None;
      }
    }
    account_state.save()?;
  }

  // re-fetch authlib-injector server metadata, servers that fail to respond keep the old data
  if refresh_and_update_auth_servers(app).await.is_ok() {
    app.state::<Mutex<AccountInfo>>().lock()?.save()?;
  }

  // authlib-injector.jar will be re-downloaded on next login or launch
  if let Ok(jar_path) = get_jar_path(app) {
    if jar_path.exists() {
      stats.merge(remove_path(&jar_path));
    }
  }

  // cookies, local storage and HTTP cache of the launcher webviews
  for webview in app.webview_windows().values() {
    if webview.clear_all_browsing_data().is_err() {
      stats.failed_count += 1;
    }
  }

  Ok(stats)
}

pub async fn clear_launcher_cache(
  app: &AppHandle,
  options: &ClearCacheOptions,
) -> XMCLResult<ClearCacheResult> {
  let mut stats = RemovalStats::default();
  let cache_dir = app.path().app_cache_dir()?;
  let download_dir: PathBuf = {
    let config = app.state::<Mutex<LauncherConfig>>();
    let config = config.lock()?;
    config.download.cache.directory.clone()
  };

  if options.login {
    stats.merge(clear_login_cache(app, options.credentials).await?);
  }

  if options.download {
    stats.merge(clear_download_cache_dir(&download_dir));
  }

  if options.temp {
    stats.merge(remove_dir_contents(&cache_dir, |path| {
      path == download_dir || path.ends_with(LAUNCHER_LOGS_DIR) || path.ends_with(GAME_LOGS_DIR)
    }));
  }

  if options.logs {
    stats.merge(clear_log_files(app, &cache_dir));
  }

  Ok(ClearCacheResult {
    freed_bytes: stats.freed_bytes,
    failed_count: stats.failed_count,
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("xmcl-cache-test-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
  }

  #[test]
  fn removes_contents_except_skipped() {
    let dir = temp_dir("skip");
    fs::create_dir_all(dir.join("LauncherLogs")).unwrap();
    fs::write(dir.join("LauncherLogs/current.log"), "log").unwrap();
    fs::create_dir_all(dir.join("nested/deep")).unwrap();
    fs::write(dir.join("nested/deep/a.bin"), [0u8; 10]).unwrap();
    fs::write(dir.join("game_versions.txt"), [0u8; 5]).unwrap();

    let stats = remove_dir_contents(&dir, |p| p.ends_with(LAUNCHER_LOGS_DIR));

    assert_eq!(stats.freed_bytes, 15);
    assert_eq!(stats.failed_count, 0);
    assert!(dir.exists());
    assert!(dir.join("LauncherLogs/current.log").exists());
    assert!(!dir.join("nested").exists());
    assert!(!dir.join("game_versions.txt").exists());
    fs::remove_dir_all(&dir).unwrap();
  }

  #[cfg(target_os = "windows")]
  #[test]
  fn locked_file_does_not_abort_removal() {
    let dir = temp_dir("locked");
    fs::write(dir.join("locked.log"), "x").unwrap();
    fs::write(dir.join("free.txt"), "yy").unwrap();
    // an open handle without FILE_SHARE_DELETE makes deletion fail on Windows
    use std::os::windows::fs::OpenOptionsExt;
    let _handle = fs::OpenOptions::new()
      .read(true)
      .share_mode(0)
      .open(dir.join("locked.log"))
      .unwrap();

    let stats = remove_dir_contents(&dir, |_| false);

    assert_eq!(stats.freed_bytes, 2);
    assert_eq!(stats.failed_count, 1);
    assert!(dir.join("locked.log").exists());
    drop(_handle);
    fs::remove_dir_all(&dir).unwrap();
  }

  #[test]
  fn refuses_root_download_dir() {
    let stats = clear_download_cache_dir(Path::new(if cfg!(windows) { "C:\\" } else { "/" }));
    assert_eq!(stats.freed_bytes, 0);
    assert_eq!(stats.failed_count, 0);
  }
}

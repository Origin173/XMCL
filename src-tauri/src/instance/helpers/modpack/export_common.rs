use crate::instance::models::misc::{ExportFileEntry, FileCategory};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use sha2::Sha512;
use std::fs;
use std::path::{Component, Path, PathBuf};
use tauri::{AppHandle, Emitter};

const EXPORT_PROGRESS_EVENT: &str = "export-modpack:progress";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportProgressPayload {
  pub current: usize,
  pub total: usize,
  pub file_name: String,
  pub stage: ExportStage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ExportStage {
  Matching,
  Packing,
  WritingManifest,
  Done,
}

pub fn emit_progress(
  app: &AppHandle,
  current: usize,
  total: usize,
  file_name: &str,
  stage: ExportStage,
) {
  let _ = app.emit_to(
    "main",
    EXPORT_PROGRESS_EVENT,
    ExportProgressPayload {
      current,
      total,
      file_name: file_name.to_string(),
      stage,
    },
  );
}

/// Returns true if the file should be shown but not selected by default.
fn is_normal_unselected(rel_path: &str, file_name: &str, is_dir: bool) -> bool {
  let normal_dirs = [
    "blueprints",
    "fonts",
    "journeymap",
    "local",
    "logs",
    "versions",
    "assets",
    "libraries",
    "natives",
    "crash-reports",
    "screenshots",
    "saves",
    "xaero",
    "backup",
    "backups",
    "cache",
    "caches",
    "downloads",
    "launcher",
    "runtime",
    "runtimes",
    "server-resource-packs",
    ".fabric",
    ".mixin.out",
    "asm",
    "debug",
    "native_libs",
  ];

  let normal_files = [
    "servers.dat",
    "options.txt",
    "optionsshaders.txt",
    "optionsof.txt",
    "realms_persistence.json",
    "servers.dat_old",
    "usernamecache.json",
    "usercache.json",
    "launcher_profiles.json",
    "manifest.json",
    "modrinth.index.json",
    "instance.json",
    "mmc-pack.json",
    "instance.cfg",
    ".DS_Store",
    "desktop.ini",
    "Thumbs.db",
    "hmclversion.cfg",
    "PCL.ini",
  ];

  let normal_exts = ["log", "old", "dat_old", "bak"];

  let top_level = rel_path.split('/').next().unwrap_or(rel_path);
  if normal_dirs.contains(&top_level) {
    return true;
  }

  // mods/VoxelMods/ subdir
  if rel_path.starts_with("mods/VoxelMods") || rel_path.starts_with("mods\\VoxelMods") {
    return true;
  }

  if !is_dir && normal_files.contains(&file_name) {
    return true;
  }

  if !is_dir && file_name.starts_with("._") {
    return true;
  }

  if !is_dir {
    if let Some(ext) = rel_path.rsplit('.').next() {
      if normal_exts.contains(&ext) {
        return true;
      }
    }
  }

  if is_dir && (file_name.ends_with("-natives") || file_name.ends_with("_natives")) {
    return true;
  }

  false
}

/// Determine the FileCategory for a path relative to the instance root.
pub fn categorize(rel_path: &str, is_dir: bool) -> FileCategory {
  // Normalize separators
  let rel_path = rel_path.replace('\\', "/");

  let file_name = rel_path.rsplit('/').next().unwrap_or(&rel_path);

  if is_normal_unselected(&rel_path, file_name, is_dir) {
    return FileCategory::Normal;
  }

  FileCategory::Suggested
}

/// Normalize a frontend-provided relative path and reject anything that can
/// escape the instance root.
pub fn normalize_relative_path(rel_path: &str) -> Option<String> {
  let path = Path::new(rel_path);
  let mut parts: Vec<String> = Vec::new();

  for component in path.components() {
    match component {
      Component::Normal(part) => parts.push(part.to_string_lossy().to_string()),
      Component::CurDir => continue,
      Component::ParentDir | Component::RootDir | Component::Prefix(_) => return None,
    }
  }

  if parts.is_empty() {
    None
  } else {
    Some(parts.join("/"))
  }
}

/// Convert selected relative paths into safe, existing files under the instance root.
pub fn selected_file_paths(
  instance_path: &Path,
  selected_files: &[String],
) -> std::io::Result<Vec<(String, PathBuf)>> {
  let canonical_root = instance_path.canonicalize()?;
  let mut files = Vec::new();

  for rel_path in selected_files {
    let Some(normalized) = normalize_relative_path(rel_path) else {
      continue;
    };
    let abs_path = instance_path.join(normalized.replace('/', std::path::MAIN_SEPARATOR_STR));

    if !abs_path.is_file() {
      continue;
    }

    let canonical_file = abs_path.canonicalize()?;
    if canonical_file.starts_with(&canonical_root) {
      files.push((normalized, canonical_file));
    }
  }

  Ok(files)
}

/// Recursively walk `dir`, producing ExportFileEntry for every non-hidden entry.
/// `base` is the instance root directory.
pub fn scan_dir(base: &Path, dir: &Path) -> std::io::Result<Vec<ExportFileEntry>> {
  let mut entries: Vec<ExportFileEntry> = Vec::new();

  let read_dir = match fs::read_dir(dir) {
    Ok(rd) => rd,
    Err(_) => return Ok(entries),
  };

  let mut dir_entries: Vec<_> = read_dir.flatten().collect();
  dir_entries.sort_by_key(|entry| entry.path());

  for entry in dir_entries {
    let path = entry.path();
    let file_type = match entry.file_type() {
      Ok(t) => t,
      Err(_) => continue,
    };
    let is_symlink = file_type.is_symlink();
    let metadata = match entry.metadata() {
      Ok(m) => m,
      Err(_) => continue,
    };
    let is_dir = !is_symlink && metadata.is_dir();

    // Build relative path using forward slashes
    let rel = match path.strip_prefix(base) {
      Ok(r) => r.to_string_lossy().replace('\\', "/"),
      Err(_) => continue,
    };

    let category = categorize(&rel, is_dir);

    let file_size = if is_dir { 0 } else { metadata.len() };

    entries.push(ExportFileEntry {
      relative_path: rel.clone(),
      is_directory: is_dir,
      category,
      file_size,
    });

    if is_dir {
      let mut children = scan_dir(base, &path)?;
      entries.append(&mut children);
    }
  }

  Ok(entries)
}

/// Calculate SHA-1 hex string for a file.
pub fn sha1_of_file(path: &PathBuf) -> std::io::Result<String> {
  let data = fs::read(path)?;
  let mut hasher = Sha1::new();
  hasher.update(&data);
  Ok(hex::encode(hasher.finalize()))
}

/// Calculate SHA-512 hex string for a file.
pub fn sha512_of_file(path: &PathBuf) -> std::io::Result<String> {
  let data = fs::read(path)?;
  let mut hasher = Sha512::new();
  hasher.update(&data);
  Ok(hex::encode(hasher.finalize()))
}

/// Paths inside an instance that should be attempted for remote matching.
pub fn is_remotely_matchable(rel_path: &str) -> bool {
  let rel = rel_path.replace('\\', "/");
  rel.starts_with("mods/") || rel.starts_with("resourcepacks/") || rel.starts_with("shaderpacks/")
}

use crate::error::XMCLResult;
use crate::instance::helpers::modpack::export_common::{
  categorize, emit_progress, normalize_relative_path, scan_dir, ExportStage,
};
use crate::instance::models::misc::{
  ExportFileEntry, ExportModpackMeta, InstanceError, ModLoader, ModLoaderType,
};
use crate::launcher_config::models::GameDirectory;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tauri::AppHandle;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

/// Manifest written at the zip root to identify an XMCL full pack and carry metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FullPackManifest {
  pub format_version: u32,
  pub name: String,
  pub version: String,
  pub description: Option<String>,
  pub author: Option<String>,
  pub minecraft_version: String,
  pub mod_loader: ModLoader,
  pub file_count: usize,
  pub total_size: u64,
}

/// Instance sub-directories shared at the game dir root when version isolation is off.
const SHARED_GAME_DIRS: &[&str] = &[
  "mods",
  "resourcepacks",
  "shaderpacks",
  "saves",
  "schematics",
  "screenshots",
  "server-resource-packs",
  "config",
  "datapacks",
];

/// Root-level game files shared at the game dir root when version isolation is off.
const SHARED_GAME_FILES: &[&str] = &[
  "options.txt",
  "optionsshaders.txt",
  "optionsof.txt",
  "servers.dat",
  "realms_persistence.json",
];

/// Files that are mandatory for the imported instance to launch; always packed,
/// never shown in the file selection tree.
const CORE_INSTANCE_FILES: &[&str] = &["xmclcfg.json"];

/// Directories under the instance root that are rebuilt at launch and should not be packed.
const IGNORED_INSTANCE_DIR_PREFIXES: &[&str] = &["natives"];

/// Files with these extensions are already compressed; store them as-is to save CPU time.
fn should_compress(path: &str) -> bool {
  let ext = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
  !matches!(
    ext.as_str(),
    "jar"
      | "zip"
      | "gz"
      | "7z"
      | "png"
      | "jpg"
      | "jpeg"
      | "gif"
      | "webp"
      | "ogg"
      | "mp3"
      | "mp4"
      | "webm"
      | "bin"
      | "nib"
      | "class"
      | "pack"
      | "icns"
      | "ico"
  )
}

fn compression_options(path: &str) -> SimpleFileOptions {
  if should_compress(path) {
    SimpleFileOptions::default().compression_method(CompressionMethod::Deflated)
  } else {
    SimpleFileOptions::default().compression_method(CompressionMethod::Stored)
  }
}

/// Check whether a path is inside `base`, and reject anything escaping it.
fn safe_join(base: &Path, rel: &str) -> Option<std::path::PathBuf> {
  let normalized = normalize_relative_path(rel)?;
  Some(base.join(normalized.replace('/', std::path::MAIN_SEPARATOR_STR)))
}

/// Recursively collect (relative_path, absolute_path) for every file under `base`.
fn walk_files(base: &Path, dir: &Path, out: &mut Vec<(String, PathBuf)>) -> std::io::Result<()> {
  let read_dir = match fs::read_dir(dir) {
    Ok(rd) => rd,
    Err(_) => return Ok(()),
  };
  let mut entries: Vec<_> = read_dir.flatten().collect();
  entries.sort_by_key(|entry| entry.path());
  for entry in entries {
    let path = entry.path();
    let file_type = match entry.file_type() {
      Ok(t) => t,
      Err(_) => continue,
    };
    if file_type.is_symlink() {
      continue;
    }
    if file_type.is_dir() {
      walk_files(base, &path, out)?;
      continue;
    }
    let rel = match path.strip_prefix(base) {
      Ok(r) => r.to_string_lossy().replace('\\', "/"),
      Err(_) => continue,
    };
    out.push((rel, path));
  }
  Ok(())
}

fn is_core_file(name: &str, rel: &str) -> bool {
  let first = rel.split('/').next().unwrap_or(rel);
  if CORE_INSTANCE_FILES.contains(&first) {
    return true;
  }
  // The client jar and client json are named after the instance and are required
  // for the instance to be recognized and launched after import.
  rel == format!("{}.jar", name) || rel == format!("{}.json", name)
}

/// Scan an instance for the full pack export.
///
/// The scan base is always the version path (`versions/<name>/`) so the produced
/// zip has a uniform, version-isolated layout. When version isolation is off,
/// shared directories at the game dir root (`mods/`, `config/`, ...) are merged
/// into the listing under their plain directory names.
pub fn scan_instance_for_full_export(
  version_path: &Path,
  game_dir: &Path,
  version_isolation: bool,
) -> std::io::Result<Vec<ExportFileEntry>> {
  let name = version_path
    .file_name()
    .map(|n| n.to_string_lossy().to_string())
    .unwrap_or_default();

  let mut entries = scan_dir(version_path, version_path)?;
  entries.retain(|entry| {
    let first = entry.relative_path.split('/').next().unwrap_or("");
    let ignored_dir = IGNORED_INSTANCE_DIR_PREFIXES
      .iter()
      .any(|prefix| first.starts_with(prefix));
    !ignored_dir && !is_core_file(&name, &entry.relative_path)
  });

  if !version_isolation {
    for dir in SHARED_GAME_DIRS {
      let shared = game_dir.join(dir);
      if !shared.is_dir() {
        continue;
      }
      // Skip when the instance root already owns this directory (isolation is off
      // for game content but the version dir may still hold some leftovers).
      if version_path.join(dir).exists() {
        continue;
      }
      let mut children = scan_dir(&shared, &shared)?;
      for child in &mut children {
        child.relative_path = format!("{}/{}", dir, child.relative_path);
      }
      entries.extend(children);
    }
    for file in SHARED_GAME_FILES {
      let shared = game_dir.join(file);
      if shared.is_file() {
        let metadata = fs::metadata(&shared)?;
        entries.push(ExportFileEntry {
          relative_path: file.to_string(),
          is_directory: false,
          category: categorize(file, false),
          file_size: metadata.len(),
        });
      }
    }
  }

  Ok(entries)
}

/// Map a selected relative path to its actual source file.
///
/// The scan base is the version path, but when version isolation is off the
/// shared directories live at the game dir root, so try both locations.
fn selected_source_path(
  version_path: &Path,
  game_dir: &Path,
  rel: &str,
  version_isolation: bool,
) -> Option<PathBuf> {
  let in_version = safe_join(version_path, rel)?;
  if in_version.is_file() {
    return Some(in_version);
  }
  if !version_isolation {
    let in_game_dir = safe_join(game_dir, rel)?;
    if in_game_dir.is_file() {
      return Some(in_game_dir);
    }
  }
  None
}

/// Export an instance as a self-contained full pack (.zip).
///
/// Zip layout:
/// ```text
/// <zip>/
/// ├── xmcl-full-pack.json   # identification manifest
/// ├── instance/<name>/      # version-isolated instance content
/// ├── libraries/            # game dir shared libraries
/// └── assets/               # game dir shared assets (indexes + objects)
/// ```
/// After import nothing needs to be downloaded: the game jar, libraries and
/// assets are all inside the pack, so launch validation passes offline.
pub fn export_full_pack(
  app: &AppHandle,
  version_path: &Path,
  game_dir: &Path,
  meta: &ExportModpackMeta,
  mc_version: &str,
  mod_loader_type: &ModLoaderType,
  mod_loader_version: &str,
  version_isolation: bool,
  selected_files: &[String],
  output_path: &Path,
) -> XMCLResult<()> {
  let name = version_path
    .file_name()
    .map(|n| n.to_string_lossy().to_string())
    .unwrap_or_default();

  let output_file = fs::File::create(output_path).map_err(|_| InstanceError::FileCreationFailed)?;
  let mut zip = ZipWriter::new(output_file);

  // --- Collect instance content (user selection + mandatory core files) ---
  let mut pack_files: Vec<(String, PathBuf)> = Vec::new();

  // Mandatory files that make the instance recognizable and launchable.
  for core in CORE_INSTANCE_FILES {
    let abs = version_path.join(core);
    if abs.is_file() {
      pack_files.push((core.to_string(), abs));
    }
  }
  for core in [format!("{}.jar", name), format!("{}.json", name)] {
    let abs = version_path.join(&core);
    if abs.is_file() {
      pack_files.push((core, abs));
    }
  }

  // User selection, deduplicated against the core files.
  let mut seen: std::collections::HashSet<String> =
    pack_files.iter().map(|(rel, _)| rel.clone()).collect();
  for rel in selected_files {
    let Some(rel) = normalize_relative_path(rel) else {
      continue;
    };
    if !seen.insert(rel.clone()) {
      continue;
    }
    if is_core_file(&name, &rel) {
      continue;
    }
    if let Some(abs) = selected_source_path(version_path, game_dir, &rel, version_isolation) {
      pack_files.push((rel, abs));
    }
  }

  let total_size: u64 = pack_files
    .iter()
    .filter_map(|(_, abs)| fs::metadata(abs).ok().map(|m| m.len()))
    .sum();

  // --- Write identification manifest ---
  let manifest = FullPackManifest {
    format_version: 1,
    name: name.clone(),
    version: meta.version.clone(),
    description: meta.description.clone(),
    author: Some(meta.author.clone()),
    minecraft_version: mc_version.to_string(),
    mod_loader: ModLoader {
      loader_type: mod_loader_type.clone(),
      version: mod_loader_version.to_string(),
      ..Default::default()
    },
    file_count: pack_files.len(),
    total_size,
  };
  let manifest_json = serde_json::to_string_pretty(&manifest)
    .map_err(|_| InstanceError::ModpackManifestParseError)?;
  zip
    .start_file(
      "xmcl-full-pack.json",
      compression_options("xmcl-full-pack.json"),
    )
    .map_err(|_| InstanceError::ZipFileProcessFailed)?;
  zip
    .write_all(manifest_json.as_bytes())
    .map_err(|_| InstanceError::ZipFileProcessFailed)?;

  // --- Pack instance content ---
  let total = pack_files.len();
  for (i, (rel, abs)) in pack_files.iter().enumerate() {
    let file_name = rel.rsplit('/').next().unwrap_or(rel);
    emit_progress(app, i + 1, total, file_name, ExportStage::Packing);
    let zip_path = format!("instance/{}/{}", name, rel);
    zip
      .start_file(&zip_path, compression_options(&rel))
      .map_err(|_| InstanceError::ZipFileProcessFailed)?;
    let data = fs::read(abs).map_err(|_| InstanceError::FileNotFoundError)?;
    zip
      .write_all(&data)
      .map_err(|_| InstanceError::ZipFileProcessFailed)?;
  }

  // --- Pack shared game libraries ---
  let libs_dir = game_dir.join("libraries");
  let mut lib_files = Vec::new();
  if libs_dir.is_dir() {
    walk_files(&libs_dir, &libs_dir, &mut lib_files)?;
  }
  let lib_total = lib_files.len();
  for (i, (rel, abs)) in lib_files.iter().enumerate() {
    if i % 25 == 0 || i + 1 == lib_total {
      emit_progress(
        app,
        i + 1,
        lib_total,
        rel.rsplit('/').next().unwrap_or(rel),
        ExportStage::Copying,
      );
    }
    let zip_path = format!("libraries/{}", rel);
    zip
      .start_file(&zip_path, compression_options(rel))
      .map_err(|_| InstanceError::ZipFileProcessFailed)?;
    let data = fs::read(abs).map_err(|_| InstanceError::FileNotFoundError)?;
    zip
      .write_all(&data)
      .map_err(|_| InstanceError::ZipFileProcessFailed)?;
  }

  // --- Pack shared game assets ---
  let assets_dir = game_dir.join("assets");
  let mut asset_files = Vec::new();
  if assets_dir.is_dir() {
    walk_files(&assets_dir, &assets_dir, &mut asset_files)?;
  }
  let asset_total = asset_files.len();
  for (i, (rel, abs)) in asset_files.iter().enumerate() {
    if i % 25 == 0 || i + 1 == asset_total {
      emit_progress(
        app,
        i + 1,
        asset_total,
        rel.rsplit('/').next().unwrap_or(rel),
        ExportStage::Copying,
      );
    }
    let zip_path = format!("assets/{}", rel);
    zip
      .start_file(&zip_path, compression_options(rel))
      .map_err(|_| InstanceError::ZipFileProcessFailed)?;
    let data = fs::read(abs).map_err(|_| InstanceError::FileNotFoundError)?;
    zip
      .write_all(&data)
      .map_err(|_| InstanceError::ZipFileProcessFailed)?;
  }

  zip
    .finish()
    .map_err(|_| InstanceError::ZipFileProcessFailed)?;

  emit_progress(app, 0, 0, "", ExportStage::Done);

  Ok(())
}

/// Import a full pack into the given game directory.
///
/// Pure extraction — nothing is downloaded. Returns the new instance id
/// (`<gameDirectoryName>:<instanceName>`).
pub fn import_full_pack(directory: &GameDirectory, pack_path: &Path) -> XMCLResult<String> {
  let file = fs::File::open(pack_path).map_err(|_| InstanceError::FileNotFoundError)?;
  let mut archive = ZipArchive::new(file).map_err(|_| InstanceError::ModpackManifestParseError)?;

  // Read the identification manifest.
  let mut manifest_str = String::new();
  archive
    .by_name("xmcl-full-pack.json")
    .map_err(|_| InstanceError::ModpackManifestParseError)?
    .read_to_string(&mut manifest_str)
    .map_err(|_| InstanceError::ModpackManifestParseError)?;
  let manifest: FullPackManifest =
    serde_json::from_str(&manifest_str).map_err(|_| InstanceError::ModpackManifestParseError)?;
  if manifest.format_version != 1 {
    return Err(InstanceError::ModpackManifestParseError.into());
  }

  let name = manifest.name.clone();
  let version_path = directory.dir.join("versions").join(&name);
  if version_path.exists() {
    return Err(InstanceError::ConflictNameError.into());
  }

  let instance_prefix = format!("instance/{}/", name);
  let libraries_dir = directory.dir.join("libraries");
  let assets_dir = directory.dir.join("assets");

  for i in 0..archive.len() {
    let mut entry = archive
      .by_index(i)
      .map_err(|_| InstanceError::ZipFileProcessFailed)?;
    if entry.is_dir() {
      continue;
    }
    let entry_name = entry.name().to_string();
    let target: Option<PathBuf> = if let Some(rel) = entry_name.strip_prefix(&instance_prefix) {
      safe_join(&version_path, rel)
    } else if let Some(rel) = entry_name.strip_prefix("libraries/") {
      safe_join(&libraries_dir, rel)
    } else if let Some(rel) = entry_name.strip_prefix("assets/") {
      safe_join(&assets_dir, rel)
    } else {
      None
    };

    let Some(target) = target else { continue };

    // Shared layers are merged: skip files that already exist.
    if target.exists() {
      continue;
    }
    if let Some(parent) = target.parent() {
      fs::create_dir_all(parent).map_err(|_| InstanceError::FolderCreationFailed)?;
    }
    let mut out = fs::File::create(&target).map_err(|_| InstanceError::FileCreationFailed)?;
    std::io::copy(&mut entry, &mut out).map_err(|_| InstanceError::ZipFileProcessFailed)?;
  }

  // Rewrite the instance id so it matches the target game directory.
  rewrite_instance_id(&version_path, &format!("{}:{}", directory.name, name))?;

  Ok(format!("{}:{}", directory.name, name))
}

/// Update the `id` field of `xmclcfg.json` to the target directory's composed id.
fn rewrite_instance_id(version_path: &Path, new_id: &str) -> XMCLResult<()> {
  let cfg_path = version_path.join("xmclcfg.json");
  if !cfg_path.is_file() {
    return Ok(());
  }
  let raw = match fs::read_to_string(&cfg_path) {
    Ok(r) => r,
    Err(_) => return Ok(()),
  };
  let mut value: serde_json::Value = match serde_json::from_str(&raw) {
    Ok(v) => v,
    Err(_) => return Ok(()),
  };
  if let Some(obj) = value.as_object_mut() {
    obj.insert(
      "id".to_string(),
      serde_json::Value::String(new_id.to_string()),
    );
  }
  let _ = fs::write(
    &cfg_path,
    serde_json::to_string_pretty(&value).unwrap_or(raw),
  );
  Ok(())
}

use crate::error::XMCLResult;
use crate::instance::helpers::modpack::export_common::{
  emit_progress, is_remotely_matchable, selected_file_paths, sha1_of_file, sha512_of_file,
  ExportStage,
};
use crate::instance::models::misc::{ExportModpackMeta, InstanceError, ModLoaderType};
use crate::resource::helpers::curseforge::fetch_remote_resource_by_local_curseforge;
use crate::resource::helpers::modrinth::fetch_remote_resource_by_local_modrinth;
use futures::stream::{self, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;
use tauri::AppHandle;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

const REMOTE_MATCH_CONCURRENCY: usize = 8;

enum ModrinthMatchResult {
  Remote(ModrinthExportFile),
  Local(String),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ModrinthExportHashes {
  sha1: String,
  sha512: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ModrinthExportEnv {
  client: String,
  server: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ModrinthExportFile {
  path: String,
  hashes: ModrinthExportHashes,
  #[serde(skip_serializing_if = "Option::is_none")]
  env: Option<ModrinthExportEnv>,
  downloads: Vec<String>,
  file_size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ModrinthExportManifest {
  format_version: u32,
  game: String,
  version_id: String,
  name: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  summary: Option<String>,
  files: Vec<ModrinthExportFile>,
  dependencies: HashMap<String, String>,
}

/// Export an instance as a Modrinth .mrpack file.
///
/// `selected_files` is a list of relative paths (using `/` separators) that the
/// user has chosen to include.  Files in remotely-matchable directories
/// (mods/, resourcepacks/, shaderpacks/) are looked up on Modrinth; those that
/// match are stored as remote downloads.  All other selected files are packed
/// into client-overrides/.
pub async fn export_modrinth(
  app: &AppHandle,
  instance_path: &Path,
  meta: &ExportModpackMeta,
  mc_version: &str,
  mod_loader_type: &ModLoaderType,
  mod_loader_version: &str,
  selected_files: &[String],
  output_path: &Path,
) -> XMCLResult<()> {
  let output_file = fs::File::create(output_path).map_err(|_| InstanceError::FileCreationFailed)?;
  let mut zip = ZipWriter::new(output_file);

  let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

  let file_list = selected_file_paths(instance_path, selected_files)
    .map_err(|_| InstanceError::FileNotFoundError)?;
  let total = file_list.len();

  // Check export options
  let no_create_remote_files = meta.no_create_remote_files.unwrap_or(false);
  let skip_curseforge = meta.skip_curseforge_remote_files.unwrap_or(false);

  // Attempt remote matching for matchable files.
  let mut matched_count = 0usize;
  let match_results: Vec<ModrinthMatchResult> = if no_create_remote_files {
    // Skip all remote matching - all files go to local overrides
    // Emit initial progress
    emit_progress(app, 0, total, "Preparing files...", ExportStage::Matching);
    file_list
      .iter()
      .map(|(rel_path, _)| {
        matched_count += 1;
        let file_name = rel_path.rsplit('/').next().unwrap_or(rel_path);
        emit_progress(app, matched_count, total, file_name, ExportStage::Matching);
        ModrinthMatchResult::Local(rel_path.clone())
      })
      .collect()
  } else {
    // Normal remote matching
    stream::iter(file_list.iter().cloned())
      .map(|(rel_path, abs_path)| {
        let app = app.clone();
        let skip_curseforge = skip_curseforge;
        async move {
          if !is_remotely_matchable(&rel_path) {
            return ModrinthMatchResult::Local(rel_path);
          }

          let abs_str = abs_path.to_string_lossy().to_string();

          // First try Modrinth
          let mut downloads = Vec::new();
          if let Ok(file_info) = fetch_remote_resource_by_local_modrinth(&app, &abs_str).await {
            downloads.push(file_info.download_url.clone());
          }

          // Then try CurseForge if not skipped
          if !skip_curseforge {
            if let Ok(file_info) = fetch_remote_resource_by_local_curseforge(&app, &abs_str).await {
              downloads.push(file_info.download_url.clone());
            }
          }

          if downloads.is_empty() {
            return ModrinthMatchResult::Local(rel_path);
          }

          let sha1 = sha1_of_file(&abs_path).unwrap_or_default();
          let sha512 = sha512_of_file(&abs_path).unwrap_or_default();
          let file_size = fs::metadata(&abs_path).map(|m| m.len()).unwrap_or(0);
          let env = if rel_path.ends_with(".disabled") {
            Some(ModrinthExportEnv {
              client: "optional".to_string(),
              server: "unsupported".to_string(),
            })
          } else {
            None
          };

          ModrinthMatchResult::Remote(ModrinthExportFile {
            path: rel_path,
            hashes: ModrinthExportHashes { sha1, sha512 },
            env,
            downloads,
            file_size,
          })
        }
      })
      .buffer_unordered(REMOTE_MATCH_CONCURRENCY)
      .then(|result| {
        matched_count += 1;
        let file_name = match &result {
          ModrinthMatchResult::Remote(file) => file.path.rsplit('/').next().unwrap_or(&file.path),
          ModrinthMatchResult::Local(path) => path.rsplit('/').next().unwrap_or(path),
        };
        emit_progress(app, matched_count, total, file_name, ExportStage::Matching);
        async move { result }
      })
      .collect()
      .await
  };

  let mut remote_files: Vec<ModrinthExportFile> = Vec::new();
  let mut local_override_paths: Vec<String> = Vec::new();
  for result in match_results {
    match result {
      ModrinthMatchResult::Remote(file) => remote_files.push(file),
      ModrinthMatchResult::Local(path) => local_override_paths.push(path),
    }
  }
  // Pack client-overrides/.
  let pack_total = local_override_paths.len();
  for (i, rel_path) in local_override_paths.iter().enumerate() {
    let abs_path = instance_path.join(rel_path.replace('/', std::path::MAIN_SEPARATOR_STR));
    if !abs_path.is_file() {
      continue;
    }
    let file_name = rel_path.rsplit('/').next().unwrap_or(rel_path);
    emit_progress(app, i + 1, pack_total, file_name, ExportStage::Packing);
    let zip_path = format!("client-overrides/{}", rel_path);
    zip
      .start_file(&zip_path, options)
      .map_err(|_| InstanceError::ZipFileProcessFailed)?;
    let data = fs::read(&abs_path).map_err(|_| InstanceError::FileNotFoundError)?;
    zip
      .write_all(&data)
      .map_err(|_| InstanceError::ZipFileProcessFailed)?;
  }

  // --- Build dependencies map ---
  let mut dependencies: HashMap<String, String> = HashMap::new();
  dependencies.insert("minecraft".to_string(), mc_version.to_string());
  let loader_key = match mod_loader_type {
    ModLoaderType::Fabric => Some("fabric-loader"),
    ModLoaderType::Forge | ModLoaderType::LegacyForge => Some("forge"),
    ModLoaderType::NeoForge => Some("neoforge"),
    ModLoaderType::Quilt => Some("quilt-loader"),
    _ => None,
  };
  if let Some(key) = loader_key {
    if !mod_loader_version.is_empty() {
      dependencies.insert(key.to_string(), mod_loader_version.to_string());
    }
  }

  // --- Write modrinth.index.json ---
  emit_progress(
    app,
    0,
    0,
    "modrinth.index.json",
    ExportStage::WritingManifest,
  );
  let manifest = ModrinthExportManifest {
    format_version: 1,
    game: "minecraft".to_string(),
    version_id: meta.version.clone(),
    name: meta.name.clone(),
    summary: meta.description.clone(),
    files: remote_files,
    dependencies,
  };

  let manifest_json = serde_json::to_string_pretty(&manifest)
    .map_err(|_| InstanceError::ModpackManifestParseError)?;

  zip
    .start_file("modrinth.index.json", options)
    .map_err(|_| InstanceError::ZipFileProcessFailed)?;
  zip
    .write_all(manifest_json.as_bytes())
    .map_err(|_| InstanceError::ZipFileProcessFailed)?;

  zip
    .finish()
    .map_err(|_| InstanceError::ZipFileProcessFailed)?;

  emit_progress(app, 0, 0, "", ExportStage::Done);

  Ok(())
}

use crate::error::XMCLResult;
use crate::instance::helpers::modpack::curseforge::{CurseForgeFiles, CurseForgeModLoader};
use crate::instance::helpers::modpack::export_common::{
  emit_progress, is_remotely_matchable, selected_file_paths, ExportStage,
};
use crate::instance::models::misc::{ExportModpackMeta, InstanceError, ModLoaderType};
use crate::resource::helpers::curseforge::misc::{
  get_curseforge_api, make_curseforge_request, CurseForgeFingerprintRes,
};
use crate::resource::models::{OtherResourceApiEndpoint, OtherResourceRequestType};
use futures::stream::{self, StreamExt};
use hex;
use murmur2::murmur2;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha1::{Digest, Sha1};
use std::fs;
use std::io::Write;
use std::path::Path;
use tauri::{AppHandle, Manager};
use tauri_plugin_http::reqwest;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;
const REMOTE_MATCH_CONCURRENCY: usize = 8;

enum CurseForgeMatchResult {
  Remote(CurseForgeFiles),
  Local(String),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurseForgeMinecraft {
  version: String,
  #[serde(rename = "modLoaders")]
  mod_loaders: Vec<CurseForgeModLoader>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurseForgeExportManifest {
  minecraft: CurseForgeMinecraft,
  manifest_type: String,
  manifest_version: u32,
  name: String,
  version: String,
  author: String,
  files: Vec<CurseForgeFiles>,
  overrides: String,
}

/// Try to look up a local file on CurseForge by murmur2 fingerprint.
/// Returns `(project_id, file_id)` on success.
async fn lookup_curseforge_ids(app: &AppHandle, file_path: &Path) -> Option<(u64, u64)> {
  let file_content = fs::read(file_path).ok()?;

  // Calculate SHA-1 for verification
  let mut sha1_hasher = Sha1::new();
  sha1_hasher.update(&file_content);
  let local_sha1 = hex::encode(sha1_hasher.finalize());

  // Murmur2 fingerprint (whitespace-filtered)
  let filtered: Vec<u8> = file_content
    .iter()
    .filter(|&&b| !matches!(b, 0x09 | 0x0a | 0x0d | 0x20))
    .copied()
    .collect();
  let fingerprint = murmur2(&filtered, 1) as u64;

  let url = get_curseforge_api(OtherResourceApiEndpoint::FromLocal, None).ok()?;
  let payload = json!({ "fingerprints": [fingerprint] });
  let client = app.state::<reqwest::Client>();

  let resp: CurseForgeFingerprintRes =
    make_curseforge_request(&client, &url, OtherResourceRequestType::Post(&payload))
      .await
      .ok()?;

  let exact = resp.data.exact_matches.first()?;
  let cf_file = &exact.file;

  // Verify SHA1
  let remote_sha1 = cf_file.hashes.iter().find(|h| h.algo == 1)?;
  if remote_sha1.value.to_lowercase() != local_sha1.to_lowercase() {
    return None;
  }

  Some((cf_file.mod_id as u64, cf_file.id as u64))
}

/// Export an instance as a CurseForge .zip modpack.
pub async fn export_curseforge(
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

  let mut matched_count = 0usize;
  let match_results: Vec<CurseForgeMatchResult> = if no_create_remote_files {
    // Skip all remote matching - all files go to local overrides
    // Emit initial progress
    emit_progress(app, 0, total, "Preparing files...", ExportStage::Matching);
    file_list
      .iter()
      .map(|(rel_path, _)| {
        matched_count += 1;
        let file_name = rel_path.rsplit('/').next().unwrap_or(rel_path);
        emit_progress(app, matched_count, total, file_name, ExportStage::Matching);
        CurseForgeMatchResult::Local(rel_path.clone())
      })
      .collect()
  } else {
    stream::iter(file_list.iter().cloned())
      .map(|(rel_path, abs_path)| {
        let app = app.clone();
        async move {
          if !is_remotely_matchable(&rel_path) {
            return CurseForgeMatchResult::Local(rel_path);
          }

          if let Some((project_id, file_id)) = lookup_curseforge_ids(&app, &abs_path).await {
            return CurseForgeMatchResult::Remote(CurseForgeFiles {
              project_id,
              file_id,
              required: true,
            });
          }

          CurseForgeMatchResult::Local(rel_path)
        }
      })
      .buffer_unordered(REMOTE_MATCH_CONCURRENCY)
      .then(|result| {
        matched_count += 1;
        let file_name = match &result {
          CurseForgeMatchResult::Remote(_) => "",
          CurseForgeMatchResult::Local(path) => path.rsplit('/').next().unwrap_or(path),
        };
        emit_progress(app, matched_count, total, file_name, ExportStage::Matching);
        async move { result }
      })
      .collect()
      .await
  };

  let mut cf_files: Vec<CurseForgeFiles> = Vec::new();
  let mut override_paths: Vec<String> = Vec::new();
  for result in match_results {
    match result {
      CurseForgeMatchResult::Remote(file) => cf_files.push(file),
      CurseForgeMatchResult::Local(path) => override_paths.push(path),
    }
  }
  // Pack overrides/
  let pack_total = override_paths.len();
  for (i, rel_path) in override_paths.iter().enumerate() {
    let abs_path = instance_path.join(rel_path.replace('/', std::path::MAIN_SEPARATOR_STR));
    if !abs_path.is_file() {
      continue;
    }
    let file_name = rel_path.rsplit('/').next().unwrap_or(rel_path);
    emit_progress(app, i + 1, pack_total, file_name, ExportStage::Packing);
    let zip_path = format!("overrides/{}", rel_path);
    zip
      .start_file(&zip_path, options)
      .map_err(|_| InstanceError::ZipFileProcessFailed)?;
    let data = fs::read(&abs_path).map_err(|_| InstanceError::FileNotFoundError)?;
    zip
      .write_all(&data)
      .map_err(|_| InstanceError::ZipFileProcessFailed)?;
  }

  // Build mod loader entry
  let loader_id_str = match mod_loader_type {
    ModLoaderType::Fabric => format!("fabric-{}", mod_loader_version),
    ModLoaderType::Forge | ModLoaderType::LegacyForge => format!("forge-{}", mod_loader_version),
    ModLoaderType::NeoForge => format!("neoforge-{}", mod_loader_version),
    ModLoaderType::Quilt => format!("quilt-{}", mod_loader_version),
    _ => String::new(),
  };

  let mod_loaders = if loader_id_str.is_empty() {
    vec![]
  } else {
    vec![CurseForgeModLoader {
      id: loader_id_str,
      primary: true,
    }]
  };

  emit_progress(app, 0, 0, "manifest.json", ExportStage::WritingManifest);
  let manifest = CurseForgeExportManifest {
    minecraft: CurseForgeMinecraft {
      version: mc_version.to_string(),
      mod_loaders,
    },
    manifest_type: "minecraftModpack".to_string(),
    manifest_version: 1,
    name: meta.name.clone(),
    version: meta.version.clone(),
    author: meta.author.clone(),
    files: cf_files,
    overrides: "overrides".to_string(),
  };

  let manifest_json = serde_json::to_string_pretty(&manifest)
    .map_err(|_| InstanceError::ModpackManifestParseError)?;

  zip
    .start_file("manifest.json", options)
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

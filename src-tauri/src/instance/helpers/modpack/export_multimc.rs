use crate::error::XMCLResult;
use crate::instance::helpers::modpack::export_common::{
  emit_progress, selected_file_paths, ExportStage,
};
use crate::instance::helpers::modpack::multimc::MultiMcComponent;
use crate::instance::models::misc::{ExportModpackMeta, InstanceError, ModLoaderType};
use serde_json;
use std::fs;
use std::io::Write;
use std::path::Path;
use tauri::AppHandle;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

fn loader_uid(loader_type: &ModLoaderType) -> Option<(&'static str, &'static str)> {
  match loader_type {
    ModLoaderType::Fabric => Some(("net.fabricmc.fabric-loader", "Fabric Loader")),
    ModLoaderType::Forge | ModLoaderType::LegacyForge => Some(("net.minecraftforge", "Forge")),
    ModLoaderType::NeoForge => Some(("net.neoforged", "NeoForge")),
    ModLoaderType::Quilt => Some(("org.quiltmc.quilt-loader", "Quilt Loader")),
    _ => None,
  }
}

/// Export an instance as a MultiMC .zip modpack.
pub fn export_multimc(
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

  // Build components
  let mut components: Vec<MultiMcComponent> = vec![MultiMcComponent {
    uid: "net.minecraft".to_string(),
    version: Some(mc_version.to_string()),
    cached_name: Some("Minecraft".to_string()),
    cached_version: Some(mc_version.to_string()),
    important: Some(true),
    cached_requires: None,
    cached_volatile: None,
    dependency_only: None,
  }];

  if let Some((uid, name)) = loader_uid(mod_loader_type) {
    if !mod_loader_version.is_empty() {
      components.push(MultiMcComponent {
        uid: uid.to_string(),
        version: Some(mod_loader_version.to_string()),
        cached_name: Some(name.to_string()),
        cached_version: Some(mod_loader_version.to_string()),
        important: None,
        cached_requires: None,
        cached_volatile: None,
        dependency_only: None,
      });
    }
  }

  // mmc-pack.json (reuse existing struct minus #[serde(skip)] fields)
  let mmc_pack = serde_json::json!({
    "components": components,
    "formatVersion": 1
  });
  let mmc_pack_json = serde_json::to_string_pretty(&mmc_pack)
    .map_err(|_| InstanceError::ModpackManifestParseError)?;

  zip
    .start_file("mmc-pack.json", options)
    .map_err(|_| InstanceError::ZipFileProcessFailed)?;
  zip
    .write_all(mmc_pack_json.as_bytes())
    .map_err(|_| InstanceError::ZipFileProcessFailed)?;

  // instance.cfg
  let min_mem = meta.min_memory.unwrap_or(512);
  let max_mem = std::cmp::max(min_mem * 2, 4096);
  let instance_cfg = format!(
    "InstanceType=OneSix\nname={}\nJvmArgs=\nMaxMemAlloc={}\nMinMemAlloc={}\n",
    meta.name, max_mem, min_mem
  );
  zip
    .start_file("instance.cfg", options)
    .map_err(|_| InstanceError::ZipFileProcessFailed)?;
  zip
    .write_all(instance_cfg.as_bytes())
    .map_err(|_| InstanceError::ZipFileProcessFailed)?;

  // .packignore (empty marker file)
  zip
    .start_file(".packignore", options)
    .map_err(|_| InstanceError::ZipFileProcessFailed)?;

  emit_progress(app, 0, 0, "mmc-pack.json", ExportStage::WritingManifest);

  // Pack all selected files into .minecraft/.
  let file_list = selected_file_paths(instance_path, selected_files)
    .map_err(|_| InstanceError::FileNotFoundError)?;
  let total = file_list.len();

  for (i, (rel_path, abs_path)) in file_list.iter().enumerate() {
    let file_name = rel_path.rsplit('/').next().unwrap_or(rel_path);
    emit_progress(app, i + 1, total, file_name, ExportStage::Packing);
    let zip_path = format!(".minecraft/{}", rel_path);
    zip
      .start_file(&zip_path, options)
      .map_err(|_| InstanceError::ZipFileProcessFailed)?;
    let data = fs::read(&abs_path).map_err(|_| InstanceError::FileNotFoundError)?;
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

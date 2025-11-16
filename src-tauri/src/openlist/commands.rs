use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_http::reqwest;

use crate::{
  error::XMCLError as Error,
  launcher_config::models::LauncherConfig,
  tasks::{commands::schedule_progressive_task_group, download::DownloadParam, PTaskParam},
};

const OPENLIST_BASE_URL: &str = env!("XMCL_OPENLIST_BASE_URL");

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenListResponse {
  pub code: i32,
  pub message: String,
  pub data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BrowseRequest {
  pub path: String,
  pub page: Option<i32>,
  #[serde(rename = "perPage")]
  pub per_page: Option<i32>,
}

#[tauri::command]
pub async fn openlist_browse(request: BrowseRequest) -> Result<OpenListResponse, Error> {
  println!(
    "[Rust] OpenList browse request: path={}, page={:?}, perPage={:?}",
    request.path, request.page, request.per_page
  );

  let client = reqwest::Client::new();

  let response = client
    .post(format!("{}/api/fs/list", OPENLIST_BASE_URL))
    .json(&serde_json::json!({
      "path": request.path,
      "page": request.page.unwrap_or(1),
      "per_page": request.per_page.unwrap_or(100)
    }))
    .send()
    .await
    .map_err(|e| Error(format!("API 请求失败: {}", e)))?;

  println!("[Rust] OpenList API response status: {}", response.status());

  let result: OpenListResponse = response
    .json()
    .await
    .map_err(|e| Error(format!("解析响应失败: {}", e)))?;

  println!(
    "[Rust] OpenList API response code: {}, items: {}",
    result.code,
    result
      .data
      .get("content")
      .and_then(|c| c.as_array())
      .map(|a| a.len())
      .unwrap_or(0)
  );

  Ok(result)
}

#[tauri::command]
pub async fn openlist_download_modpack(
  file_name: String,
  file_path: String,
  _sign: String,
  size: u64,
  app: AppHandle,
  launcher_config: tauri::State<'_, std::sync::Mutex<LauncherConfig>>,
) -> Result<String, Error> {
  println!("[Rust] Starting OpenList modpack download:");
  println!("  - Name: {}", file_name);
  println!("  - Path: {}", file_path);
  println!("  - Size: {} bytes", size);

  // 获取下载缓存目录（使用用户配置的下载目录）
  let download_cache_dir = {
    let config = launcher_config.lock().unwrap();
    config.download.cache.directory.clone()
  };

  // 创建 OpenList 子目录
  let temp_dir = download_cache_dir.join("openlist_downloads");
  tokio::fs::create_dir_all(&temp_dir).await?;
  println!("[Rust] Download directory: {:?}", temp_dir);

  // 目标文件路径
  let dest_file = temp_dir.join(&file_name);

  // 检查是否已完成下载
  if dest_file.exists() {
    if let Ok(metadata) = tokio::fs::metadata(&dest_file).await {
      if metadata.len() == size {
        println!("[Rust] File already exists with matching size, skipping download");
        // 返回 JSON 格式,与下载完成时的返回格式保持一致
        let result = serde_json::json!({
          "path": dest_file.to_string_lossy().to_string(),
          "taskGroupPrefix": null, // 文件已存在,无需任务组
        });
        return serde_json::to_string(&result)
          .map_err(|e| Error(format!("序列化 JSON 失败: {}", e)));
      }
    }
  }

  // 获取真实的下载 URL（raw_url）
  println!("[Rust] Fetching real download URL from API...");
  let client = reqwest::Client::new();

  let get_response = client
    .post(format!("{}/api/fs/get", OPENLIST_BASE_URL))
    .json(&serde_json::json!({
      "path": file_path,
    }))
    .send()
    .await
    .map_err(|e| Error(format!("获取文件信息失败: {}", e)))?;

  let get_result: OpenListResponse = get_response
    .json()
    .await
    .map_err(|e| Error(format!("解析文件信息失败: {}", e)))?;

  if get_result.code != 200 {
    return Err(Error(format!(
      "获取文件信息失败: {} ({})",
      get_result.message, get_result.code
    )));
  }

  // 从响应中提取 raw_url
  let raw_url = get_result
    .data
    .get("raw_url")
    .and_then(|v| v.as_str())
    .ok_or_else(|| Error("API 响应中未找到 raw_url 字段".to_string()))?;

  println!("[Rust] Real download URL: {}", raw_url);

  // 使用现有的任务系统创建下载任务
  // 注意：必须使用绝对路径，否则任务系统会解析到默认下载目录
  let download_param = DownloadParam {
    src: raw_url
      .parse()
      .map_err(|e| Error(format!("无效的下载 URL: {}", e)))?,
    dest: dest_file.clone(), // 使用完整路径
    filename: Some(file_name.clone()),
    sha1: None, // OpenList 暂不验证 SHA1
  };

  // 创建任务组（使用文件名作为任务组名）
  let task_group_prefix = format!("openlist:{}", file_name);

  let task_group_desc = schedule_progressive_task_group(
    app.clone(),
    task_group_prefix.clone(),
    vec![PTaskParam::Download(download_param)],
    true, // 使用时间戳确保唯一性
  )
  .await
  .map_err(|e| Error(format!("创建下载任务失败: {:?}", e)))?;

  println!(
    "[Rust] Download task created: task_group={}",
    task_group_desc.task_group
  );
  println!(
    "[Rust] Task group prefix (for frontend): {}",
    task_group_prefix
  );

  // 返回一个 JSON 对象，包含文件路径和任务组前缀
  // 前端需要用这个前缀来匹配完成事件
  // 使用 serde_json::to_string 确保路径中的反斜杠被正确转义
  let result = serde_json::json!({
    "path": dest_file.to_string_lossy().to_string(),
    "taskGroupPrefix": task_group_prefix,
  });

  serde_json::to_string(&result).map_err(|e| Error(format!("序列化 JSON 失败: {}", e)))
}

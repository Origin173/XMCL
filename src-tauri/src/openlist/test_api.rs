use serde_json::json;
use tauri::AppHandle;
use tauri::Manager;
use tauri_plugin_http::reqwest;

const OPENLIST_BASE_URL: &str = env!("XMCL_OPENLIST_BASE_URL");

#[tauri::command]
pub async fn test_openlist_connection(app: AppHandle) -> Result<String, String> {
  let url = format!("{}/api/fs/list", OPENLIST_BASE_URL);
  let client = app.state::<reqwest::Client>().inner().clone();

  let body = json!({
    "path": "/",
    "page": 1,
    "per_page": 20
  });

  match client
    .post(url)
    .header("Content-Type", "application/json")
    .json(&body)
    .send()
    .await
  {
    Ok(response) => {
      let status = response.status();
      let text = response
        .text()
        .await
        .unwrap_or_else(|_| "Failed to read body".to_string());
      Ok(format!("Status: {}\nBody: {}", status, text))
    }
    Err(e) => Err(format!("Request failed: {}", e)),
  }
}

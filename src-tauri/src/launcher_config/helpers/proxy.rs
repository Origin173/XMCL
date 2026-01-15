use crate::error::XMCLResult;
use crate::launcher_config::models::{LauncherConfig, ProxyType};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Manager};
use tauri_plugin_http::reqwest;

pub async fn test_proxy_connectivity(
  app: &AppHandle,
  test_url: String,
) -> XMCLResult<bool> {
  let config_binding = app.state::<Mutex<LauncherConfig>>();
  let (proxy_enabled, proxy_type, proxy_host, proxy_port) = {
    let config = config_binding.lock()?;
    (
      config.download.proxy.enabled,
      config.download.proxy.selected_type.clone(),
      config.download.proxy.host.clone(),
      config.download.proxy.port,
    )
  };

  if !proxy_enabled || proxy_host.is_empty() || proxy_port == 0 {
    return Ok(false);
  }

  let proxy_url = match proxy_type {
    ProxyType::Http => format!("http://{}:{}", proxy_host, proxy_port),
    ProxyType::Socks => format!("socks5://{}:{}", proxy_host, proxy_port),
  };

  let proxy = reqwest::Proxy::all(&proxy_url).map_err(|_| "Invalid proxy configuration")?;

  let client = reqwest::Client::builder()
    .proxy(proxy)
    .timeout(Duration::from_secs(10))
    .build()
    .map_err(|_| "Failed to build HTTP client")?;

  match client.get(&test_url).send().await {
    Ok(response) => Ok(response.status().is_success()),
    Err(_) => Ok(false),
  }
}

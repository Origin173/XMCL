use crate::error::{XMCLError, XMCLResult};
use crate::launcher_config::models::{LauncherConfig, ProxyType};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Manager};
use tauri_plugin_http::reqwest;

pub async fn test_proxy_connectivity(app: &AppHandle, test_url: String) -> XMCLResult<bool> {
  let config_binding = app.state::<Mutex<LauncherConfig>>();
  let (proxy_enabled, follow_system_proxy, proxy_type, proxy_host, proxy_port) = {
    let config = config_binding.lock()?;
    (
      config.download.proxy.enabled,
      config.download.proxy.follow_system_proxy,
      config.download.proxy.selected_type.clone(),
      config.download.proxy.host.clone(),
      config.download.proxy.port,
    )
  };

  if !proxy_enabled {
    return Ok(false);
  }

  let client_builder = reqwest::Client::builder().timeout(Duration::from_secs(10));

  // If follow_system_proxy is enabled, use system proxy
  // Otherwise use manual proxy settings
  let client = if follow_system_proxy {
    client_builder.build().map_err(|e| XMCLError::from(e))?
  } else {
    if proxy_host.is_empty() || proxy_port == 0 {
      return Ok(false);
    }

    let proxy_url = match proxy_type {
      ProxyType::Http => format!("http://{}:{}", proxy_host, proxy_port),
      ProxyType::Socks => format!("socks5://{}:{}", proxy_host, proxy_port),
    };

    let proxy = reqwest::Proxy::all(&proxy_url).map_err(|e| XMCLError::from(e))?;

    client_builder
      .proxy(proxy)
      .build()
      .map_err(|e| XMCLError::from(e))?
  };

  match client.get(&test_url).send().await {
    Ok(response) => Ok(response.status().is_success()),
    Err(_) => Ok(false),
  }
}

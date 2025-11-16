use crate::account::helpers::authlib_injector::common::{parse_profile, retrieve_profile};
use crate::account::models::{AccountError, PlayerInfo};
use crate::error::XMCLResult;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::AppHandle;
use tauri_plugin_http::reqwest::{self, Client};
use urlencoding::decode;

const CAUC_BASE_URL: &str = "https://skin.cauc.fun";

async fn get_player_name(client: &Client, cookie_jar: &[(String, String)]) -> XMCLResult<String> {
  let user_page_url = format!("{}/user", CAUC_BASE_URL);

  // 构建 Cookie header
  let cookie_header = cookie_jar
    .iter()
    .map(|(k, v)| format!("{}={}", k, v))
    .collect::<Vec<_>>()
    .join("; ");

  let response = client
    .get(&user_page_url)
    .header("Cookie", cookie_header)
    .send()
    .await
    .map_err(|_| AccountError::NetworkError)?;

  let html = response
    .text()
    .await
    .map_err(|_| AccountError::NetworkError)?;

  // 使用正则提取 data-mark="nickname">玩家名<
  let re = regex::Regex::new(r#"data-mark="nickname">([^<]+)<"#).unwrap();
  if let Some(caps) = re.captures(&html) {
    if let Some(name) = caps.get(1) {
      return Ok(name.as_str().trim().to_string());
    }
  }

  Err(AccountError::Invalid.into())
}

const CAUC_YGGDRASIL_URL: &str = "https://skin.cauc.fun/api/yggdrasil";

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct CAUCLoginResponse {
  success: bool,
  #[serde(default)]
  message: String,
  #[serde(default)]
  requires_bind: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct CAUCBindRequest {
  player_name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct CAUCAuthResponse {
  access_token: String,
  selected_profile: Option<YggdrasilProfile>,
  available_profiles: Option<Vec<YggdrasilProfile>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct YggdrasilProfile {
  id: String,
  name: String,
}

#[derive(Debug, Clone)]
pub struct CAUCAuthState {
  pub student_id: String,
  pub cookie_jar: Vec<(String, String)>,
  pub requires_bind: bool,
  pub xsrf_token: Option<String>,
  pub client: reqwest::Client,
  pub player_name: Option<String>,
}

pub async fn eduroam_login(
  _app: &AppHandle,
  student_id: String,
  oa_password: String,
) -> XMCLResult<CAUCAuthState> {
  log::info!("CAUC eduroam login attempt for student: {}", student_id);

  // 创建一个不自动跟随重定向的客户端
  let client = reqwest::Client::builder()
    .redirect(reqwest::redirect::Policy::none())
    .cookie_store(true)
    .build()
    .map_err(|e| {
      log::error!("Failed to build HTTP client: {}", e);
      AccountError::NetworkError
    })?;

  // 步骤 1: 先访问登录页面获取 CSRF token
  let login_page_url = format!("{}/auth/eduroam/login", CAUC_BASE_URL);
  log::info!("Fetching login page: {}", login_page_url);

  let page_response = client.get(&login_page_url).send().await.map_err(|e| {
    log::error!("Failed to fetch login page: {}", e);
    AccountError::NetworkError
  })?;

  // 保存 cookies
  let mut cookie_jar = Vec::new();
  for cookie_header in page_response.headers().get_all("set-cookie").iter() {
    if let Ok(cookie_str) = cookie_header.to_str() {
      log::info!("Received cookie from login page: {}", cookie_str);
      if let Some((key, rest)) = cookie_str.split_once('=') {
        if let Some((value, _)) = rest.split_once(';') {
          cookie_jar.push((key.trim().to_string(), value.trim().to_string()));
        }
      }
    }
  }

  // 从 HTML 中提取 CSRF token
  let page_html = page_response.text().await.map_err(|e| {
    log::error!("Failed to read login page HTML: {}", e);
    AccountError::NetworkError
  })?;

  let csrf_token = page_html
    .split("name=\"_token\" value=\"")
    .nth(1)
    .and_then(|s| s.split('"').next())
    .ok_or_else(|| {
      log::error!("Failed to extract CSRF token from login page");
      AccountError::ParseError
    })?;

  log::info!("Extracted CSRF token: {}", csrf_token);

  // 步骤 2: 使用正确的字段名提交登录
  let cookie_header = cookie_jar
    .iter()
    .map(|(k, v)| format!("{}={}", k, v))
    .collect::<Vec<_>>()
    .join("; ");

  log::info!("Submitting login with cookie: {}", cookie_header);

  let response = client
    .post(&login_page_url)
    .header("Cookie", cookie_header.clone())
    .form(&[
      ("student_number", student_id.as_str()),
      ("password", oa_password.as_str()),
      ("_token", csrf_token),
    ])
    .send()
    .await
    .map_err(|e| {
      log::error!("CAUC eduroam login network error: {}", e);
      AccountError::NetworkError
    })?;

  // 更新 cookie jar
  for cookie_header_val in response.headers().get_all("set-cookie").iter() {
    if let Ok(cookie_str) = cookie_header_val.to_str() {
      log::info!("Received cookie from login response: {}", cookie_str);
      if let Some((key, rest)) = cookie_str.split_once('=') {
        if let Some((value, _)) = rest.split_once(';') {
          // 更新或添加 cookie
          if let Some(existing) = cookie_jar.iter_mut().find(|(k, _)| k == key.trim()) {
            existing.1 = value.trim().to_string();
          } else {
            cookie_jar.push((key.trim().to_string(), value.trim().to_string()));
          }
        }
      }
    }
  }

  let status = response.status();
  log::info!("CAUC eduroam login response status: {}", status);

  // 记录Location头用于调试重定向
  if let Some(location_header) = response.headers().get("location") {
    log::info!("Location header: {:?}", location_header);
  }

  // Laravel 登录成功通常是 302 重定向
  // 记录最新的 XSRF token
  let xsrf_token = cookie_jar
    .iter()
    .find(|(k, _)| k.eq_ignore_ascii_case("XSRF-TOKEN"))
    .and_then(|(_, v)| {
      log::info!("Found XSRF-TOKEN cookie (encoded): {}", v);
      decode(v).ok()
    })
    .map(|token| {
      let decoded = token.to_string();
      log::info!("Decoded XSRF-TOKEN: {}", &decoded[..decoded.len().min(50)]);
      decoded
    });

  if xsrf_token.is_none() {
    log::warn!("No XSRF-TOKEN found in cookie jar!");
  }

  if status.is_redirection() {
    let location = response
      .headers()
      .get("location")
      .and_then(|v| v.to_str().ok())
      .unwrap_or("");

    log::info!("Login redirect to: {}", location);

    // 检查重定向位置来判断是否需要绑定昵称
    let requires_bind = location.contains("/user/player/bind");

    log::info!(
      "Detected requires_bind: {} (location contains '/user/player/bind': {})",
      requires_bind,
      location.contains("/user/player/bind")
    );

    // 如果不需要绑定,访问 /user 页面获取玩家名
    let player_name = if !requires_bind {
      match get_player_name(&client, &cookie_jar).await {
        Ok(name) => {
          log::info!("Retrieved player name: {}", name);
          Some(name)
        }
        Err(e) => {
          log::warn!("Failed to get player name: {:?}", e);
          None
        }
      }
    } else {
      None
    };

    // 关键修复:如果重定向到/user但没有玩家名,说明是新用户需要绑定
    let requires_bind = requires_bind || (location.contains("/user") && player_name.is_none());

    log::info!(
      "CAUC eduroam login successful, requires_bind: {}, player_name: {:?}",
      requires_bind,
      player_name
    );

    Ok(CAUCAuthState {
      student_id: student_id.clone(),
      cookie_jar,
      requires_bind,
      xsrf_token,
      client,
      player_name,
    })
  } else if status.is_success() {
    // 如果返回 200,可能是AJAX请求返回的JSON响应
    let response_text = response.text().await.unwrap_or_default();
    log::info!("Login returned 200, checking if it's a JSON response...");
    log::debug!(
      "Response body (first 500 chars): {}",
      &response_text[..500.min(response_text.len())]
    );

    // 尝试解析为JSON,查看是否包含重定向信息
    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&response_text) {
      log::info!("Parsed JSON response: {:?}", json_value);

      // 检查是否有redirect字段
      if let Some(redirect_url) = json_value.get("redirect").and_then(|v| v.as_str()) {
        log::info!("Found redirect URL in JSON: {}", redirect_url);
        let requires_bind = redirect_url.contains("/user/player/bind");

        log::info!(
          "CAUC eduroam login successful (JSON response), requires_bind: {}",
          requires_bind
        );

        return Ok(CAUCAuthState {
          student_id: student_id.clone(),
          cookie_jar,
          requires_bind,
          xsrf_token,
          client,
          player_name: None, // 新用户暂无玩家名
        });
      }
    }

    log::error!(
      "Login returned 200 but format is unexpected. Response: {}",
      &response_text[..200.min(response_text.len())]
    );
    Err(AccountError::Invalid.into())
  } else {
    log::error!("CAUC eduroam login failed: status {}", status);
    Err(AccountError::Invalid.into())
  }
}

pub async fn bind_player_name(
  _app: &AppHandle,
  auth_state: &CAUCAuthState,
  player_name: String,
) -> XMCLResult<()> {
  let client = auth_state.client.clone();

  let bind_url = format!("{}/user/player/bind", CAUC_BASE_URL);

  // 构建 Cookie 头
  let cookie_header = auth_state
    .cookie_jar
    .iter()
    .map(|(k, v)| format!("{}={}", k, v))
    .collect::<Vec<_>>()
    .join("; ");

  log::info!("Sending bind request for player_name: {}", player_name);
  log::info!("Bind URL: {}", bind_url);
  log::info!(
    "Cookie header (length: {}): {}",
    cookie_header.len(),
    &cookie_header[..cookie_header.len().min(200)]
  );
  log::info!("Cookie jar contents:");
  for (k, v) in &auth_state.cookie_jar {
    log::info!("  {} = {} (length: {})", k, &v[..v.len().min(50)], v.len());
  }

  let mut request = client
    .post(&bind_url)
    .header("Cookie", cookie_header)
    .header("Origin", CAUC_BASE_URL)
    .header("Referer", format!("{}/user/player/bind", CAUC_BASE_URL))
    .header("X-Requested-With", "XMLHttpRequest");

  if let Some(token) = &auth_state.xsrf_token {
    log::info!("Using XSRF-TOKEN: {}", token);
    request = request.header("X-XSRF-TOKEN", token);
  } else {
    log::warn!("No XSRF-TOKEN available for bind request");
  }

  let response = request
    .json(&json!({
        "player": player_name
    }))
    .send()
    .await
    .map_err(|e| {
      log::error!("CAUC bind player name network error: {}", e);
      AccountError::NetworkError
    })?;

  let status = response.status();
  let redirect_location = response
    .headers()
    .get("location")
    .and_then(|v| v.to_str().ok())
    .map(|s| s.to_string());
  let response_text = response.text().await.unwrap_or_default();

  log::info!("Bind response status: {}", status);
  log::info!("Bind response body: {}", response_text);

  if !(status.is_success() || status.is_redirection()) {
    log::error!(
      "CAUC bind player name failed: status {}, body: {}",
      status,
      response_text
    );
    return Err(AccountError::Invalid.into());
  }

  if status.is_redirection() {
    if let Some(location) = redirect_location {
      log::info!("Bind redirected to {}", location);
    }
  }

  log::info!("Successfully bound player name: {}", player_name);
  Ok(())
}

pub async fn authenticate(
  app: &AppHandle,
  auth_state: &CAUCAuthState,
  oa_password: String,
) -> XMCLResult<(Vec<PlayerInfo>, bool)> {
  let mut candidate_usernames: Vec<String> = vec![];

  if let Some(player_name) = auth_state.player_name.as_ref() {
    let trimmed = player_name.trim();
    if !trimmed.is_empty() {
      candidate_usernames.push(trimmed.to_string());
    }
  }

  if !candidate_usernames
    .iter()
    .any(|name| name.eq_ignore_ascii_case(&auth_state.student_id))
  {
    candidate_usernames.push(auth_state.student_id.clone());
  }

  if candidate_usernames.is_empty() {
    candidate_usernames.push(auth_state.student_id.clone());
  }

  log::info!(
    "Authenticating with Yggdrasil API, candidates: {:?}",
    candidate_usernames
  );

  let mut last_error = AccountError::Invalid;
  let mut auth_response: Option<CAUCAuthResponse> = None;
  let mut successful_username: Option<String> = None;

  for username in &candidate_usernames {
    match perform_auth_request(auth_state, username, &oa_password).await {
      Ok(response) => {
        log::info!("CAUC authenticate succeeded with username: {}", username);
        successful_username = Some(username.clone());
        auth_response = Some(response);
        break;
      }
      Err(AccountError::Invalid) => {
        log::warn!(
          "CAUC authenticate rejected credentials for username {}, trying fallback if available",
          username
        );
        last_error = AccountError::Invalid;
      }
      Err(err) => {
        return Err(err.into());
      }
    }
  }

  let content = match auth_response {
    Some(content) => content,
    None => {
      log::error!("CAUC authenticate failed for all candidate usernames");
      return Err(last_error.into());
    }
  };

  if let Some(username) = successful_username {
    log::info!("CAUC authenticate finalized with username: {}", username);
  }

  let access_token = content.access_token;

  log::info!(
    "CAUC auth response: selected_profile={:?}, available_profiles={:?}",
    content.selected_profile,
    content.available_profiles
  );

  if let Some(selected_profile) = content.selected_profile {
    let id = selected_profile.id.clone();
    log::info!("Fetching profile for selected player ID: {}", id);

    let profile = match retrieve_profile(app, CAUC_YGGDRASIL_URL.to_string(), id.clone()).await {
      Ok(p) => p,
      Err(e) => {
        log::error!("Failed to retrieve profile for ID {}: {:?}", id, e);
        return Err(e);
      }
    };

    log::info!("Retrieved profile: {:?}", profile.name);

    let player = match parse_profile(
      app,
      &profile,
      Some(access_token),
      None,
      Some(CAUC_YGGDRASIL_URL.to_string()),
      Some(auth_state.student_id.clone()),
    )
    .await
    {
      Ok(p) => p,
      Err(e) => {
        log::error!("Failed to parse profile: {:?}", e);
        return Err(e);
      }
    };

    log::info!("Parsed player: {} ({})", player.name, player.uuid);

    Ok((vec![player], true))
  } else {
    let available_profiles = content.available_profiles.unwrap_or_default();

    if available_profiles.is_empty() {
      return Err(AccountError::NotFound.into());
    }

    let mut players = vec![];

    for profile in available_profiles {
      let mc_profile = retrieve_profile(app, CAUC_YGGDRASIL_URL.to_string(), profile.id).await?;
      let player = parse_profile(
        app,
        &mc_profile,
        Some(access_token.clone()),
        None,
        Some(CAUC_YGGDRASIL_URL.to_string()),
        Some(auth_state.student_id.clone()),
      )
      .await?;

      players.push(player);
    }

    Ok((players, false))
  }
}

async fn perform_auth_request(
  auth_state: &CAUCAuthState,
  username: &str,
  oa_password: &str,
) -> Result<CAUCAuthResponse, AccountError> {
  let client = auth_state.client.clone();
  let auth_url = format!("{}/authserver/authenticate", CAUC_YGGDRASIL_URL);

  let cookie_header = auth_state
    .cookie_jar
    .iter()
    .map(|(k, v)| format!("{}={}", k, v))
    .collect::<Vec<_>>()
    .join("; ");

  log::info!(
    "Attempting CAUC authenticate request for username: {}",
    username
  );

  let mut request_builder = client
    .post(&auth_url)
    .header("Cookie", cookie_header)
    .header("Origin", CAUC_BASE_URL)
    .header("Referer", format!("{}/", CAUC_BASE_URL))
    .header("X-Requested-With", "XMLHttpRequest")
    .header("Content-Type", "application/json");

  if let Some(token) = &auth_state.xsrf_token {
    request_builder = request_builder.header("X-XSRF-TOKEN", token);
  }

  let response = request_builder
    .json(&json!({
        "username": username,
        "password": oa_password,
        "agent": {
            "name": "Minecraft",
            "version": 1
        },
    }))
    .send()
    .await
    .map_err(|e| {
      log::error!(
        "CAUC authenticate network error for username {}: {}",
        username,
        e
      );
      AccountError::NetworkError
    })?;

  let status = response.status();

  let response_text = response.text().await.map_err(|e| {
    log::error!(
      "Failed to read authenticate response for username {}: {}",
      username,
      e
    );
    AccountError::NetworkError
  })?;

  if !status.is_success() {
    let trimmed = response_text[..response_text.len().min(200)].to_string();
    log::warn!(
      "CAUC authenticate rejected username {}: status {}, body {}",
      username,
      status,
      trimmed
    );

    if status.is_client_error() {
      return Err(AccountError::Invalid);
    }

    return Err(AccountError::NetworkError);
  }

  log::info!(
    "CAUC authenticate response for username {}: {}",
    username,
    &response_text[..response_text.len().min(200)]
  );

  let content: CAUCAuthResponse = serde_json::from_str(&response_text).map_err(|e| {
    log::error!(
      "CAUC auth response parse error for username {}: {}, body {}",
      username,
      e,
      &response_text[..response_text.len().min(200)]
    );
    AccountError::ParseError
  })?;

  Ok(content)
}

pub async fn login_flow(
  app: &AppHandle,
  student_id: String,
  oa_password: String,
  player_name: Option<String>,
) -> XMCLResult<LoginFlowResult> {
  // 步骤 1: eduroam 登录
  let auth_state = eduroam_login(app, student_id.clone(), oa_password.clone()).await?;

  // 步骤 2: 检查是否需要绑定
  if auth_state.requires_bind {
    if let Some(name) = player_name {
      // 如果提供了昵称,直接绑定
      bind_player_name(app, &auth_state, name).await?;

      // 绑定后重新认证
      let (players, has_selected) = authenticate(app, &auth_state, oa_password.clone()).await?;
      Ok(LoginFlowResult::Success {
        players,
        has_selected,
      })
    } else {
      // 需要用户输入昵称
      Ok(LoginFlowResult::RequiresBind { auth_state })
    }
  } else {
    // 步骤 3: 直接认证
    let (players, has_selected) = authenticate(app, &auth_state, oa_password).await?;
    Ok(LoginFlowResult::Success {
      players,
      has_selected,
    })
  }
}

#[derive(Debug, Clone)]
pub enum LoginFlowResult {
  Success {
    players: Vec<PlayerInfo>,
    has_selected: bool,
  },
  RequiresBind {
    auth_state: CAUCAuthState,
  },
}

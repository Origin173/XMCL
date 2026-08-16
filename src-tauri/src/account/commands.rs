use crate::account::constants::TEXTURE_ROLES;
use crate::account::helpers::authlib_injector::info::{
  fetch_auth_server_info, fetch_auth_url, get_auth_server_info_by_url,
  refresh_and_update_auth_servers,
};
use crate::account::helpers::authlib_injector::jar::check_authlib_jar;
use crate::account::helpers::authlib_injector::{self};
use crate::account::helpers::{microsoft, misc, offline};
use crate::account::models::{
  AccountError, AccountInfo, AuthServer, DeviceAuthResponseInfo, Player, PlayerInfo, PlayerType,
};
use crate::error::XMCLResult;
use crate::launcher_config::models::LauncherConfig;
use crate::storage::Storage;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};
use url::Url;

#[tauri::command]
pub fn retrieve_player_list(app: AppHandle) -> XMCLResult<Vec<Player>> {
  let binding = app.state::<Mutex<AccountInfo>>();
  let state = binding.lock()?;

  let player_list: Vec<Player> = state
    .clone()
    .players
    .into_iter()
    .map(Player::from)
    .collect();
  Ok(player_list)
}

#[tauri::command]
pub async fn add_player_offline(app: AppHandle, username: String, uuid: String) -> XMCLResult<()> {
  let new_player = offline::login(&app, username, uuid).await?;

  let account_binding = app.state::<Mutex<AccountInfo>>();
  let mut account_state = account_binding.lock()?;

  let config_binding = app.state::<Mutex<LauncherConfig>>();
  let mut config_state = config_binding.lock()?;

  if account_state
    .players
    .iter()
    .any(|player| player.id == new_player.id)
  {
    return Err(AccountError::Duplicate.into());
  }

  config_state.partial_update(
    &app,
    "states.shared.selected_player_id",
    &serde_json::to_string(&new_player.id).unwrap_or_default(),
  )?;
  config_state.save()?;

  account_state.players.push(new_player);
  account_state.save()?;
  Ok(())
}

#[tauri::command]
pub async fn fetch_oauth_code(
  app: AppHandle,
  server_type: PlayerType,
  auth_server_url: String,
) -> XMCLResult<DeviceAuthResponseInfo> {
  if server_type == PlayerType::ThirdParty {
    let auth_server = AuthServer::from(get_auth_server_info_by_url(&app, auth_server_url)?);

    authlib_injector::oauth::device_authorization(
      &app,
      auth_server.features.openid_configuration_url,
      auth_server.client_id,
    )
    .await
  } else if server_type == PlayerType::Microsoft {
    microsoft::oauth::device_authorization(&app).await
  } else {
    Err(AccountError::Invalid.into())
  }
}

#[tauri::command]
pub async fn add_player_oauth(
  app: AppHandle,
  server_type: PlayerType,
  auth_info: DeviceAuthResponseInfo,
  auth_server_url: String,
) -> XMCLResult<()> {
  let new_player = match server_type {
    PlayerType::ThirdParty => {
      let _ = check_authlib_jar(&app).await; // ignore the error when logging in

      let auth_server =
        AuthServer::from(get_auth_server_info_by_url(&app, auth_server_url.clone())?);

      authlib_injector::oauth::login(
        &app,
        auth_server_url,
        auth_server.features.openid_configuration_url,
        auth_server.client_id,
        auth_info,
      )
      .await?
    }

    PlayerType::Microsoft => microsoft::oauth::login(&app, auth_info).await?,

    PlayerType::Offline => {
      return Err(AccountError::Invalid.into());
    }
  };

  {
    let account_binding = app.state::<Mutex<AccountInfo>>();
    let mut account_state = account_binding.lock()?;

    let config_binding = app.state::<Mutex<LauncherConfig>>();
    let mut config_state = config_binding.lock()?;

    if account_state
      .players
      .iter()
      .any(|player| player.id == new_player.id)
    {
      return Err(AccountError::Duplicate.into());
    }

    config_state.partial_update(
      &app,
      "states.shared.selected_player_id",
      &serde_json::to_string(&new_player.id).unwrap_or_default(),
    )?;
    config_state.save()?;

    account_state.players.push(new_player);
    account_state.save()?;
  }

  misc::check_full_login_availability(&app).await
}

#[tauri::command]
pub async fn relogin_player_oauth(
  app: AppHandle,
  player_id: String,
  auth_info: DeviceAuthResponseInfo,
) -> XMCLResult<()> {
  let account_binding = app.state::<Mutex<AccountInfo>>();

  let cloned_account_state = account_binding.lock()?.clone();

  let old_player = cloned_account_state
    .players
    .iter()
    .find(|player| player.id == player_id)
    .ok_or(AccountError::NotFound)?;

  let new_player = match old_player.player_type {
    PlayerType::ThirdParty => {
      let auth_server = AuthServer::from(get_auth_server_info_by_url(
        &app,
        old_player.auth_server_url.clone().unwrap_or_default(),
      )?);

      authlib_injector::oauth::login(
        &app,
        old_player.auth_server_url.clone().unwrap_or_default(),
        auth_server.features.openid_configuration_url,
        auth_server.client_id,
        auth_info,
      )
      .await?
    }

    PlayerType::Microsoft => microsoft::oauth::login(&app, auth_info).await?,

    PlayerType::Offline => {
      return Err(AccountError::Invalid.into());
    }
  };

  {
    let mut account_state = account_binding.lock()?;

    if let Some(player) = account_state
      .players
      .iter_mut()
      .find(|player| player.id == player_id)
    {
      *player = new_player;
      account_state.save()?;
    }
  }

  misc::check_full_login_availability(&app).await
}

#[tauri::command]
pub fn cancel_oauth(app: AppHandle) -> XMCLResult<()> {
  let account_binding = app.state::<Mutex<AccountInfo>>();
  let mut account_state = account_binding.lock()?;
  account_state.is_oauth_processing = false;

  Ok(())
}

#[tauri::command]
pub async fn add_player_3rdparty_password(
  app: AppHandle,
  auth_server_url: String,
  username: String,
  password: String,
) -> XMCLResult<Vec<Player>> {
  let _ = check_authlib_jar(&app).await; // ignore the error when logging in

  let (mut new_players, is_token_binded) =
    authlib_injector::password::login(&app, auth_server_url, username, password).await?;

  if new_players.is_empty() {
    return Err(AccountError::NotFound.into());
  }

  {
    let account_binding = app.state::<Mutex<AccountInfo>>();
    let account_state = account_binding.lock()?;
    new_players.retain_mut(|new_player| {
      account_state
        .players
        .iter()
        .all(|player| new_player.id != player.id)
    });
  }

  if new_players.is_empty() {
    Err(AccountError::Duplicate.into())
  } else if new_players.len() == 1 {
    // if only one player will be added, save it and return **an empty vector** to inform the frontend not to trigger selector.
    if !is_token_binded {
      // if the token is not binded, refresh it to bind the token.
      new_players[0] = authlib_injector::password::refresh(&app, &new_players[0], true).await?;
    }
    {
      let account_binding = app.state::<Mutex<AccountInfo>>();
      let mut account_state = account_binding.lock()?;
      let config_binding = app.state::<Mutex<LauncherConfig>>();
      let mut config_state = config_binding.lock()?;

      config_state.partial_update(
        &app,
        "states.shared.selected_player_id",
        &serde_json::to_string(&new_players[0].id).unwrap_or_default(),
      )?;
      account_state.players.push(new_players[0].clone());

      account_state.save()?;
      config_state.save()?;
    }

    Ok(vec![])
  } else {
    // if more than one player will be added, return the players to inform the frontend to trigger selector.
    let players = new_players
      .iter()
      .map(|player| Player::from(player.clone()))
      .collect::<Vec<Player>>();

    Ok(players)
  }
}

#[tauri::command]
pub async fn relogin_player_3rdparty_password(
  app: AppHandle,
  player_id: String,
  password: String,
) -> XMCLResult<()> {
  let account_binding = app.state::<Mutex<AccountInfo>>();

  let cloned_account_state = account_binding.lock()?.clone();

  let old_player = cloned_account_state
    .players
    .iter()
    .find(|player| player.id == player_id)
    .ok_or(AccountError::NotFound)?;

  if old_player.player_type != PlayerType::ThirdParty {
    return Err(AccountError::Invalid.into());
  }

  let (player_list, is_token_binded) = authlib_injector::password::login(
    &app,
    old_player.auth_server_url.clone().unwrap_or_default(),
    old_player.auth_account.clone().unwrap_or_default(),
    password,
  )
  .await?;

  let mut new_player = player_list
    .into_iter()
    .find(|player| player.uuid == old_player.uuid)
    .ok_or(AccountError::NotFound)?;

  if !is_token_binded {
    new_player = authlib_injector::password::refresh(&app, &new_player, true).await?;
  }

  {
    let mut account_state = account_binding.lock()?;

    if let Some(player) = account_state
      .players
      .iter_mut()
      .find(|player| player.id == player_id)
    {
      *player = new_player;
      account_state.save()?;
    }
  }

  misc::check_full_login_availability(&app).await
}

#[tauri::command]
pub async fn add_player_from_selection(app: AppHandle, player: Player) -> XMCLResult<()> {
  let player_info: PlayerInfo = player.into();
  let refreshed_player = authlib_injector::password::refresh(&app, &player_info, true).await?;

  {
    let account_binding = app.state::<Mutex<AccountInfo>>();
    let mut account_state = account_binding.lock()?;

    let config_binding = app.state::<Mutex<LauncherConfig>>();
    let mut config_state = config_binding.lock()?;

    if account_state
      .players
      .iter()
      .any(|x| x.id == refreshed_player.id)
    {
      return Err(AccountError::Duplicate.into());
    }

    config_state.partial_update(
      &app,
      "states.shared.selected_player_id",
      &serde_json::to_string(&refreshed_player.id).unwrap_or_default(),
    )?;
    account_state.players.push(refreshed_player);

    account_state.save()?;
    config_state.save()?;
  }

  misc::check_full_login_availability(&app).await
}

#[tauri::command]
pub fn update_player_skin_offline_preset(
  app: AppHandle,
  player_id: String,
  preset_role: String,
) -> XMCLResult<()> {
  let account_binding = app.state::<Mutex<AccountInfo>>();
  let mut account_state = account_binding.lock()?;

  let player = account_state
    .get_player_by_id_mut(player_id.clone())
    .ok_or(AccountError::NotFound)?;

  if player.player_type != PlayerType::Offline {
    return Err(AccountError::Invalid.into());
  }

  if TEXTURE_ROLES.contains(&preset_role.as_str()) {
    player.textures = offline::load_preset_skin(&app, preset_role)?;
  } else {
    return Err(AccountError::TextureError.into());
  }

  account_state.save()?;
  Ok(())
}

#[tauri::command]
pub async fn delete_player(app: AppHandle, player_id: String) -> XMCLResult<()> {
  {
    let account_binding = app.state::<Mutex<AccountInfo>>();
    let mut account_state = account_binding.lock()?;

    let config_binding = app.state::<Mutex<LauncherConfig>>();
    let mut config_state = config_binding.lock()?;

    let initial_len = account_state.players.len();
    account_state.players.retain(|s| s.id != player_id);
    if account_state.players.len() == initial_len {
      return Err(AccountError::NotFound.into());
    }

    if config_state.states.shared.selected_player_id == player_id {
      config_state.partial_update(
        &app,
        "states.shared.selected_player_id",
        &serde_json::to_string(
          &account_state
            .players
            .first()
            .map_or("".to_string(), |player| player.id.clone()),
        )
        .unwrap_or_default(),
      )?;
      config_state.save()?;
    }

    account_state.save()?;
  }

  misc::check_full_login_availability(&app).await
}

#[tauri::command]
pub async fn refresh_player(app: AppHandle, player_id: String) -> XMCLResult<()> {
  let account_binding = app.state::<Mutex<AccountInfo>>();

  let cloned_account_state = account_binding.lock()?.clone();

  let player = cloned_account_state
    .players
    .iter()
    .find(|player| player.id == player_id)
    .ok_or(AccountError::NotFound)?;

  let refreshed_player = match player.player_type {
    PlayerType::ThirdParty => {
      let auth_server = AuthServer::from(get_auth_server_info_by_url(
        &app,
        player.auth_server_url.clone().unwrap_or_default(),
      )?);

      authlib_injector::common::refresh(&app, player, &auth_server).await?
    }

    PlayerType::Microsoft => microsoft::oauth::refresh(&app, player).await?,

    PlayerType::Offline => {
      return Err(AccountError::Invalid.into());
    }
  };

  let mut account_state = account_binding.lock()?;

  if let Some(player) = account_state
    .players
    .iter_mut()
    .find(|player| player.id == player_id)
  {
    *player = refreshed_player;
    account_state.save()?;
  }

  Ok(())
}

#[tauri::command]
pub async fn retrieve_auth_server_list(app: AppHandle) -> XMCLResult<Vec<AuthServer>> {
  // Check if auth servers need to be refreshed (if metadata is null)
  let needs_refresh = {
    let binding = app.state::<Mutex<AccountInfo>>();
    let state = binding.lock()?;
    state.auth_servers.iter().any(|s| s.metadata.is_null())
  };

  // If any server has null metadata, refresh them first
  if needs_refresh {
    log::info!("Auth servers metadata is null, refreshing...");
    refresh_and_update_auth_servers(&app).await?;
    log::info!("Auth servers metadata refreshed successfully");
  }

  let binding = app.state::<Mutex<AccountInfo>>();
  let state = binding.lock()?;
  let auth_servers: Vec<AuthServer> = state
    .auth_servers
    .iter()
    .map(|server| {
      let auth_server = AuthServer::from(server.clone());
      log::debug!(
        "Auth server: {} ({})",
        auth_server.name,
        auth_server.auth_url
      );
      auth_server
    })
    .collect();
  Ok(auth_servers)
}

#[tauri::command]
pub async fn fetch_auth_server(app: AppHandle, url: String) -> XMCLResult<AuthServer> {
  // check the url integrity following the standard
  // https://github.com/yushijinhun/authlib-injector/wiki/%E5%90%AF%E5%8A%A8%E5%99%A8%E6%8A%80%E6%9C%AF%E8%A7%84%E8%8C%83#%E5%9C%A8%E5%90%AF%E5%8A%A8%E5%99%A8%E4%B8%AD%E8%BE%93%E5%85%A5%E5%9C%B0%E5%9D%80
  let parsed_url = Url::parse(&url)
    .or(Url::parse(&format!("https://{}", url)))
    .map_err(|_| AccountError::Invalid)?;

  let auth_url = fetch_auth_url(&app, parsed_url).await?;

  if get_auth_server_info_by_url(&app, auth_url.clone()).is_ok() {
    return Err(AccountError::Duplicate.into());
  }

  Ok(AuthServer::from(
    fetch_auth_server_info(&app, auth_url).await?,
  ))
}

#[tauri::command]
pub async fn add_auth_server(app: AppHandle, auth_url: String) -> XMCLResult<()> {
  if get_auth_server_info_by_url(&app, auth_url.clone()).is_ok() {
    return Err(AccountError::Duplicate.into());
  }

  let server = fetch_auth_server_info(&app, auth_url).await?;

  let binding = app.state::<Mutex<AccountInfo>>();
  let mut state = binding.lock()?;
  state.auth_servers.push(server);
  state.save()?;
  Ok(())
}

#[tauri::command]
pub fn delete_auth_server(app: AppHandle, url: String) -> XMCLResult<()> {
  let account_binding = app.state::<Mutex<AccountInfo>>();
  let mut account_state = account_binding.lock()?;

  let config_binding = app.state::<Mutex<LauncherConfig>>();
  let mut config_state = config_binding.lock()?;

  let initial_len = account_state.auth_servers.len();

  // try to remove the server from the storage
  account_state
    .auth_servers
    .retain(|server| server.auth_url != url);
  if account_state.auth_servers.len() == initial_len {
    return Err(AccountError::NotFound.into());
  }

  // remove all players using this server & check if the selected player needs reset
  let mut need_reset = false;
  let selected_id = config_state.states.shared.selected_player_id.clone();

  account_state.players.retain(|player| {
    let should_remove = player.auth_server_url == Some(url.clone());
    if should_remove && player.id == selected_id {
      need_reset = true;
    }
    !should_remove
  });

  if need_reset {
    config_state.partial_update(
      &app,
      "states.shared.selected_player_id",
      &serde_json::to_string(
        &(if let Some(first_player) = account_state.players.first() {
          first_player.id.clone()
        } else {
          "".to_string()
        }),
      )
      .unwrap_or_default(),
    )?;
  }

  account_state.save()?;
  config_state.save()?;
  Ok(())
}

// ==================== CAUC 登录命令 ====================

use crate::account::helpers::authlib_injector::cauc;

/// CAUC 步骤 1: 表单登录
///
/// 返回是否需要绑定昵称
#[tauri::command]
pub async fn cauc_login(
  app: AppHandle,
  student_id: String,
  oa_password: String,
) -> XMCLResult<bool> {
  let auth_state = cauc::cauc_login(&app, student_id, oa_password).await?;

  // 将 auth_state 临时存储到应用状态中
  // 这样后续的绑定操作可以使用
  app
    .state::<std::sync::Mutex<Option<cauc::CAUCAuthState>>>()
    .lock()
    .map_err(|_| AccountError::Invalid)?
    .replace(auth_state.clone());

  Ok(auth_state.requires_bind)
}

/// CAUC 步骤 2: 绑定游戏昵称
#[tauri::command]
pub async fn cauc_bind_player_name(app: AppHandle, player_name: String) -> XMCLResult<()> {
  // 从应用状态中获取之前保存的 auth_state
  let auth_state_mutex = app.state::<std::sync::Mutex<Option<cauc::CAUCAuthState>>>();
  let auth_state = {
    let guard = auth_state_mutex.lock().map_err(|_| AccountError::Invalid)?;
    guard.clone().ok_or(AccountError::Invalid)?
  };

  // 执行绑定
  cauc::bind_player_name(&app, &auth_state, player_name.clone()).await?;

  // 更新 auth_state 中的 player_name
  let mut guard = auth_state_mutex.lock().map_err(|_| AccountError::Invalid)?;
  if let Some(state) = guard.as_mut() {
    state.player_name = Some(player_name);
    log::info!("Updated auth_state with player_name after binding");
  }

  Ok(())
}

/// CAUC 步骤 3: 完成登录并添加账号
#[tauri::command]
pub async fn cauc_complete_login(
  app: AppHandle,
  _student_id: String,
  oa_password: String,
) -> XMCLResult<Vec<Player>> {
  log::info!("CAUC complete_login called");

  // 从应用状态中获取之前保存的 auth_state
  let auth_state_option = app
    .state::<std::sync::Mutex<Option<cauc::CAUCAuthState>>>()
    .lock()
    .map_err(|_| AccountError::Invalid)?
    .clone();

  log::info!("Auth state retrieved: {:?}", auth_state_option.is_some());

  let auth_state = auth_state_option.ok_or_else(|| {
    log::error!("Auth state is None - user must call cauc_login first!");
    AccountError::Invalid
  })?;

  // 使用 OA 密码进行 Yggdrasil 认证
  let (mut new_players, has_selected_player) =
    cauc::authenticate(&app, &auth_state, oa_password).await?;

  log::info!("Authentication returned {} players", new_players.len());
  if !new_players.is_empty() {
    log::info!(
      "First player: {} ({})",
      new_players[0].name,
      new_players[0].id
    );
  }

  if new_players.is_empty() {
    log::error!("No players returned from authenticate");
    return Err(AccountError::NotFound.into());
  }

  let original_count = new_players.len();

  {
    let account_binding = app.state::<Mutex<AccountInfo>>();
    let account_state = account_binding.lock()?;
    log::info!(
      "Current account list has {} players",
      account_state.players.len()
    );
    for existing_player in &account_state.players {
      log::info!(
        "  Existing: {} ({})",
        existing_player.name,
        existing_player.id
      );
    }
    new_players.retain_mut(|new_player| {
      let is_new = account_state
        .players
        .iter()
        .all(|player| new_player.id != player.id);
      if !is_new {
        log::info!(
          "Player {} ({}) already exists, filtering out",
          new_player.name,
          new_player.id
        );
      }
      is_new
    });
  }

  log::info!(
    "After deduplication: {} players (was {})",
    new_players.len(),
    original_count
  );

  if new_players.is_empty() {
    log::warn!("All players were duplicates");
    // 清除临时状态
    app
      .state::<std::sync::Mutex<Option<cauc::CAUCAuthState>>>()
      .lock()
      .map_err(|_| AccountError::Invalid)?
      .take();
    return Err(AccountError::Duplicate.into());
  }

  if new_players.len() == 1 || has_selected_player {
    // 只有一个玩家或服务器已选定,直接添加
    let new_player = new_players[0].clone();
    log::info!(
      "Adding single player: {} ({})",
      new_player.name,
      new_player.id
    );

    {
      let account_binding = app.state::<Mutex<AccountInfo>>();
      let mut account_state = account_binding.lock()?;
      let config_binding = app.state::<Mutex<LauncherConfig>>();
      let mut config_state = config_binding.lock()?;

      log::info!("Updating config with current account ID: {}", new_player.id);
      config_state.partial_update(
        &app,
        "states.shared.selected_player_id",
        &serde_json::to_string(&new_player.id).unwrap_or_default(),
      )?;

      log::info!("Pushing player to account state");
      account_state.players.push(new_player.clone());

      log::info!("Saving account state");
      account_state.save().map_err(|e| {
        log::error!("Failed to save account state: {:?}", e);
        e
      })?;

      log::info!("Saving config state");
      config_state.save().map_err(|e| {
        log::error!("Failed to save config state: {:?}", e);
        e
      })?;

      log::info!("Successfully saved player: {}", new_player.name);
    }

    // 清除临时状态
    log::info!("Clearing temporary auth state");
    app
      .state::<std::sync::Mutex<Option<cauc::CAUCAuthState>>>()
      .lock()
      .map_err(|_| AccountError::Invalid)?
      .take();

    log::info!("CAUC login completed successfully");
    Ok(vec![])
  } else {
    // 多个玩家,返回列表让前端选择
    let players = new_players
      .iter()
      .map(|player| Player::from(player.clone()))
      .collect();
    Ok(players)
  }
}

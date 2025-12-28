use crate::account::constants::TEXTURE_ROLES;
use crate::account::models::{AccountError, PlayerInfo, PlayerType, Texture};
use crate::error::XMCLResult;
use crate::utils::image::decode_image;
use rand::seq::IndexedRandom;
use tauri::AppHandle;
use uuid::Uuid;

pub fn load_preset_skin(_app: &AppHandle, preset_role: String) -> XMCLResult<Vec<Texture>> {
  let texture_bytes = match preset_role.as_str() {
    "alex" => include_bytes!("../../../assets/skins/alex.png").to_vec(),
    _ => include_bytes!("../../../assets/skins/steve.png").to_vec(),
  };

  let texture_img = decode_image(texture_bytes).map_err(|_| AccountError::TextureError)?;

  Ok(vec![Texture {
    texture_type: "SKIN".to_string(),
    image: texture_img.into(),
    model: "default".to_string(),
    preset: Some(preset_role),
  }])
}

pub async fn login(app: &AppHandle, username: String, raw_uuid: String) -> XMCLResult<PlayerInfo> {
  let name_with_prefix = format!("OfflinePlayer:{}", username);
  let uuid = if let Ok(id) = Uuid::parse_str(&raw_uuid) {
    id
  } else {
    if !raw_uuid.is_empty() {
      // user uses custom UUID, but it's invalid
      return Err(AccountError::Invalid)?;
    }
    Uuid::new_v5(&Uuid::NAMESPACE_URL, name_with_prefix.as_bytes())
  };
  let texture_role = TEXTURE_ROLES.choose(&mut rand::rng()).unwrap_or(&"steve");

  Ok(
    PlayerInfo {
      id: "".to_string(),
      name: username,
      uuid,
      player_type: PlayerType::Offline,
      auth_account: None,
      auth_server_url: None,
      access_token: None,
      refresh_token: None,
      textures: load_preset_skin(app, texture_role.to_string())?,
    }
    .with_generated_id(),
  )
}

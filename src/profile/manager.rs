use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::Profile;

pub const PROFILE_MAIN_URL: &str = "https://chat.qwen.ai";

fn profiles_root() -> PathBuf {
    let dir = crate::config::paths::config_dir().join("profiles");
    let _ = fs::create_dir_all(&dir);
    dir
}

fn profiles_file() -> PathBuf {
    profiles_root().join("profiles.json")
}

pub fn data_dir_for(profile_id: &str) -> PathBuf {
    let dir = profiles_root().join(profile_id);
    let _ = fs::create_dir_all(&dir);
    dir
}

fn session_file(profile_id: &str) -> PathBuf {
    profiles_root().join(format!("{}.session.json", profile_id))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieData {
    pub name: String,
    pub value: String,
    pub domain: Option<String>,
    pub path: Option<String>,
    pub expires: Option<i64>,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: i32,
    pub max_age: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Session {
    #[serde(default)]
    pub cookies: Vec<CookieData>,
    #[serde(default)]
    pub local_storage: HashMap<String, String>,
}

pub fn save_session(profile_id: &str, session: &Session) -> Result<(), String> {
    let path = session_file(profile_id);
    let content = serde_json::to_string(session).map_err(|e| e.to_string())?;
    // Dirty-check: só grava se conteúdo mudou, reduz I/O em idle
    if let Ok(existing) = fs::read_to_string(&path) {
        if existing == content {
            return Ok(());
        }
    }
    let tmp = path.with_extension(format!("session.json.tmp.{}", std::process::id()));
    fs::write(&tmp, &content).map_err(|e| e.to_string())?;
    fs::rename(&tmp, &path).map_err(|e| e.to_string())
}

pub fn load_session(profile_id: &str) -> Option<Session> {
    let path = session_file(profile_id);
    fs::read_to_string(&path)
        .ok()
        .and_then(|c| serde_json::from_str::<Session>(&c).ok())
}

pub fn load() -> Vec<Profile> {
    let path = profiles_file();
    fs::read_to_string(&path)
        .ok()
        .and_then(|c| serde_json::from_str::<Vec<Profile>>(&c).ok())
        .unwrap_or_default()
}

fn save(profiles: &[Profile]) -> Result<(), String> {
    let path = profiles_file();
    let content = serde_json::to_string(profiles).map_err(|e| e.to_string())?;
    if let Ok(existing) = fs::read_to_string(&path) {
        if existing == content {
            return Ok(());
        }
    }
    let tmp = path.with_extension(format!("json.tmp.{}", std::process::id()));
    fs::write(&tmp, &content).map_err(|e| e.to_string())?;
    fs::rename(&tmp, &path).map_err(|e| e.to_string())
}

pub fn create(name: &str, category: Option<&str>, icon: Option<&str>) -> Result<Profile, String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("Profile name cannot be empty".into());
    }
    let mut profiles = load();
    if profiles.iter().any(|p| p.name.eq_ignore_ascii_case(&name)) {
        return Err("A profile with this name already exists".into());
    }
    let id = make_unique_id(&name, &profiles);
    let profile = Profile {
        id,
        name,
        category: category
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        icon: icon.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
    };
    profiles.push(profile.clone());
    save(&profiles)?;
    Ok(profile)
}

pub fn rename(id: &str, name: &str) -> Result<(), String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("Profile name cannot be empty".into());
    }
    let mut profiles = load();
    if profiles
        .iter()
        .any(|p| p.id != id && p.name.eq_ignore_ascii_case(&name))
    {
        return Err("A profile with this name already exists".into());
    }
    let entry = profiles
        .iter_mut()
        .find(|p| p.id == id)
        .ok_or("Profile not found")?;
    entry.name = name;
    save(&profiles)
}

pub fn delete(id: &str) -> Result<(), String> {
    let mut profiles = load();
    if !profiles.iter().any(|p| p.id == id) {
        return Err("Profile not found".into());
    }
    profiles.retain(|p| p.id != id);
    save(&profiles)?;
    let data = data_dir_for(id);
    let _ = fs::remove_dir_all(&data);
    let session = session_file(id);
    let _ = fs::remove_file(&session);
    Ok(())
}

pub fn list_categories() -> Vec<String> {
    let profiles = load();
    let mut categories: Vec<String> = profiles
        .iter()
        .filter_map(|p| p.category.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    categories.sort();
    categories
}

pub fn update_profile(
    id: &str,
    name: Option<&str>,
    category: Option<&str>,
    icon: Option<&str>,
) -> Result<Profile, String> {
    let mut profiles = load();

    if let Some(n) = name {
        let n = n.trim().to_string();
        if n.is_empty() {
            return Err("Profile name cannot be empty".into());
        }
        if profiles
            .iter()
            .any(|p| p.id != id && p.name.eq_ignore_ascii_case(&n))
        {
            return Err("A profile with this name already exists".into());
        }
    }

    let entry = profiles
        .iter_mut()
        .find(|p| p.id == id)
        .ok_or("Profile not found")?;

    if let Some(n) = name {
        entry.name = n.trim().to_string();
    }
    if category.is_some() {
        entry.category = category
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
    }
    if icon.is_some() {
        entry.icon = icon.map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    }

    let result = entry.clone();
    save(&profiles)?;
    Ok(result)
}

pub fn reorder(ordered_ids: &[String]) -> Result<(), String> {
    let mut profiles = load();

    for id in ordered_ids {
        if !profiles.iter().any(|p| p.id == *id) {
            return Err(format!("Profile not found: {}", id));
        }
    }

    if ordered_ids.len() != profiles.len() {
        return Err("Invalid reorder: length mismatch".into());
    }

    profiles.sort_by_key(|p| ordered_ids.iter().position(|id| id == &p.id));

    save(&profiles)
}

fn make_unique_id(name: &str, existing: &[Profile]) -> String {
    let base: String = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let base = if base.is_empty() { "profile" } else { &base };

    if !existing.iter().any(|p| p.id == base) {
        return base.to_string();
    }
    let mut n = 2;
    loop {
        let candidate = format!("{}-{}", base, n);
        if !existing.iter().any(|p| p.id == candidate) {
            return candidate;
        }
        n += 1;
    }
}

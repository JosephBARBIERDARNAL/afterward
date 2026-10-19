use super::*;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Resource, Serialize, Deserialize)]
pub(super) struct AudioSettings {
    pub music_volume_percent: u8,
    pub sfx_volume_percent: u8,
    #[serde(skip)]
    pub selected_slider: u8,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            music_volume_percent: 100,
            sfx_volume_percent: 100,
            selected_slider: 0,
        }
    }
}

pub(super) fn initialize_settings(data: Res<GameData>, mut settings: ResMut<AudioSettings>) {
    settings.music_volume_percent = data.gameplay.window.music_volume_percent;
    settings.sfx_volume_percent = data.gameplay.window.sfx_volume_percent;
    let Some(path) = preferences_path("settings.yaml") else {
        return;
    };
    let Ok(serialized) = fs::read_to_string(path) else {
        return;
    };
    let Ok(saved) = yaml_serde::from_str::<AudioSettings>(&serialized) else {
        return;
    };
    settings.music_volume_percent = saved.music_volume_percent.min(100);
    settings.sfx_volume_percent = saved.sfx_volume_percent.min(100);
}

pub(super) fn save_settings(settings: &AudioSettings) {
    let Some(path) = preferences_path("settings.yaml") else {
        return;
    };
    let Some(parent) = path.parent() else {
        return;
    };
    let Ok(serialized) = yaml_serde::to_string(settings) else {
        return;
    };
    if fs::create_dir_all(parent).is_ok() {
        let _ = fs::write(path, serialized);
    }
}

pub(super) fn preferences_path(file_name: &str) -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    let directory = PathBuf::from(std::env::var_os("APPDATA")?).join("Afterward");

    #[cfg(target_os = "macos")]
    let directory = PathBuf::from(std::env::var_os("HOME")?)
        .join("Library")
        .join("Application Support")
        .join("Afterward");

    #[cfg(target_os = "linux")]
    let directory = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?
        .join("afterward");

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    let directory = PathBuf::from(std::env::var_os("HOME")?).join(".afterward");

    Some(directory.join(file_name))
}

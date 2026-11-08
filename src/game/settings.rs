use super::*;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

/// How far one press of an arrow key moves a volume.
const VOLUME_STEP_PERCENT: u8 = 5;

#[derive(Resource, Serialize, Deserialize)]
pub(super) struct AudioSettings {
    pub music_volume_percent: u8,
    pub sfx_volume_percent: u8,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            music_volume_percent: 100,
            sfx_volume_percent: 100,
        }
    }
}

impl AudioSettings {
    pub(super) fn volume_of(&self, channel: VolumeChannel) -> u8 {
        match channel {
            VolumeChannel::Music => self.music_volume_percent,
            VolumeChannel::Sound => self.sfx_volume_percent,
        }
    }

    fn volume_mut(&mut self, channel: VolumeChannel) -> &mut u8 {
        match channel {
            VolumeChannel::Music => &mut self.music_volume_percent,
            VolumeChannel::Sound => &mut self.sfx_volume_percent,
        }
    }

    /// Moves a volume one step, the way an arrow key does.
    pub(super) fn step_volume(&mut self, channel: VolumeChannel, up: bool) {
        let volume = self.volume_mut(channel);
        *volume = if up {
            volume.saturating_add(VOLUME_STEP_PERCENT).min(100)
        } else {
            volume.saturating_sub(VOLUME_STEP_PERCENT)
        };
    }

    /// Sets a volume to how far along its bar a click landed, 0 to 1.
    pub(super) fn set_volume(&mut self, channel: VolumeChannel, along_bar: f32) {
        *self.volume_mut(channel) = (along_bar.clamp(0.0, 1.0) * 100.0).round() as u8;
    }
}

/// The menu the settings screen goes back to, and so whether it and the
/// controls screen stand over the main menu or over a paused run.
#[derive(Resource, Clone, Copy, Default)]
pub(super) struct SettingsReturn(pub(super) GameScreen);

impl SettingsReturn {
    /// Whether `screen` is drawn over the main menu: the menu itself, or the
    /// settings and controls screens when the main menu opened them.
    pub(super) fn on_the_main_menu(self, screen: GameScreen) -> bool {
        match screen {
            GameScreen::MainMenu => true,
            GameScreen::Settings | GameScreen::Controls => self.0 == GameScreen::MainMenu,
            _ => false,
        }
    }
}

pub(super) fn initialize_settings(
    data: Res<GameData>,
    preferences: Res<PreferencesDir>,
    mut settings: ResMut<AudioSettings>,
) {
    settings.music_volume_percent = data.gameplay.window.music_volume_percent;
    settings.sfx_volume_percent = data.gameplay.window.sfx_volume_percent;
    let Some(path) = preferences.file("settings.yaml") else {
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

pub(super) fn save_settings(settings: &AudioSettings, preferences: &PreferencesDir) {
    let Some(path) = preferences.file("settings.yaml") else {
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

/// The folder the player's settings and key bindings persist in. `None`
/// keeps them in memory only, so tests never read or overwrite the player's.
#[derive(Resource)]
pub(crate) struct PreferencesDir(pub(crate) Option<PathBuf>);

impl Default for PreferencesDir {
    fn default() -> Self {
        let directory = || -> Option<PathBuf> {
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
                .or_else(|| {
                    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config"))
                })?
                .join("afterward");

            #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
            let directory = PathBuf::from(std::env::var_os("HOME")?).join(".afterward");
            Some(directory)
        };
        Self(directory())
    }
}

impl PreferencesDir {
    pub(crate) fn file(&self, file_name: &str) -> Option<PathBuf> {
        self.0.as_ref().map(|directory| directory.join(file_name))
    }
}

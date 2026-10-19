mod data;
mod game;

use std::{error::Error, path::Path};

use bevy::prelude::*;
use bevy::window::{MonitorSelection, WindowMode};
use data::GameData;

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let game_data = GameData::load(&asset_root)?;
    let starts_fullscreen = game_data.gameplay.window.start_borderless_fullscreen;
    let is_resizable = game_data.gameplay.window.resizable_windowed;

    let asset_path = asset_root.to_string_lossy().into_owned();

    App::new()
        .insert_resource(game_data)
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_path,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Afterward".to_string(),
                        mode: if starts_fullscreen {
                            WindowMode::BorderlessFullscreen(MonitorSelection::Current)
                        } else {
                            WindowMode::Windowed
                        },
                        resizable: is_resizable,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(game::GamePlugin)
        .run();

    Ok(())
}

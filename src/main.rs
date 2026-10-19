mod data;
mod game;

use std::{error::Error, path::Path};

use bevy::{
    prelude::*,
    window::{MonitorSelection, WindowMode},
};
use data::GameData;

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let visual_walkthrough = std::env::args().any(|argument| argument == "--visual-walkthrough");
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let game_data = GameData::load(&asset_root)?;
    let starts_fullscreen = game_data.gameplay.window.start_borderless_fullscreen;
    let is_resizable = game_data.gameplay.window.resizable_windowed;

    let asset_path = asset_root.to_string_lossy().into_owned();

    let mut app = App::new();
    app.insert_resource(game_data)
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
                            WindowMode::BorderlessFullscreen(MonitorSelection::Primary)
                        } else {
                            WindowMode::Windowed
                        },
                        resizable: is_resizable,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(game::GamePlugin);

    if visual_walkthrough {
        app.add_plugins(game::VisualWalkthroughPlugin);
    }

    app.run();

    Ok(())
}

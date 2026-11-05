mod data;
mod game;

use std::{error::Error, path::Path};

use bevy::{
    prelude::*,
    window::{MonitorSelection, WindowMode},
};
use data::{GameData, WindowStartMode};

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let arguments = std::env::args().collect::<Vec<_>>();
    let visual_walkthrough = arguments
        .iter()
        .any(|argument| argument == "--visual-walkthrough");
    let start_room = arguments
        .iter()
        .position(|argument| argument == "--room")
        .map(|index| {
            arguments
                .get(index + 1)
                .ok_or("--room expects a room ID")?
                .parse::<game::StartRoom>()
        })
        .transpose()?
        .unwrap_or_default();
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let game_data = GameData::load(&asset_root)?;
    if !game_data.rooms.contains_key(&start_room.0) {
        return Err(format!(
            "--room {} does not exist; available rooms: {:?}",
            start_room.0,
            game_data.rooms.keys().collect::<Vec<_>>()
        )
        .into());
    }
    let start_mode = game_data.gameplay.window.start_mode;
    let mut window = Window {
        title: "Afterward".to_string(),
        mode: if start_mode == WindowStartMode::BorderlessFullscreen {
            WindowMode::BorderlessFullscreen(MonitorSelection::Primary)
        } else {
            WindowMode::Windowed
        },
        resizable: game_data.gameplay.window.resizable_windowed,
        ..default()
    };
    window.set_maximized(start_mode == WindowStartMode::Maximized);

    let asset_path = asset_root.to_string_lossy().into_owned();

    let mut app = App::new();
    app.insert_resource(game_data)
        .insert_resource(start_room)
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_path,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(window),
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

mod data;
mod game;

use std::{error::Error, path::Path, time::Duration};

use bevy::{
    app::ScheduleRunnerPlugin,
    audio::{AudioLoader, AudioPlugin, AudioSource},
    prelude::*,
    render::RenderPlugin,
    window::{ExitCondition, MonitorSelection, WindowMode},
    winit::WinitPlugin,
};
use data::{GameData, WindowStartMode};

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let arguments = std::env::args().collect::<Vec<_>>();
    let visual_walkthrough = arguments
        .iter()
        .position(|argument| argument == "--visual-walkthrough")
        .map(|index| {
            arguments
                .get(index + 1)
                .ok_or(
                    "--visual-walkthrough expects what to look at: `menus`, \
                     a room ID such as 3, or a span of rooms such as 3-5",
                )?
                .parse::<game::WalkthroughScope>()
        })
        .transpose()?;
    let room_shot = arguments.iter().any(|argument| argument == "--shot");
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
    // A walkthrough of rooms opens on the first of them.
    let start_room = match visual_walkthrough {
        Some(game::WalkthroughScope::Rooms { first, .. }) => game::StartRoom(first),
        _ => start_room,
    };
    // A release archive ships `assets/` beside the executable; a build run
    // from the checkout reads the sources in place.
    let asset_root = std::env::current_exe()?
        .parent()
        .map(|dir| dir.join("assets"))
        .filter(|dir| dir.is_dir())
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets"));
    let game_data = GameData::load(&asset_root)?;
    let last_room = match visual_walkthrough {
        Some(game::WalkthroughScope::Rooms { last, .. }) => last,
        _ => start_room.0,
    };
    for room in [start_room.0, last_room] {
        if !game_data.rooms.contains_key(&room) {
            return Err(format!(
                "room {room} does not exist; available rooms: {:?}",
                game_data.rooms.keys().collect::<Vec<_>>()
            )
            .into());
        }
    }
    let asset_plugin = AssetPlugin {
        file_path: asset_root.to_string_lossy().into_owned(),
        ..default()
    };

    let mut app = App::new();
    app.insert_resource(start_room);

    if room_shot {
        // A picture of the room and nothing else: no window to open, no sound
        // to play, and every shader compiled before the frame that needs it.
        app.insert_resource(game_data)
            .add_plugins(
                DefaultPlugins
                    .set(asset_plugin)
                    .set(WindowPlugin {
                        primary_window: None,
                        exit_condition: ExitCondition::DontExit,
                        ..default()
                    })
                    .set(RenderPlugin {
                        synchronous_pipeline_compilation: true,
                        ..default()
                    })
                    .disable::<WinitPlugin>()
                    .disable::<AudioPlugin>(),
            )
            .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
                1.0 / 60.0,
            )))
            // The sounds still load, so nothing complains; nothing plays them.
            .init_asset::<AudioSource>()
            .init_asset_loader::<AudioLoader>()
            .add_plugins(game::GamePlugin)
            .add_plugins(game::RoomShotPlugin {
                grid: arguments.iter().any(|argument| argument == "--grid"),
            });
        return match app.run() {
            AppExit::Success => Ok(()),
            AppExit::Error(_) => Err("the room shot failed".into()),
        };
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

    app.insert_resource(game_data)
        .add_plugins(DefaultPlugins.set(asset_plugin).set(WindowPlugin {
            primary_window: Some(window),
            ..default()
        }))
        .add_plugins(game::GamePlugin);

    if let Some(scope) = visual_walkthrough {
        app.add_plugins(game::VisualWalkthroughPlugin { scope });
    }

    app.run();

    Ok(())
}

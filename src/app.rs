use bevy::prelude::*;

use crate::bevy_game::GamePlugin;
use crate::bevy_game::config::VIEW_SIZE;

pub(crate) fn run() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "AFTERWARD".into(),
                resolution: (VIEW_SIZE.x as u32, VIEW_SIZE.y as u32).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(GamePlugin)
        .run();
}

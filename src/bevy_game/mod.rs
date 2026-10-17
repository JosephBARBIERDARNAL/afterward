use bevy::prelude::*;

mod arena;
mod combat;
pub(crate) mod config;
pub(crate) mod model;
mod progression;
mod ui;

use crate::presentation::visual::{
    animate_action_effects, animate_attack_telegraphs, animate_damage_numbers,
};
use crate::presentation::{audio::setup_hit_sounds, visual::setup_art};
use arena::{ArenaGeometry, sync_scene};
use combat::update_game;
use model::Game;
use ui::{sync_enemy_health_bars, sync_overlay, update_player_health_display};

pub(crate) struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Game>()
            .init_resource::<ArenaGeometry>()
            .add_systems(Startup, (setup_camera, setup_art, setup_hit_sounds))
            .add_systems(
                Update,
                (
                    sync_scene,
                    sync_overlay,
                    animate_action_effects,
                    animate_attack_telegraphs,
                    animate_damage_numbers,
                    update_game,
                    update_player_health_display,
                    sync_enemy_health_bars,
                )
                    .chain(),
            );
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scale: config::CAMERA_SCALE,
            ..OrthographicProjection::default_2d()
        }),
    ));
}

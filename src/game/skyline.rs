//! The city the screens standing over no room are drawn over — the main menu,
//! the difficulty it opens, the screen a run ends on — and the prologue writes
//! its words over: the sky and layers of `effects.skyline.profile`, panning
//! slowly by, the far towers crawling and the near ones passing, with the lit
//! windows breathing as they do behind a room.
//!
//! It is laid once, under one root kept on the camera and exactly covering the
//! viewport, behind everything while a menu stands over it and in front of the
//! room while the prologue does.

use super::*;
use crate::game::prologue::Prologue;
use bevy::image::ImageLoaderSettings;

/// Behind everything else the camera draws.
const SKYLINE_Z: f32 = -100.0;
/// In front of it instead, for the prologue: the city stands over the room
/// the run is leaving while its words are written.
const VEIL_Z: f32 = 50.0;
/// How far apart its layers stand, from the sky to the dark veil over them.
const LAYER_STEP_Z: f32 = 0.1;

/// The root every piece of the skyline hangs from, and the materials it fades
/// in and out with the prologue.
#[derive(Component)]
pub(super) struct Skyline {
    materials: Vec<Handle<CityLightsMaterial>>,
    /// How opaque it was last drawn, so a material is only touched when that
    /// changes.
    opacity: f32,
    /// How opaque the dark veil over it is drawn while it is whole.
    dim: f32,
}

/// One tile of a layer: which of the tiles laid side by side it is, how wide
/// a tile is, and how fast its layer slides, in world units per second.
#[derive(Component)]
pub(super) struct SkylineTile {
    slot: f32,
    width: f32,
    speed: f32,
}

/// The dark veil drawn over the layers, so the menus stand out against them.
#[derive(Component)]
pub(super) struct SkylineDim;

/// Lays the skyline, hidden until a screen stands over it.
pub(super) fn spawn_the_skyline(
    mut commands: Commands,
    data: Res<GameData>,
    asset_server: Res<AssetServer>,
    mut assets: ShaderAssets,
) {
    let config = &data.gameplay.effects.skyline;
    let profile = &data.backgrounds.profiles[&config.profile];
    let view = Vec2::new(
        data.gameplay.viewport_width_h(),
        data.gameplay.viewport.height_h,
    ) * H_WORLD_UNITS;
    let tile = Vec2::new(profile.tile_width_h(), profile.tile_h) * H_WORLD_UNITS;
    // Enough tiles side by side to cover the view wherever the slide stands.
    let slots = (view.x / tile.x).ceil() as i32 + 1;
    let layers = std::iter::once((profile.backdrop.as_str(), 0.0)).chain(
        profile
            .layers
            .iter()
            .map(|layer| (layer.asset.as_str(), layer.parallax)),
    );

    let mut materials = Vec::new();
    let mut tiles = Vec::new();
    for (depth, (asset, parallax)) in layers.enumerate() {
        let filtering = profile.filtering;
        let image = asset_server
            .load_builder()
            .with_settings(move |settings: &mut ImageLoaderSettings| {
                settings.sampler = filtering.into();
            })
            .load(asset.to_string());
        let material = assets.city_lights.add(CityLightsMaterial::new(
            image,
            Color::WHITE,
            depth,
            &data.gameplay.effects.city_lights,
        ));
        for slot in -1..slots {
            tiles.push((
                Mesh2d(assets.quad.0.clone()),
                MeshMaterial2d(material.clone()),
                Transform::from_xyz(0.0, 0.0, depth as f32 * LAYER_STEP_Z)
                    .with_scale(tile.extend(1.0)),
                SkylineTile {
                    slot: slot as f32,
                    width: tile.x,
                    speed: config.drift_h_s * parallax * H_WORLD_UNITS,
                },
            ));
        }
        materials.push(material);
    }
    let dim_z = (materials.len() + 1) as f32 * LAYER_STEP_Z;
    commands
        .spawn((
            Transform::from_xyz(0.0, 0.0, SKYLINE_Z),
            Visibility::Hidden,
            Skyline {
                materials,
                opacity: 1.0,
                dim: config.dim,
            },
        ))
        .with_children(|root| {
            for tile in tiles {
                root.spawn(tile);
            }
            root.spawn((
                Sprite::from_color(Color::BLACK.with_alpha(config.dim), view),
                Transform::from_xyz(0.0, 0.0, dim_z),
                SkylineDim,
            ));
        });
}

/// Shows the skyline under the screens that stand over no room, wherever the
/// camera was left standing — and stands it in front of the room instead
/// while the prologue writes over it.
pub(in crate::game) fn show_the_skyline(
    screen: Res<State<GameScreen>>,
    settings_return: Res<SettingsReturn>,
    prologue: Res<Prologue>,
    mut materials: ResMut<Assets<CityLightsMaterial>>,
    camera: Query<&Transform, (With<Camera2d>, Without<Skyline>)>,
    mut roots: Query<(&mut Transform, &mut Visibility, &mut Skyline)>,
    mut dims: Query<&mut Sprite, With<SkylineDim>>,
) {
    let Ok(camera) = camera.single() else {
        return;
    };
    let over_the_skyline = settings_return.over_the_skyline(*screen.get());
    let (depth, opacity) = if prologue.veil > 0.0 {
        (VEIL_Z, prologue.veil)
    } else {
        (SKYLINE_Z, 1.0)
    };
    for (mut transform, mut visibility, mut skyline) in &mut roots {
        transform.translation = camera.translation.truncate().extend(depth);
        *visibility = if over_the_skyline || prologue.veil > 0.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if skyline.opacity != opacity {
            skyline.opacity = opacity;
            for handle in &skyline.materials {
                if let Some(mut material) = materials.get_mut(handle) {
                    material.set_opacity(opacity);
                }
            }
            for mut dim in &mut dims {
                dim.color = Color::BLACK.with_alpha(skyline.dim * opacity);
            }
        }
    }
}

/// Slides every layer on at its own pace, wrapping a tile round once it has
/// slid a whole tile's width.
pub(in crate::game) fn slide_the_skyline(
    time: Res<Time>,
    mut tiles: Query<(&mut Transform, &SkylineTile)>,
) {
    let elapsed_s = time.elapsed_secs();
    for (mut transform, tile) in &mut tiles {
        let slid = (elapsed_s * tile.speed).rem_euclid(tile.width);
        transform.translation.x = tile.slot * tile.width - slid;
    }
}

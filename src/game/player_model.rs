//! Prototype 3D player: an off-screen orthographic camera renders the glTF
//! model into a texture, and that texture is the player's sprite image. The 2D
//! animation state machine keeps running unchanged; this module mirrors its
//! current clip onto the model's animations.

use super::*;
use bevy::{
    camera::{RenderTarget, ScalingMode},
    render::render_resource::TextureFormat,
    world_serialization::{WorldAssetRoot, WorldInstanceReady},
};
use std::time::Duration;

#[derive(Resource)]
pub(super) struct PlayerModelRig {
    graph: Handle<AnimationGraph>,
    /// Animation graph node and clip per player sprite-sheet key.
    nodes: BTreeMap<String, (AnimationNodeIndex, Handle<AnimationClip>)>,
    playing: Option<String>,
    last_action_timer: f32,
}

/// The glTF entity whose `AnimationPlayer` animates the player model.
#[derive(Component)]
pub(super) struct PlayerModelAnimator;

pub(super) fn setup_player_model(
    mut commands: Commands,
    data: Res<GameData>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    mut sheets: ResMut<SpriteSheets>,
) {
    let model = &data.player_model;
    if !model.enabled {
        return;
    }
    let size = model.render_size_px;
    let target = images.add(Image::new_target_texture(
        size,
        size,
        TextureFormat::Rgba8Unorm,
        Some(TextureFormat::Rgba8UnormSrgb),
    ));

    // Every player clip shows the whole render texture on each of its frames,
    // so the clip timings from animations.yaml keep driving gameplay.
    for (key, clip) in data.player_animations.keyed() {
        let mut layout = TextureAtlasLayout::new_empty(UVec2::splat(size));
        for _ in 0..clip.frames {
            layout.add_texture(URect::new(0, 0, size, size));
        }
        sheets.0.insert(
            key,
            SpriteSheet {
                image: target.clone(),
                layout: atlas_layouts.add(layout),
                frame_size: Vec2::splat(size as f32),
                frames: clip.frames,
                fps: clip.fps,
            },
        );
    }

    let mut graph = AnimationGraph::new();
    let nodes = model
        .clips
        .iter()
        .map(|(name, index)| {
            let clip = asset_server
                .load(GltfAssetLabel::Animation(*index).from_asset(model.scene.clone()));
            let node = graph.add_clip(clip.clone(), 1.0, graph.root);
            (format!("player_{name}"), (node, clip))
        })
        .collect();
    commands.insert_resource(PlayerModelRig {
        graph: graphs.add(graph),
        nodes,
        playing: None,
        last_action_timer: 0.0,
    });

    commands
        .spawn((
            WorldAssetRoot(
                asset_server.load(GltfAssetLabel::Scene(0).from_asset(model.scene.clone())),
            ),
            Transform::from_rotation(Quat::from_rotation_y(model.facing_yaw_degrees.to_radians())),
        ))
        .observe(
            |ready: On<WorldInstanceReady>,
             mut commands: Commands,
             children: Query<&Children>,
             players: Query<(), With<AnimationPlayer>>,
             rig: Res<PlayerModelRig>| {
                for entity in children.iter_descendants(ready.entity) {
                    if players.contains(entity) {
                        commands.entity(entity).insert((
                            AnimationGraphHandle(rig.graph.clone()),
                            AnimationTransitions::new(),
                            PlayerModelAnimator,
                        ));
                    }
                }
            },
        );

    // The camera frames `view_height` model units with the feet `feet_margin`
    // above the bottom edge; `world::spawn_player` uses the same framing to
    // place the texture on the collision box.
    let center_y = model.view_height / 2.0 - model.feet_margin;
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: -1,
            clear_color: ClearColorConfig::Custom(Color::NONE),
            ..default()
        },
        RenderTarget::Image(target.into()),
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::Fixed {
                width: model.view_height,
                height: model.view_height,
            },
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(0.0, center_y, 20.0).looking_at(Vec3::new(0.0, center_y, 0.0), Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 6000.0,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

/// Plays the model animation mapped to the player's current sprite clip,
/// restarting one-shot actions (sword swings) each time they replay.
pub(super) fn animate_player_model(
    data: Res<GameData>,
    rig: Option<ResMut<PlayerModelRig>>,
    clips: Res<Assets<AnimationClip>>,
    players: Query<(&PlayerBody, &AnimatedSprite)>,
    mut animators: Query<
        (&mut AnimationPlayer, &mut AnimationTransitions),
        With<PlayerModelAnimator>,
    >,
) {
    let Some(mut rig) = rig else {
        return;
    };
    let (Ok((body, sprite_animation)), Ok((mut animation_player, mut transitions))) =
        (players.single(), animators.single_mut())
    else {
        return;
    };
    let restarted = body.action_animation_timer > rig.last_action_timer;
    rig.last_action_timer = body.action_animation_timer;
    if !restarted && rig.playing.as_deref() == Some(sprite_animation.sheet.as_str()) {
        return;
    }
    let Some((node, clip)) = rig.nodes.get(&sprite_animation.sheet).cloned() else {
        return;
    };
    let active = transitions.play(
        &mut animation_player,
        node,
        Duration::from_secs_f32(data.player_model.blend_s),
    );
    if sprite_animation.looping {
        active.repeat();
    } else {
        // Fit one-shot model animations into the sprite clip's duration.
        let sprite_duration =
            sprite_animation.frames as f32 * sprite_animation.timer.duration().as_secs_f32();
        if let Some(clip) = clips.get(&clip) {
            active.set_speed(clip.duration() / sprite_duration);
        }
        active.replay();
    }
    rig.playing = Some(sprite_animation.sheet.clone());
}

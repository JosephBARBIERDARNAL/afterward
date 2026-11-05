use super::*;
use crate::data::RoomConfig;
use bevy::{image::ImageLoaderSettings, sprite::Anchor};

const ENEMY_SPRITE_SCALE: f32 = 0.62;

pub(super) fn spawn_room(
    mut commands: Commands,
    data: Res<GameData>,
    mut session: ResMut<RunSession>,
    mut camera: Query<(&mut Transform, &mut RoomCamera), With<Camera2d>>,
    asset_server: Res<AssetServer>,
    sheets: Res<SpriteSheets>,
) {
    if session.world_loaded {
        return;
    }
    let room = &data.rooms[&session.room_id];
    draw_room(&mut commands, room, &data, &asset_server);
    let ammo = session.magazine_capacity(&data);
    spawn_player(&mut commands, room, &data, &sheets, ammo);
    spawn_enemies(&mut commands, room, &data, &sheets);

    if let Ok((mut transform, mut rig)) = camera.single_mut() {
        rig.size_h = Vec2::new(room.width_h(&data.gameplay), room.size.height_h);
        let center_h = rig.clamp_center_h(
            Vec2::new(room.player_spawn.x_h, room.player_spawn.y_h + 0.5),
            &data.gameplay,
        );
        transform.translation.x = center_h.x * H_WORLD_UNITS;
        transform.translation.y = center_h.y * H_WORLD_UNITS;
    }
    session.world_loaded = true;
}

pub(super) fn draw_room(
    commands: &mut Commands,
    room: &RoomConfig,
    data: &GameData,
    asset_server: &AssetServer,
) {
    let profile = &data.backgrounds.profiles[&room.background_profile];
    let tile_h = profile.tile_h;
    let tile_width_h =
        tile_h * profile.source_size_px.width as f32 / profile.source_size_px.height as f32;
    let room_width_h = room.width_h(&data.gameplay);
    let first_tile = if profile.repeat_horizontal { -2 } else { 0 };
    let last_tile = if profile.repeat_horizontal {
        (room_width_h / tile_width_h).ceil() as i32 + 2
    } else {
        0
    };
    let mut layers = Vec::with_capacity(profile.layers.len() + 1);
    layers.push((profile.backdrop.as_str(), 0.0));
    layers.extend(
        profile
            .layers
            .iter()
            .map(|layer| (layer.asset.as_str(), layer.parallax)),
    );

    let filtering = profile.filtering;
    for (layer_index, (asset, parallax)) in layers.into_iter().enumerate() {
        let image = asset_server
            .load_builder()
            .with_settings(move |settings: &mut ImageLoaderSettings| {
                settings.sampler = filtering.into();
            })
            .load(asset.to_string());
        for tile_index in first_tile..=last_tile {
            let base_x = (tile_index as f32 + 0.5) * tile_width_h * H_WORLD_UNITS;
            let mut sprite = Sprite::from_image(image.clone());
            sprite.custom_size = Some(Vec2::new(
                tile_width_h * H_WORLD_UNITS,
                tile_h * H_WORLD_UNITS,
            ));
            commands.spawn((
                sprite,
                // `update_background_parallax` keeps it centered on the camera vertically.
                Transform::from_xyz(base_x, 0.0, -80.0 + layer_index as f32),
                BackgroundParallax {
                    base_x,
                    factor: parallax,
                },
                RoomEntity,
            ));
        }
    }

    spawn_room_platforms(commands, room);

    for decoration in &room.decorations {
        let mut sprite = Sprite::from_image(asset_server.load(decoration.asset.clone()));
        sprite.custom_size = Some(Vec2::splat(H_WORLD_UNITS));
        let z = if decoration.visual_layer == "foreground" {
            6.0
        } else {
            -10.0
        };
        commands.spawn((
            sprite,
            Transform::from_xyz(
                decoration.position_h.x_h * H_WORLD_UNITS,
                decoration.position_h.y_h * H_WORLD_UNITS,
                z,
            ),
            RoomEntity,
        ));
    }
}

pub(super) fn spawn_room_platforms(commands: &mut Commands, room: &RoomConfig) {
    let blocks = room
        .terrain
        .iter()
        .map(|block| (block, false))
        .chain(room.platforms.iter().map(|block| (block, true)));
    for (block, one_way) in blocks {
        let rect = block.rect_h;
        let surface = PlatformSurface {
            left: rect.x * H_WORLD_UNITS,
            right: rect.right() * H_WORLD_UNITS,
            bottom: rect.y * H_WORLD_UNITS,
            top: rect.top() * H_WORLD_UNITS,
            material: block.surface,
            one_way,
        };
        let size = Vec2::new(surface.right - surface.left, surface.top - surface.bottom);
        let center = Vec2::new(
            (surface.left + surface.right) / 2.0,
            (surface.bottom + surface.top) / 2.0,
        );
        commands.spawn((surface, RoomEntity));
        // Placeholder art, kept separate from the data-only collider so a
        // future tileset can replace these shapes without changing physics.
        let (body, edge) = match block.surface {
            PlatformMaterial::Soil => (Color::srgb(0.17, 0.13, 0.11), Color::srgb(0.29, 0.38, 0.2)),
            PlatformMaterial::Stone => {
                (Color::srgb(0.2, 0.21, 0.24), Color::srgb(0.42, 0.43, 0.42))
            }
            PlatformMaterial::Wood => {
                (Color::srgb(0.36, 0.24, 0.14), Color::srgb(0.55, 0.39, 0.22))
            }
        };
        commands.spawn((
            Sprite::from_color(body, size),
            Transform::from_xyz(center.x, center.y, 0.0),
            RoomEntity,
        ));
        let edge_height = 5.0_f32.min(size.y);
        commands.spawn((
            Sprite::from_color(edge, Vec2::new(size.x, edge_height)),
            Transform::from_xyz(center.x, surface.top - edge_height / 2.0, 0.2),
            RoomEntity,
        ));
    }
}

pub(super) fn spawn_player(
    commands: &mut Commands,
    room: &RoomConfig,
    data: &GameData,
    sheets: &SpriteSheets,
    ammo: u32,
) {
    let animations = &data.player_animations;
    let key = "player_idle";
    let sheet = &sheets.0[key];
    let mut sprite = Sprite::from_atlas_image(
        sheet.image.clone(),
        TextureAtlas {
            layout: sheet.layout.clone(),
            index: 0,
        },
    );
    // Draw the body `body_height_h` tall with its feet on the collision box's
    // baseline; the anchor sits at the box's center, half an H above the feet.
    sprite.custom_size = Some(
        sheet.frame_size * animations.body_height_h * H_WORLD_UNITS / animations.body_height_px,
    );
    let px_per_h = animations.body_height_px / animations.body_height_h;
    let body_center_from_bottom_px = sheet.frame_size.y - animations.feet_row_px + px_per_h / 2.0;
    commands.spawn((
        sprite,
        Anchor(Vec2::new(
            0.0,
            body_center_from_bottom_px / sheet.frame_size.y - 0.5,
        )),
        Transform::from_xyz(
            room.player_spawn.x_h * H_WORLD_UNITS,
            (room.player_spawn.y_h + 0.5) * H_WORLD_UNITS,
            4.0,
        ),
        AnimatedSprite {
            sheet: key.to_string(),
            frame: 0,
            frames: sheet.frames,
            timer: Timer::from_seconds(1.0 / sheet.fps, TimerMode::Repeating),
            looping: true,
        },
        PlayerBody { ammo, ..default() },
        RoomEntity,
    ));
}

pub(super) fn spawn_enemies(
    commands: &mut Commands,
    room: &RoomConfig,
    data: &GameData,
    sheets: &SpriteSheets,
) {
    // Enemies appear one after another, evenly spread over the intro window.
    let appear_spread_s = data.gameplay.room_flow.room_intro.enemy_appear_spread_s;
    let last_index = room.enemy_spawns.len().saturating_sub(1).max(1) as f32;
    for (index, spawn) in room.enemy_spawns.iter().enumerate() {
        let profile = &data.enemies.types[&spawn.type_id];
        let key = spawn.type_id.idle_sheet();
        let sheet = &sheets.0[key];
        let scale = if spawn.type_id == "evil_wizard" {
            1.2
        } else {
            ENEMY_SPRITE_SCALE
        };
        // Source pixels: empty rows above and below the figure, and the height
        // of the entity origin above the feet. Combat and AI measure from that
        // origin, so it sits mid-body rather than mid-frame for padded sheets.
        let (frame_top_padding_px, frame_bottom_padding_px, origin_above_feet_px) =
            match &*spawn.type_id {
                "swordsman" => (61.0, 0.0, sheet.frame_size.y / 2.0),
                "infantryman" => (56.0, 0.0, sheet.frame_size.y / 2.0),
                "destroyer" | "gorgon_1" | "gorgon_2" | "gorgon_3" => {
                    (51.0, 0.0, sheet.frame_size.y / 2.0)
                }
                "evil_wizard" => (63.0, 83.0, 26.0),
                _ => (0.0, 0.0, sheet.frame_size.y / 2.0),
            };
        let mut attacks = if profile.attacks.is_empty() {
            profile
                .attack
                .iter()
                .map(EnemyAttack::from)
                .collect::<Vec<_>>()
        } else {
            profile
                .attacks
                .iter()
                .map(EnemyAttack::from)
                .collect::<Vec<_>>()
        };
        let health = spawn
            .stat_overrides
            .as_ref()
            .and_then(|overrides| overrides.health)
            .unwrap_or(profile.health);
        let movement_speed_h_s = spawn
            .stat_overrides
            .as_ref()
            .and_then(|overrides| overrides.movement_speed_h_s)
            .unwrap_or(profile.movement_speed_h_s);
        if let Some(damage) = spawn
            .stat_overrides
            .as_ref()
            .and_then(|overrides| overrides.attack_damage)
        {
            for attack in &mut attacks {
                attack.damage = damage;
            }
        }
        let preferred_distance_h = profile
            .preferred_distance_h
            .as_ref()
            .map(|range| (range.min, range.max));
        let phase_two = profile
            .phase_two
            .as_ref()
            .map(|phase| (phase.health_fraction_at_or_below, phase.cooldown_multiplier));
        let feet_offset = origin_above_feet_px * scale;
        let visible_top_offset = (sheet.frame_size.y
            - frame_top_padding_px
            - frame_bottom_padding_px
            - origin_above_feet_px)
            * scale;
        let home_y = spawn.position_h.y_h * H_WORLD_UNITS + feet_offset;
        let enemy = commands
            .spawn((
                Sprite::from_atlas_image(
                    sheet.image.clone(),
                    TextureAtlas {
                        layout: sheet.layout.clone(),
                        index: 0,
                    },
                ),
                Anchor(Vec2::new(
                    0.0,
                    (frame_bottom_padding_px + origin_above_feet_px) / sheet.frame_size.y - 0.5,
                )),
                Transform::from_xyz(spawn.position_h.x_h * H_WORLD_UNITS, home_y, 3.0)
                    .with_scale(Vec3::splat(scale)),
                AnimatedSprite {
                    sheet: key.to_string(),
                    frame: 0,
                    frames: sheet.frames,
                    timer: Timer::from_seconds(1.0 / sheet.fps, TimerMode::Repeating),
                    looping: true,
                },
                EnemyBody {
                    type_id: spawn.type_id.clone(),
                    health,
                    max_health: health,
                    movement_speed_h_s,
                    home_y,
                    feet_offset,
                    visible_top_offset,
                    vertical_velocity: 0.0,
                    grounded: true,
                    jump_height_h: profile
                        .jump_height_h
                        .unwrap_or(data.enemies.jumping.jump_height_h),
                    jump_cooldown: 0.0,
                    jump: None,
                    drop_through_timer: 0.0,
                    behavior: profile.behavior.clone(),
                    procedural_movement: profile.procedural_movement.as_ref().map(|movement| {
                        ProceduralMovementState {
                            phase_radians: 0.0,
                            bob_height_h: movement.bob_height_h,
                            cycles_per_h: movement.cycles_per_h,
                            lean_radians: movement.lean_degrees.to_radians(),
                        }
                    }),
                    preferred_distance_h,
                    attacks,
                    engaged: false,
                    attack_cooldown: 0.0,
                    windup_remaining: 0.0,
                    combo_hit_pending: false,
                    active_attack: 0,
                    next_attack: 0,
                    hurt_timer: 0.0,
                    knockback_velocity: 0.0,
                    phase_two,
                    phase_two_announced: false,
                    marked_timer: 0.0,
                    movement_sound_timer: 0.0,
                },
                room_intro::EnemyEntrance {
                    base_scale: Vec3::splat(scale),
                    delay_s: index as f32 / last_index * appear_spread_s,
                },
                RoomEntity,
            ))
            .id();
        let bar_position = Vec3::new(
            spawn.position_h.x_h * H_WORLD_UNITS,
            home_y + visible_top_offset + 6.0,
            5.0,
        );
        commands.spawn((
            Sprite::from_color(Color::BLACK, Vec2::new(52.0, 6.0)),
            Transform::from_translation(bar_position - Vec3::Z * 0.1),
            EnemyHealthBarBackground { enemy },
            RoomEntity,
        ));
        commands.spawn((
            Sprite::from_color(Color::srgb(0.8, 0.12, 0.2), Vec2::new(48.0, 4.0)),
            Transform::from_translation(bar_position),
            EnemyHealthBar {
                enemy,
                max_health: health,
            },
            RoomEntity,
        ));
    }
}

pub(super) fn cleanup_room(
    mut commands: Commands,
    entities: Query<Entity, With<RoomEntity>>,
    mut session: ResMut<RunSession>,
) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
    session.world_loaded = false;
}

#[derive(Component)]
pub(crate) struct RoomEntity;

#[derive(Component)]
pub(crate) struct ExitDoor;

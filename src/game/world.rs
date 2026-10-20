use super::*;
use crate::data::RoomConfig;
use bevy::sprite::Anchor;

const PLAYER_VISUAL_HEIGHT_FACTOR: f32 = 1.4;
const PLAYER_SPRITE_BOTTOM_PADDING_FRACTION: f32 = 0.2;
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
    spawn_player(&mut commands, room, &sheets);
    spawn_enemies(&mut commands, room, &data, &sheets);

    if let Ok((mut transform, mut rig)) = camera.single_mut() {
        rig.width_h = room.width_h(&data.gameplay);
        rig.fixed_y_h = room.camera.fixed_y_h;
        rig.follows_player = room.camera.follow_player_x;
        let half_viewport_width_h = data.gameplay.viewport_width_h() / 2.0;
        transform.translation.x = room
            .player_spawn
            .x_h
            .clamp(half_viewport_width_h, rig.width_h - half_viewport_width_h)
            * H_WORLD_UNITS;
        transform.translation.y = rig.fixed_y_h * H_WORLD_UNITS;
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

    for (layer_index, (asset, parallax)) in layers.into_iter().enumerate() {
        for tile_index in first_tile..=last_tile {
            let base_x = (tile_index as f32 + 0.5) * tile_width_h * H_WORLD_UNITS;
            let mut sprite = Sprite::from_image(asset_server.load(asset.to_string()));
            sprite.custom_size = Some(Vec2::new(
                tile_width_h * H_WORLD_UNITS,
                tile_h * H_WORLD_UNITS,
            ));
            commands.spawn((
                sprite,
                Transform::from_xyz(
                    base_x,
                    room.camera.fixed_y_h * H_WORLD_UNITS,
                    -80.0 + layer_index as f32,
                ),
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
    for platform in &room.platforms {
        let center = Vec2::new(
            platform.position_h.x_h * H_WORLD_UNITS,
            platform.position_h.y_h * H_WORLD_UNITS,
        );
        let size = Vec2::new(
            platform.size_h.width_h * H_WORLD_UNITS,
            platform.size_h.height_h * H_WORLD_UNITS,
        );
        commands.spawn((
            PlatformSurface {
                left: center.x - size.x / 2.0,
                right: center.x + size.x / 2.0,
                bottom: center.y - size.y / 2.0,
                top: center.y + size.y / 2.0,
                material: platform.surface,
            },
            RoomEntity,
        ));
        if platform.surface == PlatformMaterial::Stone {
            // Keep placeholder art separate from the data-only collider so a future
            // stone sprite or tilemap can replace these shapes without changing physics.
            commands.spawn((
                Sprite::from_color(Color::srgb(0.32, 0.34, 0.36), size),
                Transform::from_xyz(center.x, center.y, 0.0),
                RoomEntity,
            ));
            commands.spawn((
                Sprite::from_color(Color::srgb(0.48, 0.49, 0.46), Vec2::new(size.x, 5.0)),
                Transform::from_xyz(center.x, center.y + size.y / 2.0 - 2.5, 0.2),
                RoomEntity,
            ));
        }
    }
}

pub(super) fn spawn_player(commands: &mut Commands, room: &RoomConfig, sheets: &SpriteSheets) {
    let key = "player_idle_sword";
    let sheet = &sheets.0[key];
    let mut sprite = Sprite::from_atlas_image(
        sheet.image.clone(),
        TextureAtlas {
            layout: sheet.layout.clone(),
            index: 0,
        },
    );
    let visual_height = H_WORLD_UNITS * PLAYER_VISUAL_HEIGHT_FACTOR;
    sprite.custom_size = Some(Vec2::new(
        sheet.frame_size.x * visual_height / sheet.frame_size.y,
        visual_height,
    ));
    commands.spawn((
        sprite,
        Anchor(Vec2::new(
            0.0,
            (1.0 - PLAYER_VISUAL_HEIGHT_FACTOR) / (2.0 * PLAYER_VISUAL_HEIGHT_FACTOR)
                + PLAYER_SPRITE_BOTTOM_PADDING_FRACTION,
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
        PlayerBody {
            velocity: Vec2::ZERO,
            facing: 1.0,
            weapon_mode: PlayerWeapon::Sword,
            grounded: true,
            jumps_used: 0,
            dash_timer: 0.0,
            dash_cooldown: 0.0,
            dash_direction: 1.0,
            sword_cooldown: 0.0,
            gun_cooldown: 0.0,
            invulnerability_timer: 0.0,
            dash_invulnerability_timer: 0.0,
            dash_strike_timer: 0.0,
            stationary_timer: 0.0,
            rhythm_timer: 0.0,
            rhythm_stacks: 0,
            chain_reaction_timer: 0.0,
            sated_fury_timer: 0.0,
            action_animation_timer: 0.0,
            footstep_timer: 0.0,
        },
        RoomEntity,
    ));
}

pub(super) fn spawn_enemies(
    commands: &mut Commands,
    room: &RoomConfig,
    data: &GameData,
    sheets: &SpriteSheets,
) {
    for spawn in &room.enemy_spawns {
        let profile = &data.enemies.types[&spawn.type_id];
        let key = spawn.type_id.idle_sheet();
        let sheet = &sheets.0[key];
        let scale = if spawn.type_id == "warrior" {
            2.0
        } else {
            ENEMY_SPRITE_SCALE
        };
        let (frame_top_padding_px, frame_bottom_padding_px) = match &*spawn.type_id {
            "swordsman" => (61.0, 0.0),
            "infantryman" => (56.0, 0.0),
            "destroyer" | "gorgon_1" | "gorgon_2" | "gorgon_3" => (51.0, 0.0),
            "warrior" => (10.0, 7.0),
            _ => (0.0, 0.0),
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
        let visual_height = sheet.frame_size.y * scale;
        let visible_top_offset =
            (sheet.frame_size.y / 2.0 - frame_top_padding_px - frame_bottom_padding_px) * scale;
        let home_y = spawn.position_h.y_h * H_WORLD_UNITS + visual_height / 2.0;
        let enemy = commands
            .spawn((
                Sprite::from_atlas_image(
                    sheet.image.clone(),
                    TextureAtlas {
                        layout: sheet.layout.clone(),
                        index: 0,
                    },
                ),
                Anchor(Vec2::new(0.0, frame_bottom_padding_px / sheet.frame_size.y)),
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
                    visual_height,
                    visible_top_offset,
                    vertical_velocity: 0.0,
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

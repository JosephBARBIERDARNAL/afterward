use super::*;
use crate::data::RoomConfig;
use bevy::{image::ImageLoaderSettings, sprite::Anchor};

const ENEMY_SPRITE_SCALE: f32 = 0.62;

/// What building a room reads and writes: the rooms themselves, the run it is
/// built for, and the slide that may have brought the player to its door.
#[derive(SystemParam)]
pub(super) struct RoomBuild<'w> {
    data: Res<'w, GameData>,
    pan: ResMut<'w, RoomPan>,
    session: ResMut<'w, RunSession>,
}

pub(super) fn spawn_room(
    mut commands: Commands,
    build: RoomBuild,
    mut camera: Query<(&mut Transform, &mut RoomCamera), With<Camera2d>>,
    asset_server: Res<AssetServer>,
    sheets: Res<SpriteSheets>,
    mut shaders: ShaderAssets,
) {
    let RoomBuild {
        data,
        mut pan,
        mut session,
    } = build;
    if session.world_loaded {
        return;
    }
    // Until the slide says otherwise, this room is walked into from nowhere:
    // its entrance starts over rather than carrying on from a crossing.
    pan.came_through_a_door = false;
    let room = &data.rooms[&session.room_id];
    draw_room(&mut commands, room, &data, &asset_server, &mut shaders);
    spawn_player(&mut commands, room, &data, &sheets);
    if session.has_enemies(&data) {
        spawn_enemies(&mut commands, room, &data, &sheets);
    }
    spawn_exit_door(&mut commands, room, &data, &sheets);
    spawn_entry_door(&mut commands, room, &data, &sheets);

    if let Ok((mut transform, mut rig)) = camera.single_mut() {
        rig.size_h = Vec2::new(room.size.width_h, room.size.height_h);
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
    shaders: &mut ShaderAssets,
) {
    // The profiles are mixed rather than laid one over the other. Their skies
    // are blended in equal parts, each drawn over the ones before it at the
    // share that leaves them all even. Their layers are then dealt together
    // from the furthest to the nearest, by how fast each scrolls, so one
    // skyline's far towers stand behind the other's near ones.
    let profiles = room
        .background_profiles
        .iter()
        .map(|profile| &data.backgrounds.profiles[profile])
        .collect::<Vec<_>>();
    let skies = profiles.iter().enumerate().map(|(index, profile)| {
        (
            *profile,
            profile.backdrop.as_str(),
            0.0,
            1.0 / (index + 1) as f32,
        )
    });
    let mut scenery = profiles
        .iter()
        .flat_map(|profile| {
            profile
                .layers
                .iter()
                .map(|layer| (*profile, layer.asset.as_str(), layer.parallax, 1.0))
        })
        .collect::<Vec<_>>();
    scenery.sort_by(|first, second| first.2.total_cmp(&second.2));
    for (depth, (profile, asset, parallax, alpha)) in skies.chain(scenery).enumerate() {
        let tile = Vec2::new(profile.tile_width_h(), profile.tile_h) * H_WORLD_UNITS;
        let (first_tile, last_tile) = if profile.repeat_horizontal {
            (
                -2,
                (room.size.width_h * H_WORLD_UNITS / tile.x).ceil() as i32 + 2,
            )
        } else {
            (0, 0)
        };
        let filtering = profile.filtering;
        let image = asset_server
            .load_builder()
            .with_settings(move |settings: &mut ImageLoaderSettings| {
                settings.sampler = filtering.into();
            })
            .load(asset.to_string());
        // One material per layer, so its tiles batch together: it draws the
        // layer as a sprite would, with its lit windows and stars breathing.
        let material = shaders.city_lights.add(CityLightsMaterial::new(
            image,
            Color::WHITE.with_alpha(alpha),
            depth,
            &data.gameplay.effects.city_lights,
        ));
        for tile_index in first_tile..=last_tile {
            let base_x = (tile_index as f32 + 0.5) * tile.x;
            commands.spawn((
                Mesh2d(shaders.quad.0.clone()),
                MeshMaterial2d(material.clone()),
                // `update_background_parallax` keeps it centered on the
                // camera vertically.
                Transform::from_xyz(base_x, 0.0, -80.0 + depth as f32).with_scale(tile.extend(1.0)),
                BackgroundParallax {
                    base_x,
                    factor: parallax,
                    tile,
                },
                RoomEntity,
            ));
        }
    }

    spawn_room_platforms(commands, room, data);
    room_art::draw(commands, room, data, asset_server);
}

/// Spawns the colliders of the room's platforms; `room_art::draw` draws them.
pub(super) fn spawn_room_platforms(commands: &mut Commands, room: &RoomConfig, data: &GameData) {
    for block in &room.platforms {
        commands.spawn((PlatformSurface::of(block, data), RoomEntity));
    }
}

/// The exit door, standing on the room's `exit`. It is locked until the room's
/// bonus item has been picked up and a bonus chosen.
pub(super) fn spawn_exit_door(
    commands: &mut Commands,
    room: &RoomConfig,
    data: &GameData,
    sheets: &SpriteSheets,
) {
    let interact = &data.gameplay.room_flow.interact;
    spawn_door(
        commands,
        Vec2::new(room.exit.x_h, room.exit.y_h),
        data,
        sheets,
        (
            ExitDoor,
            Interactable {
                reach_h: Vec2::new(interact.door_reach_x_h, interact.door_reach_y_h),
                top_h: data.gameplay.room_flow.exit_door.height_h / 2.0,
            },
        ),
    );
}

/// The door the player came in through, standing where their walk into the
/// room begins: `room_intro` opens it, walks them out of it, and sends it
/// away once they are in.
pub(super) fn spawn_entry_door(
    commands: &mut Commands,
    room: &RoomConfig,
    data: &GameData,
    sheets: &SpriteSheets,
) {
    let intro = &data.gameplay.room_flow.room_intro;
    spawn_door(
        commands,
        Vec2::new(
            room.player_spawn.x_h - intro.player_walk_in_s * intro.player_walk_in_speed_h_s,
            room.player_spawn.y_h,
        ),
        data,
        sheets,
        EntryDoor,
    );
}

/// A closed door from the exit-door sheet, standing with its foot on `at_h`.
fn spawn_door(
    commands: &mut Commands,
    at_h: Vec2,
    data: &GameData,
    sheets: &SpriteSheets,
    door: impl Bundle,
) {
    let sheet = &sheets.0[ExitDoor::SHEET];
    let height = data.gameplay.room_flow.exit_door.height_h * H_WORLD_UNITS;
    let mut sprite = Sprite::from_atlas_image(
        sheet.image.clone(),
        TextureAtlas {
            layout: sheet.layout.clone(),
            index: 0,
        },
    );
    sprite.custom_size = Some(Vec2::new(
        height * sheet.frame_size.x / sheet.frame_size.y,
        height,
    ));
    commands.spawn((
        sprite,
        Transform::from_xyz(
            at_h.x * H_WORLD_UNITS,
            at_h.y * H_WORLD_UNITS + height / 2.0,
            2.0,
        ),
        door,
        RoomEntity,
    ));
}

pub(super) fn spawn_player(
    commands: &mut Commands,
    room: &RoomConfig,
    data: &GameData,
    sheets: &SpriteSheets,
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
        PlayerBody::default(),
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
        let scale = match (&profile.sprite, &*spawn.type_id) {
            (Some(sprite), _) => sprite.scale,
            (None, "emitter_unit") => 1.2,
            // Pixel art, so a whole number: 31 px of knight stand 1.9 H tall.
            (None, "black_knight") => 3.0,
            (None, _) => ENEMY_SPRITE_SCALE,
        };
        // Source pixels: empty rows above and below the figure, and the height
        // of the entity origin above the feet. Combat and AI measure from that
        // origin, so it sits mid-body rather than mid-frame for padded sheets.
        let (frame_top_padding_px, frame_bottom_padding_px, origin_above_feet_px) =
            match (&profile.sprite, &*spawn.type_id) {
                (Some(sprite), _) => (
                    sprite.top_padding_px,
                    sprite.bottom_padding_px,
                    sprite.origin_above_feet_px,
                ),
                (None, "blade_unit" | "dark_blade_unit") => (61.0, 0.0, sheet.frame_size.y / 2.0),
                (None, "rifle_unit" | "dark_rifle_unit") => (56.0, 0.0, sheet.frame_size.y / 2.0),
                (None, "heavy_unit" | "dark_heavy_unit") => (51.0, 0.0, sheet.frame_size.y / 2.0),
                (None, "emitter_unit") => (63.0, 83.0, 26.0),
                (None, "black_knight") => (31.0, 0.0, 16.0),
                (None, _) => (0.0, 0.0, sheet.frame_size.y / 2.0),
            };
        let attacks = if profile.attacks.is_empty() {
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
        let health = profile.health;
        let armor = spawn.armor_layers;
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
        let facing = f32::from(spawn.facing);
        let enemy = commands
            .spawn((
                Sprite {
                    // An idle enemy watches the way the room points it.
                    flip_x: facing < 0.0,
                    ..Sprite::from_atlas_image(
                        sheet.image.clone(),
                        TextureAtlas {
                            layout: sheet.layout.clone(),
                            index: 0,
                        },
                    )
                },
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
                    material: profile.material,
                    armor,
                    health,
                    max_health: health,
                    movement_speed_h_s: profile.movement_speed_h_s,
                    home_y,
                    feet_offset,
                    visible_top_offset,
                    hurtbox_size: Vec2::new(profile.hurtbox_h.width, profile.hurtbox_h.height)
                        * H_WORLD_UNITS,
                    vertical_velocity: 0.0,
                    grounded: true,
                    jump_height_h: profile
                        .jump_height_h
                        .unwrap_or(data.enemies.jumping.jump_height_h),
                    jump_cooldown: 0.0,
                    jump: None,
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
                    facing,
                    engaged: false,
                    attack_cooldown: 0.0,
                    windup_remaining: 0.0,
                    combo_hit_pending: false,
                    active_attack: 0,
                    next_attack: 0,
                    hurt_timer: 0.0,
                    flash_s: 0.0,
                    knockback_velocity: 0.0,
                    phase_two,
                    phase_two_announced: false,
                    marked_timer: 0.0,
                    movement_sound_timer: 0.0,
                    stance: None,
                    countering: false,
                    wounds: 0,
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
            Sprite::from_color(
                if armor > 0 {
                    combat::SHIELDED_HEALTH_BAR
                } else {
                    combat::HEALTH_BAR
                },
                Vec2::new(48.0, 4.0),
            ),
            Transform::from_translation(bar_position),
            EnemyHealthBar {
                enemy,
                max_health: health,
            },
            RoomEntity,
        ));
        let Some(shield) = sheets.0.get(combat::SHIELD_SHEET) else {
            continue;
        };
        for layer in 0..armor {
            let icon = EnemyShieldIcon::of_layer(enemy, layer, armor);
            commands.spawn((
                Sprite::from_atlas_image(
                    shield.image.clone(),
                    TextureAtlas {
                        layout: shield.layout.clone(),
                        index: 0,
                    },
                ),
                Transform::from_translation(
                    bar_position + Vec3::new(icon.offset_x, combat::SHIELD_OFFSET_Y, 0.0),
                ),
                icon,
                RoomEntity,
            ));
        }
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

/// The exit door; its sprite's atlas index shows its state.
#[derive(Component)]
pub(crate) struct ExitDoor;

impl ExitDoor {
    pub(crate) const SHEET: &str = "exit_door";
    pub(crate) const UNLOCKED_FRAME: usize = 1;
    pub(crate) const FIRST_OPENING_FRAME: usize = 2;
}

/// The door the player steps out of when a room opens, drawn from the same
/// sheet as the exit door and gone once they are standing in the room.
#[derive(Component)]
pub(crate) struct EntryDoor;

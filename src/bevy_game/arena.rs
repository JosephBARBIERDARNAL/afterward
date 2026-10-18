use bevy::prelude::*;

use super::config::{ROOM_COUNT, VIEW_SIZE};
use super::model::Scene;
use super::model::*;
use super::ui::{spawn_door, spawn_enemy_health_bar, spawn_player_health_display, spawn_text};
use crate::presentation::visual::{Art, spawn_player_weapon};

#[derive(Resource, Default)]
pub(crate) struct ArenaGeometry {
    pub(crate) obstacles: Vec<ArenaObstacle>,
    pub(crate) cache: Option<Vec2>,
}

#[derive(Clone, Copy)]
pub(crate) struct ArenaObstacle {
    pub(crate) center: Vec2,
    pub(crate) size: Vec2,
}

pub(crate) struct RoomLayout {
    pub(crate) title: &'static str,
    pub(crate) player_start: Vec2,
    pub(crate) enemies: Vec<EnemySpawn>,
    pub(crate) obstacles: Vec<ArenaObstacle>,
    pub(crate) cache: Option<Vec2>,
}

pub(crate) fn sync_scene(
    mut commands: Commands,
    art: Res<Art>,
    mut game: ResMut<Game>,
    mut geometry: ResMut<ArenaGeometry>,
    old: Query<Entity, With<SceneEntity>>,
) {
    let key = (game.scene, game.room);
    if game.last_scene_key == Some(key) {
        return;
    }
    for entity in &old {
        commands.entity(entity).despawn();
    }
    match game.scene {
        Scene::Home => {
            geometry.obstacles.clear();
            geometry.cache = None;
            spawn_home(&mut commands, &art);
        }
        Scene::Run => {
            let layout = room_layout(game.room);
            geometry.obstacles.clone_from(&layout.obstacles);
            geometry.cache = layout.cache;
            game.cache_claimed = false;
            spawn_room(&mut commands, &art, game.room, &game.bonuses, layout);
        }
    }
    game.last_scene_key = Some(key);
}

pub(crate) fn spawn_home(commands: &mut Commands, art: &Art) {
    spawn_arena(commands, true, &[]);
    spawn_disc(
        commands,
        art,
        &art.player,
        Vec2::new(-280.0, -20.0),
        16.0,
        2.0,
        Player,
    );
    spawn_player_weapon(commands);
    spawn_door(commands, Vec2::new(380.0, 0.0));
    spawn_text(
        commands,
        "THE ASHEN REST",
        Vec2::new(0.0, 205.0),
        28.0,
        Color::srgb(0.89, 0.78, 0.56),
    );
    spawn_text(
        commands,
        "SAFE ROOM  ·  NO HOSTILES",
        Vec2::new(0.0, 174.0),
        16.0,
        Color::srgb(0.73, 0.70, 0.65),
    );
    spawn_text(
        commands,
        "MOVE  ARROWS / ZQSD     E  BEGIN RUN",
        Vec2::new(0.0, -220.0),
        15.0,
        Color::srgb(0.73, 0.70, 0.65),
    );
}

pub(crate) fn spawn_room(
    commands: &mut Commands,
    art: &Art,
    room: usize,
    bonuses: &[Bonus],
    layout: RoomLayout,
) {
    spawn_arena(commands, false, &layout.obstacles);
    spawn_disc(
        commands,
        art,
        &art.player,
        layout.player_start,
        16.0,
        2.0,
        Player,
    );
    spawn_player_weapon(commands);
    spawn_player_health_display(commands);
    if let Some(position) = layout.cache {
        spawn_bonus_cache(commands, position);
    }
    for (index, spawn) in layout.enemies.into_iter().enumerate() {
        let EnemySpawn {
            kind,
            position,
            elite,
        } = spawn;
        let color = match kind {
            EnemyKind::Melee => &art.melee,
            EnemyKind::Ranged => &art.ranged,
            EnemyKind::Heavy => &art.heavy,
            EnemyKind::Charger => &art.charger,
            EnemyKind::Skirmisher => &art.skirmisher,
            EnemyKind::Bomber => &art.bomber,
        };
        if elite.is_some() {
            commands.spawn((
                Mesh2d(art.circle.clone()),
                MeshMaterial2d(art.elite_ring.clone()),
                Transform::from_xyz(position.x, position.y, 1.7).with_scale(Vec3::splat(21.0)),
                SceneEntity,
            ));
        }
        let max_health = kind.health() * if elite.is_some() { 1.45 } else { 1.0 };
        let enemy_entity = commands
            .spawn((
                Mesh2d(art.circle.clone()),
                MeshMaterial2d(color.clone()),
                Transform::from_xyz(position.x, position.y, 2.0)
                    .with_scale(Vec3::splat(if elite.is_some() { 17.0 } else { 15.0 })),
                SceneEntity,
                Enemy {
                    kind,
                    elite,
                    health: max_health,
                    cooldown: 0.6,
                    windup: 0.0,
                    target: Vec2::ZERO,
                    charge_time: 0.0,
                    charge_direction: Vec2::ZERO,
                    stun: 0.0,
                    orbit_side: if index % 2 == 0 { 1.0 } else { -1.0 },
                    telegraph: None,
                },
            ))
            .id();
        spawn_enemy_health_bar(
            commands,
            enemy_entity,
            position,
            max_health,
            elite.is_some(),
        );
    }
    spawn_door(commands, Vec2::new(420.0, 0.0));
    spawn_text(
        commands,
        &format!("ROOM {} / {}  ·  {}", room + 1, ROOM_COUNT, layout.title),
        Vec2::new(0.0, 232.0),
        18.0,
        Color::srgb(0.89, 0.78, 0.56),
    );
    if !bonuses.is_empty() {
        let picked = bonuses
            .iter()
            .map(|bonus| bonus.title())
            .collect::<Vec<_>>()
            .join("\n");
        spawn_text(
            commands,
            &picked,
            Vec2::new(-365.0, 150.0),
            11.0,
            Color::srgb(0.82, 0.71, 0.52),
        );
    }
    spawn_text(
        commands,
        "J  ATTACK     K  HEAVY     SPACE  DASH     E  DOOR     ESC  PAUSE",
        Vec2::new(0.0, -232.0),
        13.0,
        Color::srgb(0.73, 0.70, 0.65),
    );
}

pub(crate) fn spawn_arena(commands: &mut Commands, home: bool, obstacles: &[ArenaObstacle]) {
    commands.spawn((
        Sprite::from_color(Color::srgb(0.10, 0.10, 0.11), VIEW_SIZE),
        Transform::from_xyz(0.0, 0.0, -3.0),
        SceneEntity,
    ));
    for row in 0..8 {
        for col in 0..14 {
            let shade = if (row + col) % 2 == 0 { 0.16 } else { 0.17 };
            let color = Color::srgb(shade, shade, shade + 0.01);
            commands.spawn((
                Sprite::from_color(color, Vec2::splat(62.0)),
                Transform::from_xyz(-403.0 + col as f32 * 62.0, -217.0 + row as f32 * 62.0, -2.0),
                SceneEntity,
            ));
        }
    }
    for obstacle in obstacles {
        let shadow_position = obstacle.center + Vec2::new(5.0, -8.0);
        commands.spawn((
            Sprite::from_color(
                Color::srgba(0.035, 0.035, 0.04, 0.75),
                obstacle.size + Vec2::new(10.0, 10.0),
            ),
            Transform::from_xyz(shadow_position.x, shadow_position.y, -1.5),
            SceneEntity,
        ));
        commands.spawn((
            Sprite::from_color(
                Color::srgb(0.51, 0.45, 0.36),
                obstacle.size + Vec2::splat(4.0),
            ),
            Transform::from_xyz(obstacle.center.x, obstacle.center.y, 0.1),
            SceneEntity,
        ));
        commands.spawn((
            Sprite::from_color(Color::srgb(0.33, 0.30, 0.27), obstacle.size),
            Transform::from_xyz(obstacle.center.x, obstacle.center.y, 0.2),
            SceneEntity,
        ));
        commands.spawn((
            Sprite::from_color(
                Color::srgb(0.61, 0.53, 0.40),
                Vec2::new((obstacle.size.x - 8.0).max(4.0), 4.0),
            ),
            Transform::from_xyz(
                obstacle.center.x,
                obstacle.center.y + obstacle.size.y * 0.5 - 5.0,
                0.3,
            ),
            SceneEntity,
        ));
    }
    let label = if home { "AFTERWARD" } else { "THE ASHEN ARENA" };
    spawn_text(
        commands,
        label,
        Vec2::new(-405.0, 252.0),
        15.0,
        Color::srgb(0.90, 0.86, 0.79),
    );
    spawn_text(
        commands,
        "ONE ARENA. NO SECOND CHANCES.",
        Vec2::new(-300.0, -254.0),
        12.0,
        Color::srgb(0.47, 0.45, 0.42),
    );
}

pub(crate) fn spawn_bonus_cache(commands: &mut Commands, position: Vec2) {
    for (offset, size, color, z) in [
        (
            Vec2::new(3.0, -5.0),
            Vec2::new(40.0, 25.0),
            Color::srgb(0.055, 0.05, 0.045),
            0.5,
        ),
        (
            Vec2::ZERO,
            Vec2::new(34.0, 21.0),
            Color::srgb(0.29, 0.25, 0.19),
            0.6,
        ),
        (
            Vec2::new(0.0, 3.0),
            Vec2::new(34.0, 5.0),
            Color::srgb(0.72, 0.59, 0.34),
            0.7,
        ),
    ] {
        commands.spawn((
            Sprite::from_color(color, size),
            Transform::from_xyz(position.x + offset.x, position.y + offset.y, z),
            SceneEntity,
            BonusCache,
        ));
    }
}

pub(crate) fn arena_position(x: f32, y: f32) -> Vec2 {
    Vec2::new(x - 480.0, 270.0 - y)
}

pub(crate) fn arena_obstacle(x: f32, y: f32, width: f32, height: f32) -> ArenaObstacle {
    ArenaObstacle {
        center: arena_position(x, y),
        size: Vec2::new(width, height),
    }
}

pub(crate) fn room_layout(room: usize) -> RoomLayout {
    match room {
        0 => RoomLayout {
            title: "THE ENTRY",
            player_start: arena_position(480.0, 292.0),
            enemies: vec![
                enemy_spawn(EnemyKind::Melee, -235.0, 80.0),
                enemy_spawn(EnemyKind::Ranged, 245.0, -65.0),
            ],
            obstacles: vec![
                arena_obstacle(342.0, 226.0, 94.0, 54.0),
                arena_obstacle(625.0, 343.0, 106.0, 52.0),
                arena_obstacle(748.0, 164.0, 44.0, 44.0),
            ],
            cache: None,
        },
        1 => RoomLayout {
            title: "THE OUTER WALK",
            player_start: arena_position(82.0, 270.0),
            enemies: vec![
                enemy_spawn(EnemyKind::Melee, 14.0, 96.0),
                enemy_spawn(EnemyKind::Melee, 167.0, -113.0),
                enemy_spawn(EnemyKind::Ranged, 277.0, -2.0),
                enemy_spawn(EnemyKind::Charger, 230.0, -146.0),
            ],
            obstacles: vec![
                arena_obstacle(302.0, 215.0, 74.0, 104.0),
                arena_obstacle(510.0, 150.0, 124.0, 44.0),
                arena_obstacle(638.0, 378.0, 86.0, 48.0),
                arena_obstacle(789.0, 243.0, 44.0, 44.0),
            ],
            cache: None,
        },
        2 => RoomLayout {
            title: "THE CROSSING",
            player_start: arena_position(82.0, 270.0),
            enemies: vec![
                enemy_spawn(EnemyKind::Heavy, 90.0, 35.0),
                enemy_spawn(EnemyKind::Bomber, 24.0, -110.0),
                enemy_spawn(EnemyKind::Ranged, 252.0, -106.0),
            ],
            obstacles: vec![
                arena_obstacle(322.0, 163.0, 122.0, 46.0),
                arena_obstacle(486.0, 270.0, 80.0, 94.0),
                arena_obstacle(323.0, 377.0, 122.0, 46.0),
            ],
            cache: Some(arena_position(444.0, 454.0)),
        },
        3 => RoomLayout {
            title: "THE LOWER GALLERY",
            player_start: arena_position(82.0, 270.0),
            enemies: vec![
                enemy_spawn(EnemyKind::Melee, -36.0, 110.0),
                enemy_spawn(EnemyKind::Heavy, 146.0, -10.0),
                enemy_spawn(EnemyKind::Skirmisher, 22.0, -127.0),
                enemy_spawn(EnemyKind::Ranged, 282.0, 95.0),
            ],
            obstacles: vec![
                arena_obstacle(301.0, 160.0, 86.0, 56.0),
                arena_obstacle(421.0, 270.0, 74.0, 116.0),
                arena_obstacle(531.0, 155.0, 90.0, 48.0),
                arena_obstacle(546.0, 395.0, 122.0, 42.0),
                arena_obstacle(773.0, 365.0, 48.0, 48.0),
            ],
            cache: None,
        },
        _ => RoomLayout {
            title: "THE INNER SEAL",
            player_start: arena_position(82.0, 270.0),
            enemies: vec![
                enemy_spawn(EnemyKind::Melee, -87.0, 86.0),
                enemy_spawn(EnemyKind::Ranged, -105.0, -128.0),
                enemy_spawn(EnemyKind::Heavy, 166.0, -13.0),
                elite_spawn(EnemyKind::Charger, 248.0, 115.0, EliteKind::Vanguard),
                elite_spawn(EnemyKind::Ranged, 264.0, -130.0, EliteKind::Volley),
            ],
            obstacles: vec![
                arena_obstacle(280.0, 270.0, 70.0, 120.0),
                arena_obstacle(493.0, 143.0, 104.0, 46.0),
                arena_obstacle(500.0, 392.0, 118.0, 46.0),
                arena_obstacle(761.0, 270.0, 42.0, 120.0),
            ],
            cache: None,
        },
    }
}

pub(crate) fn spawn_disc<T: Component>(
    commands: &mut Commands,
    art: &Art,
    material: &Handle<ColorMaterial>,
    pos: Vec2,
    radius: f32,
    z: f32,
    marker: T,
) {
    commands.spawn((
        Mesh2d(art.circle.clone()),
        MeshMaterial2d(material.clone()),
        Transform::from_xyz(pos.x, pos.y, z).with_scale(Vec3::splat(radius)),
        SceneEntity,
        marker,
    ));
}

pub(crate) fn resolve_circle_obstacles(
    position: &mut Vec2,
    radius: f32,
    obstacles: &[ArenaObstacle],
) {
    for _ in 0..2 {
        for obstacle in obstacles {
            let half_size = obstacle.size * 0.5;
            let min = obstacle.center - half_size;
            let max = obstacle.center + half_size;
            let nearest = position.clamp(min, max);
            let away = *position - nearest;
            let distance_squared = away.length_squared();
            if distance_squared >= radius * radius {
                continue;
            }

            if distance_squared > 0.0001 {
                let distance = distance_squared.sqrt();
                *position += away / distance * (radius - distance);
            } else {
                let distances = [
                    (position.x - min.x, 0),
                    (max.x - position.x, 1),
                    (position.y - min.y, 2),
                    (max.y - position.y, 3),
                ];
                let side = distances
                    .into_iter()
                    .min_by(|left, right| left.0.total_cmp(&right.0))
                    .unwrap()
                    .1;
                match side {
                    0 => position.x = min.x - radius,
                    1 => position.x = max.x + radius,
                    2 => position.y = min.y - radius,
                    _ => position.y = max.y + radius,
                }
            }
        }
    }
}

pub(crate) fn point_hits_obstacle(point: Vec2, radius: f32, obstacle: ArenaObstacle) -> bool {
    let half_size = obstacle.size * 0.5 + Vec2::splat(radius);
    let delta = (point - obstacle.center).abs();
    delta.x < half_size.x && delta.y < half_size.y
}

pub(crate) fn line_clear(start: Vec2, end: Vec2, obstacles: &[ArenaObstacle]) -> bool {
    let distance = start.distance(end);
    let steps = (distance / 8.0).ceil() as usize;
    for step in 1..steps {
        let point = start.lerp(end, step as f32 / steps as f32);
        if obstacles
            .iter()
            .copied()
            .any(|obstacle| point_hits_obstacle(point, 0.0, obstacle))
        {
            return false;
        }
    }
    true
}

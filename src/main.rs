use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

const VIEW_SIZE: Vec2 = Vec2::new(1280.0, 720.0);
const CAMERA_SCALE: f32 = 0.75;
const PLAYER_MOVE_SPEED: f32 = 340.0;
const HEAVY_MOVE_SPEED: f32 = 155.0;
const DASH_MOVE_SPEED: f32 = 610.0;
const DAMAGE_NUMBER_LIFETIME: f32 = 0.65;
const PLAYER_HEALTH_BAR_WIDTH: f32 = 144.0;
const PLAYER_HEALTH_BAR_HEIGHT: f32 = 9.0;
const ROOM_COUNT: usize = 5;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "TOO LATE".into(),
                resolution: (VIEW_SIZE.x as u32, VIEW_SIZE.y as u32).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .init_resource::<Game>()
        .add_systems(Startup, (setup_camera, setup_art))
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
        )
        .run();
}

#[derive(Resource)]
struct Art {
    circle: Handle<Mesh>,
    player: Handle<ColorMaterial>,
    melee: Handle<ColorMaterial>,
    ranged: Handle<ColorMaterial>,
    heavy: Handle<ColorMaterial>,
    charger: Handle<ColorMaterial>,
    skirmisher: Handle<ColorMaterial>,
    bomber: Handle<ColorMaterial>,
    elite_ring: Handle<ColorMaterial>,
    warning: Handle<ColorMaterial>,
}

#[derive(Resource)]
struct Game {
    scene: Scene,
    room: usize,
    player_health: f32,
    max_health: f32,
    facing: Vec2,
    primary_cooldown: f32,
    heavy_windup: f32,
    heavy_recovery: f32,
    heavy_pending: bool,
    dash_time: f32,
    dash_cooldown: f32,
    invulnerable: f32,
    room_clear: bool,
    overlay: Overlay,
    bonuses: Vec<Bonus>,
    bonus_damage: f32,
    bonus_range: f32,
    combo_index: usize,
    combo_timer: f32,
    bonus_seed: u64,
    last_scene_key: Option<(Scene, usize)>,
    last_overlay_key: String,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            scene: Scene::Home,
            room: 0,
            player_health: 100.0,
            max_health: 100.0,
            facing: Vec2::X,
            primary_cooldown: 0.0,
            heavy_windup: 0.0,
            heavy_recovery: 0.0,
            heavy_pending: false,
            dash_time: 0.0,
            dash_cooldown: 0.0,
            invulnerable: 0.0,
            room_clear: false,
            overlay: Overlay::None,
            bonuses: Vec::new(),
            bonus_damage: 0.0,
            bonus_range: 0.0,
            combo_index: 0,
            combo_timer: 0.0,
            bonus_seed: 0,
            last_scene_key: None,
            last_overlay_key: String::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Scene {
    Home,
    Run,
}

#[derive(Clone, Debug)]
enum Overlay {
    None,
    Pause(usize),
    Catalog,
    BonusChoice(usize),
    Ended(bool),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Bonus {
    Vitality,
    Edge,
    Mend,
    DashNova,
    QuickStep,
    Reach,
    WideSwing,
    CinderCombo,
    ComboBlast,
    Siphon,
}

impl Bonus {
    const ALL: [Self; 10] = [
        Self::Vitality,
        Self::Edge,
        Self::Mend,
        Self::DashNova,
        Self::QuickStep,
        Self::Reach,
        Self::WideSwing,
        Self::CinderCombo,
        Self::ComboBlast,
        Self::Siphon,
    ];

    const fn title(self) -> &'static str {
        match self {
            Self::Vitality => "VITALITY  +20 MAX LIFE",
            Self::Edge => "SHARPENED  +3 BASE DAMAGE",
            Self::Mend => "MENDING  RESTORE 35 LIFE",
            Self::DashNova => "DASH NOVA  DASH ENDS IN A BURST",
            Self::QuickStep => "QUICK STEP  FASTER DASH RECHARGE",
            Self::Reach => "LONG REACH  WIDER ATTACKS",
            Self::WideSwing => "WIDE SWING  HEAVY ATTACK COVERS MORE",
            Self::CinderCombo => "CINDER COMBO  FINAL HIT BURNS",
            Self::ComboBlast => "COMBO BLAST  FINAL HIT BURSTS",
            Self::Siphon => "SIPHON  ATTACKS RESTORE LIFE",
        }
    }
}

#[derive(Component)]
struct SceneEntity;

#[derive(Component)]
struct OverlayEntity;

#[derive(Component)]
struct DamageNumber {
    age: f32,
    received: bool,
}

#[derive(Component)]
struct PlayerWeapon {
    offset: f32,
}

#[derive(Component)]
struct ActionEffect {
    age: f32,
    lifetime: f32,
    start_scale: f32,
    end_scale: f32,
    rise_speed: f32,
    rgba: [f32; 4],
}

#[derive(Component)]
struct AttackTelegraph {
    age: f32,
    lifetime: f32,
    radius: f32,
}

#[derive(Component)]
struct PlayerHealthFill;

#[derive(Component)]
struct PlayerHealthReadout;

#[derive(Component)]
struct EnemyHealthBar {
    target: Entity,
    max_health: f32,
    width: f32,
    y_offset: f32,
    fill: bool,
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy {
    kind: EnemyKind,
    elite: Option<EliteKind>,
    health: f32,
    cooldown: f32,
    windup: f32,
    target: Vec2,
    charge_time: f32,
    charge_direction: Vec2,
    stun: f32,
    orbit_side: f32,
    telegraph: Option<Entity>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum EnemyKind {
    Melee,
    Ranged,
    Heavy,
    Charger,
    Skirmisher,
    Bomber,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum EliteKind {
    Vanguard,
    Volley,
}

struct EnemySpawn {
    kind: EnemyKind,
    position: Vec2,
    elite: Option<EliteKind>,
}

fn enemy_spawn(kind: EnemyKind, x: f32, y: f32) -> EnemySpawn {
    EnemySpawn {
        kind,
        position: Vec2::new(x, y),
        elite: None,
    }
}

fn elite_spawn(kind: EnemyKind, x: f32, y: f32, elite: EliteKind) -> EnemySpawn {
    EnemySpawn {
        kind,
        position: Vec2::new(x, y),
        elite: Some(elite),
    }
}

impl EnemyKind {
    const fn health(self) -> f32 {
        match self {
            Self::Melee => 74.0,
            Self::Ranged => 60.0,
            Self::Heavy => 96.0,
            Self::Charger => 82.0,
            Self::Skirmisher => 58.0,
            Self::Bomber => 76.0,
        }
    }

    const fn speed(self) -> f32 {
        match self {
            Self::Melee => 91.0,
            Self::Ranged => 78.0,
            Self::Heavy => 58.0,
            Self::Charger => 78.0,
            Self::Skirmisher => 132.0,
            Self::Bomber => 54.0,
        }
    }

    const fn damage(self) -> f32 {
        match self {
            Self::Melee => 17.0,
            Self::Ranged => 13.0,
            Self::Heavy => 23.0,
            Self::Charger => 21.0,
            Self::Skirmisher => 13.0,
            Self::Bomber => 20.0,
        }
    }

    const fn attack_range(self) -> f32 {
        match self {
            Self::Melee => 82.0,
            Self::Ranged => 218.0,
            Self::Heavy => 102.0,
            Self::Charger => 270.0,
            Self::Skirmisher => 84.0,
            Self::Bomber => 174.0,
        }
    }

    const fn attack_radius(self) -> f32 {
        match self {
            Self::Melee => 48.0,
            Self::Ranged => 12.0,
            Self::Heavy => 59.0,
            Self::Charger => 31.0,
            Self::Skirmisher => 32.0,
            Self::Bomber => 68.0,
        }
    }

    const fn attack_windup(self) -> f32 {
        match self {
            Self::Melee => 0.58,
            Self::Ranged => 0.64,
            Self::Heavy => 0.82,
            Self::Charger => 0.72,
            Self::Skirmisher => 0.34,
            Self::Bomber => 0.90,
        }
    }

    const fn attack_cooldown(self) -> f32 {
        match self {
            Self::Melee => 1.12,
            Self::Ranged => 1.75,
            Self::Heavy => 1.48,
            Self::Charger => 1.75,
            Self::Skirmisher => 1.10,
            Self::Bomber => 2.05,
        }
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scale: CAMERA_SCALE,
            ..OrthographicProjection::default_2d()
        }),
    ));
}

fn setup_art(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(Art {
        circle: meshes.add(Circle::new(1.0)),
        player: materials.add(Color::srgb(0.91, 0.84, 0.66)),
        melee: materials.add(Color::srgb(0.77, 0.42, 0.31)),
        ranged: materials.add(Color::srgb(0.60, 0.51, 0.73)),
        heavy: materials.add(Color::srgb(0.66, 0.53, 0.30)),
        charger: materials.add(Color::srgb(0.76, 0.29, 0.26)),
        skirmisher: materials.add(Color::srgb(0.38, 0.67, 0.63)),
        bomber: materials.add(Color::srgb(0.77, 0.49, 0.27)),
        elite_ring: materials.add(Color::srgba(0.90, 0.72, 0.40, 0.72)),
        warning: materials.add(Color::srgba(0.96, 0.16, 0.12, 0.52)),
    });
}

fn sync_scene(
    mut commands: Commands,
    art: Res<Art>,
    mut game: ResMut<Game>,
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
        Scene::Home => spawn_home(&mut commands, &art),
        Scene::Run => spawn_room(&mut commands, &art, game.room, &game.bonuses),
    }
    game.last_scene_key = Some(key);
}

fn spawn_home(commands: &mut Commands, art: &Art) {
    spawn_arena(commands, true);
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

#[derive(Component)]
struct Door;

fn spawn_room(commands: &mut Commands, art: &Art, room: usize, bonuses: &[Bonus]) {
    spawn_arena(commands, false);
    let (name, player_start, enemies) = room_layout(room);
    spawn_disc(commands, art, &art.player, player_start, 16.0, 2.0, Player);
    spawn_player_weapon(commands);
    spawn_player_health_display(commands);
    for (index, spawn) in enemies.into_iter().enumerate() {
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
        &format!("ROOM {} / {}  ·  {name}", room + 1, ROOM_COUNT),
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

fn spawn_player_health_display(commands: &mut Commands) {
    let left = -PLAYER_HEALTH_BAR_WIDTH / 2.0 - 325.0;
    let center = left + PLAYER_HEALTH_BAR_WIDTH / 2.0;
    let bar_position = Vec3::new(center, 203.0, 5.0);
    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.11, 0.09, 0.08),
            Vec2::new(PLAYER_HEALTH_BAR_WIDTH, PLAYER_HEALTH_BAR_HEIGHT),
        ),
        Transform::from_translation(bar_position),
        SceneEntity,
    ));
    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.62, 0.76, 0.43),
            Vec2::new(PLAYER_HEALTH_BAR_WIDTH, PLAYER_HEALTH_BAR_HEIGHT),
        ),
        Transform::from_xyz(center, bar_position.y, 5.1),
        SceneEntity,
        PlayerHealthFill,
    ));
    commands.spawn((
        Text2d::new("LIFE 100 / 100"),
        TextFont {
            font_size: FontSize::Px(14.0),
            font_smoothing: FontSmoothing::None,
            ..default()
        },
        TextColor(Color::srgb(0.89, 0.84, 0.74)),
        Transform::from_xyz(center, 220.0, 5.2),
        SceneEntity,
        PlayerHealthReadout,
    ));
}

fn spawn_enemy_health_bar(
    commands: &mut Commands,
    target: Entity,
    position: Vec2,
    max_health: f32,
    elite: bool,
) {
    let width = if elite { 60.0 } else { 50.0 };
    let height = 8.0;
    let y = position.y + if elite { 31.0 } else { 28.0 };
    let fill_color = if elite {
        Color::srgb(0.97, 0.58, 0.25)
    } else {
        Color::srgb(0.91, 0.28, 0.21)
    };
    commands.spawn((
        Sprite::from_color(Color::srgb(0.10, 0.07, 0.06), Vec2::new(width, height)),
        Transform::from_xyz(position.x, y, 3.8),
        SceneEntity,
        EnemyHealthBar {
            target,
            max_health,
            width,
            y_offset: if elite { 31.0 } else { 28.0 },
            fill: false,
        },
    ));
    commands.spawn((
        Sprite::from_color(fill_color, Vec2::new(width, height)),
        Transform::from_xyz(position.x, y, 3.9),
        SceneEntity,
        EnemyHealthBar {
            target,
            max_health,
            width,
            y_offset: if elite { 31.0 } else { 28.0 },
            fill: true,
        },
    ));
}

fn spawn_arena(commands: &mut Commands, home: bool) {
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
    let label = if home { "TOO LATE" } else { "THE ASHEN ARENA" };
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

fn room_layout(room: usize) -> (&'static str, Vec2, Vec<EnemySpawn>) {
    match room {
        0 => (
            "THE ENTRY",
            Vec2::new(-390.0, -22.0),
            vec![
                enemy_spawn(EnemyKind::Melee, -230.0, 80.0),
                enemy_spawn(EnemyKind::Ranged, 245.0, -65.0),
            ],
        ),
        1 => (
            "THE OUTER WALK",
            Vec2::new(-390.0, 0.0),
            vec![
                enemy_spawn(EnemyKind::Melee, 10.0, 95.0),
                enemy_spawn(EnemyKind::Melee, 170.0, -110.0),
                enemy_spawn(EnemyKind::Ranged, 300.0, 0.0),
                enemy_spawn(EnemyKind::Charger, 245.0, -150.0),
            ],
        ),
        2 => (
            "THE CROSSING",
            Vec2::new(-390.0, 0.0),
            vec![
                enemy_spawn(EnemyKind::Heavy, 95.0, 40.0),
                enemy_spawn(EnemyKind::Bomber, 30.0, -110.0),
                enemy_spawn(EnemyKind::Ranged, 260.0, -105.0),
            ],
        ),
        3 => (
            "THE LOWER GALLERY",
            Vec2::new(-390.0, 0.0),
            vec![
                enemy_spawn(EnemyKind::Melee, -20.0, 100.0),
                enemy_spawn(EnemyKind::Heavy, 155.0, 0.0),
                enemy_spawn(EnemyKind::Skirmisher, 35.0, -120.0),
                enemy_spawn(EnemyKind::Ranged, 300.0, 100.0),
            ],
        ),
        _ => (
            "THE INNER SEAL",
            Vec2::new(-390.0, 0.0),
            vec![
                enemy_spawn(EnemyKind::Melee, -40.0, 100.0),
                enemy_spawn(EnemyKind::Ranged, -55.0, -120.0),
                enemy_spawn(EnemyKind::Heavy, 185.0, 0.0),
                elite_spawn(EnemyKind::Charger, 300.0, 115.0, EliteKind::Vanguard),
                elite_spawn(EnemyKind::Ranged, 305.0, -125.0, EliteKind::Volley),
            ],
        ),
    }
}

fn spawn_disc<T: Component>(
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

fn spawn_player_weapon(commands: &mut Commands) {
    for (offset, size, color) in [
        (31.0, Vec2::new(30.0, 5.0), Color::srgb(0.98, 0.90, 0.69)),
        (17.0, Vec2::new(6.0, 11.0), Color::srgb(0.70, 0.48, 0.25)),
    ] {
        commands.spawn((
            Sprite::from_color(color, size),
            Transform::from_xyz(0.0, 0.0, 3.5),
            SceneEntity,
            PlayerWeapon { offset },
        ));
    }
}

fn update_player_weapon(weapon: &mut WeaponQuery<'_, '_>, player: Vec2, facing: Vec2) {
    let facing = facing.normalize_or_zero();
    let facing = if facing == Vec2::ZERO {
        Vec2::X
    } else {
        facing
    };
    let rotation = Quat::from_rotation_z(facing.to_angle());
    for (mut transform, weapon_part) in weapon.iter_mut() {
        transform.translation = (player + facing * weapon_part.offset).extend(3.5);
        transform.rotation = rotation;
    }
}

struct ActionVisual {
    position: Vec2,
    angle: f32,
    size: Vec2,
    rgba: [f32; 4],
    lifetime: f32,
    start_scale: f32,
    end_scale: f32,
    rise_speed: f32,
    z: f32,
}

fn spawn_action_effect(commands: &mut Commands, visual: ActionVisual) {
    commands.spawn((
        Sprite::from_color(
            Color::srgba(
                visual.rgba[0],
                visual.rgba[1],
                visual.rgba[2],
                visual.rgba[3],
            ),
            visual.size,
        ),
        Transform::from_xyz(visual.position.x, visual.position.y, visual.z)
            .with_rotation(Quat::from_rotation_z(visual.angle))
            .with_scale(Vec3::splat(visual.start_scale)),
        SceneEntity,
        ActionEffect {
            age: 0.0,
            lifetime: visual.lifetime,
            start_scale: visual.start_scale,
            end_scale: visual.end_scale,
            rise_speed: visual.rise_speed,
            rgba: visual.rgba,
        },
    ));
}

fn animate_action_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut effects: Query<(Entity, &mut Transform, &mut Sprite, &mut ActionEffect)>,
) {
    let dt = time.delta_secs().min(0.05);
    for (entity, mut transform, mut sprite, mut effect) in &mut effects {
        effect.age += dt;
        if effect.age >= effect.lifetime {
            commands.entity(entity).despawn();
            continue;
        }

        let progress = (effect.age / effect.lifetime).clamp(0.0, 1.0);
        let scale = effect.start_scale + (effect.end_scale - effect.start_scale) * progress;
        transform.scale = Vec3::splat(scale);
        transform.translation.y += effect.rise_speed * dt;
        sprite.color = Color::srgba(
            effect.rgba[0],
            effect.rgba[1],
            effect.rgba[2],
            effect.rgba[3] * (1.0 - progress),
        );
    }
}

fn animate_attack_telegraphs(
    time: Res<Time>,
    mut warnings: Query<(&mut Transform, &mut AttackTelegraph)>,
) {
    let dt = time.delta_secs().min(0.05);
    for (mut transform, mut warning) in &mut warnings {
        warning.age += dt;
        let progress = (warning.age / warning.lifetime).clamp(0.0, 1.0);
        let pulse = if progress > 0.75 {
            1.0 + (progress * 32.0).sin().abs() * 0.12
        } else {
            1.0
        };
        let scale = warning.radius * (0.12 + 0.88 * progress) * pulse;
        transform.scale = Vec3::splat(scale);
    }
}

fn update_player_health_display(
    game: Res<Game>,
    mut fills: Query<(&mut Sprite, &mut Transform), With<PlayerHealthFill>>,
    mut readouts: Query<&mut Text2d, With<PlayerHealthReadout>>,
) {
    let fraction = (game.player_health / game.max_health).clamp(0.0, 1.0);
    let fill_width = PLAYER_HEALTH_BAR_WIDTH * fraction;
    let left = -325.0 - PLAYER_HEALTH_BAR_WIDTH / 2.0;
    let color = if fraction <= 0.25 {
        Color::srgb(0.91, 0.28, 0.21)
    } else if fraction <= 0.55 {
        Color::srgb(0.95, 0.64, 0.26)
    } else {
        Color::srgb(0.62, 0.76, 0.43)
    };
    for (mut sprite, mut transform) in &mut fills {
        sprite.custom_size = Some(Vec2::new(fill_width, PLAYER_HEALTH_BAR_HEIGHT));
        sprite.color = color;
        transform.translation.x = left + fill_width / 2.0;
    }

    let current = game.player_health.round() as i32;
    let maximum = game.max_health.round() as i32;
    let value = format!("LIFE {current} / {maximum}");
    for mut readout in &mut readouts {
        if readout.0 != value {
            readout.0.clone_from(&value);
        }
    }
}

fn sync_enemy_health_bars(
    mut commands: Commands,
    enemies: EnemyHealthTargetQuery,
    mut bars: EnemyHealthBarQuery,
) {
    for (bar_entity, mut transform, mut sprite, health_bar) in &mut bars {
        let Ok((_, enemy_transform, enemy)) = enemies.get(health_bar.target) else {
            commands.entity(bar_entity).despawn();
            continue;
        };
        if enemy.health <= 0.0 {
            commands.entity(bar_entity).despawn();
            continue;
        }

        let fraction = (enemy.health / health_bar.max_health).clamp(0.0, 1.0);
        let fill_width = health_bar.width * fraction;
        let width = if health_bar.fill {
            fill_width
        } else {
            health_bar.width
        };
        let center_x = if health_bar.fill {
            enemy_transform.translation.x - health_bar.width / 2.0 + fill_width / 2.0
        } else {
            enemy_transform.translation.x
        };
        sprite.custom_size = Some(Vec2::new(width, 8.0));
        transform.translation.x = center_x;
        transform.translation.y = enemy_transform.translation.y + health_bar.y_offset;
    }
}

fn spawn_door(commands: &mut Commands, position: Vec2) {
    commands.spawn((
        Sprite::from_color(Color::srgb(0.47, 0.38, 0.25), Vec2::new(48.0, 112.0)),
        Transform::from_xyz(position.x, position.y, 1.0),
        SceneEntity,
        Door,
    ));
    spawn_text(
        commands,
        "E",
        position + Vec2::new(0.0, -74.0),
        14.0,
        Color::srgb(0.89, 0.78, 0.56),
    );
}

fn spawn_text(commands: &mut Commands, value: &str, pos: Vec2, size: f32, color: Color) {
    commands.spawn((
        Text2d::new(value),
        TextFont {
            font_size: FontSize::Px(size),
            font_smoothing: FontSmoothing::None,
            ..default()
        },
        TextColor(color),
        Transform::from_xyz(pos.x, pos.y, 4.0),
        SceneEntity,
    ));
}

fn sync_overlay(
    mut commands: Commands,
    mut game: ResMut<Game>,
    old: Query<Entity, With<OverlayEntity>>,
) {
    let key = format!("{:?}", game.overlay);
    if game.last_overlay_key == key {
        return;
    }
    for entity in &old {
        commands.entity(entity).despawn();
    }
    let text = match &game.overlay {
        Overlay::None => String::new(),
        Overlay::Pause(selected) => format!(
            "PAUSED\n\n{} CONTINUE\n{} RESTART RUN\n{} BONUS CATALOG\n{} BACK HOME\n\nARROWS + ENTER  ·  ESC CLOSES",
            mark(*selected, 0),
            mark(*selected, 1),
            mark(*selected, 2),
            mark(*selected, 3)
        ),
        Overlay::Catalog => format!(
            "RUN BONUS CATALOG\n\n{}\n\nPRESS ESC OR ENTER TO RETURN",
            Bonus::ALL.map(Bonus::title).join("\n")
        ),
        Overlay::BonusChoice(selected) => {
            let choices = bonus_choices(game.bonus_seed, game.room);
            format!(
                "CHOOSE ONE RUN BONUS\n\n{}\n{}\n{}\n\nARROWS + ENTER",
                mark_bonus(*selected, 0, choices[0]),
                mark_bonus(*selected, 1, choices[1]),
                mark_bonus(*selected, 2, choices[2])
            )
        }
        Overlay::Ended(victory) => {
            if *victory {
                "RUN COMPLETE\n\nROOMS CLEARED: 5\n\nENTER  RETURN HOME".to_owned()
            } else {
                "RUN ENDED\n\nYOUR TEMPORARY BUILD IS LOST\n\nENTER  RETURN HOME".to_owned()
            }
        }
    };
    if !text.is_empty() {
        commands.spawn((
            Text2d::new(text),
            TextFont {
                font_size: FontSize::Px(24.0),
                font_smoothing: FontSmoothing::None,
                ..default()
            },
            TextColor(Color::srgb(0.93, 0.85, 0.68)),
            TextLayout::justify(Justify::Center),
            Transform::from_xyz(0.0, 0.0, 10.0),
            OverlayEntity,
        ));
    }
    game.last_overlay_key = key;
}

fn mark(selected: usize, index: usize) -> &'static str {
    if selected == index { ">" } else { " " }
}

fn mark_bonus(selected: usize, index: usize, bonus: Bonus) -> String {
    let prefix = if selected == index { ">" } else { " " };
    format!("{prefix} {}", bonus.title())
}

fn bonus_choices(seed: u64, room: usize) -> [Bonus; 3] {
    let mut state = seed ^ (room as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    let mut indices = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    for index in (1..indices.len()).rev() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        indices.swap(index, state as usize % (index + 1));
    }
    [
        Bonus::ALL[indices[0]],
        Bonus::ALL[indices[1]],
        Bonus::ALL[indices[2]],
    ]
}

fn update_game(mut commands: Commands, params: GameParams) {
    let GameParams {
        keys,
        time,
        windows,
        cameras,
        mut game,
        art,
        mut player,
        mut weapon,
        mut enemies,
        mut projectiles,
        doors,
    } = params;
    let dt = time.delta_secs().min(0.05);
    if game.scene == Scene::Home {
        if let Ok((mut transform, _)) = player.single_mut() {
            move_player(
                &keys,
                &mut transform,
                &mut game.facing,
                PLAYER_MOVE_SPEED,
                dt,
            );
            update_player_weapon(&mut weapon, transform.translation.truncate(), game.facing);
            let near_door = doors.iter().any(|door| {
                door.translation
                    .truncate()
                    .distance(transform.translation.truncate())
                    < 82.0
            });
            if keys.just_pressed(KeyCode::KeyE) && near_door {
                start_run(&mut game, time.elapsed_secs_f64().to_bits());
            }
        }
        return;
    }

    if !matches!(game.overlay, Overlay::None) {
        update_overlay_input(&keys, &mut game, time.elapsed_secs_f64().to_bits());
        if matches!(game.overlay, Overlay::Ended(_)) && keys.just_pressed(KeyCode::Enter) {
            game.scene = Scene::Home;
            game.overlay = Overlay::None;
            game.room = 0;
            game.last_scene_key = None;
        }
        return;
    }
    if keys.just_pressed(KeyCode::Escape) {
        game.overlay = Overlay::Pause(0);
        game.last_overlay_key.clear();
        return;
    }

    let Ok((mut player_transform, _)) = player.single_mut() else {
        return;
    };
    game.primary_cooldown = (game.primary_cooldown - dt).max(0.0);
    game.heavy_recovery = (game.heavy_recovery - dt).max(0.0);
    game.heavy_windup = (game.heavy_windup - dt).max(0.0);
    game.combo_timer = (game.combo_timer - dt).max(0.0);
    game.dash_cooldown = (game.dash_cooldown - dt).max(0.0);
    game.dash_time = (game.dash_time - dt).max(0.0);
    game.invulnerable = (game.invulnerable - dt).max(0.0);

    if let (Ok(window), Ok((camera, camera_transform))) = (windows.single(), cameras.single())
        && let Some(cursor) = window.cursor_position()
        && let Ok(world) = camera.viewport_to_world_2d(camera_transform, cursor)
    {
        let delta = world - player_transform.translation.truncate();
        if delta.length_squared() > 4.0 {
            game.facing = delta.normalize();
        }
    }

    if keys.just_pressed(KeyCode::Space) && game.dash_cooldown <= 0.0 {
        spawn_action_effect(
            &mut commands,
            ActionVisual {
                position: player_transform.translation.truncate() - game.facing * 16.0,
                angle: game.facing.to_angle(),
                size: Vec2::new(50.0, 7.0),
                rgba: [0.96, 0.82, 0.54, 0.64],
                lifetime: 0.20,
                start_scale: 0.45,
                end_scale: 1.18,
                rise_speed: 0.0,
                z: 2.4,
            },
        );
        game.dash_time = 0.18;
        game.dash_cooldown = (0.78 - bonus_count(&game, Bonus::QuickStep) as f32 * 0.1).max(0.25);
        game.invulnerable = 0.24;
        if game.bonuses.contains(&Bonus::DashNova) {
            hit_enemies(
                &mut commands,
                &mut enemies,
                Attack {
                    origin: player_transform.translation.truncate(),
                    facing: game.facing,
                    range: 88.0,
                    damage: 18.0,
                    half_arc: -1.0,
                },
            );
        }
    }
    let speed = if game.dash_time > 0.0 {
        DASH_MOVE_SPEED
    } else if game.heavy_windup > 0.0 {
        HEAVY_MOVE_SPEED
    } else {
        PLAYER_MOVE_SPEED
    };
    move_player(&keys, &mut player_transform, &mut game.facing, speed, dt);
    let player_pos = player_transform.translation.truncate();
    update_player_weapon(&mut weapon, player_pos, game.facing);

    if keys.just_pressed(KeyCode::KeyJ)
        && game.primary_cooldown <= 0.0
        && game.heavy_recovery <= 0.0
    {
        game.primary_cooldown = 0.31;
        spawn_action_effect(
            &mut commands,
            ActionVisual {
                position: player_pos + game.facing * 38.0,
                angle: game.facing.to_angle() + std::f32::consts::FRAC_PI_2,
                size: Vec2::new(58.0, 9.0),
                rgba: [1.0, 0.93, 0.72, 0.96],
                lifetime: 0.14,
                start_scale: 0.38,
                end_scale: 1.12,
                rise_speed: 0.0,
                z: 4.4,
            },
        );
        if game.combo_timer <= 0.0 {
            game.combo_index = 0;
        }
        let final_hit = game.combo_index == 2;
        let combo_damage = [13.0, 13.0, 20.0][game.combo_index]
            + game.bonus_damage
            + if final_hit && game.bonuses.contains(&Bonus::CinderCombo) {
                6.0
            } else {
                0.0
            };
        let hit_count = hit_enemies(
            &mut commands,
            &mut enemies,
            Attack {
                origin: player_pos,
                facing: game.facing,
                range: 74.0 + game.bonus_range,
                damage: combo_damage,
                half_arc: 0.72,
            },
        );
        if final_hit && game.bonuses.contains(&Bonus::ComboBlast) {
            hit_enemies(
                &mut commands,
                &mut enemies,
                Attack {
                    origin: player_pos,
                    facing: game.facing,
                    range: 78.0,
                    damage: 14.0,
                    half_arc: -1.0,
                },
            );
        }
        game.player_health = (game.player_health
            + hit_count as f32 * bonus_count(&game, Bonus::Siphon) as f32 * 2.0)
            .min(game.max_health);
        game.combo_index = (game.combo_index + 1) % 3;
        game.combo_timer = 0.9;
    }
    if keys.just_pressed(KeyCode::KeyK) && game.heavy_recovery <= 0.0 && game.heavy_windup <= 0.0 {
        game.heavy_windup = 0.43;
        game.heavy_pending = true;
        spawn_action_effect(
            &mut commands,
            ActionVisual {
                position: player_pos,
                angle: 0.0,
                size: Vec2::splat(54.0),
                rgba: [0.94, 0.55, 0.22, 0.58],
                lifetime: game.heavy_windup,
                start_scale: 0.28,
                end_scale: 0.95,
                rise_speed: 0.0,
                z: 2.7,
            },
        );
    }
    if game.heavy_pending && game.heavy_windup <= 0.0 && game.heavy_recovery <= 0.0 {
        game.heavy_recovery = 0.56;
        game.heavy_pending = false;
        spawn_action_effect(
            &mut commands,
            ActionVisual {
                position: player_pos + game.facing * 48.0,
                angle: game.facing.to_angle() + std::f32::consts::FRAC_PI_2,
                size: Vec2::new(80.0, 14.0),
                rgba: [1.0, 0.67, 0.31, 0.96],
                lifetime: 0.20,
                start_scale: 0.45,
                end_scale: 1.12,
                rise_speed: 0.0,
                z: 4.5,
            },
        );
        let wide = game.bonuses.contains(&Bonus::WideSwing);
        let hit_count = hit_enemies(
            &mut commands,
            &mut enemies,
            Attack {
                origin: player_pos,
                facing: game.facing,
                range: 100.0 + game.bonus_range,
                damage: 44.0 + game.bonus_damage,
                half_arc: if wide { -0.1 } else { 0.52 },
            },
        );
        game.player_health = (game.player_health
            + hit_count as f32 * bonus_count(&game, Bonus::Siphon) as f32 * 2.0)
            .min(game.max_health);
    }

    update_enemies(&mut commands, &art, &mut game, &mut enemies, player_pos, dt);
    update_projectiles(&mut commands, &mut game, &mut projectiles, player_pos, dt);
    let near_door = doors
        .iter()
        .any(|door| door.translation.truncate().distance(player_pos) < 82.0);
    if keys.just_pressed(KeyCode::KeyE) && near_door && game.room_clear {
        if game.room + 1 >= ROOM_COUNT {
            game.overlay = Overlay::Ended(true);
        } else {
            game.overlay = Overlay::BonusChoice(0);
        }
        game.last_overlay_key.clear();
    }
}

#[derive(Component)]
struct Projectile {
    velocity: Vec2,
    lifetime: f32,
    damage: f32,
}

fn move_player(
    keys: &ButtonInput<KeyCode>,
    transform: &mut Transform,
    facing: &mut Vec2,
    speed: f32,
    dt: f32,
) {
    let mut direction = Vec2::ZERO;
    // KeyCode is physical: on AZERTY, the physical W/A positions are labeled Z/Q.
    if keys.pressed(KeyCode::ArrowUp) || keys.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if keys.pressed(KeyCode::ArrowDown) || keys.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }
    if keys.pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }
    if direction != Vec2::ZERO {
        direction = direction.normalize();
        *facing = direction;
        transform.translation += (direction * speed * dt).extend(0.0);
    }
    transform.translation.x = transform.translation.x.clamp(-430.0, 430.0);
    transform.translation.y = transform.translation.y.clamp(-220.0, 220.0);
}

type PlayerQuery<'w, 's> =
    Query<'w, 's, (&'static mut Transform, &'static mut Player), (With<Player>, Without<Door>)>;
type EnemyQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static mut Transform, &'static mut Enemy),
    (With<Enemy>, Without<Player>, Without<Door>),
>;
type ProjectileQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static mut Transform, &'static mut Projectile),
    (
        With<Projectile>,
        Without<Player>,
        Without<Enemy>,
        Without<Door>,
    ),
>;
type WeaponQuery<'w, 's> = Query<
    'w,
    's,
    (&'static mut Transform, &'static PlayerWeapon),
    (
        With<PlayerWeapon>,
        Without<Player>,
        Without<Enemy>,
        Without<Projectile>,
        Without<Door>,
    ),
>;
type EnemyHealthTargetQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static Transform, &'static Enemy),
    (With<Enemy>, Without<EnemyHealthBar>),
>;
type EnemyHealthBarQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut Transform,
        &'static mut Sprite,
        &'static EnemyHealthBar,
    ),
    (With<EnemyHealthBar>, Without<Enemy>),
>;

#[derive(SystemParam)]
struct GameParams<'w, 's> {
    keys: Res<'w, ButtonInput<KeyCode>>,
    time: Res<'w, Time>,
    windows: Query<'w, 's, &'static Window>,
    cameras: Query<'w, 's, (&'static Camera, &'static GlobalTransform)>,
    game: ResMut<'w, Game>,
    art: Res<'w, Art>,
    player: PlayerQuery<'w, 's>,
    weapon: WeaponQuery<'w, 's>,
    enemies: EnemyQuery<'w, 's>,
    projectiles: ProjectileQuery<'w, 's>,
    doors: Query<'w, 's, &'static Transform, With<Door>>,
}

struct Attack {
    origin: Vec2,
    facing: Vec2,
    range: f32,
    damage: f32,
    half_arc: f32,
}

fn hit_enemies(commands: &mut Commands, enemies: &mut EnemyQuery<'_, '_>, attack: Attack) -> usize {
    let mut hits = 0;
    for (entity, transform, mut enemy) in enemies.iter_mut() {
        let offset = transform.translation.truncate() - attack.origin;
        let distance = offset.length();
        if distance <= attack.range
            && (distance < 1.0 || offset.normalize().dot(attack.facing) >= attack.half_arc)
        {
            enemy.health -= attack.damage;
            enemy.stun = enemy.stun.max(0.13);
            hits += 1;
            spawn_damage(
                commands,
                transform.translation.truncate(),
                attack.damage,
                false,
            );
            if enemy.health <= 0.0 {
                if let Some(telegraph) = enemy.telegraph.take() {
                    commands.entity(telegraph).despawn();
                }
                commands.entity(entity).despawn();
            }
        }
    }
    hits
}

fn update_enemies(
    commands: &mut Commands,
    art: &Art,
    game: &mut Game,
    enemies: &mut EnemyQuery<'_, '_>,
    player: Vec2,
    dt: f32,
) {
    let mut count = 0;
    for (entity, mut transform, mut enemy) in enemies.iter_mut() {
        if enemy.health <= 0.0 {
            continue;
        }
        count += 1;
        enemy.cooldown = (enemy.cooldown - dt).max(0.0);
        enemy.stun = (enemy.stun - dt).max(0.0);
        let position = transform.translation.truncate();
        let delta = player - position;
        let distance = delta.length().max(0.01);
        let toward = delta / distance;
        let vanguard = enemy.elite == Some(EliteKind::Vanguard);
        let speed_multiplier = if vanguard { 1.12 } else { 1.0 };

        if enemy.windup > 0.0 {
            enemy.windup = (enemy.windup - dt).max(0.0);
            if enemy.windup == 0.0 {
                resolve_enemy_attack(commands, art, game, &mut enemy, position, player);
            }
            continue;
        }

        if enemy.stun > 0.0 {
            if enemy.charge_time > 0.0 {
                enemy.charge_time = 0.0;
            }
            continue;
        }

        if enemy.charge_time > 0.0 {
            enemy.charge_time = (enemy.charge_time - dt).max(0.0);
            transform.translation +=
                (enemy.charge_direction * 375.0 * speed_multiplier * dt).extend(0.0);
            if transform.translation.truncate().distance(player) < 28.0 {
                hurt_player(commands, game, player, enemy.kind.damage());
                enemy.charge_time = 0.0;
            }
            if enemy.charge_time == 0.0 {
                enemy.stun = 0.28;
            }
            continue;
        }

        match enemy.kind {
            EnemyKind::Ranged => {
                let preferred_distance = 218.0;
                if enemy.cooldown <= 0.0 && distance < preferred_distance + 95.0 {
                    begin_enemy_attack(commands, art, &mut enemy, player);
                    continue;
                }
                let radial = if distance > preferred_distance + 30.0 {
                    1.0
                } else if distance < preferred_distance - 36.0 {
                    -1.0
                } else {
                    0.0
                };
                transform.translation += (toward * enemy.kind.speed() * radial * dt).extend(0.0);
            }
            EnemyKind::Charger => {
                if enemy.cooldown <= 0.0 && distance <= enemy.kind.attack_range() {
                    begin_enemy_attack(commands, art, &mut enemy, player);
                    continue;
                }
                transform.translation +=
                    (toward * enemy.kind.speed() * speed_multiplier * dt).extend(0.0);
            }
            EnemyKind::Skirmisher => {
                if enemy.cooldown <= 0.0 && distance <= enemy.kind.attack_range() {
                    begin_enemy_attack(commands, art, &mut enemy, player);
                    continue;
                }
                let preferred_distance = 98.0;
                let radial = if distance > preferred_distance {
                    0.8
                } else if distance < preferred_distance - 18.0 {
                    -0.3
                } else {
                    0.0
                };
                let lateral = 0.58 * enemy.orbit_side;
                let orbit = Vec2::new(
                    toward.x * radial - toward.y * lateral,
                    toward.y * radial + toward.x * lateral,
                );
                transform.translation +=
                    (orbit * enemy.kind.speed() * speed_multiplier * dt).extend(0.0);
            }
            EnemyKind::Melee | EnemyKind::Heavy | EnemyKind::Bomber => {
                let attack_range = enemy.kind.attack_range();
                if enemy.cooldown <= 0.0 && distance <= attack_range {
                    begin_enemy_attack(commands, art, &mut enemy, player);
                    continue;
                }
                if distance > attack_range - 8.0 {
                    transform.translation +=
                        (toward * enemy.kind.speed() * speed_multiplier * dt).extend(0.0);
                }
            }
        }

        if enemy.health <= 0.0 {
            if let Some(telegraph) = enemy.telegraph.take() {
                commands.entity(telegraph).despawn();
            }
            commands.entity(entity).despawn();
        }
    }
    if count == 0 && !game.room_clear {
        game.room_clear = true;
        spawn_text(
            commands,
            "ROOM CLEAR  ·  APPROACH THE EAST DOOR AND PRESS E",
            Vec2::new(0.0, 190.0),
            18.0,
            Color::srgb(0.89, 0.78, 0.56),
        );
    }
    if game.player_health <= 0.0 {
        game.player_health = 0.0;
        game.overlay = Overlay::Ended(false);
        game.last_overlay_key.clear();
    }
}

fn begin_enemy_attack(commands: &mut Commands, art: &Art, enemy: &mut Enemy, target: Vec2) {
    enemy.target = target;
    enemy.windup = enemy.kind.attack_windup();
    let radius = enemy.kind.attack_radius();
    let lifetime = enemy.windup;
    enemy.telegraph = Some(
        commands
            .spawn((
                Mesh2d(art.circle.clone()),
                MeshMaterial2d(art.warning.clone()),
                Transform::from_xyz(target.x, target.y, 0.8).with_scale(Vec3::splat(radius * 0.12)),
                SceneEntity,
                AttackTelegraph {
                    age: 0.0,
                    lifetime,
                    radius,
                },
            ))
            .id(),
    );
}

fn resolve_enemy_attack(
    commands: &mut Commands,
    art: &Art,
    game: &mut Game,
    enemy: &mut Enemy,
    origin: Vec2,
    player: Vec2,
) {
    if let Some(telegraph) = enemy.telegraph.take() {
        commands.entity(telegraph).despawn();
    }
    let cooldown_multiplier = if enemy.elite == Some(EliteKind::Vanguard) {
        0.82
    } else {
        1.0
    };
    enemy.cooldown = enemy.kind.attack_cooldown() * cooldown_multiplier;

    match enemy.kind {
        EnemyKind::Charger => {
            enemy.charge_direction = (enemy.target - origin).normalize_or_zero();
            if enemy.charge_direction == Vec2::ZERO {
                enemy.charge_direction = Vec2::X;
            }
            enemy.charge_time = 0.34;
        }
        EnemyKind::Ranged => {
            let direction = (enemy.target - origin).normalize_or_zero();
            let spreads: &[f32] = if enemy.elite == Some(EliteKind::Volley) {
                &[-0.16, 0.0, 0.16]
            } else {
                &[0.0]
            };
            for spread in spreads {
                commands.spawn((
                    Mesh2d(art.circle.clone()),
                    MeshMaterial2d(art.ranged.clone()),
                    Transform::from_xyz(origin.x, origin.y, 3.0).with_scale(Vec3::splat(7.0)),
                    SceneEntity,
                    Projectile {
                        velocity: direction.rotate(Vec2::from_angle(*spread)) * 265.0,
                        lifetime: 2.8,
                        damage: enemy.kind.damage(),
                    },
                ));
            }
        }
        _ => {
            if player.distance(enemy.target) < enemy.kind.attack_radius() {
                hurt_player(commands, game, player, enemy.kind.damage());
            }
        }
    }
}

fn hurt_player(commands: &mut Commands, game: &mut Game, position: Vec2, damage: f32) {
    if game.invulnerable > 0.0 {
        return;
    }
    game.player_health = (game.player_health - damage).max(0.0);
    game.invulnerable = 0.72;
    spawn_damage(commands, position, damage, true);
    spawn_action_effect(
        commands,
        ActionVisual {
            position,
            angle: 0.0,
            size: Vec2::splat(44.0),
            rgba: [0.98, 0.16, 0.12, 0.64],
            lifetime: 0.24,
            start_scale: 0.60,
            end_scale: 1.08,
            rise_speed: 0.0,
            z: 2.8,
        },
    );
}

fn update_projectiles(
    commands: &mut Commands,
    game: &mut Game,
    projectiles: &mut ProjectileQuery<'_, '_>,
    player: Vec2,
    dt: f32,
) {
    for (entity, mut transform, mut projectile) in projectiles.iter_mut() {
        projectile.lifetime -= dt;
        transform.translation += (projectile.velocity * dt).extend(0.0);
        if transform.translation.truncate().distance(player) < 22.0 {
            hurt_player(commands, game, player, projectile.damage);
            commands.entity(entity).despawn();
        } else if projectile.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_damage(commands: &mut Commands, position: Vec2, amount: f32, received: bool) {
    let color = if received {
        Color::srgb(0.95, 0.20, 0.16)
    } else {
        Color::srgb(0.98, 0.90, 0.72)
    };
    let weight = if received {
        FontWeight::BOLD
    } else {
        FontWeight::SEMIBOLD
    };
    commands.spawn((
        Text2d::new(format!("-{}", amount as i32)),
        TextFont {
            font_size: FontSize::Px(if received { 25.0 } else { 22.0 }),
            weight,
            font_smoothing: FontSmoothing::None,
            ..default()
        },
        TextColor(color),
        Transform::from_xyz(position.x, position.y + 25.0, 8.0).with_scale(Vec3::splat(0.72)),
        SceneEntity,
        DamageNumber { age: 0.0, received },
    ));
}

fn animate_damage_numbers(
    mut commands: Commands,
    time: Res<Time>,
    mut numbers: Query<(Entity, &mut Transform, &mut TextColor, &mut DamageNumber)>,
) {
    let dt = time.delta_secs().min(0.05);
    for (entity, mut transform, mut color, mut number) in &mut numbers {
        number.age += dt;
        if number.age >= DAMAGE_NUMBER_LIFETIME {
            commands.entity(entity).despawn();
            continue;
        }

        transform.translation.y += 42.0 * dt;
        let scale = if number.age < 0.09 {
            0.72 + number.age / 0.09 * 0.58
        } else if number.age < 0.19 {
            1.30 - (number.age - 0.09) / 0.10 * 0.30
        } else {
            1.0
        };
        transform.scale = Vec3::splat(scale);

        let alpha = if number.age > 0.40 {
            ((DAMAGE_NUMBER_LIFETIME - number.age) / (DAMAGE_NUMBER_LIFETIME - 0.40))
                .clamp(0.0, 1.0)
        } else {
            1.0
        };
        let rgb = if number.received {
            (0.95, 0.20, 0.16)
        } else {
            (0.98, 0.90, 0.72)
        };
        color.0 = Color::srgba(rgb.0, rgb.1, rgb.2, alpha);
    }
}

fn start_run(game: &mut Game, bonus_seed: u64) {
    game.scene = Scene::Run;
    game.room = 0;
    game.player_health = 100.0;
    game.max_health = 100.0;
    game.bonuses.clear();
    game.bonus_damage = 0.0;
    game.bonus_range = 0.0;
    game.combo_index = 0;
    game.combo_timer = 0.0;
    game.room_clear = false;
    game.bonus_seed = bonus_seed;
    game.primary_cooldown = 0.0;
    game.heavy_windup = 0.0;
    game.heavy_recovery = 0.0;
    game.dash_cooldown = 0.0;
    game.overlay = Overlay::None;
    game.last_scene_key = None;
}

fn bonus_count(game: &Game, bonus: Bonus) -> usize {
    game.bonuses
        .iter()
        .filter(|picked| **picked == bonus)
        .count()
}

fn update_overlay_input(keys: &ButtonInput<KeyCode>, game: &mut Game, bonus_seed: u64) {
    match game.overlay.clone() {
        Overlay::Pause(mut selected) => {
            if keys.just_pressed(KeyCode::Escape) {
                game.overlay = Overlay::None;
            }
            if keys.just_pressed(KeyCode::ArrowDown) {
                selected = (selected + 1) % 4;
                game.overlay = Overlay::Pause(selected);
            }
            if keys.just_pressed(KeyCode::ArrowUp) {
                selected = (selected + 3) % 4;
                game.overlay = Overlay::Pause(selected);
            }
            if keys.just_pressed(KeyCode::Enter) {
                match selected {
                    0 => game.overlay = Overlay::None,
                    1 => start_run(game, bonus_seed),
                    2 => game.overlay = Overlay::Catalog,
                    _ => {
                        game.overlay = Overlay::None;
                        game.scene = Scene::Home;
                        game.last_scene_key = None;
                    }
                }
            }
        }
        Overlay::Catalog => {
            if keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::Enter) {
                game.overlay = Overlay::Pause(0);
            }
        }
        Overlay::BonusChoice(mut selected) => {
            if keys.just_pressed(KeyCode::ArrowLeft) {
                selected = (selected + 2) % 3;
                game.overlay = Overlay::BonusChoice(selected);
            }
            if keys.just_pressed(KeyCode::ArrowRight) {
                selected = (selected + 1) % 3;
                game.overlay = Overlay::BonusChoice(selected);
            }
            if keys.just_pressed(KeyCode::Enter) {
                let choices = bonus_choices(game.bonus_seed, game.room);
                apply_bonus(game, choices[selected]);
                game.room += 1;
                game.room_clear = false;
                game.overlay = Overlay::None;
                game.last_scene_key = None;
            }
        }
        Overlay::Ended(_) | Overlay::None => {}
    }
    game.last_overlay_key.clear();
}

fn apply_bonus(game: &mut Game, bonus: Bonus) {
    game.bonuses.push(bonus);
    match bonus {
        Bonus::Vitality => {
            game.max_health += 20.0;
            game.player_health = (game.player_health + 20.0).min(game.max_health);
        }
        Bonus::Edge => game.bonus_damage += 3.0,
        Bonus::Mend => game.player_health = (game.player_health + 35.0).min(game.max_health),
        Bonus::QuickStep => {}
        Bonus::Reach => game.bonus_range += 10.0,
        Bonus::DashNova
        | Bonus::WideSwing
        | Bonus::CinderCombo
        | Bonus::ComboBlast
        | Bonus::Siphon => {}
    }
}

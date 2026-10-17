use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

const VIEW_SIZE: Vec2 = Vec2::new(960.0, 540.0);
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
        .add_systems(Update, (sync_scene, sync_overlay, update_game).chain())
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
struct Player;

#[derive(Component)]
struct Enemy {
    kind: EnemyKind,
    health: f32,
    cooldown: f32,
}

#[derive(Clone, Copy)]
enum EnemyKind {
    Melee,
    Ranged,
    Heavy,
    Charger,
    Skirmisher,
    Bomber,
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
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
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
    for (kind, x, y) in enemies {
        let color = match kind {
            EnemyKind::Melee => &art.melee,
            EnemyKind::Ranged => &art.ranged,
            EnemyKind::Heavy => &art.heavy,
            EnemyKind::Charger => &art.charger,
            EnemyKind::Skirmisher => &art.skirmisher,
            EnemyKind::Bomber => &art.bomber,
        };
        commands.spawn((
            Mesh2d(art.circle.clone()),
            MeshMaterial2d(color.clone()),
            Transform::from_xyz(x, y, 2.0).with_scale(Vec3::splat(15.0)),
            SceneEntity,
            Enemy {
                kind,
                health: kind.health(),
                cooldown: 0.6,
            },
        ));
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

fn room_layout(room: usize) -> (&'static str, Vec2, Vec<(EnemyKind, f32, f32)>) {
    match room {
        0 => (
            "THE ENTRY",
            Vec2::new(-390.0, -22.0),
            vec![
                (EnemyKind::Melee, -230.0, 80.0),
                (EnemyKind::Ranged, 245.0, -65.0),
            ],
        ),
        1 => (
            "THE OUTER WALK",
            Vec2::new(-390.0, 0.0),
            vec![
                (EnemyKind::Melee, 10.0, 95.0),
                (EnemyKind::Melee, 170.0, -110.0),
                (EnemyKind::Ranged, 300.0, 0.0),
                (EnemyKind::Charger, 245.0, -150.0),
            ],
        ),
        2 => (
            "THE CROSSING",
            Vec2::new(-390.0, 0.0),
            vec![
                (EnemyKind::Heavy, 95.0, 40.0),
                (EnemyKind::Bomber, 30.0, -110.0),
                (EnemyKind::Ranged, 260.0, -105.0),
            ],
        ),
        3 => (
            "THE LOWER GALLERY",
            Vec2::new(-390.0, 0.0),
            vec![
                (EnemyKind::Melee, -20.0, 100.0),
                (EnemyKind::Heavy, 155.0, 0.0),
                (EnemyKind::Skirmisher, 35.0, -120.0),
                (EnemyKind::Ranged, 300.0, 100.0),
            ],
        ),
        _ => (
            "THE INNER SEAL",
            Vec2::new(-390.0, 0.0),
            vec![
                (EnemyKind::Melee, -40.0, 100.0),
                (EnemyKind::Ranged, -55.0, -120.0),
                (EnemyKind::Heavy, 185.0, 0.0),
                (EnemyKind::Charger, 300.0, 115.0),
                (EnemyKind::Ranged, 305.0, -125.0),
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
        mut enemies,
        mut projectiles,
        doors,
    } = params;
    let dt = time.delta_secs().min(0.05);
    if game.scene == Scene::Home {
        if let Ok((mut transform, _)) = player.single_mut() {
            move_player(&keys, &mut transform, &mut game.facing, 205.0, dt);
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
        game.dash_time = 0.18;
        game.dash_cooldown = (0.78 - bonus_count(&game, Bonus::QuickStep) as f32 * 0.1).max(0.25);
        game.invulnerable = 0.24;
        if game.bonuses.contains(&Bonus::DashNova) {
            hit_enemies(
                &mut commands,
                &art,
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
        610.0
    } else if game.heavy_windup > 0.0 {
        95.0
    } else {
        205.0
    };
    move_player(&keys, &mut player_transform, &mut game.facing, speed, dt);
    let player_pos = player_transform.translation.truncate();

    if keys.just_pressed(KeyCode::KeyJ)
        && game.primary_cooldown <= 0.0
        && game.heavy_recovery <= 0.0
    {
        game.primary_cooldown = 0.31;
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
            &art,
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
                &art,
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
    }
    if game.heavy_pending && game.heavy_windup <= 0.0 && game.heavy_recovery <= 0.0 {
        game.heavy_recovery = 0.56;
        game.heavy_pending = false;
        let wide = game.bonuses.contains(&Bonus::WideSwing);
        let hit_count = hit_enemies(
            &mut commands,
            &art,
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
    if keys.pressed(KeyCode::ArrowUp) || keys.pressed(KeyCode::KeyZ) {
        direction.y += 1.0;
    }
    if keys.pressed(KeyCode::ArrowDown) || keys.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }
    if keys.pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::KeyQ) {
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

#[derive(SystemParam)]
struct GameParams<'w, 's> {
    keys: Res<'w, ButtonInput<KeyCode>>,
    time: Res<'w, Time>,
    windows: Query<'w, 's, &'static Window>,
    cameras: Query<'w, 's, (&'static Camera, &'static GlobalTransform)>,
    game: ResMut<'w, Game>,
    art: Res<'w, Art>,
    player: Query<'w, 's, (&'static mut Transform, &'static mut Player)>,
    enemies: Query<'w, 's, (Entity, &'static mut Transform, &'static mut Enemy), Without<Player>>,
    projectiles: Query<'w, 's, (Entity, &'static mut Transform, &'static mut Projectile)>,
    doors: Query<'w, 's, &'static Transform, With<Door>>,
}

struct Attack {
    origin: Vec2,
    facing: Vec2,
    range: f32,
    damage: f32,
    half_arc: f32,
}

fn hit_enemies(
    commands: &mut Commands,
    art: &Art,
    enemies: &mut Query<(Entity, &mut Transform, &mut Enemy), Without<Player>>,
    attack: Attack,
) -> usize {
    let mut hits = 0;
    for (entity, transform, mut enemy) in enemies.iter_mut() {
        let offset = transform.translation.truncate() - attack.origin;
        let distance = offset.length();
        if distance <= attack.range
            && (distance < 1.0 || offset.normalize().dot(attack.facing) >= attack.half_arc)
        {
            enemy.health -= attack.damage;
            hits += 1;
            spawn_damage(
                commands,
                art,
                transform.translation.truncate(),
                attack.damage,
                false,
            );
            if enemy.health <= 0.0 {
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
    enemies: &mut Query<(Entity, &mut Transform, &mut Enemy), Without<Player>>,
    player: Vec2,
    dt: f32,
) {
    let mut count = 0;
    for (_, mut transform, mut enemy) in enemies.iter_mut() {
        count += 1;
        enemy.cooldown -= dt;
        let position = transform.translation.truncate();
        let delta = player - position;
        let distance = delta.length().max(0.01);
        let toward = delta / distance;
        let desired_range = match enemy.kind {
            EnemyKind::Melee => 48.0,
            EnemyKind::Ranged => 210.0,
            EnemyKind::Heavy => 58.0,
            EnemyKind::Charger => 48.0,
            EnemyKind::Skirmisher => 82.0,
            EnemyKind::Bomber => 68.0,
        };
        if enemy.cooldown <= 0.0
            && distance
                < if matches!(enemy.kind, EnemyKind::Ranged) {
                    280.0
                } else {
                    84.0
                }
        {
            enemy.cooldown = match enemy.kind {
                EnemyKind::Ranged => 1.75,
                EnemyKind::Heavy => 1.48,
                EnemyKind::Bomber => 2.05,
                _ => 1.1,
            };
            if distance <= desired_range + 26.0 {
                if matches!(enemy.kind, EnemyKind::Ranged) {
                    commands.spawn((
                        Mesh2d(art.circle.clone()),
                        MeshMaterial2d(art.ranged.clone()),
                        Transform::from_xyz(position.x, position.y, 3.0)
                            .with_scale(Vec3::splat(5.0)),
                        SceneEntity,
                        Projectile {
                            velocity: toward * 265.0,
                            lifetime: 1.4,
                            damage: enemy.kind.damage(),
                        },
                    ));
                } else if game.invulnerable <= 0.0 {
                    game.player_health -= enemy.kind.damage();
                    game.invulnerable = 0.72;
                    spawn_damage(commands, art, player, enemy.kind.damage(), true);
                }
            }
        }
        if distance > desired_range {
            transform.translation += (toward * enemy.kind.speed() * dt).extend(0.0);
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

fn update_projectiles(
    commands: &mut Commands,
    game: &mut Game,
    projectiles: &mut Query<(Entity, &mut Transform, &mut Projectile)>,
    player: Vec2,
    dt: f32,
) {
    for (entity, mut transform, mut projectile) in projectiles.iter_mut() {
        projectile.lifetime -= dt;
        transform.translation += (projectile.velocity * dt).extend(0.0);
        if transform.translation.truncate().distance(player) < 22.0 {
            if game.invulnerable <= 0.0 {
                game.player_health -= projectile.damage;
                game.invulnerable = 0.72;
            }
            commands.entity(entity).despawn();
        } else if projectile.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_damage(commands: &mut Commands, _art: &Art, position: Vec2, amount: f32, received: bool) {
    let color = if received {
        Color::srgb(0.95, 0.20, 0.16)
    } else {
        Color::srgb(0.98, 0.90, 0.72)
    };
    commands.spawn((
        Text2d::new(format!("{}", amount as i32)),
        TextFont {
            font_size: FontSize::Px(22.0),
            ..default()
        },
        TextColor(color),
        Transform::from_xyz(position.x, position.y + 25.0, 8.0),
        SceneEntity,
    ));
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

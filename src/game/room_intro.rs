//! Room entrance: the player walks in from the left onto its spawn point and
//! holds idle (input locked) while the enemies materialize one after another,
//! then the fight starts. Timings live in `room_flow.room_intro` of
//! `config/gameplay.yaml`.

use super::*;

/// How far into the room's appearance sequence an enemy starts materializing,
/// and the sprite scale it settles at.
#[derive(Component)]
pub(super) struct EnemyEntrance {
    pub(super) base_scale: Vec3,
    pub(super) delay_s: f32,
}

/// Shadow tint an enemy materializes from before settling on its own colors.
const ENEMY_APPEAR_TINT: Vec3 = Vec3::new(0.42, 0.18, 0.72);
/// Horizontal stretch of a materializing enemy at the start of its animation.
const ENEMY_APPEAR_STRETCH_X: f32 = 0.5;
/// Overshoot of the enemy's rise ("back out" easing; 1.70158 is the classic ~10%).
const ENEMY_APPEAR_OVERSHOOT: f32 = 1.70158;

pub(super) fn start_room_intro(data: Res<GameData>, mut timer: ResMut<PhaseTimer>) {
    timer.0 = Timer::from_seconds(
        data.gameplay.room_flow.room_intro.duration_s(),
        TimerMode::Once,
    );
}

pub(super) fn update_room_intro(
    time: Res<Time>,
    data: Res<GameData>,
    session: Res<RunSession>,
    mut timer: ResMut<PhaseTimer>,
    mut next_screen: ResMut<NextState<GameScreen>>,
    mut sounds: ResMut<SoundQueue>,
) {
    let enemies_start_s = data.gameplay.room_flow.room_intro.enemies_start_s();
    let before_s = timer.0.elapsed_secs();
    timer.0.tick(time.delta());
    if before_s < enemies_start_s && timer.0.elapsed_secs() >= enemies_start_s {
        sounds.push(SoundCue::EnemiesAppear);
    }
    if timer.0.is_finished() {
        next_screen.set(if session.room_id == 10 {
            GameScreen::BossIntro
        } else {
            GameScreen::Playing
        });
    }
}

#[derive(SystemParam)]
pub(super) struct RoomIntroPoseResources<'w> {
    screen: Res<'w, State<GameScreen>>,
    timer: Res<'w, PhaseTimer>,
    data: Res<'w, GameData>,
    session: Res<'w, RunSession>,
    sheets: Res<'w, SpriteSheets>,
}

type IntroPlayers<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut PlayerBody,
        &'static mut Transform,
        &'static mut Sprite,
        &'static mut AnimatedSprite,
    ),
    Without<EnemyBody>,
>;
type IntroEnemies<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static EnemyBody,
        &'static EnemyEntrance,
        &'static mut Transform,
        &'static mut Sprite,
    ),
    Without<PlayerBody>,
>;
type IntroHealthBars<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Sprite,
        AnyOf<(
            &'static EnemyHealthBar,
            &'static EnemyHealthBarBackground,
            &'static EnemyShieldIcon,
        )>,
    ),
    (Without<PlayerBody>, Without<EnemyBody>),
>;

/// Poses the player and enemies for the current point of the intro. It also
/// runs while the room fades in, holding everyone at the intro's start so the
/// room opens empty.
pub(super) fn pose_room_intro(
    resources: RoomIntroPoseResources,
    mut players: IntroPlayers,
    mut enemies: IntroEnemies,
    mut health_bars: IntroHealthBars,
) {
    let RoomIntroPoseResources {
        screen,
        timer,
        data,
        session,
        sheets,
    } = resources;
    let intro = &data.gameplay.room_flow.room_intro;
    let elapsed_s = if *screen.get() == GameScreen::RoomIntro {
        timer.0.elapsed_secs()
    } else {
        0.0
    };

    if let Ok((mut player, mut transform, mut sprite, mut animation)) = players.single_mut() {
        let walk = (elapsed_s / intro.player_walk_in_s).clamp(0.0, 1.0);
        let walk_in_distance_h = intro.player_walk_in_s * intro.player_walk_in_speed_h_s;
        let spawn_x_h = data.rooms[&session.room_id].player_spawn.x_h;
        transform.translation.x = (spawn_x_h - (1.0 - walk) * walk_in_distance_h) * H_WORLD_UNITS;
        player.velocity = Vec2::ZERO;
        player.facing = 1.0;
        sprite.flip_x = false;
        sprite.color = Color::WHITE.with_alpha((elapsed_s / intro.player_fade_in_s).min(1.0));
        let key = if walk < 1.0 {
            "player_walk"
        } else {
            "player_idle_alert"
        };
        set_animation(&mut sprite, &mut animation, &sheets, key, true);
    }

    let mut appearances = BTreeMap::new();
    for (entity, enemy, entrance, mut transform, mut sprite) in &mut enemies {
        let appear = ((elapsed_s - intro.enemies_start_s() - entrance.delay_s)
            / intro.enemy_appear_s)
            .clamp(0.0, 1.0);
        // Rise out of the ground with a slight overshoot, feet kept on the
        // spawn baseline, unstretching from a shadow tint to full color.
        let remaining = appear - 1.0;
        let rise = 1.0
            + (ENEMY_APPEAR_OVERSHOOT + 1.0) * remaining.powi(3)
            + ENEMY_APPEAR_OVERSHOOT * remaining.powi(2);
        transform.scale = Vec3::new(
            entrance.base_scale.x * (1.0 + ENEMY_APPEAR_STRETCH_X * (1.0 - appear)),
            entrance.base_scale.y * rise,
            entrance.base_scale.z,
        );
        transform.translation.y = enemy.home_y - enemy.feet_offset * (1.0 - rise);
        let tint = ENEMY_APPEAR_TINT.lerp(Vec3::ONE, appear);
        sprite.color = Color::srgba(tint.x, tint.y, tint.z, appear);
        appearances.insert(entity, appear);
    }
    for (mut sprite, (fill, background, shield)) in &mut health_bars {
        let enemy = fill
            .map(|bar| bar.enemy)
            .or_else(|| background.map(|bar| bar.enemy))
            .or_else(|| shield.map(|icon| icon.enemy));
        if let Some(appear) = enemy.and_then(|enemy| appearances.get(&enemy)) {
            sprite.color = sprite.color.with_alpha(*appear);
        }
    }
}

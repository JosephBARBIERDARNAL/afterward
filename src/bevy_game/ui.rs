use bevy::prelude::*;

use super::model::*;
use super::progression::bonus_choices;

pub(crate) const PLAYER_HEALTH_BAR_WIDTH: f32 = 144.0;

pub(crate) const PLAYER_HEALTH_BAR_HEIGHT: f32 = 9.0;

#[derive(Component)]
pub(crate) struct OverlayEntity;

#[derive(Component)]
pub(crate) struct PlayerHealthFill;

#[derive(Component)]
pub(crate) struct PlayerHealthReadout;

#[derive(Component)]
pub(crate) struct EnemyHealthBar {
    pub(crate) target: Entity,
    pub(crate) max_health: f32,
    pub(crate) width: f32,
    pub(crate) y_offset: f32,
    pub(crate) fill: bool,
}

pub(crate) fn spawn_player_health_display(commands: &mut Commands) {
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

pub(crate) fn spawn_enemy_health_bar(
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

pub(crate) fn update_player_health_display(
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

pub(crate) fn sync_enemy_health_bars(
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

pub(crate) fn spawn_door(commands: &mut Commands, position: Vec2) {
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

pub(crate) fn spawn_text(commands: &mut Commands, value: &str, pos: Vec2, size: f32, color: Color) {
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

pub(crate) fn sync_overlay(
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
        Overlay::BonusChoice { selected, source } => {
            let source_seed = match source {
                BonusSource::Door => game.bonus_seed,
                BonusSource::Cache => game.bonus_seed ^ 0xCA5E_CACE_5EED_0001,
            };
            let choices = bonus_choices(source_seed, game.room);
            let heading = match source {
                BonusSource::Door => "CHOOSE ONE RUN BONUS",
                BonusSource::Cache => "BONUS CACHE  ·  CHOOSE ONE",
            };
            format!(
                "{heading}\n\n{}\n{}\n{}\n\nARROWS + ENTER",
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

pub(crate) fn mark(selected: usize, index: usize) -> &'static str {
    if selected == index { ">" } else { " " }
}

pub(crate) fn mark_bonus(selected: usize, index: usize, bonus: Bonus) -> String {
    let prefix = if selected == index { ">" } else { " " };
    format!("{prefix} {}", bonus.title())
}

pub(crate) type EnemyHealthTargetQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static Transform, &'static Enemy),
    (With<Enemy>, Without<EnemyHealthBar>),
>;

pub(crate) type EnemyHealthBarQuery<'w, 's> = Query<
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

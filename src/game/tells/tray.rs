//! The row of what the run's bonuses hold in force, under the health bar:
//! one line per bonus at work, named in its rarity's colour, with what it
//! is worth now and, for one that runs out, a bar emptying. A keystone shows
//! what the build has made it worth, a once-a-room bonus that it is still
//! ready, a timed one how long it has left.

use super::*;
use bevy::text::FontSize;

/// How many lines the tray can show at once.
const ROWS: usize = 12;
const BAR_WIDTH_PX: f32 = 56.0;

#[derive(Component)]
pub(crate) struct TrayRow(usize);

#[derive(Component)]
pub(crate) struct TrayLabel(usize);

#[derive(Component)]
pub(crate) struct TrayValue(usize);

#[derive(Component)]
pub(crate) struct TrayTrough(usize);

#[derive(Component)]
pub(crate) struct TrayFill(usize);

pub(crate) fn spawn_the_tray(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(26),
                top: px(52),
                flex_direction: FlexDirection::Column,
                row_gap: px(3),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|tray| {
            for index in 0..ROWS {
                tray.spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: px(8),
                        ..default()
                    },
                    Visibility::Hidden,
                    TrayRow(index),
                    Pickable::IGNORE,
                ))
                .with_children(|row| {
                    let shadow = TextShadow {
                        offset: Vec2::ONE,
                        color: Color::srgba(0.0, 0.0, 0.0, 0.8),
                    };
                    row.spawn((
                        Text::new(""),
                        TextFont {
                            font_size: FontSize::Px(13.0),
                            ..default()
                        },
                        FontRole::BoldItalic,
                        shadow,
                        TrayLabel(index),
                    ));
                    row.spawn((
                        Node {
                            width: px(BAR_WIDTH_PX),
                            height: px(4),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
                        TrayTrough(index),
                    ))
                    .with_child((
                        Node {
                            height: percent(100),
                            ..default()
                        },
                        BackgroundColor(Color::WHITE),
                        TrayFill(index),
                    ));
                    row.spawn((
                        Text::new(""),
                        TextFont {
                            font_size: FontSize::Px(13.0),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        shadow,
                        TrayValue(index),
                    ));
                });
            }
        });
}

/// One line of the tray: a bonus at work. The build window reads them too,
/// to say what each bonus is doing.
pub(crate) struct Line {
    pub(crate) bonus: &'static str,
    /// What the bonus is worth now, or that it is ready.
    pub(crate) value: String,
    /// What is left of it, for one that runs out.
    pub(crate) left: Option<f32>,
}

fn percent_of(value: f32) -> String {
    format!("{:+.0}%", value * 100.0)
}

/// The lines the run's state calls for, in a fixed order. `wounded_at_rest`:
/// the player is hurt and no enemy is engaged, which is when Recovery heals.
pub(crate) fn lines(
    session: &RunSession,
    data: &GameData,
    player: &PlayerBody,
    wounded_at_rest: bool,
) -> Vec<Line> {
    let mut lines = Vec::new();
    let mut add = |bonus: &'static str, value: String, left: Option<f32>| {
        if session.has_bonus(bonus) {
            lines.push(Line { bonus, value, left });
        }
    };
    if let Some((bonus, multiplier)) = player.empowered {
        add(bonus, format!("next blow x{multiplier:.1}"), None);
    }
    let fraction = |left: f32, full: f32| (full > 0.0).then(|| (left / full).clamp(0.0, 1.0));
    // The height Wall Runner has left before the player next stands.
    if !player.grounded && player.wall_run_left > 0.0 {
        let full = session.bonus_effect(data, "wall_runner", "wall_run_h");
        let left = player.wall_run_left / H_WORLD_UNITS;
        add(
            "wall_runner",
            format!("{left:.1} H up a wall"),
            fraction(left, full),
        );
    }
    if player.chain_reaction_timer > 0.0 {
        let full = session.bonus_effect(data, "chain_reaction", "kill_damage_duration_s");
        add(
            "chain_reaction",
            percent_of(session.bonus_effect(data, "chain_reaction", "kill_damage_bonus")),
            fraction(player.chain_reaction_timer, full),
        );
        add(
            "bloodrush",
            format!(
                "chain x{:.1}",
                session.bonus_effect(data, "bloodrush", "combo_speed_multiplier")
            ),
            fraction(player.chain_reaction_timer, full),
        );
    }
    if player.sated_fury_timer > 0.0 {
        add(
            "sated_fury",
            format!(
                "chain x{:.2}",
                session.bonus_effect(data, "sated_fury", "healing_pickup_combo_speed_multiplier")
            ),
            fraction(
                player.sated_fury_timer,
                session.bonus_effect(data, "sated_fury", "healing_pickup_combo_speed_duration_s"),
            ),
        );
    }
    if player.rhythm_stacks > 0 {
        add(
            "rhythm_training",
            percent_of(
                f32::from(player.rhythm_stacks)
                    * session.bonus_effect(data, "rhythm_training", "combo_speed_step"),
            ),
            fraction(
                player.rhythm_timer,
                session.bonus_effect(data, "rhythm_training", "hit_window_s"),
            ),
        );
    }
    if session.health / session.max_health
        < session.bonus_effect(data, "adrenaline", "low_health_fraction")
    {
        add(
            "adrenaline",
            percent_of(
                session.bonus_effect(data, "adrenaline", "low_health_damage_taken_multiplier")
                    - 1.0,
            ),
            None,
        );
    }
    if wounded_at_rest {
        add(
            "recovery",
            format!(
                "+{:.0}/s",
                session.bonus_effect(data, "recovery", "idle_regeneration_per_s")
            ),
            None,
        );
    }
    if !session.room_ward_used {
        add("room_ward", "ready".to_owned(), None);
    }
    if !session.deflector_used {
        add("deflector", "ready".to_owned(), None);
    }
    if !session.last_stand_used {
        add("last_stand", "ready".to_owned(), None);
    }
    if session.has_bonus("lifesteal") {
        let cap = if session.has_bonus("bloodlust") {
            session.bonus_effect(data, "bloodlust", "room_healing_cap")
        } else {
            session.bonus_effect(data, "lifesteal", "room_healing_cap")
        };
        let left = (cap - player.lifesteal_healing).max(0.0);
        add(
            "lifesteal",
            format!("{left:.0}/{cap:.0} left"),
            fraction(left, cap),
        );
    }
    for keystone in ["arsenal", "harmony", "tailwind", "insight"] {
        add(keystone, percent_of(session.keystone(data, keystone)), None);
    }
    add(
        "fortress",
        percent_of(
            -session.keystone(data, "fortress").min(session.bonus_effect(
                data,
                "fortress",
                "max_reduction",
            )),
        ),
        None,
    );
    lines
}

#[derive(SystemParam)]
pub(crate) struct TrayResources<'w, 's> {
    screen: Res<'w, State<GameScreen>>,
    data: Res<'w, GameData>,
    session: Res<'w, RunSession>,
    players: Query<'w, 's, &'static PlayerBody>,
    enemies: Query<'w, 's, &'static EnemyBody>,
}

pub(crate) fn update_the_tray(
    resources: TrayResources,
    mut rows: Query<(&TrayRow, &mut Visibility)>,
    mut labels: Query<(&TrayLabel, &mut Text, &mut TextColor), Without<TrayValue>>,
    mut values: Query<(&TrayValue, &mut Text), Without<TrayLabel>>,
    mut troughs: Query<(&TrayTrough, &mut Node), Without<TrayFill>>,
    mut fills: Query<(&TrayFill, &mut Node, &mut BackgroundColor), Without<TrayTrough>>,
) {
    let TrayResources {
        screen,
        data,
        session,
        players,
        enemies,
    } = resources;
    let lines = match players.single() {
        Ok(player) if screen.get().shows_the_hud() => {
            let at_rest = !enemies
                .iter()
                .any(|enemy| enemy.health > 0.0 && enemy.engaged);
            lines(
                &session,
                &data,
                player,
                at_rest && session.health < session.max_health,
            )
        }
        _ => Vec::new(),
    };
    // A bonus is named in the colour of the rarity it was taken at.
    let named = |index: usize| {
        lines.get(index).and_then(|line| {
            session
                .owned(&data)
                .find(|(bonus, _)| bonus.id == line.bonus)
                .map(|(bonus, rarity)| {
                    (
                        line,
                        bonus,
                        Color::from(data.bonuses.rarities[rarity].color),
                    )
                })
        })
    };
    for (row, mut visibility) in &mut rows {
        *visibility = if named(row.0).is_some() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    for (label, mut text, mut color) in &mut labels {
        if let Some((_, bonus, rarity_color)) = named(label.0) {
            let name = bonus.name.to_uppercase();
            if text.0 != name {
                text.0 = name;
            }
            color.0 = rarity_color;
        }
    }
    for (value, mut text) in &mut values {
        if let Some((line, ..)) = named(value.0)
            && text.0 != line.value
        {
            text.0.clone_from(&line.value);
        }
    }
    for (trough, mut node) in &mut troughs {
        node.display = match named(trough.0) {
            Some((line, ..)) if line.left.is_some() => Display::Flex,
            _ => Display::None,
        };
    }
    for (fill, mut node, mut background) in &mut fills {
        if let Some((line, _, rarity_color)) = named(fill.0) {
            node.width = percent(line.left.unwrap_or(0.0) * 100.0);
            background.0 = rarity_color;
        }
    }
}

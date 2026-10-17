use bevy::prelude::*;

use super::model::Scene;
use super::model::*;

pub(crate) fn bonus_choices(seed: u64, room: usize) -> [Bonus; 3] {
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

pub(crate) fn start_run(game: &mut Game, bonus_seed: u64) {
    game.scene = Scene::Run;
    game.room = 0;
    game.player_health = 100.0;
    game.max_health = 100.0;
    game.bonuses.clear();
    game.bonus_damage = 0.0;
    game.bonus_range = 0.0;
    game.combo_index = 0;
    game.combo_timer = 0.0;
    game.cache_claimed = false;
    game.room_clear = false;
    game.bonus_seed = bonus_seed;
    game.primary_cooldown = 0.0;
    game.heavy_windup = 0.0;
    game.heavy_recovery = 0.0;
    game.heavy_pending = false;
    game.dash_time = 0.0;
    game.dash_cooldown = 0.0;
    game.invulnerable = 0.0;
    game.overlay = Overlay::None;
    game.last_scene_key = None;
}

pub(crate) fn return_home(game: &mut Game) {
    game.scene = Scene::Home;
    game.room = 0;
    game.player_health = 100.0;
    game.max_health = 100.0;
    game.bonuses.clear();
    game.bonus_damage = 0.0;
    game.bonus_range = 0.0;
    game.combo_index = 0;
    game.combo_timer = 0.0;
    game.cache_claimed = false;
    game.room_clear = false;
    game.primary_cooldown = 0.0;
    game.heavy_windup = 0.0;
    game.heavy_recovery = 0.0;
    game.heavy_pending = false;
    game.dash_time = 0.0;
    game.dash_cooldown = 0.0;
    game.invulnerable = 0.0;
    game.overlay = Overlay::None;
    game.last_scene_key = None;
}

pub(crate) fn bonus_count(game: &Game, bonus: Bonus) -> usize {
    game.bonuses
        .iter()
        .filter(|picked| **picked == bonus)
        .count()
}

pub(crate) fn update_overlay_input(keys: &ButtonInput<KeyCode>, game: &mut Game, bonus_seed: u64) {
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
                    _ => return_home(game),
                }
            }
        }
        Overlay::Catalog => {
            if keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::Enter) {
                game.overlay = Overlay::Pause(0);
            }
        }
        Overlay::BonusChoice {
            mut selected,
            source,
        } => {
            if keys.just_pressed(KeyCode::ArrowLeft) {
                selected = (selected + 2) % 3;
                game.overlay = Overlay::BonusChoice { selected, source };
            }
            if keys.just_pressed(KeyCode::ArrowRight) {
                selected = (selected + 1) % 3;
                game.overlay = Overlay::BonusChoice { selected, source };
            }
            if keys.just_pressed(KeyCode::Enter) {
                let source_seed = match source {
                    BonusSource::Door => game.bonus_seed,
                    BonusSource::Cache => game.bonus_seed ^ 0xCA5E_CACE_5EED_0001,
                };
                let choices = bonus_choices(source_seed, game.room);
                apply_bonus(game, choices[selected]);
                if matches!(source, BonusSource::Door) {
                    game.room += 1;
                    game.room_clear = false;
                    game.cache_claimed = false;
                    game.last_scene_key = None;
                }
                game.overlay = Overlay::None;
            }
        }
        Overlay::Ended(_) | Overlay::None => {}
    }
    game.last_overlay_key.clear();
}

pub(crate) fn apply_bonus(game: &mut Game, bonus: Bonus) {
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

//! Hiding from the enemies. Smoke Step, Unseen Entry, Fade, Vanishing Kill
//! and Shadow Climb each hide the player for `vanish_s` when what they wait
//! for happens, then wait `vanish_cooldown_s` before they can again. While
//! hidden, no enemy can see the player: one hunting them walks to where it
//! last saw them and stands down there (`enemy_ai.rs`), and a blow already on
//! its way still lands. The player is drawn as liquid glass meanwhile
//! (`shaders/glass.rs`).
//!
//! What sets each one off is read from what the game already keeps — a dash
//! begun, a wave arrived, health lost, an enemy fallen, a wall run begun — so
//! nothing else has to tell this module.

use super::*;

/// What the last frame left, to tell what changed since.
#[derive(Default)]
pub(in crate::game) struct LastSeen {
    dashing: bool,
    wall_running: bool,
    health: f32,
    /// The room and the wave last fought in it.
    wave: Option<(u16, u8)>,
}

#[derive(SystemParam)]
pub(in crate::game) struct VanishResources<'w> {
    time: Res<'w, Time>,
    data: Res<'w, GameData>,
    session: Res<'w, RunSession>,
    tells: MessageWriter<'w, BonusTell>,
}

/// Hides the player when one of their bonuses is set off, and keeps the
/// hiding and the waits that follow running down.
pub(in crate::game) fn vanish(
    resources: VanishResources,
    mut last: Local<LastSeen>,
    mut players: Query<(&mut PlayerBody, &Transform)>,
    enemies: Query<&EnemyBody>,
    fallen: Query<(), Added<DeadEnemy>>,
) {
    let VanishResources {
        time,
        data,
        session,
        mut tells,
    } = resources;
    let Ok((mut player, transform)) = players.single_mut() else {
        return;
    };
    let dt = time.delta_secs().min(0.05);
    player.invisible_s = (player.invisible_s - dt).max(0.0);
    for wait in player.vanish_cooldowns.values_mut() {
        *wait = (*wait - dt).max(0.0);
    }

    let dashing = player.dash_s.is_some();
    let fighting = enemies.iter().any(|enemy| enemy.health > 0.0);
    let wave = fighting.then_some((session.room_id, session.wave));
    let set_off = [
        ("smoke_step", dashing && !last.dashing),
        ("unseen_entry", wave.is_some() && wave != last.wave),
        ("fade", session.health < last.health && session.health > 0.0),
        ("vanishing_kill", !fallen.is_empty()),
        ("shadow_climb", player.wall_running && !last.wall_running),
    ];
    last.dashing = dashing;
    last.wall_running = player.wall_running;
    last.health = session.health;
    if wave.is_some() {
        last.wave = wave;
    }

    for (bonus, happened) in set_off {
        if !happened
            || !session.has_bonus(bonus)
            || player
                .vanish_cooldowns
                .get(bonus)
                .is_some_and(|wait| *wait > 0.0)
            || player.dying_s.is_some()
        {
            continue;
        }
        let was_hidden = player.invisible_s > 0.0;
        player.invisible_s = player
            .invisible_s
            .max(session.bonus_effect(&data, bonus, "vanish_s"));
        player.vanish_cooldowns.insert(
            bonus,
            session.bonus_effect(&data, bonus, "vanish_cooldown_s"),
        );
        tells.write(
            BonusTell::new(bonus, TellAt::Spot(transform.translation))
                .with(TellEffect::new("glint", 1.2)),
        );
        if was_hidden {
            continue;
        }
        // Phantom: a double is left where the player vanished, for
        // `plant_decoy` to stand up, and the hunt turns on it.
        if session.has_bonus("phantom") {
            player.dash_origin = Some(transform.translation);
            tells.write(BonusTell::new(
                "phantom",
                TellAt::Spot(transform.translation),
            ));
        }
        // Cloak and Dagger: the next blow out of hiding is heavier.
        if session.has_bonus("cloak_and_dagger") && player.empowered.is_none() {
            player.empowered = Some((
                "cloak_and_dagger",
                session.bonus_effect(&data, "cloak_and_dagger", "empowered_blow_multiplier"),
            ));
            tells.write(BonusTell::new("cloak_and_dagger", TellAt::Player));
        }
    }
}

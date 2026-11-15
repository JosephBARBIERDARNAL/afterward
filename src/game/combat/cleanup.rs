use super::*;

/// What a fight leaves behind and goes with it once it is won: shots still
/// flying, damage numbers, corpses, impacts, the double nothing is left to
/// strike at, and what the dead laid burning on the floor.
type LeftByTheFight = Or<(
    With<CombatProjectile>,
    With<DamageNumber>,
    With<DeadEnemy>,
    With<ImpactEffect>,
    With<Decoy>,
    With<Hazard>,
)>;

pub(in crate::game) fn clear_room_combat_effects(
    mut commands: Commands,
    left: Query<Entity, LeftByTheFight>,
    mut players: Query<&mut PlayerBody>,
) {
    for entity in &left {
        commands.entity(entity).despawn();
    }
    // What a bonus had stirred up in the player stands down with the fight,
    // as the devices power down and the music settles, so the walk to the
    // bonus is a true calm: no empowered blow carried into the next room, no
    // Chain Reaction or Sated Fury running out, no Rhythm stacks or chain to
    // keep going. Adrenaline goes on its own, with the health the clear
    // gives back.
    for mut player in &mut players {
        player.empowered = None;
        player.chain_reaction_timer = 0.0;
        player.sated_fury_timer = 0.0;
        player.rhythm_stacks = 0;
        player.rhythm_timer = 0.0;
        player.combo_step = 0;
        player.combo_window_s = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::{prelude::MinimalPlugins, state::app::StatesPlugin};

    #[test]
    fn room_clear_removes_what_the_fight_left_and_stands_the_bonuses_down() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_state::<GameScreen>()
            .add_systems(OnEnter(GameScreen::RoomClear), clear_room_combat_effects);
        app.update();

        let projectile = app
            .world_mut()
            .spawn(CombatProjectile {
                velocity_x: 1.0,
                velocity_y: 0.0,
                damage: 15.0,
                life_s: 1.0,
                remaining_range: 100.0,
                half_size: Vec2::splat(4.0),
            })
            .id();
        let damage_number = app
            .world_mut()
            .spawn(DamageNumber {
                timer: Timer::from_seconds(0.65, TimerMode::Once),
                color: Color::WHITE,
            })
            .id();
        let corpse = app
            .world_mut()
            .spawn(DeadEnemy {
                timer: Timer::from_seconds(0.9, TimerMode::Once),
            })
            .id();
        let impact = app
            .world_mut()
            .spawn(ImpactEffect {
                timer: Timer::from_seconds(0.35, TimerMode::Once),
                velocity: Vec2::ZERO,
                growth: 0.5,
                color: Color::WHITE,
            })
            .id();

        let player = app
            .world_mut()
            .spawn(PlayerBody {
                empowered: Some(("riposte", 1.5)),
                chain_reaction_timer: 2.0,
                sated_fury_timer: 2.0,
                rhythm_stacks: 3,
                rhythm_timer: 1.0,
                combo_step: 2,
                combo_window_s: 0.3,
                ..default()
            })
            .id();
        app.world_mut()
            .resource_mut::<NextState<GameScreen>>()
            .set(GameScreen::RoomClear);
        app.update();

        assert!(app.world().get_entity(projectile).is_err());
        assert!(app.world().get_entity(damage_number).is_err());
        assert!(app.world().get_entity(corpse).is_err());
        assert!(app.world().get_entity(impact).is_err());
        let player = app
            .world()
            .get::<PlayerBody>(player)
            .expect("the player stays");
        assert!(player.empowered.is_none(), "the empowered blow is spent");
        assert_eq!(player.chain_reaction_timer, 0.0);
        assert_eq!(player.sated_fury_timer, 0.0);
        assert_eq!(player.rhythm_stacks, 0);
        assert_eq!(player.combo_step, 0);
    }
}

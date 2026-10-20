use super::*;

pub(in crate::game) fn clear_room_combat_effects(
    mut commands: Commands,
    projectiles: Query<Entity, With<CombatProjectile>>,
    damage_numbers: Query<Entity, With<DamageNumber>>,
    corpses: Query<Entity, With<DeadEnemy>>,
) {
    for entity in &projectiles {
        commands.entity(entity).despawn();
    }
    for entity in &damage_numbers {
        commands.entity(entity).despawn();
    }
    for entity in &corpses {
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::{prelude::MinimalPlugins, state::app::StatesPlugin};

    #[test]
    fn room_clear_removes_frozen_projectiles_damage_numbers_and_corpses() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_state::<GameScreen>()
            .add_systems(OnEnter(GameScreen::RoomClear), clear_room_combat_effects);
        app.update();

        let projectile = app
            .world_mut()
            .spawn(CombatProjectile {
                velocity_x: 1.0,
                damage: 15.0,
                knockback_h_s: 0.0,
                life_s: 1.0,
                remaining_range: 100.0,
                friendly: true,
                half_size: Vec2::splat(4.0),
            })
            .id();
        let damage_number = app
            .world_mut()
            .spawn(DamageNumber {
                timer: Timer::from_seconds(0.65, TimerMode::Once),
                received: false,
            })
            .id();
        let corpse = app
            .world_mut()
            .spawn(DeadEnemy {
                timer: Timer::from_seconds(0.9, TimerMode::Once),
            })
            .id();

        app.world_mut()
            .resource_mut::<NextState<GameScreen>>()
            .set(GameScreen::RoomClear);
        app.update();

        assert!(app.world().get_entity(projectile).is_err());
        assert!(app.world().get_entity(damage_number).is_err());
        assert!(app.world().get_entity(corpse).is_err());
    }
}

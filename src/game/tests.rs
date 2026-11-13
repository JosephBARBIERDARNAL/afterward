use super::*;
mod black_knight;
mod bonus_screen_idle;
mod chapter_one_run;
mod control_bindings;
mod decoy;
mod e2e;
mod enemy_jumping;
mod enemy_shield;
mod enemy_tactics;
mod enemy_vision;
mod falling;
mod gameplay_controls;
mod melee_combo;
mod monsters;
mod platform_movement;
mod player_controls;
mod player_stances;
mod procedural_movement;
mod punch_sounds;
mod slam_and_roll;
mod slopes;
mod variable_jump;
mod wall_jump;

#[test]
fn projectiles_collide_with_platforms_along_their_travel_path() {
    // Bodies pass through a platform from any side but the top; shots don't.
    let platform = PlatformSurface {
        left: 100.0,
        right: 200.0,
        bottom: 0.0,
        top: 20.0,
        footstep: FootstepSound::Stone,
        tilt: None,
    };

    assert!(platform.intersects_projectile(
        Vec2::new(0.0, 10.0),
        Vec2::new(110.0, 10.0),
        Vec2::splat(6.0),
    ));
    assert!(!platform.intersects_projectile(
        Vec2::new(0.0, 30.0),
        Vec2::new(110.0, 30.0),
        Vec2::splat(6.0),
    ));
    assert!(!platform.intersects_projectile(
        Vec2::new(0.0, 10.0),
        Vec2::new(50.0, 10.0),
        Vec2::splat(6.0),
    ));
}

#[test]
fn platforms_hide_what_stands_behind_them() {
    // Sight travels like a shot: a platform between two points hides one from
    // the other, and one beside or below the line does not.
    let pillar = PlatformSurface {
        left: 100.0,
        right: 120.0,
        bottom: 0.0,
        top: 50.0,
        footstep: FootstepSound::Stone,
        tilt: None,
    };

    assert!(pillar.blocks_sight(Vec2::new(60.0, 25.0), Vec2::new(160.0, 25.0)));
    assert!(!pillar.blocks_sight(Vec2::new(60.0, 60.0), Vec2::new(160.0, 60.0)));
    // A sight line that stops short of the pillar is not blocked by it.
    assert!(!pillar.blocks_sight(Vec2::new(60.0, 25.0), Vec2::new(90.0, 25.0)));
    // Climbing over it: the line clears the top before reaching its near side.
    assert!(!pillar.blocks_sight(Vec2::new(90.0, 45.0), Vec2::new(130.0, 120.0)));
}

#[test]
fn start_room_parses_room_ids_and_rejects_anything_else() {
    assert_eq!("7".parse::<StartRoom>(), Ok(StartRoom(7)));
    assert!("boss".parse::<StartRoom>().is_err());
    assert_eq!(StartRoom::default(), StartRoom(0));
}

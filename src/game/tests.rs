use super::*;
mod bonus_screen_idle;
mod chapter_one_run;
mod control_bindings;
mod e2e;
mod enemy_jumping;
mod enemy_shield;
mod gameplay_controls;
mod gun;
mod player_controls;
mod player_stances;
mod procedural_movement;
mod terrain_movement;
mod variable_jump;

#[test]
fn projectiles_collide_with_solid_terrain_along_their_travel_path() {
    let platform = PlatformSurface {
        left: 100.0,
        right: 200.0,
        bottom: 0.0,
        top: 20.0,
        footstep: FootstepSound::Soil,
        one_way: false,
    };
    let plank = PlatformSurface {
        one_way: true,
        footstep: FootstepSound::Wood,
        ..platform
    };
    assert!(
        !plank.intersects_projectile(
            Vec2::new(0.0, 10.0),
            Vec2::new(110.0, 10.0),
            Vec2::splat(6.0)
        ),
        "projectiles fly through one-way platforms"
    );

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
fn start_room_parses_room_ids_and_rejects_anything_else() {
    assert_eq!("7".parse::<StartRoom>(), Ok(StartRoom(7)));
    assert!("boss".parse::<StartRoom>().is_err());
    assert_eq!(StartRoom::default(), StartRoom(1));
}

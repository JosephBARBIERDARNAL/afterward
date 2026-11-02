use super::*;
mod chapter_one_run;
mod control_bindings;
mod enemy_jumping;
mod gameplay_controls;
mod player_controls;
mod procedural_movement;

#[test]
fn projectiles_collide_with_platforms_along_their_travel_path() {
    let platform = PlatformSurface {
        left: 100.0,
        right: 200.0,
        bottom: 0.0,
        top: 20.0,
        material: PlatformMaterial::Soil,
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
fn start_room_parses_room_ids_and_rejects_anything_else() {
    assert_eq!("7".parse::<StartRoom>(), Ok(StartRoom(7)));
    assert!("boss".parse::<StartRoom>().is_err());
    assert_eq!(StartRoom::default(), StartRoom(1));
}

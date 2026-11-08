use super::*;
use crate::game::room_pan::DepartingRoom;

/// A door is crossed in the open: the room being left stays on screen while
/// the camera slides right into the next one, where the player steps out of
/// the door they came in through and it closes behind them.
#[test]
fn the_camera_slides_into_the_next_room_and_the_entry_door_goes_once_it_lands() {
    let mut game = HeadlessGame::new();
    game.tap(KeyCode::Enter);
    game.wait_for_screen(GameScreen::Playing, 10.0);
    game.take_bonus();
    game.tap(KeyCode::Digit1);
    game.leave_through_the_door();

    game.wait_for_screen(GameScreen::Panning, 3.0);
    assert!(
        game.departing_room_pieces() > 0,
        "the room being left is still drawn while the camera crosses"
    );
    assert_eq!(
        game.players(),
        1,
        "the player it left behind is gone, and the one waiting in the next \
         room has yet to walk in"
    );
    assert_eq!(
        game.player_opacity(),
        0.0,
        "so nobody is on show while the camera travels"
    );
    let started_at = game.camera_center().x;
    game.run_seconds(game.data().gameplay.room_flow.room_pan_s / 2.0);
    let halfway = game.camera_center().x;
    assert!(
        halfway > started_at,
        "the camera travels right, from {started_at} to {halfway}"
    );

    game.wait_for_screen(GameScreen::RoomIntro, 3.0);
    assert_eq!(game.session().room_id, 1);
    assert_eq!(
        game.departing_room_pieces(),
        0,
        "the room behind is let go the moment the camera lands"
    );
    assert_eq!(
        game.entry_doors(),
        1,
        "and the door the player came in through stands in the new room"
    );
    assert!(
        game.camera_center().x > halfway,
        "with the camera in the room it crossed into"
    );

    game.wait_for_screen(GameScreen::Playing, 5.0);
    assert_eq!(
        game.entry_doors(),
        0,
        "once the player is in, the door they came through is gone"
    );
    assert_eq!(game.players(), 1, "and they are the one standing there");
}

impl HeadlessGame {
    fn camera_center(&mut self) -> Vec2 {
        let world = self.app.world_mut();
        world
            .query_filtered::<&Transform, With<Camera2d>>()
            .single(world)
            .expect("the game draws through one camera")
            .translation
            .truncate()
    }

    fn departing_room_pieces(&mut self) -> usize {
        let world = self.app.world_mut();
        world
            .query_filtered::<Entity, With<DepartingRoom>>()
            .iter(world)
            .count()
    }

    fn entry_doors(&mut self) -> usize {
        let world = self.app.world_mut();
        world
            .query_filtered::<Entity, With<EntryDoor>>()
            .iter(world)
            .count()
    }

    /// How visible the player is, from 0 before they have walked in to 1 once
    /// they are standing in the room.
    fn player_opacity(&mut self) -> f32 {
        let world = self.app.world_mut();
        world
            .query_filtered::<&Sprite, With<PlayerBody>>()
            .single(world)
            .expect("a room holds one player")
            .color
            .alpha()
    }

    fn players(&mut self) -> usize {
        let world = self.app.world_mut();
        world
            .query_filtered::<Entity, With<PlayerBody>>()
            .iter(world)
            .count()
    }
}

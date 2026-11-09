use super::*;
use crate::game::{collision::PlatformSurface, interface::HealthBarFrame, room_pan::DepartingRoom};

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
    // How long the crossing takes is its own business: it runs at a steady
    // speed, so a wide room takes longer than a narrow one.
    let crossing_s = game
        .app
        .world()
        .resource::<PhaseTimer>()
        .0
        .duration()
        .as_secs_f32();
    game.run_seconds(crossing_s / 2.0);
    let halfway = game.camera_center().x;
    assert!(
        halfway > started_at,
        "the camera travels right, from {started_at} to {halfway}"
    );

    assert_eq!(
        game.hud_shown(),
        Visibility::Visible,
        "crossing a door is still being in the run, so the HUD stays up"
    );

    // The room is entered before the camera has landed: with the last of the
    // crossing left to run, the door is already open and the player already on
    // their way out of it.
    let early_start_s = game.data().gameplay.room_flow.room_intro.early_start_s;
    while game.screen() == GameScreen::Panning && game.crossing_left_s() > early_start_s / 2.0 {
        game.app.update();
    }
    assert_eq!(
        game.screen(),
        GameScreen::Panning,
        "the camera is still on its way"
    );
    assert!(
        game.player_opacity() > 0.0,
        "with the player stepping out of the door to meet it"
    );
    assert!(
        game.entry_door_frames()
            .first()
            .is_some_and(|frame| *frame >= 2),
        "which has been opening for them"
    );

    game.wait_for_screen(GameScreen::RoomIntro, 3.0);
    assert_eq!(game.session().room_id, 1);
    assert_eq!(
        game.departing_room_pieces(),
        0,
        "the room behind is let go the moment the camera lands"
    );
    assert_eq!(
        game.entry_door_frames().len(),
        1,
        "and the door the player came in through stands in the new room"
    );
    assert!(
        game.camera_center().x > halfway,
        "with the camera in the room it crossed into"
    );

    game.wait_for_screen(GameScreen::Playing, 5.0);
    assert_eq!(
        game.platform_colliders(),
        game.data().rooms[&1].platforms.len(),
        "the room being played is the only one the player can walk into: a \
         platform of the room left behind would stand in this one as an \
         invisible wall"
    );
    assert_eq!(
        game.entry_door_frames().len(),
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

    /// The frame each entry door on show is drawn on: 0 shut, 2 and up
    /// opening.
    fn entry_door_frames(&mut self) -> Vec<usize> {
        let world = self.app.world_mut();
        world
            .query_filtered::<&Sprite, With<EntryDoor>>()
            .iter(world)
            .filter_map(|sprite| sprite.texture_atlas.as_ref())
            .map(|atlas| atlas.index)
            .collect()
    }

    /// How many platforms the player can walk into, drawn or not.
    fn platform_colliders(&mut self) -> usize {
        let world = self.app.world_mut();
        world
            .query_filtered::<Entity, With<PlatformSurface>>()
            .iter(world)
            .count()
    }

    /// How much of the crossing is left to run.
    fn crossing_left_s(&self) -> f32 {
        self.app.world().resource::<PhaseTimer>().0.remaining_secs()
    }

    fn hud_shown(&mut self) -> Visibility {
        let world = self.app.world_mut();
        *world
            .query_filtered::<&Visibility, With<HealthBarFrame>>()
            .single(world)
            .expect("the run draws one health bar")
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

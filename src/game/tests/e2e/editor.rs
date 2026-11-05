use super::*;
use crate::data::RoomConfig;

/// Presses Cmd+`letter` as an AZERTY Mac sends it: the key is reported by its
/// QWERTY position (`code`) and typed as `letter`, and while Cmd is held macOS
/// sends no key-up, so a repeat arrives as a second press of a held key.
fn command_presses(game: &mut HeadlessGame, code: KeyCode, letter: &str, presses: usize) {
    game.typed_key(KeyCode::SuperLeft, Key::Super, ButtonState::Pressed);
    game.app.update();
    for _ in 0..presses {
        game.typed_key(code, Key::Character(letter.into()), ButtonState::Pressed);
        game.app.update();
    }
    game.typed_key(code, Key::Character(letter.into()), ButtonState::Released);
    game.typed_key(KeyCode::SuperLeft, Key::Super, ButtonState::Released);
    game.app.update();
}

#[test]
fn the_editor_edits_undoes_with_cmd_z_on_azerty_saves_and_playtests() {
    let mut game = HeadlessGame::with_copied_data();
    game.tap(KeyCode::KeyD);
    assert_eq!(game.screen(), GameScreen::Editor);
    let width = |game: &HeadlessGame| game.data().rooms[&1].size.width_viewports;
    let original = width(&game);
    let limits = &game.data().gameplay.room_size_limits.width_viewports;
    let (step, edited) = if f32::from(original) < limits.max {
        (KeyCode::ArrowRight, original + 1)
    } else {
        (KeyCode::ArrowLeft, original - 1)
    };

    // Room fields: Down selects the width, then two edits.
    game.tap(KeyCode::ArrowDown);
    game.tap(step);
    assert_eq!(width(&game), edited);
    game.tap(KeyCode::ArrowDown);
    game.tap(KeyCode::ArrowDown);
    game.tap(KeyCode::ArrowRight);
    let background = game.data().rooms[&1].background_profile.clone();

    // Cmd+Z, typed with AZERTY's Z key (QWERTY's W), undoes both edits.
    command_presses(&mut game, KeyCode::KeyW, "z", 2);
    assert_eq!(width(&game), original, "two Cmd+Z presses undo both edits");
    assert_ne!(game.data().rooms[&1].background_profile, background);

    // Cmd+S writes the edited room to the data folder.
    game.tap(KeyCode::ArrowUp);
    game.tap(KeyCode::ArrowUp);
    game.tap(step);
    command_presses(&mut game, KeyCode::KeyS, "s", 1);
    let saved_path = RoomConfig::file_path(&game.data().asset_root, 1);
    let saved: RoomConfig = yaml_serde::from_str(
        &fs::read_to_string(&saved_path).expect("the editor wrote the room file"),
    )
    .expect("the saved room is valid YAML");
    assert_eq!(saved.size.width_viewports, edited);

    // Cmd+R playtests the edited room.
    command_presses(&mut game, KeyCode::KeyR, "r", 1);
    game.wait_for_screen(GameScreen::Playing, 10.0);
    assert_eq!(game.session().room_id, 1);
    assert_eq!(width(&game), edited);
}

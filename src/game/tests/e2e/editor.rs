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
    let (step, edited) = (KeyCode::ArrowRight, original + 1);

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

#[test]
fn the_editor_selects_all_copies_into_another_room_adds_and_paints_a_decoration() {
    let mut game = HeadlessGame::with_copied_data();
    game.tap(KeyCode::KeyD);
    assert_eq!(game.screen(), GameScreen::Editor);
    let room = |game: &HeadlessGame, room_id: u16| game.data().rooms[&room_id].clone();
    let first = room(&game, 1);
    let second = room(&game, 2);
    let on_disk: RoomConfig = yaml_serde::from_str(
        &fs::read_to_string(RoomConfig::file_path(&game.data().asset_root, 1))
            .expect("room 1 is readable"),
    )
    .expect("room 1 is valid YAML");
    assert_eq!(first, on_disk, "the D that opens the editor adds nothing");

    // Cmd+A and Cmd+C copy every block, decoration, and enemy of room 1;
    // Cmd+V pastes them into room 2, and Cmd+Z takes them out again.
    command_presses(&mut game, KeyCode::KeyA, "a", 1);
    command_presses(&mut game, KeyCode::KeyC, "c", 1);
    game.tap(KeyCode::PageDown);
    command_presses(&mut game, KeyCode::KeyV, "v", 1);
    let pasted = room(&game, 2);
    assert_eq!(
        pasted.terrain.len() + pasted.platforms.len(),
        second.terrain.len() + second.platforms.len() + first.terrain.len() + first.platforms.len()
    );
    assert_eq!(
        pasted.enemy_spawns.len(),
        second.enemy_spawns.len() + first.enemy_spawns.len()
    );
    command_presses(&mut game, KeyCode::KeyZ, "z", 1);
    assert_eq!(room(&game, 2), second);
    assert_eq!(room(&game, 1), first, "copying leaves room 1 as it was");

    // D adds a decoration; Down to its Material field and Right repaints it.
    game.tap(KeyCode::KeyD);
    let added = room(&game, 2).decorations.len();
    assert_eq!(added, second.decorations.len() + 1);
    let material = |game: &HeadlessGame| room(game, 2).decorations[added - 1].material.clone();
    let before = material(&game);
    game.tap(KeyCode::ArrowDown);
    game.tap(KeyCode::ArrowRight);
    assert_ne!(material(&game), before);

    // Cmd+S writes the decoration to room 2's file.
    command_presses(&mut game, KeyCode::KeyS, "s", 1);
    let saved: RoomConfig = yaml_serde::from_str(
        &fs::read_to_string(RoomConfig::file_path(&game.data().asset_root, 2))
            .expect("the editor wrote the room file"),
    )
    .expect("the saved room is valid YAML");
    assert_eq!(saved.decorations.len(), added);
    assert_eq!(saved.decorations[added - 1].material, material(&game));
}

#[test]
fn the_editor_redraws_an_edited_room_with_its_background_in_place() {
    let mut game = HeadlessGame::with_copied_data();
    game.tap(KeyCode::KeyD);
    assert_eq!(game.screen(), GameScreen::Editor);

    // T adds a block and redraws the room on the frame of the press: the new
    // background layers must already sit on the camera, not flash elsewhere.
    game.key(KeyCode::KeyT, ButtonState::Pressed);
    game.app.update();
    let world = game.app.world_mut();
    let camera_y = world
        .query_filtered::<&Transform, With<Camera2d>>()
        .single(world)
        .expect("one camera")
        .translation
        .y;
    let backgrounds = world
        .query_filtered::<&Transform, With<camera::BackgroundParallax>>()
        .iter(world)
        .map(|transform| transform.translation.y)
        .collect::<Vec<_>>();
    assert!(!backgrounds.is_empty());
    assert!(
        backgrounds.iter().all(|&y| y == camera_y),
        "background layers at {backgrounds:?}, camera at {camera_y}"
    );
}

#[test]
fn the_editor_shows_its_shortcuts_only_while_h_is_held() {
    let mut game = HeadlessGame::with_copied_data();
    game.tap(KeyCode::KeyD);
    assert_eq!(game.screen(), GameScreen::Editor);
    let help_visible = |game: &mut HeadlessGame| {
        let world = game.app.world_mut();
        world
            .query::<(&Text, &Visibility)>()
            .iter(world)
            .find(|(text, _)| text.0.contains("box-select"))
            .map(|(_, visibility)| *visibility != Visibility::Hidden)
    };
    assert_eq!(help_visible(&mut game), Some(false), "hidden by default");
    game.key(KeyCode::KeyH, ButtonState::Pressed);
    game.app.update();
    assert_eq!(help_visible(&mut game), Some(true), "shown while H is held");
    game.key(KeyCode::KeyH, ButtonState::Released);
    game.app.update();
    assert_eq!(
        help_visible(&mut game),
        Some(false),
        "hidden again on release"
    );
}

use super::*;
use crate::data::{Fill, RoomConfig};

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
    // The editor opens the start room, the empty room 0; Page Down moves on
    // to room 1, which this scenario edits.
    game.tap(KeyCode::PageDown);
    let width = |game: &HeadlessGame| game.data().rooms[&1].size.width_h;
    let original = width(&game);
    let (step, edited) = (KeyCode::ArrowRight, original + 0.5);

    // Room fields: Down selects the width, then two edits.
    game.tap(KeyCode::ArrowDown);
    game.tap(step);
    assert_eq!(width(&game), edited);
    game.tap(KeyCode::ArrowDown);
    game.tap(KeyCode::ArrowDown);
    game.tap(KeyCode::ArrowRight);
    let background = game.data().rooms[&1].background_profiles.clone();

    // Cmd+Z, typed with AZERTY's Z key (QWERTY's W), undoes both edits.
    command_presses(&mut game, KeyCode::KeyW, "z", 2);
    assert_eq!(width(&game), original, "two Cmd+Z presses undo both edits");
    assert_ne!(game.data().rooms[&1].background_profiles, background);

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
    assert_eq!(saved.size.width_h, edited);

    // Cmd+R asks how the room should be played, and Enter plays it.
    command_presses(&mut game, KeyCode::KeyR, "r", 1);
    assert_eq!(game.screen(), GameScreen::Editor);
    game.tap(KeyCode::Enter);
    game.wait_for_screen(GameScreen::Playing, 10.0);
    assert_eq!(game.session().room_id, 1);
    assert_eq!(width(&game), edited);
}

#[test]
fn the_editor_selects_all_copies_into_another_room_adds_and_paints_a_decoration() {
    let mut game = HeadlessGame::with_copied_data();
    game.tap(KeyCode::KeyD);
    assert_eq!(game.screen(), GameScreen::Editor);
    // Page Down leaves the start room 0 for room 1, the room copied below.
    game.tap(KeyCode::PageDown);
    let room = |game: &HeadlessGame, room_id: u16| game.data().rooms[&room_id].clone();
    let first = room(&game, 1);
    let second = room(&game, 2);
    let on_disk: RoomConfig = yaml_serde::from_str(
        &fs::read_to_string(RoomConfig::file_path(&game.data().asset_root, 1))
            .expect("room 1 is readable"),
    )
    .expect("room 1 is valid YAML");
    assert_eq!(first, on_disk, "the D that opens the editor adds nothing");

    // Cmd+A and Cmd+C copy every platform, decoration, and enemy of room 1;
    // Cmd+V pastes them into room 2, and Cmd+Z takes them out again.
    command_presses(&mut game, KeyCode::KeyA, "a", 1);
    command_presses(&mut game, KeyCode::KeyC, "c", 1);
    game.tap(KeyCode::PageDown);
    command_presses(&mut game, KeyCode::KeyV, "v", 1);
    let pasted = room(&game, 2);
    assert_eq!(
        pasted.platforms.len(),
        second.platforms.len() + first.platforms.len()
    );
    assert_eq!(
        pasted.enemy_spawns.len(),
        second.enemy_spawns.len() + first.enemy_spawns.len()
    );
    command_presses(&mut game, KeyCode::KeyZ, "z", 1);
    assert_eq!(room(&game, 2), second);
    assert_eq!(room(&game, 1), first, "copying leaves room 1 as it was");

    // D adds a decoration; Down to its Fill field and Right repaints it.
    game.tap(KeyCode::KeyD);
    let added = room(&game, 2).decorations.len();
    assert_eq!(added, second.decorations.len() + 1);
    let fill = |game: &HeadlessGame| room(game, 2).decorations[added - 1].fill.clone();
    let before = fill(&game);
    game.tap(KeyCode::ArrowDown);
    game.tap(KeyCode::ArrowRight);
    assert_ne!(fill(&game), before);

    // Cmd+S writes the decoration to room 2's file.
    command_presses(&mut game, KeyCode::KeyS, "s", 1);
    let saved: RoomConfig = yaml_serde::from_str(
        &fs::read_to_string(RoomConfig::file_path(&game.data().asset_root, 2))
            .expect("the editor wrote the room file"),
    )
    .expect("the saved room is valid YAML");
    assert_eq!(saved.decorations.len(), added);
    assert_eq!(saved.decorations[added - 1].fill, fill(&game));
}

#[test]
fn the_editor_redraws_an_edited_room_with_its_background_in_place() {
    let mut game = HeadlessGame::with_copied_data();
    game.tap(KeyCode::KeyD);
    assert_eq!(game.screen(), GameScreen::Editor);

    // P adds a platform and redraws the room on the frame of the press: the
    // new background layers must already sit on the camera, not flash elsewhere.
    game.key(KeyCode::KeyP, ButtonState::Pressed);
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
            .query::<(&Text, &InheritedVisibility)>()
            .iter(world)
            .find(|(text, _)| text.0 == "box-select")
            .map(|(_, visibility)| visibility.get())
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

#[test]
fn the_editor_toggles_the_viewport_and_sight_overlays() {
    let mut game = HeadlessGame::with_copied_data();
    game.tap(KeyCode::KeyD);
    assert_eq!(game.screen(), GameScreen::Editor);
    // Room 1, which has enemies, so the sight overlay draws its bands and
    // lines for real rather than finding nothing to draw.
    game.tap(KeyCode::PageDown);
    // The status bar names the overlays that are on, and says nothing of
    // them while none is, so a toggle is visible even without a window to
    // draw them in.
    let shown = |game: &mut HeadlessGame| {
        let world = game.app.world_mut();
        world.query::<&Text>().iter(world).find_map(|text| {
            text.0
                .split("     ")
                .find(|part| part.starts_with("Overlays"))
                .map(str::to_string)
        })
    };
    assert_eq!(shown(&mut game), None, "none are on to begin with");

    game.tap(KeyCode::KeyF);
    assert_eq!(shown(&mut game).as_deref(), Some("Overlays: frame"));
    game.tap(KeyCode::KeyV);
    assert_eq!(shown(&mut game).as_deref(), Some("Overlays: frame, sight"));

    // Each key turns its own overlay back off, and the room is never edited.
    let room = game.data().rooms[&1].clone();
    game.tap(KeyCode::KeyF);
    assert_eq!(shown(&mut game).as_deref(), Some("Overlays: sight"));
    game.tap(KeyCode::KeyV);
    assert_eq!(shown(&mut game), None);
    assert_eq!(game.data().rooms[&1], room, "the toggles edit nothing");
}

#[test]
fn a_playtest_is_played_the_way_its_settings_ask() {
    let mut game = HeadlessGame::with_copied_data();
    game.tap(KeyCode::KeyD);
    game.tap(KeyCode::PageDown);
    assert!(!game.data().rooms[&1].enemy_spawns.is_empty());

    // Cmd+R opens the settings on their first row. Right turns infinite
    // health on, Down and Right turn the enemies off, and Down and two
    // Rights ask for every bonus.
    command_presses(&mut game, KeyCode::KeyR, "r", 1);
    game.tap(KeyCode::ArrowRight);
    game.tap(KeyCode::ArrowDown);
    game.tap(KeyCode::ArrowRight);
    game.tap(KeyCode::ArrowDown);
    game.tap(KeyCode::ArrowRight);
    game.tap(KeyCode::ArrowRight);
    // Escape puts the settings away without leaving the editor or playing.
    game.tap(KeyCode::Escape);
    assert_eq!(game.screen(), GameScreen::Editor);

    // They are remembered, and a second Cmd+R plays the room with them.
    command_presses(&mut game, KeyCode::KeyR, "r", 2);
    game.wait_for_screen(GameScreen::Playing, 10.0);
    assert_eq!(game.session().room_id, 1);
    assert!(game.enemies().is_empty(), "the room opens empty");
    assert_eq!(
        game.session().owned_bonuses.len(),
        game.data().bonuses.bonuses.len()
    );
    let health = game.session().health;
    game.hit_player(health * 2.0);
    game.run_seconds(1.0);
    assert_eq!(game.session().health, health, "a hit costs no health");
    assert_eq!(game.screen(), GameScreen::Playing);
}

#[test]
fn tab_moves_the_selection_to_the_closest_item_of_the_same_kind() {
    let mut game = HeadlessGame::with_copied_data();
    game.tap(KeyCode::KeyD);
    game.tap(KeyCode::PageDown);
    let enemies = |game: &HeadlessGame| game.data().rooms[&1].enemy_spawns.clone();
    let authored = enemies(&game).len();

    // Two enemies added on the same spot; the second, left selected, is
    // given another type so the two can be told apart.
    game.tap(KeyCode::KeyE);
    game.tap(KeyCode::KeyE);
    game.tap(KeyCode::ArrowDown);
    game.tap(KeyCode::ArrowRight);
    let (first, second) = (
        enemies(&game)[authored].clone(),
        enemies(&game)[authored + 1].clone(),
    );
    assert_ne!(first.type_id, second.type_id);

    // Tab leaves the second for the first, the closest enemy there is, so
    // Delete removes the first and only the first.
    game.tap(KeyCode::Tab);
    game.tap(KeyCode::Delete);
    assert_eq!(enemies(&game).len(), authored + 1);
    assert_eq!(enemies(&game)[authored], second);
}

#[test]
fn the_editor_adds_tunes_saves_and_deletes_a_vent() {
    let mut game = HeadlessGame::with_copied_data();
    game.tap(KeyCode::KeyD);
    game.tap(KeyCode::PageDown);
    let vents = |game: &HeadlessGame| game.data().rooms[&1].hazards.clone();
    let authored = vents(&game).len();

    // G adds a vent and selects it; its rows run Label, X, Y, Width,
    // Height, then Damage and the cycle down to the offset.
    game.tap(KeyCode::KeyG);
    assert_eq!(vents(&game).len(), authored + 1);
    let added = vents(&game)[authored].clone();
    for _ in 0..5 {
        game.tap(KeyCode::ArrowDown);
    }
    game.tap(KeyCode::ArrowRight);
    assert_eq!(vents(&game)[authored].damage, added.damage + 1.0);
    for _ in 0..4 {
        game.tap(KeyCode::ArrowDown);
    }
    game.tap(KeyCode::ArrowRight);
    assert_eq!(vents(&game)[authored].offset_s, added.offset_s + 0.1);
    // C starts its cycle as the room opens again.
    game.tap(KeyCode::KeyC);
    assert_eq!(vents(&game)[authored].offset_s, 0.0);

    // Cmd+S writes it to room 1's file, and Delete takes it out again.
    command_presses(&mut game, KeyCode::KeyS, "s", 1);
    let saved: RoomConfig = yaml_serde::from_str(
        &fs::read_to_string(RoomConfig::file_path(&game.data().asset_root, 1))
            .expect("the editor wrote the room file"),
    )
    .expect("the saved room is valid YAML");
    assert_eq!(saved.hazards, vents(&game));
    game.tap(KeyCode::Delete);
    assert_eq!(vents(&game).len(), authored);
}

#[test]
fn the_editor_adds_a_prop_at_its_own_size_repaints_it_and_saves_it() {
    let mut game = HeadlessGame::with_copied_data();
    game.tap(KeyCode::KeyD);
    game.tap(KeyCode::PageDown);
    let decorations = |game: &HeadlessGame| game.data().rooms[&1].decorations.clone();
    let authored = decorations(&game).len();

    // O adds the catalog's first prop as a background decoration exactly the
    // size of its sprite.
    game.tap(KeyCode::KeyO);
    assert_eq!(decorations(&game).len(), authored + 1);
    let added = decorations(&game)[authored].clone();
    let Fill::Prop(prop) = &added.fill else {
        panic!("O adds a prop, not {}", added.fill);
    };
    let surfaces = &game.data().surfaces;
    let [width_px, height_px] = surfaces.props[prop].size_px;
    assert_eq!(added.rect_h.width, width_px as f32 / surfaces.prop_px_per_h);
    assert_eq!(
        added.rect_h.height,
        height_px as f32 / surfaces.prop_px_per_h
    );

    // Down to its Fill row, and Right paints it with the next prop.
    game.tap(KeyCode::ArrowDown);
    game.tap(KeyCode::ArrowRight);
    let repainted = decorations(&game)[authored].fill.clone();
    assert!(matches!(&repainted, Fill::Prop(next) if next != prop));

    // Cmd+S writes it to room 1's file as `prop:`, and reads back the same.
    command_presses(&mut game, KeyCode::KeyS, "s", 1);
    let saved: RoomConfig = yaml_serde::from_str(
        &fs::read_to_string(RoomConfig::file_path(&game.data().asset_root, 1))
            .expect("the editor wrote the room file"),
    )
    .expect("the saved room is valid YAML");
    assert_eq!(saved.decorations[authored].fill, repainted);
}

use super::super::*;
use std::path::Path;

#[test]
fn fullscreen_default_binding_can_be_remapped() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let data = GameData::load(&asset_root).expect("Chapter 1 data should load");
    let mut app = App::new();
    app.insert_resource(data)
        .init_resource::<controls::ControlBindings>()
        .insert_resource(PreferencesDir(None))
        .init_resource::<ButtonInput<KeyCode>>()
        .add_systems(Startup, controls::initialize_control_bindings);
    app.update();

    let default_label = app
        .world()
        .resource::<controls::ControlBindings>()
        .binding_label(GameAction::ToggleFullscreen);
    assert_eq!(default_label, "F");

    app.world_mut()
        .resource_mut::<controls::ControlBindings>()
        .rebinding = Some(GameAction::ToggleFullscreen);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyK);

    let remapped =
        app.world_mut()
            .resource_scope(|world, mut bindings: Mut<controls::ControlBindings>| {
                bindings.capture_key(world.resource::<ButtonInput<KeyCode>>())
            });
    assert!(remapped);
    assert_eq!(
        app.world()
            .resource::<controls::ControlBindings>()
            .binding_label(GameAction::ToggleFullscreen),
        "K"
    );
}

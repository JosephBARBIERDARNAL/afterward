use super::super::*;
use std::path::Path;

#[test]
fn fullscreen_default_binding_can_be_remapped() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let data = GameData::load(&asset_root).expect("Chapter 1 data should load");
    let mut app = App::new();
    app.insert_resource(data)
        .init_resource::<controls::ControlBindings>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_systems(Startup, controls::initialize_default_control_bindings);
    app.update();

    let default_label = app
        .world()
        .resource::<controls::ControlBindings>()
        .binding_label(GameAction::ToggleFullscreen);
    assert_eq!(default_label, "F");

    {
        let mut bindings = app.world_mut().resource_mut::<controls::ControlBindings>();
        for _ in 0..9 {
            bindings.move_selection(1);
        }
        assert_eq!(bindings.selected_action(), GameAction::ToggleFullscreen);
        bindings.begin_rebinding();
    }
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyL);

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
        "L"
    );
}

use super::*;
use bevy::{
    app::AppExit,
    render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk},
    window::{PrimaryWindow, WindowMode},
};
use std::{fs, path::PathBuf};

pub struct VisualWalkthroughPlugin;

impl Plugin for VisualWalkthroughPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(VisualWalkthrough::default())
            .add_systems(Startup, initialize_visual_walkthrough)
            .add_systems(
                Update,
                drive_visual_walkthrough
                    .before(flow::handle_bonus_input)
                    .before(flow::handle_door_touch)
                    .before(flow::update_room_clear)
                    .before(flow::update_boss_intro)
                    .before(flow::update_room_transition)
                    .before(room_pan::update_room_pan)
                    .before(flow::update_room_transition_fade_out),
            );
    }
}

#[derive(Resource)]
struct VisualWalkthrough {
    output_dir: PathBuf,
    active_scene: Option<VisualScene>,
    elapsed_s: f32,
    capture_requested: bool,
    capture_completed: bool,
    action_started: bool,
    pause_shown: bool,
}

impl Default for VisualWalkthrough {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/visual-walkthrough"),
            active_scene: None,
            elapsed_s: 0.0,
            capture_requested: false,
            capture_completed: false,
            action_started: false,
            pause_shown: false,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct VisualScene {
    screen: GameScreen,
    room_id: u16,
    room_cleared: bool,
    door_open: bool,
}

#[derive(SystemParam)]
struct VisualWalkthroughResources<'w, 's> {
    commands: Commands<'w, 's>,
    time: Res<'w, Time>,
    screen: Res<'w, State<GameScreen>>,
    next_screen: ResMut<'w, NextState<GameScreen>>,
    timer: ResMut<'w, PhaseTimer>,
    session: Res<'w, RunSession>,
    data: Res<'w, GameData>,
    walkthrough: ResMut<'w, VisualWalkthrough>,
    settings_return: ResMut<'w, SettingsReturn>,
    enemies: Query<'w, 's, Entity, With<EnemyBody>>,
    players: Query<'w, 's, &'static mut Transform, With<PlayerBody>>,
    keys: ResMut<'w, ButtonInput<KeyCode>>,
}

fn initialize_visual_walkthrough(
    walkthrough: Res<VisualWalkthrough>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if let Err(error) = fs::create_dir_all(&walkthrough.output_dir) {
        error!(
            "Could not create visual walkthrough output directory {}: {error}",
            walkthrough.output_dir.display()
        );
        return;
    }
    if let Ok(mut window) = windows.single_mut() {
        window.mode = WindowMode::Windowed;
        window.set_maximized(false);
        window.resolution.set(1280.0, 720.0);
    }
}

fn drive_visual_walkthrough(resources: VisualWalkthroughResources) {
    let VisualWalkthroughResources {
        mut commands,
        time,
        screen,
        mut next_screen,
        mut timer,
        session,
        data,
        mut walkthrough,
        mut settings_return,
        enemies,
        mut players,
        mut keys,
    } = resources;
    let scene = VisualScene {
        screen: *screen.get(),
        room_id: session.room_id,
        room_cleared: session.room_cleared,
        door_open: session.door_open,
    };
    if walkthrough.active_scene != Some(scene) {
        walkthrough.active_scene = Some(scene);
        walkthrough.elapsed_s = 0.0;
        walkthrough.capture_requested = false;
        walkthrough.capture_completed = false;
        walkthrough.action_started = false;
        timer.0.unpause();
    }
    walkthrough.elapsed_s += time.delta_secs().min(0.05);

    // Stand beside whatever the shot is about, so the camera has settled on it
    // by the time the screenshot is taken.
    if scene.screen == GameScreen::Playing
        && (scene.door_open || scene.room_cleared)
        && !walkthrough.capture_completed
        && let (Some(room), Ok(mut player)) = (data.rooms.get(&scene.room_id), players.single_mut())
    {
        let spot = if scene.door_open {
            room.exit
        } else {
            room.bonus
        };
        player.translation.x = (spot.x_h - 4.0) * H_WORLD_UNITS;
        player.translation.y = spot.y_h * H_WORLD_UNITS + H_WORLD_UNITS / 2.0;
    }

    let capture_delay_s = match scene.screen {
        GameScreen::MainMenu => 0.5,
        GameScreen::Playing if scene.door_open => 0.35,
        GameScreen::Playing if scene.room_cleared => 0.35,
        GameScreen::Playing => 0.75,
        GameScreen::Paused | GameScreen::Settings | GameScreen::Controls => 0.35,
        GameScreen::RoomClear => 0.2,
        GameScreen::Bonus => 0.5,
        GameScreen::BossIntro => 0.35,
        GameScreen::Transitioning | GameScreen::TransitioningOut => 0.15,
        // Halfway across the gap, with both rooms in frame.
        GameScreen::Panning => data.gameplay.room_flow.room_pan_s / 2.0,
        GameScreen::Complete => 0.3,
        _ => return,
    };
    if !walkthrough.capture_requested && walkthrough.elapsed_s >= capture_delay_s {
        let screenshot_name = match scene.screen {
            GameScreen::MainMenu => String::from("main-menu"),
            GameScreen::Playing if scene.door_open => {
                format!("room_{:03}_door", scene.room_id)
            }
            GameScreen::Playing if scene.room_cleared => {
                format!("room_{:03}_bonus-item", scene.room_id)
            }
            GameScreen::Playing => format!("room_{:03}_combat", scene.room_id),
            GameScreen::Paused => String::from("paused-overlay"),
            GameScreen::Settings => String::from("settings"),
            GameScreen::Controls => String::from("controls"),
            GameScreen::RoomClear => format!("room_{:03}_clear", scene.room_id),
            GameScreen::Bonus => format!("room_{:03}_bonus", scene.room_id),
            GameScreen::BossIntro => format!("room_{:03}_boss-intro", scene.room_id),
            GameScreen::Transitioning => format!("room_{:03}_transition", scene.room_id),
            GameScreen::Panning => format!("room_{:03}_pan", scene.room_id),
            GameScreen::TransitioningOut => {
                format!("room_{:03}_transition-out", scene.room_id)
            }
            GameScreen::Complete => String::from("chapter-complete"),
            _ => unreachable!(),
        };
        let path = walkthrough
            .output_dir
            .join(format!("{screenshot_name}.png"));
        commands.spawn(Screenshot::primary_window()).observe(
            move |captured: On<ScreenshotCaptured>,
                  mut walkthrough: ResMut<VisualWalkthrough>,
                  mut app_exit: MessageWriter<AppExit>| {
                save_to_disk(&path)(captured);
                walkthrough.capture_completed = true;
                if scene.screen == GameScreen::Complete {
                    app_exit.write(AppExit::Success);
                }
            },
        );
        walkthrough.capture_requested = true;
        timer.0.pause();
        return;
    }
    if walkthrough.capture_requested && !walkthrough.capture_completed {
        timer.0.pause();
    } else {
        timer.0.unpause();
    }
    if !walkthrough.capture_completed || walkthrough.action_started {
        return;
    }

    walkthrough.action_started = true;
    match scene.screen {
        GameScreen::MainMenu => next_screen.set(GameScreen::Playing),
        GameScreen::Playing if scene.door_open => {
            if let (Some(room), Ok(mut player)) =
                (data.rooms.get(&scene.room_id), players.single_mut())
            {
                player.translation.x = room.exit.x_h * H_WORLD_UNITS;
                player.translation.y = room.exit.y_h * H_WORLD_UNITS + H_WORLD_UNITS / 2.0;
            }
        }
        // The cleared room's bonus item waits to be touched.
        GameScreen::Playing if scene.room_cleared => {
            if let (Some(room), Ok(mut player)) =
                (data.rooms.get(&scene.room_id), players.single_mut())
            {
                player.translation.x = room.bonus.x_h * H_WORLD_UNITS;
                player.translation.y = room.bonus.y_h * H_WORLD_UNITS + H_WORLD_UNITS / 2.0;
            }
        }
        GameScreen::Playing if !walkthrough.pause_shown => {
            walkthrough.pause_shown = true;
            next_screen.set(GameScreen::Paused);
        }
        GameScreen::Paused => {
            // The walkthrough steps between screens rather than pressing the
            // menu's buttons, so it says for itself where the settings were
            // opened from: that is what decides the backdrop behind them.
            *settings_return = SettingsReturn(GameScreen::Paused);
            next_screen.set(GameScreen::Settings);
        }
        GameScreen::Settings => next_screen.set(GameScreen::Controls),
        GameScreen::Controls => next_screen.set(GameScreen::Playing),
        GameScreen::Playing => {
            for entity in &enemies {
                commands.entity(entity).despawn();
            }
        }
        GameScreen::Bonus => {
            keys.release_all();
            keys.clear();
            keys.press(KeyCode::Digit1);
        }
        _ => {}
    }
}

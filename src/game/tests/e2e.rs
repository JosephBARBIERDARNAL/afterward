//! End-to-end scenarios: the real `GamePlugin` inside Bevy's default plugins,
//! headless (no window, GPU, or audio device), driven by keyboard events at a
//! fixed frame rate with a fixed random seed.
//!
//! Scenarios press keys as a player would. Three steps take a shortcut so room
//! layout edits never break them: `defeat_enemies` finishes a room's enemies,
//! `take_bonus` steps onto the bonus item it leaves behind and takes it, and
//! `step_toward_exit` puts the player within reach of the door.

use super::super::*;
use crate::game::{
    audio::{AudioClass, ManagedAudio},
    bonus_pickup::BonusPickup,
    combat::CombatProjectile,
    interact::InteractBadge,
    menus::{MenuAction, MenuButton},
    ripple::Ripple,
    shaders::{Flood, GlowAttached},
};
use bevy::{
    audio::{AudioPlugin, AudioSource},
    input::{
        ButtonState,
        keyboard::{Key, KeyboardInput, NativeKey},
    },
    log::LogPlugin,
    picking::{
        backend::HitData,
        pointer::{Location, PointerId},
    },
    render::{RenderPlugin, settings::WgpuSettings},
    time::TimeUpdateStrategy,
    window::{ExitCondition, WindowRef},
    winit::WinitPlugin,
};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

mod bonus;
mod death;
mod difficulty;
mod editor;
mod hitboxes;
mod impact;
mod main_menu;
mod menus;
mod move_bonuses;
mod opening_room;
mod pause;
mod prologue;
mod room_pan;
mod run;
mod waves;

const FRAME_S: f32 = 1.0 / 60.0;

/// The game running headless, with helpers to play it.
struct HeadlessGame {
    app: App,
    /// Temporary copy of the YAML data that saves go to, removed on drop.
    data_copy: Option<PathBuf>,
}

impl HeadlessGame {
    /// The game at its main menu, reading the real `src/assets` data.
    fn new() -> Self {
        let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
        Self::with_data_root(asset_root, None)
    }

    /// The game reading a temporary copy of the YAML data (and the art and
    /// sounds the data validation checks), so editor saves never touch `src/assets`.
    fn with_copied_data() -> Self {
        let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_nanos());
        let copy =
            std::env::temp_dir().join(format!("afterward-e2e-{}-{nanos}", std::process::id()));
        let mut pending = vec![PathBuf::from("config"), PathBuf::from("rooms")];
        pending.extend(
            [
                "player",
                "background",
                "audio",
                "surfaces",
                "doors",
                "pickups",
                "fonts",
            ]
            .map(PathBuf::from),
        );
        while let Some(relative) = pending.pop() {
            fs::create_dir_all(copy.join(&relative)).expect("the temp folder is writable");
            let entries = fs::read_dir(asset_root.join(&relative)).expect("assets are readable");
            for entry in entries.flatten() {
                let child = relative.join(entry.file_name());
                if entry.path().is_dir() {
                    pending.push(child);
                } else {
                    fs::copy(entry.path(), copy.join(&child)).expect("assets copy");
                }
            }
        }
        fs::create_dir_all(copy.join("enemies")).expect("the temp folder is writable");
        fs::copy(
            asset_root.join("enemies/enemy_types.yaml"),
            copy.join("enemies/enemy_types.yaml"),
        )
        .expect("enemy types copy");
        Self::with_data_root(copy.clone(), Some(copy))
    }

    fn with_data_root(data_root: PathBuf, data_copy: Option<PathBuf>) -> Self {
        let art_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
        let data = GameData::load(&data_root).expect("Chapter 1 data should load");
        let mut app = App::new();
        app.add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: art_root.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: ExitCondition::DontExit,
                    ..default()
                })
                .set(RenderPlugin {
                    render_creation: WgpuSettings {
                        backends: None,
                        ..default()
                    }
                    .into(),
                    ..default()
                })
                .disable::<WinitPlugin>()
                .disable::<LogPlugin>()
                .disable::<AudioPlugin>(),
        )
        .init_asset::<AudioSource>()
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            FRAME_S,
        )))
        .insert_resource(data)
        .insert_resource(PreferencesDir(None))
        .insert_resource(RandomState(0x6166_7465_7277_6172))
        .add_plugins(GamePlugin);
        app.update();
        Self { app, data_copy }
    }

    fn screen(&self) -> GameScreen {
        *self.app.world().resource::<State<GameScreen>>().get()
    }

    fn session(&self) -> &RunSession {
        self.app.world().resource::<RunSession>()
    }

    fn data(&self) -> &GameData {
        self.app.world().resource::<GameData>()
    }

    fn run_seconds(&mut self, seconds: f32) {
        for _ in 0..(seconds / FRAME_S).ceil() as usize {
            self.app.update();
        }
    }

    /// Runs until the game shows `screen`, failing after `timeout_s`.
    fn wait_for_screen(&mut self, screen: GameScreen, timeout_s: f32) {
        let mut waited_s = 0.0;
        while self.screen() != screen {
            assert!(
                waited_s < timeout_s,
                "expected {screen:?} within {timeout_s} s, still on {:?}",
                self.screen()
            );
            self.app.update();
            waited_s += FRAME_S;
        }
    }

    /// Sends a key event the way a QWERTY keyboard types it.
    fn key(&mut self, code: KeyCode, state: ButtonState) {
        let typed = format!("{code:?}")
            .strip_prefix("Key")
            .map(str::to_ascii_lowercase);
        let logical_key = typed.map_or(Key::Unidentified(NativeKey::Unidentified), |letter| {
            Key::Character(letter.into())
        });
        self.typed_key(code, logical_key, state);
    }

    fn typed_key(&mut self, key_code: KeyCode, logical_key: Key, state: ButtonState) {
        self.app.world_mut().write_message(KeyboardInput {
            key_code,
            logical_key,
            state,
            text: None,
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
    }

    /// The button a menu draws for `action`, the way the mouse finds it.
    fn menu_button(&mut self, action: MenuAction) -> Entity {
        let world = self.app.world_mut();
        world
            .query::<(Entity, &MenuButton)>()
            .iter(world)
            .find(|(_, button)| button.action == action)
            .map(|(entity, _)| entity)
            .unwrap_or_else(|| panic!("the menu on show draws a button for {action:?}"))
    }

    /// Where the mouse is, as the picking backend reports it. A window-less
    /// test has no real pointer, so the scenarios write the events the
    /// backend would have written.
    fn mouse_at(&self, at: Vec2) -> bevy::picking::events::Pointer {
        let target = WindowRef::Entity(Entity::PLACEHOLDER)
            .normalize(None)
            .expect("a named window normalizes without a primary one");
        bevy::picking::events::Pointer::new(
            PointerId::Mouse,
            Location {
                target: bevy::camera::NormalizedRenderTarget::Window(target),
                position: at,
            },
        )
    }

    /// Moves the mouse onto a button, at `at` in window pixels. Like `tap`,
    /// it runs a second frame, so a screen the touch asked for has landed.
    fn point_mouse_at(&mut self, button: Entity, at: Vec2) {
        let pointer = self.mouse_at(at);
        self.app.world_mut().write_message(PointerOver {
            entity: button,
            pointer,
            hit: HitData::new(Entity::PLACEHOLDER, 0.0, None, None),
        });
        self.app.update();
        self.app.update();
    }

    /// Clicks a button, at `at` in window pixels.
    fn click_mouse_on(&mut self, button: Entity, at: Vec2) {
        let pointer = self.mouse_at(at);
        self.app.world_mut().write_message(PointerClick {
            entity: button,
            pointer,
            button: PointerButton::Primary,
            hit: HitData::new(Entity::PLACEHOLDER, 0.0, None, None),
            duration: Duration::from_millis(40),
            count: 1,
        });
        self.app.update();
        self.app.update();
    }

    /// How many sounds of `class` are playing: the background layers are one
    /// entity each, so this is 1 while a layer is up and 0 once it is gone.
    fn layers(&mut self, class: AudioClass) -> usize {
        let world = self.app.world_mut();
        world
            .query::<&ManagedAudio>()
            .iter(world)
            .filter(|audio| audio.class == class)
            .count()
    }

    /// How many droplets are falling over the menus.
    fn droplets(&mut self) -> usize {
        let world = self.app.world_mut();
        world.query::<&Ripple>().iter(world).count()
    }

    /// Presses and releases `code` over two frames.
    fn tap(&mut self, code: KeyCode) {
        self.key(code, ButtonState::Pressed);
        self.app.update();
        self.key(code, ButtonState::Released);
        self.app.update();
    }

    /// From the main menu, opens a run in room 0: Begin asks how
    /// hard it should be, and the difficulty screen opens on the choice
    /// already made, which is Normal until one is made.
    fn begin_run(&mut self) {
        assert_eq!(self.screen(), GameScreen::MainMenu);
        self.tap(KeyCode::Enter);
        assert_eq!(self.screen(), GameScreen::Difficulty);
        self.tap(KeyCode::Enter);
        self.wait_for_screen(GameScreen::Playing, 10.0);
    }

    /// From the main menu, starts a run and plays it to room 1: a run opens
    /// in room 0, whose enemies (if it has any) are defeated and whose bonus
    /// item is picked up to unlock its door.
    fn start_run(&mut self) {
        self.begin_run();
        assert_eq!(self.session().room_id, 0);
        self.defeat_enemies();
        self.take_bonus();
        self.tap(KeyCode::Digit1);
        self.leave_through_the_door();
        self.wait_for_screen(GameScreen::Playing, 10.0);
    }

    /// Uses the room's open door and waits for the next room.
    fn leave_through_the_door(&mut self) {
        self.step_toward_exit();
        self.tap(KeyCode::KeyE);
        self.wait_for_screen(GameScreen::Transitioning, 3.0);
        self.skip_any_prologue();
    }

    /// The words the city is entered on, hurried along the way a player would:
    /// the first Enter finishes the writing, the next presses the button that
    /// walks in. Only the door out of the opening room leads through them.
    fn skip_any_prologue(&mut self) {
        for _ in 0..600 {
            match self.screen() {
                GameScreen::Prologue => self.tap(KeyCode::Enter),
                GameScreen::Transitioning => self.app.update(),
                _ => return,
            }
        }
        panic!("the prologue should give way once it is hurried along");
    }

    fn enemies(&mut self) -> Vec<(f32, f32)> {
        let world = self.app.world_mut();
        world
            .query::<&EnemyBody>()
            .iter(world)
            .map(|enemy| (enemy.health, enemy.max_health))
            .collect()
    }

    /// Shortcut: waits for the cleared room's bonus item, steps onto it, and
    /// waits for the bonus screen it opens. Where the item lies is the room's
    /// business, so the scenarios never name the spot.
    fn take_bonus(&mut self) {
        let mut waited_s = 0.0;
        while self
            .app
            .world_mut()
            .query_filtered::<Entity, With<BonusPickup>>()
            .iter(self.app.world())
            .next()
            .is_none()
        {
            assert!(
                waited_s < 3.0,
                "expected a bonus item in the cleared room, still on {:?}",
                self.screen()
            );
            self.app.update();
            waited_s += FRAME_S;
        }
        // Stand on the spot the item hovers over, rather than inside it: the
        // item is smaller than a body and floats clear of the ground.
        let room = &self.data().rooms[&self.session().room_id];
        let position = Vec3::new(
            room.bonus.x_h * H_WORLD_UNITS,
            (room.bonus.y_h + 0.5) * H_WORLD_UNITS,
            4.0,
        );
        let world = self.app.world_mut();
        for (mut transform, mut body) in world
            .query::<(&mut Transform, &mut PlayerBody)>()
            .iter_mut(world)
        {
            transform.translation = position;
            body.velocity = Vec2::ZERO;
            body.grounded = true;
        }
        self.tap(KeyCode::KeyE);
        self.wait_for_screen(GameScreen::Bonus, 3.0);
    }

    /// Shortcut: brings every enemy to 0 health, as if shot dead.
    fn defeat_enemies(&mut self) {
        // Wave after wave, until the room's last one is down.
        loop {
            let world = self.app.world_mut();
            for mut enemy in world.query::<&mut EnemyBody>().iter_mut(world) {
                enemy.health = 0.0;
            }
            let wave = self.session().wave;
            let more_to_come = self.session().has_enemies(self.data())
                && self.data().rooms[&self.session().room_id]
                    .enemy_spawns
                    .iter()
                    .any(|spawn| spawn.wave > wave);
            if !more_to_come {
                break;
            }
            let mut waited_s = 0.0;
            while self.session().wave == wave {
                assert!(waited_s < 5.0, "the next wave should have arrived");
                self.app.update();
                waited_s += FRAME_S;
            }
        }
    }

    /// Shortcut: puts the player on the floor half an H left of the exit,
    /// within reach of it.
    fn step_toward_exit(&mut self) {
        // The bonus screen holds a moment after a choice, and a door is only
        // used from the room itself.
        self.wait_for_screen(GameScreen::Playing, 3.0);
        let room = &self.data().rooms[&self.session().room_id];
        let position = Vec3::new(
            (room.exit.x_h - 0.5) * H_WORLD_UNITS,
            (room.exit.y_h + 0.5) * H_WORLD_UNITS,
            4.0,
        );
        let world = self.app.world_mut();
        for (mut transform, mut body) in world
            .query::<(&mut Transform, &mut PlayerBody)>()
            .iter_mut(world)
        {
            transform.translation = position;
            body.velocity = Vec2::ZERO;
            body.grounded = true;
        }
    }

    /// Fires an enemy projectile carrying `damage` into the player.
    fn hit_player(&mut self, damage: f32) {
        let world = self.app.world_mut();
        let Some(position) = world
            .query_filtered::<&Transform, With<PlayerBody>>()
            .iter(world)
            .next()
            .map(|transform| transform.translation)
        else {
            return;
        };
        world.spawn((
            Transform::from_translation(position + Vec3::X * 10.0),
            CombatProjectile {
                velocity_x: -H_WORLD_UNITS,
                damage,
                life_s: 1.0,
                remaining_range: H_WORLD_UNITS,
                half_size: Vec2::splat(4.0),
            },
        ));
        self.app.update();
    }
}

impl Drop for HeadlessGame {
    fn drop(&mut self) {
        if let Some(copy) = self.data_copy.take() {
            // Best effort: a leftover temp folder is harmless.
            drop(fs::remove_dir_all(copy));
        }
    }
}

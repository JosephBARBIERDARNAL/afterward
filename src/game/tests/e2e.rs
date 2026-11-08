//! End-to-end scenarios: the real `GamePlugin` inside Bevy's default plugins,
//! headless (no window, GPU, or audio device), driven by keyboard events at a
//! fixed frame rate with a fixed random seed.
//!
//! Scenarios press keys as a player would. Three steps take a shortcut so room
//! layout edits never break them: `defeat_enemies` finishes a room's enemies,
//! `take_bonus` steps onto the bonus item it leaves behind, and
//! `step_toward_exit` puts the player beside the door.

use super::super::*;
use crate::game::{bonus_pickup::BonusPickup, combat::CombatProjectile, shaders::GlowAttached};
use bevy::{
    audio::{AudioPlugin, AudioSource},
    input::{
        ButtonState,
        keyboard::{Key, KeyboardInput, NativeKey},
    },
    log::LogPlugin,
    render::{RenderPlugin, settings::WgpuSettings},
    time::TimeUpdateStrategy,
    window::ExitCondition,
    winit::WinitPlugin,
};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

mod bonus;
mod death;
mod editor;
mod hitboxes;
mod opening_room;
mod pause;
mod run;

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

    /// Presses and releases `code` over two frames.
    fn tap(&mut self, code: KeyCode) {
        self.key(code, ButtonState::Pressed);
        self.app.update();
        self.key(code, ButtonState::Released);
        self.app.update();
    }

    /// From the main menu, starts a run and plays it to room 1, where the
    /// first fight waits: a run opens in the empty room 0, whose bonus item is
    /// picked up to unlock its door.
    fn start_run(&mut self) {
        assert_eq!(self.screen(), GameScreen::MainMenu);
        self.tap(KeyCode::Enter);
        self.wait_for_screen(GameScreen::Playing, 10.0);
        assert_eq!(self.session().room_id, 0);
        self.take_bonus();
        self.tap(KeyCode::Digit1);
        self.leave_through_the_door();
        self.wait_for_screen(GameScreen::Playing, 10.0);
    }

    /// Walks the player into the room's open door and waits for the next room.
    fn leave_through_the_door(&mut self) {
        self.step_toward_exit();
        self.key(KeyCode::KeyD, ButtonState::Pressed);
        self.wait_for_screen(GameScreen::Transitioning, 3.0);
        self.key(KeyCode::KeyD, ButtonState::Released);
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
        self.wait_for_screen(GameScreen::Bonus, 3.0);
    }

    /// Shortcut: brings every enemy to 0 health, as if shot dead.
    fn defeat_enemies(&mut self) {
        let world = self.app.world_mut();
        for mut enemy in world.query::<&mut EnemyBody>().iter_mut(world) {
            enemy.health = 0.0;
        }
    }

    /// Shortcut: puts the player on the floor 1 H left of the exit, facing it.
    fn step_toward_exit(&mut self) {
        let room = &self.data().rooms[&self.session().room_id];
        let position = Vec3::new(
            (room.exit.x_h - 1.0) * H_WORLD_UNITS,
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

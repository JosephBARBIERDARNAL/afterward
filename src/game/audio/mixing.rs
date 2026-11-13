use super::*;
use crate::data::AudioConfig;

/// How loud each managed layer should be, as a fraction of its volume setting,
/// plus how muffled the music is. Every field moves towards its target over a
/// few frames so nothing jumps when a screen changes.
#[derive(Clone, Copy)]
pub(crate) struct AudioMix {
    music: f32,
    gameplay_effects: f32,
    /// 0 leaves the music untouched; 1 cuts its highs hardest, as if it played
    /// in another room.
    muffle: f32,
    /// How fast the music plays: 1 is its recorded speed, and below that it
    /// drags, dropping in pitch as well as in tempo.
    music_speed: f32,
}

impl AudioMix {
    fn target(screen: GameScreen, settings_return: SettingsReturn, audio: &AudioConfig) -> Self {
        // The settings and controls screens sound like a pause only when they
        // stand over one: opened from the main menu, they leave its music be.
        let over_a_paused_run = matches!(
            screen,
            GameScreen::Paused | GameScreen::Settings | GameScreen::Controls
        ) && !settings_return.over_the_flood(screen);
        Self {
            music: match screen {
                _ if over_a_paused_run => audio.paused_music_volume_multiplier,
                GameScreen::RoomClear | GameScreen::Bonus | GameScreen::BossIntro => {
                    audio.room_clear_music_volume_multiplier
                }
                _ => 1.0,
            },
            gameplay_effects: if gameplay_effects_muted(screen) {
                0.0
            } else {
                1.0
            },
            muffle: match screen {
                _ if over_a_paused_run => audio.paused_music_muffle,
                // A door is crossed through the wall between two rooms, and
                // everything playing is heard through it.
                GameScreen::Transitioning | GameScreen::RoomLoading | GameScreen::Panning => {
                    audio.crossing_music_muffle
                }
                _ => 0.0,
            },
            music_speed: match screen {
                GameScreen::RoomClear | GameScreen::Bonus | GameScreen::BossIntro => {
                    audio.room_clear_music_speed
                }
                _ => 1.0,
            },
        }
    }

    fn move_towards(self, target: Self, max_delta: f32) -> Self {
        Self {
            music: move_towards(self.music, target.music, max_delta),
            gameplay_effects: move_towards(
                self.gameplay_effects,
                target.gameplay_effects,
                max_delta,
            ),
            muffle: move_towards(self.muffle, target.muffle, max_delta),
            music_speed: move_towards(self.music_speed, target.music_speed, max_delta),
        }
    }

    fn differs_from(self, other: Self) -> bool {
        [
            (self.music, other.music),
            (self.gameplay_effects, other.gameplay_effects),
            (self.muffle, other.muffle),
            (self.music_speed, other.music_speed),
        ]
        .into_iter()
        .any(|(mine, theirs)| (mine - theirs).abs() > 0.0001)
    }

    fn of(self, class: AudioClass) -> f32 {
        match class {
            // The three background layers ride the same fader: the menu or
            // the pause screen lowers the city with the music over it.
            AudioClass::IntroMusic | AudioClass::Music | AudioClass::Ambience => self.music,
            AudioClass::GameplayEffect => self.gameplay_effects,
            AudioClass::InterfaceEffect | AudioClass::TerminalEffect => 1.0,
        }
    }
}

/// What the mix is computed from: the screen that decides the targets, the
/// tuning values, the volume settings, and the music streams to muffle.
#[derive(SystemParam)]
pub(crate) struct MixInputs<'w> {
    screen: Res<'w, State<GameScreen>>,
    data: Res<'w, GameData>,
    settings: Res<'w, AudioSettings>,
    settings_return: Res<'w, SettingsReturn>,
    music_streams: Res<'w, Assets<MusicStream>>,
}

/// How close to its target a fade counts as settled: a level moved towards a
/// target never quite lands on it.
const FADE_SETTLED: f32 = 0.001;

/// How loud a layer is while it fades, multiplied into its volume. A layer
/// that has faded away is done playing and despawns.
#[derive(Component)]
pub(crate) struct AudioFade {
    level: f32,
    target: f32,
    per_second: f32,
}

impl AudioFade {
    /// Rises from silence to full over `seconds`.
    pub(super) fn in_over(seconds: f32) -> Self {
        Self {
            level: 0.0,
            target: 1.0,
            per_second: 1.0 / seconds,
        }
    }

    /// Falls from where it is to silence over `seconds`, taking the layer
    /// with it.
    pub(super) fn out_over(seconds: f32) -> Self {
        Self {
            level: 1.0,
            target: 0.0,
            per_second: 1.0 / seconds,
        }
    }

    fn moving(&self) -> bool {
        (self.level - self.target).abs() > FADE_SETTLED
    }

    /// True once a layer on its way out is too quiet to be heard again: it has
    /// finished bowing out and its entity can go.
    fn faded_away(&self) -> bool {
        self.target <= 0.0 && self.level < FADE_SETTLED
    }
}

type MixedAudio = (
    Entity,
    &'static ManagedAudio,
    Option<&'static mut AudioFade>,
    Option<&'static mut AudioSink>,
);

pub fn update_audio_volume(
    mut commands: Commands,
    time: Res<Time>,
    inputs: MixInputs,
    mut audio_mix: Local<Option<AudioMix>>,
    mut active_audio: Query<MixedAudio>,
) {
    let settings = &inputs.settings;
    let target = AudioMix::target(
        *inputs.screen.get(),
        *inputs.settings_return,
        &inputs.data.gameplay.audio,
    );
    let previous_mix = *audio_mix;
    let next_mix = previous_mix
        .unwrap_or(target)
        .move_towards(target, time.delta_secs() * 4.0);
    *audio_mix = Some(next_mix);
    let mut fading = false;
    for (entity, _, fade, _) in &mut active_audio {
        let Some(mut fade) = fade.filter(|fade| fade.moving()) else {
            continue;
        };
        fade.level = move_towards(fade.level, fade.target, time.delta_secs() * fade.per_second);
        fading = true;
        if fade.faded_away() {
            commands.entity(entity).despawn();
        }
    }
    if !fading
        && !settings.is_changed()
        && previous_mix.is_some_and(|mix| !mix.differs_from(next_mix))
    {
        return;
    }
    for (_, stream) in inputs.music_streams.iter() {
        stream.set_muffle(next_mix.muffle);
    }
    for (_, audio, fade, sink) in &mut active_audio {
        let level = fade.map_or(1.0, |fade| fade.level);
        let Some(mut sink) = sink else {
            continue;
        };
        let channel_volume = match audio.class {
            AudioClass::IntroMusic | AudioClass::Music | AudioClass::Ambience => {
                settings.music_volume_percent
            }
            AudioClass::GameplayEffect
            | AudioClass::InterfaceEffect
            | AudioClass::TerminalEffect => settings.sfx_volume_percent,
        };
        sink.set_volume(Volume::Linear(
            channel_volume as f32 / 100.0 * audio.gain * next_mix.of(audio.class) * level,
        ));
        if audio.class == AudioClass::Music {
            sink.set_speed(next_mix.music_speed);
        }
    }
}

/// The screens where gameplay sounds have no business being heard: the menus
/// standing over a run, and the ones outside one.
pub(super) fn gameplay_effects_muted(screen: GameScreen) -> bool {
    matches!(
        screen,
        GameScreen::Paused
            | GameScreen::Settings
            | GameScreen::Controls
            | GameScreen::MainMenu
            | GameScreen::Difficulty
            | GameScreen::Dead
            | GameScreen::Complete
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AudioAsset, RecordedEffects};

    #[test]
    fn gameplay_audio_fades_outside_active_gameplay() {
        for screen in [
            GameScreen::MainMenu,
            GameScreen::Difficulty,
            GameScreen::Paused,
            GameScreen::Settings,
            GameScreen::Controls,
            GameScreen::Dead,
            GameScreen::Complete,
        ] {
            assert!(gameplay_effects_muted(screen));
        }

        for screen in [
            GameScreen::RoomIntro,
            GameScreen::Playing,
            GameScreen::RoomClear,
            GameScreen::Bonus,
            GameScreen::BossIntro,
            GameScreen::Transitioning,
            GameScreen::Prologue,
            GameScreen::Panning,
            GameScreen::RoomLoading,
            GameScreen::TransitioningOut,
        ] {
            assert!(!gameplay_effects_muted(screen));
        }
    }

    /// The tuning values the mix is read against; the shipped ones, so a test
    /// fails if the two drift apart.
    fn tuning() -> AudioConfig {
        let sound = || AudioAsset {
            asset: String::new(),
            gain: 1.0,
        };
        AudioConfig {
            footstep_interval_s: 0.42,
            paused_music_volume_multiplier: 0.35,
            room_clear_music_volume_multiplier: 0.55,
            room_clear_music_speed: 0.8,
            paused_music_muffle: 0.9,
            crossing_music_muffle: 0.55,
            music_tracks: Vec::new(),
            intro_music: sound(),
            intro_music_fade_out_s: 2.5,
            ambience: sound(),
            ambience_fade_in_s: 2.0,
            recorded_effects: RecordedEffects {
                door_open: sound(),
                bonus_chosen: sound(),
                bonus_reroll: sound(),
                damage_neutral: sound(),
                damage_metal: sound(),
                damage_metal_armor: sound(),
                punch_hits: Vec::new(),
                punch_missed: sound(),
                land: sound(),
            },
        }
    }

    /// The settings and controls screens stand over a paused run, so they
    /// sound like one: the music stays as lowered and as muffled as it was.
    #[test]
    fn pausing_muffles_and_lowers_the_music_without_stopping_it() {
        let audio = tuning();
        let over_a_run = SettingsReturn(GameScreen::Paused);
        let playing = AudioMix::target(GameScreen::Playing, over_a_run, &audio);

        for screen in [
            GameScreen::Paused,
            GameScreen::Settings,
            GameScreen::Controls,
        ] {
            let paused = AudioMix::target(screen, over_a_run, &audio);

            assert!(paused.music < playing.music, "{screen:?} lowers the music");
            assert!(paused.music > 0.0, "{screen:?} keeps the music playing");
            assert!(
                paused.muffle > playing.muffle,
                "{screen:?} muffles the music"
            );
        }
        assert!(
            playing.muffle < 0.0001,
            "playing leaves the music unfiltered"
        );
    }

    /// Opened from the main menu, the settings and controls screens stand over
    /// no run at all, and the music the menu was playing carries on untouched.
    #[test]
    fn settings_opened_from_the_main_menu_leave_the_music_alone() {
        let audio = tuning();
        let from_the_menu = SettingsReturn(GameScreen::MainMenu);
        let menu = AudioMix::target(GameScreen::MainMenu, from_the_menu, &audio);

        for screen in [GameScreen::Settings, GameScreen::Controls] {
            let settings = AudioMix::target(screen, from_the_menu, &audio);

            assert_eq!(settings.music, menu.music, "{screen:?} keeps it as loud");
            assert_eq!(settings.muffle, menu.muffle, "{screen:?} keeps it clear");
        }
    }

    /// Walking through a door is walking through a wall: what is playing is
    /// heard through it, without being turned down or slowed.
    #[test]
    fn crossing_between_rooms_muffles_what_is_playing() {
        let audio = tuning();
        let over_a_run = SettingsReturn(GameScreen::Paused);
        let playing = AudioMix::target(GameScreen::Playing, over_a_run, &audio);

        for screen in [
            GameScreen::Transitioning,
            GameScreen::RoomLoading,
            GameScreen::Panning,
        ] {
            let crossing = AudioMix::target(screen, over_a_run, &audio);

            assert!(
                crossing.muffle > playing.muffle,
                "{screen:?} muffles what is playing"
            );
            assert!(
                crossing.muffle < AudioMix::target(GameScreen::Paused, over_a_run, &audio).muffle,
                "{screen:?} less than a pause does"
            );
            assert_eq!(crossing.music, playing.music, "{screen:?} keeps it as loud");
        }
    }

    #[test]
    fn clearing_a_room_lowers_the_music_and_drags_it_down() {
        let audio = tuning();
        let over_a_run = SettingsReturn(GameScreen::Paused);
        let playing = AudioMix::target(GameScreen::Playing, over_a_run, &audio);

        for screen in [
            GameScreen::RoomClear,
            GameScreen::Bonus,
            GameScreen::BossIntro,
        ] {
            let cleared = AudioMix::target(screen, over_a_run, &audio);

            assert!(cleared.music < playing.music, "{screen:?} lowers the music");
            assert!(cleared.music > 0.0, "{screen:?} keeps the music playing");
            assert!(
                cleared.music_speed < playing.music_speed,
                "{screen:?} slows the music down"
            );
            assert!(
                cleared.music_speed > 0.0,
                "{screen:?} does not stall the music"
            );
        }

        assert!(
            (playing.music_speed - 1.0).abs() < 0.0001,
            "playing leaves the music at its recorded speed"
        );
    }
}

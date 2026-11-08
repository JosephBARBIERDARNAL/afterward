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
    fn target(screen: GameScreen, audio: &AudioConfig) -> Self {
        let paused = matches!(screen, GameScreen::Paused | GameScreen::Settings);
        Self {
            music: match screen {
                GameScreen::Paused | GameScreen::Settings => audio.paused_music_volume_multiplier,
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
            muffle: if paused {
                audio.paused_music_muffle
            } else {
                0.0
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
            AudioClass::Music => self.music,
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
    music_streams: Res<'w, Assets<MusicStream>>,
}

pub fn update_audio_volume(
    time: Res<Time>,
    inputs: MixInputs,
    mut audio_mix: Local<Option<AudioMix>>,
    mut active_audio: Query<(&ManagedAudio, Option<&mut AudioSink>)>,
) {
    let settings = &inputs.settings;
    let target = AudioMix::target(*inputs.screen.get(), &inputs.data.gameplay.audio);
    let previous_mix = *audio_mix;
    let next_mix = previous_mix
        .unwrap_or(target)
        .move_towards(target, time.delta_secs() * 4.0);
    *audio_mix = Some(next_mix);
    if !settings.is_changed() && previous_mix.is_some_and(|mix| !mix.differs_from(next_mix)) {
        return;
    }
    for (_, stream) in inputs.music_streams.iter() {
        stream.set_muffle(next_mix.muffle);
    }
    for (audio, sink) in &mut active_audio {
        let Some(mut sink) = sink else {
            continue;
        };
        let channel_volume = match audio.class {
            AudioClass::Music => settings.music_volume_percent,
            AudioClass::GameplayEffect
            | AudioClass::InterfaceEffect
            | AudioClass::TerminalEffect => settings.sfx_volume_percent,
        };
        sink.set_volume(Volume::Linear(
            channel_volume as f32 / 100.0 * audio.gain * next_mix.of(audio.class),
        ));
        if audio.class == AudioClass::Music {
            sink.set_speed(next_mix.music_speed);
        }
    }
}

fn gameplay_effects_muted(screen: GameScreen) -> bool {
    matches!(
        screen,
        GameScreen::Paused
            | GameScreen::Settings
            | GameScreen::MainMenu
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
            GameScreen::Paused,
            GameScreen::Settings,
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
            music_tracks: Vec::new(),
            recorded_effects: RecordedEffects {
                gun_shot: sound(),
                reload: sound(),
                door_open: sound(),
                bonus_chosen: sound(),
                bonus_reroll: sound(),
                damage_neutral: sound(),
                damage_metal: sound(),
                damage_metal_armor: sound(),
            },
        }
    }

    #[test]
    fn pausing_muffles_and_lowers_the_music_without_stopping_it() {
        let audio = tuning();
        let playing = AudioMix::target(GameScreen::Playing, &audio);
        let paused = AudioMix::target(GameScreen::Paused, &audio);

        assert!(paused.music < playing.music, "pausing lowers the music");
        assert!(paused.music > 0.0, "the music keeps playing while paused");
        assert!(paused.muffle > playing.muffle, "pausing muffles the music");
        assert!(
            playing.muffle < 0.0001,
            "playing leaves the music unfiltered"
        );
    }

    #[test]
    fn clearing_a_room_lowers_the_music_and_drags_it_down() {
        let audio = tuning();
        let playing = AudioMix::target(GameScreen::Playing, &audio);

        for screen in [
            GameScreen::RoomClear,
            GameScreen::Bonus,
            GameScreen::BossIntro,
        ] {
            let cleared = AudioMix::target(screen, &audio);

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

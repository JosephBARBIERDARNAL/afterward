use super::*;
use crate::data::AudioConfig;

/// How loud each managed layer should be, as a fraction of its volume setting,
/// plus how muffled the music is. Every field moves towards its target over a
/// few frames so nothing jumps when a screen changes.
#[derive(Clone, Copy)]
pub(crate) struct AudioMix {
    combat_music: f32,
    calm_music: f32,
    ambience: f32,
    track: f32,
    gameplay_effects: f32,
    /// 0 leaves the music untouched; 1 cuts its highs hardest, as if it played
    /// in another room.
    muffle: f32,
}

impl AudioMix {
    fn target(screen: GameScreen, session: &RunSession, audio: &AudioConfig) -> Self {
        let paused = matches!(screen, GameScreen::Paused | GameScreen::Settings);
        let music_volume = match screen {
            GameScreen::Paused | GameScreen::Settings => audio.paused_music_volume_multiplier,
            GameScreen::RoomClear | GameScreen::Bonus | GameScreen::BossIntro => {
                audio.room_clear_music_volume_multiplier
            }
            _ => 1.0,
        };
        let calm = matches!(screen, GameScreen::RoomClear | GameScreen::Bonus) || session.door_open;
        Self {
            combat_music: if calm { 0.0 } else { music_volume },
            calm_music: if calm { music_volume } else { 0.0 },
            ambience: music_volume,
            track: music_volume,
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
        }
    }

    fn move_towards(self, target: Self, max_delta: f32) -> Self {
        Self {
            combat_music: move_towards(self.combat_music, target.combat_music, max_delta),
            calm_music: move_towards(self.calm_music, target.calm_music, max_delta),
            ambience: move_towards(self.ambience, target.ambience, max_delta),
            track: move_towards(self.track, target.track, max_delta),
            gameplay_effects: move_towards(
                self.gameplay_effects,
                target.gameplay_effects,
                max_delta,
            ),
            muffle: move_towards(self.muffle, target.muffle, max_delta),
        }
    }

    fn differs_from(self, other: Self) -> bool {
        [
            (self.combat_music, other.combat_music),
            (self.calm_music, other.calm_music),
            (self.ambience, other.ambience),
            (self.track, other.track),
            (self.gameplay_effects, other.gameplay_effects),
            (self.muffle, other.muffle),
        ]
        .into_iter()
        .any(|(mine, theirs)| (mine - theirs).abs() > 0.0001)
    }

    fn of(self, class: AudioClass) -> f32 {
        match class {
            AudioClass::Music => self.combat_music,
            AudioClass::CalmMusic => self.calm_music,
            AudioClass::Ambience => self.ambience,
            AudioClass::Track => self.track,
            AudioClass::GameplayEffect => self.gameplay_effects,
            AudioClass::InterfaceEffect | AudioClass::TerminalEffect => 1.0,
        }
    }
}

/// What the mix is computed from: the screen and run that decide the targets,
/// the tuning values, the volume settings, and the music streams to muffle.
#[derive(SystemParam)]
pub(crate) struct MixInputs<'w> {
    screen: Res<'w, State<GameScreen>>,
    session: Res<'w, RunSession>,
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
    let target = AudioMix::target(
        *inputs.screen.get(),
        &inputs.session,
        &inputs.data.gameplay.audio,
    );
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
            AudioClass::Music
            | AudioClass::CalmMusic
            | AudioClass::Ambience
            | AudioClass::Track => settings.music_volume_percent,
            AudioClass::GameplayEffect
            | AudioClass::InterfaceEffect
            | AudioClass::TerminalEffect => settings.sfx_volume_percent,
        };
        sink.set_volume(Volume::Linear(
            channel_volume as f32 / 100.0 * audio.gain * next_mix.of(audio.class),
        ));
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

    #[test]
    fn pausing_muffles_and_lowers_the_music_without_stopping_it() {
        let sound = || AudioAsset {
            asset: String::new(),
            gain: 1.0,
        };
        let audio = AudioConfig {
            footstep_interval_s: 0.42,
            paused_music_volume_multiplier: 0.35,
            room_clear_music_volume_multiplier: 0.55,
            chapter_one_music_gain: 0.2,
            chapter_one_ambience_gain: 0.1,
            paused_music_muffle: 0.9,
            music_tracks: Vec::new(),
            recorded_effects: RecordedEffects {
                gun_shot: sound(),
                reload: sound(),
                door_open: sound(),
                damage_neutral: sound(),
                damage_metal: sound(),
                damage_metal_armor: sound(),
            },
        };
        let session = RunSession::default();
        let playing = AudioMix::target(GameScreen::Playing, &session, &audio);
        let paused = AudioMix::target(GameScreen::Paused, &session, &audio);

        assert!(paused.track < playing.track, "pausing lowers the music");
        assert!(paused.track > 0.0, "the music keeps playing while paused");
        assert!(paused.muffle > playing.muffle, "pausing muffles the music");
        assert!(
            playing.muffle < 0.0001,
            "playing leaves the music unfiltered"
        );
    }
}

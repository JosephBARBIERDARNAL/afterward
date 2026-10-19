use super::*;
use bevy::audio::{
    AudioPlayer, AudioSink, AudioSinkPlayback, AudioSource, PlaybackSettings, Volume,
};

mod synthesis;

use synthesis::{ChapterOneForestCalmLoop, ChapterOneForestLoop, ToneSpec, cue_specs};

#[derive(Resource, Default)]
pub(super) struct SoundQueue(Vec<SoundCue>);

impl SoundQueue {
    pub fn push(&mut self, cue: SoundCue) {
        if !self.0.contains(&cue) {
            self.0.push(cue);
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum SoundCue {
    UiMove,
    UiConfirm,
    UiCancel,
    UiInvalid,
    Jump,
    Landing,
    Footstep,
    Dash,
    SwordSwing,
    SwordHit,
    Gunshot,
    BulletHit,
    BulletEnvironmentHit,
    PlayerHurt,
    Pickup,
    EnemyIdle(EnemySoundStyle),
    EnemyMovement(EnemySoundStyle),
    EnemyTelegraph(EnemySoundStyle),
    EnemyAttack(EnemySoundStyle),
    BossIntro,
    EnemyHurt(EnemySoundStyle),
    EnemyDeath(EnemySoundStyle),
    BossPhase,
    RoomClear,
    DoorAppear,
    DoorOpen,
    BonusSelected,
    RunEnded,
    Victory,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum EnemySoundStyle {
    Generic,
    Swordsman,
    Infantryman,
    Destroyer,
    GorgonOne,
    GorgonTwo,
    GorgonThree,
    Warrior,
}

impl EnemySoundStyle {
    const ALL: [Self; 8] = [
        Self::Generic,
        Self::Swordsman,
        Self::Infantryman,
        Self::Destroyer,
        Self::GorgonOne,
        Self::GorgonTwo,
        Self::GorgonThree,
        Self::Warrior,
    ];

    fn base_frequency_hz(self) -> f32 {
        match self {
            Self::Generic => 170.0,
            Self::Swordsman => 220.0,
            Self::Infantryman => 380.0,
            Self::Destroyer => 100.0,
            Self::GorgonOne => 560.0,
            Self::GorgonTwo => 300.0,
            Self::GorgonThree => 145.0,
            Self::Warrior => 72.0,
        }
    }
}

impl From<&EnemyTypeId> for EnemySoundStyle {
    fn from(type_id: &EnemyTypeId) -> Self {
        match &**type_id {
            "swordsman" => Self::Swordsman,
            "infantryman" => Self::Infantryman,
            "destroyer" => Self::Destroyer,
            "gorgon_1" => Self::GorgonOne,
            "gorgon_2" => Self::GorgonTwo,
            "gorgon_3" => Self::GorgonThree,
            "warrior" => Self::Warrior,
            _ => Self::Generic,
        }
    }
}

impl SoundCue {
    fn is_interface(self) -> bool {
        matches!(
            self,
            Self::UiMove | Self::UiConfirm | Self::UiCancel | Self::UiInvalid
        )
    }

    fn is_terminal(self) -> bool {
        matches!(self, Self::RunEnded | Self::Victory)
    }

    pub(super) fn enemy_idle(type_id: &EnemyTypeId) -> Self {
        Self::EnemyIdle(EnemySoundStyle::from(type_id))
    }
}

#[derive(Resource, Default)]
pub(super) struct SoundBank {
    cues: BTreeMap<SoundCue, (Handle<AudioSource>, f32)>,
    chapter_one_music: Option<Handle<AudioSource>>,
    chapter_one_calm_music: Option<Handle<AudioSource>>,
}

#[derive(Component, Clone, Copy, Eq, PartialEq)]
enum AudioClass {
    Music,
    CalmMusic,
    GameplayEffect,
    InterfaceEffect,
    TerminalEffect,
}

#[derive(Component)]
pub(super) struct ManagedAudio {
    class: AudioClass,
    gain: f32,
}

pub(super) fn initialize_sound_bank(
    mut bank: ResMut<SoundBank>,
    mut audio_sources: ResMut<Assets<AudioSource>>,
) {
    for (cue, start_hz, end_hz, seconds, gain) in cue_specs() {
        let source = audio_sources.add(AudioSource::from(ToneSpec {
            start_hz,
            end_hz,
            seconds,
            amplitude: 0.8,
        }));
        bank.cues.insert(cue, (source, gain));
    }
    bank.chapter_one_music = Some(audio_sources.add(AudioSource::from(ChapterOneForestLoop)));
    bank.chapter_one_calm_music =
        Some(audio_sources.add(AudioSource::from(ChapterOneForestCalmLoop)));
}

pub(super) fn queue_victory_cue(mut queue: ResMut<SoundQueue>) {
    queue.push(SoundCue::Victory);
}

pub(super) fn queue_boss_intro(mut queue: ResMut<SoundQueue>) {
    queue.push(SoundCue::BossIntro);
}

pub(super) fn start_chapter_music(
    mut commands: Commands,
    bank: Res<SoundBank>,
    settings: Res<AudioSettings>,
    data: Res<GameData>,
    active_audio: Query<&ManagedAudio>,
) {
    if active_audio
        .iter()
        .any(|audio| matches!(audio.class, AudioClass::Music | AudioClass::CalmMusic))
    {
        return;
    }
    let (Some(combat_source), Some(calm_source)) = (
        bank.chapter_one_music.clone(),
        bank.chapter_one_calm_music.clone(),
    ) else {
        return;
    };
    let gain = data.gameplay.audio.chapter_one_music_gain;
    commands.spawn((
        AudioPlayer::new(combat_source),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(
            settings.music_volume_percent as f32 / 100.0 * gain,
        )),
        ManagedAudio {
            class: AudioClass::Music,
            gain,
        },
    ));
    commands.spawn((
        AudioPlayer::new(calm_source),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(0.0)),
        ManagedAudio {
            class: AudioClass::CalmMusic,
            gain,
        },
    ));
}

pub(super) fn stop_chapter_music(
    mut commands: Commands,
    active_audio: Query<(Entity, &ManagedAudio)>,
) {
    for (entity, audio) in &active_audio {
        if matches!(audio.class, AudioClass::Music | AudioClass::CalmMusic) {
            commands.entity(entity).despawn();
        }
    }
}

pub(super) fn play_sound_queue(
    mut commands: Commands,
    mut queue: ResMut<SoundQueue>,
    bank: Res<SoundBank>,
    settings: Res<AudioSettings>,
    screen: Res<State<GameScreen>>,
) {
    for cue in queue.0.drain(..) {
        let Some((source, gain)) = bank.cues.get(&cue) else {
            continue;
        };
        let class = if cue.is_interface() {
            AudioClass::InterfaceEffect
        } else if cue.is_terminal() {
            AudioClass::TerminalEffect
        } else {
            AudioClass::GameplayEffect
        };
        if class == AudioClass::GameplayEffect && gameplay_effects_muted(*screen.get()) {
            continue;
        }
        commands.spawn((
            AudioPlayer::new(source.clone()),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(
                settings.sfx_volume_percent as f32 / 100.0 * gain,
            )),
            ManagedAudio { class, gain: *gain },
        ));
    }
}

pub(super) fn update_audio_volume(
    time: Res<Time>,
    screen: Res<State<GameScreen>>,
    session: Res<RunSession>,
    data: Res<GameData>,
    settings: Res<AudioSettings>,
    mut audio_mix: Local<Option<(f32, f32, f32)>>,
    mut active_audio: Query<(&ManagedAudio, Option<&mut AudioSink>)>,
) {
    let target_music_volume = match screen.get() {
        GameScreen::Paused | GameScreen::Settings => {
            data.gameplay.audio.paused_music_volume_multiplier
        }
        GameScreen::RoomClear | GameScreen::Bonus | GameScreen::BossIntro => {
            data.gameplay.audio.room_clear_music_volume_multiplier
        }
        _ => 1.0,
    };
    let calm_music_active =
        matches!(screen.get(), GameScreen::RoomClear | GameScreen::Bonus) || session.door_open;
    let target_combat_music_mix = if calm_music_active {
        0.0
    } else {
        target_music_volume
    };
    let target_calm_music_mix = if calm_music_active {
        target_music_volume
    } else {
        0.0
    };
    let target_gameplay_effects_mix = if gameplay_effects_muted(*screen.get()) {
        0.0
    } else {
        1.0
    };
    let previous_mix = *audio_mix;
    let (current_combat_music_mix, current_calm_music_mix, current_gameplay_effects_mix) =
        previous_mix.unwrap_or((
            target_combat_music_mix,
            target_calm_music_mix,
            target_gameplay_effects_mix,
        ));
    let mix_speed = time.delta_secs() * 4.0;
    let next_combat_music_mix =
        move_towards(current_combat_music_mix, target_combat_music_mix, mix_speed);
    let next_calm_music_mix =
        move_towards(current_calm_music_mix, target_calm_music_mix, mix_speed);
    let next_gameplay_effects_mix = move_towards(
        current_gameplay_effects_mix,
        target_gameplay_effects_mix,
        mix_speed,
    );
    *audio_mix = Some((
        next_combat_music_mix,
        next_calm_music_mix,
        next_gameplay_effects_mix,
    ));
    let mix_changed = previous_mix.is_none_or(|(combat, calm, effects)| {
        (combat - next_combat_music_mix).abs() > 0.0001
            || (calm - next_calm_music_mix).abs() > 0.0001
            || (effects - next_gameplay_effects_mix).abs() > 0.0001
    });
    if !settings.is_changed() && !mix_changed {
        return;
    }
    for (audio, sink) in &mut active_audio {
        let Some(mut sink) = sink else {
            continue;
        };
        let (channel_volume, mix) = match audio.class {
            AudioClass::Music => (settings.music_volume_percent, next_combat_music_mix),
            AudioClass::CalmMusic => (settings.music_volume_percent, next_calm_music_mix),
            AudioClass::GameplayEffect => (settings.sfx_volume_percent, next_gameplay_effects_mix),
            AudioClass::InterfaceEffect | AudioClass::TerminalEffect => {
                (settings.sfx_volume_percent, 1.0)
            }
        };
        sink.set_volume(Volume::Linear(
            channel_volume as f32 / 100.0 * audio.gain * mix,
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
    fn interface_and_terminal_cues_keep_their_own_audio_class() {
        assert!(SoundCue::UiMove.is_interface());
        assert!(SoundCue::RunEnded.is_terminal());
        assert!(!SoundCue::Jump.is_interface());
        assert!(!SoundCue::Victory.is_interface());
    }
}

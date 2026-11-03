use super::*;
use crate::data::PlatformMaterial;
use bevy::audio::{
    AudioPlayer, AudioSink, AudioSinkPlayback, AudioSource, PlaybackSettings, Volume,
};

mod mixing;
mod synthesis;

pub(super) use mixing::update_audio_volume;

use synthesis::{
    ChapterOneForestAmbience, ChapterOneForestCalmLoop, ChapterOneForestLoop, ToneSpec, cue_specs,
};

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
    Footstep(PlatformMaterial),
    Dash,
    SwordSwing,
    SwordHit,
    SwordFinisherHit,
    ProjectileEnvironmentHit,
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
    RunRestarted,
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
    EvilWizard,
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
        Self::EvilWizard,
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
            Self::EvilWizard => 72.0,
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
            "evil_wizard" => Self::EvilWizard,
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
        matches!(self, Self::RunEnded | Self::RunRestarted | Self::Victory)
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
    chapter_one_ambience: Option<Handle<AudioSource>>,
}

#[derive(Component, Clone, Copy, Eq, PartialEq)]
enum AudioClass {
    Music,
    CalmMusic,
    Ambience,
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
    bank.chapter_one_ambience =
        Some(audio_sources.add(AudioSource::from(ChapterOneForestAmbience)));
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
    if active_audio.iter().any(|audio| {
        matches!(
            audio.class,
            AudioClass::Music | AudioClass::CalmMusic | AudioClass::Ambience
        )
    }) {
        return;
    }
    let (Some(combat_source), Some(calm_source), Some(ambience_source)) = (
        bank.chapter_one_music.clone(),
        bank.chapter_one_calm_music.clone(),
        bank.chapter_one_ambience.clone(),
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
    let ambience_gain = data.gameplay.audio.chapter_one_ambience_gain;
    commands.spawn((
        AudioPlayer::new(ambience_source),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(
            settings.music_volume_percent as f32 / 100.0 * ambience_gain,
        )),
        ManagedAudio {
            class: AudioClass::Ambience,
            gain: ambience_gain,
        },
    ));
}

pub(super) fn stop_chapter_music(
    mut commands: Commands,
    active_audio: Query<(Entity, &ManagedAudio)>,
) {
    for (entity, audio) in &active_audio {
        if matches!(
            audio.class,
            AudioClass::Music | AudioClass::CalmMusic | AudioClass::Ambience
        ) {
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
    fn interface_and_terminal_cues_keep_their_own_audio_class() {
        assert!(SoundCue::UiMove.is_interface());
        assert!(SoundCue::RunEnded.is_terminal());
        assert!(SoundCue::RunRestarted.is_terminal());
        assert!(!SoundCue::Jump.is_interface());
        assert!(!SoundCue::Victory.is_interface());
    }
}

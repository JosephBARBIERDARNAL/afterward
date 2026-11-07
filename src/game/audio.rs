use super::*;
use crate::data::FootstepSound;
use bevy::audio::{
    AudioPlayer, AudioSink, AudioSinkPlayback, AudioSource, PlaybackSettings, Volume,
};

mod mixing;
mod music;
mod synthesis;

pub(super) use mixing::update_audio_volume;
pub(super) use music::{MusicStream, MusicStreamLoader};

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
    Footstep(FootstepSound),
    Dash,
    GunShot,
    BulletHit,
    LastRoundHit,
    Reload,
    CoverBlock,
    ProjectileEnvironmentHit,
    PlayerHurt,
    Pickup,
    EnemyIdle(EnemySoundStyle),
    EnemyMovement(EnemySoundStyle),
    EnemyTelegraph(EnemySoundStyle),
    EnemyAttack(EnemySoundStyle),
    BossIntro,
    EnemiesAppear,
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
    /// The background tracks a run picks from, with their gains.
    music_tracks: Vec<(Handle<MusicStream>, f32)>,
    chapter_one_music: Option<Handle<MusicStream>>,
    chapter_one_calm_music: Option<Handle<MusicStream>>,
    chapter_one_ambience: Option<Handle<MusicStream>>,
}

#[derive(Component, Clone, Copy, Eq, PartialEq)]
enum AudioClass {
    Music,
    CalmMusic,
    Ambience,
    /// The recorded background track picked for the run.
    Track,
    GameplayEffect,
    InterfaceEffect,
    TerminalEffect,
}

impl AudioClass {
    /// Whether the class is one of the layers that play for a whole run.
    fn is_music(self) -> bool {
        matches!(
            self,
            Self::Music | Self::CalmMusic | Self::Ambience | Self::Track
        )
    }
}

#[derive(Component)]
pub(super) struct ManagedAudio {
    class: AudioClass,
    gain: f32,
}

pub(super) fn initialize_sound_bank(
    mut bank: ResMut<SoundBank>,
    mut audio_sources: ResMut<Assets<AudioSource>>,
    mut music_streams: ResMut<Assets<MusicStream>>,
    asset_server: Res<AssetServer>,
    data: Res<GameData>,
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
    let recorded = &data.gameplay.audio.recorded_effects;
    for (cue, effect) in [
        (SoundCue::GunShot, &recorded.gun_shot),
        (SoundCue::Reload, &recorded.reload),
        (SoundCue::DoorOpen, &recorded.door_open),
    ] {
        bank.cues
            .insert(cue, (asset_server.load(effect.asset.clone()), effect.gain));
    }
    bank.music_tracks = data
        .gameplay
        .audio
        .music_tracks
        .iter()
        .map(|track| (asset_server.load(track.asset.clone()), track.gain))
        .collect();
    bank.chapter_one_music =
        Some(music_streams.add(MusicStream::from(AudioSource::from(ChapterOneForestLoop))));
    bank.chapter_one_calm_music = Some(music_streams.add(MusicStream::from(AudioSource::from(
        ChapterOneForestCalmLoop,
    ))));
    bank.chapter_one_ambience = Some(music_streams.add(MusicStream::from(AudioSource::from(
        ChapterOneForestAmbience,
    ))));
}

pub(super) fn queue_victory_cue(mut queue: ResMut<SoundQueue>) {
    queue.push(SoundCue::Victory);
}

pub(super) fn queue_boss_intro(mut queue: ResMut<SoundQueue>) {
    queue.push(SoundCue::BossIntro);
}

/// Starts the music of a run, on the first room it enters. Later rooms find it
/// already playing and leave it alone, so one background track plays from the
/// first room to the end of the run.
pub(super) fn start_chapter_music(
    mut commands: Commands,
    bank: Res<SoundBank>,
    settings: Res<AudioSettings>,
    data: Res<GameData>,
    mut random: ResMut<RandomState>,
    active_audio: Query<&ManagedAudio>,
) {
    if active_audio.iter().any(|audio| audio.class.is_music()) {
        return;
    }
    if !bank.music_tracks.is_empty() {
        let index = random.index(bank.music_tracks.len());
        if let Some((source, gain)) = bank.music_tracks.get(index) {
            commands.spawn((
                AudioPlayer(source.clone()),
                // `MusicStream` loops itself, so that its filter keeps working
                // past the first play-through.
                PlaybackSettings::ONCE.with_volume(Volume::Linear(
                    settings.music_volume_percent as f32 / 100.0 * gain,
                )),
                ManagedAudio {
                    class: AudioClass::Track,
                    gain: *gain,
                },
            ));
        }
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
        AudioPlayer(combat_source),
        PlaybackSettings::ONCE.with_volume(Volume::Linear(
            settings.music_volume_percent as f32 / 100.0 * gain,
        )),
        ManagedAudio {
            class: AudioClass::Music,
            gain,
        },
    ));
    commands.spawn((
        AudioPlayer(calm_source),
        PlaybackSettings::ONCE.with_volume(Volume::Linear(0.0)),
        ManagedAudio {
            class: AudioClass::CalmMusic,
            gain,
        },
    ));
    let ambience_gain = data.gameplay.audio.chapter_one_ambience_gain;
    commands.spawn((
        AudioPlayer(ambience_source),
        PlaybackSettings::ONCE.with_volume(Volume::Linear(
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
        if audio.class.is_music() {
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

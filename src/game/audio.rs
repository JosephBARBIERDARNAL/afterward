use super::*;
use crate::data::{EnemyMaterial, FootstepSound};
use bevy::audio::{
    AudioPlayer, AudioSink, AudioSinkPlayback, AudioSource, PlaybackSettings, Volume,
};

mod mixing;
mod music;

use mixing::gameplay_effects_muted;
pub(super) use mixing::{AudioFade, update_audio_volume};
pub(super) use music::{MusicStream, MusicStreamLoader};

#[derive(Resource, Default)]
pub(super) struct SoundQueue(pub(super) Vec<SoundCue>);

impl SoundQueue {
    pub fn push(&mut self, cue: SoundCue) {
        if !self.0.contains(&cue) {
            self.0.push(cue);
        }
    }
}

/// Something that happened and wants to be heard. A cue plays the recording
/// `audio.recorded_effects` lists for it and is silent otherwise, so most of
/// these wait on sound design rather than on code.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum SoundCue {
    UiMove,
    UiConfirm,
    UiCancel,
    UiInvalid,
    Jump,
    Landing,
    Footstep(FootstepSound),
    /// A blow of the melee chain being thrown.
    MeleeSwing,
    /// A blow connecting, by which of `recorded_effects.punch_hits` it plays.
    MeleeHit(usize),
    /// A blow arriving on nobody.
    MeleeMiss,
    CoverBlock,
    ProjectileEnvironmentHit,
    /// A hit landing, on the player or an enemy, by what it lands on.
    Damage(DamageSound),
    Pickup,
    EnemyIdle(EnemySoundStyle),
    EnemyMovement(EnemySoundStyle),
    EnemyTelegraph(EnemySoundStyle),
    EnemyAttack(EnemySoundStyle),
    BossIntro,
    EnemiesAppear,
    EnemyDeath(EnemySoundStyle),
    BossPhase,
    RoomClear,
    /// The bonus item landing on a cleared room's bonus spot.
    BonusAppear,
    DoorAppear,
    DoorOpen,
    BonusSelected,
    BonusRerolled,
    RunEnded,
    RunRestarted,
    Victory,
}

/// What a hit sounds like, by what it lands on.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum DamageSound {
    /// The player, and enemies of polymer.
    Neutral,
    /// A metal enemy whose shield is already gone.
    Metal,
    /// The shield of a metal enemy, as the hit breaks it.
    MetalArmor,
}

impl DamageSound {
    pub(super) fn of(material: EnemyMaterial, shielded: bool) -> Self {
        match (material, shielded) {
            (EnemyMaterial::Metal, true) => Self::MetalArmor,
            (EnemyMaterial::Metal, false) => Self::Metal,
            (EnemyMaterial::Polymer, _) => Self::Neutral,
        }
    }
}

/// Which enemy a cue came from, kept so enemy sounds can differ once they are
/// recorded; no cue of a style has a recording yet.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum EnemySoundStyle {
    Generic,
    BladeUnit,
    RifleUnit,
    HeavyUnit,
    StalkerUnit,
    LancerUnit,
    BreakerUnit,
    EmitterUnit,
}

impl From<&EnemyTypeId> for EnemySoundStyle {
    fn from(type_id: &EnemyTypeId) -> Self {
        match &**type_id {
            "blade_unit" => Self::BladeUnit,
            "rifle_unit" => Self::RifleUnit,
            "heavy_unit" => Self::HeavyUnit,
            "stalker_unit" => Self::StalkerUnit,
            "lancer_unit" => Self::LancerUnit,
            "breaker_unit" => Self::BreakerUnit,
            "emitter_unit" => Self::EmitterUnit,
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
    music_tracks: Vec<Layer>,
    /// The music the game opens on, and the city under the whole run.
    intro_music: Option<Layer>,
    ambience: Option<Layer>,
    /// The typing heard under the prologue's words.
    typing: Option<(Handle<AudioSource>, f32)>,
}

/// A looping background layer: the file it plays and how loud it plays it.
type Layer = (Handle<MusicStream>, f32);

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum AudioClass {
    /// The music the game opens on, from launch until a run leaves room 0.
    IntroMusic,
    /// The background track picked for the run.
    Music,
    /// The city heard under a run from end to end.
    Ambience,
    GameplayEffect,
    InterfaceEffect,
    TerminalEffect,
}

#[derive(Component)]
pub(super) struct ManagedAudio {
    pub(super) class: AudioClass,
    gain: f32,
}

pub(super) fn initialize_sound_bank(
    mut bank: ResMut<SoundBank>,
    asset_server: Res<AssetServer>,
    data: Res<GameData>,
) {
    let recorded = &data.gameplay.audio.recorded_effects;
    for (cue, effect) in [
        (SoundCue::DoorOpen, &recorded.door_open),
        (
            SoundCue::Damage(DamageSound::Neutral),
            &recorded.damage_neutral,
        ),
        (SoundCue::Damage(DamageSound::Metal), &recorded.damage_metal),
        (
            SoundCue::Damage(DamageSound::MetalArmor),
            &recorded.damage_metal_armor,
        ),
        (SoundCue::BonusSelected, &recorded.bonus_chosen),
        (SoundCue::BonusRerolled, &recorded.bonus_reroll),
        (SoundCue::MeleeMiss, &recorded.punch_missed),
    ]
    .into_iter()
    .chain(
        recorded
            .punch_hits
            .iter()
            .enumerate()
            .map(|(punch, effect)| (SoundCue::MeleeHit(punch), effect)),
    ) {
        bank.cues
            .insert(cue, (asset_server.load(effect.asset.clone()), effect.gain));
    }
    let audio = &data.gameplay.audio;
    let layer =
        |sound: &crate::data::AudioAsset| (asset_server.load(sound.asset.clone()), sound.gain);
    bank.music_tracks = audio.music_tracks.iter().map(layer).collect();
    bank.intro_music = Some(layer(&audio.intro_music));
    bank.ambience = Some(layer(&audio.ambience));
    let typing = &data.gameplay.prologue.typing_sound;
    bank.typing = Some((asset_server.load(typing.asset.clone()), typing.gain));
}

/// The typing heard while the prologue writes itself out.
#[derive(Component)]
pub(super) struct TypingSound;

/// Starts that typing: a loop that runs for as long as the words take, rather
/// than a cue fired once.
pub(super) fn start_typing(commands: &mut Commands, bank: &SoundBank, settings: &AudioSettings) {
    let Some((source, gain)) = bank.typing.as_ref() else {
        return;
    };
    commands.spawn((
        AudioPlayer::new(source.clone()),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(
            settings.sfx_volume_percent as f32 / 100.0 * gain,
        )),
        ManagedAudio {
            class: AudioClass::GameplayEffect,
            gain: *gain,
        },
        TypingSound,
    ));
}

/// Lays down a looping background layer. `MusicStream` loops itself, so that
/// the filter the mixer runs it through keeps working past the first
/// play-through; Bevy's own looping would buffer the decoded source and freeze
/// it.
fn play_layer(
    commands: &mut Commands,
    (source, gain): &Layer,
    class: AudioClass,
    settings: &AudioSettings,
    fade: Option<AudioFade>,
) {
    let mut layer = commands.spawn((
        AudioPlayer(source.clone()),
        PlaybackSettings::ONCE.with_volume(Volume::Linear(
            settings.music_volume_percent as f32 / 100.0 * gain,
        )),
        ManagedAudio { class, gain: *gain },
    ));
    if let Some(fade) = fade {
        layer.insert(fade);
    }
}

/// Starts the music the game opens on: it plays from launch, through the main
/// menu and the opening room, until a run crosses into room 1 and a chapter
/// track takes over. Returning to the main menu starts it again.
pub(super) fn start_intro_music(
    mut commands: Commands,
    bank: Res<SoundBank>,
    settings: Res<AudioSettings>,
    active_audio: Query<&ManagedAudio>,
) {
    let Some(intro) = bank.intro_music.as_ref() else {
        return;
    };
    if active_audio
        .iter()
        .any(|audio| audio.class == AudioClass::IntroMusic)
    {
        return;
    }
    play_layer(
        &mut commands,
        intro,
        AudioClass::IntroMusic,
        &settings,
        None,
    );
}

/// Opens the city under the run, on the first room it enters: it rises over
/// `audio.ambience_fade_in_s` and stays until the run ends.
pub(super) fn start_run_ambience(
    mut commands: Commands,
    bank: Res<SoundBank>,
    data: Res<GameData>,
    settings: Res<AudioSettings>,
    active_audio: Query<&ManagedAudio>,
) {
    let Some(ambience) = bank.ambience.as_ref() else {
        return;
    };
    if active_audio
        .iter()
        .any(|audio| audio.class == AudioClass::Ambience)
    {
        return;
    }
    play_layer(
        &mut commands,
        ambience,
        AudioClass::Ambience,
        &settings,
        Some(AudioFade::in_over(data.gameplay.audio.ambience_fade_in_s)),
    );
}

/// Sends the opening music away over `audio.intro_music_fade_out_s`: the run
/// is leaving the opening room behind, and a chapter track is about to stand
/// in its place.
pub(super) fn fade_out_intro_music(
    mut commands: Commands,
    data: Res<GameData>,
    session: Res<RunSession>,
    active_audio: Query<(Entity, &ManagedAudio)>,
) {
    // Only the opening room hands over to a chapter track this way; every
    // other door leads from one chapter track to the same one.
    if session.room_id != 0 {
        return;
    }
    fade_intro_music_out(&mut commands, &data, &active_audio);
}

fn fade_intro_music_out(
    commands: &mut Commands,
    data: &GameData,
    active_audio: &Query<(Entity, &ManagedAudio)>,
) {
    for (entity, audio) in active_audio {
        if audio.class == AudioClass::IntroMusic {
            commands.entity(entity).insert(AudioFade::out_over(
                data.gameplay.audio.intro_music_fade_out_s,
            ));
        }
    }
}

pub(super) fn queue_victory_cue(mut queue: ResMut<SoundQueue>) {
    queue.push(SoundCue::Victory);
}

pub(super) fn queue_boss_intro(mut queue: ResMut<SoundQueue>) {
    queue.push(SoundCue::BossIntro);
}

/// Starts the music of a run, on the first room it enters that belongs to a
/// chapter. Room 0 is left to the opening music, which bows out as the run
/// crosses into room 1; later rooms find the track already playing and leave
/// it alone, so one track plays to the end of the run.
pub(super) fn start_chapter_music(
    mut commands: Commands,
    bank: Res<SoundBank>,
    data: Res<GameData>,
    session: Res<RunSession>,
    settings: Res<AudioSettings>,
    mut random: ResMut<RandomState>,
    active_audio: Query<(Entity, &ManagedAudio)>,
) {
    if session.room_id == 0
        || active_audio
            .iter()
            .any(|(_, audio)| audio.class == AudioClass::Music)
        || bank.music_tracks.is_empty()
    {
        return;
    }
    let index = random.index(bank.music_tracks.len());
    let Some(track) = bank.music_tracks.get(index) else {
        return;
    };
    play_layer(&mut commands, track, AudioClass::Music, &settings, None);
    // A run started past the opening room (`--room 7`) never crossed the door
    // that would have sent the opening music away; it goes now.
    fade_intro_music_out(&mut commands, &data, &active_audio);
}

/// Takes down what a run was playing: its chapter track and the city under
/// it. The opening music is not a run's to stop.
pub(super) fn stop_run_audio(mut commands: Commands, active_audio: Query<(Entity, &ManagedAudio)>) {
    for (entity, audio) in &active_audio {
        if matches!(audio.class, AudioClass::Music | AudioClass::Ambience) {
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

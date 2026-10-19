use super::*;
use bevy::audio::{
    AudioPlayer, AudioSink, AudioSinkPlayback, AudioSource, PlaybackSettings, Volume,
};
use std::sync::Arc;

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
    PlayerHurt,
    Pickup,
    EnemyAttack,
    BossIntro,
    BossAttack,
    EnemyHurt,
    EnemyDeath,
    BossPhase,
    RoomClear,
    DoorAppear,
    DoorOpen,
    BonusSelected,
    RunEnded,
    Victory,
}

#[derive(Resource, Default)]
pub(super) struct SoundBank {
    cues: BTreeMap<SoundCue, (Handle<AudioSource>, f32)>,
    chapter_one_music: Option<Handle<AudioSource>>,
}

#[derive(Component, Clone, Copy)]
enum AudioChannel {
    Music,
    Effects,
}

#[derive(Component)]
pub(super) struct ManagedAudio {
    channel: AudioChannel,
    gain: f32,
}

pub(super) fn initialize_sound_bank(
    mut bank: ResMut<SoundBank>,
    mut audio_sources: ResMut<Assets<AudioSource>>,
) {
    for (cue, start_hz, end_hz, seconds, gain) in cue_specs() {
        let source = audio_sources.add(make_tone(start_hz, end_hz, seconds, 0.8));
        bank.cues.insert(cue, (source, gain));
    }
    bank.chapter_one_music = Some(audio_sources.add(make_ambient_loop()));
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
        .any(|audio| matches!(audio.channel, AudioChannel::Music))
    {
        return;
    }
    let Some(source) = bank.chapter_one_music.clone() else {
        return;
    };
    let gain = data.gameplay.audio.chapter_one_music_gain;
    commands.spawn((
        AudioPlayer::new(source),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(
            settings.music_volume_percent as f32 / 100.0 * gain,
        )),
        ManagedAudio {
            channel: AudioChannel::Music,
            gain,
        },
    ));
}

pub(super) fn stop_chapter_music(
    mut commands: Commands,
    active_audio: Query<(Entity, &ManagedAudio)>,
) {
    for (entity, audio) in &active_audio {
        if matches!(audio.channel, AudioChannel::Music) {
            commands.entity(entity).despawn();
        }
    }
}

pub(super) fn play_sound_queue(
    mut commands: Commands,
    mut queue: ResMut<SoundQueue>,
    bank: Res<SoundBank>,
    settings: Res<AudioSettings>,
) {
    for cue in queue.0.drain(..) {
        let Some((source, gain)) = bank.cues.get(&cue) else {
            continue;
        };
        commands.spawn((
            AudioPlayer::new(source.clone()),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(
                settings.sfx_volume_percent as f32 / 100.0 * gain,
            )),
            ManagedAudio {
                channel: AudioChannel::Effects,
                gain: *gain,
            },
        ));
    }
}

pub(super) fn update_audio_volume(
    time: Res<Time>,
    screen: Res<State<GameScreen>>,
    data: Res<GameData>,
    settings: Res<AudioSettings>,
    mut music_mix: Local<Option<f32>>,
    mut active_audio: Query<(&ManagedAudio, Option<&mut AudioSink>)>,
) {
    let target_mix = match screen.get() {
        GameScreen::Paused | GameScreen::Settings => {
            data.gameplay.audio.paused_music_volume_multiplier
        }
        GameScreen::RoomClear | GameScreen::Bonus | GameScreen::BossIntro => {
            data.gameplay.audio.room_clear_music_volume_multiplier
        }
        _ => 1.0,
    };
    let previous_mix = *music_mix;
    let current_mix = previous_mix.unwrap_or(target_mix);
    let next_mix = move_towards(current_mix, target_mix, time.delta_secs() * 4.0);
    *music_mix = Some(next_mix);
    let mix_changed = previous_mix.is_none_or(|previous| (previous - next_mix).abs() > 0.0001);
    if !settings.is_changed() && !mix_changed {
        return;
    }
    for (audio, sink) in &mut active_audio {
        let Some(mut sink) = sink else {
            continue;
        };
        let (channel_volume, mix) = match audio.channel {
            AudioChannel::Music => (settings.music_volume_percent, next_mix),
            AudioChannel::Effects => (settings.sfx_volume_percent, 1.0),
        };
        sink.set_volume(Volume::Linear(
            channel_volume as f32 / 100.0 * audio.gain * mix,
        ));
    }
}

fn cue_specs() -> [(SoundCue, f32, f32, f32, f32); 26] {
    use SoundCue::*;
    [
        (UiMove, 720.0, 920.0, 0.045, 0.08),
        (UiConfirm, 660.0, 990.0, 0.11, 0.11),
        (UiCancel, 440.0, 300.0, 0.09, 0.10),
        (UiInvalid, 220.0, 180.0, 0.10, 0.10),
        (Jump, 360.0, 590.0, 0.12, 0.13),
        (Landing, 120.0, 80.0, 0.09, 0.12),
        (Footstep, 110.0, 75.0, 0.07, 0.055),
        (Dash, 180.0, 90.0, 0.18, 0.18),
        (SwordSwing, 540.0, 190.0, 0.13, 0.12),
        (SwordHit, 920.0, 380.0, 0.12, 0.18),
        (Gunshot, 150.0, 70.0, 0.09, 0.20),
        (BulletHit, 510.0, 260.0, 0.08, 0.12),
        (PlayerHurt, 190.0, 90.0, 0.24, 0.25),
        (Pickup, 620.0, 1040.0, 0.20, 0.16),
        (EnemyAttack, 360.0, 420.0, 0.15, 0.12),
        (BossIntro, 72.0, 220.0, 0.9, 0.3),
        (BossAttack, 100.0, 65.0, 0.24, 0.25),
        (EnemyHurt, 280.0, 190.0, 0.10, 0.12),
        (EnemyDeath, 300.0, 110.0, 0.20, 0.16),
        (BossPhase, 110.0, 220.0, 0.55, 0.22),
        (RoomClear, 440.0, 880.0, 0.45, 0.19),
        (DoorAppear, 260.0, 440.0, 0.14, 0.10),
        (DoorOpen, 310.0, 660.0, 0.28, 0.17),
        (BonusSelected, 740.0, 1110.0, 0.18, 0.16),
        (RunEnded, 310.0, 70.0, 0.75, 0.22),
        (Victory, 520.0, 1040.0, 1.1, 0.20),
    ]
}

fn make_tone(start_hz: f32, end_hz: f32, seconds: f32, amplitude: f32) -> AudioSource {
    const SAMPLE_RATE: u32 = 22_050;
    let sample_count = (SAMPLE_RATE as f32 * seconds).max(1.0) as usize;
    let mut samples = Vec::with_capacity(sample_count * 2);
    let mut phase = 0.0_f32;
    for index in 0..sample_count {
        let progress = index as f32 / sample_count as f32;
        let frequency = start_hz + (end_hz - start_hz) * progress;
        phase += std::f32::consts::TAU * frequency / SAMPLE_RATE as f32;
        let attack = (progress / 0.08).clamp(0.0, 1.0);
        let release = ((1.0 - progress) / 0.28).clamp(0.0, 1.0);
        let value = (phase.sin() * attack.min(release) * amplitude * f32::from(i16::MAX)) as i16;
        samples.extend_from_slice(&value.to_le_bytes());
    }
    AudioSource {
        bytes: Arc::from(wav_header_and_data(SAMPLE_RATE, 1, 16, &samples)),
    }
}

fn make_ambient_loop() -> AudioSource {
    const SAMPLE_RATE: u32 = 22_050;
    const SECONDS: f32 = 8.0;
    let sample_count = (SAMPLE_RATE as f32 * SECONDS) as usize;
    let mut samples = Vec::with_capacity(sample_count * 2);
    for index in 0..sample_count {
        let progress = index as f32 / sample_count as f32;
        let t = index as f32 / SAMPLE_RATE as f32;
        let chord = (std::f32::consts::TAU * 82.41 * t).sin() * 0.38
            + (std::f32::consts::TAU * 110.0 * t).sin() * 0.24
            + (std::f32::consts::TAU * 164.81 * t).sin() * 0.12;
        let pulse = 0.72 + 0.28 * (std::f32::consts::TAU * 0.125 * t).sin();
        let loop_fade = (progress.min(1.0 - progress) / 0.02).clamp(0.0, 1.0);
        let value = (chord * pulse * loop_fade * 0.045 * f32::from(i16::MAX)) as i16;
        samples.extend_from_slice(&value.to_le_bytes());
    }
    AudioSource {
        bytes: Arc::from(wav_header_and_data(SAMPLE_RATE, 1, 16, &samples)),
    }
}

fn wav_header_and_data(
    sample_rate: u32,
    channels: u16,
    bits_per_sample: u16,
    data: &[u8],
) -> Vec<u8> {
    let data_len = data.len() as u32;
    let bytes_per_sample = u32::from(bits_per_sample / 8);
    let byte_rate = sample_rate * u32::from(channels) * bytes_per_sample;
    let block_align = channels * (bits_per_sample / 8);
    let mut wav = Vec::with_capacity(44 + data.len());
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_len).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16_u32.to_le_bytes());
    wav.extend_from_slice(&1_u16.to_le_bytes());
    wav.extend_from_slice(&channels.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&bits_per_sample.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_len.to_le_bytes());
    wav.extend_from_slice(data);
    wav
}

use super::{EnemySoundStyle, SoundCue};
use crate::data::PlatformMaterial;
use bevy::audio::AudioSource;
use std::sync::Arc;

pub(super) fn cue_specs() -> Vec<(SoundCue, f32, f32, f32, f32)> {
    use SoundCue::*;

    let mut specs = vec![
        (UiMove, 720.0, 920.0, 0.045, 0.08),
        (UiConfirm, 660.0, 990.0, 0.11, 0.11),
        (UiCancel, 440.0, 300.0, 0.09, 0.10),
        (UiInvalid, 220.0, 180.0, 0.10, 0.10),
        (Jump, 360.0, 590.0, 0.12, 0.13),
        (Landing, 120.0, 80.0, 0.09, 0.12),
        (Footstep(PlatformMaterial::Soil), 90.0, 58.0, 0.07, 0.055),
        (Footstep(PlatformMaterial::Stone), 240.0, 390.0, 0.055, 0.05),
        (Footstep(PlatformMaterial::Wood), 160.0, 210.0, 0.06, 0.05),
        (Dash, 180.0, 90.0, 0.18, 0.18),
        (SwordSwing, 540.0, 190.0, 0.13, 0.12),
        (SwordHit, 920.0, 380.0, 0.12, 0.18),
        (SwordFinisherHit, 1180.0, 300.0, 0.2, 0.22),
        (ProjectileEnvironmentHit, 230.0, 95.0, 0.075, 0.085),
        (PlayerHurt, 190.0, 90.0, 0.24, 0.25),
        (Pickup, 620.0, 1040.0, 0.20, 0.16),
        (BossIntro, 72.0, 220.0, 0.9, 0.3),
        (EnemiesAppear, 140.0, 460.0, 0.38, 0.13),
        (BossPhase, 110.0, 220.0, 0.55, 0.22),
        (RoomClear, 440.0, 880.0, 0.45, 0.19),
        (DoorAppear, 260.0, 440.0, 0.14, 0.10),
        (DoorOpen, 310.0, 660.0, 0.28, 0.17),
        (BonusSelected, 740.0, 1110.0, 0.18, 0.16),
        (RunEnded, 310.0, 70.0, 0.75, 0.22),
        (RunRestarted, 350.0, 780.0, 0.32, 0.20),
        (Victory, 520.0, 1040.0, 1.1, 0.20),
    ];

    for style in EnemySoundStyle::ALL {
        let base = style.base_frequency_hz();
        let telegraph_gain = if style == EnemySoundStyle::EvilWizard {
            0.30
        } else {
            0.14
        };
        specs.extend([
            (EnemyIdle(style), base * 0.52, base * 0.68, 0.08, 0.035),
            (EnemyMovement(style), base * 0.46, base * 0.32, 0.055, 0.04),
            (
                EnemyTelegraph(style),
                base * 0.68,
                base * 1.8,
                0.20,
                telegraph_gain,
            ),
            (EnemyAttack(style), base * 1.7, base * 0.62, 0.12, 0.18),
            (EnemyHurt(style), base * 1.3, base * 0.42, 0.11, 0.14),
            (EnemyDeath(style), base * 1.45, base * 0.28, 0.23, 0.17),
        ]);
    }

    specs
}

pub(super) struct ToneSpec {
    pub start_hz: f32,
    pub end_hz: f32,
    pub seconds: f32,
    pub amplitude: f32,
}

impl From<ToneSpec> for AudioSource {
    fn from(spec: ToneSpec) -> Self {
        const SAMPLE_RATE: u32 = 22_050;
        let sample_count = (SAMPLE_RATE as f32 * spec.seconds).max(1.0) as usize;
        let mut samples = Vec::with_capacity(sample_count * 2);
        let mut phase = 0.0_f32;
        for index in 0..sample_count {
            let progress = index as f32 / sample_count as f32;
            let frequency = spec.start_hz + (spec.end_hz - spec.start_hz) * progress;
            phase += std::f32::consts::TAU * frequency / SAMPLE_RATE as f32;
            let attack = (progress / 0.08).clamp(0.0, 1.0);
            let release = ((1.0 - progress) / 0.28).clamp(0.0, 1.0);
            let value =
                (phase.sin() * attack.min(release) * spec.amplitude * f32::from(i16::MAX)) as i16;
            samples.extend_from_slice(&value.to_le_bytes());
        }
        Self {
            bytes: Arc::from(wav_header_and_data(SAMPLE_RATE, 1, 16, &samples)),
        }
    }
}

pub(super) struct ChapterOneForestLoop;

impl From<ChapterOneForestLoop> for AudioSource {
    fn from(_: ChapterOneForestLoop) -> Self {
        const SAMPLE_RATE: u32 = 22_050;
        const SECONDS: f32 = 8.0;
        const NOTE_SECONDS: f32 = 0.25;
        const MELODY: [usize; 32] = [
            0, 2, 3, 1, 4, 3, 2, 1, 0, 2, 4, 3, 1, 2, 3, 4, 2, 3, 4, 2, 1, 2, 3, 4, 0, 1, 3, 2, 4,
            3, 2, 0,
        ];
        const PENTATONIC_RATIOS: [f32; 5] = [1.0, 1.2, 1.333_333_4, 1.5, 1.777_777_8];
        const BAR_ROOTS_HZ: [f32; 4] = [110.0, 87.31, 98.0, 110.0];

        let sample_count = (SAMPLE_RATE as f32 * SECONDS) as usize;
        let mut samples = Vec::with_capacity(sample_count * 2);
        for index in 0..sample_count {
            let progress = index as f32 / sample_count as f32;
            let t = index as f32 / SAMPLE_RATE as f32;
            let bar = (t / 2.0) as usize % BAR_ROOTS_HZ.len();
            let root_hz = BAR_ROOTS_HZ[bar];
            let chord = (std::f32::consts::TAU * root_hz * t).sin() * 0.24
                + (std::f32::consts::TAU * root_hz * 1.2 * t).sin() * 0.12
                + (std::f32::consts::TAU * root_hz * 1.5 * t).sin() * 0.08;

            let note_index = (t / NOTE_SECONDS) as usize % MELODY.len();
            let note_time = t % NOTE_SECONDS;
            let note_hz = root_hz * PENTATONIC_RATIOS[MELODY[note_index]];
            let note_envelope = (note_time / 0.015).clamp(0.0, 1.0)
                * ((NOTE_SECONDS - note_time) / 0.12).clamp(0.0, 1.0);
            let melody = (std::f32::consts::TAU * note_hz * note_time).sin() * note_envelope * 0.22;

            let beat_time = t % 0.5;
            let kick_frequency = 55.0 + (1.0 - beat_time / 0.5) * 35.0;
            let kick = (std::f32::consts::TAU * kick_frequency * beat_time).sin()
                * (-beat_time * 18.0).exp()
                * 0.16;
            let noise = (index.wrapping_mul(1_103_515_245).wrapping_add(12_345) >> 16 & 0x7fff)
                as f32
                / 16_384.0
                - 1.0;
            let offbeat_time = beat_time - 0.25;
            let hi_hat = if offbeat_time >= 0.0 {
                noise * (-offbeat_time * 55.0).exp() * 0.025
            } else {
                0.0
            };

            let loop_fade = (progress.min(1.0 - progress) / 0.005).clamp(0.0, 1.0);
            let value =
                ((chord + melody + kick + hi_hat) * loop_fade * 0.18 * f32::from(i16::MAX)) as i16;
            samples.extend_from_slice(&value.to_le_bytes());
        }
        Self {
            bytes: Arc::from(wav_header_and_data(SAMPLE_RATE, 1, 16, &samples)),
        }
    }
}

pub(super) struct ChapterOneForestCalmLoop;

impl From<ChapterOneForestCalmLoop> for AudioSource {
    fn from(_: ChapterOneForestCalmLoop) -> Self {
        const SAMPLE_RATE: u32 = 22_050;
        const SECONDS: f32 = 8.0;
        const MELODY: [usize; 8] = [0, 2, 3, 2, 4, 3, 2, 0];
        const PENTATONIC_RATIOS: [f32; 5] = [1.0, 1.2, 1.333_333_4, 1.5, 1.777_777_8];
        const BAR_ROOTS_HZ: [f32; 4] = [110.0, 87.31, 98.0, 110.0];

        let sample_count = (SAMPLE_RATE as f32 * SECONDS) as usize;
        let mut samples = Vec::with_capacity(sample_count * 2);
        for index in 0..sample_count {
            let progress = index as f32 / sample_count as f32;
            let t = index as f32 / SAMPLE_RATE as f32;
            let bar = (t / 2.0) as usize % BAR_ROOTS_HZ.len();
            let root_hz = BAR_ROOTS_HZ[bar];
            let chord = (std::f32::consts::TAU * root_hz * t).sin() * 0.28
                + (std::f32::consts::TAU * root_hz * 1.2 * t).sin() * 0.15
                + (std::f32::consts::TAU * root_hz * 1.5 * t).sin() * 0.1;
            let note_index = (t as usize) % MELODY.len();
            let note_time = t % 1.0;
            let note_hz = root_hz * PENTATONIC_RATIOS[MELODY[note_index]];
            let note_envelope =
                (note_time / 0.04).clamp(0.0, 1.0) * ((1.0 - note_time) / 0.4).clamp(0.0, 1.0);
            let melody = (std::f32::consts::TAU * note_hz * note_time).sin() * note_envelope * 0.12;
            let wind = (std::f32::consts::TAU * 0.17 * t).sin() * 0.04;
            let loop_fade = (progress.min(1.0 - progress) / 0.005).clamp(0.0, 1.0);
            let value = ((chord + melody + wind) * loop_fade * 0.16 * f32::from(i16::MAX)) as i16;
            samples.extend_from_slice(&value.to_le_bytes());
        }
        Self {
            bytes: Arc::from(wav_header_and_data(SAMPLE_RATE, 1, 16, &samples)),
        }
    }
}

pub(super) struct ChapterOneForestAmbience;

impl From<ChapterOneForestAmbience> for AudioSource {
    fn from(_: ChapterOneForestAmbience) -> Self {
        const SAMPLE_RATE: u32 = 22_050;
        const SECONDS: f32 = 8.0;

        let sample_count = (SAMPLE_RATE as f32 * SECONDS) as usize;
        let mut samples = Vec::with_capacity(sample_count * 2);
        let mut filtered_noise = 0.0_f32;
        for index in 0..sample_count {
            let progress = index as f32 / sample_count as f32;
            let t = index as f32 / SAMPLE_RATE as f32;
            let noise = (index.wrapping_mul(1_103_515_245).wrapping_add(12_345) >> 16 & 0x7fff)
                as f32
                / 16_384.0
                - 1.0;
            filtered_noise += (noise - filtered_noise) * 0.045;
            let swell = 0.72 + (std::f32::consts::TAU * 0.125 * t).sin() * 0.2;
            let low_wind = (std::f32::consts::TAU * 0.19 * t).sin() * 0.08;
            let loop_fade = (progress.min(1.0 - progress) / 0.01).clamp(0.0, 1.0);
            let value =
                ((filtered_noise * swell + low_wind) * loop_fade * 0.06 * f32::from(i16::MAX))
                    as i16;
            samples.extend_from_slice(&value.to_le_bytes());
        }

        Self {
            bytes: Arc::from(wav_header_and_data(SAMPLE_RATE, 1, 16, &samples)),
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn enemies_and_special_events_have_placeholder_cues() {
        let cues = cue_specs()
            .into_iter()
            .map(|(cue, ..)| cue)
            .collect::<BTreeSet<_>>();

        for style in EnemySoundStyle::ALL {
            assert!(cues.contains(&SoundCue::EnemyIdle(style)));
            assert!(cues.contains(&SoundCue::EnemyMovement(style)));
            assert!(cues.contains(&SoundCue::EnemyTelegraph(style)));
            assert!(cues.contains(&SoundCue::EnemyAttack(style)));
            assert!(cues.contains(&SoundCue::EnemyHurt(style)));
            assert!(cues.contains(&SoundCue::EnemyDeath(style)));
        }
        assert!(cues.contains(&SoundCue::Footstep(PlatformMaterial::Soil)));
        assert!(cues.contains(&SoundCue::Footstep(PlatformMaterial::Stone)));
        assert!(cues.contains(&SoundCue::Footstep(PlatformMaterial::Wood)));
        assert!(cues.contains(&SoundCue::RunRestarted));
    }

    #[test]
    fn generated_forest_ambience_is_an_eight_second_wav_loop() {
        let source = AudioSource::from(ChapterOneForestAmbience);

        assert_eq!(&source.bytes[..4], b"RIFF");
        assert_eq!(source.bytes.len(), 44 + 22_050 * 8 * 2);
    }
}

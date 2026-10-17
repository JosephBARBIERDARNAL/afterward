use bevy::audio::Volume;
use bevy::prelude::*;

#[derive(Resource)]
pub(crate) struct HitSounds {
    pub(crate) enemy: Handle<AudioSource>,
    pub(crate) player: Handle<AudioSource>,
}

pub(crate) fn setup_hit_sounds(mut commands: Commands, mut sources: ResMut<Assets<AudioSource>>) {
    commands.insert_resource(HitSounds {
        enemy: sources.add(AudioSource {
            bytes: impact_wave(182.0, 72.0, 0.18, 0.32).into(),
        }),
        player: sources.add(AudioSource {
            bytes: impact_wave(118.0, 42.0, 0.23, 0.24).into(),
        }),
    });
}

pub(crate) fn play_hit_sound(commands: &mut Commands, sound: &Handle<AudioSource>, volume: f32) {
    commands.spawn((
        AudioPlayer::new(sound.clone()),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(volume)),
    ));
}

fn impact_wave(
    start_frequency: f32,
    end_frequency: f32,
    duration: f32,
    noise_level: f32,
) -> Vec<u8> {
    const SAMPLE_RATE: u32 = 22_050;
    let sample_count = (duration * SAMPLE_RATE as f32) as usize;
    let data_size = (sample_count * 2) as u32;
    let mut wav = Vec::with_capacity(44 + data_size as usize);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_size).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16_u32.to_le_bytes());
    wav.extend_from_slice(&1_u16.to_le_bytes());
    wav.extend_from_slice(&1_u16.to_le_bytes());
    wav.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    wav.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes());
    wav.extend_from_slice(&2_u16.to_le_bytes());
    wav.extend_from_slice(&16_u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_size.to_le_bytes());

    let mut phase = 0.0;
    let mut noise_state = 0xA341_316C_u32;
    for sample_index in 0..sample_count {
        let time = sample_index as f32 / SAMPLE_RATE as f32;
        let progress = time / duration;
        let frequency = start_frequency * (end_frequency / start_frequency).powf(progress);
        phase += std::f32::consts::TAU * frequency / SAMPLE_RATE as f32;
        let envelope = (1.0 - progress).powf(1.4) * (-time * 7.0).exp();
        noise_state = noise_state
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        let noise = noise_state as f32 / u32::MAX as f32 * 2.0 - 1.0;
        let noise_envelope = (1.0 - time / 0.04).clamp(0.0, 1.0);
        let tick = (std::f32::consts::TAU * 1_180.0 * time).sin() * (-time * 100.0).exp();
        let value = ((phase.sin() * envelope * 0.82)
            + (noise * noise_envelope * noise_level)
            + (tick * 0.10))
            .clamp(-1.0, 1.0);
        wav.extend_from_slice(&((value * i16::MAX as f32) as i16).to_le_bytes());
    }
    wav
}

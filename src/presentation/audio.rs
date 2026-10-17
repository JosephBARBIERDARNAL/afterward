use bevy::audio::Volume;
use bevy::prelude::*;

#[derive(Resource)]
pub(crate) struct GameSounds {
    primary: Handle<AudioSource>,
    heavy: Handle<AudioSource>,
    dash: Handle<AudioSource>,
    dash_burst: Handle<AudioSource>,
    combo_burst: Handle<AudioSource>,
    player_hit: Handle<AudioSource>,
    enemy_hit: Handle<AudioSource>,
    enemy_down: Handle<AudioSource>,
    death: Handle<AudioSource>,
    clear: Handle<AudioSource>,
}

#[derive(Clone, Copy)]
pub(crate) enum SoundCue {
    Primary,
    Heavy,
    Dash,
    DashBurst,
    ComboBurst,
    PlayerHit,
    EnemyHit,
    EnemyDown,
    Death,
    Clear,
}

impl GameSounds {
    fn source(&self, cue: SoundCue) -> (&Handle<AudioSource>, f32) {
        match cue {
            SoundCue::Primary => (&self.primary, 0.38),
            SoundCue::Heavy => (&self.heavy, 0.60),
            SoundCue::Dash => (&self.dash, 0.32),
            SoundCue::DashBurst => (&self.dash_burst, 0.72),
            SoundCue::ComboBurst => (&self.combo_burst, 0.70),
            SoundCue::PlayerHit => (&self.player_hit, 0.82),
            SoundCue::EnemyHit => (&self.enemy_hit, 0.86),
            SoundCue::EnemyDown => (&self.enemy_down, 0.50),
            SoundCue::Death => (&self.death, 0.72),
            SoundCue::Clear => (&self.clear, 0.45),
        }
    }
}

pub(crate) fn setup_game_sounds(mut commands: Commands, mut sources: ResMut<Assets<AudioSource>>) {
    commands.insert_resource(GameSounds {
        primary: add_sound(&mut sources, 240.0, 130.0, 0.09, 0.06),
        heavy: add_sound(&mut sources, 112.0, 52.0, 0.20, 0.10),
        dash: add_sound(&mut sources, 410.0, 170.0, 0.13, 0.03),
        dash_burst: add_sound(&mut sources, 180.0, 58.0, 0.20, 0.10),
        combo_burst: add_sound(&mut sources, 285.0, 92.0, 0.18, 0.08),
        player_hit: add_sound(&mut sources, 118.0, 42.0, 0.23, 0.24),
        enemy_hit: add_sound(&mut sources, 182.0, 72.0, 0.18, 0.32),
        enemy_down: add_sound(&mut sources, 165.0, 76.0, 0.26, 0.08),
        death: add_sound(&mut sources, 120.0, 38.0, 0.48, 0.12),
        clear: add_sound(&mut sources, 260.0, 520.0, 0.30, 0.02),
    });
}

pub(crate) fn play_sound(commands: &mut Commands, sounds: &GameSounds, cue: SoundCue) {
    let (source, volume) = sounds.source(cue);
    commands.spawn((
        AudioPlayer::new(source.clone()),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(volume)),
    ));
}

fn add_sound(
    sources: &mut Assets<AudioSource>,
    start_frequency: f32,
    end_frequency: f32,
    duration: f32,
    noise_level: f32,
) -> Handle<AudioSource> {
    sources.add(AudioSource {
        bytes: impact_wave(start_frequency, end_frequency, duration, noise_level).into(),
    })
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

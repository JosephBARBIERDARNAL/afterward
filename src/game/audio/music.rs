//! Looping music streams whose highs can be muffled while the game is paused,
//! so the music sounds like it keeps playing in another room.
//!
//! Bevy's own looping wraps the source in a buffer that is filled on the first
//! play-through and replayed afterwards, which would freeze the filter at
//! whatever it was doing the first time around. These streams therefore loop
//! themselves — and filter after looping — and play with `PlaybackMode::Once`.

use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    audio::{AudioSource, ChannelCount, Decodable, Sample, SampleRate, Source},
    prelude::*,
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
    time::Duration,
};

/// A music file that loops forever, with a low-pass filter the mixer can open
/// and close while it plays.
#[derive(Asset, Clone, TypePath)]
pub(crate) struct MusicStream {
    source: AudioSource,
    /// The bits of the current muffle amount, shared with every decoder of this
    /// stream: 0 leaves the sound untouched, 1 cuts the highs hardest.
    muffle: Arc<AtomicU32>,
}

impl MusicStream {
    pub(crate) fn set_muffle(&self, amount: f32) {
        self.muffle
            .store(amount.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
    }
}

impl From<AudioSource> for MusicStream {
    fn from(source: AudioSource) -> Self {
        Self {
            source,
            muffle: Arc::new(AtomicU32::new(0.0_f32.to_bits())),
        }
    }
}

impl Decodable for MusicStream {
    type Decoder = MuffledLoop;

    fn decoder(&self) -> Self::Decoder {
        MuffledLoop {
            decoder: self.source.decoder(),
            source: self.source.clone(),
            muffle: Arc::clone(&self.muffle),
            filtered: [0.0; MAX_CHANNELS],
            channel: 0,
        }
    }
}

/// Loads `.mp3` files as [`MusicStream`]s. Bevy's own loader still owns the
/// extension for [`AudioSource`]; loaders are picked by asset type first, so
/// the two live side by side.
#[derive(Default, TypePath)]
pub(crate) struct MusicStreamLoader;

impl AssetLoader for MusicStreamLoader {
    type Asset = MusicStream;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<MusicStream, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        Ok(MusicStream::from(AudioSource {
            bytes: bytes.into(),
        }))
    }

    fn extensions(&self) -> &[&str] {
        &["mp3"]
    }
}

/// How far a muffle amount of 1 closes the low-pass filter. The filter passes
/// everything through at 1 and keeps only the lowest rumble near 0, so the
/// strongest setting still leaves a little of the sound.
const STRONGEST_MUFFLE: f32 = 0.985;

/// Room for the channels of any file we are likely to be handed; a stream with
/// more channels than this plays unfiltered rather than not at all.
const MAX_CHANNELS: usize = 8;

/// A music file played end to end, over and over, through a one-pole low-pass
/// filter per channel.
pub(crate) struct MuffledLoop {
    decoder: <AudioSource as Decodable>::Decoder,
    source: AudioSource,
    muffle: Arc<AtomicU32>,
    filtered: [Sample; MAX_CHANNELS],
    channel: usize,
}

impl Iterator for MuffledLoop {
    type Item = Sample;

    fn next(&mut self) -> Option<Sample> {
        let sample = match self.decoder.next() {
            Some(sample) => sample,
            None => {
                self.decoder = self.source.decoder();
                self.channel = 0;
                self.decoder.next()?
            }
        };
        let channel = self.channel;
        self.channel = (channel + 1) % usize::from(self.decoder.channels().get());
        let Some(previous) = self.filtered.get_mut(channel) else {
            return Some(sample);
        };
        let muffle = f32::from_bits(self.muffle.load(Ordering::Relaxed)).clamp(0.0, 1.0);
        let smoothing = 1.0 - muffle * STRONGEST_MUFFLE;
        *previous += (sample - *previous) * smoothing;
        Some(*previous)
    }
}

impl Source for MuffledLoop {
    /// The stream never ends and always holds the same file, so its format
    /// never changes.
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> ChannelCount {
        self.decoder.channels()
    }

    fn sample_rate(&self) -> SampleRate {
        self.decoder.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::GameData;
    use std::path::Path;

    const TONE_SAMPLE_RATE: u32 = 22_050;
    /// The length of the tones below: 0.05 s, short enough to loop quickly.
    const TONE_SAMPLES: usize = (TONE_SAMPLE_RATE as f32 * 0.05) as usize;

    /// A test signal of `hz`, as the mono 16-bit WAV a decoder expects: the
    /// one thing these tests need that no recording gives them, a tone of a
    /// known frequency to hold the filter against. The game plays no
    /// generated audio.
    fn tone(hz: f32) -> AudioSource {
        let mut samples = Vec::with_capacity(TONE_SAMPLES * 2);
        for index in 0..TONE_SAMPLES {
            let seconds = index as f32 / TONE_SAMPLE_RATE as f32;
            let value =
                ((std::f32::consts::TAU * hz * seconds).sin() * 0.8 * f32::from(i16::MAX)) as i16;
            samples.extend_from_slice(&value.to_le_bytes());
        }
        let data_len = samples.len() as u32;
        let mut wav = Vec::with_capacity(44 + samples.len());
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&(36 + data_len).to_le_bytes());
        wav.extend_from_slice(b"WAVEfmt ");
        wav.extend_from_slice(&16_u32.to_le_bytes());
        wav.extend_from_slice(&1_u16.to_le_bytes()); // PCM
        wav.extend_from_slice(&1_u16.to_le_bytes()); // mono
        wav.extend_from_slice(&TONE_SAMPLE_RATE.to_le_bytes());
        wav.extend_from_slice(&(TONE_SAMPLE_RATE * 2).to_le_bytes()); // byte rate
        wav.extend_from_slice(&2_u16.to_le_bytes()); // block align
        wav.extend_from_slice(&16_u16.to_le_bytes()); // bits per sample
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&data_len.to_le_bytes());
        wav.extend_from_slice(&samples);
        AudioSource {
            bytes: Arc::from(wav),
        }
    }

    #[test]
    fn a_stream_starts_over_when_it_reaches_the_end_of_its_file() {
        let stream = MusicStream::from(tone(440.0));

        let played: Vec<Sample> = stream.decoder().take(TONE_SAMPLES * 3).collect();

        assert_eq!(played.len(), TONE_SAMPLES * 3, "the stream never runs out");
        assert!(
            played
                .iter()
                .skip(TONE_SAMPLES * 2)
                .any(|sample| sample.abs() > 0.01),
            "the third play-through is audible, not silence"
        );
    }

    #[test]
    fn muffling_a_stream_swallows_its_highs_while_it_keeps_playing() {
        let peak = |stream: &MusicStream| {
            stream
                .decoder()
                .take(TONE_SAMPLES)
                .fold(0.0_f32, |peak, sample| peak.max(sample.abs()))
        };

        let clear = MusicStream::from(tone(4000.0));
        let clear_peak = peak(&clear);
        clear.set_muffle(1.0);
        let muffled_peak = peak(&clear);

        assert!(clear_peak > 0.1, "the unmuffled tone plays at full volume");
        assert!(
            muffled_peak < clear_peak / 10.0,
            "muffling cuts the tone down ({muffled_peak} against {clear_peak})"
        );
        assert!(muffled_peak > 0.0, "the muffled tone is still playing");
    }

    /// Every looping layer: the chapter tracks, the music the game opens on,
    /// and the city under a run.
    #[test]
    fn every_background_layer_the_game_lists_decodes() {
        let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
        let data = GameData::load(&asset_root).expect("Chapter 1 data should load");
        let audio = &data.gameplay.audio;

        for track in audio
            .music_tracks
            .iter()
            .chain([&audio.intro_music, &audio.ambience])
        {
            let bytes = std::fs::read(asset_root.join(&track.asset))
                .unwrap_or_else(|error| panic!("{} should be readable: {error}", track.asset));
            let stream = MusicStream::from(AudioSource {
                bytes: bytes.into(),
            });

            let played = stream.decoder().take(10_000).count();

            assert_eq!(played, 10_000, "{} should decode", track.asset);
        }
    }
}

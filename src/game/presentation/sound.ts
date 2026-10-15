type Cue = 'primary' | 'heavy' | 'dash' | 'player-hit' | 'enemy-hit' | 'enemy-down' | 'death' | 'clear';

const CUES: Record<Cue, { frequency: number; endFrequency: number; duration: number; volume: number; wave: OscillatorType }> = {
  primary: { frequency: 240, endFrequency: 130, duration: 0.09, volume: 0.055, wave: 'triangle' },
  heavy: { frequency: 112, endFrequency: 52, duration: 0.2, volume: 0.09, wave: 'sawtooth' },
  dash: { frequency: 410, endFrequency: 170, duration: 0.13, volume: 0.045, wave: 'sine' },
  'player-hit': { frequency: 95, endFrequency: 42, duration: 0.21, volume: 0.11, wave: 'square' },
  'enemy-hit': { frequency: 330, endFrequency: 170, duration: 0.1, volume: 0.06, wave: 'triangle' },
  'enemy-down': { frequency: 165, endFrequency: 76, duration: 0.26, volume: 0.075, wave: 'triangle' },
  death: { frequency: 120, endFrequency: 38, duration: 0.48, volume: 0.11, wave: 'sawtooth' },
  clear: { frequency: 260, endFrequency: 520, duration: 0.3, volume: 0.065, wave: 'sine' },
};

export class Sound {
  private context: AudioContext | undefined;

  play(cue: Cue): void {
    if (typeof window === 'undefined' || !('AudioContext' in window)) return;
    this.context ??= new AudioContext();
    if (this.context.state === 'suspended') void this.context.resume();

    const preset = CUES[cue];
    const oscillator = this.context.createOscillator();
    const gain = this.context.createGain();
    const now = this.context.currentTime;
    oscillator.type = preset.wave;
    oscillator.frequency.setValueAtTime(preset.frequency, now);
    oscillator.frequency.exponentialRampToValueAtTime(preset.endFrequency, now + preset.duration);
    gain.gain.setValueAtTime(preset.volume, now);
    gain.gain.exponentialRampToValueAtTime(0.001, now + preset.duration);
    oscillator.connect(gain).connect(this.context.destination);
    oscillator.start(now);
    oscillator.stop(now + preset.duration);
  }

  destroy(): void {
    if (this.context && this.context.state !== 'closed') void this.context.close();
    this.context = undefined;
  }
}

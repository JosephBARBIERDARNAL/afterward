import Phaser from 'phaser';
import './style.css';
import { CombatScene } from './game/CombatScene';
import { HubScene } from './game/HubScene';

const game = new Phaser.Game({
  type: Phaser.AUTO,
  parent: 'game',
  width: 960,
  height: 540,
  backgroundColor: '#18191c',
  pixelArt: true,
  roundPixels: true,
  antialias: false,
  input: { gamepad: true, mouse: true, keyboard: true, touch: false },
  physics: {
    default: 'arcade',
    arcade: { gravity: { x: 0, y: 0 }, debug: false },
  },
  scale: {
    mode: Phaser.Scale.FIT,
    autoCenter: Phaser.Scale.CENTER_BOTH,
    width: 960,
    height: 540,
  },
  scene: [HubScene, CombatScene],
});

window.addEventListener('beforeunload', () => game.destroy(true));

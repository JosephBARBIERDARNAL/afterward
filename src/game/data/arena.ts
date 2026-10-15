import type { EnemyKind } from './balance';

export interface ArenaObstacle {
  x: number;
  y: number;
  width: number;
  height: number;
}

interface EnemySpawn {
  id: string;
  kind: EnemyKind;
  x: number;
  y: number;
}

interface ArenaRoom {
  id: string;
  title: string;
  playerStart: { x: number; y: number };
  enemies: EnemySpawn[];
  obstacles: ArenaObstacle[];
}

export const ARENA = {
  width: 960,
  height: 540,
  bounds: { x: 38, y: 38, width: 884, height: 464 },
  exit: { x: 902, y: 270, triggerX: 884, halfHeight: 42 },
} as const;

export const ROOMS: ArenaRoom[] = [
  {
    id: 'entry',
    title: 'THE ENTRY',
    playerStart: { x: 480, y: 292 },
    enemies: [
      { id: 'pursuer', kind: 'melee', x: 245, y: 190 },
      { id: 'slinger', kind: 'ranged', x: 725, y: 335 },
    ],
    obstacles: [
      { x: 342, y: 226, width: 94, height: 54 },
      { x: 625, y: 343, width: 106, height: 52 },
      { x: 748, y: 164, width: 44, height: 44 },
    ],
  },
  {
    id: 'crossing',
    title: 'THE CROSSING',
    playerStart: { x: 82, y: 270 },
    enemies: [
      { id: 'warden', kind: 'heavy', x: 570, y: 235 },
      { id: 'slinger', kind: 'ranged', x: 732, y: 376 },
    ],
    obstacles: [
      { x: 322, y: 163, width: 122, height: 46 },
      { x: 486, y: 270, width: 80, height: 94 },
      { x: 323, y: 377, width: 122, height: 46 },
    ],
  },
  {
    id: 'inner-seal',
    title: 'THE INNER SEAL',
    playerStart: { x: 82, y: 270 },
    enemies: [
      { id: 'pursuer', kind: 'melee', x: 408, y: 180 },
      { id: 'warden', kind: 'heavy', x: 646, y: 283 },
      { id: 'slinger', kind: 'ranged', x: 698, y: 392 },
    ],
    obstacles: [
      { x: 280, y: 270, width: 70, height: 120 },
      { x: 493, y: 143, width: 104, height: 46 },
      { x: 500, y: 392, width: 118, height: 46 },
      { x: 761, y: 169, width: 42, height: 42 },
    ],
  },
];

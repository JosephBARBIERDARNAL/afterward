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

export interface ArenaRoom {
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
    id: 'outer-walk',
    title: 'THE OUTER WALK',
    playerStart: { x: 82, y: 270 },
    enemies: [
      { id: 'pursuer-a', kind: 'melee', x: 494, y: 174 },
      { id: 'pursuer-b', kind: 'melee', x: 647, y: 383 },
      { id: 'slinger', kind: 'ranged', x: 757, y: 272 },
    ],
    obstacles: [
      { x: 302, y: 215, width: 74, height: 104 },
      { x: 510, y: 150, width: 124, height: 44 },
      { x: 638, y: 378, width: 86, height: 48 },
      { x: 789, y: 243, width: 44, height: 44 },
    ],
  },
  {
    id: 'crossing',
    title: 'THE CROSSING',
    playerStart: { x: 82, y: 270 },
    enemies: [
      { id: 'warden', kind: 'heavy', x: 570, y: 235 },
      { id: 'pursuer', kind: 'melee', x: 504, y: 380 },
      { id: 'slinger', kind: 'ranged', x: 732, y: 376 },
    ],
    obstacles: [
      { x: 322, y: 163, width: 122, height: 46 },
      { x: 486, y: 270, width: 80, height: 94 },
      { x: 323, y: 377, width: 122, height: 46 },
    ],
  },
  {
    id: 'lower-gallery',
    title: 'THE LOWER GALLERY',
    playerStart: { x: 82, y: 270 },
    enemies: [
      { id: 'pursuer-a', kind: 'melee', x: 444, y: 160 },
      { id: 'warden', kind: 'heavy', x: 626, y: 280 },
      { id: 'pursuer-b', kind: 'melee', x: 502, y: 397 },
      { id: 'slinger', kind: 'ranged', x: 762, y: 175 },
    ],
    obstacles: [
      { x: 301, y: 160, width: 86, height: 56 },
      { x: 421, y: 270, width: 74, height: 116 },
      { x: 531, y: 155, width: 90, height: 48 },
      { x: 546, y: 395, width: 122, height: 42 },
      { x: 773, y: 365, width: 48, height: 48 },
    ],
  },
  {
    id: 'inner-seal',
    title: 'THE INNER SEAL',
    playerStart: { x: 82, y: 270 },
    enemies: [
      { id: 'pursuer-a', kind: 'melee', x: 393, y: 184 },
      { id: 'slinger-a', kind: 'ranged', x: 375, y: 398 },
      { id: 'warden', kind: 'heavy', x: 646, y: 283 },
      { id: 'pursuer-b', kind: 'melee', x: 728, y: 155 },
      { id: 'slinger-b', kind: 'ranged', x: 744, y: 400 },
    ],
    obstacles: [
      { x: 280, y: 270, width: 70, height: 120 },
      { x: 493, y: 143, width: 104, height: 46 },
      { x: 500, y: 392, width: 118, height: 46 },
      { x: 761, y: 270, width: 42, height: 120 },
    ],
  },
];

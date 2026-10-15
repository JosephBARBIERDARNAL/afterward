export const ARENA = {
  width: 960,
  height: 540,
  bounds: { x: 38, y: 38, width: 884, height: 464 },
  playerStart: { x: 480, y: 292 },
  enemies: [
    { id: 'pursuer', kind: 'melee' as const, x: 245, y: 190 },
    { id: 'slinger', kind: 'ranged' as const, x: 725, y: 335 },
  ],
  obstacles: [
    { x: 342, y: 226, width: 94, height: 54 },
    { x: 625, y: 343, width: 106, height: 52 },
    { x: 748, y: 164, width: 44, height: 44 },
  ],
} as const;

export type ArenaObstacle = (typeof ARENA.obstacles)[number];

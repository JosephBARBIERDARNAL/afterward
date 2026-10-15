export const PLAYER = {
  maxHealth: 100,
  moveSpeed: 205,
  dashSpeed: 610,
  dashDuration: 0.18,
  dashInvulnerability: 0.24,
  dashCooldown: 0.78,
  primaryCooldown: 0.31,
  primaryRange: 74,
  primaryDamage: [13, 13, 20],
  comboReset: 0.9,
  heavyWindup: 0.43,
  heavyRecovery: 0.56,
  heavyRange: 100,
  heavyDamage: 44,
  damageInvulnerability: 0.72,
} as const;

export const ENEMIES = {
  melee: {
    maxHealth: 74,
    moveSpeed: 91,
    attackRange: 82,
    attackRadius: 48,
    attackWindup: 0.58,
    attackCooldown: 1.12,
    damage: 17,
    tint: 0xc56a4f,
  },
  ranged: {
    maxHealth: 60,
    moveSpeed: 78,
    keepDistance: 218,
    fireWindup: 0.64,
    attackCooldown: 1.75,
    projectileSpeed: 265,
    damage: 13,
    tint: 0x9a83ba,
  },
} as const;

export const COLORS = {
  ink: 0x18191c,
  floorA: 0x292a2c,
  floorB: 0x2b2c2e,
  seam: 0x343436,
  gold: 0xd6b678,
  bone: 0xe5dcc9,
  damage: 0xe46d59,
  heal: 0x91b988,
  white: 0xf2eee6,
} as const;

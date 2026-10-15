export type PermanentUpgradeId = 'vitality' | 'sharpened' | 'quickStep';

export interface PermanentUpgradeLevels {
  vitality: number;
  sharpened: number;
  quickStep: number;
}

export interface ProgressionSave {
  version: 1;
  resources: number;
  upgrades: PermanentUpgradeLevels;
}

export interface PermanentUpgrade {
  id: PermanentUpgradeId;
  title: string;
  effect: string;
  levels: { cost: number; amount: number }[];
}

export const PERMANENT_UPGRADES: PermanentUpgrade[] = [
  {
    id: 'vitality',
    title: 'VITALITY',
    effect: '+10 maximum health',
    levels: [
      { cost: 3, amount: 10 },
      { cost: 4, amount: 10 },
      { cost: 5, amount: 10 },
      { cost: 6, amount: 10 },
      { cost: 7, amount: 10 },
    ],
  },
  {
    id: 'sharpened',
    title: 'SHARPENED',
    effect: '+1 damage to every attack',
    levels: [
      { cost: 4, amount: 1 },
      { cost: 5, amount: 1 },
      { cost: 6, amount: 1 },
      { cost: 7, amount: 1 },
      { cost: 8, amount: 1 },
    ],
  },
  {
    id: 'quickStep',
    title: 'QUICK STEP',
    effect: '-0.04 seconds dash cooldown',
    levels: [
      { cost: 4, amount: 0.04 },
      { cost: 5, amount: 0.04 },
      { cost: 6, amount: 0.04 },
      { cost: 7, amount: 0.04 },
    ],
  },
];

const SAVE_KEY = 'too-late.progression.v1';

export function formatMoney(amount: number): string {
  return `$${amount}`;
}

export function createProgressionSave(): ProgressionSave {
  return {
    version: 1,
    resources: 0,
    upgrades: { vitality: 0, sharpened: 0, quickStep: 0 },
  };
}

export function loadProgressionSave(): ProgressionSave {
  try {
    const raw = globalThis.localStorage?.getItem(SAVE_KEY);
    if (!raw) return createProgressionSave();
    const parsed: unknown = JSON.parse(raw);
    if (!parsed || typeof parsed !== 'object') return createProgressionSave();

    const candidate = parsed as Partial<ProgressionSave>;
    if (candidate.version !== 1 || !candidate.upgrades || typeof candidate.upgrades !== 'object')
      return createProgressionSave();

    return {
      version: 1,
      resources: Number.isFinite(candidate.resources) ? Math.max(0, Math.floor(candidate.resources as number)) : 0,
      upgrades: {
        vitality: clampUpgradeLevel(candidate.upgrades.vitality, 'vitality'),
        sharpened: clampUpgradeLevel(candidate.upgrades.sharpened, 'sharpened'),
        quickStep: clampUpgradeLevel(candidate.upgrades.quickStep, 'quickStep'),
      },
    };
  } catch {
    return createProgressionSave();
  }
}

export function saveProgression(save: ProgressionSave): boolean {
  try {
    if (!globalThis.localStorage) return false;
    globalThis.localStorage.setItem(SAVE_KEY, JSON.stringify(save));
    return true;
  } catch {
    return false;
  }
}

export function buyPermanentUpgrade(save: ProgressionSave, id: PermanentUpgradeId): ProgressionSave | null {
  const definition = PERMANENT_UPGRADES.find((upgrade) => upgrade.id === id);
  if (!definition) return null;
  const level = save.upgrades[id];
  const nextLevel = definition.levels[level];
  if (!nextLevel || save.resources < nextLevel.cost) return null;

  return {
    ...save,
    resources: save.resources - nextLevel.cost,
    upgrades: { ...save.upgrades, [id]: level + 1 },
  };
}

function clampUpgradeLevel(value: number | undefined, id: PermanentUpgradeId): number {
  const maxLevel = PERMANENT_UPGRADES.find((upgrade) => upgrade.id === id)?.levels.length ?? 0;
  return Number.isFinite(value) ? Math.max(0, Math.min(maxLevel, Math.floor(value as number))) : 0;
}

import Phaser from 'phaser';
import { ARENA, ROOMS, type ArenaObstacle } from './data/arena';
import { COLORS, ENEMIES, PLAYER, type EliteKind, type EnemyKind } from './data/balance';
import {
  createProgressionSave,
  formatMoney,
  loadProgressionSave,
  saveProgression,
  type ProgressionSave,
} from './progression';
import { Sound } from './presentation/sound';
import {
  createFighter,
  destroyActorShadow,
  drawArena,
  drawEnemyMark,
  drawHealthBar,
  drawPlayerMark,
  syncActorShadow,
} from './presentation/visuals';

type RunState = 'playing' | 'dead' | 'clear' | 'abandoned';

interface Enemy {
  id: string;
  kind: EnemyKind;
  elite?: EliteKind;
  body: Phaser.GameObjects.Arc;
  marks: Phaser.GameObjects.Graphics;
  hp: number;
  maxHp: number;
  attackCooldown: number;
  windup: number;
  targetX: number;
  targetY: number;
  stun: number;
  flash: number;
  burnTime: number;
  burnTick: number;
  charging: number;
  chargeX: number;
  chargeY: number;
}

interface Projectile {
  body: Phaser.GameObjects.Arc;
  vx: number;
  vy: number;
  life: number;
}

interface InputFrame {
  moveX: number;
  moveY: number;
  primary: boolean;
  heavyPressed: boolean;
  dashPressed: boolean;
  interactPressed: boolean;
  aimX: number;
  aimY: number;
}

type BonusId =
  | 'vitality'
  | 'edge'
  | 'mend'
  | 'dashNova'
  | 'quickStep'
  | 'reach'
  | 'wideHeavy'
  | 'cinderCombo'
  | 'comboBlast'
  | 'siphon';

interface RunBonus {
  id: BonusId;
  title: string;
  detail: string;
  description: string;
  limit: number;
}

const RUN_BONUSES: RunBonus[] = [
  {
    id: 'vitality',
    title: 'VITALITY',
    detail: '+20 MAX LIFE',
    description: 'Gain 20 maximum health and restore 20 health immediately.',
    limit: 3,
  },
  {
    id: 'edge',
    title: 'SHARPENED',
    detail: '+3 BASE DAMAGE',
    description: 'Primary and heavy attacks deal 3 more damage.',
    limit: 4,
  },
  {
    id: 'mend',
    title: 'MENDING',
    detail: 'RESTORE 35 LIFE',
    description: 'Restore up to 35 health, without exceeding your maximum.',
    limit: 4,
  },
  {
    id: 'dashNova',
    title: 'DASH NOVA',
    detail: 'DASH ENDS IN A BURST',
    description: 'Your dash ends with a burst that deals 18 damage to enemies within 88 pixels.',
    limit: 1,
  },
  {
    id: 'quickStep',
    title: 'QUICK STEP',
    detail: 'DASH RECHARGE -0.1S',
    description: 'Reduce dash cooldown by 0.1 seconds per stack, up to 0.3 seconds.',
    limit: 3,
  },
  {
    id: 'reach',
    title: 'LONG REACH',
    detail: '+10 ATTACK RANGE',
    description: 'Increase the reach of primary and heavy attacks by 10 pixels per stack.',
    limit: 3,
  },
  {
    id: 'wideHeavy',
    title: 'WIDE SWING',
    detail: 'HEAVY ATTACK HITS A WIDER ARC',
    description: 'Heavy attacks cover a wider 128-degree arc.',
    limit: 1,
  },
  {
    id: 'cinderCombo',
    title: 'CINDER COMBO',
    detail: 'FINAL COMBO HIT BURNS',
    description: 'The third primary hit burns its target for 6 damage every 0.45 seconds, up to four times.',
    limit: 1,
  },
  {
    id: 'comboBlast',
    title: 'COMBO BLAST',
    detail: 'FINAL COMBO HIT BURSTS',
    description: 'The third primary hit bursts for 14 damage to enemies within 78 pixels.',
    limit: 1,
  },
  {
    id: 'siphon',
    title: 'SIPHON',
    detail: 'ATTACKS RESTORE 2 LIFE',
    description: 'Each attack hit restores 2 health per stack.',
    limit: 3,
  },
];

export class CombatScene extends Phaser.Scene {
  private player!: Phaser.GameObjects.Arc;
  private playerBody!: Phaser.Physics.Arcade.Body;
  private playerMarks!: Phaser.GameObjects.Graphics;
  private arenaMarks!: Phaser.GameObjects.Graphics;
  private fx!: Phaser.GameObjects.Graphics;
  private hud!: Phaser.GameObjects.Graphics;
  private overlay!: Phaser.GameObjects.Container;
  private enemies: Enemy[] = [];
  private projectiles: Projectile[] = [];
  private obstacles: ArenaObstacle[] = [];
  private obstacleBodies: Phaser.GameObjects.Rectangle[] = [];
  private obstacleDecor: Phaser.GameObjects.GameObject[] = [];
  private roomColliders: Phaser.Physics.Arcade.Collider[] = [];
  private roomArt?: Phaser.GameObjects.Graphics;
  private exitMark?: Phaser.GameObjects.Graphics;
  private explorationCache?: { x: number; y: number; claimed: boolean };
  private explorationCacheMark?: Phaser.GameObjects.Graphics;
  private roomIndex = 0;
  private roomCleared = false;
  private doorOpen = false;
  private transitioning = false;
  private bonusOpen = false;
  private pauseOpen = false;
  private catalogOpen = false;
  private bonusChoiceIndex = 0;
  private bonusChoices: RunBonus[] = [];
  private bonusCardBackgrounds: Phaser.GameObjects.Rectangle[] = [];
  private previousBonusPadButtons: boolean[] = [];
  private damageNumbers: Phaser.GameObjects.Text[] = [];
  private playerMaxHealth: number = PLAYER.maxHealth;
  private bonusDamage = 0;
  private bonusRange = 0;
  private bonusDashCooldownReduction = 0;
  private dashNovaUnlocked = false;
  private dashNovaEffect = 0;
  private resourceText!: Phaser.GameObjects.Text;
  private progression: ProgressionSave = createProgressionSave();
  private resourcesEarnedThisRun = 0;
  private saveUnavailable = false;
  private wideHeavyUnlocked = false;
  private cinderComboUnlocked = false;
  private comboBlastUnlocked = false;
  private siphonStacks = 0;
  private comboBlastEffect = 0;
  private comboBlastX = 0;
  private comboBlastY = 0;
  private readonly bonusCounts: Record<BonusId, number> = {
    vitality: 0,
    edge: 0,
    mend: 0,
    dashNova: 0,
    quickStep: 0,
    reach: 0,
    wideHeavy: 0,
    cinderCombo: 0,
    comboBlast: 0,
    siphon: 0,
  };
  private sfx!: Sound;
  private state: RunState = 'playing';
  private playerHealth: number = PLAYER.maxHealth;
  private dashCooldown = 0;
  private dashTime = 0;
  private dashX = 1;
  private dashY = 0;
  private invulnerability = 0;
  private primaryCooldown = 0;
  private comboTimer = 0;
  private comboIndex = 0;
  private heavyWindup = 0;
  private heavyRecovery = 0;
  private heavyHit = false;
  private attackEffect = 0;
  private attackEffectAngle = 0;
  private attackEffectKind: 'primary' | 'heavy' = 'primary';
  private playerAngle = 0;
  private pointerHasAimed = false;
  private queuedPrimary = false;
  private queuedHeavy = false;
  private previousPadButtons: boolean[] = [];
  private readonly handlePointerDown = (pointer: Phaser.Input.Pointer): void => {
    if (this.state !== 'playing' || this.bonusOpen || this.pauseOpen || this.catalogOpen) return;
    if (pointer.button === 0) this.queuedPrimary = true;
    if (pointer.button === 2) this.queuedHeavy = true;
  };
  private readonly handleBonusKeyDown = (event: KeyboardEvent): void => {
    if (!this.bonusOpen) return;
    let handled = true;
    switch (event.code) {
      case 'ArrowLeft':
        this.highlightBonus(this.bonusChoiceIndex - 1);
        break;
      case 'ArrowRight':
        this.highlightBonus(this.bonusChoiceIndex + 1);
        break;
      case 'Enter':
      case 'NumpadEnter':
        this.selectBonus(this.bonusChoiceIndex);
        break;
      default:
        handled = false;
    }
    if (handled) event.preventDefault();
  };
  private readonly handlePointerMove = (): void => {
    this.pointerHasAimed = true;
  };
  private keys!: Record<
    'up' | 'down' | 'left' | 'right' | 'primary' | 'heavy' | 'dash' | 'restart' | 'pause' | 'interact',
    Phaser.Input.Keyboard.Key
  >;
  private arrowKeys!: Record<'up' | 'down' | 'left' | 'right', Phaser.Input.Keyboard.Key>;
  private healthText!: Phaser.GameObjects.Text;
  private enemyText!: Phaser.GameObjects.Text;
  private bonusText!: Phaser.GameObjects.Text;
  private roomText!: Phaser.GameObjects.Text;
  private roomTitleText!: Phaser.GameObjects.Text;
  private gateText!: Phaser.GameObjects.Text;
  private comboText!: Phaser.GameObjects.Text;
  private actionText!: Phaser.GameObjects.Text;
  private dashText!: Phaser.GameObjects.Text;
  private hintText!: Phaser.GameObjects.Text;
  private damageFlash = 0;

  constructor() {
    super('CombatScene');
  }

  create(): void {
    this.progression = (this.registry.get('progressionSave') as ProgressionSave | undefined) ?? loadProgressionSave();
    this.saveUnavailable = Boolean(this.registry.get('saveUnavailable'));
    this.resetRunState();
    this.input.off('pointerdown', this.handlePointerDown);
    this.input.off('pointermove', this.handlePointerMove);
    this.physics.world.setBounds(ARENA.bounds.x, ARENA.bounds.y, ARENA.bounds.width, ARENA.bounds.height);
    this.createPlayer();
    this.createHud();
    this.setupInput();
    this.input.keyboard?.on('keydown', this.handleBonusKeyDown);
    this.enterRoom(0);
    this.sfx = new Sound();
    this.events.once(Phaser.Scenes.Events.SHUTDOWN, () => {
      this.input.off('pointerdown', this.handlePointerDown);
      this.input.off('pointermove', this.handlePointerMove);
      this.input.keyboard?.off('keydown', this.handleBonusKeyDown);
      this.sfx.destroy();
    });
  }

  private resetRunState(): void {
    this.physics.world.resume();
    this.tweens.resumeAll();
    this.state = 'playing';
    this.playerHealth = PLAYER.maxHealth + this.progression.upgrades.vitality * 10;
    this.dashCooldown = 0;
    this.dashTime = 0;
    this.dashX = 1;
    this.dashY = 0;
    this.invulnerability = 0;
    this.primaryCooldown = 0;
    this.comboTimer = 0;
    this.comboIndex = 0;
    this.heavyWindup = 0;
    this.heavyRecovery = 0;
    this.heavyHit = false;
    this.attackEffect = 0;
    this.dashNovaEffect = 0;
    this.comboBlastEffect = 0;
    this.wideHeavyUnlocked = false;
    this.cinderComboUnlocked = false;
    this.comboBlastUnlocked = false;
    this.siphonStacks = 0;
    this.attackEffectAngle = 0;
    this.attackEffectKind = 'primary';
    this.playerAngle = 0;
    this.pointerHasAimed = false;
    this.queuedPrimary = false;
    this.queuedHeavy = false;
    this.previousPadButtons = [];
    this.damageFlash = 0;
    this.enemies = [];
    this.projectiles = [];
    this.obstacles = [];
    this.obstacleBodies = [];
    this.obstacleDecor = [];
    this.roomColliders = [];
    this.roomArt = undefined;
    this.exitMark = undefined;
    this.explorationCache = undefined;
    this.explorationCacheMark = undefined;
    this.roomIndex = 0;
    this.roomCleared = false;
    this.doorOpen = false;
    this.transitioning = false;
    this.bonusOpen = false;
    this.pauseOpen = false;
    this.catalogOpen = false;
    this.bonusChoiceIndex = 0;
    this.bonusChoices = [];
    this.bonusCardBackgrounds = [];
    const pad = navigator.getGamepads?.().find((candidate) => candidate?.connected) ?? null;
    this.previousBonusPadButtons = pad?.buttons.map((button) => button.pressed) ?? [];
    this.damageNumbers = [];
    this.playerMaxHealth = this.playerHealth;
    this.resourcesEarnedThisRun = 0;
    this.bonusDamage = 0;
    this.bonusRange = 0;
    this.bonusDashCooldownReduction = 0;
    this.dashNovaUnlocked = false;
    for (const id of Object.keys(this.bonusCounts) as BonusId[]) this.bonusCounts[id] = 0;
    this.cameras.main.setScroll(0, 0).resetFX();
  }

  update(_time: number, deltaMs: number): void {
    if (this.state !== 'playing') {
      if (Phaser.Input.Keyboard.JustDown(this.keys.restart)) this.scene.restart();
      return;
    }
    if (this.catalogOpen) {
      if (Phaser.Input.Keyboard.JustDown(this.keys.pause)) this.showPauseMenu();
      return;
    }
    if (this.pauseOpen) {
      if (Phaser.Input.Keyboard.JustDown(this.keys.pause)) this.closePauseMenu();
      return;
    }
    if (this.bonusOpen) {
      this.updateBonusChoice();
      return;
    }
    if (Phaser.Input.Keyboard.JustDown(this.keys.pause)) {
      this.openPauseMenu();
      return;
    }
    if (this.transitioning) return;

    const delta = Math.min(deltaMs / 1000, 0.05);
    const controls = this.readInput();
    this.updateTimers(delta);
    this.updateAim(controls.aimX, controls.aimY);
    this.updatePlayer(controls, delta);
    this.updateEnemies(delta);
    this.updateProjectiles(delta);

    if (this.state !== 'playing') return;

    if (this.enemies.length === 0 && !this.roomCleared) {
      this.roomCleared = true;
      this.awardRoomResource();
      this.sfx.play('clear');
      if (this.roomIndex === ROOMS.length - 1) {
        this.gateText.setText('ALL ROOMS CLEAR');
        this.updateExitMark();
        this.finish('clear');
        return;
      }
      this.gateText.setText('DOOR READY · APPROACH TO OPEN');
      this.drawExplorationCache();
      this.updateExitMark();
      return;
    }

    const nearExit =
      this.player.x >= ARENA.exit.triggerX - 52 && Math.abs(this.player.y - ARENA.exit.y) <= ARENA.exit.halfHeight;
    const cache = this.explorationCache;
    const nearCache = Boolean(
      cache &&
        !cache.claimed &&
        this.roomCleared &&
        Phaser.Math.Distance.Between(this.player.x, this.player.y, cache.x, cache.y) <= 48,
    );
    if (nearCache) {
      this.gateText.setText('PRESS E / A TO SEARCH CACHE');
      if (controls.interactPressed) {
        this.openExplorationCache();
        return;
      }
    } else if (this.roomCleared && !this.doorOpen) {
      if (cache && !cache.claimed) {
        this.gateText.setText(nearExit ? 'E / A OPEN DOOR · CACHE AVAILABLE' : 'CACHE AVAILABLE · DOOR READY');
      } else {
        this.gateText.setText(nearExit ? 'PRESS E / A TO OPEN' : 'DOOR READY · APPROACH TO OPEN');
      }
      if (nearExit && controls.interactPressed) {
        this.openExitDoor();
        return;
      }
    }

    if (
      this.roomCleared &&
      this.doorOpen &&
      this.player.x >= ARENA.exit.triggerX &&
      Math.abs(this.player.y - ARENA.exit.y) <= ARENA.exit.halfHeight
    ) {
      this.transitionToNextRoom();
    }

    this.updatePresentation(delta);
  }

  private createObstacles(obstacles: ArenaObstacle[]): void {
    for (const obstacle of obstacles) {
      this.obstacles.push(obstacle);
      const shadow = this.add.rectangle(
        obstacle.x + 4,
        obstacle.y + 8,
        obstacle.width + 5,
        obstacle.height + 4,
        0x090a0c,
        0.55,
      );
      shadow.setDepth(obstacle.y - obstacle.height / 2);
      const block = this.add.rectangle(obstacle.x, obstacle.y, obstacle.width, obstacle.height, 0x544c43, 1);
      block.setStrokeStyle(2, 0x82745f, 0.85).setDepth(obstacle.y + obstacle.height / 2);
      const top = this.add.rectangle(
        obstacle.x,
        obstacle.y - obstacle.height / 2 + 5,
        obstacle.width - 8,
        4,
        0x9c8867,
        0.7,
      );
      top.setDepth(obstacle.y + obstacle.height / 2 + 1);
      this.physics.add.existing(block, true);
      this.obstacleBodies.push(block);
      this.obstacleDecor.push(shadow, top);
      this.roomColliders.push(this.physics.add.collider(this.player, block));
    }
  }

  private createPlayer(): void {
    this.player = createFighter(this, 0, 0, 14, 0xd9c7a4, 0xf2e6ce);
    this.physics.add.existing(this.player);
    this.playerBody = this.player.body as Phaser.Physics.Arcade.Body;
    this.playerBody.setCircle(13).setCollideWorldBounds(true).setMaxVelocity(PLAYER.dashSpeed);
    this.playerMarks = this.add.graphics().setDepth(30);
    this.arenaMarks = this.add.graphics().setDepth(25);
    this.fx = this.add.graphics().setDepth(40);
  }

  private createEnemies(spawns: (typeof ROOMS)[number]['enemies']): void {
    for (const entry of spawns) {
      const { kind, elite } = entry;
      const data = ENEMIES[kind];
      const baseRadius = kind === 'heavy' ? 20 : kind === 'melee' ? 16 : kind === 'charger' ? 18 : 15;
      const radius = baseRadius + (elite ? 3 : 0);
      const outline = elite ? COLORS.gold : kind === 'heavy' ? 0xe7c17b : kind === 'melee' ? 0xe29a70 : data.tint;
      const body = createFighter(this, entry.x, entry.y, radius, data.tint, outline);
      const marks = this.add.graphics().setDepth(entry.y + 2);
      this.physics.add.existing(body);
      const physicsBody = body.body as Phaser.Physics.Arcade.Body;
      physicsBody.setCircle(radius - 1).setCollideWorldBounds(true);
      for (const block of this.obstacleBodies) this.roomColliders.push(this.physics.add.collider(body, block));
      this.enemies.push({
        id: entry.id,
        kind,
        elite,
        body,
        marks,
        hp: Math.round(data.maxHealth * (elite ? 1.45 : 1)),
        maxHp: Math.round(data.maxHealth * (elite ? 1.45 : 1)),
        attackCooldown: data.attackCooldown,
        windup: 0,
        targetX: entry.x,
        targetY: entry.y,
        stun: 0,
        flash: 0,
        burnTime: 0,
        burnTick: 0,
        charging: 0,
        chargeX: 0,
        chargeY: 0,
      });
    }
  }

  private createExplorationCache(position?: { x: number; y: number }): void {
    if (!position) return;
    this.explorationCache = { ...position, claimed: false };
    this.explorationCacheMark = this.add.graphics().setDepth(position.y + 2);
    this.drawExplorationCache();
  }

  private drawExplorationCache(): void {
    const cache = this.explorationCache;
    const mark = this.explorationCacheMark;
    if (!cache || !mark) return;

    const available = this.roomCleared && !cache.claimed;
    const trim = available ? COLORS.gold : 0x786f5e;
    const alpha = cache.claimed ? 0.48 : available ? 0.95 : 0.5;
    mark.clear();
    if (available) mark.fillStyle(COLORS.gold, 0.15).fillCircle(cache.x, cache.y, 25);
    mark.fillStyle(cache.claimed ? 0x1a1917 : 0x383127, 0.98).fillRect(cache.x - 15, cache.y - 4, 30, 15);
    mark.lineStyle(2, trim, alpha).strokeRect(cache.x - 15, cache.y - 4, 30, 15);
    mark.lineStyle(2, trim, alpha).lineBetween(cache.x - 17, cache.y - 7, cache.x + 17, cache.y - 7);
    mark.lineBetween(cache.x, cache.y - 4, cache.x, cache.y + 11);
    if (!cache.claimed) mark.fillStyle(trim, alpha).fillRect(cache.x - 2, cache.y + 1, 4, 5);
    else {
      mark.lineStyle(2, trim, alpha).lineBetween(cache.x - 12, cache.y - 6, cache.x - 8, cache.y - 13);
      mark.lineBetween(cache.x + 12, cache.y - 6, cache.x + 8, cache.y - 13);
    }
  }

  private enterRoom(index: number): void {
    if (this.roomArt) this.clearRoom();
    const room = ROOMS[index];
    if (!room) throw new Error(`Unknown room index: ${index}`);
    this.roomIndex = index;
    this.roomCleared = false;
    this.doorOpen = false;
    this.roomArt = drawArena(this);
    this.exitMark = this.add.graphics().setDepth(18);
    this.createObstacles(room.obstacles);
    this.createEnemies(room.enemies);
    this.createExplorationCache(room.explorationCache);
    this.playerBody.reset(room.playerStart.x, room.playerStart.y);
    this.player.setAlpha(1).setDepth(this.player.y + 1);
    this.playerMarks.clear();
    this.arenaMarks.clear();
    this.fx.clear();
    this.roomText.setText(`ROOM  ${String(index + 1).padStart(2, '0')} / ${String(ROOMS.length).padStart(2, '0')}`);
    this.roomTitleText.setText(room.title);
    this.enemyText.setText(`HOSTILES  ${String(this.enemies.length).padStart(2, '0')}`);
    this.gateText.setText('EAST GATE LOCKED');
    this.updateExitMark();
  }

  private clearRoom(): void {
    for (const collider of this.roomColliders) this.physics.world.removeCollider(collider);
    for (const enemy of [...this.enemies]) {
      enemy.marks.destroy();
      destroyActorShadow(enemy.body);
      enemy.body.destroy();
    }
    for (const projectile of this.projectiles) projectile.body.destroy();
    for (const number of this.damageNumbers) {
      this.tweens.killTweensOf(number);
      number.destroy();
    }
    for (const object of [...this.obstacleDecor, ...this.obstacleBodies]) object.destroy();
    this.roomArt?.destroy();
    this.exitMark?.destroy();
    this.explorationCacheMark?.destroy();
    this.enemies = [];
    this.projectiles = [];
    this.obstacles = [];
    this.obstacleBodies = [];
    this.obstacleDecor = [];
    this.roomColliders = [];
    this.damageNumbers = [];
    this.explorationCache = undefined;
    this.explorationCacheMark = undefined;
  }

  private transitionToNextRoom(): void {
    if (this.transitioning) return;
    this.transitioning = true;
    const camera = this.cameras.main;
    camera.once(Phaser.Cameras.Scene2D.Events.FADE_OUT_COMPLETE, () => {
      this.enterRoom(this.roomIndex + 1);
      camera.once(Phaser.Cameras.Scene2D.Events.FADE_IN_COMPLETE, () => {
        this.transitioning = false;
      });
      camera.fadeIn(220, 10, 10, 12);
    });
    camera.fadeOut(180, 10, 10, 12);
  }

  private showBonusChoices(source: 'door' | 'cache'): void {
    this.bonusOpen = true;
    this.bonusChoiceIndex = 0;
    this.previousBonusPadButtons = [];
    const eligible = RUN_BONUSES.filter(
      (bonus) =>
        this.bonusCounts[bonus.id] < bonus.limit && (bonus.id !== 'mend' || this.playerHealth < this.playerMaxHealth),
    );
    this.bonusChoices = Phaser.Utils.Array.Shuffle([...eligible]).slice(0, 3);
    this.overlay.removeAll(true);
    const shade = this.add.rectangle(0, 0, ARENA.width, ARENA.height, 0x0a0b0d, 0.82).setOrigin(0);
    const panel = this.add.rectangle(480, 270, 720, 260, 0x1c1c1e, 0.99).setStrokeStyle(1, 0x695b46, 0.95);
    const title = this.add
      .text(480, 166, 'CHOOSE A BONUS', {
        fontFamily: 'Manrope, sans-serif',
        fontSize: '24px',
        fontStyle: 'bold',
        color: '#e3c88f',
      })
      .setOrigin(0.5);
    const subtitle = this.add
      .text(
        480,
        198,
        source === 'door' ? 'DOOR OPENED  ·  +$1 EARNED  ·  PICK ONE BONUS' : 'CACHE OPENED  ·  PICK ONE BONUS',
        {
          fontFamily: 'DM Mono, monospace',
          fontSize: '9px',
          color: '#b9a47f',
          letterSpacing: 0.8,
        },
      )
      .setOrigin(0.5);
    this.overlay.add([shade, panel, title, subtitle]);
    this.bonusCardBackgrounds = [];
    this.bonusChoices.forEach((bonus, index) => {
      const card = this.add.container(240 + index * 240, 286);
      const background = this.add.rectangle(0, 0, 218, 132, 0x272522, 1).setStrokeStyle(1, 0x6d6250, 0.9);
      const heading = this.add
        .text(0, -36, bonus.title, {
          fontFamily: 'Manrope, sans-serif',
          fontSize: '15px',
          fontStyle: 'bold',
          color: '#f0e7d7',
        })
        .setOrigin(0.5);
      const detail = this.add
        .text(0, 0, bonus.detail, {
          fontFamily: 'DM Mono, monospace',
          fontSize: '9px',
          color: '#d6b678',
          align: 'center',
        })
        .setOrigin(0.5);
      const context = this.bonusCounts[bonus.id] > 0 ? `OWNED  ×${this.bonusCounts[bonus.id]}` : 'NEW BONUS';
      const status = this.add
        .text(0, 29, context, {
          fontFamily: 'DM Mono, monospace',
          fontSize: '8px',
          color: '#aaa39a',
          letterSpacing: 0.5,
        })
        .setOrigin(0.5);
      card.add([background, heading, detail, status]);
      card.setSize(218, 132);
      this.overlay.add(card);
      this.bonusCardBackgrounds.push(background);
    });
    const instructions = this.add
      .text(480, 382, 'LEFT / RIGHT OR D-PAD TO CHOOSE  ·  ENTER / A TO CONFIRM', {
        fontFamily: 'DM Mono, monospace',
        fontSize: '9px',
        color: '#8b8882',
        letterSpacing: 0.8,
      })
      .setOrigin(0.5);
    this.overlay.add(instructions);
    this.highlightBonus(0);
    this.overlay.setVisible(true);
  }

  private openExitDoor(): void {
    if (!this.roomCleared || this.doorOpen || this.bonusOpen || this.state !== 'playing') return;
    this.doorOpen = true;
    this.gateText.setText('CHOOSE A BONUS');
    this.updateExitMark();
    this.showBonusChoices('door');
  }

  private openExplorationCache(): void {
    const cache = this.explorationCache;
    if (!cache || cache.claimed || !this.roomCleared || this.bonusOpen || this.state !== 'playing') return;
    cache.claimed = true;
    this.drawExplorationCache();
    this.sfx.play('clear');
    this.gateText.setText('CACHE OPENED');
    this.showBonusChoices('cache');
  }

  private updateBonusChoice(): void {
    const pad = navigator.getGamepads?.().find((candidate) => candidate?.connected) ?? null;
    const buttons = pad?.buttons.map((button) => button.pressed) ?? [];
    const justPressed = (index: number) => Boolean(buttons[index] && !this.previousBonusPadButtons[index]);
    if (justPressed(14)) this.highlightBonus(this.bonusChoiceIndex - 1);
    else if (justPressed(15)) this.highlightBonus(this.bonusChoiceIndex + 1);
    else if (justPressed(0)) this.selectBonus(this.bonusChoiceIndex);
    this.previousBonusPadButtons = buttons;
  }

  private highlightBonus(index: number): void {
    if (this.bonusCardBackgrounds.length === 0) return;
    const selected = Phaser.Math.Wrap(index, 0, this.bonusCardBackgrounds.length);
    this.bonusChoiceIndex = selected;
    this.bonusCardBackgrounds.forEach((background, cardIndex) => {
      background.setStrokeStyle(cardIndex === selected ? 2 : 1, cardIndex === selected ? COLORS.gold : 0x6d6250, 0.95);
      background.setFillStyle(cardIndex === selected ? 0x342f27 : 0x272522, 1);
    });
  }

  private selectBonus(index: number): void {
    const bonus = this.bonusChoices[index];
    if (!bonus || !this.bonusOpen) return;
    this.bonusOpen = false;
    this.overlay.removeAll(true);
    this.applyBonus(bonus);
    this.updateBonusHud();
    this.gateText.setText(this.doorOpen ? 'EAST GATE OPEN' : 'DOOR READY · APPROACH TO OPEN');
    this.updateExitMark();
  }

  private openPauseMenu(): void {
    if (this.pauseOpen || this.bonusOpen || this.transitioning) return;
    this.pauseOpen = true;
    this.physics.world.pause();
    this.tweens.pauseAll();
    this.showPauseMenu();
  }

  private closePauseMenu(): void {
    this.pauseOpen = false;
    this.catalogOpen = false;
    this.overlay.removeAll(true).setVisible(false);
    this.physics.world.resume();
    this.tweens.resumeAll();
  }

  private showPauseMenu(): void {
    this.catalogOpen = false;
    this.overlay.removeAll(true);
    const shade = this.add.rectangle(0, 0, ARENA.width, ARENA.height, 0x0a0b0d, 0.82).setOrigin(0);
    const panel = this.add.rectangle(480, 270, 410, 390, 0x1c1c1e, 0.99).setStrokeStyle(1, 0x695b46, 0.95);
    const title = this.add
      .text(480, 112, 'PAUSED', {
        fontFamily: 'Manrope, sans-serif',
        fontSize: '26px',
        fontStyle: 'bold',
        color: '#e3c88f',
      })
      .setOrigin(0.5);
    this.overlay.add([shade, panel, title]);
    this.addMenuButton('CONTINUE', 480, 172, () => this.closePauseMenu());
    this.addMenuButton('RESTART RUN', 480, 230, () => {
      this.registry.set('progressionSave', this.progression);
      this.registry.set('saveUnavailable', this.saveUnavailable);
      this.registry.set('lastRunSummary', undefined);
      this.scene.restart();
    });
    this.addMenuButton('BONUS', 480, 288, () => this.showBonusCatalog());
    this.addMenuButton('BACK HOME', 480, 346, () => this.finish('abandoned'));
    const hint = this.add
      .text(480, 418, 'PRESS ESC TO CONTINUE', {
        fontFamily: 'DM Mono, monospace',
        fontSize: '9px',
        color: '#8b8882',
        letterSpacing: 0.8,
      })
      .setOrigin(0.5);
    this.overlay.add(hint);
    this.overlay.setVisible(true);
  }

  private showBonusCatalog(): void {
    this.catalogOpen = true;
    this.overlay.removeAll(true);
    const shade = this.add.rectangle(0, 0, ARENA.width, ARENA.height, 0x0a0b0d, 0.86).setOrigin(0);
    const panel = this.add.rectangle(480, 270, 870, 470, 0x1c1c1e, 0.99).setStrokeStyle(1, 0x695b46, 0.95);
    const title = this.add
      .text(480, 61, 'RUN BONUS CATALOG', {
        fontFamily: 'Manrope, sans-serif',
        fontSize: '22px',
        fontStyle: 'bold',
        color: '#e3c88f',
      })
      .setOrigin(0.5);
    const subtitle = this.add
      .text(480, 87, 'EFFECTS AND BONUSES COLLECTED THIS RUN', {
        fontFamily: 'DM Mono, monospace',
        fontSize: '8px',
        color: '#b9a47f',
        letterSpacing: 0.8,
      })
      .setOrigin(0.5);
    this.overlay.add([shade, panel, title, subtitle]);

    RUN_BONUSES.forEach((bonus, index) => {
      const column = Math.floor(index / 5);
      const row = index % 5;
      const x = column === 0 ? 265 : 695;
      const y = 125 + row * 66;
      const card = this.add.rectangle(x, y + 21, 405, 62, 0x272522, 1).setStrokeStyle(1, 0x514a3f, 0.9);
      const heading = this.add
        .text(x - 183, y, bonus.title, {
          fontFamily: 'Manrope, sans-serif',
          fontSize: '10px',
          fontStyle: 'bold',
          color: '#f0e7d7',
        })
        .setOrigin(0, 0.5);
      const description = this.add
        .text(x - 183, y + 18, bonus.description, {
          fontFamily: 'DM Mono, monospace',
          fontSize: '8px',
          color: '#b9b1a4',
          lineSpacing: 1,
          wordWrap: { width: 310, useAdvancedWrap: true },
        })
        .setOrigin(0, 0);
      const count = this.bonusCounts[bonus.id];
      const status = this.add
        .text(x + 184, y, count > 0 ? `×${count}` : '—', {
          fontFamily: 'DM Mono, monospace',
          fontSize: '9px',
          fontStyle: 'bold',
          color: count > 0 ? '#d6b678' : '#77736d',
        })
        .setOrigin(1, 0.5);
      this.overlay.add([card, heading, description, status]);
    });
    this.addMenuButton('BACK TO PAUSE', 480, 470, () => this.showPauseMenu(), 300);
    this.overlay.setVisible(true);
  }

  private addMenuButton(label: string, x: number, y: number, onClick: () => void, width = 270): void {
    const button = this.add.container(x, y);
    const background = this.add.rectangle(0, 0, width, 48, 0x292722, 1).setStrokeStyle(1, 0x6d6250, 0.95);
    const text = this.add
      .text(0, 0, label, {
        fontFamily: 'Manrope, sans-serif',
        fontSize: '12px',
        fontStyle: 'bold',
        color: '#eee4d1',
        letterSpacing: 1,
      })
      .setOrigin(0.5);
    button.add([background, text]);
    button.setSize(width, 48).setInteractive({ useHandCursor: true });
    button.on('pointerover', () => {
      background.setFillStyle(0x393328, 1).setStrokeStyle(2, COLORS.gold, 1);
      text.setColor('#f5dca6');
    });
    button.on('pointerout', () => {
      background.setFillStyle(0x292722, 1).setStrokeStyle(1, 0x6d6250, 0.95);
      text.setColor('#eee4d1');
    });
    button.on('pointerdown', onClick);
    this.overlay.add(button);
  }

  private awardRoomResource(): void {
    this.resourcesEarnedThisRun += 1;
    this.progression = { ...this.progression, resources: this.progression.resources + 1 };
    this.saveUnavailable = !saveProgression(this.progression);
    this.registry.set('progressionSave', this.progression);
    this.registry.set('saveUnavailable', this.saveUnavailable);
  }

  private applyBonus(bonus: RunBonus): void {
    this.bonusCounts[bonus.id] += 1;
    switch (bonus.id) {
      case 'vitality':
        this.playerMaxHealth += 20;
        this.healPlayer(20);
        break;
      case 'edge':
        this.bonusDamage += 3;
        break;
      case 'mend':
        this.healPlayer(35);
        break;
      case 'dashNova':
        this.dashNovaUnlocked = true;
        break;
      case 'quickStep':
        this.bonusDashCooldownReduction = Math.min(0.3, this.bonusDashCooldownReduction + 0.1);
        break;
      case 'reach':
        this.bonusRange += 10;
        break;
      case 'wideHeavy':
        this.wideHeavyUnlocked = true;
        break;
      case 'cinderCombo':
        this.cinderComboUnlocked = true;
        break;
      case 'comboBlast':
        this.comboBlastUnlocked = true;
        break;
      case 'siphon':
        this.siphonStacks += 1;
        break;
    }
  }

  private updateBonusHud(): void {
    const labels: string[] = [];
    if (this.bonusCounts.vitality > 0)
      labels.push(`VITALITY ×${this.bonusCounts.vitality}  MAX LIFE +${this.bonusCounts.vitality * 20}`);
    if (this.bonusCounts.edge > 0) labels.push(`SHARPENED ×${this.bonusCounts.edge}  DAMAGE +${this.bonusDamage}`);
    if (this.bonusCounts.mend > 0) labels.push(`MENDING ×${this.bonusCounts.mend}`);
    if (this.dashNovaUnlocked) labels.push('DASH NOVA');
    if (this.bonusCounts.quickStep > 0)
      labels.push(`QUICK STEP ×${this.bonusCounts.quickStep}  DASH -${this.bonusDashCooldownReduction.toFixed(1)}S`);
    if (this.bonusCounts.reach > 0) labels.push(`LONG REACH ×${this.bonusCounts.reach}  RANGE +${this.bonusRange}`);
    if (this.wideHeavyUnlocked) labels.push('WIDE SWING');
    if (this.cinderComboUnlocked) labels.push('CINDER COMBO');
    if (this.comboBlastUnlocked) labels.push('COMBO BLAST');
    if (this.siphonStacks > 0) labels.push(`SIPHON ×${this.siphonStacks}  HEAL +${this.siphonStacks * 2}`);
    this.bonusText.setText(labels.length > 0 ? labels.join('\n') : 'NONE YET');
  }

  private healPlayer(amount: number): void {
    const healed = Math.min(this.playerMaxHealth, this.playerHealth + amount) - this.playerHealth;
    if (healed <= 0) return;
    this.playerHealth += healed;
    this.showDamageNumber(this.player.x, this.player.y - 44, healed, 'healed');
  }

  private updateExitMark(): void {
    if (!this.exitMark) return;
    this.exitMark.clear();
    const color = this.roomCleared ? COLORS.gold : 0x786a54;
    this.exitMark.fillStyle(this.doorOpen ? 0x30291e : 0x161719, 0.96).fillRect(914, 231, 14, 78);
    this.exitMark.lineStyle(2, color, 0.92).strokeRect(914, 231, 14, 78);
    if (this.doorOpen) {
      this.exitMark.lineStyle(2, COLORS.gold, 0.9).lineBetween(918, 241, 918, 299).lineBetween(924, 241, 924, 299);
    } else {
      this.exitMark.lineStyle(2, color, 0.72).lineBetween(917, 244, 925, 252).lineBetween(925, 244, 917, 252);
      this.exitMark.lineBetween(917, 288, 925, 296).lineBetween(925, 288, 917, 296);
    }
  }

  private createHud(): void {
    const fixed = (
      text: string,
      x: number,
      y: number,
      size: string,
      color: string,
      extra: Record<string, unknown> = {},
    ) =>
      this.add
        .text(x, y, text, {
          fontFamily: '"DM Mono", ui-monospace, monospace',
          fontSize: size,
          color,
          letterSpacing: 1,
          ...extra,
        })
        .setScrollFactor(0)
        .setDepth(1000);

    this.roomText = fixed('ROOM  01 / 05', 34, 27, '11px', '#d6b678', { fontStyle: 'bold' });
    this.roomTitleText = fixed('THE ENTRY', 34, 44, '9px', '#8b8882');
    this.healthText = fixed('VITALITY  100 / 100', 34, 66, '10px', '#e9e1d4');
    fixed('RUN BONUSES', 34, 101, '8px', '#b9a47f');
    this.bonusText = fixed('NONE YET', 34, 114, '8px', '#d6b678', {
      maxWidth: 520,
      lineSpacing: 2,
      wordWrap: { width: 520, useAdvancedWrap: true },
    });
    this.enemyText = fixed('HOSTILES  02', 758, 28, '10px', '#e9e1d4', { align: 'right' });
    this.gateText = fixed('EAST GATE LOCKED', 758, 45, '9px', '#8b8882', { align: 'right' });
    this.resourceText = fixed(formatMoney(0), 758, 62, '9px', '#d6b678', { align: 'right' });
    this.comboText = fixed('COMBO  —', 34, 493, '9px', '#d6b678');
    this.dashText = fixed('DASH', 758, 477, '8px', '#8b8882');
    this.actionText = fixed('HEAVY READY', 758, 497, '8px', '#d7cdbd');
    this.hintText = fixed(
      'ARROWS / ZQSD MOVE  ·  MOUSE AIM  ·  J ATTACK  ·  K HEAVY  ·  SPACE DASH  ·  E OPEN',
      480,
      519,
      '8px',
      '#77736d',
      { align: 'center' },
    );
    this.hud = this.add.graphics().setScrollFactor(0).setDepth(999);

    this.overlay = this.add.container(0, 0).setDepth(2000).setScrollFactor(0).setVisible(false);
    this.input.on('pointerdown', this.handlePointerDown);
    this.input.mouse?.disableContextMenu();
    this.input.on('pointermove', this.handlePointerMove);
  }

  private setupInput(): void {
    const keyboard = this.input.keyboard;
    if (!keyboard) throw new Error('Keyboard input is unavailable.');
    this.keys = {
      up: keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.Z),
      down: keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.S),
      left: keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.Q),
      right: keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.D),
      primary: keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.J),
      heavy: keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.K),
      dash: keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.SPACE),
      restart: keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.R),
      pause: keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.ESC),
      interact: keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.E),
    };
    this.arrowKeys = {
      up: keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.UP),
      down: keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.DOWN),
      left: keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.LEFT),
      right: keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.RIGHT),
    };
  }

  private readInput(): InputFrame {
    let moveX =
      Number(this.keys.right.isDown || this.arrowKeys.right.isDown) -
      Number(this.keys.left.isDown || this.arrowKeys.left.isDown);
    let moveY =
      Number(this.keys.down.isDown || this.arrowKeys.down.isDown) -
      Number(this.keys.up.isDown || this.arrowKeys.up.isDown);

    const pad = navigator.getGamepads?.().find((candidate) => candidate?.connected) ?? null;
    const buttons = pad?.buttons.map((button) => button.pressed) ?? [];
    const justPressed = (index: number) => Boolean(buttons[index] && !this.previousPadButtons[index]);
    if (pad) {
      const stickX = this.deadzone(pad.axes[0] ?? 0);
      const stickY = this.deadzone(pad.axes[1] ?? 0);
      if (Math.hypot(stickX, stickY) > Math.hypot(moveX, moveY)) {
        moveX = stickX;
        moveY = stickY;
      }
    }
    if (Math.hypot(moveX, moveY) > 1) {
      const length = Math.hypot(moveX, moveY);
      moveX /= length;
      moveY /= length;
    }

    let aimX = this.playerAngle ? Math.cos(this.playerAngle) : 1;
    let aimY = this.playerAngle ? Math.sin(this.playerAngle) : 0;
    const rightX = this.deadzone(pad?.axes[2] ?? 0);
    const rightY = this.deadzone(pad?.axes[3] ?? 0);
    if (Math.hypot(rightX, rightY) > 0.22) {
      aimX = rightX;
      aimY = rightY;
      this.pointerHasAimed = false;
    } else if (this.pointerHasAimed) {
      const pointer = this.input.activePointer;
      aimX = pointer.worldX - this.player.x;
      aimY = pointer.worldY - this.player.y;
    } else if (Math.hypot(moveX, moveY) > 0.1) {
      aimX = moveX;
      aimY = moveY;
    }
    const aimLength = Math.hypot(aimX, aimY) || 1;
    aimX /= aimLength;
    aimY /= aimLength;

    const primary = this.keys.primary.isDown || this.queuedPrimary || Boolean(buttons[2] || buttons[5]);
    const heavyPressed =
      Phaser.Input.Keyboard.JustDown(this.keys.heavy) || this.queuedHeavy || justPressed(3) || justPressed(6);
    const dashPressed = Phaser.Input.Keyboard.JustDown(this.keys.dash) || justPressed(1);
    const interactPressed = Phaser.Input.Keyboard.JustDown(this.keys.interact) || justPressed(0);

    this.queuedPrimary = false;
    this.queuedHeavy = false;
    this.previousPadButtons = buttons;
    return { moveX, moveY, primary, heavyPressed, dashPressed, interactPressed, aimX, aimY };
  }

  private updateAim(x: number, y: number): void {
    if (Math.hypot(x, y) > 0.15) this.playerAngle = Math.atan2(y, x);
  }

  private currentDashCooldown(): number {
    return Math.max(
      0.25,
      PLAYER.dashCooldown - this.progression.upgrades.quickStep * 0.04 - this.bonusDashCooldownReduction,
    );
  }

  private updateTimers(delta: number): void {
    this.primaryCooldown = Math.max(0, this.primaryCooldown - delta);
    this.comboTimer = Math.max(0, this.comboTimer - delta);
    if (this.comboTimer === 0) this.comboIndex = 0;
    this.dashCooldown = Math.max(0, this.dashCooldown - delta);
    const previousDashTime = this.dashTime;
    this.dashTime = Math.max(0, this.dashTime - delta);
    if (previousDashTime > 0 && this.dashTime === 0 && this.dashNovaUnlocked) this.performDashNova();
    this.invulnerability = Math.max(0, this.invulnerability - delta);
    this.heavyRecovery = Math.max(0, this.heavyRecovery - delta);
    this.damageFlash = Math.max(0, this.damageFlash - delta);
    this.attackEffect = Math.max(0, this.attackEffect - delta);
    this.dashNovaEffect = Math.max(0, this.dashNovaEffect - delta);
    this.comboBlastEffect = Math.max(0, this.comboBlastEffect - delta);
  }

  private updatePlayer(controls: InputFrame, delta: number): void {
    if (controls.dashPressed && this.dashCooldown <= 0 && this.heavyWindup <= 0 && this.heavyRecovery <= 0) {
      const directionLength = Math.hypot(controls.moveX, controls.moveY);
      this.dashX = directionLength > 0.1 ? controls.moveX / directionLength : controls.aimX;
      this.dashY = directionLength > 0.1 ? controls.moveY / directionLength : controls.aimY;
      this.dashTime = PLAYER.dashDuration;
      this.dashCooldown = this.currentDashCooldown();
      this.invulnerability = Math.max(this.invulnerability, PLAYER.dashInvulnerability);
      this.sfx.play('dash');
    }

    if (controls.heavyPressed && this.heavyWindup <= 0 && this.heavyRecovery <= 0 && this.dashTime <= 0) {
      this.heavyWindup = PLAYER.heavyWindup;
      this.heavyHit = false;
    }
    if (this.heavyWindup > 0) {
      this.heavyWindup = Math.max(0, this.heavyWindup - delta);
      if (this.heavyWindup === 0 && !this.heavyHit) {
        this.heavyHit = true;
        this.heavyRecovery = PLAYER.heavyRecovery;
        this.performAttack('heavy');
      }
    }

    if (
      controls.primary &&
      this.primaryCooldown <= 0 &&
      this.heavyWindup <= 0 &&
      this.heavyRecovery <= 0 &&
      this.dashTime <= 0
    ) {
      if (this.comboTimer <= 0) this.comboIndex = 0;
      this.performAttack('primary');
      this.comboIndex = (this.comboIndex + 1) % PLAYER.primaryDamage.length;
      this.comboTimer = PLAYER.comboReset;
      this.primaryCooldown = PLAYER.primaryCooldown;
    }

    const actionSpeed = this.heavyWindup > 0 || this.heavyRecovery > 0 ? 0.42 : 1;
    if (this.dashTime > 0) {
      this.playerBody.setVelocity(this.dashX * PLAYER.dashSpeed, this.dashY * PLAYER.dashSpeed);
    } else {
      this.playerBody.setVelocity(
        controls.moveX * PLAYER.moveSpeed * actionSpeed,
        controls.moveY * PLAYER.moveSpeed * actionSpeed,
      );
    }

    this.player.setDepth(this.player.y + 1);
    syncActorShadow(this.player);
    this.playerMarks.setDepth(this.player.y + 2);
    const flashing = this.invulnerability > 0 && Math.floor(this.time.now / 65) % 2 === 0;
    this.player.setAlpha(flashing ? 0.48 : 1);
    if (this.damageFlash > 0) this.player.setFillStyle(COLORS.damage, 1);
    else this.player.setFillStyle(0xd9c7a4, 1);
  }

  private performAttack(kind: 'primary' | 'heavy'): void {
    const isHeavy = kind === 'heavy';
    const combo = this.comboIndex;
    const reach = (isHeavy ? PLAYER.heavyRange : PLAYER.primaryRange + (combo === 2 ? 8 : 0)) + this.bonusRange;
    const halfArc = isHeavy ? (this.wideHeavyUnlocked ? 1.12 : 0.68) : combo === 2 ? 0.72 : 0.58;
    const damage =
      (isHeavy ? PLAYER.heavyDamage : PLAYER.primaryDamage[combo]) +
      this.progression.upgrades.sharpened +
      this.bonusDamage;
    this.attackEffect = isHeavy ? 0.22 : 0.14;
    this.attackEffectAngle = this.playerAngle;
    this.attackEffectKind = kind;
    this.sfx.play(isHeavy ? 'heavy' : 'primary');
    this.cameras.main.shake(isHeavy ? 95 : 48, isHeavy ? 0.0024 : 0.0012);

    const hitEnemies: Enemy[] = [];
    let blastOrigin: { x: number; y: number } | undefined;
    for (const enemy of [...this.enemies]) {
      const dx = enemy.body.x - this.player.x;
      const dy = enemy.body.y - this.player.y;
      const distance = Math.hypot(dx, dy);
      const angleDifference = Phaser.Math.Angle.Wrap(Math.atan2(dy, dx) - this.playerAngle);
      if (
        distance > reach + (enemy.kind === 'heavy' ? 19 : enemy.kind === 'melee' ? 15 : 14) ||
        Math.abs(angleDifference) > halfArc
      )
        continue;
      if (!this.lineClear(this.player.x, this.player.y, enemy.body.x, enemy.body.y)) continue;
      if (!blastOrigin) blastOrigin = { x: enemy.body.x, y: enemy.body.y };
      hitEnemies.push(enemy);
      if (!isHeavy && combo === 2 && this.cinderComboUnlocked) {
        enemy.burnTime = 1.9;
        enemy.burnTick = 0.45;
      }
      this.damageEnemy(enemy, damage, dx / (distance || 1), dy / (distance || 1));
    }
    if (!isHeavy && combo === 2 && this.comboBlastUnlocked && blastOrigin) {
      this.performComboBlast(blastOrigin.x, blastOrigin.y, hitEnemies);
    }
  }

  private performComboBlast(x: number, y: number, directlyHit: Enemy[]): void {
    this.comboBlastEffect = 0.24;
    this.comboBlastX = x;
    this.comboBlastY = y;
    this.sfx.play('combo-burst');
    for (const enemy of [...this.enemies]) {
      if (directlyHit.includes(enemy)) continue;
      const dx = enemy.body.x - x;
      const dy = enemy.body.y - y;
      const distance = Math.hypot(dx, dy);
      if (distance > 78 || !this.lineClear(x, y, enemy.body.x, enemy.body.y)) continue;
      this.damageEnemy(enemy, 14, dx / (distance || 1), dy / (distance || 1));
    }
    this.cameras.main.shake(80, 0.0018);
  }

  private performDashNova(): void {
    this.dashNovaEffect = 0.24;
    this.sfx.play('dash-burst');
    for (const enemy of [...this.enemies]) {
      const dx = enemy.body.x - this.player.x;
      const dy = enemy.body.y - this.player.y;
      const distance = Math.hypot(dx, dy);
      if (distance > 88 || !this.lineClear(this.player.x, this.player.y, enemy.body.x, enemy.body.y)) continue;
      this.damageEnemy(enemy, 18, dx / (distance || 1), dy / (distance || 1));
    }
    this.cameras.main.shake(75, 0.0018);
  }

  private damageEnemy(enemy: Enemy, damage: number, knockX: number, knockY: number, grantSiphon = true): void {
    enemy.hp -= damage;
    enemy.flash = 0.12;
    enemy.stun = Math.max(enemy.stun, 0.13);
    this.showDamageNumber(enemy.body.x, enemy.body.y - (enemy.kind === 'heavy' ? 31 : 27), damage, 'dealt');
    if (grantSiphon && this.siphonStacks > 0) this.healPlayer(this.siphonStacks * 2);
    const body = enemy.body.body as Phaser.Physics.Arcade.Body;
    body.setVelocity(knockX * 170, knockY * 170);
    this.sfx.play('enemy-hit');
    if (enemy.hp <= 0) {
      this.sfx.play('enemy-down');
      enemy.marks.destroy();
      destroyActorShadow(enemy.body);
      enemy.body.destroy();
      this.enemies = this.enemies.filter((candidate) => candidate !== enemy);
    }
  }

  private showDamageNumber(x: number, y: number, damage: number, kind: 'dealt' | 'received' | 'healed'): void {
    const number = this.add
      .text(x + Phaser.Math.Between(-8, 8), y, `${kind === 'healed' ? '+' : '-'}${damage}`, {
        fontFamily: 'Manrope, sans-serif',
        fontSize: kind === 'received' ? '25px' : '20px',
        fontStyle: 'bold',
        color: kind === 'received' ? '#ff514d' : kind === 'healed' ? '#9ecf8f' : '#f2dfb7',
        stroke: '#171513',
        strokeThickness: 4,
        shadow: { color: '#000000', blur: 5, offsetX: 0, offsetY: 2, fill: true },
      })
      .setOrigin(0.5)
      .setDepth(500)
      .setScale(0.58);
    this.damageNumbers.push(number);
    this.tweens.add({
      targets: number,
      scale: 1.16,
      duration: 115,
      ease: 'Back.Out',
      onComplete: () =>
        this.tweens.add({
          targets: number,
          y: y - 44,
          scale: 0.94,
          alpha: 0,
          delay: 80,
          duration: 620,
          ease: 'Cubic.Out',
          onComplete: () => {
            this.damageNumbers = this.damageNumbers.filter((candidate) => candidate !== number);
            number.destroy();
          },
        }),
    });
  }

  private updateEnemies(delta: number): void {
    for (const enemy of [...this.enemies]) {
      const data = ENEMIES[enemy.kind];
      const body = enemy.body.body as Phaser.Physics.Arcade.Body;
      const dx = this.player.x - enemy.body.x;
      const dy = this.player.y - enemy.body.y;
      const distance = Math.hypot(dx, dy) || 1;
      enemy.flash = Math.max(0, enemy.flash - delta);
      enemy.stun = Math.max(0, enemy.stun - delta);
      enemy.attackCooldown = Math.max(0, enemy.attackCooldown - delta);
      enemy.body.setAlpha(enemy.flash > 0 ? 0.5 : 1);
      enemy.body.setDepth(enemy.body.y + 1);
      syncActorShadow(enemy.body);

      if (enemy.burnTime > 0) {
        enemy.burnTime = Math.max(0, enemy.burnTime - delta);
        enemy.burnTick -= delta;
        if (enemy.burnTick <= 0) {
          this.damageEnemy(enemy, 6, 0, 0, false);
          enemy.burnTick = 0.45;
          if (enemy.hp <= 0) continue;
        }
      }

      if (enemy.windup > 0) {
        enemy.windup = Math.max(0, enemy.windup - delta);
        body.setVelocity(0, 0);
        if (enemy.windup === 0) this.resolveEnemyAttack(enemy);
        continue;
      }
      if (enemy.stun > 0) {
        if (enemy.charging > 0) {
          enemy.charging = 0;
          body.setVelocity(0, 0);
        }
        continue;
      }

      if (enemy.charging > 0) {
        enemy.charging = Math.max(0, enemy.charging - delta);
        const chargeSpeed = ENEMIES.charger.chargeSpeed * (enemy.elite === 'vanguard' ? 1.12 : 1);
        body.setVelocity(enemy.chargeX * chargeSpeed, enemy.chargeY * chargeSpeed);
        if (Phaser.Math.Distance.Between(this.player.x, this.player.y, enemy.body.x, enemy.body.y) < 28) {
          this.hurtPlayer(data.damage, enemy.body.x, enemy.body.y);
          enemy.charging = 0;
        }
        if (enemy.charging === 0) {
          body.setVelocity(0, 0);
          enemy.stun = Math.max(enemy.stun, 0.28);
        }
        continue;
      }

      if (enemy.kind !== 'ranged') {
        if (enemy.kind === 'charger') {
          if (distance <= ENEMIES.charger.chargeRange && enemy.attackCooldown <= 0) {
            enemy.windup = ENEMIES.charger.attackWindup;
            enemy.targetX = this.player.x;
            enemy.targetY = this.player.y;
            body.setVelocity(0, 0);
            continue;
          }
          const speed = data.moveSpeed * (enemy.elite === 'vanguard' ? 1.12 : 1);
          body.setVelocity((dx / distance) * speed, (dy / distance) * speed);
          continue;
        }

        if (enemy.kind === 'skirmisher') {
          if (distance <= ENEMIES.skirmisher.attackRange && enemy.attackCooldown <= 0) {
            enemy.windup = ENEMIES.skirmisher.attackWindup;
            enemy.targetX = this.player.x;
            enemy.targetY = this.player.y;
            body.setVelocity(0, 0);
            continue;
          }
          const preferredDistance = ENEMIES.skirmisher.preferredDistance;
          const radial = distance > preferredDistance ? 0.8 : distance < preferredDistance - 18 ? -0.3 : 0;
          const direction = enemy.id.charCodeAt(0) % 2 === 0 ? 1 : -1;
          const lateral = 0.58 * direction;
          const speed = ENEMIES.skirmisher.moveSpeed * (enemy.elite === 'vanguard' ? 1.12 : 1);
          body.setVelocity(
            ((dx / distance) * radial - (dy / distance) * lateral) * speed,
            ((dy / distance) * radial + (dx / distance) * lateral) * speed,
          );
          continue;
        }

        const attack = ENEMIES[enemy.kind];
        if (distance <= attack.attackRange && enemy.attackCooldown <= 0) {
          enemy.windup = attack.attackWindup;
          enemy.targetX = this.player.x;
          enemy.targetY = this.player.y;
          body.setVelocity(0, 0);
          continue;
        }
        if (distance > attack.attackRange - 8) {
          const speed = data.moveSpeed * (enemy.elite === 'vanguard' ? 1.12 : 1);
          body.setVelocity((dx / distance) * speed, (dy / distance) * speed);
        } else body.setVelocity(0, 0);
      } else {
        const ideal = ENEMIES.ranged.keepDistance;
        if (enemy.attackCooldown <= 0 && distance < ideal + 95) {
          enemy.windup = ENEMIES.ranged.fireWindup;
          enemy.targetX = this.player.x;
          enemy.targetY = this.player.y;
          body.setVelocity(0, 0);
          continue;
        }
        const toward = distance > ideal + 30 ? 1 : distance < ideal - 36 ? -1 : 0;
        body.setVelocity((dx / distance) * data.moveSpeed * toward, (dy / distance) * data.moveSpeed * toward);
      }
    }
  }

  private resolveEnemyAttack(enemy: Enemy): void {
    const data = ENEMIES[enemy.kind];
    enemy.attackCooldown = data.attackCooldown * (enemy.elite === 'vanguard' ? 0.82 : 1);
    if (enemy.kind === 'charger') {
      const dx = enemy.targetX - enemy.body.x;
      const dy = enemy.targetY - enemy.body.y;
      const distance = Math.hypot(dx, dy) || 1;
      enemy.chargeX = dx / distance;
      enemy.chargeY = dy / distance;
      enemy.charging = ENEMIES.charger.chargeDuration;
      return;
    }
    if (enemy.kind === 'ranged') {
      const spread = enemy.elite === 'volley' ? [-0.16, 0, 0.16] : [0];
      for (const angle of spread) {
        this.spawnProjectile(enemy.body.x, enemy.body.y, enemy.targetX, enemy.targetY, angle);
      }
    } else {
      const attack = ENEMIES[enemy.kind];
      if (
        Phaser.Math.Distance.Between(this.player.x, this.player.y, enemy.targetX, enemy.targetY) < attack.attackRadius
      ) {
        this.hurtPlayer(attack.damage, enemy.targetX, enemy.targetY);
      }
    }
  }

  private spawnProjectile(x: number, y: number, targetX: number, targetY: number, spreadAngle = 0): void {
    const dx = targetX - x;
    const dy = targetY - y;
    const direction = Math.atan2(dy, dx) + spreadAngle;
    const body = this.add.circle(x, y, 7, COLORS.damage, 1).setStrokeStyle(2, 0xffb188, 1).setDepth(35);
    this.projectiles.push({
      body,
      vx: Math.cos(direction) * ENEMIES.ranged.projectileSpeed,
      vy: Math.sin(direction) * ENEMIES.ranged.projectileSpeed,
      life: 2.8,
    });
  }

  private updateProjectiles(delta: number): void {
    for (let index = this.projectiles.length - 1; index >= 0; index -= 1) {
      const projectile = this.projectiles[index];
      projectile.body.x += projectile.vx * delta;
      projectile.body.y += projectile.vy * delta;
      projectile.life -= delta;
      const hitsPlayer =
        Phaser.Math.Distance.Between(projectile.body.x, projectile.body.y, this.player.x, this.player.y) < 21;
      const blocked = this.obstacles.some(
        (obstacle) =>
          projectile.body.x > obstacle.x - obstacle.width / 2 - 7 &&
          projectile.body.x < obstacle.x + obstacle.width / 2 + 7 &&
          projectile.body.y > obstacle.y - obstacle.height / 2 - 7 &&
          projectile.body.y < obstacle.y + obstacle.height / 2 + 7,
      );
      if (hitsPlayer) this.hurtPlayer(ENEMIES.ranged.damage, projectile.body.x, projectile.body.y);
      if (
        hitsPlayer ||
        blocked ||
        projectile.life <= 0 ||
        projectile.body.x < 20 ||
        projectile.body.x > 940 ||
        projectile.body.y < 20 ||
        projectile.body.y > 520
      ) {
        projectile.body.destroy();
        this.projectiles.splice(index, 1);
      }
    }
  }

  private hurtPlayer(damage: number, sourceX: number, sourceY: number): void {
    if (this.invulnerability > 0 || this.state !== 'playing') return;
    this.playerHealth = Math.max(0, this.playerHealth - damage);
    this.showDamageNumber(this.player.x, this.player.y - 27, damage, 'received');
    this.invulnerability = PLAYER.damageInvulnerability;
    this.damageFlash = 0.2;
    this.heavyWindup = 0;
    this.heavyRecovery = 0.3;
    this.sfx.play('player-hit');
    this.cameras.main.shake(180, 0.006);
    const dx = this.player.x - sourceX;
    const dy = this.player.y - sourceY;
    const length = Math.hypot(dx, dy) || 1;
    this.playerBody.setVelocity((dx / length) * 210, (dy / length) * 210);
    if (this.playerHealth <= 0) {
      this.finish('dead');
    }
  }

  private updatePresentation(delta: number): void {
    this.playerMarks.clear();
    drawPlayerMark(
      this.playerMarks,
      this.player.x,
      this.player.y - 2,
      this.playerAngle,
      this.heavyWindup > 0 ? COLORS.damage : 0x4a4034,
    );

    this.arenaMarks.clear();
    for (const enemy of this.enemies) {
      enemy.marks.clear().setDepth(enemy.body.y + 2);
      drawEnemyMark(enemy.marks, enemy.body.x, enemy.body.y - 1, enemy.kind, 0x241f1b);
      if (enemy.elite) {
        enemy.marks
          .lineStyle(2, COLORS.gold, 0.88)
          .strokeCircle(enemy.body.x, enemy.body.y, enemy.kind === 'heavy' ? 28 : 23)
          .lineBetween(enemy.body.x, enemy.body.y - 44, enemy.body.x + 5, enemy.body.y - 39)
          .lineBetween(enemy.body.x + 5, enemy.body.y - 39, enemy.body.x, enemy.body.y - 34)
          .lineBetween(enemy.body.x, enemy.body.y - 34, enemy.body.x - 5, enemy.body.y - 39)
          .lineBetween(enemy.body.x - 5, enemy.body.y - 39, enemy.body.x, enemy.body.y - 44);
      }
      if (enemy.burnTime > 0) {
        enemy.marks.lineStyle(2, 0xe46d59, 0.72).strokeCircle(enemy.body.x, enemy.body.y, 23);
      }
      const healthColor = enemy.elite
        ? COLORS.gold
        : enemy.kind === 'heavy'
          ? COLORS.gold
          : enemy.kind === 'melee'
            ? 0xd7775d
            : ENEMIES[enemy.kind].tint;
      drawHealthBar(enemy.marks, enemy.body.x - 21, enemy.body.y - 32, 42, enemy.hp / enemy.maxHp, healthColor);
      if (enemy.windup > 0) {
        if (enemy.kind === 'charger') {
          this.arenaMarks
            .lineStyle(3, COLORS.damage, 0.62)
            .lineBetween(enemy.body.x, enemy.body.y, enemy.targetX, enemy.targetY);
          this.arenaMarks.lineStyle(2, COLORS.damage, 0.82).strokeCircle(enemy.targetX, enemy.targetY, 12);
        } else if (enemy.kind === 'ranged') {
          this.arenaMarks
            .lineStyle(2, COLORS.damage, 0.52)
            .lineBetween(enemy.body.x, enemy.body.y, enemy.targetX, enemy.targetY);
          this.arenaMarks.fillStyle(COLORS.damage, 0.58).fillCircle(enemy.targetX, enemy.targetY, 8);
        } else {
          const attack = ENEMIES[enemy.kind];
          const warningColor = enemy.kind === 'heavy' ? COLORS.gold : COLORS.damage;
          this.arenaMarks
            .lineStyle(enemy.kind === 'heavy' ? 3 : 2, warningColor, 0.68)
            .strokeCircle(enemy.targetX, enemy.targetY, attack.attackRadius);
          this.arenaMarks
            .lineStyle(1, 0xffca9a, 0.3)
            .strokeCircle(enemy.targetX, enemy.targetY, attack.attackRadius + 9);
        }
      }
    }

    if (this.heavyWindup > 0) {
      const endX = this.player.x + Math.cos(this.playerAngle) * 92;
      const endY = this.player.y + Math.sin(this.playerAngle) * 92;
      this.arenaMarks.lineStyle(6, COLORS.damage, 0.32).lineBetween(this.player.x, this.player.y, endX, endY);
    }

    this.fx.clear().setDepth(this.player.y + 8);
    if (this.dashNovaEffect > 0) {
      const progress = 1 - this.dashNovaEffect / 0.24;
      const radius = 28 + progress * 66;
      const alpha = 0.85 * (1 - progress);
      this.fx.lineStyle(5, COLORS.gold, alpha).strokeCircle(this.player.x, this.player.y, radius);
      this.fx.lineStyle(2, 0xffefc8, alpha * 0.8).strokeCircle(this.player.x, this.player.y, radius - 8);
    }
    if (this.comboBlastEffect > 0) {
      const progress = 1 - this.comboBlastEffect / 0.24;
      const radius = 20 + progress * 58;
      const alpha = 0.88 * (1 - progress);
      this.fx.lineStyle(5, 0xf2dfb7, alpha).strokeCircle(this.comboBlastX, this.comboBlastY, radius);
      this.fx.lineStyle(2, 0xffb775, alpha * 0.8).strokeCircle(this.comboBlastX, this.comboBlastY, radius - 7);
    }
    if (this.attackEffect > 0) {
      const heavy = this.attackEffectKind === 'heavy';
      const angle = this.attackEffectAngle;
      const length = heavy ? 82 : 60;
      const side = heavy ? (this.wideHeavyUnlocked ? 1.12 : 0.72) : 0.49;
      const fromX = this.player.x + Math.cos(angle - side) * 23;
      const fromY = this.player.y + Math.sin(angle - side) * 23;
      const toX = this.player.x + Math.cos(angle + side) * length;
      const toY = this.player.y + Math.sin(angle + side) * length;
      this.fx.lineStyle(heavy ? 9 : 6, heavy ? 0xd6b678 : 0xece0c8, Math.min(0.9, this.attackEffect * 6));
      this.fx.lineBetween(fromX, fromY, toX, toY);
      this.fx.lineStyle(2, 0xffffff, Math.min(0.8, this.attackEffect * 5));
      this.fx.lineBetween(fromX + 3, fromY - 3, toX + 3, toY - 3);
    }

    this.hud.clear();
    this.hud.fillStyle(0x111214, 0.92).fillRect(34, 84, 158, 7);
    const healthColor = this.playerHealth <= 30 ? COLORS.damage : COLORS.heal;
    this.hud.fillStyle(healthColor, 1).fillRect(36, 86, 154 * (this.playerHealth / this.playerMaxHealth), 3);
    this.hud.fillStyle(0x111214, 0.95).fillRect(802, 478, 124, 8);
    const dashProgress = 1 - Phaser.Math.Clamp(this.dashCooldown / this.currentDashCooldown(), 0, 1);
    this.hud.fillStyle(COLORS.gold, 1).fillRect(804, 480, 120 * dashProgress, 4);
    this.healthText.setText(`VITALITY  ${String(this.playerHealth).padStart(3, '0')} / ${this.playerMaxHealth}`);
    this.resourceText.setText(formatMoney(this.progression.resources));
    this.enemyText.setText(`HOSTILES  ${String(this.enemies.length).padStart(2, '0')}`);
    this.comboText.setText(`COMBO  ${this.comboIndex === 0 ? '—' : `${this.comboIndex} / 3`}`);
    const heavy = this.heavyWindup > 0 || this.heavyRecovery > 0 ? 'HEAVY SETTLING' : 'HEAVY READY';
    this.dashText.setColor(this.dashCooldown > 0 ? '#8b8882' : '#d6b678');
    this.actionText.setText(heavy);
    this.actionText.setColor(this.heavyWindup > 0 || this.heavyRecovery > 0 ? '#8b8882' : '#d7cdbd');
    if (delta > 0) this.hintText.setAlpha(this.pointerHasAimed ? 0.8 : 1);
  }

  private finish(state: Exclude<RunState, 'playing'>): void {
    if (this.state !== 'playing') return;
    this.state = state;
    this.playerBody.setVelocity(0, 0);
    for (const enemy of this.enemies) (enemy.body.body as Phaser.Physics.Arcade.Body).setVelocity(0, 0);
    this.registry.set('progressionSave', this.progression);
    this.registry.set('saveUnavailable', this.saveUnavailable);
    this.registry.set('lastRunSummary', {
      result: state,
      roomsCleared: state === 'clear' ? this.roomIndex + 1 : this.roomIndex,
      resourcesEarned: this.resourcesEarnedThisRun,
      saveUnavailable: this.saveUnavailable,
    });
    this.scene.start('HubScene');
  }

  private lineClear(x1: number, y1: number, x2: number, y2: number): boolean {
    const steps = Math.ceil(Phaser.Math.Distance.Between(x1, y1, x2, y2) / 8);
    for (let step = 1; step < steps; step += 1) {
      const fraction = step / steps;
      const x = Phaser.Math.Linear(x1, x2, fraction);
      const y = Phaser.Math.Linear(y1, y2, fraction);
      if (
        this.obstacles.some(
          (obstacle) =>
            x > obstacle.x - obstacle.width / 2 &&
            x < obstacle.x + obstacle.width / 2 &&
            y > obstacle.y - obstacle.height / 2 &&
            y < obstacle.y + obstacle.height / 2,
        )
      )
        return false;
    }
    return true;
  }

  private deadzone(value: number): number {
    const magnitude = Math.abs(value);
    if (magnitude < 0.18) return 0;
    return (Math.sign(value) * (magnitude - 0.18)) / 0.82;
  }
}

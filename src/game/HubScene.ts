import Phaser from 'phaser';
import { ARENA, ROOMS } from './data/arena';
import { COLORS } from './data/balance';
import {
  buyPermanentUpgrade,
  createProgressionSave,
  formatMoney,
  loadProgressionSave,
  PERMANENT_UPGRADES,
  saveProgression,
  type PermanentUpgrade,
  type PermanentUpgradeId,
  type ProgressionSave,
} from './progression';
import { createFighter, drawArena, drawPlayerMark, syncActorShadow } from './presentation/visuals';

interface RunSummary {
  result: 'dead' | 'clear' | 'abandoned';
  roomsCleared: number;
  resourcesEarned: number;
  saveUnavailable: boolean;
}

export class HubScene extends Phaser.Scene {
  private progression: ProgressionSave = createProgressionSave();
  private player!: Phaser.GameObjects.Arc;
  private playerBody!: Phaser.Physics.Arcade.Body;
  private playerMark!: Phaser.GameObjects.Graphics;
  private playerLabel!: Phaser.GameObjects.Text;
  private doorHint!: Phaser.GameObjects.Text;
  private resourcesText!: Phaser.GameObjects.Text;
  private shopBalanceText!: Phaser.GameObjects.Text;
  private shopAvailabilityText!: Phaser.GameObjects.Text;
  private messageText!: Phaser.GameObjects.Text;
  private shopModal?: Phaser.GameObjects.Container;
  private shopLayer?: Phaser.GameObjects.Container;
  private shopOpen = false;
  private saveUnavailable = false;
  private startingRun = false;
  private previousPadConfirmPressed = false;
  private keys!: {
    up: Phaser.Input.Keyboard.Key[];
    down: Phaser.Input.Keyboard.Key[];
    left: Phaser.Input.Keyboard.Key[];
    right: Phaser.Input.Keyboard.Key[];
    interact: Phaser.Input.Keyboard.Key;
  };
  constructor() {
    super('HubScene');
  }

  create(): void {
    const savedInSession = this.registry.get('progressionSave') as ProgressionSave | undefined;
    this.progression = savedInSession ?? loadProgressionSave();
    this.saveUnavailable = Boolean(this.registry.get('saveUnavailable'));
    this.startingRun = false;
    this.shopOpen = false;
    this.registry.set('progressionSave', this.progression);

    this.physics.world.setBounds(ARENA.bounds.x, ARENA.bounds.y, ARENA.bounds.width, ARENA.bounds.height);
    drawArena(this);
    this.createRoomArt();
    this.createPlayer();
    this.createHeader();
    this.createShop();
    this.createDoor();
    this.showRunSummary(this.registry.get('lastRunSummary') as RunSummary | undefined);
    this.registry.set('lastRunSummary', undefined);
    this.setupInput();
  }

  update(): void {
    const pad = navigator.getGamepads?.().find((candidate) => candidate?.connected) ?? null;
    let moveX = Number(this.keys.right.some((key) => key.isDown)) - Number(this.keys.left.some((key) => key.isDown));
    let moveY = Number(this.keys.down.some((key) => key.isDown)) - Number(this.keys.up.some((key) => key.isDown));

    if (pad) {
      const stickX = this.deadzone(pad.axes[0] ?? 0);
      const stickY = this.deadzone(pad.axes[1] ?? 0);
      if (Math.hypot(stickX, stickY) > Math.hypot(moveX, moveY)) {
        moveX = stickX;
        moveY = stickY;
      }
    }

    const length = Math.hypot(moveX, moveY);
    if (length > 1) {
      moveX /= length;
      moveY /= length;
    }
    this.playerBody.setVelocity(moveX * 180, moveY * 180);
    this.player.setDepth(this.player.y + 1);
    this.playerMark.clear();
    drawPlayerMark(this.playerMark, this.player.x, this.player.y, 0, 0x18191c);
    this.playerMark.setDepth(this.player.y + 2);
    this.playerLabel.setPosition(this.player.x, this.player.y + 38).setDepth(this.player.y + 2);
    syncActorShadow(this.player);

    const nearDoor = Phaser.Math.Distance.Between(this.player.x, this.player.y, 836, 275) <= 58;
    const hint = nearDoor ? 'E / A TO BEGIN' : 'WALK TO DOOR · E / A';
    if (this.doorHint.text !== hint) this.doorHint.setText(hint);

    const padConfirmPressed = Boolean(pad?.buttons[0]?.pressed);
    const padConfirmJustPressed = padConfirmPressed && !this.previousPadConfirmPressed;
    this.previousPadConfirmPressed = padConfirmPressed;
    if (nearDoor && (Phaser.Input.Keyboard.JustDown(this.keys.interact) || padConfirmJustPressed)) {
      this.startRun();
    }
  }

  private createRoomArt(): void {
    const art = this.add.graphics().setDepth(-5);
    art.fillStyle(0x211f1b, 0.82).fillRoundedRect(48, 130, 270, 330, 8);
    art.lineStyle(1, 0x695b46, 0.92).strokeRoundedRect(48, 130, 270, 330, 8);
    art.lineStyle(1, 0x514a3f, 0.7);
    art.lineBetween(65, 143, 301, 143);
    art.lineBetween(65, 447, 301, 447);
    art.fillStyle(0x4c4539, 0.72).fillRect(778, 390, 130, 3);
  }

  private createPlayer(): void {
    this.player = createFighter(this, 480, 278, 16, 0xd9c7a4, 0xf2e6ce);
    this.physics.add.existing(this.player);
    this.playerBody = this.player.body as Phaser.Physics.Arcade.Body;
    this.playerBody.setCircle(15).setCollideWorldBounds(true).setMaxVelocity(180);
    this.playerMark = this.add.graphics().setDepth(300);
    drawPlayerMark(this.playerMark, this.player.x, this.player.y, 0, 0x18191c);
    this.playerLabel = this.add
      .text(480, 316, 'YOU', {
        fontFamily: 'DM Mono, monospace',
        fontSize: '8px',
        color: '#b9a47f',
        letterSpacing: 1,
      })
      .setOrigin(0.5)
      .setDepth(300);
  }

  private createHeader(): void {
    this.add
      .text(480, 35, 'THE ASHEN REST', {
        fontFamily: 'Manrope, sans-serif',
        fontSize: '22px',
        fontStyle: 'bold',
        color: '#e3c88f',
        letterSpacing: 1,
      })
      .setOrigin(0.5)
      .setDepth(300);
    this.add
      .text(480, 62, 'SAFE ROOM  ·  NO HOSTILES  ·  PREPARE BEFORE THE NEXT RUN', {
        fontFamily: 'DM Mono, monospace',
        fontSize: '8px',
        color: '#aaa39a',
        letterSpacing: 0.65,
      })
      .setOrigin(0.5)
      .setDepth(300);

    this.resourcesText = this.add
      .text(480, 91, formatMoney(this.progression.resources), {
        fontFamily: 'Manrope, sans-serif',
        fontSize: '14px',
        fontStyle: 'bold',
        color: '#d6b678',
      })
      .setOrigin(0.5)
      .setDepth(300);
    this.add
      .text(480, 108, 'EARN $1 PER ROOM CLEARED · CLICK THE SHOP TO BROWSE', {
        fontFamily: 'DM Mono, monospace',
        fontSize: '8px',
        color: '#817d75',
        letterSpacing: 0.5,
      })
      .setOrigin(0.5)
      .setDepth(300);
  }

  private createShop(): void {
    const shop = this.add.container(183, 295).setDepth(300);
    const cabinet = this.add.rectangle(0, 0, 218, 142, 0x292722, 0.98).setStrokeStyle(2, 0x695b46, 1);
    const header = this.add
      .text(0, -32, 'UPGRADE SHOP', {
        fontFamily: 'Manrope, sans-serif',
        fontSize: '13px',
        fontStyle: 'bold',
        color: '#e3c88f',
        letterSpacing: 1,
      })
      .setOrigin(0.5);
    this.shopBalanceText = this.add
      .text(0, -5, formatMoney(this.progression.resources), {
        fontFamily: 'DM Mono, monospace',
        fontSize: '12px',
        color: '#d6b678',
      })
      .setOrigin(0.5);
    const prompt = this.add
      .text(0, 36, 'CLICK TO VIEW UPGRADES', {
        fontFamily: 'DM Mono, monospace',
        fontSize: '8px',
        color: '#b9a47f',
        letterSpacing: 0.6,
      })
      .setOrigin(0.5);
    shop.add([cabinet, header, this.shopBalanceText, prompt]);
    shop.setSize(218, 142).setInteractive({ useHandCursor: true });
    shop.on('pointerdown', () => this.openShop());
  }

  private renderShopRows(): void {
    if (!this.shopLayer) return;
    this.shopLayer.removeAll(true);
    PERMANENT_UPGRADES.forEach((upgrade, index) => {
      const y = 184 + index * 84;
      const background = this.add.rectangle(480, y, 610, 72, 0x292722, 0.98).setStrokeStyle(1, 0x514a3f, 0.95);
      const title = this.add
        .text(205, y - 15, upgrade.title, {
          fontFamily: 'Manrope, sans-serif',
          fontSize: '11px',
          fontStyle: 'bold',
          color: '#f0e7d7',
        })
        .setOrigin(0, 0.5);
      const effect = this.add
        .text(205, y + 10, upgrade.effect.toUpperCase(), {
          fontFamily: 'DM Mono, monospace',
          fontSize: '8px',
          color: '#b9a47f',
        })
        .setOrigin(0, 0.5);
      const level = this.progression.upgrades[upgrade.id];
      const nextLevel = upgrade.levels[level];
      const status = this.add
        .text(758, y - 15, `LEVEL ${level} / ${upgrade.levels.length}`, {
          fontFamily: 'DM Mono, monospace',
          fontSize: '8px',
          color: '#aaa39a',
        })
        .setOrigin(1, 0.5);
      const affordable = Boolean(nextLevel && this.progression.resources >= nextLevel.cost);
      const actionLabel = nextLevel
        ? `${affordable ? 'BUY' : 'NEED'}  ·  ${formatMoney(nextLevel.cost)}`
        : 'MAX LEVEL';
      const action = this.add.container(705, y + 16);
      const actionBackground = this.add
        .rectangle(0, 0, 124, 28, affordable ? 0x393328 : 0x211f1d, 1)
        .setStrokeStyle(1, affordable ? COLORS.gold : 0x49443b, 0.95);
      const actionText = this.add
        .text(0, 0, actionLabel, {
          fontFamily: 'DM Mono, monospace',
          fontSize: '8px',
          fontStyle: 'bold',
          color: affordable ? '#f5dca6' : '#77736d',
        })
        .setOrigin(0.5);
      action.add([actionBackground, actionText]);
      action.setSize(124, 28).setInteractive({ useHandCursor: true });
      action.on('pointerdown', () => this.purchaseUpgrade(upgrade));
      this.shopLayer.add([background, title, effect, status, action]);
    });
  }

  private openShop(): void {
    if (this.shopOpen) return;
    this.shopOpen = true;
    this.shopModal = this.add.container(0, 0).setDepth(2000);
    const shade = this.add.rectangle(0, 0, ARENA.width, ARENA.height, 0x0a0b0d, 0.82).setOrigin(0);
    const panel = this.add.rectangle(480, 270, 700, 360, 0x1c1c1e, 0.99).setStrokeStyle(1, 0x695b46, 0.95);
    const title = this.add
      .text(480, 112, 'PERMANENT UPGRADES', {
        fontFamily: 'Manrope, sans-serif',
        fontSize: '22px',
        fontStyle: 'bold',
        color: '#e3c88f',
      })
      .setOrigin(0.5);
    this.shopAvailabilityText = this.add
      .text(480, 139, `AVAILABLE  ${formatMoney(this.progression.resources)}  ·  EFFECTIVE NEXT RUN`, {
        fontFamily: 'DM Mono, monospace',
        fontSize: '9px',
        color: '#b9a47f',
      })
      .setOrigin(0.5);
    const close = this.add.container(770, 112);
    const closeBackground = this.add.rectangle(0, 0, 84, 30, 0x292722, 1).setStrokeStyle(1, 0x6d6250, 0.95);
    const closeText = this.add
      .text(0, 0, 'CLOSE', {
        fontFamily: 'DM Mono, monospace',
        fontSize: '8px',
        fontStyle: 'bold',
        color: '#eee4d1',
      })
      .setOrigin(0.5);
    close.add([closeBackground, closeText]);
    close.setSize(84, 30).setInteractive({ useHandCursor: true });
    close.on('pointerdown', () => this.closeShop());
    this.shopLayer = this.add.container(0, 0);
    this.shopModal.add([shade, panel, title, this.shopAvailabilityText, close, this.shopLayer]);
    this.renderShopRows();
  }

  private closeShop(): void {
    this.shopOpen = false;
    this.shopModal?.destroy(true);
    this.shopModal = undefined;
    this.shopLayer = undefined;
  }

  private createDoor(): void {
    const door = this.add.container(836, 275).setDepth(400);
    const frame = this.add.rectangle(0, 0, 132, 190, 0x26231e, 1).setStrokeStyle(3, 0x9a8050, 1);
    const inner = this.add.rectangle(0, -2, 112, 168, 0x393127, 1).setStrokeStyle(1, 0x65553b, 1);
    const seam = this.add.rectangle(-28, -2, 2, 168, 0x8b744d, 0.75);
    const knob = this.add.circle(34, 3, 4, COLORS.gold, 1);
    const label = this.add
      .text(0, 118, 'BEGIN RUN', {
        fontFamily: 'Manrope, sans-serif',
        fontSize: '10px',
        fontStyle: 'bold',
        color: '#e3c88f',
        letterSpacing: 1,
      })
      .setOrigin(0.5);
    this.doorHint = this.add
      .text(0, 139, 'WALK TO DOOR · E / A', {
        fontFamily: 'DM Mono, monospace',
        fontSize: '7px',
        color: '#aaa39a',
        letterSpacing: 0.5,
      })
      .setOrigin(0.5);
    door.add([frame, inner, seam, knob, label, this.doorHint]);
  }

  private setupInput(): void {
    const keyboard = this.input.keyboard;
    if (!keyboard) throw new Error('Keyboard input is unavailable.');
    this.keys = {
      up: [keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.Z), keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.UP)],
      down: [keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.S), keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.DOWN)],
      left: [keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.Q), keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.LEFT)],
      right: [keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.D), keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.RIGHT)],
      interact: keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.E),
    };
    keyboard.addCapture([
      Phaser.Input.Keyboard.KeyCodes.UP,
      Phaser.Input.Keyboard.KeyCodes.DOWN,
      Phaser.Input.Keyboard.KeyCodes.LEFT,
      Phaser.Input.Keyboard.KeyCodes.RIGHT,
    ]);
  }

  private deadzone(value: number): number {
    return Math.abs(value) < 0.18 ? 0 : value;
  }

  private startRun(): void {
    if (this.startingRun) return;
    this.startingRun = true;
    this.scene.start('CombatScene');
  }

  private purchaseUpgrade(upgrade: PermanentUpgrade): void {
    const next = buyPermanentUpgrade(this.progression, upgrade.id as PermanentUpgradeId);
    if (!next) return;
    this.progression = next;
    this.saveUnavailable = !saveProgression(next);
    this.registry.set('progressionSave', next);
    this.registry.set('saveUnavailable', this.saveUnavailable);
    this.resourcesText.setText(formatMoney(next.resources));
    this.shopBalanceText.setText(formatMoney(next.resources));
    this.shopAvailabilityText?.setText(`AVAILABLE  ${formatMoney(next.resources)}  ·  EFFECTIVE NEXT RUN`);
    this.renderShopRows();
    this.showMessage(`${upgrade.title} PURCHASED · EFFECTIVE NEXT RUN`);
  }

  private showRunSummary(summary?: RunSummary): void {
    const message = summary
      ? `${summary.result === 'clear' ? 'RUN COMPLETE' : summary.result === 'abandoned' ? 'RUN ABANDONED' : 'RUN ENDED'}  ·  ${summary.roomsCleared} / ${ROOMS.length} ROOMS  ·  +${formatMoney(summary.resourcesEarned)}`
      : 'MOVE WITH ARROWS / ZQSD · WALK TO THE DOOR, THEN PRESS E';
    this.messageText = this.add
      .text(480, 479, message, {
        fontFamily: 'DM Mono, monospace',
        fontSize: '8px',
        color: '#b9a47f',
        letterSpacing: 0.5,
      })
      .setOrigin(0.5)
      .setDepth(400);
    if (summary?.saveUnavailable || this.saveUnavailable) {
      this.messageText.setText('SAVE UNAVAILABLE · PROGRESS MAY NOT SURVIVE A RELOAD').setColor('#e1896b');
    }
  }

  private showMessage(message: string): void {
    this.messageText.setText(message);
    this.tweens.killTweensOf(this.messageText);
    this.messageText.setAlpha(1);
    this.tweens.add({
      targets: this.messageText,
      alpha: { from: 1, to: 0.72 },
      duration: 850,
      yoyo: true,
      repeat: 1,
    });
  }
}

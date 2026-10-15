import Phaser from 'phaser';
import { ARENA, type ArenaObstacle } from './data/arena';
import { COLORS, ENEMIES, PLAYER } from './data/balance';
import { Sound } from './presentation/sound';
import { createFighter, destroyActorShadow, drawArena, drawEnemyMark, drawHealthBar, drawPlayerMark, syncActorShadow } from './presentation/visuals';

type EnemyKind = 'melee' | 'ranged';
type RunState = 'playing' | 'dead' | 'clear';

interface Enemy {
  id: string;
  kind: EnemyKind;
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
  aimX: number;
  aimY: number;
}

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
  private queuedDash = false;
  private previousPadButtons: boolean[] = [];
  private readonly handlePointerDown = (pointer: Phaser.Input.Pointer): void => {
    if (this.state !== 'playing') return;
    if (pointer.button === 0) this.queuedPrimary = true;
    if (pointer.button === 2) this.queuedHeavy = true;
  };
  private readonly handlePointerMove = (): void => { this.pointerHasAimed = true; };
  private keys!: Record<'up' | 'down' | 'left' | 'right' | 'primary' | 'heavy' | 'dash' | 'restart', Phaser.Input.Keyboard.Key>;
  private arrowKeys!: Record<'up' | 'down' | 'left' | 'right', Phaser.Input.Keyboard.Key>;
  private healthText!: Phaser.GameObjects.Text;
  private enemyText!: Phaser.GameObjects.Text;
  private comboText!: Phaser.GameObjects.Text;
  private actionText!: Phaser.GameObjects.Text;
  private dashText!: Phaser.GameObjects.Text;
  private hintText!: Phaser.GameObjects.Text;
  private damageFlash = 0;

  constructor() {
    super('CombatScene');
  }

  create(): void {
    this.resetRunState();
    this.input.off('pointerdown', this.handlePointerDown);
    this.input.off('pointermove', this.handlePointerMove);
    this.physics.world.setBounds(ARENA.bounds.x, ARENA.bounds.y, ARENA.bounds.width, ARENA.bounds.height);
    drawArena(this);
    this.createObstacles();
    this.createActors();
    this.createHud();
    this.setupInput();
    this.sfx = new Sound();
    this.events.once(Phaser.Scenes.Events.SHUTDOWN, () => {
      this.input.off('pointerdown', this.handlePointerDown);
      this.input.off('pointermove', this.handlePointerMove);
      this.sfx.destroy();
    });
  }

  private resetRunState(): void {
    this.state = 'playing';
    this.playerHealth = PLAYER.maxHealth;
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
    this.attackEffectAngle = 0;
    this.attackEffectKind = 'primary';
    this.playerAngle = 0;
    this.pointerHasAimed = false;
    this.queuedPrimary = false;
    this.queuedHeavy = false;
    this.queuedDash = false;
    this.previousPadButtons = [];
    this.damageFlash = 0;
    this.enemies = [];
    this.projectiles = [];
    this.obstacles = [];
    this.obstacleBodies = [];
    this.cameras.main.setScroll(0, 0).resetFX();
  }

  update(_time: number, deltaMs: number): void {
    if (this.state !== 'playing') {
      if (Phaser.Input.Keyboard.JustDown(this.keys.restart)) this.scene.restart();
      return;
    }

    const delta = Math.min(deltaMs / 1000, 0.05);
    const controls = this.readInput();
    this.updateTimers(delta);
    this.updateAim(controls.aimX, controls.aimY);
    this.updatePlayer(controls, delta);
    this.updateEnemies(delta);
    this.updateProjectiles(delta);
    this.updatePresentation(delta);

    if (this.enemies.length === 0) this.finish('clear');
  }

  private createObstacles(): void {
    for (const obstacle of ARENA.obstacles) {
      this.obstacles.push(obstacle);
      const shadow = this.add.rectangle(obstacle.x + 4, obstacle.y + 8, obstacle.width + 5, obstacle.height + 4, 0x090a0c, 0.55);
      shadow.setDepth(obstacle.y - obstacle.height / 2);
      const block = this.add.rectangle(obstacle.x, obstacle.y, obstacle.width, obstacle.height, 0x544c43, 1);
      block.setStrokeStyle(2, 0x82745f, 0.85).setDepth(obstacle.y + obstacle.height / 2);
      const top = this.add.rectangle(obstacle.x, obstacle.y - obstacle.height / 2 + 5, obstacle.width - 8, 4, 0x9c8867, 0.7);
      top.setDepth(obstacle.y + obstacle.height / 2 + 1);
      this.physics.add.existing(block, true);
      this.obstacleBodies.push(block);
    }
  }

  private createActors(): void {
    this.player = createFighter(this, ARENA.playerStart.x, ARENA.playerStart.y, 14, 0xd9c7a4, 0xf2e6ce);
    this.physics.add.existing(this.player);
    this.playerBody = this.player.body as Phaser.Physics.Arcade.Body;
    this.playerBody.setCircle(13).setCollideWorldBounds(true).setMaxVelocity(PLAYER.dashSpeed);
    for (const block of this.obstacleBodies) this.physics.add.collider(this.player, block);

    this.playerMarks = this.add.graphics().setDepth(30);
    this.arenaMarks = this.add.graphics().setDepth(25);
    this.fx = this.add.graphics().setDepth(40);

    for (const entry of ARENA.enemies) {
      const kind = entry.kind;
      const data = ENEMIES[kind];
      const body = createFighter(this, entry.x, entry.y, kind === 'melee' ? 16 : 15, data.tint, kind === 'melee' ? 0xe29a70 : 0xd2b9ec);
      const marks = this.add.graphics().setDepth(entry.y + 2);
      this.physics.add.existing(body);
      const physicsBody = body.body as Phaser.Physics.Arcade.Body;
      physicsBody.setCircle(kind === 'melee' ? 15 : 14).setCollideWorldBounds(true);
      for (const block of this.obstacleBodies) this.physics.add.collider(body, block);
      this.enemies.push({
        id: entry.id,
        kind,
        body,
        marks,
        hp: data.maxHealth,
        maxHp: data.maxHealth,
        attackCooldown: kind === 'melee' ? 0.9 : 1.25,
        windup: 0,
        targetX: entry.x,
        targetY: entry.y,
        stun: 0,
        flash: 0,
      });
    }

  }

  private createHud(): void {
    const fixed = (text: string, x: number, y: number, size: string, color: string, extra: Record<string, unknown> = {}) =>
      this.add.text(x, y, text, {
        fontFamily: '"DM Mono", ui-monospace, monospace',
        fontSize: size,
        color,
        letterSpacing: 1,
        ...extra,
      }).setScrollFactor(0).setDepth(1000);

    fixed('THE ARENA', 34, 27, '11px', '#d6b678', { fontStyle: 'bold' });
    fixed('STAY MOVING', 34, 44, '9px', '#8b8882');
    this.healthText = fixed('VITALITY  100 / 100', 34, 66, '10px', '#e9e1d4');
    this.enemyText = fixed('HOSTILES  02', 758, 28, '10px', '#e9e1d4', { align: 'right' });
    fixed('BREAK THE SEAL', 758, 45, '9px', '#8b8882', { align: 'right' });
    this.comboText = fixed('COMBO  —', 34, 493, '9px', '#d6b678');
    this.dashText = fixed('DASH', 758, 477, '8px', '#8b8882');
    this.actionText = fixed('HEAVY READY', 758, 497, '8px', '#d7cdbd');
    this.hintText = fixed('ARROWS / ZQSD MOVE  ·  MOUSE AIM  ·  J ATTACK  ·  K HEAVY  ·  SPACE DASH', 480, 519, '8px', '#77736d', { align: 'center' });
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
    };
    this.arrowKeys = {
      up: keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.UP),
      down: keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.DOWN),
      left: keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.LEFT),
      right: keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.RIGHT),
    };
  }

  private readInput(): InputFrame {
    let moveX = Number(this.keys.right.isDown || this.arrowKeys.right.isDown) - Number(this.keys.left.isDown || this.arrowKeys.left.isDown);
    let moveY = Number(this.keys.down.isDown || this.arrowKeys.down.isDown) - Number(this.keys.up.isDown || this.arrowKeys.up.isDown);

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
    const heavyPressed = Phaser.Input.Keyboard.JustDown(this.keys.heavy)
      || this.queuedHeavy
      || justPressed(3)
      || justPressed(6);
    const dashPressed = Phaser.Input.Keyboard.JustDown(this.keys.dash)
      || this.queuedDash
      || justPressed(1);

    this.queuedPrimary = false;
    this.queuedHeavy = false;
    this.queuedDash = false;
    this.previousPadButtons = buttons;
    return { moveX, moveY, primary, heavyPressed, dashPressed, aimX, aimY };
  }

  private updateAim(x: number, y: number): void {
    if (Math.hypot(x, y) > 0.15) this.playerAngle = Math.atan2(y, x);
  }

  private updateTimers(delta: number): void {
    this.primaryCooldown = Math.max(0, this.primaryCooldown - delta);
    this.comboTimer = Math.max(0, this.comboTimer - delta);
    if (this.comboTimer === 0) this.comboIndex = 0;
    this.dashCooldown = Math.max(0, this.dashCooldown - delta);
    this.dashTime = Math.max(0, this.dashTime - delta);
    this.invulnerability = Math.max(0, this.invulnerability - delta);
    this.heavyRecovery = Math.max(0, this.heavyRecovery - delta);
    this.damageFlash = Math.max(0, this.damageFlash - delta);
    this.attackEffect = Math.max(0, this.attackEffect - delta);
  }

  private updatePlayer(controls: InputFrame, delta: number): void {
    if (controls.dashPressed && this.dashCooldown <= 0 && this.heavyWindup <= 0 && this.heavyRecovery <= 0) {
      const directionLength = Math.hypot(controls.moveX, controls.moveY);
      this.dashX = directionLength > 0.1 ? controls.moveX / directionLength : controls.aimX;
      this.dashY = directionLength > 0.1 ? controls.moveY / directionLength : controls.aimY;
      this.dashTime = PLAYER.dashDuration;
      this.dashCooldown = PLAYER.dashCooldown;
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

    if (controls.primary && this.primaryCooldown <= 0 && this.heavyWindup <= 0 && this.heavyRecovery <= 0 && this.dashTime <= 0) {
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
      this.playerBody.setVelocity(controls.moveX * PLAYER.moveSpeed * actionSpeed, controls.moveY * PLAYER.moveSpeed * actionSpeed);
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
    const reach = isHeavy ? PLAYER.heavyRange : PLAYER.primaryRange + (combo === 2 ? 8 : 0);
    const halfArc = isHeavy ? 0.68 : combo === 2 ? 0.72 : 0.58;
    const damage = isHeavy ? PLAYER.heavyDamage : PLAYER.primaryDamage[combo];
    this.attackEffect = isHeavy ? 0.22 : 0.14;
    this.attackEffectAngle = this.playerAngle;
    this.attackEffectKind = kind;
    this.sfx.play(isHeavy ? 'heavy' : 'primary');
    this.cameras.main.shake(isHeavy ? 95 : 48, isHeavy ? 0.0024 : 0.0012);

    for (const enemy of [...this.enemies]) {
      const dx = enemy.body.x - this.player.x;
      const dy = enemy.body.y - this.player.y;
      const distance = Math.hypot(dx, dy);
      const angleDifference = Phaser.Math.Angle.Wrap(Math.atan2(dy, dx) - this.playerAngle);
      if (distance > reach + (enemy.kind === 'melee' ? 15 : 14) || Math.abs(angleDifference) > halfArc) continue;
      if (!this.lineClear(this.player.x, this.player.y, enemy.body.x, enemy.body.y)) continue;
      this.damageEnemy(enemy, damage, dx / (distance || 1), dy / (distance || 1));
    }
  }

  private damageEnemy(enemy: Enemy, damage: number, knockX: number, knockY: number): void {
    enemy.hp -= damage;
    enemy.flash = 0.12;
    enemy.stun = Math.max(enemy.stun, 0.13);
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

  private updateEnemies(delta: number): void {
    for (const enemy of this.enemies) {
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

      if (enemy.windup > 0) {
        enemy.windup = Math.max(0, enemy.windup - delta);
        body.setVelocity(0, 0);
        if (enemy.windup === 0) this.resolveEnemyAttack(enemy);
        continue;
      }
      if (enemy.stun > 0) continue;

      if (enemy.kind === 'melee') {
        if (distance <= ENEMIES.melee.attackRange && enemy.attackCooldown <= 0) {
          enemy.windup = ENEMIES.melee.attackWindup;
          enemy.targetX = this.player.x;
          enemy.targetY = this.player.y;
          body.setVelocity(0, 0);
          continue;
        }
        if (distance > ENEMIES.melee.attackRange - 8) body.setVelocity((dx / distance) * data.moveSpeed, (dy / distance) * data.moveSpeed);
        else body.setVelocity(0, 0);
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
    enemy.attackCooldown = ENEMIES[enemy.kind].attackCooldown;
    if (enemy.kind === 'melee') {
      if (Phaser.Math.Distance.Between(this.player.x, this.player.y, enemy.targetX, enemy.targetY) < ENEMIES.melee.attackRadius) {
        this.hurtPlayer(ENEMIES.melee.damage, enemy.targetX, enemy.targetY);
      }
    } else {
      this.spawnProjectile(enemy.body.x, enemy.body.y, enemy.targetX, enemy.targetY);
    }
  }

  private spawnProjectile(x: number, y: number, targetX: number, targetY: number): void {
    const dx = targetX - x;
    const dy = targetY - y;
    const length = Math.hypot(dx, dy) || 1;
    const body = this.add.circle(x, y, 7, COLORS.damage, 1).setStrokeStyle(2, 0xffb188, 1).setDepth(35);
    this.projectiles.push({
      body,
      vx: (dx / length) * ENEMIES.ranged.projectileSpeed,
      vy: (dy / length) * ENEMIES.ranged.projectileSpeed,
      life: 2.8,
    });
  }

  private updateProjectiles(delta: number): void {
    for (let index = this.projectiles.length - 1; index >= 0; index -= 1) {
      const projectile = this.projectiles[index];
      projectile.body.x += projectile.vx * delta;
      projectile.body.y += projectile.vy * delta;
      projectile.life -= delta;
      const hitsPlayer = Phaser.Math.Distance.Between(projectile.body.x, projectile.body.y, this.player.x, this.player.y) < 21;
      const blocked = this.obstacles.some((obstacle) =>
        projectile.body.x > obstacle.x - obstacle.width / 2 - 7
        && projectile.body.x < obstacle.x + obstacle.width / 2 + 7
        && projectile.body.y > obstacle.y - obstacle.height / 2 - 7
        && projectile.body.y < obstacle.y + obstacle.height / 2 + 7,
      );
      if (hitsPlayer) this.hurtPlayer(ENEMIES.ranged.damage, projectile.body.x, projectile.body.y);
      if (hitsPlayer || blocked || projectile.life <= 0 || projectile.body.x < 20 || projectile.body.x > 940 || projectile.body.y < 20 || projectile.body.y > 520) {
        projectile.body.destroy();
        this.projectiles.splice(index, 1);
      }
    }
  }

  private hurtPlayer(damage: number, sourceX: number, sourceY: number): void {
    if (this.invulnerability > 0 || this.state !== 'playing') return;
    this.playerHealth = Math.max(0, this.playerHealth - damage);
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
    drawPlayerMark(this.playerMarks, this.player.x, this.player.y - 2, this.playerAngle, this.heavyWindup > 0 ? COLORS.damage : 0x4a4034);

    this.arenaMarks.clear();
    for (const enemy of this.enemies) {
      enemy.marks.clear().setDepth(enemy.body.y + 2);
      drawEnemyMark(enemy.marks, enemy.body.x, enemy.body.y - 1, enemy.kind, 0x241f1b);
      drawHealthBar(enemy.marks, enemy.body.x - 18, enemy.body.y - 28, 36, enemy.hp / enemy.maxHp, enemy.kind === 'melee' ? 0xd7775d : 0xb29bd0);
      if (enemy.windup > 0) {
        if (enemy.kind === 'melee') {
          this.arenaMarks.lineStyle(2, COLORS.damage, 0.68).strokeCircle(enemy.targetX, enemy.targetY, ENEMIES.melee.attackRadius);
          this.arenaMarks.lineStyle(1, 0xffca9a, 0.3).strokeCircle(enemy.targetX, enemy.targetY, ENEMIES.melee.attackRadius + 9);
        } else {
          this.arenaMarks.lineStyle(2, COLORS.damage, 0.52).lineBetween(enemy.body.x, enemy.body.y, enemy.targetX, enemy.targetY);
          this.arenaMarks.fillStyle(COLORS.damage, 0.58).fillCircle(enemy.targetX, enemy.targetY, 8);
        }
      }
    }

    if (this.heavyWindup > 0) {
      const endX = this.player.x + Math.cos(this.playerAngle) * 92;
      const endY = this.player.y + Math.sin(this.playerAngle) * 92;
      this.arenaMarks.lineStyle(6, COLORS.damage, 0.32).lineBetween(this.player.x, this.player.y, endX, endY);
    }

    this.fx.clear().setDepth(this.player.y + 8);
    if (this.attackEffect > 0) {
      const heavy = this.attackEffectKind === 'heavy';
      const angle = this.attackEffectAngle;
      const length = heavy ? 82 : 60;
      const side = heavy ? 0.72 : 0.49;
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
    this.hud.fillStyle(healthColor, 1).fillRect(36, 86, 154 * (this.playerHealth / PLAYER.maxHealth), 3);
    this.hud.fillStyle(0x111214, 0.95).fillRect(802, 478, 124, 8);
    const dashProgress = 1 - Phaser.Math.Clamp(this.dashCooldown / PLAYER.dashCooldown, 0, 1);
    this.hud.fillStyle(COLORS.gold, 1).fillRect(804, 480, 120 * dashProgress, 4);
    this.healthText.setText(`VITALITY  ${String(this.playerHealth).padStart(3, '0')} / ${PLAYER.maxHealth}`);
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
    this.overlay.removeAll(true);
    const shade = this.add.rectangle(0, 0, ARENA.width, ARENA.height, 0x0a0b0d, 0.76).setOrigin(0);
    const panel = this.add.rectangle(480, 271, 404, 198, 0x1c1c1e, 0.98).setStrokeStyle(1, 0x695b46, 0.9);
    const title = this.add.text(480, 220, state === 'dead' ? 'TOO LATE.' : 'ARENA CLEAR.', {
      fontFamily: 'Manrope, sans-serif', fontSize: '31px', fontStyle: 'bold', color: state === 'dead' ? '#e1896b' : '#e3c88f',
    }).setOrigin(0.5);
    const copy = this.add.text(480, 264, state === 'dead' ? 'The run ends here. Take one more.' : 'A clean fight. There is always another.', {
      fontFamily: 'DM Mono, monospace', fontSize: '10px', color: '#aaa39a', letterSpacing: 0.5,
    }).setOrigin(0.5);
    const prompt = this.add.text(480, 319, 'PRESS R   /   CLICK TO RUN IT AGAIN', {
      fontFamily: 'DM Mono, monospace', fontSize: '10px', color: '#e1d0ae', letterSpacing: 1.1, backgroundColor: '#342d25', padding: { x: 15, y: 10 },
    }).setOrigin(0.5).setInteractive({ useHandCursor: true });
    prompt.on('pointerdown', () => this.scene.restart());
    this.overlay.add([shade, panel, title, copy, prompt]);
    this.overlay.setVisible(true);
    this.sfx.play(state === 'dead' ? 'death' : 'clear');
  }

  private lineClear(x1: number, y1: number, x2: number, y2: number): boolean {
    const steps = Math.ceil(Phaser.Math.Distance.Between(x1, y1, x2, y2) / 8);
    for (let step = 1; step < steps; step += 1) {
      const fraction = step / steps;
      const x = Phaser.Math.Linear(x1, x2, fraction);
      const y = Phaser.Math.Linear(y1, y2, fraction);
      if (this.obstacles.some((obstacle) =>
        x > obstacle.x - obstacle.width / 2 && x < obstacle.x + obstacle.width / 2
        && y > obstacle.y - obstacle.height / 2 && y < obstacle.y + obstacle.height / 2,
      )) return false;
    }
    return true;
  }

  private deadzone(value: number): number {
    const magnitude = Math.abs(value);
    if (magnitude < 0.18) return 0;
    return Math.sign(value) * (magnitude - 0.18) / 0.82;
  }
}

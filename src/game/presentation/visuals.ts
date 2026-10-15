import Phaser from 'phaser';
import { ARENA } from '../data/arena';
import { COLORS, type EnemyKind } from '../data/balance';

export function drawArena(scene: Phaser.Scene): Phaser.GameObjects.Graphics {
  const art = scene.add.graphics().setDepth(-20);
  const tile = 48;

  art.fillStyle(COLORS.ink, 1).fillRect(0, 0, ARENA.width, ARENA.height);
  for (let row = 0; row < Math.ceil(ARENA.height / tile); row += 1) {
    for (let column = 0; column < Math.ceil(ARENA.width / tile); column += 1) {
      const color = (row + column) % 2 === 0 ? COLORS.floorA : COLORS.floorB;
      const x = column * tile;
      const y = row * tile;
      art.fillStyle(color, 1).fillRect(x + 2, y + 2, tile - 4, tile - 4);
      if ((row * 7 + column * 11) % 13 === 0) {
        art.fillStyle(0x3a3937, 0.38).fillRect(x + 9, y + 11, 2, 2);
        art.fillRect(x + 31, y + 29, 3, 2);
      }
    }
  }

  art.lineStyle(2, 0x70634f, 0.72);
  art.lineBetween(27, 27, 933, 27).lineBetween(933, 27, 933, 226);
  art.lineBetween(933, 314, 933, 513).lineBetween(933, 513, 27, 513);
  art.lineBetween(27, 513, 27, 27);
  art.lineStyle(1, 0x49433b, 0.78);
  art.lineBetween(33, 33, 927, 33).lineBetween(927, 33, 927, 228);
  art.lineBetween(927, 312, 927, 507).lineBetween(927, 507, 33, 507);
  art.lineBetween(33, 507, 33, 33);
  art.lineStyle(1, COLORS.seam, 0.75);
  for (let x = 68; x < 920; x += 96) art.lineBetween(x, 37, x, 47);
  for (let x = 116; x < 900; x += 144) art.lineBetween(x, 493, x, 503);
  for (let y = 86; y < 490; y += 96) art.lineBetween(37, y, 47, y);
  for (let y = 130; y < 480; y += 144) art.lineBetween(913, y, 923, y);

  return art;
}

export function createFighter(
  scene: Phaser.Scene,
  x: number,
  y: number,
  radius: number,
  fill: number,
  outline: number,
): Phaser.GameObjects.Arc {
  const shadow = scene.add.ellipse(x + 2, y + 10, radius * 2.5, radius * 0.92, 0x08090b, 0.46);
  shadow.setDepth(y - 1);
  const actor = scene.add.circle(x, y, radius, fill, 1).setStrokeStyle(2, outline, 1);
  actor.setDepth(y + 1);
  actor.setData('shadow', shadow);
  return actor;
}

export function syncActorShadow(actor: Phaser.GameObjects.Arc): void {
  const shadow = actor.getData('shadow') as Phaser.GameObjects.Ellipse | undefined;
  shadow?.setPosition(actor.x + 2, actor.y + 10).setDepth(actor.y - 1);
}

export function destroyActorShadow(actor: Phaser.GameObjects.Arc): void {
  const shadow = actor.getData('shadow') as Phaser.GameObjects.Ellipse | undefined;
  shadow?.destroy();
}

export function drawPlayerMark(
  graphics: Phaser.GameObjects.Graphics,
  x: number,
  y: number,
  angle: number,
  color: number,
): void {
  const tip = 9;
  const side = 4;
  const cos = Math.cos(angle);
  const sin = Math.sin(angle);
  const px = -sin;
  const py = cos;
  graphics.fillStyle(color, 1);
  graphics.fillTriangle(
    x + cos * tip,
    y + sin * tip,
    x - cos * 3 + px * side,
    y - sin * 3 + py * side,
    x - cos * 3 - px * side,
    y - sin * 3 - py * side,
  );
}

export function drawEnemyMark(
  graphics: Phaser.GameObjects.Graphics,
  x: number,
  y: number,
  kind: EnemyKind,
  color: number,
): void {
  graphics.lineStyle(2, color, 1);
  if (kind === 'melee') {
    graphics.lineBetween(x - 6, y - 3, x, y - 8);
    graphics.lineBetween(x, y - 8, x + 6, y - 3);
    graphics.lineBetween(x - 5, y + 3, x + 5, y + 3);
  } else if (kind === 'ranged') {
    graphics.strokeRect(x - 5, y - 5, 10, 10);
    graphics.lineBetween(x - 3, y, x + 3, y);
    graphics.lineBetween(x, y - 3, x, y + 3);
  } else if (kind === 'heavy') {
    graphics.lineBetween(x, y - 8, x + 8, y);
    graphics.lineBetween(x + 8, y, x, y + 8);
    graphics.lineBetween(x, y + 8, x - 8, y);
    graphics.lineBetween(x - 8, y, x, y - 8);
  } else if (kind === 'charger') {
    graphics.fillTriangle(x + 9, y, x - 6, y - 7, x - 6, y + 7);
    graphics.lineBetween(x - 10, y - 6, x - 6, y);
    graphics.lineBetween(x - 10, y + 6, x - 6, y);
  } else if (kind === 'skirmisher') {
    graphics.lineBetween(x - 7, y + 6, x + 3, y - 6);
    graphics.lineBetween(x - 1, y + 7, x + 9, y - 5);
    graphics.lineBetween(x - 8, y - 3, x - 4, y - 7);
  } else {
    graphics.strokeCircle(x, y, 7);
    graphics.lineBetween(x - 5, y - 5, x + 5, y + 5);
    graphics.lineBetween(x + 5, y - 5, x - 5, y + 5);
  }
}

export function drawHealthBar(
  graphics: Phaser.GameObjects.Graphics,
  x: number,
  y: number,
  width: number,
  fraction: number,
  color: number,
): void {
  const clamped = Phaser.Math.Clamp(fraction, 0, 1);
  graphics.fillStyle(0x111214, 0.95).fillRect(x - 1, y - 1, width + 2, 6);
  graphics.fillStyle(color, 1).fillRect(x, y, width * clamped, 4);
}

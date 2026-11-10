//! The armor an enemy carries into a room, each layer of which soaks one hit.
//!
//! One icon per layer rides above the enemy's health bar and watches
//! `EnemyBody` rather than being told when its layer goes: the frame a blow
//! takes `armor` below it, the break starts here and the icon despawns once
//! it has played out.

use super::*;

pub(in crate::game) const SHIELD_SHEET: &str = "enemy_shield";

type ShieldedEnemiesQuery<'w, 's> = Query<
    'w,
    's,
    (&'static EnemyBody, &'static Transform),
    (Without<EnemyShieldIcon>, Without<DeadEnemy>),
>;
type ShieldIconsQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut Transform,
        &'static mut Sprite,
        &'static mut EnemyShieldIcon,
    ),
    Without<EnemyBody>,
>;

pub(in crate::game) fn update_enemy_shields(
    mut commands: Commands,
    time: Res<Time>,
    sheets: Res<SpriteSheets>,
    enemies: ShieldedEnemiesQuery,
    mut icons: ShieldIconsQuery,
) {
    let Some(sheet) = sheets.0.get(SHIELD_SHEET) else {
        return;
    };
    // Frame 0 is the shield itself; the rest are the break.
    let break_frames = sheet.frames.saturating_sub(1);
    for (icon_entity, mut transform, mut sprite, mut icon) in &mut icons {
        let Ok((enemy, enemy_transform)) = enemies.get(icon.enemy) else {
            commands.entity(icon_entity).despawn();
            continue;
        };
        if enemy.health <= 0.0 {
            commands.entity(icon_entity).despawn();
            continue;
        }
        transform.translation.x = enemy_transform.translation.x + icon.offset_x;
        transform.translation.y =
            enemy_transform.translation.y + enemy.visible_top_offset + SHIELD_OFFSET_Y;
        if icon.breaking.is_none() {
            if enemy.armor > icon.layer {
                continue;
            }
            icon.breaking = Some(Timer::from_seconds(
                break_frames as f32 / sheet.fps,
                TimerMode::Once,
            ));
        }
        let Some(timer) = icon.breaking.as_mut() else {
            continue;
        };
        timer.tick(time.delta());
        if timer.is_finished() {
            commands.entity(icon_entity).despawn();
            continue;
        }
        if let Some(atlas) = sprite.texture_atlas.as_mut() {
            let step = (timer.fraction() * break_frames as f32) as usize;
            atlas.index = 1 + step.min(break_frames.saturating_sub(1));
        }
    }
}

/// How far above the enemy's health bar the shield icons sit: the bar is six
/// units tall around its own center, and an icon is sixteen.
pub(in crate::game) const SHIELD_OFFSET_Y: f32 = 17.0;
/// How far apart the icons of two layers stand: an icon is twenty-one wide.
const SHIELD_SPACING_X: f32 = 22.0;

impl EnemyShieldIcon {
    /// The icon of one of an enemy's `layers`, placed so the row they make
    /// is centered over the health bar.
    pub(in crate::game) fn of_layer(enemy: Entity, layer: u8, layers: u8) -> Self {
        Self {
            enemy,
            layer,
            offset_x: (f32::from(layer) - f32::from(layers.saturating_sub(1)) / 2.0)
                * SHIELD_SPACING_X,
            breaking: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_break_steps_through_every_frame_after_the_shield() {
        let frames = 6;
        let break_frames = frames - 1;
        let indices: Vec<usize> = (0..break_frames)
            .map(|step| {
                let fraction = step as f32 / break_frames as f32;
                1 + ((fraction * break_frames as f32) as usize).min(break_frames - 1)
            })
            .collect();

        assert_eq!(indices, vec![1, 2, 3, 4, 5], "every break frame is shown");
    }
}

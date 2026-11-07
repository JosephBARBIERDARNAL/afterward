//! The shield a metal enemy carries into a room, which soaks its first hit.
//!
//! The icon rides above the enemy's health bar and watches `EnemyBody` rather
//! than being told when the shield goes: the frame a shot clears `shielded`,
//! the break starts here and the icon despawns once it has played out.

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
        transform.translation.x = enemy_transform.translation.x;
        transform.translation.y =
            enemy_transform.translation.y + enemy.visible_top_offset + SHIELD_OFFSET_Y;
        if icon.breaking.is_none() {
            if enemy.shielded {
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

/// How far above the enemy's health bar the shield icon sits: the bar is six
/// units tall around its own center, and the icon is sixteen.
const SHIELD_OFFSET_Y: f32 = 17.0;

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

//! What an idle enemy can see. An enemy only starts chasing the player it has
//! actually spotted: close enough — nearer still when it stands at the
//! enemy's back — roughly at its own height, and not behind a platform. The rules and their tuning live in
//! `enemies/enemy_types.yaml` under `detection:`.

use super::*;

impl EnemyBody {
    /// Whether this enemy, standing at `transform`, sees a player whose
    /// hurtbox is `player`. Sight runs between the two hurtbox centers, so a
    /// tall enemy looks out from higher up than a short one.
    pub(crate) fn sees_player(
        &self,
        transform: &Transform,
        player: Rect,
        detection: &EnemyDetectionConfig,
        platforms: &Platforms,
    ) -> bool {
        let eyes = self.hurtbox(transform).center();
        let target = player.center();
        let offset = target - eyes;
        offset.x.abs() <= detection.reach_h(offset.x * self.facing) * H_WORLD_UNITS
            && offset.y.abs() <= detection.height_h * H_WORLD_UNITS
            && !platforms.blocks_sight(eyes, target)
    }
}

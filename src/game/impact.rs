//! What a landed blow does to the moment: the game holds still for a few
//! frames and the camera is thrown off its mark, so a hit is felt rather than
//! only seen. Tuned from `player.melee.impact` of `config/gameplay.yaml`.

use super::*;

/// A blow that just landed. Anything that wants the moment marked spawns one;
/// `feel_the_impacts` takes it the next frame.
#[derive(Component)]
pub(crate) struct Impact {
    /// How long the game holds still.
    pub(crate) freeze_s: f32,
    /// How far the camera is thrown, in H.
    pub(crate) shake_h: f32,
}

/// The camera's tremble, worn by the camera itself. `camera::follow_camera`
/// adds `offset_h` to where the camera would otherwise stand.
#[derive(Component, Default)]
pub(crate) struct Shake {
    strength_h: f32,
    left_s: f32,
    pub(crate) offset_h: Vec2,
}

/// Holds the game still and shakes the camera for every `Impact`.
///
/// The freeze pauses the virtual clock, so everything that reads `Time` stops
/// together while input is still heard: a press made during it is buffered,
/// not lost. The freeze itself and the shake run on the real clock, which is
/// why the camera still trembles over a frozen frame.
pub(super) fn feel_the_impacts(
    mut commands: Commands,
    real: Res<Time<Real>>,
    mut clock: ResMut<Time<Virtual>>,
    data: Res<GameData>,
    impacts: Query<(Entity, &Impact)>,
    mut cameras: Query<&mut Shake>,
    mut frozen_s: Local<f32>,
) {
    let shake_s = data.gameplay.player.melee.impact.shake_s;
    for (entity, impact) in &impacts {
        *frozen_s = frozen_s.max(impact.freeze_s);
        for mut shake in &mut cameras {
            shake.strength_h = shake.strength_h.max(impact.shake_h);
            shake.left_s = shake_s;
        }
        commands.entity(entity).despawn();
    }
    if *frozen_s > 0.0 {
        if clock.is_paused() {
            *frozen_s -= real.delta_secs();
        } else {
            clock.pause();
        }
        if *frozen_s <= 0.0 {
            *frozen_s = 0.0;
            clock.unpause();
        }
    }
    let t = real.elapsed_secs();
    for mut shake in &mut cameras {
        shake.left_s = (shake.left_s - real.delta_secs()).max(0.0);
        if shake.left_s == 0.0 {
            shake.strength_h = 0.0;
            shake.offset_h = Vec2::ZERO;
            continue;
        }
        // Two sines that never line up: a tremble with no pattern to read,
        // dying away as it settles.
        let settling = shake.left_s / shake_s.max(f32::EPSILON);
        shake.offset_h =
            Vec2::new((t * 97.0).sin(), (t * 71.0 + 1.3).sin()) * shake.strength_h * settling;
    }
}

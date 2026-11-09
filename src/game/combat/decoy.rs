//! The double the Decoy bonus leaves behind. A roll plants a still copy of
//! the hero where it began; an engaged enemy walks at and swings at whichever
//! of the two stands nearer, and a blow that lands on the double costs the run
//! nothing but the double's own health.

use super::*;
use bevy::sprite::Anchor;

/// A standing double of the player: it has health and nothing else, never
/// moves, never strikes, and goes with the room it was planted in.
#[derive(Component)]
pub(crate) struct Decoy {
    pub(crate) health: f32,
}

impl Decoy {
    /// Its sheet's key in `SpriteSheets`: the hero raising an arm as the pale
    /// figure forms. It plays once and holds its last frame while it stands.
    pub(crate) const SHEET: &'static str = "player_summon";
}

/// The doubles standing in a room. The filters keep it disjoint from the
/// player, the enemies and the shots, so it sits beside their queries.
pub(super) type DecoyQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static Transform, &'static mut Decoy),
    (
        Without<PlayerBody>,
        Without<EnemyBody>,
        Without<CombatProjectile>,
    ),
>;

/// Plants the double the frame a roll begins, for a run that owns Decoy. It
/// stands on the spot `player/mod.rs` noted as the roll started rather than
/// where the first frame of the roll has already carried the player, and only
/// one stands at a time: a second roll replaces the first one's double.
pub(in crate::game) fn plant_decoy(
    mut commands: Commands,
    data: Res<GameData>,
    session: Res<RunSession>,
    sheets: Res<SpriteSheets>,
    mut players: Query<&mut PlayerBody>,
    standing: Query<Entity, With<Decoy>>,
) {
    let Ok(mut player) = players.single_mut() else {
        return;
    };
    let Some(origin) = player.roll_origin.take() else {
        return;
    };
    if !session.has_bonus("decoy") {
        return;
    }
    let Some(sheet) = sheets.0.get(Decoy::SHEET) else {
        return;
    };
    for entity in &standing {
        commands.entity(entity).despawn();
    }
    // Drawn exactly as `world::spawn_player` draws the player: the body
    // `body_height_h` tall, its feet on the collision box's baseline, and the
    // anchor at the box's center half an H above them.
    let animations = &data.player_animations;
    let mut sprite = Sprite::from_atlas_image(
        sheet.image.clone(),
        TextureAtlas {
            layout: sheet.layout.clone(),
            index: 0,
        },
    );
    sprite.custom_size = Some(
        sheet.frame_size * animations.body_height_h * H_WORLD_UNITS / animations.body_height_px,
    );
    sprite.flip_x = player.facing < 0.0;
    let px_per_h = animations.body_height_px / animations.body_height_h;
    let body_center_from_bottom_px = sheet.frame_size.y - animations.feet_row_px + px_per_h / 2.0;
    commands.spawn((
        sprite,
        Anchor(Vec2::new(
            0.0,
            body_center_from_bottom_px / sheet.frame_size.y - 0.5,
        )),
        // Just behind the player, who rolls out through it.
        Transform::from_xyz(origin.x, origin.y, 3.9),
        AnimatedSprite {
            sheet: Decoy::SHEET.to_string(),
            frame: 0,
            frames: sheet.frames,
            timer: Timer::from_seconds(1.0 / sheet.fps, TimerMode::Repeating),
            looping: false,
        },
        Decoy {
            health: data.bonus_effect("decoy", "decoy_health"),
        },
        RoomEntity,
    ));
}

/// Takes the blow a double caught and breaks it once its health is gone.
pub(super) fn hurt_the_decoy(
    commands: &mut Commands,
    sounds: &mut SoundQueue,
    decoys: &mut DecoyQuery,
    entity: Entity,
    damage: f32,
) {
    let Ok((_, transform, mut decoy)) = decoys.get_mut(entity) else {
        return;
    };
    let standing_at = transform.translation;
    decoy.health -= damage;
    let broken = decoy.health <= 0.0;
    spawn_damage_number(commands, standing_at, damage, true);
    sounds.push(SoundCue::Damage(DamageSound::Neutral));
    if broken {
        commands.entity(entity).despawn();
    }
}

/// What an engaged enemy goes after: the player, or a double standing nearer.
#[derive(Clone, Copy)]
pub(super) struct Mark {
    /// Where a strike or a shot has to land to hurt it.
    pub(super) hurtbox: Rect,
    pub(super) feet: Vec2,
    /// Whether it stands on a floor, and so sets a level to climb to.
    pub(super) grounded: bool,
    /// The double it is, if it is not the player.
    pub(super) decoy: Option<Entity>,
}

impl Mark {
    /// Whichever of this mark and the standing doubles is nearest `x`. A
    /// double stands still on its own feet, so it always counts as grounded.
    pub(super) fn nearest_to(self, x: f32, decoys: &DecoyQuery, data: &GameData) -> Self {
        decoys
            .iter()
            .map(|(entity, transform, _)| Self {
                hurtbox: player_hurtbox(transform.translation, data),
                feet: Vec2::new(
                    transform.translation.x,
                    transform.translation.y - 0.5 * H_WORLD_UNITS,
                ),
                grounded: true,
                decoy: Some(entity),
            })
            .fold(self, |nearest, double| {
                if (double.feet.x - x).abs() < (nearest.feet.x - x).abs() {
                    double
                } else {
                    nearest
                }
            })
    }

    /// Hurts what the blow reached. A double takes it in the player's place,
    /// so `session.health` is only ever touched for the player themselves.
    pub(super) fn strike(
        &self,
        resources: PlayerDamageResources<'_, '_, '_>,
        target: PlayerDamageTarget<'_>,
        decoys: &mut DecoyQuery,
        damage: f32,
    ) {
        match self.decoy {
            Some(entity) => {
                hurt_the_decoy(resources.commands, resources.sounds, decoys, entity, damage)
            }
            None => apply_player_damage(resources, target, damage),
        }
    }
}

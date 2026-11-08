//! The bonus item lying on a cleared room's `bonus` spot. Its category is
//! drawn when it appears; touching it opens the bonus screen, which then
//! offers two bonuses of that category.

use super::*;
use crate::game::ui_style::load_pixel_art;

#[derive(Component)]
pub(super) struct BonusPickup {
    pub(super) category: String,
    /// Where the item hovers around, in world units.
    base_y: f32,
    bob_phase_s: f32,
}

/// Lays the item out once the room is cleared, as long as no bonus has been
/// picked there yet. A room without enemies is cleared the moment it opens, so
/// its item is simply there from the start.
pub(super) fn spawn_bonus_pickup(
    mut commands: Commands,
    data: Res<GameData>,
    session: Res<RunSession>,
    asset_server: Res<AssetServer>,
    mut random: ResMut<RandomState>,
    pickups: Query<Entity, With<BonusPickup>>,
    mut sounds: ResMut<SoundQueue>,
) {
    if !session.room_cleared || session.door_open || !pickups.is_empty() {
        return;
    }
    let categories = &data.bonuses.categories;
    let index = random.index(categories.len());
    let Some((category, config)) = categories.iter().nth(index) else {
        return;
    };
    let room = &data.rooms[&session.room_id];
    let item = &data.gameplay.room_flow.bonus_pickup;
    let size = item.size_h * H_WORLD_UNITS;
    let base_y = (room.bonus.y_h + item.size_h / 2.0) * H_WORLD_UNITS;
    let mut sprite = Sprite::from_image(load_pixel_art(&asset_server, config.pickup.clone()));
    sprite.custom_size = Some(Vec2::splat(size));
    commands.spawn((
        sprite,
        Transform::from_xyz(room.bonus.x_h * H_WORLD_UNITS, base_y, 2.5),
        BonusPickup {
            category: category.clone(),
            base_y,
            bob_phase_s: 0.0,
        },
        RoomEntity,
    ));
    sounds.push(SoundCue::BonusAppear);
}

#[derive(SystemParam)]
pub(super) struct BonusPickupResources<'w> {
    time: Res<'w, Time>,
    data: Res<'w, GameData>,
    random: ResMut<'w, RandomState>,
    session: ResMut<'w, RunSession>,
    next_screen: ResMut<'w, NextState<GameScreen>>,
    sounds: ResMut<'w, SoundQueue>,
}

/// Hovers the item, and opens the bonus screen when the player touches it.
pub(super) fn update_bonus_pickup(
    resources: BonusPickupResources,
    mut commands: Commands,
    player: Query<&Transform, (With<PlayerBody>, Without<BonusPickup>)>,
    mut pickups: Query<(Entity, &mut Transform, &mut BonusPickup)>,
) {
    let BonusPickupResources {
        time,
        data,
        mut random,
        mut session,
        mut next_screen,
        mut sounds,
    } = resources;
    let item = &data.gameplay.room_flow.bonus_pickup;
    let Ok(player) = player.single() else {
        return;
    };
    for (entity, mut transform, mut pickup) in &mut pickups {
        pickup.bob_phase_s = (pickup.bob_phase_s + time.delta_secs()) % item.bob_period_s;
        let turn = std::f32::consts::TAU * pickup.bob_phase_s / item.bob_period_s;
        transform.translation.y = pickup.base_y + turn.sin() * item.bob_height_h * H_WORLD_UNITS;
        if (transform.translation.truncate() - player.translation.truncate()).length()
            > item.touch_radius_h * H_WORLD_UNITS
        {
            continue;
        }
        session.bonus_category = Some(pickup.category.clone());
        session.offered_bonuses =
            flow::draw_bonus_pair(&data, &session, &mut random, &BTreeSet::new());
        session.previous_offers.clear();
        let offered = session.offered_bonuses.clone();
        session.previous_offers.extend(offered);
        session.selected_choice = bonus_screen::BonusChoice::Offer(0);
        sounds.push(SoundCue::Pickup);
        commands.entity(entity).despawn();
        next_screen.set(GameScreen::Bonus);
        return;
    }
}

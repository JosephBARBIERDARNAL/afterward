//! The bonus item hovering over a cleared room's `bonus` spot: the same
//! obelisk every time, glowing in the colour of the category drawn when it
//! appears. Interact pressed within reach of it opens the bonus screen, which
//! then offers two bonuses of that category.

use super::*;

#[derive(Component)]
pub(super) struct BonusPickup {
    pub(super) category: String,
}

impl BonusPickup {
    /// Its sheet's key in `SpriteSheets`.
    pub(super) const SHEET: &'static str = "bonus_pickup";
}

/// Lays the item out once the room is cleared, as long as no bonus has been
/// picked there yet. A room without enemies is cleared the moment it opens, so
/// its item is simply there from the start. A run is longer than the bonuses
/// are many: once no category can still offer a choice of two, the room lays
/// nothing and its door simply opens.
pub(super) fn spawn_bonus_pickup(
    mut commands: Commands,
    resources: BonusPickupResources,
    sheets: Res<SpriteSheets>,
    pickups: Query<Entity, With<BonusPickup>>,
    mut doors: Query<&mut Sprite, With<ExitDoor>>,
) {
    let BonusPickupResources {
        data,
        mut random,
        mut session,
        mut sounds,
        ..
    } = resources;
    if !session.room_cleared || session.door_open || !pickups.is_empty() {
        return;
    }
    let categories = data
        .bonuses
        .categories
        .iter()
        .filter(|(name, _)| {
            data.bonuses
                .bonuses
                .iter()
                .filter(|bonus| bonus.category == **name)
                .filter(|bonus| !session.owned_bonuses.contains(&bonus.id))
                .count()
                >= 2
        })
        .collect::<Vec<_>>();
    if categories.is_empty() {
        for mut sprite in &mut doors {
            if let Some(atlas) = sprite.texture_atlas.as_mut() {
                atlas.index = ExitDoor::UNLOCKED_FRAME;
            }
        }
        session.door_open = true;
        sounds.push(SoundCue::DoorAppear);
        return;
    }
    let (category, config) = categories[random.index(categories.len())];
    let room = &data.rooms[&session.room_id];
    let item = &data.gameplay.room_flow.bonus_pickup;
    let Some(sheet) = sheets.0.get(BonusPickup::SHEET) else {
        return;
    };
    let height = item.height_h * H_WORLD_UNITS;
    let mut sprite = Sprite::from_atlas_image(
        sheet.image.clone(),
        TextureAtlas {
            layout: sheet.layout.clone(),
            index: 0,
        },
    );
    sprite.custom_size = Some(Vec2::new(
        height * sheet.frame_size.x / sheet.frame_size.y,
        height,
    ));
    commands.spawn((
        sprite,
        AnimatedSprite {
            sheet: BonusPickup::SHEET.into(),
            frame: 0,
            frames: sheet.frames,
            timer: Timer::from_seconds(1.0 / sheet.fps, TimerMode::Repeating),
            looping: true,
        },
        Transform::from_xyz(
            room.bonus.x_h * H_WORLD_UNITS,
            (room.bonus.y_h + item.height_h / 2.0) * H_WORLD_UNITS,
            2.5,
        ),
        BonusPickup {
            category: category.clone(),
        },
        Interactable {
            reach_h: Vec2::splat(item.reach_h),
            top_h: item.height_h / 2.0,
        },
        Glowing {
            color: Color::from(config.color),
            radius_h: data.gameplay.effects.glow.bonus_radius_h,
        },
        RoomEntity,
    ));
    sounds.push(SoundCue::BonusAppear);
}

#[derive(SystemParam)]
pub(super) struct BonusPickupResources<'w> {
    data: Res<'w, GameData>,
    keys: Res<'w, ButtonInput<KeyCode>>,
    controls: Res<'w, ControlBindings>,
    in_reach: ResMut<'w, InReach>,
    random: ResMut<'w, RandomState>,
    session: ResMut<'w, RunSession>,
    next_screen: ResMut<'w, NextState<GameScreen>>,
    sounds: ResMut<'w, SoundQueue>,
}

/// Opens the bonus screen when Interact is pressed within reach of the item.
pub(super) fn update_bonus_pickup(
    resources: BonusPickupResources,
    mut commands: Commands,
    pickups: Query<(Entity, &BonusPickup)>,
) {
    let BonusPickupResources {
        data,
        keys,
        controls,
        mut in_reach,
        mut random,
        mut session,
        mut next_screen,
        mut sounds,
    } = resources;
    for (entity, pickup) in &pickups {
        if in_reach.0 != Some(entity) || !controls.just_pressed(GameAction::Interact, &keys) {
            continue;
        }
        in_reach.0 = None;
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

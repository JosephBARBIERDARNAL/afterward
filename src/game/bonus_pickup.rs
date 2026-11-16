//! The bonus item hovering over a cleared room's `bonus` spot: the same
//! obelisk every time. Interact pressed within reach of it opens the bonus
//! screen, which then offers three bonuses, each at the rarity it was drawn
//! at.

use super::*;

#[derive(Component)]
pub(super) struct BonusPickup;

impl BonusPickup {
    /// Its sheet's key in `SpriteSheets`.
    pub(super) const SHEET: &'static str = "bonus_pickup";
}

/// Lays the item out once the room is cleared, as long as no bonus has been
/// picked there yet. A room without enemies is cleared the moment it opens, so
/// its item is simply there from the start. A run is longer than the bonuses
/// are many: once the pool cannot still offer a choice of two, the room lays
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
        mut session,
        mut sounds,
        ..
    } = resources;
    if !session.room_cleared || session.door_open || !pickups.is_empty() {
        return;
    }
    if !session.has_offers(&data) {
        for mut sprite in &mut doors {
            if let Some(atlas) = sprite.texture_atlas.as_mut() {
                atlas.index = ExitDoor::UNLOCKED_FRAME;
            }
        }
        session.door_open = true;
        sounds.push(SoundCue::DoorAppear);
        return;
    }
    let color = Color::from(data.gameplay.effects.glow.bonus_color);
    let untouched = &data.gameplay.room_flow.untouched;
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
    let mut pickup = commands.spawn((
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
        BonusPickup,
        Interactable {
            reach_h: Vec2::splat(item.reach_h),
            verb: "take",
        },
        Glowing {
            color,
            radius_h: data.gameplay.effects.glow.bonus_radius_h
                * if session.flawless {
                    untouched.glow_multiplier
                } else {
                    1.0
                },
            brightness: 1.0,
        },
        RoomEntity,
    ));
    // The item of a room cleared untouched says what that earned.
    if session.flawless {
        pickup.with_child((
            Text2d::new(untouched.label.clone()),
            TextFont {
                font_size: bevy::text::FontSize::Px(14.0),
                ..default()
            },
            TextColor(color),
            FontRole::Display,
            Transform::from_xyz(0.0, (item.height_h / 2.0 + 0.9) * H_WORLD_UNITS, 0.5),
        ));
    }
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
    pickups: Query<Entity, With<BonusPickup>>,
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
    for entity in &pickups {
        if in_reach.0 != Some(entity) || !controls.just_pressed(GameAction::Interact, &keys) {
            continue;
        }
        in_reach.0 = None;
        session.offered_bonuses =
            flow::draw_bonus_offers(&data, &session, &mut random, &BTreeSet::new());
        session.previous_offers = session
            .offered_bonuses
            .iter()
            .map(|offer| offer.id.clone())
            .collect();
        session.selected_choice = bonus_screen::BonusChoice::Offer(0);
        sounds.push(SoundCue::Pickup);
        commands.entity(entity).despawn();
        next_screen.set(GameScreen::Bonus);
        return;
    }
}

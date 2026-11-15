//! Rings on the edge of the screen pointing at the enemies of the wave being
//! fought that stand out of view, each showing the head of the one it stands
//! for (`head_px` of its type) and pinned where the line from the middle of
//! the screen to that enemy leaves it. Tuned from `enemy_markers:` of
//! `config/gameplay.yaml`, and turned off in the Gameplay section of the
//! settings.

use super::*;
use std::collections::HashMap;

/// A marker, the enemy it points at, the pointer on its rim, and the head
/// inside it, mirrored with the way its enemy faces.
#[derive(Component)]
pub(super) struct EnemyMarker {
    pub(super) enemy: Entity,
    pointer: Entity,
    head: Entity,
}

/// Spawns a marker for each enemy of the wave that has left the view, moves
/// it along the edge as the enemy or the camera moves, and takes it away once
/// the enemy is back in view, down, or the markers are turned off.
pub(super) fn update_enemy_markers(
    mut commands: Commands,
    (screen, settings, data, sheets): (
        Res<State<GameScreen>>,
        Res<PlayerSettings>,
        Res<GameData>,
        Res<SpriteSheets>,
    ),
    camera: Query<(&Camera, &Transform), With<Camera2d>>,
    enemies: Query<(Entity, &EnemyBody, &Transform, &Sprite), Without<Camera2d>>,
    mut markers: Query<(Entity, &EnemyMarker, &mut Node)>,
    mut pointers: Query<(&mut Node, &mut UiTransform), Without<EnemyMarker>>,
    mut heads: Query<&mut ImageNode>,
) {
    let config = &data.gameplay.enemy_markers;
    let radius = config.diameter_px / 2.0;
    // Where each enemy out of view is marked, in the viewport's UI pixels,
    // and which way from the middle of the screen it stands.
    let mut out_of_view = HashMap::new();
    if settings.enemy_markers
        && *screen.get() == GameScreen::Playing
        && let Ok((camera, camera_transform)) = camera.single()
    {
        // The view is the fixed 24 by 13.5 H around the camera, whatever the
        // window; a headless game has no screen, and its world units stand in
        // for the pixels.
        let world_view = Vec2::new(
            data.gameplay.viewport_width_h(),
            data.gameplay.viewport.height_h,
        ) * H_WORLD_UNITS;
        let view = camera.logical_viewport_size().unwrap_or(world_view);
        let pixels_per_unit = view / world_view;
        let middle = view / 2.0;
        // The ring's middle keeps this far inside each edge.
        let reach = (middle - Vec2::splat(config.edge_margin_px + radius)).max(Vec2::ZERO);
        for (entity, enemy, transform, sprite) in &enemies {
            if enemy.health <= 0.0 {
                continue;
            }
            let offset = (transform.translation - camera_transform.translation).truncate();
            if offset.abs().cmple(world_view / 2.0).all() {
                continue;
            }
            // UI y runs down.
            let toward = Vec2::new(offset.x, -offset.y) * pixels_per_unit;
            let along = (reach / toward.abs().max(Vec2::splat(f32::EPSILON))).min_element();
            out_of_view.insert(entity, (middle + toward * along, toward, sprite.flip_x));
        }
    }

    for (marker_entity, marker, mut node) in &mut markers {
        let Some((spot, toward, facing_left)) = out_of_view.remove(&marker.enemy) else {
            commands.entity(marker_entity).despawn();
            continue;
        };
        node.left = px(spot.x - radius);
        node.top = px(spot.y - radius);
        if let Ok((mut pointer, mut turn)) = pointers.get_mut(marker.pointer) {
            place_pointer(&mut pointer, &mut turn, toward, radius, config.pointer_px);
        }
        if let Ok(mut head) = heads.get_mut(marker.head) {
            head.flip_x = facing_left;
        }
    }

    // What is left out of view has no marker yet.
    for (enemy, (spot, toward, facing_left)) in out_of_view {
        let Ok((_, body, _, _)) = enemies.get(enemy) else {
            continue;
        };
        let Some(sheet) = sheets.0.get(body.type_id.idle_sheet()) else {
            continue;
        };
        // The head of the type's first idle frame, or the largest square in
        // the middle of the frame for a type that names none.
        let crop = data.enemies.types[&body.type_id].head_px.map_or_else(
            || {
                let side = sheet.frame_size.min_element();
                Rect::from_center_size(sheet.frame_size / 2.0, Vec2::splat(side))
            },
            |head| {
                Rect::from_corners(
                    Vec2::new(head.x, head.y),
                    Vec2::new(head.x + head.size, head.y + head.size),
                )
            },
        );
        let mut pointer_node = Node {
            position_type: PositionType::Absolute,
            width: px(config.pointer_px),
            height: px(config.pointer_px),
            ..default()
        };
        let mut pointer_turn = UiTransform::default();
        place_pointer(
            &mut pointer_node,
            &mut pointer_turn,
            toward,
            radius,
            config.pointer_px,
        );
        // Drawn before the ring, so only its tip shows past the rim.
        let pointer = commands
            .spawn((
                pointer_node,
                pointer_turn,
                BackgroundColor(Color::from(config.border_color)),
                Pickable::IGNORE,
            ))
            .id();
        let head = commands
            .spawn((
                Node {
                    width: percent(100),
                    height: percent(100),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                ImageNode {
                    texture_atlas: Some(TextureAtlas {
                        layout: sheet.layout.clone(),
                        index: 0,
                    }),
                    rect: Some(crop),
                    flip_x: facing_left,
                    ..ImageNode::new(sheet.image.clone())
                },
                Pickable::IGNORE,
            ))
            .id();
        let ring = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: percent(100),
                    height: percent(100),
                    border: UiRect::all(px(config.border_px)),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BackgroundColor(Color::from(config.fill_color)),
                BorderColor::all(Color::from(config.border_color)),
                Pickable::IGNORE,
            ))
            .add_child(head)
            .id();
        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(spot.x - radius),
                    top: px(spot.y - radius),
                    width: px(config.diameter_px),
                    height: px(config.diameter_px),
                    ..default()
                },
                EnemyMarker {
                    enemy,
                    pointer,
                    head,
                },
                Pickable::IGNORE,
            ))
            .add_children(&[pointer, ring]);
    }
}

/// Stands the pointer, a square turned onto its corner, on the ring's rim
/// with one corner aimed `toward` the enemy.
fn place_pointer(node: &mut Node, turn: &mut UiTransform, toward: Vec2, radius: f32, side: f32) {
    let direction = toward.normalize_or(Vec2::X);
    let middle = Vec2::splat(radius) + direction * radius;
    node.left = px(middle.x - side / 2.0);
    node.top = px(middle.y - side / 2.0);
    // The square's corner stands at 45 degrees; UI y runs down, so a
    // clockwise turn by the enemy's angle less that aims it.
    turn.rotation = Rot2::radians(direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_4);
}

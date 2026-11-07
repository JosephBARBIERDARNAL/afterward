//! Mouse editing: click to select (Shift+click to add or remove), drag empty
//! space to box-select, drag to move the selection, drag a selected block's
//! edge to resize, two-finger scroll or right/middle-drag to pan, and pinch or
//! the mouse wheel to zoom.

use super::*;
use bevy::{
    input::{
        gestures::PinchGesture,
        mouse::{AccumulatedMouseScroll, MouseScrollUnit},
    },
    window::PrimaryWindow,
};

/// How close to a selected block's edge, in H at 1x zoom, a press resizes it.
const EDGE_GRAB_H: f32 = 0.2;
/// Zoom change per wheel line.
const ZOOM_PER_LINE: f32 = 1.12;
/// Wheel lines of zoom per unit of pinch (its magnification change).
const PINCH_TO_LINES: f32 = 8.0;

#[derive(SystemParam)]
pub(super) struct EditorPointerResources<'w, 's> {
    mouse: Res<'w, ButtonInput<MouseButton>>,
    keys: Res<'w, ButtonInput<KeyCode>>,
    scroll: Res<'w, AccumulatedMouseScroll>,
    pinches: MessageReader<'w, 's, PinchGesture>,
    editor: ResMut<'w, RoomEditor>,
    data: ResMut<'w, GameData>,
    windows: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
    camera: Query<
        'w,
        's,
        (
            &'static Camera,
            &'static GlobalTransform,
            &'static mut Transform,
            &'static mut Projection,
        ),
        With<Camera2d>,
    >,
}

pub(super) fn handle_pointer(resources: EditorPointerResources) {
    let EditorPointerResources {
        mouse,
        keys,
        scroll,
        mut pinches,
        mut editor,
        mut data,
        windows,
        mut camera,
    } = resources;
    let (Ok(window), Ok((camera, camera_transform, mut transform, mut projection))) =
        (windows.single(), camera.single_mut())
    else {
        return;
    };
    let cursor_h = window
        .cursor_position()
        .and_then(|position| camera.viewport_to_world_2d(camera_transform, position).ok())
        .map(|world| world / H_WORLD_UNITS);
    editor.cursor_h = cursor_h;
    let Projection::Orthographic(orthographic) = &mut *projection else {
        return;
    };

    // A trackpad scrolls in pixels and pans, with the content following the
    // fingers; a mouse wheel scrolls in lines and zooms, as does a pinch.
    let mut zoom_lines = pinches
        .read()
        .map(|pinch| pinch.0 * PINCH_TO_LINES)
        .sum::<f32>();
    match scroll.unit {
        MouseScrollUnit::Line => zoom_lines += scroll.delta.y,
        MouseScrollUnit::Pixel => {
            let world_per_pixel = camera.logical_viewport_size().map_or(0.0, |size| {
                data.gameplay.viewport.height_h * H_WORLD_UNITS * orthographic.scale / size.y
            });
            transform.translation.x -= scroll.delta.x * world_per_pixel;
            transform.translation.y += scroll.delta.y * world_per_pixel;
        }
    }
    if zoom_lines.abs() > f32::EPSILON {
        let old_scale = orthographic.scale;
        orthographic.scale =
            (old_scale * ZOOM_PER_LINE.powf(-zoom_lines)).clamp(MIN_ZOOM, MAX_ZOOM);
        // Keep the point under the cursor (or the view's center) in place.
        let camera_h = transform.translation.truncate() / H_WORLD_UNITS;
        let anchor_h = cursor_h.unwrap_or(camera_h);
        let camera_h = anchor_h + (camera_h - anchor_h) * orthographic.scale / old_scale;
        transform.translation.x = camera_h.x * H_WORLD_UNITS;
        transform.translation.y = camera_h.y * H_WORLD_UNITS;
    }

    let Some(cursor_h) = cursor_h else {
        return;
    };
    if mouse.any_just_pressed([MouseButton::Right, MouseButton::Middle]) {
        editor.drag = Some(Drag::Pan { anchor: cursor_h });
    }
    if let Some(Drag::Pan { anchor }) = editor.drag {
        if mouse.any_pressed([MouseButton::Right, MouseButton::Middle]) {
            let shift = (anchor - cursor_h) * H_WORLD_UNITS;
            transform.translation.x += shift.x;
            transform.translation.y += shift.y;
        } else {
            editor.drag = None;
        }
        return;
    }

    let room_id = editor.room_id;
    let Some(room) = data.rooms.get_mut(&room_id) else {
        return;
    };
    let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let fine = keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]);

    if mouse.just_pressed(MouseButton::Left)
        && editor.text_entry.is_none()
        && !editor.pointer_over_panel
    {
        let grab_h = EDGE_GRAB_H * orthographic.scale;
        let only = match editor.selected.as_slice() {
            [item] if !shift && !item.is_body() => Some(*item),
            _ => None,
        };
        let resize = only
            .and_then(|item| item.bounds(room).map(|rect| (item, rect)))
            .and_then(|(item, rect)| {
                let reach = Rect::new(rect.x, rect.y, rect.right(), rect.top()).inflate(grab_h);
                let near = |edge: f32, at: f32| (edge - at).abs() <= grab_h;
                let edges = Edges {
                    left: near(rect.x, cursor_h.x),
                    right: near(rect.right(), cursor_h.x) && !near(rect.x, cursor_h.x),
                    bottom: near(rect.y, cursor_h.y),
                    top: near(rect.top(), cursor_h.y) && !near(rect.y, cursor_h.y),
                };
                (reach.contains(cursor_h) && edges != Edges::default()).then_some((item, edges))
            });
        if let Some((item, edges)) = resize {
            editor.drag = Some(Drag::Resize {
                item,
                edges,
                before: room.clone(),
            });
        } else {
            let picked = EditorItem::at(room, cursor_h);
            editor.text_entry = None;
            match picked {
                // Shift+click adds the item to the selection or takes it out.
                Some(item) if shift => {
                    match editor
                        .selected
                        .iter()
                        .position(|selected| *selected == item)
                    {
                        Some(position) => {
                            editor.selected.remove(position);
                        }
                        None => editor.selected.push(item),
                    }
                    editor.field = 0;
                }
                Some(item) => {
                    // Pressing a selected item drags the whole selection.
                    if !editor.selected.contains(&item) {
                        editor.select(Some(item));
                    }
                    if let EditorItem::Enemy(index) = item {
                        editor.enemy_type = room
                            .enemy_spawns
                            .get(index)
                            .map(|spawn| spawn.type_id.clone());
                    }
                    if let Some(material) = item.material(room) {
                        editor.material = Some(material.clone());
                    }
                    let starts = editor
                        .selected
                        .iter()
                        .filter_map(|selected| {
                            selected.origin(room).map(|origin| (*selected, origin))
                        })
                        .collect();
                    editor.drag = item.origin(room).map(|origin| Drag::Move {
                        item,
                        grab: cursor_h - origin,
                        starts,
                        before: room.clone(),
                    });
                }
                // Empty space starts a selection box; Shift keeps the selection.
                None => {
                    if !shift {
                        editor.select(None);
                    }
                    editor.drag = Some(Drag::Marquee { start: cursor_h });
                }
            }
        }
    }

    if mouse.pressed(MouseButton::Left) {
        let before = room.clone();
        match &editor.drag {
            Some(Drag::Move {
                item, grab, starts, ..
            }) => {
                let target = cursor_h - *grab;
                let dragged = Vec2::new(snap(target.x, fine), snap(target.y, fine));
                // The rest of the selection keeps its offset from the dragged item.
                let offset = starts
                    .iter()
                    .find(|(moved, _)| moved == item)
                    .map_or(Vec2::ZERO, |(_, start)| dragged - *start);
                for (moved, start) in starts {
                    let origin = if moved == item {
                        dragged
                    } else {
                        *start + offset
                    };
                    moved.set_origin(room, origin);
                }
            }
            Some(Drag::Resize { item, edges, .. }) => {
                if let Some(rect) = item.rect_mut(room) {
                    let (right, top) = (rect.right(), rect.top());
                    let x = snap(cursor_h.x, fine);
                    let y = snap(cursor_h.y, fine);
                    let min_size = if fine { FINE_SNAP_H } else { SNAP_H };
                    if edges.left {
                        rect.x = x.min(right - min_size);
                        rect.width = right - rect.x;
                    }
                    if edges.right {
                        rect.width = (x - rect.x).max(min_size);
                    }
                    if edges.bottom {
                        rect.y = y.min(top - min_size);
                        rect.height = top - rect.y;
                    }
                    if edges.top {
                        rect.height = (y - rect.y).max(min_size);
                    }
                }
            }
            Some(Drag::Pan { .. } | Drag::Marquee { .. }) | None => {}
        }
        if *room != before {
            editor.view_dirty = true;
        }
    }

    if mouse.just_released(MouseButton::Left) {
        let edited = room.clone();
        match editor.drag.take() {
            Some(Drag::Move { before, .. } | Drag::Resize { before, .. }) if edited != before => {
                editor.record(before);
            }
            Some(Drag::Marquee { start }) => {
                // Selects every item lying entirely inside the box.
                let area = Rect::from_corners(start, cursor_h);
                let boxed = EditorItem::all(room)
                    .filter(|item| {
                        item.bounds(room).is_some_and(|rect| {
                            area.contains(Vec2::new(rect.x, rect.y))
                                && area.contains(Vec2::new(rect.right(), rect.top()))
                        })
                    })
                    .collect::<Vec<_>>();
                for item in boxed {
                    if !editor.selected.contains(&item) {
                        editor.selected.push(item);
                    }
                }
                editor.field = 0;
            }
            _ => {}
        }
    }

    editor.hovered = if editor.drag.is_none() && !editor.pointer_over_panel {
        data.rooms
            .get(&room_id)
            .and_then(|room| EditorItem::at(room, cursor_h))
    } else {
        None
    };
}

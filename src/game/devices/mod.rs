//! A room's devices at work (`devices:` of a room file; their data and
//! tuning in `src/data/devices.rs`, their art in `devices/devices.yaml`).
//!
//! Each device is one entity standing at the middle of its bounds, carrying
//! a `Device` (its room entry, its clock and the frame it shows) and drawn by
//! child `DevicePiece` sprites that each show their part of that frame. What
//! of it hurts is its `Hurts`, which `combat::hurt_by_devices` applies; what
//! of it is solid is its `PlatformSurface`, which every body collides with.
//! `traps.rs` runs the ones that hurt, `platforms.rs` the ones bodies stand
//! on, and `doors.rs` the buttons, doors and ladders.

use super::*;
use crate::{
    data::{
        DeviceArt, DeviceChoice, DeviceConfig, DeviceKind, DeviceSheet, DeviceTag, Facing,
        FireTrapConfig, HoverSize, LaserConfig, Orientation, PressConfig, RoomConfig,
    },
    game::interact::Interactable,
};
use bevy::math::Rect;

mod doors;
mod platforms;
mod terminals;
mod traps;

pub(super) use doors::{Ladder, press_buttons};
#[cfg(test)]
pub(super) use terminals::BubblePart;
pub(super) use terminals::read_terminals;

/// Devices stand over the platforms and the background props, behind the
/// bodies.
const DEVICE_Z: f32 = 2.0;

pub(super) struct DevicesPlugin;

impl Plugin for DevicesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<terminals::Reading>().add_systems(
            Update,
            (
                // Before the player moves: what carries or holds them up
                // must stand where it is this frame.
                (
                    platforms::move_platforms,
                    platforms::carry_riders,
                    platforms::fold_platforms,
                    doors::open_doors,
                )
                    .chain()
                    .before(player::update_player),
                (
                    traps::run_cycles,
                    traps::run_turrets,
                    platforms::run_conveyors,
                    terminals::type_terminal_text,
                    show_frames,
                )
                    .chain()
                    .after(player::update_player),
            )
                .run_if(in_state(GameScreen::Playing)),
        );
    }
}

/// A device of the room at work.
#[derive(Component)]
pub(crate) struct Device {
    pub(crate) config: DeviceConfig,
    /// Seconds since the room opened, from its `offset_s` for one on a cycle.
    pub(crate) elapsed_s: f32,
    /// The frame of its sheet it shows.
    pub(crate) frame: usize,
    /// How many frames its look's sheet has, and how fast a loop of them plays.
    frames: usize,
    fps: f32,
    /// Its look's frame size, in pixels.
    frame_px: [u32; 2],
}

/// A sprite drawing part of its device's frame: `sub` is that part, in
/// pixels of the frame, and `cell` shifts it along the strip by that many
/// frames (the spikes of a row are each a different cell of the pack's).
#[derive(Component)]
struct DevicePiece {
    sub: Rect,
    cell: usize,
}

/// Where in a cycle of stretches `elapsed_s` falls: the stretch's index and
/// how far into it, in seconds.
fn cycle(elapsed_s: f32, stretches: &[f32]) -> (usize, f32) {
    let total = stretches.iter().sum::<f32>().max(f32::EPSILON);
    let mut into = elapsed_s.rem_euclid(total);
    for (index, stretch) in stretches.iter().enumerate() {
        if into < *stretch {
            return (index, into);
        }
        into -= stretch;
    }
    (stretches.len() - 1, 0.0)
}

/// The frame a laser shows `elapsed_s` into its cycle, and whether it hurts.
fn laser_state(laser: &LaserConfig, elapsed_s: f32, fps: f32) -> (usize, bool) {
    match cycle(elapsed_s, &[laser.off_s, laser.warn_s, laser.on_s]) {
        (0, _) => (0, false),
        (1, _) => (1, false),
        // On, flickering between its two bright frames.
        (_, into) => (2 + (into * fps) as usize % 2, true),
    }
}

/// The frame a fire trap shows `elapsed_s` into its cycle, and whether it
/// burns: lighting up over its warning, flickering while it burns, dying out
/// once it rests and then cold.
fn fire_state(fire: &FireTrapConfig, elapsed_s: f32, fps: f32) -> (usize, bool) {
    match cycle(elapsed_s, &[fire.arm_s, fire.burn_s, fire.rest_s]) {
        (0, into) => (
            (into / fire.arm_s.max(f32::EPSILON) * 3.0) as usize % 3,
            false,
        ),
        (1, into) => (3 + (into * fps) as usize % 5, true),
        (_, into) => {
            let out = (into * fps) as usize;
            (if out < 4 { 8 + out } else { 0 }, false)
        }
    }
}

/// The frame a press shows `elapsed_s` into its cycle, and whether its head
/// is down (and hurts): up, three frames slamming, five held down, six rising.
fn press_state(press: &PressConfig, elapsed_s: f32, fps: f32) -> (usize, bool) {
    let slam_s = 3.0 / fps;
    let rise_s = 6.0 / fps;
    match cycle(elapsed_s, &[press.rest_s, slam_s, press.hold_s, rise_s]) {
        (0, _) => (0, false),
        (1, into) => (1 + (into * fps) as usize % 3, into * fps >= 2.0),
        (2, into) => (
            4 + (into / press.hold_s.max(f32::EPSILON) * 5.0) as usize % 5,
            true,
        ),
        (_, into) => (9 + (into * fps) as usize % 6, false),
    }
}

/// Where a device that travels stands `elapsed_s` in, from its spot: there
/// and back along `travel` at `speed`.
fn travelled(travel: Vec2, speed: f32, elapsed_s: f32) -> Vec2 {
    let length = travel.length();
    if length < f32::EPSILON || speed <= 0.0 {
        return Vec2::ZERO;
    }
    let along = (elapsed_s * speed).rem_euclid(2.0 * length);
    let fraction = if along < length {
        along / length
    } else {
        2.0 - along / length
    };
    travel * fraction
}

impl Device {
    /// The look its room entry names, as `devices.yaml` keys its sheets.
    fn look(config: &DeviceConfig) -> &'static str {
        match &config.kind {
            DeviceKind::Laser(laser) => laser.color.name(),
            DeviceKind::FireTrap(fire) => fire.color.name(),
            DeviceKind::HoverPlatform(platform) => platform.size.name(),
            DeviceKind::FoldingPlatform(platform) => platform.color.name(),
            DeviceKind::Blade(blade) => blade.style.name(),
            _ => "default",
        }
    }

    /// Its frame and whether it hurts, `elapsed_s` into its cycle; the
    /// devices with no cycle show their first frame and hurt always (spikes,
    /// blades) or never.
    fn state(config: &DeviceConfig, art: &DeviceArt, elapsed_s: f32) -> (usize, bool) {
        let fps = art
            .kinds
            .get(&config.kind.tag())
            .map_or(10.0, |kind| kind.fps);
        match &config.kind {
            DeviceKind::Laser(laser) => laser_state(laser, elapsed_s, fps),
            DeviceKind::FireTrap(fire) => fire_state(fire, elapsed_s, fps),
            DeviceKind::Press(press) => press_state(press, elapsed_s, fps),
            DeviceKind::Spikes(_) | DeviceKind::Blade(_) => (0, true),
            _ => (0, false),
        }
    }

    /// Where its clock starts as the room opens.
    fn start_s(config: &DeviceConfig) -> f32 {
        match &config.kind {
            DeviceKind::Laser(laser) => laser.offset_s,
            DeviceKind::FireTrap(fire) => fire.offset_s,
            DeviceKind::Press(press) => press.offset_s,
            DeviceKind::Turret(turret) => turret.offset_s,
            _ => 0.0,
        }
    }
}

/// A box of a device's frame (`devices.yaml`), in world units, for a device
/// whose bounds' middle is `center`. A row of cells takes the box's height
/// over the row's whole width; a turret facing right has it mirrored.
fn device_box(config: &DeviceConfig, art: &DeviceArt, name: &str, center: Vec2) -> Option<Rect> {
    let tag = config.kind.tag();
    let mut area = art.box_h(tag, name)?;
    let size = config.kind.size_h(art);
    let frame = art.frame_h(tag);
    match &config.kind {
        DeviceKind::FireTrap(_) | DeviceKind::Spikes(_) => {
            // The first cell's left edge to the last one's right edge.
            area.min.x += (frame.x - size.x) / 2.0;
            area.max.x += (size.x - frame.x) / 2.0;
        }
        DeviceKind::Conveyor(_) => {
            area.min.x = -size.x / 2.0;
            area.max.x = size.x / 2.0;
        }
        DeviceKind::HoverPlatform(platform) if platform.size == HoverSize::Small => {
            area = art.box_h(tag, "solid_small")?;
            // The small sheet's frame is half the large one's width.
            area.min.x += frame.x / 4.0;
            area.max.x += frame.x / 4.0;
        }
        DeviceKind::Turret(turret) if turret.facing == Facing::Right => {
            (area.min.x, area.max.x) = (-area.max.x, -area.min.x);
        }
        DeviceKind::Laser(laser) => {
            // Between the two emitters, the length of the laser.
            area.max.y = size.y.max(size.x) - (frame.y - area.max.y);
            if laser.orientation == Orientation::Horizontal {
                let along = (area.min.y, area.max.y);
                (area.min.y, area.max.y) = (area.min.x + size.y / 2.0, area.max.x + size.y / 2.0);
                (area.min.x, area.max.x) = (along.0 - size.x / 2.0, along.1 - size.x / 2.0);
            }
        }
        _ => {}
    }
    let bottom_middle = center - Vec2::new(0.0, size.y / 2.0) * H_WORLD_UNITS;
    Some(Rect::from_corners(
        bottom_middle + area.min * H_WORLD_UNITS,
        bottom_middle + area.max * H_WORLD_UNITS,
    ))
}

/// A box of a device's frame (`device_box`) as the device is placed:
/// mirrored and turned about its middle `center`, as its corners in world
/// units, counter-clockwise from its lower-left.
fn placed_box(
    config: &DeviceConfig,
    art: &DeviceArt,
    name: &str,
    center: Vec2,
) -> Option<[Vec2; 4]> {
    device_box(config, art, name, center).map(|rect| config.placement.corners_about(rect, center))
}

/// The collider of a solid box, given as its corners.
fn surface(corners: [Vec2; 4], data: &GameData) -> PlatformSurface {
    PlatformSurface::from_corners(corners, data.surfaces.color_footstep, data)
}

/// Builds every device of `room`, as it stands the moment the room opens.
pub(super) fn spawn_devices(
    commands: &mut Commands,
    room: &RoomConfig,
    data: &GameData,
    asset_server: &AssetServer,
) {
    let art = &data.devices;
    for config in &room.devices {
        let tag = config.kind.tag();
        let Some(kind_art) = art.kinds.get(&tag) else {
            continue;
        };
        let look = Device::look(config);
        let Some(sheet) = kind_art
            .sheets
            .get(look)
            .or_else(|| kind_art.sheets.values().next())
        else {
            continue;
        };
        let bounds = config.bounds_h(art);
        let center = bounds.center() * H_WORLD_UNITS;
        let elapsed_s = Device::start_s(config);
        let (frame, live) = Device::state(config, art, elapsed_s);
        let frame_px = sheet.frame_px.unwrap_or(kind_art.frame_px);
        let turn = match &config.kind {
            DeviceKind::Laser(laser) if laser.orientation == Orientation::Horizontal => {
                -std::f32::consts::FRAC_PI_2
            }
            _ => 0.0,
        };
        let placement = config.placement;
        let mut device = commands.spawn((
            Device {
                config: config.clone(),
                elapsed_s,
                frame,
                frames: sheet.frames,
                fps: kind_art.fps,
                frame_px,
            },
            // Mirrored in its own frame, then turned: as its boxes are.
            Transform::from_translation(center.extend(DEVICE_Z))
                .with_rotation(Quat::from_rotation_z(
                    turn + placement.rotation_deg.to_radians(),
                ))
                .with_scale(Vec3::new(
                    if placement.mirrored { -1.0 } else { 1.0 },
                    1.0,
                    1.0,
                )),
            Visibility::default(),
            RoomEntity,
        ));
        let hurt_box = match tag {
            DeviceTag::Laser => Some("beam"),
            DeviceTag::FireTrap => Some("flame"),
            DeviceTag::Spikes | DeviceTag::Blade => Some("hurt"),
            DeviceTag::Press => Some("head"),
            _ => None,
        };
        if let Some(corners) = hurt_box.and_then(|name| placed_box(config, art, name, center)) {
            device.insert(combat::Hurts {
                corners,
                damage: match &config.kind {
                    DeviceKind::Laser(laser) => laser.damage,
                    DeviceKind::FireTrap(fire) => fire.damage,
                    DeviceKind::Spikes(spikes) => spikes.damage,
                    DeviceKind::Blade(blade) => blade.damage,
                    DeviceKind::Press(press) => press.damage,
                    _ => 0.0,
                },
                live,
            });
        }
        let solid_box = match tag {
            DeviceTag::FireTrap => Some("base"),
            DeviceTag::Conveyor | DeviceTag::HoverPlatform | DeviceTag::FoldingPlatform => {
                Some("solid")
            }
            DeviceTag::Door | DeviceTag::Terminal => Some("solid"),
            DeviceTag::Press | DeviceTag::Turret => Some("body"),
            _ => None,
        };
        if let Some(corners) = solid_box.and_then(|name| placed_box(config, art, name, center)) {
            device.insert(surface(corners, data));
        }
        match &config.kind {
            DeviceKind::HoverPlatform(_) | DeviceKind::Blade(_) => {
                device.insert(platforms::Mover {
                    spot: center,
                    previous: center,
                });
            }
            DeviceKind::FoldingPlatform(_) => {
                device.insert(platforms::Folding::default());
            }
            DeviceKind::Door(_) => {
                device.insert(doors::DoorState::default());
            }
            DeviceKind::Turret(_) => {
                device.insert(traps::TurretState::default());
            }
            DeviceKind::Ladder(_) => {
                // Its column, from its foot to its top, as placed: one turned
                // is climbed straight up the room's height it spans.
                let width = device_box(config, art, "climb", center)
                    .map_or(H_WORLD_UNITS / 2.0, |climb| climb.width());
                let column = Rect::new(
                    center.x - width / 2.0,
                    bounds.min.y * H_WORLD_UNITS,
                    center.x + width / 2.0,
                    bounds.max.y * H_WORLD_UNITS,
                );
                let corners = placement.corners_about(column, center);
                device.insert(Ladder {
                    area: Rect::from_corners(
                        corners.into_iter().reduce(Vec2::min).unwrap_or_default(),
                        corners.into_iter().reduce(Vec2::max).unwrap_or_default(),
                    ),
                });
            }
            DeviceKind::Terminal(_) => {
                device.insert(Interactable {
                    reach_h: Vec2::new(0.9, 1.0),
                    verb: "read",
                });
            }
            DeviceKind::Button(_) => {
                device.insert((
                    doors::ButtonState::default(),
                    Interactable {
                        reach_h: Vec2::new(0.8, 1.0),
                        verb: "press",
                    },
                ));
            }
            _ => {}
        }
        let image = room_art::load_texture(asset_server, &sheet.sheet);
        device.with_children(|parent| {
            for (piece, local, size) in pieces(config, art, sheet, frame_px) {
                let [frame_width, _] = frame_px;
                let x = ((frame + piece.cell) as u32 * frame_width) as f32;
                parent.spawn((
                    Sprite {
                        image: image.clone(),
                        rect: Some(Rect::new(
                            x + piece.sub.min.x,
                            piece.sub.min.y,
                            x + piece.sub.max.x,
                            piece.sub.max.y,
                        )),
                        custom_size: Some(size * H_WORLD_UNITS),
                        color: Color::WHITE.with_alpha(placement.opacity),
                        flip_x: matches!(&config.kind, DeviceKind::Turret(turret)
                            if turret.facing == Facing::Right),
                        ..default()
                    },
                    Transform::from_translation((local * H_WORLD_UNITS).extend(0.0)),
                    piece,
                ));
            }
        });
    }
}

/// The sprites a device is drawn with: the part of the frame each shows,
/// where it stands from the device's middle and its size, in H.
fn pieces(
    config: &DeviceConfig,
    art: &DeviceArt,
    sheet: &DeviceSheet,
    frame_px: [u32; 2],
) -> Vec<(DevicePiece, Vec2, Vec2)> {
    let tag = config.kind.tag();
    let px = art.px_per_h(tag);
    let [width, height] = frame_px.map(|side| side as f32);
    let whole = Rect::new(0.0, 0.0, width, height);
    let size = config.kind.size_h(art);
    let frame_h = Vec2::new(width, height) / px;
    let piece = |sub: Rect, cell: usize| DevicePiece { sub, cell };
    // A row of `count` cells of `cell_px` wide, laid from left to right.
    let row = |count: u32, cell_px: f32, sub_of: &dyn Fn(u32) -> (Rect, usize)| {
        (0..count)
            .map(|index| {
                let (sub, cell) = sub_of(index);
                let x = -size.x / 2.0 + (index as f32 + 0.5) * cell_px / px;
                (
                    piece(sub, cell),
                    Vec2::new(x, 0.0),
                    Vec2::new(cell_px, sub.height()) / px,
                )
            })
            .collect::<Vec<_>>()
    };
    match &config.kind {
        DeviceKind::Laser(laser) => {
            // Drawn running up, and turned for one running sideways.
            let cap = art
                .kinds
                .get(&tag)
                .and_then(|kind| kind.boxes.get("top_cap"))
                .map_or(5.0, |cap| cap[3] as f32);
            let length = laser.length_h;
            let cap_h = cap / px;
            vec![
                (
                    piece(Rect::new(0.0, 0.0, width, cap), 0),
                    Vec2::new(0.0, length / 2.0 - cap_h / 2.0),
                    Vec2::new(frame_h.x, cap_h),
                ),
                (
                    piece(Rect::new(0.0, cap, width, height - cap), 0),
                    Vec2::ZERO,
                    Vec2::new(frame_h.x, length - 2.0 * cap_h),
                ),
                (
                    piece(Rect::new(0.0, height - cap, width, height), 0),
                    Vec2::new(0.0, cap_h / 2.0 - length / 2.0),
                    Vec2::new(frame_h.x, cap_h),
                ),
            ]
        }
        DeviceKind::FireTrap(fire) => row(fire.cells, width, &|_| (whole, 0)),
        DeviceKind::Spikes(spikes) => row(spikes.cells, width, &|index| {
            (whole, index as usize % sheet.frames.max(1))
        }),
        DeviceKind::Conveyor(belt) => {
            // Its two ends and its middle, each a third of the frame.
            let third = width / 3.0;
            let last = belt.cells - 1;
            row(belt.cells, third, &|index| {
                let part = if index == 0 {
                    0.0
                } else if index == last {
                    2.0
                } else {
                    1.0
                };
                (
                    Rect::new(part * third, 0.0, (part + 1.0) * third, height),
                    0,
                )
            })
        }
        DeviceKind::Ladder(ladder) => {
            // Its top, then a rung for every cell down to its foot.
            let cells = (ladder.height_h * px / height).ceil().max(1.0) as u32;
            (0..cells)
                .map(|index| {
                    let y = ladder.height_h / 2.0 - (index as f32 + 0.5) * frame_h.y;
                    (
                        piece(whole, usize::from(index > 0)),
                        Vec2::new(0.0, y),
                        frame_h,
                    )
                })
                .collect()
        }
        _ => vec![(piece(whole, 0), Vec2::ZERO, size.max(frame_h).min(frame_h))],
    }
}

/// Shows each device's frame on its pieces.
fn show_frames(
    devices: Query<(&Device, &Children)>,
    mut pieces: Query<(&DevicePiece, &mut Sprite)>,
) {
    for (device, children) in &devices {
        let [frame_width, _] = device.frame_px;
        for child in children {
            let Ok((piece, mut sprite)) = pieces.get_mut(*child) else {
                continue;
            };
            let x = ((device.frame + piece.cell) as u32 * frame_width) as f32;
            let rect = Rect::new(
                x + piece.sub.min.x,
                piece.sub.min.y,
                x + piece.sub.max.x,
                piece.sub.max.y,
            );
            if sprite.rect != Some(rect) {
                sprite.rect = Some(rect);
            }
        }
    }
}

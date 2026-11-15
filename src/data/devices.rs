//! Devices: the working pieces of a room (`devices:` of a room file) — traps
//! that hurt on a cycle or on touch, platforms that move or fold, a press, a
//! turret, buttons, doors, ladders, terminals and light strips. A device is placed like a prop, by the
//! middle of its bottom edge, and carries the values that tune it; what it
//! looks like and which of its pixels hurt or hold a body up is the same for
//! every device of a kind, in `devices/devices.yaml`.
//!
//! Each kind lists its values once, in `DeviceKind::params`: the room file's
//! keys, the editor's inspector rows and how Left/Right step them all come
//! from that one list.

use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
};

use bevy::math::{Rect, Vec2};
use serde::Deserialize;

use super::{Facing, Placement, SpawnPosition};

/// A device placed in a room.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DeviceConfig {
    #[serde(default)]
    pub label: Option<String>,
    /// The middle of the device's bottom edge.
    pub position_h: SpawnPosition,
    #[serde(flatten)]
    pub kind: DeviceKind,
    /// Mirrored, turned about its middle, and faded: what of it hurts and
    /// holds a body up turns and mirrors with it.
    #[serde(flatten)]
    pub placement: Placement,
}

/// What a device is, and the values that tune it.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DeviceKind {
    Laser(LaserConfig),
    FireTrap(FireTrapConfig),
    Spikes(SpikesConfig),
    Conveyor(ConveyorConfig),
    HoverPlatform(HoverPlatformConfig),
    FoldingPlatform(FoldingPlatformConfig),
    Press(PressConfig),
    Blade(BladeConfig),
    Turret(TurretConfig),
    Button(ButtonConfig),
    Door(DoorConfig),
    Ladder(LadderConfig),
    Terminal(TerminalConfig),
    LightStrip(LightStripConfig),
}

/// A device's kind alone: what the editor adds, and what decides its rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceTag {
    Laser,
    FireTrap,
    Spikes,
    Conveyor,
    HoverPlatform,
    FoldingPlatform,
    Press,
    Blade,
    Turret,
    Button,
    Door,
    Ladder,
    Terminal,
    LightStrip,
}

impl DeviceTag {
    pub(crate) const ALL: [Self; 14] = [
        Self::Laser,
        Self::FireTrap,
        Self::Spikes,
        Self::Conveyor,
        Self::HoverPlatform,
        Self::FoldingPlatform,
        Self::Press,
        Self::Blade,
        Self::Turret,
        Self::Button,
        Self::Door,
        Self::Ladder,
        Self::Terminal,
        Self::LightStrip,
    ];
}

/// The name a room file gives the kind.
impl Display for DeviceTag {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Laser => "laser",
            Self::FireTrap => "fire_trap",
            Self::Spikes => "spikes",
            Self::Conveyor => "conveyor",
            Self::HoverPlatform => "hover_platform",
            Self::FoldingPlatform => "folding_platform",
            Self::Press => "press",
            Self::Blade => "blade",
            Self::Turret => "turret",
            Self::Button => "button",
            Self::Door => "door",
            Self::Ladder => "ladder",
            Self::Terminal => "terminal",
            Self::LightStrip => "light_strip",
        })
    }
}

/// A new device of a kind: the values the editor gives one it adds.
impl From<DeviceTag> for DeviceKind {
    fn from(tag: DeviceTag) -> Self {
        match tag {
            DeviceTag::Laser => Self::Laser(LaserConfig::default()),
            DeviceTag::FireTrap => Self::FireTrap(FireTrapConfig::default()),
            DeviceTag::Spikes => Self::Spikes(SpikesConfig::default()),
            DeviceTag::Conveyor => Self::Conveyor(ConveyorConfig::default()),
            DeviceTag::HoverPlatform => Self::HoverPlatform(HoverPlatformConfig::default()),
            DeviceTag::FoldingPlatform => Self::FoldingPlatform(FoldingPlatformConfig::default()),
            DeviceTag::Press => Self::Press(PressConfig::default()),
            DeviceTag::Blade => Self::Blade(BladeConfig::default()),
            DeviceTag::Turret => Self::Turret(TurretConfig::default()),
            DeviceTag::Button => Self::Button(ButtonConfig::default()),
            DeviceTag::Door => Self::Door(DoorConfig::default()),
            DeviceTag::Ladder => Self::Ladder(LadderConfig::default()),
            DeviceTag::Terminal => Self::Terminal(TerminalConfig::default()),
            DeviceTag::LightStrip => Self::LightStrip(LightStripConfig::default()),
        }
    }
}

/// One tunable value of a device: its key in the room file, its row in the
/// editor's inspector, and the value itself.
pub(crate) struct Param<'a> {
    pub(crate) key: &'static str,
    pub(crate) name: &'static str,
    pub(crate) value: ParamValue<'a>,
}

pub(crate) enum ParamValue<'a> {
    /// A number stepped by `step`, never below `min`.
    Number {
        value: &'a mut f32,
        step: f32,
        min: f32,
    },
    /// A whole number within `min..=max`.
    Whole {
        value: &'a mut u32,
        min: u32,
        max: u32,
    },
    /// One of a few named options, walked through in turn.
    Choice(&'a mut dyn DeviceChoice),
    /// Words, typed into the inspector rather than stepped.
    Text(&'a mut String),
}

impl ParamValue<'_> {
    /// Steps the value `steps` times (negative to go back).
    pub(crate) fn step(&mut self, steps: f32) {
        match self {
            Self::Number { value, step, min } => {
                // Rounded to the step, so repeated presses don't drift.
                let stepped = ((**value + steps * *step) / *step).round() * *step;
                **value = stepped.max(*min);
            }
            Self::Whole { value, min, max } => {
                let stepped = i64::from(**value) + steps.signum() as i64;
                **value = stepped.clamp(i64::from(*min), i64::from(*max)) as u32;
            }
            Self::Choice(choice) => choice.step(steps >= 0.0),
            Self::Text(_) => {}
        }
    }
}

/// The value as the room file and the inspector write it.
impl Display for ParamValue<'_> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number { value, .. } => write!(formatter, "{value}"),
            Self::Whole { value, .. } => write!(formatter, "{value}"),
            Self::Choice(choice) => formatter.write_str(choice.name()),
            Self::Text(text) => formatter.write_str(text),
        }
    }
}

/// A device value that is one of a few named options.
pub(crate) trait DeviceChoice {
    /// The option after this one (or before it), going round.
    fn step(&mut self, forward: bool);
    /// Its name in the room file.
    fn name(&self) -> &'static str;
}

/// Declares a two-option choice of a device: its options, and the first one
/// as the default.
macro_rules! two_way_choice {
    ($(#[$doc:meta])* $name:ident { $first:ident = $first_key:literal, $second:ident = $second_key:literal }) => {
        $(#[$doc])*
        #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum $name {
            #[default]
            $first,
            $second,
        }

        impl DeviceChoice for $name {
            fn step(&mut self, _forward: bool) {
                *self = match self {
                    Self::$first => Self::$second,
                    Self::$second => Self::$first,
                };
            }

            fn name(&self) -> &'static str {
                match self {
                    Self::$first => $first_key,
                    Self::$second => $second_key,
                }
            }
        }
    };
}

two_way_choice!(
    /// A laser's colour: only a look.
    LaserColor { Red = "red", Blue = "blue" }
);
two_way_choice!(
    /// Which way a laser's beam, or a light strip, runs from its emitter at the spot.
    Orientation { Vertical = "vertical", Horizontal = "horizontal" }
);
two_way_choice!(
    /// A fire trap's colour: only a look.
    FireColor { Orange = "orange", Blue = "blue" }
);
two_way_choice!(
    /// A hovering platform's size.
    HoverSize { Large = "large", Small = "small" }
);
two_way_choice!(
    /// A folding platform's colour: only a look.
    FoldingColor { Green = "green", Steel = "steel" }
);

impl DeviceChoice for Facing {
    fn step(&mut self, _forward: bool) {
        *self = match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        };
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
        }
    }
}

/// A beam between two emitters that is off for `off_s`, warns for `warn_s`
/// (a thin beam that does not hurt), and is on for `on_s` (hurting whoever
/// touches it), `offset_s` into that cycle as the room opens.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default)]
pub struct LaserConfig {
    pub color: LaserColor,
    pub orientation: Orientation,
    /// From emitter to emitter, in H.
    pub length_h: f32,
    pub off_s: f32,
    pub warn_s: f32,
    pub on_s: f32,
    pub offset_s: f32,
    pub damage: f32,
}

impl Default for LaserConfig {
    fn default() -> Self {
        Self {
            color: LaserColor::Red,
            orientation: Orientation::Vertical,
            length_h: 3.0,
            off_s: 1.2,
            warn_s: 0.5,
            on_s: 1.0,
            offset_s: 0.0,
            damage: 15.0,
        }
    }
}

/// A row of grates that warn for `arm_s`, burn for `burn_s` and rest for
/// `rest_s`. Their bases are solid.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default)]
pub struct FireTrapConfig {
    pub color: FireColor,
    /// How many grates stand side by side.
    pub cells: u32,
    pub arm_s: f32,
    pub burn_s: f32,
    pub rest_s: f32,
    pub offset_s: f32,
    pub damage: f32,
}

impl Default for FireTrapConfig {
    fn default() -> Self {
        Self {
            color: FireColor::Orange,
            cells: 2,
            arm_s: 0.6,
            burn_s: 1.2,
            rest_s: 1.4,
            offset_s: 0.0,
            damage: 12.0,
        }
    }
}

/// A row of spikes that hurt whoever touches them.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default)]
pub struct SpikesConfig {
    pub cells: u32,
    pub damage: f32,
}

impl Default for SpikesConfig {
    fn default() -> Self {
        Self {
            cells: 4,
            damage: 10.0,
        }
    }
}

/// A solid belt that carries whoever stands on it, rightward at a positive
/// speed and leftward at a negative one.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default)]
pub struct ConveyorConfig {
    pub cells: u32,
    pub speed_h_s: f32,
}

impl Default for ConveyorConfig {
    fn default() -> Self {
        Self {
            cells: 6,
            speed_h_s: 2.0,
        }
    }
}

/// A solid platform that flies back and forth between its spot and the spot
/// `travel_x_h`, `travel_y_h` away, carrying whoever stands on it.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default)]
pub struct HoverPlatformConfig {
    pub size: HoverSize,
    pub travel_x_h: f32,
    pub travel_y_h: f32,
    pub speed_h_s: f32,
}

impl Default for HoverPlatformConfig {
    fn default() -> Self {
        Self {
            size: HoverSize::Large,
            travel_x_h: 4.0,
            travel_y_h: 0.0,
            speed_h_s: 1.5,
        }
    }
}

/// A platform that gives way once someone has stood on it for `stand_s`,
/// and comes back `return_s` later.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default)]
pub struct FoldingPlatformConfig {
    pub color: FoldingColor,
    pub stand_s: f32,
    pub return_s: f32,
}

impl Default for FoldingPlatformConfig {
    fn default() -> Self {
        Self {
            color: FoldingColor::Green,
            stand_s: 0.6,
            return_s: 2.5,
        }
    }
}

/// A press hung from a ceiling that rests up for `rest_s`, slams down,
/// holds there for `hold_s` and rises again, `offset_s` into that cycle as
/// the room opens. Its head hurts whoever is under it while it is down.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default)]
pub struct PressConfig {
    pub rest_s: f32,
    pub hold_s: f32,
    pub offset_s: f32,
    pub damage: f32,
}

impl Default for PressConfig {
    fn default() -> Self {
        Self {
            rest_s: 1.5,
            hold_s: 0.6,
            offset_s: 0.0,
            damage: 25.0,
        }
    }
}

/// A spinning blade that hurts on touch, travelling back and forth between
/// its spot and the spot `travel_x_h`, `travel_y_h` away (or standing still
/// when both are 0): slowing into each end and fastest halfway, at
/// `speed_h_s` on average.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default)]
pub struct BladeConfig {
    pub travel_x_h: f32,
    pub travel_y_h: f32,
    pub speed_h_s: f32,
    pub damage: f32,
}

impl Default for BladeConfig {
    fn default() -> Self {
        Self {
            travel_x_h: 3.0,
            travel_y_h: 0.0,
            speed_h_s: 2.0,
            damage: 15.0,
        }
    }
}

/// A turret that fires a shot the way it faces every `interval_s`, the
/// first `offset_s` in. Its shots stop at platforms.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default)]
pub struct TurretConfig {
    pub facing: Facing,
    pub interval_s: f32,
    pub offset_s: f32,
    pub shot_speed_h_s: f32,
    pub damage: f32,
}

impl Default for TurretConfig {
    fn default() -> Self {
        Self {
            facing: Facing::Left,
            interval_s: 1.8,
            offset_s: 0.0,
            shot_speed_h_s: 9.0,
            damage: 10.0,
        }
    }
}

/// A button, pressed with Interact, that opens every door sharing its `link`,
/// and closes them again when pressed a second time.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default)]
pub struct ButtonConfig {
    pub link: u32,
}

impl Default for ButtonConfig {
    fn default() -> Self {
        Self { link: 1 }
    }
}

/// A sliding door, solid until a button sharing its `link` is pressed, and
/// again once it is pressed a second time; open, the stubs left at its top
/// and bottom are still solid.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default)]
pub struct DoorConfig {
    pub link: u32,
}

impl Default for DoorConfig {
    fn default() -> Self {
        Self { link: 1 }
    }
}

/// A terminal that, used with Interact, types `text` out in a bubble over it.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default)]
pub struct TerminalConfig {
    pub text: String,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            text: String::from("..."),
        }
    }
}

/// A ladder climbed from its foot (the spot) up to `height_h`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default)]
pub struct LadderConfig {
    pub height_h: f32,
}

impl Default for LadderConfig {
    fn default() -> Self {
        Self { height_h: 3.0 }
    }
}

/// A strip of lights, pure scenery, `length_h` long from end to end: its two
/// ends with its middle repeated between them.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default)]
pub struct LightStripConfig {
    pub orientation: Orientation,
    pub length_h: f32,
}

impl Default for LightStripConfig {
    fn default() -> Self {
        Self {
            orientation: Orientation::Vertical,
            length_h: 1.5,
        }
    }
}

/// Builds a `Param`.
fn param<'a>(key: &'static str, name: &'static str, value: ParamValue<'a>) -> Param<'a> {
    Param { key, name, value }
}

/// A number stepped by `step`, never below `min`.
fn number(value: &mut f32, step: f32, min: f32) -> ParamValue<'_> {
    ParamValue::Number { value, step, min }
}

/// A duration in seconds, stepped by a tenth.
fn seconds(value: &mut f32) -> ParamValue<'_> {
    number(value, 0.1, 0.0)
}

impl DeviceKind {
    pub(crate) fn tag(&self) -> DeviceTag {
        match self {
            Self::Laser(_) => DeviceTag::Laser,
            Self::FireTrap(_) => DeviceTag::FireTrap,
            Self::Spikes(_) => DeviceTag::Spikes,
            Self::Conveyor(_) => DeviceTag::Conveyor,
            Self::HoverPlatform(_) => DeviceTag::HoverPlatform,
            Self::FoldingPlatform(_) => DeviceTag::FoldingPlatform,
            Self::Press(_) => DeviceTag::Press,
            Self::Blade(_) => DeviceTag::Blade,
            Self::Turret(_) => DeviceTag::Turret,
            Self::Button(_) => DeviceTag::Button,
            Self::Door(_) => DeviceTag::Door,
            Self::Ladder(_) => DeviceTag::Ladder,
            Self::Terminal(_) => DeviceTag::Terminal,
            Self::LightStrip(_) => DeviceTag::LightStrip,
        }
    }

    /// The device's tunable values, in the order the room file and the
    /// inspector list them.
    pub(crate) fn params(&mut self) -> Vec<Param<'_>> {
        match self {
            Self::Laser(laser) => vec![
                param("color", "Colour", ParamValue::Choice(&mut laser.color)),
                param(
                    "orientation",
                    "Runs",
                    ParamValue::Choice(&mut laser.orientation),
                ),
                param(
                    "length_h",
                    "Length (H)",
                    number(&mut laser.length_h, 0.5, 1.5),
                ),
                param("off_s", "Off for (s)", seconds(&mut laser.off_s)),
                param("warn_s", "Warns for (s)", seconds(&mut laser.warn_s)),
                param("on_s", "On for (s)", seconds(&mut laser.on_s)),
                param("offset_s", "Offset (s)", seconds(&mut laser.offset_s)),
                param("damage", "Damage", number(&mut laser.damage, 1.0, 0.0)),
            ],
            Self::FireTrap(fire) => vec![
                param("color", "Colour", ParamValue::Choice(&mut fire.color)),
                param(
                    "cells",
                    "Grates",
                    ParamValue::Whole {
                        value: &mut fire.cells,
                        min: 1,
                        max: 32,
                    },
                ),
                param("arm_s", "Warns for (s)", seconds(&mut fire.arm_s)),
                param("burn_s", "Burns for (s)", seconds(&mut fire.burn_s)),
                param("rest_s", "Rests for (s)", seconds(&mut fire.rest_s)),
                param("offset_s", "Offset (s)", seconds(&mut fire.offset_s)),
                param("damage", "Damage", number(&mut fire.damage, 1.0, 0.0)),
            ],
            Self::Spikes(spikes) => vec![
                param(
                    "cells",
                    "Cells",
                    ParamValue::Whole {
                        value: &mut spikes.cells,
                        min: 1,
                        max: 64,
                    },
                ),
                param("damage", "Damage", number(&mut spikes.damage, 1.0, 0.0)),
            ],
            Self::Conveyor(belt) => vec![
                param(
                    "cells",
                    "Cells",
                    ParamValue::Whole {
                        value: &mut belt.cells,
                        min: 2,
                        max: 64,
                    },
                ),
                param(
                    "speed_h_s",
                    "Speed (H/s, - left)",
                    number(&mut belt.speed_h_s, 0.25, f32::MIN),
                ),
            ],
            Self::HoverPlatform(platform) => vec![
                param("size", "Size", ParamValue::Choice(&mut platform.size)),
                param(
                    "travel_x_h",
                    "Travel x (H)",
                    number(&mut platform.travel_x_h, 0.5, f32::MIN),
                ),
                param(
                    "travel_y_h",
                    "Travel y (H)",
                    number(&mut platform.travel_y_h, 0.5, f32::MIN),
                ),
                param(
                    "speed_h_s",
                    "Speed (H/s)",
                    number(&mut platform.speed_h_s, 0.25, 0.25),
                ),
            ],
            Self::FoldingPlatform(platform) => vec![
                param("color", "Colour", ParamValue::Choice(&mut platform.color)),
                param("stand_s", "Holds for (s)", seconds(&mut platform.stand_s)),
                param(
                    "return_s",
                    "Back after (s)",
                    seconds(&mut platform.return_s),
                ),
            ],
            Self::Press(press) => vec![
                param("rest_s", "Rests up (s)", seconds(&mut press.rest_s)),
                param("hold_s", "Holds down (s)", seconds(&mut press.hold_s)),
                param("offset_s", "Offset (s)", seconds(&mut press.offset_s)),
                param("damage", "Damage", number(&mut press.damage, 1.0, 0.0)),
            ],
            Self::Blade(blade) => vec![
                param(
                    "travel_x_h",
                    "Travel x (H)",
                    number(&mut blade.travel_x_h, 0.5, f32::MIN),
                ),
                param(
                    "travel_y_h",
                    "Travel y (H)",
                    number(&mut blade.travel_y_h, 0.5, f32::MIN),
                ),
                param(
                    "speed_h_s",
                    "Speed (H/s)",
                    number(&mut blade.speed_h_s, 0.25, 0.25),
                ),
                param("damage", "Damage", number(&mut blade.damage, 1.0, 0.0)),
            ],
            Self::Turret(turret) => vec![
                param("facing", "Faces", ParamValue::Choice(&mut turret.facing)),
                param(
                    "interval_s",
                    "Fires every (s)",
                    number(&mut turret.interval_s, 0.1, 0.2),
                ),
                param("offset_s", "Offset (s)", seconds(&mut turret.offset_s)),
                param(
                    "shot_speed_h_s",
                    "Shot speed (H/s)",
                    number(&mut turret.shot_speed_h_s, 0.5, 0.5),
                ),
                param("damage", "Damage", number(&mut turret.damage, 1.0, 0.0)),
            ],
            Self::Button(button) => vec![param(
                "link",
                "Opens door #",
                ParamValue::Whole {
                    value: &mut button.link,
                    min: 1,
                    max: 99,
                },
            )],
            Self::Door(door) => vec![param(
                "link",
                "Door #",
                ParamValue::Whole {
                    value: &mut door.link,
                    min: 1,
                    max: 99,
                },
            )],
            Self::Ladder(ladder) => vec![param(
                "height_h",
                "Height (H)",
                number(&mut ladder.height_h, 0.5, 1.0),
            )],
            Self::Terminal(terminal) => {
                vec![param("text", "Text", ParamValue::Text(&mut terminal.text))]
            }
            Self::LightStrip(strip) => vec![
                param(
                    "orientation",
                    "Runs",
                    ParamValue::Choice(&mut strip.orientation),
                ),
                param(
                    "length_h",
                    "Length (H)",
                    number(&mut strip.length_h, 0.5, 1.0),
                ),
            ],
        }
    }

    /// The width and height the device takes up, in H, standing on its spot.
    pub(crate) fn size_h(&self, art: &DeviceArt) -> Vec2 {
        let frame = |tag: DeviceTag| art.frame_h(tag);
        match self {
            Self::Laser(laser) => match laser.orientation {
                Orientation::Vertical => Vec2::new(frame(DeviceTag::Laser).x, laser.length_h),
                Orientation::Horizontal => Vec2::new(laser.length_h, frame(DeviceTag::Laser).x),
            },
            Self::FireTrap(fire) => frame(DeviceTag::FireTrap) * Vec2::new(fire.cells as f32, 1.0),
            Self::Spikes(spikes) => frame(DeviceTag::Spikes) * Vec2::new(spikes.cells as f32, 1.0),
            Self::Conveyor(belt) => Vec2::new(
                belt.cells as f32 * frame(DeviceTag::Conveyor).x / 3.0,
                frame(DeviceTag::Conveyor).y,
            ),
            Self::HoverPlatform(platform) => match platform.size {
                HoverSize::Large => frame(DeviceTag::HoverPlatform),
                HoverSize::Small => frame(DeviceTag::HoverPlatform) * Vec2::new(0.5, 1.0),
            },
            Self::Ladder(ladder) => Vec2::new(frame(DeviceTag::Ladder).x, ladder.height_h),
            Self::LightStrip(strip) => {
                let across = frame(DeviceTag::LightStrip).x;
                match strip.orientation {
                    Orientation::Vertical => Vec2::new(across, strip.length_h),
                    Orientation::Horizontal => Vec2::new(strip.length_h, across),
                }
            }
            Self::FoldingPlatform(_)
            | Self::Press(_)
            | Self::Blade(_)
            | Self::Turret(_)
            | Self::Button(_)
            | Self::Door(_)
            | Self::Terminal(_) => frame(self.tag()),
        }
    }
}

impl DeviceConfig {
    /// The rectangle the device takes up in the room, in H.
    pub(crate) fn bounds_h(&self, art: &DeviceArt) -> Rect {
        let size = self.kind.size_h(art);
        let tag = self.kind.tag();
        let sink = art.kinds.get(&tag).map_or(0.0, |kind| kind.sink_px as f32) / art.px_per_h(tag);
        let feet = Vec2::new(self.position_h.x_h, self.position_h.y_h - sink);
        Rect::from_corners(
            feet - Vec2::new(size.x / 2.0, 0.0),
            feet + Vec2::new(size.x / 2.0, size.y),
        )
    }
}

/// How every device of a kind looks: `devices/devices.yaml`.
#[derive(Debug, Deserialize)]
pub struct DeviceArt {
    /// Pixels per H of every device's frames, unless its kind sets its own.
    pub px_per_h: f32,
    pub kinds: BTreeMap<DeviceTag, DeviceKindArt>,
}

/// A kind's sheets and the boxes of its frame that matter.
#[derive(Debug, Deserialize)]
pub struct DeviceKindArt {
    /// The sheet of each look the kind comes in (its colour, style or size,
    /// as its room entry names it), or of `default` for a kind with one look.
    pub sheets: BTreeMap<String, DeviceSheet>,
    pub frame_px: [u32; 2],
    pub fps: f32,
    /// Its own pixels per H, drawn bigger or smaller than the others.
    #[serde(default)]
    pub px_per_h: Option<f32>,
    /// How many of its frame's bottom rows stand below its spot.
    #[serde(default)]
    pub sink_px: u32,
    /// Named boxes of the frame, `[left, top, width, height]` in pixels from
    /// its top-left corner: which part hurts, which part is solid.
    #[serde(default)]
    pub boxes: BTreeMap<String, [u32; 4]>,
}

/// One look of a kind: a strip of `frames` frames laid left to right.
#[derive(Debug, Deserialize)]
pub struct DeviceSheet {
    pub sheet: String,
    pub frames: usize,
    /// Its own frame size, when it is not the kind's.
    #[serde(default)]
    pub frame_px: Option<[u32; 2]>,
}

impl DeviceArt {
    /// Pixels per H of `tag`'s frames.
    pub(crate) fn px_per_h(&self, tag: DeviceTag) -> f32 {
        self.kinds
            .get(&tag)
            .and_then(|kind| kind.px_per_h)
            .unwrap_or(self.px_per_h)
    }

    /// The size of one of `tag`'s frames, in H.
    pub(crate) fn frame_h(&self, tag: DeviceTag) -> Vec2 {
        self.kinds.get(&tag).map_or(Vec2::ONE, |kind| {
            Vec2::new(kind.frame_px[0] as f32, kind.frame_px[1] as f32) / self.px_per_h(tag)
        })
    }

    /// The box `name` of `tag`'s frame, in H, relative to the middle of the
    /// frame's bottom edge (y up).
    pub(crate) fn box_h(&self, tag: DeviceTag, name: &str) -> Option<Rect> {
        let kind = self.kinds.get(&tag)?;
        let [x, y, width, height] = *kind.boxes.get(name)?;
        let px = self.px_per_h(tag);
        let [frame_width, frame_height] = kind.frame_px;
        let left = (x as f32 - frame_width as f32 / 2.0) / px;
        let bottom = (frame_height as f32 - (y + height) as f32) / px;
        Some(Rect::new(
            left,
            bottom,
            left + width as f32 / px,
            bottom + height as f32 / px,
        ))
    }
}

//! Writes a room back to YAML in the layout of `rooms/room_NNN.yaml`, so the
//! debug editor's saves stay readable and diff cleanly against hand edits.

use std::fmt::{self, Display, Formatter};

use super::{
    Backdrop, DecorationConfig, DecorationLayer, DeviceConfig, EnemySpawnConfig, Facing, Fill,
    ParamValue, Placement, PlatformConfig, PropConfig, RectH, RoomConfig, SizeH,
};

const HEADER: &str = "\
# Room coordinates use H from the lower-left. Platform and decoration `rect_h`
# give each block's lower-left corner and size; player, enemy, and exit y
# values mark the feet baseline on a platform top.
";

impl Display for RoomConfig {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(HEADER)?;
        writeln!(formatter, "room_id: {}", self.room_id)?;
        writeln!(formatter, "chapter_id: {}", self.chapter_id)?;
        writeln!(formatter, "size:")?;
        writeln!(formatter, "  width_h: {}", self.size.width_h)?;
        writeln!(formatter, "  height_h: {}", self.size.height_h)?;
        // A color is quoted: unquoted, YAML reads `#` as a comment.
        let backdrops = self
            .background_profiles
            .iter()
            .map(|backdrop| match backdrop {
                Backdrop::Profile(profile) => profile.clone(),
                Backdrop::Color(color) => format!("\"{color}\""),
            })
            .collect::<Vec<_>>();
        writeln!(formatter, "background_profiles: [{}]", backdrops.join(", "))?;
        if !self.hidden_background_layers.is_empty() {
            let hidden = self
                .hidden_background_layers
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>();
            writeln!(
                formatter,
                "hidden_background_layers: [{}]",
                hidden.join(", ")
            )?;
        }
        if !self.layer_opacity.is_empty() {
            let faded = self
                .layer_opacity
                .iter()
                .map(|(layer, opacity)| format!("{layer}: {opacity}"))
                .collect::<Vec<_>>();
            writeln!(formatter, "layer_opacity: {{ {} }}", faded.join(", "))?;
        }
        writeln!(formatter, "player_spawn:")?;
        writeln!(formatter, "  x_h: {}", self.player_spawn.x_h)?;
        writeln!(formatter, "  y_h: {}", self.player_spawn.y_h)?;
        writeln!(
            formatter,
            "# Solid platforms: land on their top, stop against their sides, bump\n\
             # your head on their underside. They stop shots too."
        )?;
        write!(formatter, "{}", YamlList("platforms", &self.platforms))?;
        writeln!(
            formatter,
            "# Decorations: non-solid panels behind the platforms or in front of the bodies."
        )?;
        write!(formatter, "{}", YamlList("decorations", &self.decorations))?;
        if !self.props.is_empty() {
            writeln!(
                formatter,
                "# Props: sprites standing on a spot (the middle of their bottom edge)."
            )?;
            write!(formatter, "{}", YamlList("props", &self.props))?;
        }
        if !self.devices.is_empty() {
            writeln!(
                formatter,
                "# Devices: traps, moving and folding platforms, buttons, doors and\n\
                 # ladders, each standing on a spot (the middle of its bottom edge)."
            )?;
            write!(formatter, "{}", YamlList("devices", &self.devices))?;
        }
        write!(
            formatter,
            "{}",
            YamlList("enemy_spawns", &self.enemy_spawns)
        )?;
        writeln!(
            formatter,
            "# The bonus item, lying beside the door once every enemy is down."
        )?;
        writeln!(formatter, "bonus:")?;
        writeln!(formatter, "  x_h: {}", self.bonus.x_h)?;
        writeln!(formatter, "  y_h: {}", self.bonus.y_h)?;
        writeln!(formatter, "exit:")?;
        writeln!(formatter, "  x_h: {}", self.exit.x_h)?;
        writeln!(formatter, "  y_h: {}", self.exit.y_h)?;
        Ok(())
    }
}

/// A top-level YAML list: `key: []` when empty, else one entry per item.
struct YamlList<'a, T>(&'a str, &'a [T]);

impl<T: Display> Display for YamlList<'_, T> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let Self(key, items) = self;
        if items.is_empty() {
            return writeln!(formatter, "{key}: []");
        }
        writeln!(formatter, "{key}:")?;
        items
            .iter()
            .try_for_each(|item| write!(formatter, "{item}"))
    }
}

impl Display for PlatformConfig {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let label = self
            .label
            .iter()
            .map(|label| format!("label: {}", YamlText(label)));
        let fields = label
            .chain([
                YamlFill(&self.fill).to_string(),
                format!("rect_h: {}", self.rect_h),
            ])
            .chain(placement_fields(&self.placement))
            .collect();
        write!(formatter, "{}", YamlEntry(fields))
    }
}

impl Display for DecorationConfig {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let label = self
            .label
            .iter()
            .map(|label| format!("label: {}", YamlText(label)));
        let fields = label
            .chain([
                YamlFill(&self.fill).to_string(),
                format!("layer: {}", self.layer),
                format!("rect_h: {}", self.rect_h),
            ])
            .chain(placement_fields(&self.placement))
            .collect();
        write!(formatter, "{}", YamlEntry(fields))
    }
}

impl Display for EnemySpawnConfig {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let label = self
            .label
            .iter()
            .map(|label| format!("label: {}", YamlText(label)));
        let armor = (self.armor_layers > 0).then(|| format!("armor_layers: {}", self.armor_layers));
        let wave = (self.wave > 0).then(|| format!("wave: {}", self.wave));
        let fields = label
            .chain([
                format!("type_id: {}", self.type_id),
                format!(
                    "position_h: {{ x: {}, y: {} }}",
                    self.position_h.x_h, self.position_h.y_h
                ),
                format!("facing: {}", self.facing),
            ])
            .chain(armor)
            .chain(wave)
            .collect();
        write!(formatter, "{}", YamlEntry(fields))
    }
}

impl Display for PropConfig {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let label = self
            .label
            .iter()
            .map(|label| format!("label: {}", YamlText(label)));
        let fields = label
            .chain([
                format!("prop: {}", self.prop),
                format!(
                    "position_h: {{ x: {}, y: {} }}",
                    self.position_h.x_h, self.position_h.y_h
                ),
            ])
            .chain(
                (self.layer != DecorationLayer::default())
                    .then(|| format!("layer: {}", self.layer)),
            )
            .chain(self.solid.map(|solid| format!("solid: {solid}")))
            .chain(placement_fields(&self.placement))
            .collect();
        write!(formatter, "{}", YamlEntry(fields))
    }
}

impl Display for DeviceConfig {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let label = self
            .label
            .iter()
            .map(|label| format!("label: {}", YamlText(label)));
        // Every value its kind lists, in the order the kind lists them.
        let mut kind = self.kind.clone();
        let params = kind
            .params()
            .into_iter()
            .map(|param| match &param.value {
                // Words are quoted, as a label is.
                ParamValue::Text(text) => format!("{}: {}", param.key, YamlText(text)),
                value => format!("{}: {value}", param.key),
            })
            .collect::<Vec<_>>();
        let fields = label
            .chain([
                format!("kind: {}", self.kind.tag()),
                format!(
                    "position_h: {{ x: {}, y: {} }}",
                    self.position_h.x_h, self.position_h.y_h
                ),
            ])
            .chain(params)
            .chain(placement_fields(&self.placement))
            .collect();
        write!(formatter, "{}", YamlEntry(fields))
    }
}

/// The fields of a placement that differ from an item as drawn.
fn placement_fields(placement: &Placement) -> impl Iterator<Item = String> {
    // Adding 0.0 turns -0.0 into 0.0, which is no turn at all.
    let rotation = placement.rotation_deg + 0.0;
    [
        placement.mirrored.then(|| String::from("mirrored: true")),
        (rotation != 0.0).then(|| format!("rotation_deg: {rotation}")),
        (placement.opacity != 1.0).then(|| format!("opacity: {}", placement.opacity)),
        placement
            .colorway
            .as_ref()
            .map(|colorway| format!("colorway: {colorway}")),
    ]
    .into_iter()
    .flatten()
}

/// The field a fill is written as: `material: <id>` or a quoted `color:`,
/// since an unquoted `#RRGGBB` would read back as a comment.
struct YamlFill<'a>(&'a Fill);

impl Display for YamlFill<'_> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self.0 {
            Fill::Material(material) => write!(formatter, "material: {material}"),
            Fill::Color(color) => write!(formatter, "color: \"{color}\""),
        }
    }
}

/// One list entry's fields: `  - ` before the first, `    ` before the rest.
struct YamlEntry(Vec<String>);

impl Display for YamlEntry {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        self.0.iter().enumerate().try_for_each(|(index, field)| {
            let indent = if index == 0 { "  - " } else { "    " };
            writeln!(formatter, "{indent}{field}")
        })
    }
}

impl Display for RectH {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{{ x: {}, y: {}, width: {}, height: {} }}",
            self.x, self.y, self.width, self.height
        )
    }
}

/// Also written by the debug editor into `gameplay.yaml` and `enemy_types.yaml`.
impl Display for SizeH {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{{ width: {}, height: {} }}",
            self.width, self.height
        )
    }
}

impl Display for Facing {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Left => "left",
            Self::Right => "right",
        })
    }
}

impl Display for DecorationLayer {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Background => "background",
            Self::Foreground => "foreground",
        })
    }
}

/// A YAML string scalar: plain when that reads back unchanged, otherwise
/// double-quoted with `\` and `"` escaped.
struct YamlText<'a>(&'a str);

impl Display for YamlText<'_> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let text = self.0;
        let is_plain = text.chars().next().is_some_and(char::is_alphabetic)
            && !text.ends_with(' ')
            && text
                .chars()
                .all(|character| character.is_alphanumeric() || " _-.,()/'".contains(character))
            && !matches!(
                text.to_ascii_lowercase().as_str(),
                "true" | "false" | "yes" | "no" | "on" | "off" | "null" | "y" | "n"
            );
        if is_plain {
            formatter.write_str(text)
        } else {
            write!(
                formatter,
                "\"{}\"",
                text.replace('\\', "\\\\").replace('"', "\\\"")
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{
        BackgroundLayerRef, DeviceKind, DeviceTag, GameData, ProfileLayer, SpawnPosition,
    };
    use std::path::Path;

    #[test]
    fn every_room_reads_back_unchanged_after_being_written() {
        let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
        let data = GameData::load(&asset_root).expect("Chapter 1 YAML should be valid");
        for room in data.rooms.values() {
            let written = room.to_string();
            let read_back: RoomConfig =
                yaml_serde::from_str(&written).expect("written rooms are valid YAML");
            assert_eq!(&read_back, room, "room {} round trip", room.room_id);
        }
    }

    #[test]
    fn a_platform_and_a_background_painted_a_plain_color_are_written_back_quoted() {
        let room: RoomConfig = yaml_serde::from_str(
            "
room_id: 3
chapter_id: 1
size: { width_h: 24, height_h: 13.5 }
background_profiles: [city_blue_day, \"#203040\"]
player_spawn: { x_h: 2, y_h: 1 }
platforms:
  - { color: \"#2DE2D5\", rect_h: { x: 0, y: 0, width: 24, height: 1 } }
enemy_spawns: []
bonus: { x_h: 20, y_h: 1 }
exit: { x_h: 22, y_h: 1 }
",
        )
        .expect("a color fill parses");
        let written = room.to_string();
        assert!(
            written.contains("color: \"#2DE2D5\""),
            "an unquoted #RRGGBB would read back as a YAML comment: {written}"
        );
        assert!(
            written.contains("background_profiles: [city_blue_day, \"#203040\"]"),
            "a background color is quoted too: {written}"
        );
        let read_back: RoomConfig =
            yaml_serde::from_str(&written).expect("the written room is valid YAML");
        assert_eq!(read_back, room);
    }

    #[test]
    fn a_platform_keeps_its_turn_through_a_round_trip() {
        let room: RoomConfig = yaml_serde::from_str(
            "
room_id: 3
chapter_id: 1
size: { width_h: 24, height_h: 13.5 }
background_profiles: [city_blue_day]
player_spawn: { x_h: 2, y_h: 1 }
platforms:
  - { material: city_pavement, rect_h: { x: 0, y: 0, width: 24, height: 1 } }
  - { material: city_pavement, rect_h: { x: 8, y: 1, width: 6, height: 0.5 }, rotation_deg: -20 }
enemy_spawns: []
bonus: { x_h: 20, y_h: 1 }
exit: { x_h: 22, y_h: 1 }
",
        )
        .expect("a turned platform parses");
        assert_eq!(room.platforms[0].placement.rotation_deg, 0.0);
        let written = room.to_string();
        assert_eq!(
            written.matches("rotation_deg").count(),
            1,
            "only the turned platform carries it: {written}"
        );
        assert!(written.contains("rotation_deg: -20"), "{written}");
        let read_back: RoomConfig =
            yaml_serde::from_str(&written).expect("the written room is valid YAML");
        assert_eq!(read_back, room);
    }

    #[test]
    fn every_kind_of_device_keeps_all_its_values_through_a_round_trip() {
        let mut room: RoomConfig = yaml_serde::from_str(
            "
room_id: 3
chapter_id: 1
size: { width_h: 24, height_h: 13.5 }
background_profiles: [city_blue_day]
player_spawn: { x_h: 2, y_h: 1 }
platforms:
  - { material: city_pavement, rect_h: { x: 0, y: 0, width: 24, height: 1 } }
devices:
  - { kind: laser, position_h: { x: 4, y: 1 }, length_h: 4.5, orientation: horizontal }
enemy_spawns: []
bonus: { x_h: 20, y_h: 1 }
exit: { x_h: 22, y_h: 1 }
",
        )
        .expect("a device with some of its values left out parses");
        assert!(matches!(
            &room.devices[0].kind,
            DeviceKind::Laser(laser) if laser.length_h == 4.5 && laser.on_s == 1.0
        ));
        // Every kind, each value stepped off its default so a value the
        // writer drops cannot pass for one left at its default.
        for (index, tag) in DeviceTag::ALL.into_iter().enumerate() {
            let mut kind = DeviceKind::from(tag);
            for mut param in kind.params() {
                param.value.step(1.0);
            }
            room.devices.push(DeviceConfig {
                label: Some(format!("{tag}")),
                position_h: SpawnPosition {
                    x_h: index as f32,
                    y_h: 1.0,
                },
                kind,
                // Mirrored, turned, faded and recolored, so a placement the
                // writer drops cannot pass for one left as drawn.
                placement: Placement {
                    mirrored: true,
                    rotation_deg: 15.0 * index as f32,
                    opacity: 0.5,
                    colorway: "dusk".parse().ok(),
                },
            });
        }
        let written = room.to_string();
        let read_back: RoomConfig =
            yaml_serde::from_str(&written).expect("the written room is valid YAML");
        assert_eq!(read_back, room, "{written}");
    }

    #[test]
    fn faded_background_layers_keep_their_opacity_and_a_whole_one_is_left_out() {
        let mut room: RoomConfig = yaml_serde::from_str(
            "
room_id: 3
chapter_id: 1
size: { width_h: 24, height_h: 13.5 }
background_profiles: [city_blue_morning]
layer_opacity: { city_blue_morning/sky: 0.4 }
player_spawn: { x_h: 2, y_h: 1 }
platforms:
  - { material: city_pavement, rect_h: { x: 0, y: 0, width: 24, height: 1 } }
enemy_spawns: []
bonus: { x_h: 20, y_h: 1 }
exit: { x_h: 22, y_h: 1 }
",
        )
        .expect("a faded sky parses");
        let layer = |text: &str| text.parse::<BackgroundLayerRef>().expect("a layer");
        assert_eq!(
            room.layer_opacity("city_blue_morning", ProfileLayer::Sky),
            0.4
        );
        // Faded out a step, then the sky back in to whole.
        room.fade_layer(layer("city_blue_morning/2"), -0.1);
        for _ in 0..6 {
            room.fade_layer(layer("city_blue_morning/sky"), 0.1);
        }
        let written = room.to_string();
        assert!(
            written.contains("layer_opacity: { city_blue_morning/2: 0.9 }"),
            "only the faded layer is written: {written}"
        );
        let read_back: RoomConfig =
            yaml_serde::from_str(&written).expect("the written room is valid YAML");
        assert_eq!(read_back, room);
    }

    #[test]
    fn props_keep_their_spot_and_layer_through_a_round_trip() {
        let room: RoomConfig = yaml_serde::from_str(
            "
room_id: 3
chapter_id: 1
size: { width_h: 24, height_h: 13.5 }
background_profiles: [city_blue_day]
player_spawn: { x_h: 2, y_h: 1 }
platforms:
  - { material: city_pavement, rect_h: { x: 0, y: 0, width: 24, height: 1 } }
props:
  - { prop: sci_fi_small_crate, position_h: { x: 6.5, y: 1 } }
  - { label: lamp, prop: sci_fi_ceiling_lamp, position_h: { x: 10, y: 8 }, layer: foreground }
  - { prop: sci_fi_crate_stack, position_h: { x: 14, y: 1 }, solid: false }
enemy_spawns: []
bonus: { x_h: 20, y_h: 1 }
exit: { x_h: 22, y_h: 1 }
",
        )
        .expect("props parse");
        let written = room.to_string();
        assert!(
            !written.contains("layer: background"),
            "the default layer is left out: {written}"
        );
        assert_eq!(
            written.matches("solid:").count(),
            1,
            "only the prop that says whether it is solid writes it: {written}"
        );
        let read_back: RoomConfig =
            yaml_serde::from_str(&written).expect("the written room is valid YAML");
        assert_eq!(read_back, room);
    }

    #[test]
    fn an_enemy_keeps_the_way_it_faces_through_a_round_trip() {
        let room: RoomConfig = yaml_serde::from_str(
            "
room_id: 3
chapter_id: 1
size: { width_h: 24, height_h: 13.5 }
background_profiles: [city_blue_day]
player_spawn: { x_h: 2, y_h: 1 }
platforms:
  - { material: city_pavement, rect_h: { x: 0, y: 0, width: 24, height: 1 } }
enemy_spawns:
  - { type_id: blade_unit, position_h: { x: 10, y: 1 }, facing: right }
  - { type_id: blade_unit, position_h: { x: 14, y: 1 }, armor_layers: 2 }
bonus: { x_h: 20, y_h: 1 }
exit: { x_h: 22, y_h: 1 }
",
        )
        .expect("a spawn facing right parses");
        assert_eq!(room.enemy_spawns[0].facing, Facing::Right);
        // An enemy watches the way the player walks in from unless the room
        // says otherwise.
        assert_eq!(room.enemy_spawns[1].facing, Facing::Left);
        let written = room.to_string();
        assert!(
            written.contains("facing: right"),
            "every spawn is written with the way it faces: {written}"
        );
        assert!(
            written.contains("armor_layers: 2"),
            "an enemy's armor is written: {written}"
        );
        let read_back: RoomConfig =
            yaml_serde::from_str(&written).expect("the written room is valid YAML");
        assert_eq!(read_back, room);
    }

    #[test]
    fn labels_that_yaml_would_misread_are_quoted() {
        assert_eq!(YamlText("hanging rock").to_string(), "hanging rock");
        assert_eq!(YamlText("Walkway: west").to_string(), "\"Walkway: west\"");
        assert_eq!(YamlText("no").to_string(), "\"no\"");
        assert_eq!(YamlText("say \"hi\"").to_string(), "\"say \\\"hi\\\"\"");
    }
}

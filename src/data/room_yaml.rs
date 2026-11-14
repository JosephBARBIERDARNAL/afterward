//! Writes a room back to YAML in the layout of `rooms/room_NNN.yaml`, so the
//! debug editor's saves stay readable and diff cleanly against hand edits.

use std::fmt::{self, Display, Formatter};

use super::{
    Backdrop, DecorationConfig, DecorationLayer, EnemySpawnConfig, Facing, Fill, PlatformConfig,
    RectH, RoomConfig, RoomHazardConfig, SizeH,
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
        write!(
            formatter,
            "{}",
            YamlList("enemy_spawns", &self.enemy_spawns)
        )?;
        if !self.hazards.is_empty() {
            writeln!(
                formatter,
                "# Vents: each warns for arm_s, burns for burn_s, rests for rest_s, and\n\
                 # starts over, offset_s into that cycle as the room opens."
            )?;
            write!(formatter, "{}", YamlList("hazards", &self.hazards))?;
        }
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
        writeln!(formatter, "next_room_id: {}", self.next_room_id)
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
            .chain(
                // Adding 0.0 turns -0.0 into 0.0, which is no turn at all.
                Some(self.rotation_deg + 0.0)
                    .filter(|rotation| *rotation != 0.0)
                    .map(|rotation| format!("rotation_deg: {rotation}")),
            )
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

impl Display for RoomHazardConfig {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let label = self
            .label
            .iter()
            .map(|label| format!("label: {}", YamlText(label)));
        let fields = label
            .chain([
                format!("rect_h: {}", self.rect_h),
                format!("damage: {}", self.damage),
                format!("arm_s: {}", self.arm_s),
                format!("burn_s: {}", self.burn_s),
                format!("rest_s: {}", self.rest_s),
            ])
            .chain((self.offset_s != 0.0).then(|| format!("offset_s: {}", self.offset_s)))
            .collect();
        write!(formatter, "{}", YamlEntry(fields))
    }
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
    use crate::data::GameData;
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
next_room_id: 4
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
next_room_id: 4
",
        )
        .expect("a turned platform parses");
        assert_eq!(room.platforms[0].rotation_deg, 0.0);
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
next_room_id: 4
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

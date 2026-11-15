use super::*;
use crate::data::TellsConfig;
use bevy::text::FontSize;

/// What a number over a body says.
#[derive(Clone, Copy, Debug)]
pub(in crate::game) enum Number {
    /// The damage a blow dealt, `boost` times what it would have dealt
    /// without the run's bonuses: a boosted one is drawn larger, in bold
    /// italic and gold, so a bonus at work is seen on every hit it adds to.
    Dealt { damage: f32, boost: f32 },
    /// The damage the player (or their double) took, and how much more a
    /// bonus soaked, written beside it.
    Taken { damage: f32, soaked: f32 },
    /// Health given back.
    Healed(f32),
}

pub(in crate::game) fn spawn_damage_number(
    commands: &mut Commands,
    tells: &TellsConfig,
    position: Vec3,
    number: Number,
) {
    let (text, color, font_px, role) = match number {
        Number::Dealt { damage, boost } if boost > 1.001 => (
            format!("{damage:.0}"),
            Color::from(tells.boosted_color),
            16.0 * (1.0 + (boost - 1.0) * tells.boosted_number_growth)
                .min(tells.boosted_number_max),
            Some(FontRole::BoldItalic),
        ),
        Number::Dealt { damage, .. } => (format!("{damage:.0}"), Color::WHITE, 16.0, None),
        Number::Taken { damage, .. } => (
            format!("{damage:.0}"),
            Color::srgb(1.0, 0.15, 0.18),
            20.0,
            None,
        ),
        Number::Healed(healed) => (
            format!("+{healed:.0}"),
            Color::from(tells.healed_color),
            16.0,
            Some(FontRole::BoldItalic),
        ),
    };
    let mut spawned = commands.spawn((
        Text2d::new(text),
        TextFont {
            font_size: FontSize::Px(font_px),
            ..default()
        },
        TextColor(color),
        Transform::from_translation(position + Vec3::new(0.0, 34.0, 20.0)),
        DamageNumber {
            timer: Timer::from_seconds(0.65, TimerMode::Once),
            color,
        },
        RoomEntity,
    ));
    if let Some(role) = role {
        spawned.insert(role);
    }
    // What a bonus soaked rides beside the damage that got through.
    if let Number::Taken { soaked, .. } = number
        && soaked >= 0.5
    {
        let color = Color::from(tells.soaked_color);
        commands.spawn((
            Text2d::new(format!("-{soaked:.0} soaked")),
            TextFont {
                font_size: FontSize::Px(12.0),
                ..default()
            },
            TextColor(color),
            FontRole::BoldItalic,
            Transform::from_translation(position + Vec3::new(0.0, 18.0, 20.0)),
            DamageNumber {
                timer: Timer::from_seconds(0.8, TimerMode::Once),
                color,
            },
            RoomEntity,
        ));
    }
}

pub(super) fn set_enemy_animation(
    sprite: &mut Sprite,
    animation: &mut AnimatedSprite,
    sheets: &SpriteSheets,
    type_id: &EnemyTypeId,
    action: &str,
) -> bool {
    // A type whose sheets are named after it and the action (`<type>_<action>`)
    // needs no line below.
    let own = format!("{type_id}_{action}");
    let own_walk = format!("{type_id}_walk");
    let key = match action {
        _ if sheets.0.contains_key(&own) => own.as_str(),
        // A type with sheets of its own and none for the air leaps in its walk.
        "jump" | "fall" if sheets.0.contains_key(&own_walk) => own_walk.as_str(),
        // Without dedicated art, airborne enemies use their movement sheet.
        "walk" | "jump" | "fall" => match &**type_id {
            "rifle_unit" => "enemy_infantry_walk",
            "heavy_unit" => "enemy_destroyer_walk",
            "dark_rifle_unit" => "enemy_dark_infantry_walk",
            "dark_heavy_unit" => "enemy_dark_destroyer_walk",
            _ => type_id.idle_sheet(),
        },
        "attack" => match &**type_id {
            "blade_unit" => "enemy_swordsman_attack",
            "rifle_unit" => "enemy_infantry_attack",
            "heavy_unit" => "enemy_destroyer_attack",
            "dark_blade_unit" => "enemy_dark_swordsman_attack",
            "dark_rifle_unit" => "enemy_dark_infantry_attack",
            "dark_heavy_unit" => "enemy_dark_destroyer_attack",
            _ => type_id.idle_sheet(),
        },
        "hurt" => match &**type_id {
            "blade_unit" => "enemy_swordsman_hurt",
            "rifle_unit" => "enemy_infantry_hurt",
            "heavy_unit" => "enemy_destroyer_hurt",
            "dark_blade_unit" => "enemy_dark_swordsman_hurt",
            "dark_rifle_unit" => "enemy_dark_infantry_hurt",
            "dark_heavy_unit" => "enemy_dark_destroyer_hurt",
            _ => type_id.idle_sheet(),
        },
        "dead" => match &**type_id {
            "blade_unit" => "enemy_swordsman_dead",
            "rifle_unit" => "enemy_infantry_dead",
            "heavy_unit" => "enemy_destroyer_dead",
            "dark_blade_unit" => "enemy_dark_swordsman_dead",
            "dark_rifle_unit" => "enemy_dark_infantry_dead",
            "dark_heavy_unit" => "enemy_dark_destroyer_dead",
            _ => type_id.idle_sheet(),
        },
        _ => type_id.idle_sheet(),
    };
    let Some(sheet) = sheets.0.get(key) else {
        return false;
    };
    if animation.sheet == key {
        return false;
    }
    sprite.image = sheet.image.clone();
    sprite.texture_atlas = Some(TextureAtlas {
        layout: sheet.layout.clone(),
        index: 0,
    });
    animation.sheet = key.to_string();
    animation.frame = 0;
    animation.frames = sheet.frames;
    animation.looping =
        matches!(action, "walk" | "idle" | "jump" | "fall") && !key.ends_with("_jump");
    animation.timer = Timer::from_seconds(1.0 / sheet.fps, TimerMode::Repeating);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn received_damage_is_red_and_a_soaked_share_is_written_beside_it() {
        let data =
            GameData::load(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets"))
                .expect("the game data should be valid");
        let mut app = App::new();
        app.insert_resource(data);
        app.add_systems(Startup, |mut commands: Commands, data: Res<GameData>| {
            spawn_damage_number(
                &mut commands,
                &data.gameplay.effects.tells,
                Vec3::ZERO,
                Number::Taken {
                    damage: 8.0,
                    soaked: 2.0,
                },
            );
        });
        app.update();

        let mut numbers = app.world_mut().query::<(&Text2d, &TextColor)>();
        let colors = numbers
            .iter(app.world())
            .map(|(text, color)| (text.0.clone(), color.0))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(colors.get("8"), Some(&Color::srgb(1.0, 0.15, 0.18)));
        assert!(colors.contains_key("-2 soaked"));
    }
}

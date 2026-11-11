use super::*;
use bevy::text::{FontSize, FontWeight};

pub(super) fn spawn_damage_number(
    commands: &mut Commands,
    position: Vec3,
    damage: f32,
    received: bool,
) {
    let color = if received {
        Color::srgb(1.0, 0.15, 0.18)
    } else {
        Color::WHITE
    };
    commands.spawn((
        Text2d::new(format!("{damage:.0}")),
        TextFont {
            font_size: FontSize::Px(if received { 20.0 } else { 16.0 }),
            weight: if received {
                FontWeight::BOLD
            } else {
                FontWeight::NORMAL
            },
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
}

pub(super) fn set_enemy_animation(
    sprite: &mut Sprite,
    animation: &mut AnimatedSprite,
    sheets: &SpriteSheets,
    type_id: &EnemyTypeId,
    action: &str,
) {
    let key = match action {
        "jump" if &**type_id == "emitter_unit" => "boss_jump",
        "fall" if &**type_id == "emitter_unit" => "boss_fall",
        // Without dedicated art, airborne enemies use their movement sheet.
        "walk" | "jump" | "fall" => match &**type_id {
            "rifle_unit" => "enemy_infantry_walk",
            "heavy_unit" => "enemy_destroyer_walk",
            "dark_rifle_unit" => "enemy_dark_infantry_walk",
            "dark_heavy_unit" => "enemy_dark_destroyer_walk",
            "stalker_unit" => "enemy_gorgon_1_run",
            "lancer_unit" => "enemy_gorgon_2_run",
            "breaker_unit" => "enemy_gorgon_3_run",
            "scout_unit" => "enemy_dark_ninja_walk",
            "emitter_unit" => "boss_walk",
            _ => type_id.idle_sheet(),
        },
        "attack" | "attack_1" => match &**type_id {
            "blade_unit" => "enemy_swordsman_attack",
            "rifle_unit" => "enemy_infantry_attack",
            "heavy_unit" => "enemy_destroyer_attack",
            "dark_blade_unit" => "enemy_dark_swordsman_attack",
            "dark_rifle_unit" => "enemy_dark_infantry_attack",
            "dark_heavy_unit" => "enemy_dark_destroyer_attack",
            "stalker_unit" => "enemy_gorgon_1_attack",
            "lancer_unit" => "enemy_gorgon_2_attack",
            "breaker_unit" => "enemy_gorgon_3_attack",
            "scout_unit" => "enemy_dark_ninja_attack",
            "emitter_unit" => "boss_attack",
            _ => type_id.idle_sheet(),
        },
        "attack_2" => {
            if &**type_id == "emitter_unit" {
                "boss_attack_2"
            } else {
                type_id.idle_sheet()
            }
        }
        "hurt" => match &**type_id {
            "blade_unit" => "enemy_swordsman_hurt",
            "rifle_unit" => "enemy_infantry_hurt",
            "heavy_unit" => "enemy_destroyer_hurt",
            "dark_blade_unit" => "enemy_dark_swordsman_hurt",
            "dark_rifle_unit" => "enemy_dark_infantry_hurt",
            "dark_heavy_unit" => "enemy_dark_destroyer_hurt",
            "stalker_unit" => "enemy_gorgon_1_hurt",
            "lancer_unit" => "enemy_gorgon_2_hurt",
            "breaker_unit" => "enemy_gorgon_3_hurt",
            "scout_unit" => "enemy_dark_ninja_hurt",
            "emitter_unit" => "boss_hurt",
            _ => type_id.idle_sheet(),
        },
        "dead" => match &**type_id {
            "blade_unit" => "enemy_swordsman_dead",
            "rifle_unit" => "enemy_infantry_dead",
            "heavy_unit" => "enemy_destroyer_dead",
            "dark_blade_unit" => "enemy_dark_swordsman_dead",
            "dark_rifle_unit" => "enemy_dark_infantry_dead",
            "dark_heavy_unit" => "enemy_dark_destroyer_dead",
            "stalker_unit" => "enemy_gorgon_1_dead",
            "lancer_unit" => "enemy_gorgon_2_dead",
            "breaker_unit" => "enemy_gorgon_3_dead",
            "scout_unit" => "enemy_dark_ninja_dead",
            "emitter_unit" => "boss_dead",
            _ => type_id.idle_sheet(),
        },
        _ => type_id.idle_sheet(),
    };
    let Some(sheet) = sheets.0.get(key) else {
        return;
    };
    if animation.sheet == key {
        return;
    }
    sprite.image = sheet.image.clone();
    sprite.texture_atlas = Some(TextureAtlas {
        layout: sheet.layout.clone(),
        index: 0,
    });
    animation.sheet = key.to_string();
    animation.frame = 0;
    animation.frames = sheet.frames;
    animation.looping = matches!(action, "walk" | "idle" | "jump" | "fall") && key != "boss_jump";
    animation.timer = Timer::from_seconds(1.0 / sheet.fps, TimerMode::Repeating);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn received_damage_numbers_are_bold_red() {
        let mut app = App::new();
        app.add_systems(Startup, |mut commands: Commands| {
            spawn_damage_number(&mut commands, Vec3::ZERO, 8.0, true);
        });
        app.update();

        let mut numbers = app.world_mut().query::<(&TextFont, &TextColor)>();
        let (font, color) = numbers
            .single(app.world())
            .expect("the damage number should be spawned");
        assert_eq!(font.weight, FontWeight::BOLD);
        assert_eq!(color.0, Color::srgb(1.0, 0.15, 0.18));
    }
}

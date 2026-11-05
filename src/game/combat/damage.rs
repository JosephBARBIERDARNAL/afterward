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
        "jump" if &**type_id == "evil_wizard" => "boss_jump",
        "fall" if &**type_id == "evil_wizard" => "boss_fall",
        // Without dedicated art, airborne enemies use their movement sheet.
        "walk" | "jump" | "fall" => match &**type_id {
            "infantryman" => "enemy_infantry_walk",
            "destroyer" => "enemy_destroyer_walk",
            "gorgon_1" => "enemy_gorgon_1_run",
            "gorgon_2" => "enemy_gorgon_2_run",
            "gorgon_3" => "enemy_gorgon_3_run",
            "dark_ninja" => "enemy_dark_ninja_walk",
            "evil_wizard" => "boss_walk",
            _ => type_id.idle_sheet(),
        },
        "attack" | "attack_1" => match &**type_id {
            "swordsman" => "enemy_swordsman_attack",
            "infantryman" => "enemy_infantry_attack",
            "destroyer" => "enemy_destroyer_attack",
            "gorgon_1" => "enemy_gorgon_1_attack",
            "gorgon_2" => "enemy_gorgon_2_attack",
            "gorgon_3" => "enemy_gorgon_3_attack",
            "dark_ninja" => "enemy_dark_ninja_attack",
            "evil_wizard" => "boss_attack",
            _ => type_id.idle_sheet(),
        },
        "attack_2" => {
            if &**type_id == "evil_wizard" {
                "boss_attack_2"
            } else {
                type_id.idle_sheet()
            }
        }
        "hurt" => match &**type_id {
            "swordsman" => "enemy_swordsman_hurt",
            "infantryman" => "enemy_infantry_hurt",
            "destroyer" => "enemy_destroyer_hurt",
            "gorgon_1" => "enemy_gorgon_1_hurt",
            "gorgon_2" => "enemy_gorgon_2_hurt",
            "gorgon_3" => "enemy_gorgon_3_hurt",
            "dark_ninja" => "enemy_dark_ninja_hurt",
            "evil_wizard" => "boss_hurt",
            _ => type_id.idle_sheet(),
        },
        "dead" => match &**type_id {
            "swordsman" => "enemy_swordsman_dead",
            "infantryman" => "enemy_infantry_dead",
            "destroyer" => "enemy_destroyer_dead",
            "gorgon_1" => "enemy_gorgon_1_dead",
            "gorgon_2" => "enemy_gorgon_2_dead",
            "gorgon_3" => "enemy_gorgon_3_dead",
            "dark_ninja" => "enemy_dark_ninja_dead",
            "evil_wizard" => "boss_dead",
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

use bevy::prelude::*;

use crate::bevy_game::model::{Door, Enemy, Player, Projectile, SceneEntity};

pub(crate) const DAMAGE_NUMBER_LIFETIME: f32 = 0.65;

#[derive(Resource)]
pub(crate) struct Art {
    pub(crate) circle: Handle<Mesh>,
    pub(crate) player: Handle<ColorMaterial>,
    pub(crate) melee: Handle<ColorMaterial>,
    pub(crate) ranged: Handle<ColorMaterial>,
    pub(crate) heavy: Handle<ColorMaterial>,
    pub(crate) charger: Handle<ColorMaterial>,
    pub(crate) skirmisher: Handle<ColorMaterial>,
    pub(crate) bomber: Handle<ColorMaterial>,
    pub(crate) elite_ring: Handle<ColorMaterial>,
    pub(crate) warning: Handle<ColorMaterial>,
}

#[derive(Component)]
pub(crate) struct DamageNumber {
    pub(crate) age: f32,
    pub(crate) received: bool,
}

#[derive(Component)]
pub(crate) struct PlayerWeapon {
    pub(crate) offset: f32,
}

#[derive(Component)]
pub(crate) struct ActionEffect {
    pub(crate) age: f32,
    pub(crate) lifetime: f32,
    pub(crate) start_scale: f32,
    pub(crate) end_scale: f32,
    pub(crate) rise_speed: f32,
    pub(crate) rgba: [f32; 4],
}

#[derive(Component)]
pub(crate) struct AttackTelegraph {
    pub(crate) age: f32,
    pub(crate) lifetime: f32,
    pub(crate) radius: f32,
}

pub(crate) fn setup_art(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(Art {
        circle: meshes.add(Circle::new(1.0)),
        player: materials.add(Color::srgb(0.91, 0.84, 0.66)),
        melee: materials.add(Color::srgb(0.77, 0.42, 0.31)),
        ranged: materials.add(Color::srgb(0.60, 0.51, 0.73)),
        heavy: materials.add(Color::srgb(0.66, 0.53, 0.30)),
        charger: materials.add(Color::srgb(0.76, 0.29, 0.26)),
        skirmisher: materials.add(Color::srgb(0.38, 0.67, 0.63)),
        bomber: materials.add(Color::srgb(0.77, 0.49, 0.27)),
        elite_ring: materials.add(Color::srgba(0.90, 0.72, 0.40, 0.72)),
        warning: materials.add(Color::srgba(0.96, 0.16, 0.12, 0.52)),
    });
}

pub(crate) fn spawn_player_weapon(commands: &mut Commands) {
    for (offset, size, color) in [
        (31.0, Vec2::new(30.0, 5.0), Color::srgb(0.98, 0.90, 0.69)),
        (17.0, Vec2::new(6.0, 11.0), Color::srgb(0.70, 0.48, 0.25)),
    ] {
        commands.spawn((
            Sprite::from_color(color, size),
            Transform::from_xyz(0.0, 0.0, 3.5),
            SceneEntity,
            PlayerWeapon { offset },
        ));
    }
}

pub(crate) fn update_player_weapon(weapon: &mut WeaponQuery<'_, '_>, player: Vec2, facing: Vec2) {
    let facing = facing.normalize_or_zero();
    let facing = if facing == Vec2::ZERO {
        Vec2::X
    } else {
        facing
    };
    let rotation = Quat::from_rotation_z(facing.to_angle());
    for (mut transform, weapon_part) in weapon.iter_mut() {
        transform.translation = (player + facing * weapon_part.offset).extend(3.5);
        transform.rotation = rotation;
    }
}

pub(crate) struct ActionVisual {
    pub(crate) position: Vec2,
    pub(crate) angle: f32,
    pub(crate) size: Vec2,
    pub(crate) rgba: [f32; 4],
    pub(crate) lifetime: f32,
    pub(crate) start_scale: f32,
    pub(crate) end_scale: f32,
    pub(crate) rise_speed: f32,
    pub(crate) z: f32,
}

pub(crate) fn spawn_action_effect(commands: &mut Commands, visual: ActionVisual) {
    commands.spawn((
        Sprite::from_color(
            Color::srgba(
                visual.rgba[0],
                visual.rgba[1],
                visual.rgba[2],
                visual.rgba[3],
            ),
            visual.size,
        ),
        Transform::from_xyz(visual.position.x, visual.position.y, visual.z)
            .with_rotation(Quat::from_rotation_z(visual.angle))
            .with_scale(Vec3::splat(visual.start_scale)),
        SceneEntity,
        ActionEffect {
            age: 0.0,
            lifetime: visual.lifetime,
            start_scale: visual.start_scale,
            end_scale: visual.end_scale,
            rise_speed: visual.rise_speed,
            rgba: visual.rgba,
        },
    ));
}

pub(crate) fn animate_action_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut effects: Query<(Entity, &mut Transform, &mut Sprite, &mut ActionEffect)>,
) {
    let dt = time.delta_secs().min(0.05);
    for (entity, mut transform, mut sprite, mut effect) in &mut effects {
        effect.age += dt;
        if effect.age >= effect.lifetime {
            commands.entity(entity).despawn();
            continue;
        }

        let progress = (effect.age / effect.lifetime).clamp(0.0, 1.0);
        let scale = effect.start_scale + (effect.end_scale - effect.start_scale) * progress;
        transform.scale = Vec3::splat(scale);
        transform.translation.y += effect.rise_speed * dt;
        sprite.color = Color::srgba(
            effect.rgba[0],
            effect.rgba[1],
            effect.rgba[2],
            effect.rgba[3] * (1.0 - progress),
        );
    }
}

pub(crate) fn animate_attack_telegraphs(
    time: Res<Time>,
    mut warnings: Query<(&mut Transform, &mut AttackTelegraph)>,
) {
    let dt = time.delta_secs().min(0.05);
    for (mut transform, mut warning) in &mut warnings {
        warning.age += dt;
        let progress = (warning.age / warning.lifetime).clamp(0.0, 1.0);
        let pulse = if progress > 0.75 {
            1.0 + (progress * 32.0).sin().abs() * 0.12
        } else {
            1.0
        };
        let scale = warning.radius * (0.12 + 0.88 * progress) * pulse;
        transform.scale = Vec3::splat(scale);
    }
}

pub(crate) type WeaponQuery<'w, 's> = Query<
    'w,
    's,
    (&'static mut Transform, &'static PlayerWeapon),
    (
        With<PlayerWeapon>,
        Without<Player>,
        Without<Enemy>,
        Without<Projectile>,
        Without<Door>,
    ),
>;

pub(crate) fn spawn_damage(commands: &mut Commands, position: Vec2, amount: f32, received: bool) {
    let color = if received {
        Color::srgb(0.95, 0.20, 0.16)
    } else {
        Color::srgb(0.98, 0.90, 0.72)
    };
    let weight = if received {
        FontWeight::BOLD
    } else {
        FontWeight::SEMIBOLD
    };
    commands.spawn((
        Text2d::new(format!("-{}", amount as i32)),
        TextFont {
            font_size: FontSize::Px(if received { 25.0 } else { 22.0 }),
            weight,
            font_smoothing: FontSmoothing::None,
            ..default()
        },
        TextColor(color),
        Transform::from_xyz(position.x, position.y + 25.0, 8.0).with_scale(Vec3::splat(0.72)),
        SceneEntity,
        DamageNumber { age: 0.0, received },
    ));
}

pub(crate) fn animate_damage_numbers(
    mut commands: Commands,
    time: Res<Time>,
    mut numbers: Query<(Entity, &mut Transform, &mut TextColor, &mut DamageNumber)>,
) {
    let dt = time.delta_secs().min(0.05);
    for (entity, mut transform, mut color, mut number) in &mut numbers {
        number.age += dt;
        if number.age >= DAMAGE_NUMBER_LIFETIME {
            commands.entity(entity).despawn();
            continue;
        }

        transform.translation.y += 42.0 * dt;
        let scale = if number.age < 0.09 {
            0.72 + number.age / 0.09 * 0.58
        } else if number.age < 0.19 {
            1.30 - (number.age - 0.09) / 0.10 * 0.30
        } else {
            1.0
        };
        transform.scale = Vec3::splat(scale);

        let alpha = if number.age > 0.40 {
            ((DAMAGE_NUMBER_LIFETIME - number.age) / (DAMAGE_NUMBER_LIFETIME - 0.40))
                .clamp(0.0, 1.0)
        } else {
            1.0
        };
        let rgb = if number.received {
            (0.95, 0.20, 0.16)
        } else {
            (0.98, 0.90, 0.72)
        };
        color.0 = Color::srgba(rgb.0, rgb.1, rgb.2, alpha);
    }
}

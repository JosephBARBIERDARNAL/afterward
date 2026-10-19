use super::*;
use bevy::camera::ScalingMode;

pub(super) fn spawn_camera(mut commands: Commands, data: Res<GameData>) {
    let view_height = data.gameplay.viewport.height_h * H_WORLD_UNITS;
    let first_room = &data.rooms[&1];
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: view_height,
            },
            ..OrthographicProjection::default_2d()
        }),
        Camera {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        RoomCamera {
            width_h: f32::from(first_room.size.width_viewports) * 16.0,
            fixed_y_h: first_room.camera.fixed_y_h,
            follows_player: first_room.camera.follow_player_x,
        },
        Transform::from_xyz(8.0 * H_WORLD_UNITS, view_height / 2.0, 100.0),
    ));
}

pub(super) fn follow_camera(
    player: Query<&Transform, (With<PlayerBody>, Without<Camera2d>)>,
    mut camera: Query<(&mut Transform, &RoomCamera), With<Camera2d>>,
    data: Res<GameData>,
) {
    let Ok(player) = player.single() else {
        return;
    };
    let Ok((mut transform, room_camera)) = camera.single_mut() else {
        return;
    };
    if !room_camera.follows_player {
        return;
    }
    let visible_width_h = data.gameplay.viewport.height_h
        * f32::from(data.gameplay.viewport.aspect_ratio.width)
        / f32::from(data.gameplay.viewport.aspect_ratio.height);
    let half_width = visible_width_h * H_WORLD_UNITS / 2.0;
    let room_width = room_camera.width_h * H_WORLD_UNITS;
    transform.translation.x = player
        .translation
        .x
        .clamp(half_width, room_width - half_width);
    transform.translation.y = room_camera.fixed_y_h * H_WORLD_UNITS;
}

pub(super) fn update_background_parallax(
    camera: Query<&Transform, With<Camera2d>>,
    mut backgrounds: Query<(&mut Transform, &BackgroundParallax), Without<Camera2d>>,
) {
    let Ok(camera) = camera.single() else {
        return;
    };
    for (mut transform, background) in &mut backgrounds {
        transform.translation.x =
            background.base_x + camera.translation.x * (1.0 - background.factor);
    }
}

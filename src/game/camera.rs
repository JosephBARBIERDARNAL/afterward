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
        pause_blur::PauseBlur::default(),
        RoomCamera {
            size_h: Vec2::new(first_room.width_h(&data.gameplay), first_room.size.height_h),
        },
        Transform::from_xyz(
            data.gameplay.viewport_width_h() / 2.0 * H_WORLD_UNITS,
            view_height / 2.0,
            100.0,
        ),
    ));
}

/// Tracks the player horizontally, and vertically once they leave the dead
/// zone around the screen center; the view never leaves the room.
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
    let player_h = player.translation.truncate() / H_WORLD_UNITS;
    let camera_y_h = transform.translation.y / H_WORLD_UNITS;
    let dead_zone_h = data.gameplay.camera.vertical_dead_zone_h;
    let target_h = Vec2::new(
        player_h.x,
        camera_y_h.clamp(player_h.y - dead_zone_h, player_h.y + dead_zone_h),
    );
    let center_h = room_camera.clamp_center_h(target_h, &data.gameplay);
    transform.translation.x = center_h.x * H_WORLD_UNITS;
    transform.translation.y = center_h.y * H_WORLD_UNITS;
}

/// Background layers scroll horizontally at their parallax rate and stay
/// centered on the camera vertically, behind the room's drawn terrain.
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
        transform.translation.y = camera.translation.y;
    }
}

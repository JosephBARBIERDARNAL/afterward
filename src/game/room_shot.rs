//! A picture of one room, whole, for looking at a layout without playing it:
//! `--room 3 --shot` builds the room as a run would, draws all of it into an
//! image rather than a window, writes `target/room-shots/room_003.png` and
//! quits. `--grid` rules it in H, so a spot in the picture can be read back
//! into the room's YAML. `scripts/room_shot.sh` (`just shot 3`) is the way in.
//!
//! The room is the real one — `world::spawn_room`, on the `Playing` screen —
//! held on its first frame: the clock is paused, so nobody moves. Three things
//! differ from what a player sees, since the player only ever sees a 16:9
//! window of it: the camera takes in the whole room, the backdrop is enlarged
//! to cover a room taller than it, and there is no HUD.

use super::*;
use bevy::{
    app::AppExit,
    camera::{RenderTarget, ScalingMode},
    render::{
        render_resource::TextureFormat,
        view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk},
    },
    sprite::Anchor,
    text::FontSize,
};
use std::{fs, path::PathBuf};

/// The picture's longer side, at most. A room small enough is drawn at the
/// art's own size, `H_WORLD_UNITS` pixels per H, and no larger.
const LONG_SIDE_PX: f32 = 1920.0;
/// Frames the room is left to stand, once its images are in, before the
/// picture is taken: the textures still have to reach the GPU.
const SETTLE_FRAMES: u32 = 20;
/// Frames after which a room whose images never arrived is given up on.
const GIVE_UP_FRAMES: u32 = 1800;
/// The ruled lines stand this many H apart, and a number every few of them.
const GRID_LABEL_EVERY_H: usize = 5;
const GRID_LABEL_PX: f32 = 13.0;
const GRID_COLOR: Color = Color::srgb(0.75, 0.95, 1.0);
const GRID_Z: f32 = 50.0;

pub struct RoomShotPlugin {
    /// Rule the picture in H.
    pub grid: bool,
}

impl Plugin for RoomShotPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RoomShot {
            grid: self.grid,
            target: Handle::default(),
            settled_frames: 0,
            waited_frames: 0,
            requested: false,
        })
        // After `Startup`, where the camera is spawned.
        .add_systems(PostStartup, (open_the_room, aim_the_camera))
        // Once the game's own camera systems have had their say, and before
        // the frame's transforms are worked out from what this leaves.
        .add_systems(
            PostUpdate,
            (frame_the_room, take_the_shot)
                .chain()
                .before(TransformSystems::Propagate),
        );
    }
}

#[derive(Resource)]
struct RoomShot {
    grid: bool,
    /// The image the camera draws the room into.
    target: Handle<Image>,
    settled_frames: u32,
    waited_frames: u32,
    requested: bool,
}

/// Opens the start room with the clock stopped, so it holds its first frame.
fn open_the_room(
    start_room: Res<StartRoom>,
    mut session: ResMut<RunSession>,
    mut next_screen: ResMut<NextState<GameScreen>>,
    mut clock: ResMut<Time<Virtual>>,
) {
    session.room_id = start_room.0;
    next_screen.set(GameScreen::Playing);
    clock.pause();
}

/// Points the camera at an image of the room's own shape instead of a window,
/// and rules the room in H when a grid was asked for.
fn aim_the_camera(
    mut commands: Commands,
    data: Res<GameData>,
    start_room: Res<StartRoom>,
    mut shot: ResMut<RoomShot>,
    mut images: ResMut<Assets<Image>>,
    mut camera: Query<(Entity, &mut Projection), With<Camera2d>>,
) {
    let size_h = &data.rooms[&start_room.0].size;
    let size_h = Vec2::new(size_h.width_h, size_h.height_h);
    let px_per_h = (LONG_SIDE_PX / size_h.max_element()).min(H_WORLD_UNITS);
    let size_px = (size_h * px_per_h).round().as_uvec2();

    shot.target = images.add(Image::new_target_texture(
        size_px.x,
        size_px.y,
        TextureFormat::Rgba8UnormSrgb,
        None,
    ));
    if let Ok((entity, mut projection)) = camera.single_mut() {
        *projection = Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::Fixed {
                width: size_h.x * H_WORLD_UNITS,
                height: size_h.y * H_WORLD_UNITS,
            },
            ..OrthographicProjection::default_2d()
        });
        commands
            .entity(entity)
            .insert(RenderTarget::from(shot.target.clone()));
    }

    if !shot.grid {
        return;
    }
    // One picture pixel, in world units: lines that thin stay crisp.
    let px = H_WORLD_UNITS / px_per_h;
    let font = TextFont {
        font_size: FontSize::Px(GRID_LABEL_PX * px),
        ..default()
    };
    // Vertical lines first, then horizontal ones: `along` is the axis a line
    // is counted on, `across` the one it runs the length of.
    for (along, across) in [(Vec2::X, Vec2::Y), (Vec2::Y, Vec2::X)] {
        for line in 0..=size_h.dot(along).floor() as usize {
            let labelled = line % GRID_LABEL_EVERY_H == 0;
            let at = along * line as f32 * H_WORLD_UNITS;
            commands.spawn((
                Sprite::from_color(
                    GRID_COLOR.with_alpha(if labelled { 0.4 } else { 0.12 }),
                    along * px + across * size_h.dot(across) * H_WORLD_UNITS,
                ),
                Anchor::BOTTOM_LEFT,
                Transform::from_translation(at.extend(GRID_Z)),
            ));
            // The corner would carry both axes' zero: the vertical one has it.
            if labelled && (line > 0 || along == Vec2::X) {
                commands.spawn((
                    Text2d::new(line.to_string()),
                    font.clone(),
                    TextColor(GRID_COLOR),
                    Anchor::BOTTOM_LEFT,
                    Transform::from_translation((at + Vec2::splat(2.0 * px)).extend(GRID_Z)),
                ));
            }
        }
    }
}

/// What the picture waits on: the room's images, on its sprites and on the
/// layers of its backdrop.
#[derive(SystemParam)]
struct RoomImages<'w, 's> {
    images: Res<'w, Assets<Image>>,
    sprites: Query<'w, 's, &'static Sprite>,
    backdrops: Res<'w, Assets<CityLightsMaterial>>,
}

/// Frames the whole room every frame, over what the game's own camera systems
/// just did.
fn frame_the_room(
    mut commands: Commands,
    data: Res<GameData>,
    session: Res<RunSession>,
    mut camera: Query<&mut Transform, With<Camera2d>>,
    mut backgrounds: Query<(&mut Transform, &BackgroundParallax), Without<Camera2d>>,
    entry_doors: Query<Entity, With<EntryDoor>>,
) {
    let room = &data.rooms[&session.room_id];
    let size = Vec2::new(room.size.width_h, room.size.height_h) * H_WORLD_UNITS;
    if let Ok(mut transform) = camera.single_mut() {
        transform.translation.x = size.x / 2.0;
        transform.translation.y = size.y / 2.0;
    }
    // The backdrop is as tall as the player's view and follows it; here it is
    // laid flat over the room, enlarged until it covers the room's height.
    for (mut transform, background) in &mut backgrounds {
        let enlarged = (size.y / background.tile.y).max(1.0);
        transform.translation.x = background.base_x * enlarged;
        transform.translation.y = size.y / 2.0;
        transform.scale = (background.tile * enlarged).extend(1.0);
    }
    // The door the player walks in through is gone by the time they can act.
    for door in &entry_doors {
        commands.entity(door).despawn();
    }
}

/// Takes the picture once everything the room shows has loaded, and quits.
fn take_the_shot(
    mut commands: Commands,
    session: Res<RunSession>,
    mut shot: ResMut<RoomShot>,
    room_images: RoomImages,
    mut exit: MessageWriter<AppExit>,
) {
    if shot.requested {
        return;
    }
    shot.waited_frames += 1;
    let RoomImages {
        images,
        sprites,
        backdrops,
    } = room_images;
    let loaded = session.world_loaded
        && sprites
            .iter()
            .all(|sprite| images.contains(sprite.image.id()))
        && backdrops
            .iter()
            .all(|(_, backdrop)| images.contains(backdrop.layer.id()));
    if !loaded {
        shot.settled_frames = 0;
        if shot.waited_frames > GIVE_UP_FRAMES {
            error!("Room {}: its images never loaded", session.room_id);
            exit.write(AppExit::error());
        }
        return;
    }
    shot.settled_frames += 1;
    if shot.settled_frames < SETTLE_FRAMES {
        return;
    }

    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/room-shots");
    if let Err(error) = fs::create_dir_all(&directory) {
        error!("Could not create {}: {error}", directory.display());
        exit.write(AppExit::error());
        return;
    }
    let path = directory.join(format!(
        "room_{:03}{}.png",
        session.room_id,
        if shot.grid { "_grid" } else { "" }
    ));
    commands
        .spawn(Screenshot::image(shot.target.clone()))
        .observe(
            move |captured: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
                save_to_disk(&path)(captured);
                // The one line of output: where the picture is.
                println!("{}", path.display());
                exit.write(AppExit::Success);
            },
        );
    shot.requested = true;
}

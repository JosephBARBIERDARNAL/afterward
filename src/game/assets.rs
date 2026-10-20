use super::*;

mod boss_sheets;
mod enemy_sheets;
mod player_sheets;

pub(super) fn load_sprites(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let specs = player_sheets::sprites()
        .into_iter()
        .chain(enemy_sheets::sprites())
        .chain(boss_sheets::sprites())
        .collect::<Vec<_>>();

    let mut sheets = BTreeMap::new();
    for spec in specs {
        let image = asset_server.load(spec.path);
        let layout = atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::new(spec.frame_width, spec.frame_height),
            spec.frames as u32,
            1,
            None,
            None,
        ));
        sheets.insert(
            spec.key.to_string(),
            SpriteSheet {
                image,
                layout,
                frame_size: Vec2::new(spec.frame_width as f32, spec.frame_height as f32),
                frames: spec.frames,
                fps: spec.fps,
            },
        );
    }
    commands.insert_resource(SpriteSheets(sheets));
}

pub(super) fn animate_sprites(
    time: Res<Time>,
    mut query: Query<(&mut Sprite, &mut AnimatedSprite)>,
) {
    for (mut sprite, mut animation) in &mut query {
        animation.timer.tick(time.delta());
        if !animation.timer.just_finished() {
            continue;
        }
        let Some(atlas) = sprite.texture_atlas.as_mut() else {
            continue;
        };
        if animation.frame + 1 == animation.frames {
            if animation.looping {
                animation.frame = 0;
            }
        } else {
            animation.frame += 1;
        }
        atlas.index = animation.frame;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::Path};

    #[test]
    fn sprite_sheet_layouts_match_their_png_dimensions() {
        let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
        let specs = player_sheets::sprites()
            .into_iter()
            .chain(enemy_sheets::sprites())
            .chain(boss_sheets::sprites());

        for spec in specs {
            let path = asset_root.join(spec.path);
            let bytes = fs::read(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            assert!(
                bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
                "{} is not a PNG",
                path.display()
            );
            assert!(
                bytes.len() >= 24,
                "{} has a truncated PNG header",
                path.display()
            );
            assert_eq!(
                &bytes[12..16],
                b"IHDR",
                "{} has no IHDR chunk",
                path.display()
            );

            let image_width = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
            let image_height = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
            assert_eq!(
                image_width,
                spec.frame_width * spec.frames as u32,
                "{} atlas width does not match its configured frame count",
                path.display()
            );
            assert_eq!(
                image_height,
                spec.frame_height,
                "{} atlas height does not match its configured frame height",
                path.display()
            );
        }
    }
}

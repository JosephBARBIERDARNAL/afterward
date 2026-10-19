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

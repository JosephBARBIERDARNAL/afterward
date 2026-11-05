//! Terrain art: solid blocks, one-way platforms, and decoration panels drawn
//! with their material's texture tiled in world space (so neighboring blocks
//! line up), a cap strip along uncovered tops, and an outline and highlight
//! along every edge no other block covers. Drawing only: collision lives in
//! `terrain.rs`.

use super::*;
use crate::data::{DecorationLayer, RectH, RoomConfig, TerrainBlockConfig};
use bevy::image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor};

/// Depth of the first solid block; later blocks stack slightly above it.
const TERRAIN_Z: f32 = 0.0;
/// Depth of the first one-way platform, above the solid terrain.
const PLATFORM_Z: f32 = 0.1;
/// Depth step between blocks of one kind, so overlaps draw in file order.
const BLOCK_Z_STEP: f32 = 0.0001;
/// Caps, highlights, and outlines sit this far above their block's fill.
const CAP_Z_OFFSET: f32 = 0.02;
const HIGHLIGHT_Z_OFFSET: f32 = 0.03;
const OUTLINE_Z_OFFSET: f32 = 0.04;
const BACKGROUND_DECORATION_Z: f32 = -10.0;
const FOREGROUND_DECORATION_Z: f32 = 6.0;

/// A side of a block, for finding the parts of its edges left uncovered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Side {
    Top,
    Bottom,
    Left,
    Right,
}

impl Side {
    /// The stretches of `rect`'s edge on this side that no rectangle in
    /// `others` touches from outside, as (start, end) along the edge. A
    /// rectangle sharing the edge or overlapping it covers it; `rect` itself
    /// may be among `others`.
    fn uncovered(self, rect: &RectH, others: &[RectH]) -> Vec<(f32, f32)> {
        const PROBE_H: f32 = 0.01;
        let (start, end) = match self {
            Self::Top | Self::Bottom => (rect.x, rect.right()),
            Self::Left | Self::Right => (rect.y, rect.top()),
        };
        let probe = match self {
            Self::Top => rect.top() + PROBE_H,
            Self::Bottom => rect.y - PROBE_H,
            Self::Left => rect.x - PROBE_H,
            Self::Right => rect.right() + PROBE_H,
        };
        let mut covered = others
            .iter()
            .filter_map(|other| {
                let (across, along) = match self {
                    Self::Top | Self::Bottom => ((other.y, other.top()), (other.x, other.right())),
                    Self::Left | Self::Right => ((other.x, other.right()), (other.y, other.top())),
                };
                let span = (along.0.max(start), along.1.min(end));
                (across.0 < probe && probe < across.1 && span.0 < span.1).then_some(span)
            })
            .collect::<Vec<_>>();
        covered.sort_by(|first, second| first.0.total_cmp(&second.0));
        let mut gaps = Vec::new();
        let mut reached = start;
        for (from, to) in covered {
            if from > reached {
                gaps.push((reached, from));
            }
            reached = reached.max(to);
        }
        if reached < end {
            gaps.push((reached, end));
        }
        gaps
    }

    /// The strip `thickness` H deep inside `rect` along this side, limited
    /// to the stretch `from..to` of the edge.
    fn strip(self, rect: &RectH, (from, to): (f32, f32), thickness: f32) -> RectH {
        let thickness = thickness.min(rect.width).min(rect.height);
        match self {
            Self::Top => RectH {
                x: from,
                y: rect.top() - thickness,
                width: to - from,
                height: thickness,
            },
            Self::Bottom => RectH {
                x: from,
                y: rect.y,
                width: to - from,
                height: thickness,
            },
            Self::Left => RectH {
                x: rect.x,
                y: from,
                width: thickness,
                height: to - from,
            },
            Self::Right => RectH {
                x: rect.right() - thickness,
                y: from,
                width: thickness,
                height: to - from,
            },
        }
    }
}

/// Loads a terrain texture to repeat across whatever it covers, crisp when
/// scaled up.
pub(super) fn load_texture(asset_server: &AssetServer, path: &str) -> Handle<Image> {
    asset_server
        .load_builder()
        .with_settings(|settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                address_mode_u: ImageAddressMode::Repeat,
                address_mode_v: ImageAddressMode::Repeat,
                ..ImageSamplerDescriptor::nearest()
            });
        })
        .load(path.to_string())
}

/// Draws the room's blocks, platforms, and decoration panels.
pub(super) fn draw_terrain(
    commands: &mut Commands,
    room: &RoomConfig,
    data: &GameData,
    asset_server: &AssetServer,
) {
    let terrain = &data.terrain;
    let px_per_h = terrain.texture_px as f32 / terrain.texture_tile_h;
    let texel_h = terrain.texel_h();
    let texture = |path: &str| load_texture(asset_server, path);
    // A sprite of `image` sampled at `rect`'s world position, so every block
    // of one material shares one continuous pattern.
    let tiled = |image: Handle<Image>, rect: RectH, color: Color| Sprite {
        rect: Some(Rect::new(
            rect.x * px_per_h,
            -rect.top() * px_per_h,
            rect.right() * px_per_h,
            -rect.y * px_per_h,
        )),
        color,
        ..Sprite::from_image(image)
    };
    // `sprite` stretched over `rect`, at depth `z`.
    let placed = |mut sprite: Sprite, rect: RectH, z: f32| {
        sprite.custom_size = Some(Vec2::new(rect.width, rect.height) * H_WORLD_UNITS);
        let center = Vec2::new(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
        (
            sprite,
            Transform::from_translation((center * H_WORLD_UNITS).extend(z)),
            RoomEntity,
        )
    };

    let occluders = room
        .terrain
        .iter()
        .chain(&room.platforms)
        .map(|block| block.rect_h)
        .collect::<Vec<_>>();
    let edges = &terrain.edges;
    let outline = Color::from(edges.outline_color).with_alpha(edges.outline_alpha);
    let highlight = Color::from(edges.highlight_color).with_alpha(edges.highlight_alpha);
    let blocks = room
        .terrain
        .iter()
        .enumerate()
        .map(|(index, block)| (block, TERRAIN_Z + index as f32 * BLOCK_Z_STEP))
        .chain(
            room.platforms
                .iter()
                .enumerate()
                .map(|(index, block)| (block, PLATFORM_Z + index as f32 * BLOCK_Z_STEP)),
        );
    for (
        TerrainBlockConfig {
            material, rect_h, ..
        },
        z,
    ) in blocks
    {
        let Some(material) = terrain.materials.get(material) else {
            continue;
        };
        let rect = *rect_h;
        commands.spawn(placed(
            tiled(texture(&material.texture), rect, Color::WHITE),
            rect,
            z,
        ));
        let open_top = Side::Top.uncovered(&rect, &occluders);
        if let Some(cap) = &material.top {
            let image = texture(&cap.texture);
            for span in &open_top {
                let strip = Side::Top.strip(&rect, *span, cap.depth_px * texel_h);
                commands.spawn(placed(
                    tiled(image.clone(), strip, Color::WHITE),
                    strip,
                    z + CAP_Z_OFFSET,
                ));
            }
        }
        let below_outline = RectH {
            height: rect.height - edges.outline_px * texel_h,
            ..rect
        };
        for span in &open_top {
            let strip = Side::Top.strip(&below_outline, *span, edges.highlight_px * texel_h);
            commands.spawn(placed(
                Sprite::from_color(highlight, Vec2::ONE),
                strip,
                z + HIGHLIGHT_Z_OFFSET,
            ));
        }
        for side in [Side::Top, Side::Bottom, Side::Left, Side::Right] {
            for span in side.uncovered(&rect, &occluders) {
                let strip = side.strip(&rect, span, edges.outline_px * texel_h);
                commands.spawn(placed(
                    Sprite::from_color(outline, Vec2::ONE),
                    strip,
                    z + OUTLINE_Z_OFFSET,
                ));
            }
        }
    }

    let layers = &terrain.decoration_layers;
    for (index, decoration) in room.decorations.iter().enumerate() {
        let Some(material) = terrain.materials.get(&decoration.material) else {
            continue;
        };
        let (style, base_z) = match decoration.layer {
            DecorationLayer::Background => (&layers.background, BACKGROUND_DECORATION_Z),
            DecorationLayer::Foreground => (&layers.foreground, FOREGROUND_DECORATION_Z),
        };
        let z = base_z + index as f32 * BLOCK_Z_STEP;
        let color = Color::from(style.tint).with_alpha(style.alpha);
        let rect = decoration.rect_h;
        commands.spawn(placed(
            tiled(texture(&material.texture), rect, color),
            rect,
            z,
        ));
        if let Some(cap) = &material.top {
            let strip = Side::Top.strip(&rect, (rect.x, rect.right()), cap.depth_px * texel_h);
            commands.spawn(placed(
                tiled(texture(&cap.texture), strip, color),
                strip,
                z + CAP_Z_OFFSET,
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edges_are_outlined_only_where_no_neighbor_touches_them() {
        let rect = |x, y, width, height| RectH {
            x,
            y,
            width,
            height,
        };
        let floor = rect(0.0, 0.0, 10.0, 1.0);
        let step = rect(4.0, 1.0, 2.0, 1.0);
        let wall = rect(10.0, 0.0, 1.0, 5.0);
        let others = [floor, step, wall];

        assert_eq!(
            Side::Top.uncovered(&floor, &others),
            vec![(0.0, 4.0), (6.0, 10.0)],
            "the step covers the middle of the floor's top"
        );
        assert_eq!(Side::Right.uncovered(&floor, &others), vec![]);
        assert_eq!(Side::Bottom.uncovered(&step, &others), vec![]);
        assert_eq!(Side::Top.uncovered(&step, &others), vec![(4.0, 6.0)]);
        assert_eq!(
            Side::Left.uncovered(&wall, &others),
            vec![(1.0, 5.0)],
            "the floor covers the wall's left side up to its own top"
        );
    }
}

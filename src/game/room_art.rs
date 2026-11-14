//! Room art: platforms and decoration panels, painted with their
//! material's texture tiled in world space (so neighboring blocks line up) or
//! with a plain color, and outlined and highlighted along every edge no other
//! platform covers; and the room's props, each one sprite at its own size.
//! Drawing only: collision lives in `collision.rs`.

use super::*;
use crate::data::{DecorationLayer, Fill, RectH, RoomConfig};
use bevy::image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor};

/// Depth of the first platform; later platforms stack slightly above it.
const PLATFORM_Z: f32 = 0.0;
/// Depth step between platforms, so overlaps draw in file order.
const BLOCK_Z_STEP: f32 = 0.0001;
/// Highlights and outlines sit this far above their platform's fill.
const HIGHLIGHT_Z_OFFSET: f32 = 0.03;
const OUTLINE_Z_OFFSET: f32 = 0.04;
const BACKGROUND_DECORATION_Z: f32 = -10.0;
const FOREGROUND_DECORATION_Z: f32 = 6.0;
/// Background props stand over the platforms and behind the bodies;
/// foreground ones over the foreground decorations.
const BACKGROUND_PROP_Z: f32 = 1.0;
const FOREGROUND_PROP_Z: f32 = 7.0;

/// A side of a platform, for finding the parts of its edges left uncovered.
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

/// Loads a surface texture to repeat across whatever it covers, crisp when
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

/// Draws the room's platforms, decoration panels and props.
pub(super) fn draw(
    commands: &mut Commands,
    room: &RoomConfig,
    data: &GameData,
    asset_server: &AssetServer,
) {
    let surfaces = &data.surfaces;
    let texel_h = surfaces.texel_h();
    // A sprite painted with `fill`: the material's texture sampled at the
    // rectangle's world position, so every block of one material shares one
    // continuous pattern, or a plain color. `tint` multiplies both.
    let painted = |fill: &Fill, rect: RectH, tint: Color| match fill {
        Fill::Material(material) => match surfaces.materials.get(material) {
            Some(material) => {
                let tile_h = material.tile_h.unwrap_or(surfaces.texture_tile_h);
                let px_per_h = surfaces.texture_px as f32 / tile_h;
                Sprite {
                    rect: Some(Rect::new(
                        rect.x * px_per_h,
                        -rect.top() * px_per_h,
                        rect.right() * px_per_h,
                        -rect.y * px_per_h,
                    )),
                    color: tint,
                    ..Sprite::from_image(load_texture(asset_server, &material.texture))
                }
            }
            None => Sprite::from_color(tint, Vec2::ONE),
        },
        Fill::Color(color) => {
            let (base, tint) = (Srgba::from(Color::from(*color)), Srgba::from(tint));
            let tinted = Srgba {
                red: base.red * tint.red,
                green: base.green * tint.green,
                blue: base.blue * tint.blue,
                alpha: tint.alpha,
            };
            Sprite::from_color(tinted, Vec2::ONE)
        }
    };
    // `sprite` stretched over `rect`, at depth `z`, turned by `turn` radians
    // about `pivot`: the middle of the platform it belongs to.
    let placed = |mut sprite: Sprite, rect: RectH, z: f32, pivot: Vec2, turn: f32| {
        sprite.custom_size = Some(Vec2::new(rect.width, rect.height) * H_WORLD_UNITS);
        let center = Vec2::new(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
        let center = pivot + Vec2::from_angle(turn).rotate(center - pivot);
        (
            sprite,
            Transform::from_translation((center * H_WORLD_UNITS).extend(z))
                .with_rotation(Quat::from_rotation_z(turn)),
            RoomEntity,
        )
    };

    // Only platforms standing square cover a neighbor's edge: a turned one
    // lies elsewhere than its `rect_h`, and keeps all four of its own edges.
    let occluders = room
        .platforms
        .iter()
        .filter(|block| block.rotation_deg == 0.0)
        .map(|block| block.rect_h)
        .collect::<Vec<_>>();
    let edges = &surfaces.edges;
    let outline = Color::from(edges.outline_color).with_alpha(edges.outline_alpha);
    let highlight = Color::from(edges.highlight_color).with_alpha(edges.highlight_alpha);
    for (index, block) in room.platforms.iter().enumerate() {
        let z = PLATFORM_Z + index as f32 * BLOCK_Z_STEP;
        let rect = block.rect_h;
        let pivot = Vec2::new(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
        let turn = block.rotation_deg.to_radians();
        let uncovered = |side: Side| {
            if block.rotation_deg == 0.0 {
                side.uncovered(&rect, &occluders)
            } else {
                side.uncovered(&rect, &[])
            }
        };
        commands.spawn(placed(
            painted(&block.fill, rect, Color::WHITE),
            rect,
            z,
            pivot,
            turn,
        ));
        let below_outline = RectH {
            height: rect.height - edges.outline_px * texel_h,
            ..rect
        };
        for span in uncovered(Side::Top) {
            let strip = Side::Top.strip(&below_outline, span, edges.highlight_px * texel_h);
            commands.spawn(placed(
                Sprite::from_color(highlight, Vec2::ONE),
                strip,
                z + HIGHLIGHT_Z_OFFSET,
                pivot,
                turn,
            ));
        }
        for side in [Side::Top, Side::Bottom, Side::Left, Side::Right] {
            for span in uncovered(side) {
                let strip = side.strip(&rect, span, edges.outline_px * texel_h);
                commands.spawn(placed(
                    Sprite::from_color(outline, Vec2::ONE),
                    strip,
                    z + OUTLINE_Z_OFFSET,
                    pivot,
                    turn,
                ));
            }
        }
    }

    let layers = &surfaces.decoration_layers;
    for (index, decoration) in room.decorations.iter().enumerate() {
        let (style, base_z) = match decoration.layer {
            DecorationLayer::Background => (&layers.background, BACKGROUND_DECORATION_Z),
            DecorationLayer::Foreground => (&layers.foreground, FOREGROUND_DECORATION_Z),
        };
        let z = base_z + index as f32 * BLOCK_Z_STEP;
        let tint = Color::from(style.tint).with_alpha(style.alpha);
        let rect = decoration.rect_h;
        commands.spawn(placed(
            painted(&decoration.fill, rect, tint),
            rect,
            z,
            Vec2::ZERO,
            0.0,
        ));
    }

    for (index, placed) in room.props.iter().enumerate() {
        let (Some(sprite), Some(size_h)) = (
            data.props.props.get(&placed.prop),
            data.props.size_h(&placed.prop),
        ) else {
            continue;
        };
        let base_z = match placed.layer {
            DecorationLayer::Background => BACKGROUND_PROP_Z,
            DecorationLayer::Foreground => FOREGROUND_PROP_Z,
        };
        let middle = Vec2::new(
            placed.position_h.x_h,
            placed.position_h.y_h + size_h.y / 2.0,
        );
        commands.spawn((
            Sprite {
                custom_size: Some(size_h * H_WORLD_UNITS),
                ..Sprite::from_image(load_texture(asset_server, &sprite.sprite))
            },
            Transform::from_translation(
                (middle * H_WORLD_UNITS).extend(base_z + index as f32 * BLOCK_Z_STEP),
            ),
            RoomEntity,
        ));
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

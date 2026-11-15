//! The art redrawn in each colorway of `surfaces/colorways.yaml`: every image
//! a material, a prop or a device is drawn with, recolored once as the game
//! starts so that a room naming a colorway draws on its first frame.

use super::*;
use crate::data::ColorwayId;
use bevy::{
    asset::RenderAssetUsages,
    image::{
        CompressedImageFormats, ImageAddressMode, ImageSampler, ImageSamplerDescriptor, ImageType,
    },
    render::render_resource::TextureFormat,
};
use std::collections::HashMap;

/// Each image's copy in each colorway, by its path under the asset root.
#[derive(Resource, Default)]
pub(crate) struct ColorwayImages(HashMap<(String, ColorwayId), Handle<Image>>);

impl ColorwayImages {
    /// The image at `path` in `colorway`, else the image itself, repeating
    /// and crisp.
    pub(crate) fn texture(
        &self,
        asset_server: &AssetServer,
        path: &str,
        colorway: Option<&ColorwayId>,
    ) -> Handle<Image> {
        colorway
            .and_then(|colorway| self.0.get(&(path.to_string(), colorway.clone())))
            .cloned()
            .unwrap_or_else(|| room_art::load_texture(asset_server, path))
    }
}

/// Reads every image a room's items are drawn with and adds its copy in
/// each colorway. An image that cannot be read is drawn in its own colors.
pub(super) fn paint_colorways(
    mut commands: Commands,
    data: Res<GameData>,
    mut images: ResMut<Assets<Image>>,
) {
    let paths = data
        .surfaces
        .materials
        .values()
        .map(|material| material.texture.clone())
        .chain(data.props.props.values().map(|prop| prop.sprite.clone()))
        .chain(
            data.devices
                .kinds
                .values()
                .flat_map(|kind| kind.sheets.values().map(|sheet| sheet.sheet.clone())),
        )
        .collect::<BTreeSet<_>>();
    let mut painted = HashMap::new();
    for path in paths {
        let decoded = std::fs::read(data.asset_root.join(&path))
            .map_err(|error| error.to_string())
            .and_then(|bytes| {
                Image::from_buffer(
                    &bytes,
                    ImageType::Extension("png"),
                    CompressedImageFormats::NONE,
                    true,
                    ImageSampler::Default,
                    RenderAssetUsages::default(),
                )
                .map_err(|error| error.to_string())
            })
            .and_then(|image| {
                image
                    .convert(TextureFormat::Rgba8UnormSrgb)
                    .ok_or_else(|| String::from("not convertible to RGBA"))
            })
            // Set once converted, which would drop it: repeating, as a
            // material tiles, and crisp.
            .map(|mut image| {
                image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                    address_mode_u: ImageAddressMode::Repeat,
                    address_mode_v: ImageAddressMode::Repeat,
                    ..ImageSamplerDescriptor::nearest()
                });
                image
            });
        let source = match decoded {
            Ok(source) => source,
            Err(error) => {
                warn!("{path} is drawn in its own colors in every colorway: {error}");
                continue;
            }
        };
        for (id, colorway) in &data.colorways.colorways {
            let mut image = source.clone();
            // The art is drawn in few colors, so each is worked out once.
            let mut recolored = HashMap::<[u8; 4], [u8; 4]>::new();
            if let Some(pixels) = image.data.as_mut() {
                for pixel in pixels.as_chunks_mut::<4>().0 {
                    let [red, green, blue, alpha] = *pixel;
                    if alpha == 0 {
                        continue;
                    }
                    let to = *recolored
                        .entry([red, green, blue, alpha])
                        .or_insert_with(|| {
                            colorway
                                .recolor(Srgba::rgba_u8(red, green, blue, alpha))
                                .to_u8_array()
                        });
                    *pixel = to;
                }
            }
            painted.insert((path.clone(), id.clone()), images.add(image));
        }
    }
    commands.insert_resource(ColorwayImages(painted));
}

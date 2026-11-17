//! The player hidden from the enemies (`combat/stealth.rs`), drawn as liquid
//! glass: while they are hidden, a quad hung over them draws the frame their
//! sprite is on as a clear body edged in light, rippling, with a glint
//! sweeping across it, and the sprite itself fades out under it. As the
//! hiding runs out, the sprite comes back through the glass.

use super::*;
use bevy::sprite::Anchor;

pub(super) const SHADER_PATH: &str = "shaders/glass.wgsl";

/// The glass quad over the hidden player.
#[derive(Component)]
pub(in crate::game) struct Glass;

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub(in crate::game) struct GlassMaterial {
    #[uniform(0)]
    settings: GlassSettings,
    /// The sheet the player's sprite is drawn from, read texel by texel.
    #[texture(1)]
    sheet: Handle<Image>,
}

#[derive(Clone, ShaderType)]
struct GlassSettings {
    /// The frame drawn, as texel corners in its sheet: min x, min y, max x,
    /// max y.
    frame: Vec4,
    /// The colour the body is filled with, and its rim's.
    fill: Vec4,
    rim: Vec4,
    /// x: drawn mirrored (1) or not (0); y: how much of the glass shows,
    /// from 0 to 1; z: how far a row ripples sideways, in texels.
    look: Vec4,
}

impl Material2d for GlassMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

/// Hangs the glass over the hidden player, keeps it on the frame their sprite
/// shows, fades the sprite under it, and takes it down once they are seen
/// again.
pub(in crate::game) fn dress_the_hidden_player(
    mut commands: Commands,
    data: Res<GameData>,
    mut assets: ShaderAssets,
    layouts: Res<Assets<TextureAtlasLayout>>,
    mut players: Query<(Entity, &PlayerBody, &mut Sprite, &Anchor)>,
    mut glasses: Query<(Entity, &MeshMaterial2d<GlassMaterial>, &mut Transform), With<Glass>>,
) {
    let look = &data.gameplay.effects.glass;
    let Ok((player, body, mut sprite, anchor)) = players.single_mut() else {
        for (entity, _, _) in &glasses {
            commands.entity(entity).despawn();
        }
        return;
    };
    let showing = (body.invisible_s / look.fade_out_s.max(f32::EPSILON)).clamp(0.0, 1.0);
    if showing == 0.0 {
        for (entity, _, _) in &glasses {
            commands.entity(entity).despawn();
            sprite.color = sprite.color.with_alpha(1.0);
        }
        return;
    }
    sprite.color = sprite.color.with_alpha(1.0 - showing);
    let (Some(atlas), Some(size)) = (sprite.texture_atlas.as_ref(), sprite.custom_size) else {
        return;
    };
    let Some(frame) = layouts
        .get(&atlas.layout)
        .and_then(|layout| layout.textures.get(atlas.index))
    else {
        return;
    };
    let settings = GlassSettings {
        frame: Vec4::new(
            frame.min.x as f32,
            frame.min.y as f32,
            frame.max.x as f32,
            frame.max.y as f32,
        ),
        fill: LinearRgba::from(Color::from(look.fill_color).with_alpha(look.fill_opacity))
            .to_vec4(),
        rim: LinearRgba::from(Color::from(look.rim_color)).to_vec4(),
        look: Vec4::new(
            f32::from(u8::from(sprite.flip_x)),
            showing,
            look.ripple_px,
            0.0,
        ),
    };
    // Where the sprite's middle stands from its anchor, which is the body's
    // middle rather than the frame's.
    let place = Transform::from_xyz(0.0, -anchor.0.y * size.y, 0.01).with_scale(size.extend(1.0));
    if let Some((_, material, mut transform)) = glasses.iter_mut().next() {
        *transform = place;
        if let Some(mut glass) = assets.glasses.get_mut(&material.0) {
            glass.settings = settings;
            glass.sheet = sprite.image.clone();
        }
        return;
    }
    let material = assets.glasses.add(GlassMaterial {
        settings,
        sheet: sprite.image.clone(),
    });
    commands.entity(player).with_child((
        Mesh2d(assets.quad.0.clone()),
        MeshMaterial2d(material),
        place,
        Glass,
    ));
}

//! What the player wears while a bonus holds them in a state: a glow behind
//! the body and, where the VFX packs have one, a loop over it (a rune ring
//! while the next blow is empowered, sparks while Chain Reaction burns; only
//! a glow for Adrenaline), and with Deep Breath the window to chain the next
//! blow, as a bar under the feet emptying.

use super::{afterimages::category_color, *};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AuraKind {
    /// Riposte or Retaliation has made the next blow heavier.
    Empowered,
    ChainReaction,
    Adrenaline,
}

/// The glow hung on the player for the state they are in.
#[derive(Component)]
pub(crate) struct PlayerAura(AuraKind);

impl AuraKind {
    /// The loop worn over the body, if any: its sheet in `effects/vfx/` and
    /// how tall it is drawn, in H.
    fn worn(self) -> Option<(&'static str, f32)> {
        match self {
            Self::Empowered => Some(("vfx_arcane_charge", 1.6)),
            Self::ChainReaction => Some(("vfx_shock_sparks", 1.3)),
            Self::Adrenaline => None,
        }
    }
}

pub(crate) fn wear_the_aura(
    mut commands: Commands,
    data: Res<GameData>,
    session: Res<RunSession>,
    sheets: Res<SpriteSheets>,
    players: Query<(Entity, &PlayerBody)>,
    auras: Query<(Entity, &PlayerAura)>,
) {
    let Ok((entity, player)) = players.single() else {
        return;
    };
    let adrenaline = session.has_bonus("adrenaline")
        && session.health / session.max_health
            < data.bonus_effect("adrenaline", "low_health_fraction");
    // The state that matters most for the next blow wins.
    let wanted = if player.empowered.is_some() {
        Some((
            AuraKind::Empowered,
            Color::from(data.gameplay.effects.tells.boosted_color),
        ))
    } else if player.chain_reaction_timer > 0.0 && session.has_bonus("chain_reaction") {
        Some((AuraKind::ChainReaction, category_color(&data, "combat")))
    } else if adrenaline {
        Some((AuraKind::Adrenaline, Color::srgb(0.9, 0.16, 0.22)))
    } else {
        None
    };
    let worn = auras.iter().next();
    if worn.map(|(_, aura)| aura.0) == wanted.map(|(kind, _)| kind) {
        return;
    }
    if let Some((aura, _)) = worn {
        commands.entity(aura).despawn();
    }
    let Some((kind, color)) = wanted else {
        return;
    };
    let mut aura = commands.spawn((
        PlayerAura(kind),
        Glowing {
            color,
            radius_h: data.gameplay.effects.tells.aura_radius_h,
            brightness: 1.0,
        },
        Transform::default(),
        Visibility::default(),
        ChildOf(entity),
    ));
    if let Some((key, size_h)) = kind.worn()
        && let Some(sheet) = sheets.0.get(key)
    {
        let mut sprite = Sprite::from_atlas_image(
            sheet.image.clone(),
            TextureAtlas {
                layout: sheet.layout.clone(),
                index: 0,
            },
        );
        sprite.custom_size = Some(sheet.frame_size * size_h * H_WORLD_UNITS / sheet.frame_size.y);
        // Over the body, as the sparks crawl on it and the ring circles it.
        aura.with_child((
            sprite,
            Transform::from_xyz(0.0, 0.0, 0.2),
            AnimatedSprite {
                sheet: key.to_owned(),
                frame: 0,
                frames: sheet.frames,
                timer: Timer::from_seconds(1.0 / sheet.fps, TimerMode::Repeating),
                looping: true,
            },
        ));
    }
}

/// The bar under the feet that Deep Breath gives the window to chain.
#[derive(Component)]
pub(crate) struct ComboWindowBar;

const COMBO_BAR_WIDTH: f32 = 36.0;

pub(crate) fn show_the_combo_window(
    mut commands: Commands,
    data: Res<GameData>,
    session: Res<RunSession>,
    players: Query<(&Transform, &PlayerBody), Without<ComboWindowBar>>,
    mut bars: Query<(&mut Transform, &mut Sprite, &mut Visibility), With<ComboWindowBar>>,
) {
    let Ok((transform, player)) = players.single() else {
        return;
    };
    let full = data.gameplay.player.melee.combo_window_s
        * session.bonus_multiplier(&data, "deep_breath", "combo_window_multiplier");
    let open = session.has_bonus("deep_breath")
        && player.combo_window_s > 0.0
        && player.combo_step > 0
        && full > 0.0;
    let Ok((mut bar_transform, mut sprite, mut visibility)) = bars.single_mut() else {
        if open {
            commands.spawn((
                Sprite::from_color(category_color(&data, "intelligence"), Vec2::ZERO),
                Transform::default(),
                Visibility::Hidden,
                ComboWindowBar,
                RoomEntity,
            ));
        }
        return;
    };
    *visibility = if open {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    let fraction = (player.combo_window_s / full).clamp(0.0, 1.0);
    sprite.custom_size = Some(Vec2::new(COMBO_BAR_WIDTH * fraction, 3.0));
    bar_transform.translation = Vec3::new(
        transform.translation.x - COMBO_BAR_WIDTH * (1.0 - fraction) / 2.0,
        transform.translation.y - 0.5 * H_WORLD_UNITS - 6.0,
        5.0,
    );
}

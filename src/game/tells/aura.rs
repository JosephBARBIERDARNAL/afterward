//! What the player wears while a bonus holds them in a state: a glow behind
//! the body (the next blow empowered, Chain Reaction burning, Adrenaline in),
//! and with Deep Breath the window to chain the next blow, as a bar under the
//! feet emptying.

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

pub(crate) fn wear_the_aura(
    mut commands: Commands,
    data: Res<GameData>,
    session: Res<RunSession>,
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
    if let Some((kind, color)) = wanted {
        commands.entity(entity).with_child((
            PlayerAura(kind),
            Glowing {
                color,
                radius_h: data.gameplay.effects.tells.aura_radius_h,
            },
            Transform::default(),
            Visibility::default(),
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

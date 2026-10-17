use bevy::prelude::*;

#[derive(Resource)]
pub(crate) struct Game {
    pub(crate) scene: Scene,
    pub(crate) room: usize,
    pub(crate) player_health: f32,
    pub(crate) max_health: f32,
    pub(crate) facing: Vec2,
    pub(crate) primary_cooldown: f32,
    pub(crate) heavy_windup: f32,
    pub(crate) heavy_recovery: f32,
    pub(crate) heavy_pending: bool,
    pub(crate) dash_time: f32,
    pub(crate) dash_cooldown: f32,
    pub(crate) invulnerable: f32,
    pub(crate) room_clear: bool,
    pub(crate) overlay: Overlay,
    pub(crate) bonuses: Vec<Bonus>,
    pub(crate) bonus_damage: f32,
    pub(crate) bonus_range: f32,
    pub(crate) combo_index: usize,
    pub(crate) combo_timer: f32,
    pub(crate) cache_claimed: bool,
    pub(crate) bonus_seed: u64,
    pub(crate) last_scene_key: Option<(Scene, usize)>,
    pub(crate) last_overlay_key: String,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            scene: Scene::Home,
            room: 0,
            player_health: 100.0,
            max_health: 100.0,
            facing: Vec2::X,
            primary_cooldown: 0.0,
            heavy_windup: 0.0,
            heavy_recovery: 0.0,
            heavy_pending: false,
            dash_time: 0.0,
            dash_cooldown: 0.0,
            invulnerable: 0.0,
            room_clear: false,
            overlay: Overlay::None,
            bonuses: Vec::new(),
            bonus_damage: 0.0,
            bonus_range: 0.0,
            combo_index: 0,
            combo_timer: 0.0,
            cache_claimed: false,
            bonus_seed: 0,
            last_scene_key: None,
            last_overlay_key: String::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Scene {
    Home,
    Run,
}

#[derive(Clone, Debug)]
pub(crate) enum Overlay {
    None,
    Pause(usize),
    Catalog,
    BonusChoice {
        selected: usize,
        source: BonusSource,
    },
    Ended(bool),
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum BonusSource {
    Door,
    Cache,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Bonus {
    Vitality,
    Edge,
    Mend,
    DashNova,
    QuickStep,
    Reach,
    WideSwing,
    CinderCombo,
    ComboBlast,
    Siphon,
}

impl Bonus {
    pub(crate) const ALL: [Self; 10] = [
        Self::Vitality,
        Self::Edge,
        Self::Mend,
        Self::DashNova,
        Self::QuickStep,
        Self::Reach,
        Self::WideSwing,
        Self::CinderCombo,
        Self::ComboBlast,
        Self::Siphon,
    ];

    pub(crate) const fn title(self) -> &'static str {
        match self {
            Self::Vitality => "VITALITY  +20 MAX LIFE",
            Self::Edge => "SHARPENED  +3 BASE DAMAGE",
            Self::Mend => "MENDING  RESTORE 35 LIFE",
            Self::DashNova => "DASH NOVA  DASH ENDS IN A BURST",
            Self::QuickStep => "QUICK STEP  FASTER DASH RECHARGE",
            Self::Reach => "LONG REACH  WIDER ATTACKS",
            Self::WideSwing => "WIDE SWING  HEAVY ATTACK COVERS MORE",
            Self::CinderCombo => "CINDER COMBO  FINAL HIT BURNS",
            Self::ComboBlast => "COMBO BLAST  FINAL HIT BURSTS",
            Self::Siphon => "SIPHON  ATTACKS RESTORE LIFE",
        }
    }
}

#[derive(Component)]
pub(crate) struct SceneEntity;

#[derive(Component)]
pub(crate) struct Player;

#[derive(Component)]
pub(crate) struct Enemy {
    pub(crate) kind: EnemyKind,
    pub(crate) elite: Option<EliteKind>,
    pub(crate) health: f32,
    pub(crate) cooldown: f32,
    pub(crate) windup: f32,
    pub(crate) target: Vec2,
    pub(crate) charge_time: f32,
    pub(crate) charge_direction: Vec2,
    pub(crate) stun: f32,
    pub(crate) orbit_side: f32,
    pub(crate) telegraph: Option<Entity>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum EnemyKind {
    Melee,
    Ranged,
    Heavy,
    Charger,
    Skirmisher,
    Bomber,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum EliteKind {
    Vanguard,
    Volley,
}

pub(crate) struct EnemySpawn {
    pub(crate) kind: EnemyKind,
    pub(crate) position: Vec2,
    pub(crate) elite: Option<EliteKind>,
}

pub(crate) fn enemy_spawn(kind: EnemyKind, x: f32, y: f32) -> EnemySpawn {
    EnemySpawn {
        kind,
        position: Vec2::new(x, y),
        elite: None,
    }
}

pub(crate) fn elite_spawn(kind: EnemyKind, x: f32, y: f32, elite: EliteKind) -> EnemySpawn {
    EnemySpawn {
        kind,
        position: Vec2::new(x, y),
        elite: Some(elite),
    }
}

impl EnemyKind {
    pub(crate) const fn health(self) -> f32 {
        match self {
            Self::Melee => 74.0,
            Self::Ranged => 60.0,
            Self::Heavy => 96.0,
            Self::Charger => 82.0,
            Self::Skirmisher => 58.0,
            Self::Bomber => 76.0,
        }
    }

    pub(crate) const fn speed(self) -> f32 {
        match self {
            Self::Melee => 91.0,
            Self::Ranged => 78.0,
            Self::Heavy => 58.0,
            Self::Charger => 78.0,
            Self::Skirmisher => 132.0,
            Self::Bomber => 54.0,
        }
    }

    pub(crate) const fn damage(self) -> f32 {
        match self {
            Self::Melee => 17.0,
            Self::Ranged => 13.0,
            Self::Heavy => 23.0,
            Self::Charger => 21.0,
            Self::Skirmisher => 13.0,
            Self::Bomber => 20.0,
        }
    }

    pub(crate) const fn attack_range(self) -> f32 {
        match self {
            Self::Melee => 82.0,
            Self::Ranged => 218.0,
            Self::Heavy => 102.0,
            Self::Charger => 270.0,
            Self::Skirmisher => 84.0,
            Self::Bomber => 174.0,
        }
    }

    pub(crate) const fn attack_radius(self) -> f32 {
        match self {
            Self::Melee => 48.0,
            Self::Ranged => 12.0,
            Self::Heavy => 59.0,
            Self::Charger => 31.0,
            Self::Skirmisher => 32.0,
            Self::Bomber => 68.0,
        }
    }

    pub(crate) const fn attack_windup(self) -> f32 {
        match self {
            Self::Melee => 0.58,
            Self::Ranged => 0.64,
            Self::Heavy => 0.82,
            Self::Charger => 0.72,
            Self::Skirmisher => 0.34,
            Self::Bomber => 0.90,
        }
    }

    pub(crate) const fn attack_cooldown(self) -> f32 {
        match self {
            Self::Melee => 1.12,
            Self::Ranged => 1.75,
            Self::Heavy => 1.48,
            Self::Charger => 1.75,
            Self::Skirmisher => 1.10,
            Self::Bomber => 2.05,
        }
    }
}

#[derive(Component)]
pub(crate) struct Door;

#[derive(Component)]
pub(crate) struct BonusCache;

#[derive(Component)]
pub(crate) struct Projectile {
    pub(crate) velocity: Vec2,
    pub(crate) lifetime: f32,
    pub(crate) damage: f32,
}

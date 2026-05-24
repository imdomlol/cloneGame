// Sources: vault/units/ranger.md, vault/buildings/soldiers_center.md, vault/units/soldier.md

use bevy::prelude::*;
use fixed::types::I32F32;

pub const GOLD_COST: u32 = 120;
pub const WOOD_COST: u32 = 2;
pub const WORKER_COST: u32 = 1;
pub const BUILD_TIME_TICKS: u32 = 540; // 27 s × 20 Hz
pub const MAINTENANCE_GOLD: u32 = 1;

pub const BASE_HP: u32 = 60;
pub const BASE_MOVEMENT_SPEED: I32F32 = I32F32::lit("4");
pub const BASE_ATTACK_RANGE: I32F32 = I32F32::lit("6");
pub const BASE_ATTACK_SPEED: I32F32 = I32F32::lit("1");
pub const BASE_ATTACK_DAMAGE: u32 = 10;
pub const WATCH_RANGE: I32F32 = I32F32::lit("8");
pub const NOISE_PER_ATTACK: u32 = 1;
pub const EXP_REWARD: u32 = 60;

// 5% armor reduction: pass-through = 95 / 100
pub const ARMOR_NUMERATOR: u32 = 95;
pub const ARMOR_DENOMINATOR: u32 = 100;

pub const VET_ATTACK_RANGE: I32F32 = I32F32::lit("6.5");
pub const VET_ATTACK_SPEED: I32F32 = I32F32::lit("2");
pub const VET_ATTACK_DAMAGE: u32 = 12;

const SIM_HZ: u32 = 20;

#[derive(Component, Default)]
pub struct Ranger;

#[derive(Component, Clone, Copy)]
pub struct RangerHealth {
    pub current: u32,
    pub max: u32,
}

impl Default for RangerHealth {
    fn default() -> Self {
        Self { current: BASE_HP, max: BASE_HP }
    }
}

#[derive(Component, Clone, Copy)]
pub struct RangerStats {
    pub movement_speed: I32F32,
    pub attack_range: I32F32,
    pub attack_speed: I32F32,
    pub attack_damage: u32,
    pub watch_range: I32F32,
}

impl Default for RangerStats {
    fn default() -> Self {
        Self {
            movement_speed: BASE_MOVEMENT_SPEED,
            attack_range: BASE_ATTACK_RANGE,
            attack_speed: BASE_ATTACK_SPEED,
            attack_damage: BASE_ATTACK_DAMAGE,
            watch_range: WATCH_RANGE,
        }
    }
}

#[derive(Component, Default)]
pub struct RangerAttackCooldown {
    pub remaining_ticks: u32,
}

#[derive(Component)]
pub struct RangerNoise {
    pub per_attack: u32,
}

impl Default for RangerNoise {
    fn default() -> Self {
        Self { per_attack: NOISE_PER_ATTACK }
    }
}

#[derive(Component, Default)]
pub struct RangerVeteran {
    pub promoted: bool,
    pub tile_noise: u32,
}

#[derive(Bundle, Default)]
pub struct RangerBundle {
    pub ranger: Ranger,
    pub health: RangerHealth,
    pub stats: RangerStats,
    pub cooldown: RangerAttackCooldown,
    pub noise: RangerNoise,
    pub veteran: RangerVeteran,
}

#[derive(Event)]
pub struct RangerFiredEvent {
    pub ranger: Entity,
    pub noise: u32,
}

#[derive(Event)]
pub struct DamageRangerEvent {
    pub target: Entity,
    pub raw_damage: u32,
}

#[derive(Event)]
pub struct PromoteRangerEvent {
    pub entity: Entity,
}

#[derive(Event)]
pub struct DismissRangerEvent {
    pub entity: Entity,
}

pub fn ranger_attack_tick(
    mut rangers: Query<(Entity, &RangerStats, &mut RangerAttackCooldown), With<Ranger>>,
    mut fired: EventWriter<RangerFiredEvent>,
) {
    for (entity, stats, mut cd) in &mut rangers {
        if cd.remaining_ticks > 0 {
            cd.remaining_ticks -= 1;
            continue;
        }
        let reload: u32 = (I32F32::from_num(SIM_HZ) / stats.attack_speed).to_num();
        cd.remaining_ticks = reload;
        fired.send(RangerFiredEvent { ranger: entity, noise: NOISE_PER_ATTACK });
    }
}

pub fn ranger_receive_damage(
    mut events: EventReader<DamageRangerEvent>,
    mut rangers: Query<&mut RangerHealth, With<Ranger>>,
) {
    for ev in events.read() {
        if let Ok(mut hp) = rangers.get_mut(ev.target) {
            let reduced = ev.raw_damage.saturating_mul(ARMOR_NUMERATOR) / ARMOR_DENOMINATOR;
            hp.current = hp.current.saturating_sub(reduced);
        }
    }
}

pub fn ranger_apply_promotion(
    mut events: EventReader<PromoteRangerEvent>,
    mut rangers: Query<(&mut RangerStats, &mut RangerVeteran), With<Ranger>>,
) {
    for ev in events.read() {
        if let Ok((mut stats, mut vet)) = rangers.get_mut(ev.entity) {
            if vet.promoted {
                continue;
            }
            stats.attack_range = VET_ATTACK_RANGE;
            stats.attack_speed = VET_ATTACK_SPEED;
            stats.attack_damage = VET_ATTACK_DAMAGE;
            vet.promoted = true;
        }
    }
}

pub fn veteran_tile_noise_tick(
    mut rangers: Query<&mut RangerVeteran, With<Ranger>>,
) {
    for mut vet in &mut rangers {
        if vet.promoted {
            vet.tile_noise = vet.tile_noise.saturating_add(1);
        }
    }
}

pub fn ranger_handle_dismiss(
    mut events: EventReader<DismissRangerEvent>,
    rangers: Query<Entity, With<Ranger>>,
    mut commands: Commands,
) {
    for ev in events.read() {
        if rangers.get(ev.entity).is_ok() {
            commands.entity(ev.entity).despawn();
        }
    }
}

pub struct RangerPlugin;

impl Plugin for RangerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<RangerFiredEvent>()
            .add_event::<DamageRangerEvent>()
            .add_event::<PromoteRangerEvent>()
            .add_event::<DismissRangerEvent>()
            .add_systems(
                FixedUpdate,
                (
                    ranger_attack_tick,
                    ranger_receive_damage,
                    ranger_apply_promotion,
                    veteran_tile_noise_tick,
                    ranger_handle_dismiss,
                )
                    .chain(),
            );
    }
}
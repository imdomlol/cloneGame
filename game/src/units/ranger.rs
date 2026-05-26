// Sources: vault/units/ranger.md, vault/buildings/soldiers_center.md, vault/units/soldier.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, IncomingDamageEvent, NoiseEmittedEvent, SimChecksumState, SimHz, SimPosition, UnitStats};

pub const RANGER_GOLD_COST: u32 = 120;
pub const RANGER_WOOD_COST: u32 = 2;
pub const RANGER_WORKER_COST: u32 = 1;
pub const RANGER_BUILD_TIME_SECONDS: u32 = 27;
pub const RANGER_MAINTENANCE_GOLD: u32 = 1;

pub const RANGER_HP: I32F32 = I32F32::lit("60");
pub const RANGER_MOVE_SPEED: I32F32 = I32F32::lit("4");
pub const RANGER_ATTACK_RANGE: I32F32 = I32F32::lit("6");
pub const RANGER_ATTACK_SPEED: I32F32 = I32F32::lit("1");
pub const RANGER_ATTACK_DAMAGE: I32F32 = I32F32::lit("10");
pub const RANGER_WATCH_RANGE: I32F32 = I32F32::lit("8");
pub const RANGER_NOISE_PER_ATTACK: I32F32 = I32F32::lit("1");

pub const RANGER_ARMOR_REDUCTION_PERCENT: I32F32 = I32F32::lit("0.05");

pub const RANGER_VETERAN_ATTACK_RANGE: I32F32 = I32F32::lit("6.5");
pub const RANGER_VETERAN_ATTACK_SPEED: I32F32 = I32F32::lit("2");
pub const RANGER_VETERAN_ATTACK_DAMAGE: I32F32 = I32F32::lit("12");

#[derive(Component, Default)]
pub struct Ranger;

#[derive(Component, Clone, Copy)]
pub struct RangerArmor {
    pub standard_reduction: I32F32,
}

impl Default for RangerArmor {
    fn default() -> Self {
        Self {
            standard_reduction: RANGER_ARMOR_REDUCTION_PERCENT,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct RangerAttackCooldown {
    pub ticks_remaining: u32,
}

#[derive(Component, Clone, Copy, Default)]
pub struct RangerVeteran {
    pub is_veteran: bool,
}

#[derive(Component, Clone, Copy)]
pub struct RangerVeteranTileNoise {
    pub accumulated: I32F32,
}

impl Default for RangerVeteranTileNoise {
    fn default() -> Self {
        Self {
            accumulated: I32F32::ZERO,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct RangerProductionCost {
    pub gold: u32,
    pub wood: u32,
    pub workers: u32,
    pub build_time_seconds: u32,
    pub maintenance_gold: u32,
}

impl Default for RangerProductionCost {
    fn default() -> Self {
        Self {
            gold: RANGER_GOLD_COST,
            wood: RANGER_WOOD_COST,
            workers: RANGER_WORKER_COST,
            build_time_seconds: RANGER_BUILD_TIME_SECONDS,
            maintenance_gold: RANGER_MAINTENANCE_GOLD,
        }
    }
}

#[derive(Bundle, Default)]
pub struct RangerBundle {
    pub ranger: Ranger,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub armor: RangerArmor,
    pub cooldown: RangerAttackCooldown,
    pub veteran: RangerVeteran,
    pub veteran_tile_noise: RangerVeteranTileNoise,
    pub production_cost: RangerProductionCost,
}

impl Default for UnitStats {
    fn default() -> Self {
        Self {
            move_speed: RANGER_MOVE_SPEED,
            attack_range: RANGER_ATTACK_RANGE,
            attack_damage: RANGER_ATTACK_DAMAGE,
            attack_speed: RANGER_ATTACK_SPEED,
            watch_range: RANGER_WATCH_RANGE,
        }
    }
}

impl Default for SimPosition {
    fn default() -> Self {
        Self {
            x: I32F32::ZERO,
            y: I32F32::ZERO,
        }
    }
}

impl Default for Health {
    fn default() -> Self {
        Health::full(RANGER_HP)
    }
}

#[derive(Event, Clone, Copy)]
pub struct RangerAttackEvent {
    pub ranger: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct PromoteRangerEvent {
    pub ranger: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct DismissRangerEvent {
    pub ranger: Entity,
}

pub fn spawn_ranger(commands: &mut Commands, position: SimPosition) -> Entity {
    commands
        .spawn(RangerBundle {
            position,
            ..Default::default()
        })
        .id()
}

fn ranger_attack_system(
    sim_hz: Res<SimHz>,
    mut rangers: Query<(Entity, &SimPosition, &UnitStats, &mut RangerAttackCooldown), With<Ranger>>,
    mut attack_events: EventWriter<RangerAttackEvent>,
    mut noise_events: EventWriter<NoiseEmittedEvent>,
) {
    for (entity, position, stats, mut cooldown) in &mut rangers {
        if cooldown.ticks_remaining > 0 {
            cooldown.ticks_remaining -= 1;
            continue;
        }

        attack_events.send(RangerAttackEvent { ranger: entity });
        noise_events.send(NoiseEmittedEvent {
            source: entity,
            position: *position,
            amount: RANGER_NOISE_PER_ATTACK,
        });

        let mut ticks = (sim_hz.0 / stats.attack_speed).to_num::<u32>();
        if ticks == 0 {
            ticks = 1;
        }
        cooldown.ticks_remaining = ticks;
    }
}

fn ranger_apply_damage_system(
    mut incoming_damage: EventReader<IncomingDamageEvent>,
    mut rangers: Query<(&mut Health, &RangerArmor), With<Ranger>>,
) {
    for event in incoming_damage.read() {
        if let Ok((mut health, armor)) = rangers.get_mut(event.target) {
            let reduced_multiplier = I32F32::ONE - armor.standard_reduction;
            let effective_damage = event.raw_amount * reduced_multiplier;
            health.current = (health.current - effective_damage).max(I32F32::ZERO);
        }
    }
}

fn ranger_promotion_system(
    mut promotions: EventReader<PromoteRangerEvent>,
    mut rangers: Query<(&mut UnitStats, &mut RangerVeteran), With<Ranger>>,
) {
    for event in promotions.read() {
        if let Ok((mut stats, mut veteran)) = rangers.get_mut(event.ranger) {
            if veteran.is_veteran {
                continue;
            }
            veteran.is_veteran = true;
            stats.attack_range = RANGER_VETERAN_ATTACK_RANGE;
            stats.attack_speed = RANGER_VETERAN_ATTACK_SPEED;
            stats.attack_damage = RANGER_VETERAN_ATTACK_DAMAGE;
        }
    }
}

fn ranger_veteran_tile_noise_system(
    mut rangers: Query<(&RangerVeteran, &mut RangerVeteranTileNoise), With<Ranger>>,
) {
    for (veteran, mut tile_noise) in &mut rangers {
        if veteran.is_veteran {
            tile_noise.accumulated += RANGER_NOISE_PER_ATTACK;
        }
    }
}

fn ranger_dismiss_system(
    mut dismissals: EventReader<DismissRangerEvent>,
    rangers: Query<Entity, With<Ranger>>,
    mut commands: Commands,
) {
    for event in dismissals.read() {
        if rangers.get(event.ranger).is_ok() {
            commands.entity(event.ranger).despawn();
        }
    }
}

fn ranger_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    rangers: Query<
        (
            &Health,
            &UnitStats,
            &RangerArmor,
            &RangerAttackCooldown,
            &RangerVeteran,
            &RangerVeteranTileNoise,
            &RangerProductionCost,
            &SimPosition,
        ),
        With<Ranger>,
    >,
) {
    for (health, stats, armor, cooldown, veteran, tile_noise, production_cost, position) in &rangers {
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);
        checksum.accumulate(armor.standard_reduction.to_bits() as u64);
        checksum.accumulate(cooldown.ticks_remaining as u64);
        checksum.accumulate(veteran.is_veteran as u64);
        checksum.accumulate(tile_noise.accumulated.to_bits() as u64);
        checksum.accumulate(production_cost.gold as u64);
        checksum.accumulate(production_cost.wood as u64);
        checksum.accumulate(production_cost.workers as u64);
        checksum.accumulate(production_cost.build_time_seconds as u64);
        checksum.accumulate(production_cost.maintenance_gold as u64);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
    }
}

pub struct RangerPlugin;

impl Plugin for RangerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<RangerAttackEvent>()
            .add_event::<PromoteRangerEvent>()
            .add_event::<DismissRangerEvent>()
            .add_systems(
                FixedUpdate,
                (
                    ranger_attack_system,
                    ranger_apply_damage_system,
                    ranger_promotion_system,
                    ranger_veteran_tile_noise_system,
                    ranger_dismiss_system,
                    ranger_checksum_system,
                )
                    .chain(),
            );
    }
}
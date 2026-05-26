// Sources: vault/units/lucifer.md, vault/buildings/engineering_center.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{
    DamageType, Health, IncomingDamageEvent, NoiseEmittedEvent, SimChecksumState, SimHz, SimPosition,
    UnitStats,
};
use crate::units::Infected;

const LUCIFER_HP: I32F32 = I32F32::lit("500");
const LUCIFER_MOVE_SPEED: I32F32 = I32F32::lit("1.8");
const LUCIFER_ATTACK_RANGE: I32F32 = I32F32::lit("3.5");
const LUCIFER_ATTACK_SPEED: I32F32 = I32F32::lit("2");
const LUCIFER_ATTACK_DAMAGE: I32F32 = I32F32::lit("24");
const LUCIFER_WATCH_RANGE: I32F32 = I32F32::lit("6");
const LUCIFER_BURN_DAMAGE: I32F32 = I32F32::lit("4");
const LUCIFER_NOISE_PER_ATTACK: I32F32 = I32F32::lit("10");
const LUCIFER_REGEN_HP_PER_SECOND: I32F32 = I32F32::lit("40");

const LUCIFER_PHYSICAL_DAMAGE_REDUCTION: I32F32 = I32F32::lit("0.25");
const LUCIFER_VENOM_RESISTANCE: I32F32 = I32F32::lit("0.65");
const LUCIFER_FIRE_RESISTANCE: I32F32 = I32F32::lit("1");

const LUCIFER_STARTUP_SECONDS: I32F32 = I32F32::lit("0.5");
const LUCIFER_CONE_COS_SQ: I32F32 = I32F32::lit("0.25");
const LUCIFER_DEATH_EXPLOSION_RADIUS: I32F32 = I32F32::lit("1.5");
const LUCIFER_DEATH_EXPLOSION_DAMAGE: I32F32 = I32F32::lit("24");

const LUCIFER_COST_GOLD: i32 = 600;
const LUCIFER_BUILD_TIME_SECONDS: i32 = 36;
const LUCIFER_COST_FOOD: i32 = 2;
const LUCIFER_COST_WORKER: i32 = 1;
const LUCIFER_COST_OIL: i32 = 15;
const LUCIFER_MAINTENANCE_GOLD: i32 = 12;
const LUCIFER_MAINTENANCE_OIL: i32 = 1;

#[derive(Component, Default)]
pub struct Lucifer;

#[derive(Component, Clone, Copy)]
pub struct LuciferAttackProfile {
    pub burn_damage: I32F32,
    pub startup_seconds: I32F32,
    pub cone_cos_sq: I32F32,
    pub noise_per_attack: I32F32,
}

impl Default for LuciferAttackProfile {
    fn default() -> Self {
        Self {
            burn_damage: LUCIFER_BURN_DAMAGE,
            startup_seconds: LUCIFER_STARTUP_SECONDS,
            cone_cos_sq: LUCIFER_CONE_COS_SQ,
            noise_per_attack: LUCIFER_NOISE_PER_ATTACK,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct LuciferResistances {
    pub physical_damage_reduction: I32F32,
    pub venom_resistance: I32F32,
    pub fire_resistance: I32F32,
}

impl Default for LuciferResistances {
    fn default() -> Self {
        Self {
            physical_damage_reduction: LUCIFER_PHYSICAL_DAMAGE_REDUCTION,
            venom_resistance: LUCIFER_VENOM_RESISTANCE,
            fire_resistance: LUCIFER_FIRE_RESISTANCE,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct LuciferRegeneration {
    pub hp_per_second: I32F32,
}

impl Default for LuciferRegeneration {
    fn default() -> Self {
        Self {
            hp_per_second: LUCIFER_REGEN_HP_PER_SECOND,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct LuciferAttackState {
    pub startup_ticks_remaining: I32F32,
    pub attack_cooldown_ticks: I32F32,
    pub stream_active: bool,
}

#[derive(Component, Clone, Copy)]
pub struct LuciferEconomy {
    pub cost_gold: i32,
    pub build_time_seconds: i32,
    pub cost_food: i32,
    pub cost_worker: i32,
    pub cost_oil: i32,
    pub maintenance_gold: i32,
    pub maintenance_oil: i32,
}

impl Default for LuciferEconomy {
    fn default() -> Self {
        Self {
            cost_gold: LUCIFER_COST_GOLD,
            build_time_seconds: LUCIFER_BUILD_TIME_SECONDS,
            cost_food: LUCIFER_COST_FOOD,
            cost_worker: LUCIFER_COST_WORKER,
            cost_oil: LUCIFER_COST_OIL,
            maintenance_gold: LUCIFER_MAINTENANCE_GOLD,
            maintenance_oil: LUCIFER_MAINTENANCE_OIL,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct LuciferRecruitmentGate {
    pub engineering_center_completed: bool,
    pub oil_income_non_negative: bool,
}

impl Default for LuciferRecruitmentGate {
    fn default() -> Self {
        Self {
            engineering_center_completed: false,
            oil_income_non_negative: true,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct LuciferBarrelState {
    pub carrying_barrel: bool,
}

impl Default for LuciferBarrelState {
    fn default() -> Self {
        Self {
            carrying_barrel: true,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WallMaterial {
    Wood,
    Stone,
    Trap,
}

#[derive(Event, Clone, Copy)]
pub struct SetLuciferRecruitmentStateEvent {
    pub target: Entity,
    pub engineering_center_completed: bool,
    pub oil_income_non_negative: bool,
}

#[derive(Event, Clone, Copy)]
pub struct LuciferManualWallFireEvent {
    pub lucifer: Entity,
    pub wall_entity: Entity,
    pub wall_material: WallMaterial,
}

#[derive(Event, Clone, Copy)]
pub struct LuciferKilledEvent {
    pub lucifer: Entity,
}

pub fn lucifer_base_health() -> Health {
    Health::full(LUCIFER_HP)
}

pub fn lucifer_base_stats() -> UnitStats {
    UnitStats {
        move_speed: LUCIFER_MOVE_SPEED,
        attack_range: LUCIFER_ATTACK_RANGE,
        attack_damage: LUCIFER_ATTACK_DAMAGE,
        attack_speed: LUCIFER_ATTACK_SPEED,
        watch_range: LUCIFER_WATCH_RANGE,
    }
}

fn choose_primary_target(
    lucifer_pos: SimPosition,
    stats: UnitStats,
    infected_positions: &Query<(Entity, &SimPosition), With<Infected>>,
) -> Option<SimPosition> {
    let mut best: Option<(SimPosition, I32F32)> = None;
    let range_sq = stats.attack_range * stats.attack_range;

    for (_, infected_pos) in infected_positions {
        let dx = infected_pos.x - lucifer_pos.x;
        let dy = infected_pos.y - lucifer_pos.y;
        let dist_sq = dx * dx + dy * dy;
        if dist_sq > range_sq {
            continue;
        }

        match best {
            None => best = Some((*infected_pos, dist_sq)),
            Some((best_pos, best_dist_sq)) => {
                if dist_sq < best_dist_sq
                    || (dist_sq == best_dist_sq
                        && (infected_pos.x < best_pos.x
                            || (infected_pos.x == best_pos.x && infected_pos.y < best_pos.y)))
                {
                    best = Some((*infected_pos, dist_sq));
                }
            }
        }
    }

    best.map(|(pos, _)| pos)
}

pub fn lucifer_attack_tick_system(
    sim_hz: Res<SimHz>,
    mut lucifers: Query<
        (
            Entity,
            &SimPosition,
            &UnitStats,
            &LuciferAttackProfile,
            &LuciferRecruitmentGate,
            &mut LuciferAttackState,
        ),
        With<Lucifer>,
    >,
    infected_positions: Query<(Entity, &SimPosition), With<Infected>>,
    mut outgoing_damage: EventWriter<IncomingDamageEvent>,
    mut noise_events: EventWriter<NoiseEmittedEvent>,
) {
    for (lucifer_entity, lucifer_pos, stats, attack_profile, recruitment_gate, mut attack_state) in &mut lucifers {
        if !recruitment_gate.engineering_center_completed || !recruitment_gate.oil_income_non_negative {
            continue;
        }

        let Some(primary_target_pos) = choose_primary_target(*lucifer_pos, *stats, &infected_positions) else {
            attack_state.startup_ticks_remaining = I32F32::ZERO;
            attack_state.attack_cooldown_ticks = I32F32::ZERO;
            attack_state.stream_active = false;
            continue;
        };

        if !attack_state.stream_active {
            if attack_state.startup_ticks_remaining <= I32F32::ZERO {
                attack_state.startup_ticks_remaining = sim_hz.0 * attack_profile.startup_seconds;
            }
            attack_state.startup_ticks_remaining -= I32F32::ONE;
            if attack_state.startup_ticks_remaining > I32F32::ZERO {
                continue;
            }
            attack_state.stream_active = true;
            attack_state.attack_cooldown_ticks = I32F32::ZERO;
        }

        if attack_state.attack_cooldown_ticks > I32F32::ZERO {
            attack_state.attack_cooldown_ticks -= I32F32::ONE;
            continue;
        }

        let axis_x = primary_target_pos.x - lucifer_pos.x;
        let axis_y = primary_target_pos.y - lucifer_pos.y;
        let axis_len_sq = axis_x * axis_x + axis_y * axis_y;
        if axis_len_sq <= I32F32::ZERO {
            continue;
        }

        let range_sq = stats.attack_range * stats.attack_range;
        for (infected_entity, infected_pos) in &infected_positions {
            let rel_x = infected_pos.x - lucifer_pos.x;
            let rel_y = infected_pos.y - lucifer_pos.y;
            let rel_len_sq = rel_x * rel_x + rel_y * rel_y;
            if rel_len_sq > range_sq {
                continue;
            }

            let dot = rel_x * axis_x + rel_y * axis_y;
            if dot <= I32F32::ZERO {
                continue;
            }

            let dot_sq = dot * dot;
            let rhs = rel_len_sq * axis_len_sq * attack_profile.cone_cos_sq;
            if dot_sq < rhs {
                continue;
            }

            outgoing_damage.send(IncomingDamageEvent {
                target: infected_entity,
                raw_amount: stats.attack_damage,
                damage_type: DamageType::Standard,
                source: lucifer_entity,
            });
            outgoing_damage.send(IncomingDamageEvent {
                target: infected_entity,
                raw_amount: attack_profile.burn_damage,
                damage_type: DamageType::Fire,
                source: lucifer_entity,
            });
        }

        noise_events.send(NoiseEmittedEvent {
            source: lucifer_entity,
            position: *lucifer_pos,
            amount: attack_profile.noise_per_attack,
        });

        attack_state.attack_cooldown_ticks = sim_hz.0 / stats.attack_speed;
    }
}

pub fn lucifer_receive_damage_system(
    mut damage_events: EventReader<IncomingDamageEvent>,
    mut lucifers: Query<(Entity, &mut Health, &LuciferResistances), With<Lucifer>>,
    mut killed_events: EventWriter<LuciferKilledEvent>,
) {
    for event in damage_events.read() {
        if let Ok((lucifer_entity, mut health, resistances)) = lucifers.get_mut(event.target) {
            let reduction = match event.damage_type {
                DamageType::Standard => resistances.physical_damage_reduction,
                DamageType::Fire => resistances.fire_resistance,
                DamageType::Venom => resistances.venom_resistance,
            };

            let applied_damage = event.raw_amount * (I32F32::ONE - reduction);
            health.current = (health.current - applied_damage).max(I32F32::ZERO);

            if health.current <= I32F32::ZERO {
                killed_events.send(LuciferKilledEvent {
                    lucifer: lucifer_entity,
                });
            }
        }
    }
}

pub fn lucifer_regeneration_system(
    sim_hz: Res<SimHz>,
    mut lucifers: Query<(&mut Health, &LuciferRegeneration), With<Lucifer>>,
) {
    for (mut health, regeneration) in &mut lucifers {
        if health.current <= I32F32::ZERO || health.current >= health.max {
            continue;
        }
        let regen_per_tick = regeneration.hp_per_second / sim_hz.0;
        health.current = (health.current + regen_per_tick).min(health.max);
    }
}

pub fn lucifer_recruitment_gate_system(
    mut events: EventReader<SetLuciferRecruitmentStateEvent>,
    mut lucifers: Query<&mut LuciferRecruitmentGate, With<Lucifer>>,
) {
    for event in events.read() {
        if let Ok(mut gate) = lucifers.get_mut(event.target) {
            gate.engineering_center_completed = event.engineering_center_completed;
            gate.oil_income_non_negative = event.oil_income_non_negative;
        }
    }
}

pub fn lucifer_death_explosion_system(
    mut killed_events: EventReader<LuciferKilledEvent>,
    lucifer_state: Query<(&SimPosition, &LuciferBarrelState), With<Lucifer>>,
    infected_positions: Query<(Entity, &SimPosition), With<Infected>>,
    mut outgoing_damage: EventWriter<IncomingDamageEvent>,
) {
    let radius_sq = LUCIFER_DEATH_EXPLOSION_RADIUS * LUCIFER_DEATH_EXPLOSION_RADIUS;

    for killed in killed_events.read() {
        let Ok((death_position, barrel_state)) = lucifer_state.get(killed.lucifer) else {
            continue;
        };
        if !barrel_state.carrying_barrel {
            continue;
        }

        for (infected_entity, infected_pos) in &infected_positions {
            let dx = infected_pos.x - death_position.x;
            let dy = infected_pos.y - death_position.y;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq <= radius_sq {
                outgoing_damage.send(IncomingDamageEvent {
                    target: infected_entity,
                    raw_amount: LUCIFER_DEATH_EXPLOSION_DAMAGE,
                    damage_type: DamageType::Fire,
                    source: killed.lucifer,
                });
            }
        }
    }
}

pub fn lucifer_manual_wall_fire_system(
    mut events: EventReader<LuciferManualWallFireEvent>,
    lucifers: Query<(&UnitStats, &LuciferAttackProfile), With<Lucifer>>,
    mut outgoing_damage: EventWriter<IncomingDamageEvent>,
) {
    for event in events.read() {
        let Ok((stats, attack_profile)) = lucifers.get(event.lucifer) else {
            continue;
        };

        let base_fire_damage = stats.attack_damage + attack_profile.burn_damage;
        let wall_damage = match event.wall_material {
            WallMaterial::Wood => base_fire_damage,
            WallMaterial::Stone => base_fire_damage / I32F32::from_num(2),
            WallMaterial::Trap => I32F32::ZERO,
        };

        if wall_damage > I32F32::ZERO {
            outgoing_damage.send(IncomingDamageEvent {
                target: event.wall_entity,
                raw_amount: wall_damage,
                damage_type: DamageType::Fire,
                source: event.lucifer,
            });
        }
    }
}

pub fn lucifer_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    lucifers: Query<
        (
            &Health,
            &UnitStats,
            &LuciferAttackProfile,
            &LuciferResistances,
            &LuciferRegeneration,
            &LuciferAttackState,
            &LuciferEconomy,
            &LuciferRecruitmentGate,
            &LuciferBarrelState,
        ),
        With<Lucifer>,
    >,
) {
    for (health, stats, attack_profile, resistances, regeneration, attack_state, economy, gate, barrel) in &lucifers {
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);

        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);

        checksum.accumulate(attack_profile.burn_damage.to_bits() as u64);
        checksum.accumulate(attack_profile.startup_seconds.to_bits() as u64);
        checksum.accumulate(attack_profile.cone_cos_sq.to_bits() as u64);
        checksum.accumulate(attack_profile.noise_per_attack.to_bits() as u64);

        checksum.accumulate(resistances.physical_damage_reduction.to_bits() as u64);
        checksum.accumulate(resistances.venom_resistance.to_bits() as u64);
        checksum.accumulate(resistances.fire_resistance.to_bits() as u64);

        checksum.accumulate(regeneration.hp_per_second.to_bits() as u64);

        checksum.accumulate(attack_state.startup_ticks_remaining.to_bits() as u64);
        checksum.accumulate(attack_state.attack_cooldown_ticks.to_bits() as u64);
        checksum.accumulate(u64::from(attack_state.stream_active));

        checksum.accumulate(economy.cost_gold as u64);
        checksum.accumulate(economy.build_time_seconds as u64);
        checksum.accumulate(economy.cost_food as u64);
        checksum.accumulate(economy.cost_worker as u64);
        checksum.accumulate(economy.cost_oil as u64);
        checksum.accumulate(economy.maintenance_gold as u64);
        checksum.accumulate(economy.maintenance_oil as u64);

        checksum.accumulate(u64::from(gate.engineering_center_completed));
        checksum.accumulate(u64::from(gate.oil_income_non_negative));

        checksum.accumulate(u64::from(barrel.carrying_barrel));
    }
}

pub struct LuciferPlugin;

impl Plugin for LuciferPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SetLuciferRecruitmentStateEvent>()
            .add_event::<LuciferManualWallFireEvent>()
            .add_event::<LuciferKilledEvent>()
            .add_systems(
                FixedUpdate,
                (
                    lucifer_attack_tick_system,
                    lucifer_receive_damage_system,
                    lucifer_regeneration_system,
                    lucifer_recruitment_gate_system,
                    lucifer_death_explosion_system,
                    lucifer_manual_wall_fire_system,
                    lucifer_checksum_system,
                )
                    .chain(),
            );
    }
}
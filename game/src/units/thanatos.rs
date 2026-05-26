// Sources: vault/units/thanatos.md, vault/buildings/engineering_center.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{
    DamageType, Health, IncomingDamageEvent, NoiseEmittedEvent, SimChecksumState, SimHz, SimPosition, UnitStats,
};
use crate::units::Infected;

const THANATOS_HP: I32F32 = I32F32::lit("250");
const THANATOS_MOVE_SPEED: I32F32 = I32F32::lit("1.8");
const THANATOS_ATTACK_RANGE: I32F32 = I32F32::lit("10");
const THANATOS_ATTACK_SPEED: I32F32 = I32F32::lit("0.33");
const THANATOS_ATTACK_DAMAGE: I32F32 = I32F32::lit("70");
const THANATOS_WATCH_RANGE: I32F32 = I32F32::lit("12");

const THANATOS_ARMOR_REDUCTION: I32F32 = I32F32::lit("0.25");
const THANATOS_MIN_RANGE: I32F32 = I32F32::lit("3");
const THANATOS_RANGED_AOE_RADIUS: I32F32 = I32F32::lit("1.8");
const THANATOS_RANGED_NOISE: I32F32 = I32F32::lit("500");

const THANATOS_MELEE_RANGE: I32F32 = I32F32::lit("1.5");
const THANATOS_MELEE_SPEED: I32F32 = I32F32::lit("1");
const THANATOS_MELEE_DAMAGE: I32F32 = I32F32::lit("20");
const THANATOS_MELEE_NOISE: I32F32 = I32F32::lit("3");

const THANATOS_DIRECT_IMPACT_DAMAGE: I32F32 = I32F32::lit("93");
const THANATOS_DIRECT_DIAGONAL_DAMAGE: I32F32 = I32F32::lit("41");

const THANATOS_COST_GOLD: u32 = 600;
const THANATOS_COST_FOOD: u32 = 2;
const THANATOS_COST_WORKER: u32 = 1;
const THANATOS_COST_IRON: u32 = 10;
const THANATOS_COST_OIL: u32 = 20;
const THANATOS_MAINTENANCE_GOLD: u32 = 16;
const THANATOS_MAINTENANCE_OIL: u32 = 1;
const THANATOS_BUILD_TIME_SECONDS: u32 = 36;
const THANATOS_RESEARCH_COST_GOLD: u32 = 2000;

#[derive(Component, Default)]
pub struct Thanatos;

#[derive(Component, Clone, Copy)]
pub struct ThanatosAttackProfile {
    pub armor_reduction: I32F32,
    pub min_range: I32F32,
    pub ranged_aoe_radius: I32F32,
    pub melee_range: I32F32,
    pub melee_speed: I32F32,
    pub melee_damage: I32F32,
    pub ranged_noise: I32F32,
    pub melee_noise: I32F32,
    pub melee_cone_cos_sq: I32F32,
}

impl Default for ThanatosAttackProfile {
    fn default() -> Self {
        Self {
            armor_reduction: THANATOS_ARMOR_REDUCTION,
            min_range: THANATOS_MIN_RANGE,
            ranged_aoe_radius: THANATOS_RANGED_AOE_RADIUS,
            melee_range: THANATOS_MELEE_RANGE,
            melee_speed: THANATOS_MELEE_SPEED,
            melee_damage: THANATOS_MELEE_DAMAGE,
            ranged_noise: THANATOS_RANGED_NOISE,
            melee_noise: THANATOS_MELEE_NOISE,
            melee_cone_cos_sq: I32F32::lit("0.25"),
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct ThanatosAttackCooldown {
    pub ticks_remaining: u32,
}

#[derive(Component, Clone, Copy)]
pub struct ThanatosRecruitmentGate {
    pub engineering_center_completed: bool,
    pub researched: bool,
    pub oil_income_non_negative: bool,
}

impl Default for ThanatosRecruitmentGate {
    fn default() -> Self {
        Self {
            engineering_center_completed: false,
            researched: false,
            oil_income_non_negative: true,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct ThanatosHoldMode {
    pub enabled: bool,
}

#[derive(Component, Clone, Copy, Default)]
pub struct ThanatosRetreatState {
    pub retreating_for_min_range: bool,
}

#[derive(Component, Clone, Copy)]
pub struct ThanatosEconomy {
    pub cost_gold: u32,
    pub cost_food: u32,
    pub cost_worker: u32,
    pub cost_iron: u32,
    pub cost_oil: u32,
    pub maintenance_gold: u32,
    pub maintenance_oil: u32,
    pub build_time_seconds: u32,
    pub research_cost_gold: u32,
}

impl Default for ThanatosEconomy {
    fn default() -> Self {
        Self {
            cost_gold: THANATOS_COST_GOLD,
            cost_food: THANATOS_COST_FOOD,
            cost_worker: THANATOS_COST_WORKER,
            cost_iron: THANATOS_COST_IRON,
            cost_oil: THANATOS_COST_OIL,
            maintenance_gold: THANATOS_MAINTENANCE_GOLD,
            maintenance_oil: THANATOS_MAINTENANCE_OIL,
            build_time_seconds: THANATOS_BUILD_TIME_SECONDS,
            research_cost_gold: THANATOS_RESEARCH_COST_GOLD,
        }
    }
}

#[derive(Event, Clone, Copy)]
pub struct SetThanatosRecruitmentStateEvent {
    pub target: Entity,
    pub engineering_center_completed: bool,
    pub researched: bool,
    pub oil_income_non_negative: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetThanatosHoldModeEvent {
    pub target: Entity,
    pub enabled: bool,
}

pub fn thanatos_base_health() -> Health {
    Health::full(THANATOS_HP)
}

pub fn thanatos_base_stats() -> UnitStats {
    UnitStats {
        move_speed: THANATOS_MOVE_SPEED,
        attack_range: THANATOS_ATTACK_RANGE,
        attack_damage: THANATOS_ATTACK_DAMAGE,
        attack_speed: THANATOS_ATTACK_SPEED,
        watch_range: THANATOS_WATCH_RANGE,
    }
}

pub fn spawn_thanatos(commands: &mut Commands, position: SimPosition) -> Entity {
    commands
        .spawn((
            Thanatos,
            position,
            thanatos_base_health(),
            thanatos_base_stats(),
            ThanatosAttackProfile::default(),
            ThanatosAttackCooldown::default(),
            ThanatosRecruitmentGate::default(),
            ThanatosHoldMode::default(),
            ThanatosRetreatState::default(),
            ThanatosEconomy::default(),
        ))
        .id()
}

fn choose_nearest_target(
    thanatos_pos: SimPosition,
    infected_positions: &Query<(Entity, &SimPosition), With<Infected>>,
) -> Option<(Entity, SimPosition, I32F32)> {
    let mut nearest: Option<(Entity, SimPosition, I32F32)> = None;

    for (infected_entity, infected_pos) in infected_positions {
        let dx = infected_pos.x - thanatos_pos.x;
        let dy = infected_pos.y - thanatos_pos.y;
        let dist_sq = dx * dx + dy * dy;

        match nearest {
            None => nearest = Some((infected_entity, *infected_pos, dist_sq)),
            Some((_, best_pos, best_dist_sq)) => {
                if dist_sq < best_dist_sq
                    || (dist_sq == best_dist_sq
                        && (infected_pos.x < best_pos.x
                            || (infected_pos.x == best_pos.x && infected_pos.y < best_pos.y)))
                {
                    nearest = Some((infected_entity, *infected_pos, dist_sq));
                }
            }
        }
    }

    nearest
}

fn is_direct_impact(dx: I32F32, dy: I32F32) -> bool {
    dx == I32F32::ZERO && dy == I32F32::ZERO
}

fn is_direct_diagonal_adjacent(dx: I32F32, dy: I32F32) -> bool {
    let one = I32F32::ONE;
    let neg_one = -I32F32::ONE;
    (dx == one || dx == neg_one) && (dy == one || dy == neg_one)
}

pub fn thanatos_attack_tick_system(
    sim_hz: Res<SimHz>,
    mut thanatos_units: Query<
        (
            Entity,
            &SimPosition,
            &UnitStats,
            &ThanatosAttackProfile,
            &ThanatosRecruitmentGate,
            &ThanatosHoldMode,
            &mut ThanatosRetreatState,
            &mut ThanatosAttackCooldown,
        ),
        With<Thanatos>,
    >,
    infected_positions: Query<(Entity, &SimPosition), With<Infected>>,
    mut outgoing_damage: EventWriter<IncomingDamageEvent>,
    mut noise_events: EventWriter<NoiseEmittedEvent>,
) {
    for (thanatos_entity, thanatos_pos, stats, profile, gate, hold_mode, mut retreat_state, mut cooldown) in
        &mut thanatos_units
    {
        if !gate.engineering_center_completed || !gate.researched || !gate.oil_income_non_negative {
            continue;
        }

        if cooldown.ticks_remaining > 0 {
            cooldown.ticks_remaining -= 1;
            continue;
        }

        let Some((_, primary_pos, nearest_dist_sq)) = choose_nearest_target(*thanatos_pos, &infected_positions) else {
            retreat_state.retreating_for_min_range = false;
            continue;
        };

        let min_range_sq = profile.min_range * profile.min_range;
        let melee_range_sq = profile.melee_range * profile.melee_range;
        let ranged_range_sq = stats.attack_range * stats.attack_range;

        if nearest_dist_sq < min_range_sq {
            retreat_state.retreating_for_min_range = true;
            if hold_mode.enabled || nearest_dist_sq > melee_range_sq {
                continue;
            }

            let axis_x = primary_pos.x - thanatos_pos.x;
            let axis_y = primary_pos.y - thanatos_pos.y;
            let axis_len_sq = axis_x * axis_x + axis_y * axis_y;
            if axis_len_sq <= I32F32::ZERO {
                continue;
            }

            for (infected_entity, infected_pos) in &infected_positions {
                let rel_x = infected_pos.x - thanatos_pos.x;
                let rel_y = infected_pos.y - thanatos_pos.y;
                let rel_len_sq = rel_x * rel_x + rel_y * rel_y;
                if rel_len_sq > melee_range_sq {
                    continue;
                }

                let dot = rel_x * axis_x + rel_y * axis_y;
                if dot <= I32F32::ZERO {
                    continue;
                }

                let dot_sq = dot * dot;
                let rhs = rel_len_sq * axis_len_sq * profile.melee_cone_cos_sq;
                if dot_sq < rhs {
                    continue;
                }

                outgoing_damage.send(IncomingDamageEvent {
                    target: infected_entity,
                    raw_amount: profile.melee_damage,
                    damage_type: DamageType::Standard,
                    source: thanatos_entity,
                });
            }

            noise_events.send(NoiseEmittedEvent {
                source: thanatos_entity,
                position: *thanatos_pos,
                amount: profile.melee_noise,
            });

            let mut ticks = (sim_hz.0 / profile.melee_speed).to_num::<u32>();
            if ticks == 0 {
                ticks = 1;
            }
            cooldown.ticks_remaining = ticks;
            continue;
        }

        retreat_state.retreating_for_min_range = false;

        if nearest_dist_sq > ranged_range_sq {
            continue;
        }

        let aoe_sq = profile.ranged_aoe_radius * profile.ranged_aoe_radius;
        for (infected_entity, infected_pos) in &infected_positions {
            let dx = infected_pos.x - primary_pos.x;
            let dy = infected_pos.y - primary_pos.y;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq > aoe_sq {
                continue;
            }

            let raw_damage = if is_direct_impact(dx, dy) {
                THANATOS_DIRECT_IMPACT_DAMAGE
            } else if is_direct_diagonal_adjacent(dx, dy) {
                THANATOS_DIRECT_DIAGONAL_DAMAGE
            } else {
                stats.attack_damage
            };

            outgoing_damage.send(IncomingDamageEvent {
                target: infected_entity,
                raw_amount: raw_damage,
                damage_type: DamageType::Standard,
                source: thanatos_entity,
            });
        }

        noise_events.send(NoiseEmittedEvent {
            source: thanatos_entity,
            position: *thanatos_pos,
            amount: profile.ranged_noise,
        });

        let mut ticks = (sim_hz.0 / stats.attack_speed).to_num::<u32>();
        if ticks == 0 {
            ticks = 1;
        }
        cooldown.ticks_remaining = ticks;
    }
}

pub fn thanatos_receive_damage_system(
    mut damage_events: EventReader<IncomingDamageEvent>,
    mut thanatos_units: Query<(&mut Health, &ThanatosAttackProfile), With<Thanatos>>,
) {
    for event in damage_events.read() {
        if let Ok((mut health, profile)) = thanatos_units.get_mut(event.target) {
            let applied = event.raw_amount * (I32F32::ONE - profile.armor_reduction);
            health.current = (health.current - applied).max(I32F32::ZERO);
        }
    }
}

pub fn thanatos_recruitment_gate_system(
    mut events: EventReader<SetThanatosRecruitmentStateEvent>,
    mut thanatos_units: Query<&mut ThanatosRecruitmentGate, With<Thanatos>>,
) {
    for event in events.read() {
        if let Ok(mut gate) = thanatos_units.get_mut(event.target) {
            gate.engineering_center_completed = event.engineering_center_completed;
            gate.researched = event.researched;
            gate.oil_income_non_negative = event.oil_income_non_negative;
        }
    }
}

pub fn thanatos_hold_mode_system(
    mut events: EventReader<SetThanatosHoldModeEvent>,
    mut thanatos_units: Query<&mut ThanatosHoldMode, With<Thanatos>>,
) {
    for event in events.read() {
        if let Ok(mut hold_mode) = thanatos_units.get_mut(event.target) {
            hold_mode.enabled = event.enabled;
        }
    }
}

pub fn thanatos_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    thanatos_units: Query<
        (
            &Health,
            &UnitStats,
            &SimPosition,
            &ThanatosAttackProfile,
            &ThanatosAttackCooldown,
            &ThanatosRecruitmentGate,
            &ThanatosHoldMode,
            &ThanatosRetreatState,
            &ThanatosEconomy,
        ),
        With<Thanatos>,
    >,
) {
    for (health, stats, pos, profile, cooldown, gate, hold_mode, retreat, economy) in &thanatos_units {
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);

        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);

        checksum.accumulate(pos.x.to_bits() as u64);
        checksum.accumulate(pos.y.to_bits() as u64);

        checksum.accumulate(profile.armor_reduction.to_bits() as u64);
        checksum.accumulate(profile.min_range.to_bits() as u64);
        checksum.accumulate(profile.ranged_aoe_radius.to_bits() as u64);
        checksum.accumulate(profile.melee_range.to_bits() as u64);
        checksum.accumulate(profile.melee_speed.to_bits() as u64);
        checksum.accumulate(profile.melee_damage.to_bits() as u64);
        checksum.accumulate(profile.ranged_noise.to_bits() as u64);
        checksum.accumulate(profile.melee_noise.to_bits() as u64);
        checksum.accumulate(profile.melee_cone_cos_sq.to_bits() as u64);

        checksum.accumulate(cooldown.ticks_remaining as u64);

        checksum.accumulate(u64::from(gate.engineering_center_completed));
        checksum.accumulate(u64::from(gate.researched));
        checksum.accumulate(u64::from(gate.oil_income_non_negative));

        checksum.accumulate(u64::from(hold_mode.enabled));
        checksum.accumulate(u64::from(retreat.retreating_for_min_range));

        checksum.accumulate(economy.cost_gold as u64);
        checksum.accumulate(economy.cost_food as u64);
        checksum.accumulate(economy.cost_worker as u64);
        checksum.accumulate(economy.cost_iron as u64);
        checksum.accumulate(economy.cost_oil as u64);
        checksum.accumulate(economy.maintenance_gold as u64);
        checksum.accumulate(economy.maintenance_oil as u64);
        checksum.accumulate(economy.build_time_seconds as u64);
        checksum.accumulate(economy.research_cost_gold as u64);
    }
}

pub struct ThanatosPlugin;

impl Plugin for ThanatosPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SetThanatosRecruitmentStateEvent>()
            .add_event::<SetThanatosHoldModeEvent>()
            .add_systems(
                FixedUpdate,
                (
                    thanatos_attack_tick_system,
                    thanatos_receive_damage_system,
                    thanatos_recruitment_gate_system,
                    thanatos_hold_mode_system,
                    thanatos_checksum_system,
                )
                    .chain(),
            );
    }
}
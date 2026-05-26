// Sources: vault/units/mutant.md, vault/buildings/engineering_center.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{
    DamageType, Health, IncomingDamageEvent, SimChecksumState, SimHz, SimPosition, UnitStats,
};
use crate::units::Infected;

pub const MUTANT_HP: I32F32 = I32F32::lit("2000");
pub const MUTANT_MOVE_SPEED: I32F32 = I32F32::lit("6");
pub const MUTANT_ATTACK_RANGE: I32F32 = I32F32::lit("2");
pub const MUTANT_ATTACK_SPEED: I32F32 = I32F32::lit("2");
pub const MUTANT_ATTACK_DAMAGE: I32F32 = I32F32::lit("30");
pub const MUTANT_WATCH_RANGE: I32F32 = I32F32::lit("12");

pub const MUTANT_ARMOR_STANDARD: I32F32 = I32F32::lit("0.25");
pub const MUTANT_ARMOR_FIRE: I32F32 = I32F32::lit("0.50");
pub const MUTANT_ARMOR_VENOM: I32F32 = I32F32::lit("0.50");

pub const MUTANT_NOISE: I32F32 = I32F32::lit("0");
pub const MUTANT_HEALTH_REGEN_PER_SECOND: I32F32 = I32F32::lit("10");

pub const MUTANT_COST_GOLD: i32 = 5000;
pub const MUTANT_COST_FOOD: i32 = 20;
pub const MUTANT_COST_WORKERS: i32 = 1;
pub const MUTANT_COST_IRON: i32 = 20;
pub const MUTANT_COST_OIL: i32 = 60;
pub const MUTANT_MAINTENANCE_GOLD: i32 = 80;
pub const MUTANT_MAINTENANCE_OIL: i32 = 5;
pub const MUTANT_BUILD_TIME_SECONDS: i32 = 41;
pub const MUTANT_RESEARCH_COST_GOLD: i32 = 6000;
pub const MUTANT_FIRST_UNIT_TOTAL_GOLD_COST: i32 = 11000;

#[derive(Component, Default)]
pub struct Mutant;

#[derive(Component, Clone, Copy)]
pub struct MutantArmorProfile {
    pub standard_reduction: I32F32,
    pub fire_reduction: I32F32,
    pub venom_reduction: I32F32,
}

impl Default for MutantArmorProfile {
    fn default() -> Self {
        Self {
            standard_reduction: MUTANT_ARMOR_STANDARD,
            fire_reduction: MUTANT_ARMOR_FIRE,
            venom_reduction: MUTANT_ARMOR_VENOM,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct MutantRegeneration {
    pub hp_per_second: I32F32,
}

impl Default for MutantRegeneration {
    fn default() -> Self {
        Self {
            hp_per_second: MUTANT_HEALTH_REGEN_PER_SECOND,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct MutantAttackCooldown {
    pub ticks_remaining: I32F32,
}

#[derive(Component, Clone, Copy, Default)]
pub struct MutantResearchGate {
    pub researched_in_high_research: bool,
}

#[derive(Component, Clone, Copy)]
pub struct MutantRecruitmentGate {
    pub engineering_center_online: bool,
    pub oil_income_non_negative: bool,
}

impl Default for MutantRecruitmentGate {
    fn default() -> Self {
        Self {
            engineering_center_online: false,
            oil_income_non_negative: true,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct MutantCombatState {
    pub in_combat: bool,
    pub severely_injured: bool,
}

#[derive(Component, Clone, Copy)]
pub struct MutantEconomy {
    pub cost_gold: i32,
    pub cost_food: i32,
    pub cost_workers: i32,
    pub cost_iron: i32,
    pub cost_oil: i32,
    pub maintenance_gold: i32,
    pub maintenance_oil: i32,
    pub build_time_seconds: i32,
    pub research_cost_gold: i32,
    pub first_unit_total_gold_cost: i32,
    pub noise: I32F32,
}

impl Default for MutantEconomy {
    fn default() -> Self {
        Self {
            cost_gold: MUTANT_COST_GOLD,
            cost_food: MUTANT_COST_FOOD,
            cost_workers: MUTANT_COST_WORKERS,
            cost_iron: MUTANT_COST_IRON,
            cost_oil: MUTANT_COST_OIL,
            maintenance_gold: MUTANT_MAINTENANCE_GOLD,
            maintenance_oil: MUTANT_MAINTENANCE_OIL,
            build_time_seconds: MUTANT_BUILD_TIME_SECONDS,
            research_cost_gold: MUTANT_RESEARCH_COST_GOLD,
            first_unit_total_gold_cost: MUTANT_FIRST_UNIT_TOTAL_GOLD_COST,
            noise: MUTANT_NOISE,
        }
    }
}

#[derive(Event, Clone, Copy)]
pub struct MutantDamageEvent {
    pub target: Entity,
    pub raw_damage: I32F32,
    pub damage_type: DamageType,
}

#[derive(Event, Clone, Copy)]
pub struct SetMutantResearchStateEvent {
    pub target: Entity,
    pub researched_in_high_research: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetMutantRecruitmentStateEvent {
    pub target: Entity,
    pub engineering_center_online: bool,
    pub oil_income_non_negative: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetMutantCombatStateEvent {
    pub target: Entity,
    pub in_combat: bool,
    pub severely_injured: bool,
}

pub fn mutant_research_gate_system(
    mut events: EventReader<SetMutantResearchStateEvent>,
    mut mutants: Query<&mut MutantResearchGate, With<Mutant>>,
) {
    for ev in events.read() {
        if let Ok(mut gate) = mutants.get_mut(ev.target) {
            gate.researched_in_high_research = ev.researched_in_high_research;
        }
    }
}

pub fn mutant_recruitment_gate_system(
    mut events: EventReader<SetMutantRecruitmentStateEvent>,
    mut mutants: Query<&mut MutantRecruitmentGate, With<Mutant>>,
) {
    for ev in events.read() {
        if let Ok(mut gate) = mutants.get_mut(ev.target) {
            gate.engineering_center_online = ev.engineering_center_online;
            gate.oil_income_non_negative = ev.oil_income_non_negative;
        }
    }
}

pub fn mutant_combat_state_system(
    mut events: EventReader<SetMutantCombatStateEvent>,
    mut mutants: Query<&mut MutantCombatState, With<Mutant>>,
) {
    for ev in events.read() {
        if let Ok(mut state) = mutants.get_mut(ev.target) {
            state.in_combat = ev.in_combat;
            state.severely_injured = ev.severely_injured;
        }
    }
}

pub fn mutant_receive_damage_system(
    mut events: EventReader<MutantDamageEvent>,
    mut mutants: Query<(&mut Health, &MutantArmorProfile), With<Mutant>>,
) {
    for ev in events.read() {
        if let Ok((mut health, armor)) = mutants.get_mut(ev.target) {
            let reduction = match ev.damage_type {
                DamageType::Standard => armor.standard_reduction,
                DamageType::Fire => armor.fire_reduction,
                DamageType::Venom => armor.venom_reduction,
            };
            let applied = ev.raw_damage * (I32F32::ONE - reduction);
            health.current = (health.current - applied).max(I32F32::ZERO);
        }
    }
}

pub fn mutant_regeneration_system(
    sim_hz: Res<SimHz>,
    mut mutants: Query<(&mut Health, &MutantRegeneration, &MutantCombatState), With<Mutant>>,
) {
    for (mut health, regen, combat_state) in &mut mutants {
        if health.current <= I32F32::ZERO || health.current >= health.max {
            continue;
        }
        if !combat_state.severely_injured {
            continue;
        }

        let regen_per_tick = regen.hp_per_second / sim_hz.0;
        health.current = (health.current + regen_per_tick).min(health.max);
    }
}

pub fn mutant_attack_tick_system(
    sim_hz: Res<SimHz>,
    mut mutants: Query<
        (
            Entity,
            &SimPosition,
            &UnitStats,
            &mut MutantAttackCooldown,
            &MutantResearchGate,
            &MutantRecruitmentGate,
            &MutantCombatState,
            &Health,
        ),
        With<Mutant>,
    >,
    infected_positions: Query<(Entity, &SimPosition), With<Infected>>,
    mut outgoing_damage: EventWriter<IncomingDamageEvent>,
) {
    for (mutant, position, stats, mut cooldown, research, recruitment, combat, health) in &mut mutants {
        if health.current <= I32F32::ZERO {
            continue;
        }
        if !research.researched_in_high_research {
            continue;
        }
        if !recruitment.engineering_center_online || !recruitment.oil_income_non_negative {
            continue;
        }
        if !combat.in_combat {
            continue;
        }

        if cooldown.ticks_remaining > I32F32::ZERO {
            cooldown.ticks_remaining -= I32F32::ONE;
            continue;
        }

        let range_sq = stats.attack_range * stats.attack_range;
        let mut hit_any = false;

        for (infected, infected_pos) in &infected_positions {
            let dx = infected_pos.x - position.x;
            let dy = infected_pos.y - position.y;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq <= range_sq {
                hit_any = true;
                outgoing_damage.send(IncomingDamageEvent {
                    target: infected,
                    raw_amount: stats.attack_damage,
                    damage_type: DamageType::Standard,
                    source: mutant,
                });
            }
        }

        if hit_any {
            cooldown.ticks_remaining = sim_hz.0 / stats.attack_speed;
        }
    }
}

pub fn mutant_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    mutants: Query<
        (
            &Health,
            &UnitStats,
            &MutantArmorProfile,
            &MutantRegeneration,
            &MutantAttackCooldown,
            &MutantResearchGate,
            &MutantRecruitmentGate,
            &MutantCombatState,
            &MutantEconomy,
        ),
        With<Mutant>,
    >,
) {
    for (health, stats, armor, regen, cooldown, research, recruitment, combat, economy) in &mutants {
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);

        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);

        checksum.accumulate(armor.standard_reduction.to_bits() as u64);
        checksum.accumulate(armor.fire_reduction.to_bits() as u64);
        checksum.accumulate(armor.venom_reduction.to_bits() as u64);

        checksum.accumulate(regen.hp_per_second.to_bits() as u64);
        checksum.accumulate(cooldown.ticks_remaining.to_bits() as u64);

        checksum.accumulate(u64::from(research.researched_in_high_research));
        checksum.accumulate(u64::from(recruitment.engineering_center_online));
        checksum.accumulate(u64::from(recruitment.oil_income_non_negative));
        checksum.accumulate(u64::from(combat.in_combat));
        checksum.accumulate(u64::from(combat.severely_injured));

        checksum.accumulate(economy.cost_gold as u64);
        checksum.accumulate(economy.cost_food as u64);
        checksum.accumulate(economy.cost_workers as u64);
        checksum.accumulate(economy.cost_iron as u64);
        checksum.accumulate(economy.cost_oil as u64);
        checksum.accumulate(economy.maintenance_gold as u64);
        checksum.accumulate(economy.maintenance_oil as u64);
        checksum.accumulate(economy.build_time_seconds as u64);
        checksum.accumulate(economy.research_cost_gold as u64);
        checksum.accumulate(economy.first_unit_total_gold_cost as u64);
        checksum.accumulate(economy.noise.to_bits() as u64);
    }
}

pub struct MutantPlugin;

impl Plugin for MutantPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MutantDamageEvent>()
            .add_event::<SetMutantResearchStateEvent>()
            .add_event::<SetMutantRecruitmentStateEvent>()
            .add_event::<SetMutantCombatStateEvent>()
            .add_systems(
                FixedUpdate,
                (
                    mutant_research_gate_system,
                    mutant_recruitment_gate_system,
                    mutant_combat_state_system,
                    mutant_receive_damage_system,
                    mutant_regeneration_system,
                    mutant_attack_tick_system,
                    mutant_checksum_system,
                )
                    .chain(),
            );
    }
}
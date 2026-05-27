// Sources: vault/units/mutant.md, vault/buildings/engineering_center.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, SimChecksumState, SimPosition, UnitStats};

const MUTANT_HP: I32F32 = I32F32::lit("2000");
const MUTANT_MOVE_SPEED: I32F32 = I32F32::lit("6");
const MUTANT_ATTACK_RANGE: I32F32 = I32F32::lit("2");
const MUTANT_ATTACK_SPEED: I32F32 = I32F32::lit("2");
const MUTANT_ATTACK_DAMAGE: I32F32 = I32F32::lit("30");
const MUTANT_WATCH_RANGE: I32F32 = I32F32::lit("12");
const MUTANT_BASE_ARMOR_REDUCTION: I32F32 = I32F32::lit("0.25");
const MUTANT_FIRE_RESISTANCE: I32F32 = I32F32::lit("0.50");
const MUTANT_VENOM_RESISTANCE: I32F32 = I32F32::lit("0.50");
const MUTANT_REGEN_PER_SECOND: I32F32 = I32F32::lit("10");
const MUTANT_AOE_RANGE: I32F32 = I32F32::lit("2");
const MUTANT_ATTACK_NOISE: I32F32 = I32F32::lit("0");
const SIM_TICKS_PER_SECOND: i32 = 20;
const SEVERE_INJURY_RATIO: I32F32 = I32F32::lit("0.25");

#[derive(Component, Default)]
pub struct Mutant;

#[derive(Component, Clone, Copy, Default)]
pub struct ReplicatedUnitId(pub u64);

#[derive(Component, Clone, Copy)]
pub struct ArmorReduction(pub I32F32);

impl Default for ArmorReduction {
    fn default() -> Self {
        Self(MUTANT_BASE_ARMOR_REDUCTION)
    }
}

#[derive(Component, Clone, Copy)]
pub struct FireResistance(pub I32F32);

impl Default for FireResistance {
    fn default() -> Self {
        Self(MUTANT_FIRE_RESISTANCE)
    }
}

#[derive(Component, Clone, Copy)]
pub struct VenomResistance(pub I32F32);

impl Default for VenomResistance {
    fn default() -> Self {
        Self(MUTANT_VENOM_RESISTANCE)
    }
}

#[derive(Component, Clone, Copy)]
pub struct HealthRegenerationPerSecond(pub I32F32);

impl Default for HealthRegenerationPerSecond {
    fn default() -> Self {
        Self(MUTANT_REGEN_PER_SECOND)
    }
}

#[derive(Component, Clone, Copy)]
pub struct RegenPerTick(pub I32F32);

impl Default for RegenPerTick {
    fn default() -> Self {
        Self(MUTANT_REGEN_PER_SECOND / I32F32::from_num(SIM_TICKS_PER_SECOND))
    }
}

#[derive(Component, Clone, Copy)]
pub struct SevereInjuryThreshold(pub I32F32);

impl Default for SevereInjuryThreshold {
    fn default() -> Self {
        Self(SEVERE_INJURY_RATIO)
    }
}

#[derive(Component, Clone, Copy)]
pub struct AttackNoise(pub I32F32);

impl Default for AttackNoise {
    fn default() -> Self {
        Self(MUTANT_ATTACK_NOISE)
    }
}

#[derive(Component, Clone, Copy)]
pub struct MeleeAoeRange(pub I32F32);

impl Default for MeleeAoeRange {
    fn default() -> Self {
        Self(MUTANT_AOE_RANGE)
    }
}

#[derive(Bundle)]
pub struct MutantBundle {
    pub unit: Mutant,
    pub replicated_id: ReplicatedUnitId,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub armor_reduction: ArmorReduction,
    pub fire_resistance: FireResistance,
    pub venom_resistance: VenomResistance,
    pub health_regeneration_per_second: HealthRegenerationPerSecond,
    pub regen_per_tick: RegenPerTick,
    pub severe_injury_threshold: SevereInjuryThreshold,
    pub attack_noise: AttackNoise,
    pub melee_aoe_range: MeleeAoeRange,
}

#[derive(Resource, Default)]
pub struct NextReplicatedUnitId(pub u64);

#[derive(Event, Clone, Copy)]
pub struct SpawnMutantEvent {
    pub position: SimPosition,
}

impl Default for MutantBundle {
    fn default() -> Self {
        Self {
            unit: Mutant,
            replicated_id: ReplicatedUnitId::default(),
            position: SimPosition {
                x: I32F32::ZERO,
                y: I32F32::ZERO,
            },
            health: Health::full(MUTANT_HP),
            stats: UnitStats {
                move_speed: MUTANT_MOVE_SPEED,
                attack_range: MUTANT_ATTACK_RANGE,
                attack_damage: MUTANT_ATTACK_DAMAGE,
                attack_speed: MUTANT_ATTACK_SPEED,
                watch_range: MUTANT_WATCH_RANGE,
            },
            armor_reduction: ArmorReduction::default(),
            fire_resistance: FireResistance::default(),
            venom_resistance: VenomResistance::default(),
            health_regeneration_per_second: HealthRegenerationPerSecond::default(),
            regen_per_tick: RegenPerTick::default(),
            severe_injury_threshold: SevereInjuryThreshold::default(),
            attack_noise: AttackNoise::default(),
            melee_aoe_range: MeleeAoeRange::default(),
        }
    }
}

fn spawn_mutant_system(
    mut commands: Commands,
    mut events: EventReader<SpawnMutantEvent>,
    mut next_id: ResMut<NextReplicatedUnitId>,
) {
    for ev in events.read() {
        let mut bundle = MutantBundle::default();
        bundle.position = ev.position;
        bundle.replicated_id = ReplicatedUnitId(next_id.0);
        next_id.0 = next_id.0.wrapping_add(1);
        commands.spawn(bundle);
    }
}

fn mutant_regeneration_system(
    mut units: Query<
        (
            &mut Health,
            &RegenPerTick,
            &SevereInjuryThreshold,
            &HealthRegenerationPerSecond,
        ),
        With<Mutant>,
    >,
) {
    for (mut health, regen_per_tick, severe_threshold, _regen_per_second) in &mut units {
        if health.current >= health.max {
            continue;
        }

        let severe_cutoff = health.max * severe_threshold.0;
        if health.current > severe_cutoff {
            continue;
        }

        let new_health = health.current + regen_per_tick.0;
        health.current = if new_health > health.max {
            health.max
        } else {
            new_health
        };
    }
}

fn mutant_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    units: Query<
        (
            &ReplicatedUnitId,
            &SimPosition,
            &Health,
            &UnitStats,
            &ArmorReduction,
            &FireResistance,
            &VenomResistance,
            &HealthRegenerationPerSecond,
            &RegenPerTick,
            &SevereInjuryThreshold,
            &AttackNoise,
            &MeleeAoeRange,
        ),
        With<Mutant>,
    >,
) {
    for (
        replicated_id,
        pos,
        hp,
        stats,
        armor,
        fire_resistance,
        venom_resistance,
        regen_per_second,
        regen_per_tick,
        severe_threshold,
        noise,
        aoe_range,
    ) in &units
    {
        checksum.accumulate(replicated_id.0);
        checksum.accumulate(pos.x.to_bits() as u64);
        checksum.accumulate(pos.y.to_bits() as u64);
        checksum.accumulate(hp.current.to_bits() as u64);
        checksum.accumulate(hp.max.to_bits() as u64);
        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);
        checksum.accumulate(armor.0.to_bits() as u64);
        checksum.accumulate(fire_resistance.0.to_bits() as u64);
        checksum.accumulate(venom_resistance.0.to_bits() as u64);
        checksum.accumulate(regen_per_second.0.to_bits() as u64);
        checksum.accumulate(regen_per_tick.0.to_bits() as u64);
        checksum.accumulate(severe_threshold.0.to_bits() as u64);
        checksum.accumulate(noise.0.to_bits() as u64);
        checksum.accumulate(aoe_range.0.to_bits() as u64);
    }
}

pub struct MutantPlugin;

impl Plugin for MutantPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NextReplicatedUnitId>()
            .add_event::<SpawnMutantEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_mutant_system,
                    mutant_regeneration_system,
                    mutant_checksum_system,
                )
                    .chain(),
            );
    }
}
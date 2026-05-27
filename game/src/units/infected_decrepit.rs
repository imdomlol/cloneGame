// Sources: vault/infected/infected_decrepit.md, vault/infected/infected_young.md, vault/infected/infected_aged.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{
    EntityKilledEvent, Health, IncomingDamageEvent, NoiseEmittedEvent, SimChecksumState, SimPosition,
    UnitStats,
};
use crate::units::Infected;

const INFECTED_DECREPIT_HP: I32F32 = I32F32::lit("35");
const INFECTED_DECREPIT_MOVE_SPEED: I32F32 = I32F32::lit("0.4");
const INFECTED_DECREPIT_ATTACK_RANGE: I32F32 = I32F32::lit("0.5");
const INFECTED_DECREPIT_ATTACK_SPEED: I32F32 = I32F32::lit("1");
const INFECTED_DECREPIT_ATTACK_DAMAGE: I32F32 = I32F32::lit("6");
const INFECTED_DECREPIT_WATCH_RANGE: I32F32 = I32F32::lit("5");
const INFECTED_DECREPIT_NOISE_GENERATION: I32F32 = I32F32::lit("1");
const INFECTED_DECREPIT_ARMOR_REDUCTION: I32F32 = I32F32::lit("0");
const INFECTED_DECREPIT_EXPERIENCE_REWARD: I32F32 = I32F32::lit("1");

#[derive(Component, Clone, Copy, Default)]
pub struct InfectedDecrepitUnit;

#[derive(Component, Clone, Copy, Default)]
pub struct ReplicatedUnitId(pub u64);

#[derive(Component, Clone, Copy, Default)]
pub struct ArmorReduction(pub I32F32);

#[derive(Component, Clone, Copy, Default)]
pub struct NoiseGeneration(pub I32F32);

#[derive(Component, Clone, Copy, Default)]
pub struct ExperienceReward(pub I32F32);

#[derive(Component, Clone, Copy, Default)]
pub enum InfectedDecrepitAttackType {
    #[default]
    Melee,
}

#[derive(Bundle)]
pub struct InfectedDecrepitBundle {
    pub infected_faction: Infected,
    pub unit: InfectedDecrepitUnit,
    pub replicated_id: ReplicatedUnitId,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub armor_reduction: ArmorReduction,
    pub noise_generation: NoiseGeneration,
    pub experience_reward: ExperienceReward,
    pub attack_type: InfectedDecrepitAttackType,
}

impl Default for InfectedDecrepitBundle {
    fn default() -> Self {
        Self {
            infected_faction: Infected,
            unit: InfectedDecrepitUnit,
            replicated_id: ReplicatedUnitId::default(),
            position: SimPosition {
                x: I32F32::ZERO,
                y: I32F32::ZERO,
            },
            health: Health::full(INFECTED_DECREPIT_HP),
            stats: UnitStats {
                move_speed: INFECTED_DECREPIT_MOVE_SPEED,
                attack_range: INFECTED_DECREPIT_ATTACK_RANGE,
                attack_damage: INFECTED_DECREPIT_ATTACK_DAMAGE,
                attack_speed: INFECTED_DECREPIT_ATTACK_SPEED,
                watch_range: INFECTED_DECREPIT_WATCH_RANGE,
            },
            armor_reduction: ArmorReduction(INFECTED_DECREPIT_ARMOR_REDUCTION),
            noise_generation: NoiseGeneration(INFECTED_DECREPIT_NOISE_GENERATION),
            experience_reward: ExperienceReward(INFECTED_DECREPIT_EXPERIENCE_REWARD),
            attack_type: InfectedDecrepitAttackType::Melee,
        }
    }
}

#[derive(Resource, Default)]
pub struct NextInfectedDecrepitReplicatedUnitId(pub u64);

#[derive(Event, Clone, Copy)]
pub struct SpawnInfectedDecrepitEvent {
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy)]
pub struct InfectedDecrepitAttackResolvedEvent {
    pub entity: Entity,
}

fn spawn_infected_decrepit_system(
    mut commands: Commands,
    mut events: EventReader<SpawnInfectedDecrepitEvent>,
    mut next_id: ResMut<NextInfectedDecrepitReplicatedUnitId>,
) {
    for ev in events.read() {
        let mut bundle = InfectedDecrepitBundle::default();
        bundle.position = ev.position;
        bundle.replicated_id = ReplicatedUnitId(next_id.0);
        next_id.0 = next_id.0.wrapping_add(1);
        commands.spawn(bundle);
    }
}

fn infected_decrepit_attack_noise_system(
    mut events: EventReader<InfectedDecrepitAttackResolvedEvent>,
    infected_units: Query<(&SimPosition, &NoiseGeneration), With<InfectedDecrepitUnit>>,
    mut noise_writer: EventWriter<NoiseEmittedEvent>,
) {
    for ev in events.read() {
        let Ok((position, noise)) = infected_units.get(ev.entity) else {
            continue;
        };

        if noise.0 <= I32F32::ZERO {
            continue;
        }

        noise_writer.send(NoiseEmittedEvent {
            source: ev.entity,
            position: *position,
            amount: noise.0,
        });
    }
}

fn apply_infected_decrepit_damage_system(
    mut damage_events: EventReader<IncomingDamageEvent>,
    mut infected_units: Query<(&mut Health, &ArmorReduction), With<InfectedDecrepitUnit>>,
    infected_faction: Query<(), With<Infected>>,
    rewards: Query<&ExperienceReward, With<InfectedDecrepitUnit>>,
    mut killed_writer: EventWriter<EntityKilledEvent>,
) {
    for ev in damage_events.read() {
        let Ok((mut health, armor_reduction)) = infected_units.get_mut(ev.target) else {
            continue;
        };

        if health.current <= I32F32::ZERO {
            continue;
        }

        if infected_faction.get(ev.source).is_ok() {
            continue;
        }

        let multiplier = I32F32::ONE - armor_reduction.0;
        if multiplier <= I32F32::ZERO {
            continue;
        }

        let applied = ev.raw_amount * multiplier;
        if applied >= health.current {
            health.current = I32F32::ZERO;
            let exp_reward = rewards.get(ev.target).map_or(I32F32::ZERO, |r| r.0);
            killed_writer.send(EntityKilledEvent {
                entity: ev.target,
                killer: ev.source,
                exp_reward,
                difficulty_tier: 0,
            });
        } else {
            health.current = health.current - applied;
        }
    }
}

fn infected_decrepit_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    infected_units: Query<
        (
            &ReplicatedUnitId,
            &SimPosition,
            &Health,
            &UnitStats,
            &ArmorReduction,
            &NoiseGeneration,
            &ExperienceReward,
            &InfectedDecrepitAttackType,
        ),
        With<InfectedDecrepitUnit>,
    >,
) {
    for (replicated_id, position, health, stats, armor_reduction, noise_generation, experience_reward, attack_type) in
        &infected_units
    {
        checksum.accumulate(replicated_id.0);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);
        checksum.accumulate(armor_reduction.0.to_bits() as u64);
        checksum.accumulate(noise_generation.0.to_bits() as u64);
        checksum.accumulate(experience_reward.0.to_bits() as u64);

        let attack_type_bits = match attack_type {
            InfectedDecrepitAttackType::Melee => 0_u64,
        };
        checksum.accumulate(attack_type_bits);
    }
}

pub struct InfectedDecrepitPlugin;

impl Plugin for InfectedDecrepitPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NextInfectedDecrepitReplicatedUnitId>()
            .add_event::<SpawnInfectedDecrepitEvent>()
            .add_event::<InfectedDecrepitAttackResolvedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_infected_decrepit_system,
                    infected_decrepit_attack_noise_system,
                    apply_infected_decrepit_damage_system,
                    infected_decrepit_checksum_system,
                )
                    .chain(),
            );
    }
}
// Sources: vault/infected/infected_behemoth.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{
    EntityKilledEvent, Health, IncomingDamageEvent, NoiseEmittedEvent, SimChecksumState, SimPosition, UnitStats,
};
use crate::units::Infected;

const BEHEMOTH_HP: I32F32 = I32F32::lit("4000");
const BEHEMOTH_MOVE_SPEED: I32F32 = I32F32::lit("6");
const BEHEMOTH_ATTACK_RANGE: I32F32 = I32F32::lit("2");
const BEHEMOTH_ATTACK_SPEED: I32F32 = I32F32::lit("2");
const BEHEMOTH_ATTACK_DAMAGE: I32F32 = I32F32::lit("30");
const BEHEMOTH_WATCH_RANGE: I32F32 = I32F32::lit("12");
const BEHEMOTH_NOISE: I32F32 = I32F32::lit("200");
const BEHEMOTH_ARMOR_REDUCTION: I32F32 = I32F32::lit("0.10");
const BEHEMOTH_EXPERIENCE_REWARD: I32F32 = I32F32::lit("50");

#[derive(Component, Clone, Copy, Default)]
pub struct InfectedBehemothUnit;

#[derive(Component, Clone, Copy, Default)]
pub struct BehemothReplicatedUnitId(pub u64);

#[derive(Component, Clone, Copy, Default)]
pub struct ArmorReduction(pub I32F32);

#[derive(Component, Clone, Copy, Default)]
pub struct NoiseGeneration(pub I32F32);

#[derive(Component, Clone, Copy, Default)]
pub struct ExperienceReward(pub I32F32);

#[derive(Component, Clone, Copy, Default)]
pub struct HasAoe(pub bool);

#[derive(Component, Clone, Copy, Default)]
pub enum InfectedAttackType {
    #[default]
    Melee,
    Ranged,
}

#[derive(Bundle)]
pub struct InfectedBehemothBundle {
    pub infected_faction: Infected,
    pub unit: InfectedBehemothUnit,
    pub replicated_id: BehemothReplicatedUnitId,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub armor_reduction: ArmorReduction,
    pub noise_generation: NoiseGeneration,
    pub experience_reward: ExperienceReward,
    pub attack_type: InfectedAttackType,
    pub aoe: HasAoe,
}

impl Default for InfectedBehemothBundle {
    fn default() -> Self {
        Self {
            infected_faction: Infected,
            unit: InfectedBehemothUnit,
            replicated_id: BehemothReplicatedUnitId::default(),
            position: SimPosition {
                x: I32F32::ZERO,
                y: I32F32::ZERO,
            },
            health: Health::full(BEHEMOTH_HP),
            stats: UnitStats {
                move_speed: BEHEMOTH_MOVE_SPEED,
                attack_range: BEHEMOTH_ATTACK_RANGE,
                attack_damage: BEHEMOTH_ATTACK_DAMAGE,
                attack_speed: BEHEMOTH_ATTACK_SPEED,
                watch_range: BEHEMOTH_WATCH_RANGE,
            },
            armor_reduction: ArmorReduction(BEHEMOTH_ARMOR_REDUCTION),
            noise_generation: NoiseGeneration(BEHEMOTH_NOISE),
            experience_reward: ExperienceReward(BEHEMOTH_EXPERIENCE_REWARD),
            attack_type: InfectedAttackType::Melee,
            aoe: HasAoe(true),
        }
    }
}

#[derive(Resource, Default)]
pub struct NextBehemothReplicatedUnitId(pub u64);

#[derive(Event, Clone, Copy)]
pub struct SpawnInfectedBehemothEvent {
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy)]
pub struct InfectedBehemothAttackResolvedEvent {
    pub entity: Entity,
}

fn spawn_infected_behemoth_system(
    mut commands: Commands,
    mut events: EventReader<SpawnInfectedBehemothEvent>,
    mut next_id: ResMut<NextBehemothReplicatedUnitId>,
) {
    for ev in events.read() {
        let mut bundle = InfectedBehemothBundle::default();
        bundle.position = ev.position;
        bundle.replicated_id = BehemothReplicatedUnitId(next_id.0);
        next_id.0 = next_id.0.wrapping_add(1);
        commands.spawn(bundle);
    }
}

fn infected_behemoth_attack_noise_system(
    mut events: EventReader<InfectedBehemothAttackResolvedEvent>,
    behemoths: Query<(&SimPosition, &NoiseGeneration), With<InfectedBehemothUnit>>,
    mut noise_writer: EventWriter<NoiseEmittedEvent>,
) {
    for ev in events.read() {
        let Ok((position, noise)) = behemoths.get(ev.entity) else {
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

fn apply_infected_behemoth_damage_system(
    mut damage_events: EventReader<IncomingDamageEvent>,
    mut behemoths: Query<(&mut Health, &ArmorReduction), With<InfectedBehemothUnit>>,
    infected_faction: Query<(), With<Infected>>,
    rewards: Query<&ExperienceReward, With<InfectedBehemothUnit>>,
    mut killed_writer: EventWriter<EntityKilledEvent>,
) {
    for ev in damage_events.read() {
        let Ok((mut health, armor_reduction)) = behemoths.get_mut(ev.target) else {
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

fn infected_behemoth_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    behemoths: Query<
        (
            &BehemothReplicatedUnitId,
            &SimPosition,
            &Health,
            &UnitStats,
            &ArmorReduction,
            &NoiseGeneration,
            &ExperienceReward,
            &InfectedAttackType,
            &HasAoe,
        ),
        With<InfectedBehemothUnit>,
    >,
) {
    for (replicated_id, pos, health, stats, armor, noise, exp, attack_type, aoe) in &behemoths {
        checksum.accumulate(replicated_id.0);
        checksum.accumulate(pos.x.to_bits() as u64);
        checksum.accumulate(pos.y.to_bits() as u64);
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);
        checksum.accumulate(armor.0.to_bits() as u64);
        checksum.accumulate(noise.0.to_bits() as u64);
        checksum.accumulate(exp.0.to_bits() as u64);

        let attack_type_bits = match attack_type {
            InfectedAttackType::Melee => 0_u64,
            InfectedAttackType::Ranged => 1_u64,
        };
        checksum.accumulate(attack_type_bits);
        checksum.accumulate(u64::from(aoe.0));
    }
}

pub struct InfectedBehemothPlugin;

impl Plugin for InfectedBehemothPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NextBehemothReplicatedUnitId>()
            .add_event::<SpawnInfectedBehemothEvent>()
            .add_event::<InfectedBehemothAttackResolvedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_infected_behemoth_system,
                    infected_behemoth_attack_noise_system,
                    apply_infected_behemoth_damage_system,
                    infected_behemoth_checksum_system,
                )
                    .chain(),
            );
    }
}
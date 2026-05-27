// Sources: vault/infected/infected_giant.md, vault/campaign_maps/lands_of_the_giant.md, vault/campaign_maps/the_wasteland_of_the_giants.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{
    DamageType, EntityKilledEvent, Health, IncomingDamageEvent, NoiseEmittedEvent, SimChecksumState, SimPosition,
    UnitStats,
};
use crate::units::Infected;

const INFECTED_GIANT_BASE_HP: I32F32 = I32F32::lit("10000");
const INFECTED_GIANT_MIN_HP: I32F32 = I32F32::lit("4000");
const INFECTED_GIANT_MAX_HP: I32F32 = I32F32::lit("20000");
const INFECTED_GIANT_MOVE_SPEED: I32F32 = I32F32::lit("4");
const INFECTED_GIANT_ATTACK_RANGE: I32F32 = I32F32::lit("3");
const INFECTED_GIANT_ATTACK_SPEED: I32F32 = I32F32::lit("1.25");
const INFECTED_GIANT_ATTACK_DAMAGE: I32F32 = I32F32::lit("160");
const INFECTED_GIANT_WATCH_RANGE: I32F32 = I32F32::lit("10");
const INFECTED_GIANT_ARMOR_REDUCTION: I32F32 = I32F32::lit("0.30");
const INFECTED_GIANT_NOISE: I32F32 = I32F32::ZERO;
const INFECTED_GIANT_EXPERIENCE_REWARD: I32F32 = I32F32::ZERO;

#[derive(Component, Clone, Copy, Default)]
pub struct InfectedGiantUnit;

#[derive(Component, Clone, Copy, Default)]
pub struct ReplicatedUnitId(pub u64);

#[derive(Component, Clone, Copy, Default)]
pub struct ArmorReduction(pub I32F32);

#[derive(Component, Clone, Copy, Default)]
pub struct RegenerationPerSecond(pub I32F32);

#[derive(Component, Clone, Copy, Default)]
pub struct NoiseGeneration(pub I32F32);

#[derive(Component, Clone, Copy, Default)]
pub struct ExperienceReward(pub I32F32);

#[derive(Component, Clone, Copy, Default)]
pub struct UsesSightOnlyAggro(pub bool);

#[derive(Component, Clone, Copy, Default)]
pub struct UsesFriendlyFire(pub bool);

#[derive(Component, Clone, Copy, Default)]
pub struct CanSqueezeOneTileGap(pub bool);

#[derive(Component, Clone, Copy, Default)]
pub struct HpBounds {
    pub min: I32F32,
    pub max: I32F32,
}

#[derive(Component, Clone, Copy, Default)]
pub enum InfectedGiantAttackType {
    #[default]
    Melee,
}

#[derive(Component, Clone, Copy, Default)]
pub enum InfectedGiantAoeProfile {
    #[default]
    VeryLargeAoe,
}

#[derive(Bundle)]
pub struct InfectedGiantBundle {
    pub infected_faction: Infected,
    pub unit: InfectedGiantUnit,
    pub replicated_id: ReplicatedUnitId,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub armor_reduction: ArmorReduction,
    pub regeneration_per_second: RegenerationPerSecond,
    pub noise_generation: NoiseGeneration,
    pub experience_reward: ExperienceReward,
    pub uses_sight_only_aggro: UsesSightOnlyAggro,
    pub uses_friendly_fire: UsesFriendlyFire,
    pub can_squeeze_one_tile_gap: CanSqueezeOneTileGap,
    pub hp_bounds: HpBounds,
    pub attack_type: InfectedGiantAttackType,
    pub aoe_profile: InfectedGiantAoeProfile,
}

impl Default for InfectedGiantBundle {
    fn default() -> Self {
        Self {
            infected_faction: Infected,
            unit: InfectedGiantUnit,
            replicated_id: ReplicatedUnitId::default(),
            position: SimPosition {
                x: I32F32::ZERO,
                y: I32F32::ZERO,
            },
            health: Health::full(INFECTED_GIANT_BASE_HP),
            stats: UnitStats {
                move_speed: INFECTED_GIANT_MOVE_SPEED,
                attack_range: INFECTED_GIANT_ATTACK_RANGE,
                attack_damage: INFECTED_GIANT_ATTACK_DAMAGE,
                attack_speed: INFECTED_GIANT_ATTACK_SPEED,
                watch_range: INFECTED_GIANT_WATCH_RANGE,
            },
            armor_reduction: ArmorReduction(INFECTED_GIANT_ARMOR_REDUCTION),
            regeneration_per_second: RegenerationPerSecond(I32F32::lit("50")),
            noise_generation: NoiseGeneration(INFECTED_GIANT_NOISE),
            experience_reward: ExperienceReward(INFECTED_GIANT_EXPERIENCE_REWARD),
            uses_sight_only_aggro: UsesSightOnlyAggro(true),
            uses_friendly_fire: UsesFriendlyFire(true),
            can_squeeze_one_tile_gap: CanSqueezeOneTileGap(false),
            hp_bounds: HpBounds {
                min: INFECTED_GIANT_MIN_HP,
                max: INFECTED_GIANT_MAX_HP,
            },
            attack_type: InfectedGiantAttackType::Melee,
            aoe_profile: InfectedGiantAoeProfile::VeryLargeAoe,
        }
    }
}

#[derive(Resource, Default)]
pub struct NextReplicatedUnitId(pub u64);

#[derive(Event, Clone, Copy)]
pub struct SpawnInfectedGiantEvent {
    pub position: SimPosition,
    pub campaign_hp_override: Option<I32F32>,
}

#[derive(Event, Clone, Copy)]
pub struct InfectedGiantAttackResolvedEvent {
    pub entity: Entity,
}

fn clamp_hp_to_bounds(hp: I32F32) -> I32F32 {
    if hp < INFECTED_GIANT_MIN_HP {
        INFECTED_GIANT_MIN_HP
    } else if hp > INFECTED_GIANT_MAX_HP {
        INFECTED_GIANT_MAX_HP
    } else {
        hp
    }
}

fn spawn_infected_giant_system(
    mut commands: Commands,
    mut events: EventReader<SpawnInfectedGiantEvent>,
    mut next_id: ResMut<NextReplicatedUnitId>,
) {
    for ev in events.read() {
        let mut bundle = InfectedGiantBundle::default();
        let hp = ev
            .campaign_hp_override
            .map(clamp_hp_to_bounds)
            .unwrap_or(INFECTED_GIANT_BASE_HP);
        bundle.position = ev.position;
        bundle.health = Health::full(hp);
        bundle.replicated_id = ReplicatedUnitId(next_id.0);
        next_id.0 = next_id.0.wrapping_add(1);
        commands.spawn(bundle);
    }
}

fn infected_giant_attack_noise_system(
    mut events: EventReader<InfectedGiantAttackResolvedEvent>,
    giant_units: Query<(&SimPosition, &NoiseGeneration), With<InfectedGiantUnit>>,
    mut noise_writer: EventWriter<NoiseEmittedEvent>,
) {
    for ev in events.read() {
        let Ok((position, noise)) = giant_units.get(ev.entity) else {
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

fn infected_giant_regeneration_system(
    mut giant_units: Query<(&mut Health, &RegenerationPerSecond), With<InfectedGiantUnit>>,
    sim_hz: Res<crate::sim::SimHz>,
) {
    if sim_hz.0 <= I32F32::ZERO {
        return;
    }

    for (mut health, regen_per_second) in &mut giant_units {
        if health.current >= health.max {
            continue;
        }

        let regen_per_tick = regen_per_second.0 / sim_hz.0;
        let healed = health.current + regen_per_tick;
        health.current = if healed > health.max { health.max } else { healed };
    }
}

fn apply_infected_giant_damage_system(
    mut damage_events: EventReader<IncomingDamageEvent>,
    mut giant_units: Query<(&mut Health, &ArmorReduction), With<InfectedGiantUnit>>,
    rewards: Query<&ExperienceReward, With<InfectedGiantUnit>>,
    mut killed_writer: EventWriter<EntityKilledEvent>,
) {
    for ev in damage_events.read() {
        let Ok((mut health, armor_reduction)) = giant_units.get_mut(ev.target) else {
            continue;
        };

        if health.current <= I32F32::ZERO {
            continue;
        }

        let mut multiplier = I32F32::ONE - armor_reduction.0;
        if ev.damage_type == DamageType::Venom {
            multiplier = multiplier * I32F32::ONE;
        }

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

fn infected_giant_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    giant_units: Query<
        (
            &ReplicatedUnitId,
            &SimPosition,
            &Health,
            &UnitStats,
            &ArmorReduction,
            &RegenerationPerSecond,
            &NoiseGeneration,
            &ExperienceReward,
            &UsesSightOnlyAggro,
            &UsesFriendlyFire,
            &CanSqueezeOneTileGap,
            &HpBounds,
            &InfectedGiantAttackType,
            &InfectedGiantAoeProfile,
        ),
        With<InfectedGiantUnit>,
    >,
) {
    for (
        replicated_id,
        position,
        health,
        stats,
        armor_reduction,
        regen,
        noise_generation,
        experience_reward,
        uses_sight_only_aggro,
        uses_friendly_fire,
        can_squeeze_one_tile_gap,
        hp_bounds,
        attack_type,
        aoe_profile,
    ) in &giant_units
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
        checksum.accumulate(regen.0.to_bits() as u64);
        checksum.accumulate(noise_generation.0.to_bits() as u64);
        checksum.accumulate(experience_reward.0.to_bits() as u64);
        checksum.accumulate(u64::from(uses_sight_only_aggro.0));
        checksum.accumulate(u64::from(uses_friendly_fire.0));
        checksum.accumulate(u64::from(can_squeeze_one_tile_gap.0));
        checksum.accumulate(hp_bounds.min.to_bits() as u64);
        checksum.accumulate(hp_bounds.max.to_bits() as u64);

        let attack_type_bits = match attack_type {
            InfectedGiantAttackType::Melee => 0_u64,
        };
        checksum.accumulate(attack_type_bits);

        let aoe_bits = match aoe_profile {
            InfectedGiantAoeProfile::VeryLargeAoe => 0_u64,
        };
        checksum.accumulate(aoe_bits);
    }
}

pub struct InfectedGiantPlugin;

impl Plugin for InfectedGiantPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NextReplicatedUnitId>()
            .add_event::<SpawnInfectedGiantEvent>()
            .add_event::<InfectedGiantAttackResolvedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_infected_giant_system,
                    infected_giant_attack_noise_system,
                    infected_giant_regeneration_system,
                    apply_infected_giant_damage_system,
                    infected_giant_checksum_system,
                )
                    .chain(),
            );
    }
}
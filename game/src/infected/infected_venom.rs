// Sources: vault/infected/infected_venom.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{
    DamageType, EntityKilledEvent, Health, IncomingDamageEvent, NoiseEmittedEvent, SimChecksumState, SimPosition,
    UnitStats,
};
use crate::units::Infected;

const INFECTED_VENOM_HP: I32F32 = I32F32::lit("120");
const INFECTED_VENOM_MOVE_SPEED: I32F32 = I32F32::lit("1.75");
const INFECTED_VENOM_ATTACK_RANGE: I32F32 = I32F32::lit("4.5");
const INFECTED_VENOM_ATTACK_SPEED: I32F32 = I32F32::lit("0.5");
const INFECTED_VENOM_ATTACK_DAMAGE: I32F32 = I32F32::lit("30");
const INFECTED_VENOM_WATCH_RANGE: I32F32 = I32F32::lit("8");
const INFECTED_VENOM_ARMOR_REDUCTION: I32F32 = I32F32::lit("0.10");
const INFECTED_VENOM_NOISE: I32F32 = I32F32::lit("10");
const INFECTED_VENOM_EXP_REWARD: I32F32 = I32F32::lit("10");

#[derive(Component, Clone, Copy, Default)]
pub struct InfectedVenomUnit;

#[derive(Component, Clone, Copy, Default)]
pub struct ReplicatedVenomId(pub u64);

#[derive(Component, Clone, Copy, Default)]
pub struct ArmorReduction(pub I32F32);

#[derive(Component, Clone, Copy, Default)]
pub struct NoiseGeneration(pub I32F32);

#[derive(Component, Clone, Copy, Default)]
pub struct ExperienceReward(pub I32F32);

#[derive(Component, Clone, Copy, Default)]
pub struct UsesRangedProjectile(pub bool);

#[derive(Component, Clone, Copy, Default)]
pub struct HasModerateAoe(pub bool);

#[derive(Component, Clone, Copy, Default)]
pub struct ProjectileIsDodgeable(pub bool);

#[derive(Component, Clone, Copy, Default)]
pub struct TargetsWallsWhenNoVisibleTarget(pub bool);

#[derive(Component, Clone, Copy, Default)]
pub struct CannotInfectWhileTargetIsBeingRepaired(pub bool);

#[derive(Bundle)]
pub struct InfectedVenomBundle {
    pub infected_faction: Infected,
    pub unit: InfectedVenomUnit,
    pub replicated_id: ReplicatedVenomId,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub armor_reduction: ArmorReduction,
    pub noise_generation: NoiseGeneration,
    pub experience_reward: ExperienceReward,
    pub uses_ranged_projectile: UsesRangedProjectile,
    pub has_moderate_aoe: HasModerateAoe,
    pub projectile_is_dodgeable: ProjectileIsDodgeable,
    pub targets_walls_when_no_visible_target: TargetsWallsWhenNoVisibleTarget,
    pub cannot_infect_while_target_is_being_repaired: CannotInfectWhileTargetIsBeingRepaired,
}

impl Default for InfectedVenomBundle {
    fn default() -> Self {
        Self {
            infected_faction: Infected,
            unit: InfectedVenomUnit,
            replicated_id: ReplicatedVenomId::default(),
            position: SimPosition {
                x: I32F32::ZERO,
                y: I32F32::ZERO,
            },
            health: Health::full(INFECTED_VENOM_HP),
            stats: UnitStats {
                move_speed: INFECTED_VENOM_MOVE_SPEED,
                attack_range: INFECTED_VENOM_ATTACK_RANGE,
                attack_damage: INFECTED_VENOM_ATTACK_DAMAGE,
                attack_speed: INFECTED_VENOM_ATTACK_SPEED,
                watch_range: INFECTED_VENOM_WATCH_RANGE,
            },
            armor_reduction: ArmorReduction(INFECTED_VENOM_ARMOR_REDUCTION),
            noise_generation: NoiseGeneration(INFECTED_VENOM_NOISE),
            experience_reward: ExperienceReward(INFECTED_VENOM_EXP_REWARD),
            uses_ranged_projectile: UsesRangedProjectile(true),
            has_moderate_aoe: HasModerateAoe(true),
            projectile_is_dodgeable: ProjectileIsDodgeable(true),
            targets_walls_when_no_visible_target: TargetsWallsWhenNoVisibleTarget(true),
            cannot_infect_while_target_is_being_repaired: CannotInfectWhileTargetIsBeingRepaired(true),
        }
    }
}

#[derive(Resource, Default)]
pub struct NextReplicatedVenomId(pub u64);

#[derive(Event, Clone, Copy)]
pub struct SpawnInfectedVenomEvent {
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy)]
pub struct InfectedVenomAttackResolvedEvent {
    pub entity: Entity,
}

fn spawn_infected_venom_system(
    mut commands: Commands,
    mut events: EventReader<SpawnInfectedVenomEvent>,
    mut next_id: ResMut<NextReplicatedVenomId>,
) {
    for ev in events.read() {
        let mut bundle = InfectedVenomBundle::default();
        bundle.position = ev.position;
        bundle.replicated_id = ReplicatedVenomId(next_id.0);
        next_id.0 = next_id.0.wrapping_add(1);
        commands.spawn(bundle);
    }
}

fn infected_venom_attack_noise_system(
    mut events: EventReader<InfectedVenomAttackResolvedEvent>,
    venom_units: Query<(&SimPosition, &NoiseGeneration), With<InfectedVenomUnit>>,
    mut noise_writer: EventWriter<NoiseEmittedEvent>,
) {
    for ev in events.read() {
        let Ok((position, noise)) = venom_units.get(ev.entity) else {
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

fn apply_infected_venom_damage_system(
    mut damage_events: EventReader<IncomingDamageEvent>,
    mut venom_units: Query<(&mut Health, &ArmorReduction), With<InfectedVenomUnit>>,
    infected_faction: Query<(), With<Infected>>,
    rewards: Query<&ExperienceReward, With<InfectedVenomUnit>>,
    mut killed_writer: EventWriter<EntityKilledEvent>,
) {
    for ev in damage_events.read() {
        let Ok((mut health, armor_reduction)) = venom_units.get_mut(ev.target) else {
            continue;
        };

        if health.current <= I32F32::ZERO {
            continue;
        }

        if infected_faction.get(ev.source).is_ok() {
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

fn infected_venom_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    venom_units: Query<
        (
            &ReplicatedVenomId,
            &SimPosition,
            &Health,
            &UnitStats,
            &ArmorReduction,
            &NoiseGeneration,
            &ExperienceReward,
            &UsesRangedProjectile,
            &HasModerateAoe,
            &ProjectileIsDodgeable,
            &TargetsWallsWhenNoVisibleTarget,
            &CannotInfectWhileTargetIsBeingRepaired,
        ),
        With<InfectedVenomUnit>,
    >,
) {
    for (
        replicated_id,
        pos,
        health,
        stats,
        armor_reduction,
        noise_generation,
        experience_reward,
        uses_ranged_projectile,
        has_moderate_aoe,
        projectile_is_dodgeable,
        targets_walls_when_no_visible_target,
        cannot_infect_while_target_is_being_repaired,
    ) in &venom_units
    {
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
        checksum.accumulate(armor_reduction.0.to_bits() as u64);
        checksum.accumulate(noise_generation.0.to_bits() as u64);
        checksum.accumulate(experience_reward.0.to_bits() as u64);
        checksum.accumulate(u64::from(uses_ranged_projectile.0));
        checksum.accumulate(u64::from(has_moderate_aoe.0));
        checksum.accumulate(u64::from(projectile_is_dodgeable.0));
        checksum.accumulate(u64::from(targets_walls_when_no_visible_target.0));
        checksum.accumulate(u64::from(cannot_infect_while_target_is_being_repaired.0));
    }
}

pub struct InfectedVenomPlugin;

impl Plugin for InfectedVenomPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NextReplicatedVenomId>()
            .add_event::<SpawnInfectedVenomEvent>()
            .add_event::<InfectedVenomAttackResolvedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_infected_venom_system,
                    infected_venom_attack_noise_system,
                    apply_infected_venom_damage_system,
                    infected_venom_checksum_system,
                )
                    .chain(),
            );
    }
}
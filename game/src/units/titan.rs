// Sources: vault/units/titan.md, vault/buildings/engineering_center.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{
    DamageType, EntityKilledEvent, Health, IncomingDamageEvent, NoiseEmittedEvent, SimChecksumState,
    SimPosition, UnitStats,
};

const TITAN_HP: I32F32 = I32F32::lit("800");
const TITAN_MOVE_SPEED: I32F32 = I32F32::lit("3");
const TITAN_ATTACK_RANGE: I32F32 = I32F32::lit("9");
const TITAN_ATTACK_SPEED: I32F32 = I32F32::lit("5");
const TITAN_ATTACK_DAMAGE: I32F32 = I32F32::lit("32");
const TITAN_WATCH_RANGE: I32F32 = I32F32::lit("12");

const TITAN_ARMOR_REDUCTION: I32F32 = I32F32::lit("0.40");
const TITAN_ARMOR_POINTS: I32F32 = I32F32::lit("9");
const TITAN_ATTACK_NOISE: I32F32 = I32F32::lit("20");

const TITAN_STANDARD_DAMAGE_MULTIPLIER: I32F32 = I32F32::lit("0.60");
const TITAN_FIRE_DAMAGE_MULTIPLIER: I32F32 = I32F32::lit("0.60");
const TITAN_VENOM_DAMAGE_MULTIPLIER: I32F32 = I32F32::ONE;

#[derive(Component, Default)]
pub struct Titan;

#[derive(Component, Clone, Copy, Default)]
pub struct ReplicatedUnitId(pub u64);

#[derive(Component, Clone, Copy, Default)]
pub struct ArmorReduction(pub I32F32);

#[derive(Component, Clone, Copy, Default)]
pub struct ArmorPoints(pub I32F32);

#[derive(Component, Clone, Copy, Default)]
pub struct AttackNoise(pub I32F32);

#[derive(Component, Clone, Copy, Default)]
pub struct MediumAreaOfEffect(pub bool);

#[derive(Component, Clone, Copy, Default)]
pub struct LargeCollisionSize(pub bool);

#[derive(Component, Clone, Copy, Default)]
pub struct BetterVsGiant(pub bool);

#[derive(Component, Clone, Copy, Default)]
pub struct WeakerVsVenom(pub bool);

#[derive(Bundle)]
pub struct TitanBundle {
    pub unit: Titan,
    pub replicated_id: ReplicatedUnitId,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub armor_reduction: ArmorReduction,
    pub armor_points: ArmorPoints,
    pub attack_noise: AttackNoise,
    pub medium_area_of_effect: MediumAreaOfEffect,
    pub large_collision_size: LargeCollisionSize,
    pub better_vs_giant: BetterVsGiant,
    pub weaker_vs_venom: WeakerVsVenom,
}

impl Default for TitanBundle {
    fn default() -> Self {
        Self {
            unit: Titan,
            replicated_id: ReplicatedUnitId::default(),
            position: SimPosition {
                x: I32F32::ZERO,
                y: I32F32::ZERO,
            },
            health: Health::full(TITAN_HP),
            stats: UnitStats {
                move_speed: TITAN_MOVE_SPEED,
                attack_range: TITAN_ATTACK_RANGE,
                attack_damage: TITAN_ATTACK_DAMAGE,
                attack_speed: TITAN_ATTACK_SPEED,
                watch_range: TITAN_WATCH_RANGE,
            },
            armor_reduction: ArmorReduction(TITAN_ARMOR_REDUCTION),
            armor_points: ArmorPoints(TITAN_ARMOR_POINTS),
            attack_noise: AttackNoise(TITAN_ATTACK_NOISE),
            medium_area_of_effect: MediumAreaOfEffect(true),
            large_collision_size: LargeCollisionSize(true),
            better_vs_giant: BetterVsGiant(true),
            weaker_vs_venom: WeakerVsVenom(true),
        }
    }
}

#[derive(Resource, Default)]
pub struct NextReplicatedUnitId(pub u64);

#[derive(Event, Clone, Copy)]
pub struct SpawnTitanEvent {
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy)]
pub struct TitanFiredEvent {
    pub entity: Entity,
}

fn spawn_titan_system(
    mut commands: Commands,
    mut events: EventReader<SpawnTitanEvent>,
    mut next_id: ResMut<NextReplicatedUnitId>,
) {
    for ev in events.read() {
        let mut bundle = TitanBundle::default();
        bundle.position = ev.position;
        bundle.replicated_id = ReplicatedUnitId(next_id.0);
        next_id.0 = next_id.0.wrapping_add(1);
        commands.spawn(bundle);
    }
}

fn titan_fire_noise_system(
    mut events: EventReader<TitanFiredEvent>,
    titans: Query<(&SimPosition, &AttackNoise), With<Titan>>,
    mut noise_writer: EventWriter<NoiseEmittedEvent>,
) {
    for ev in events.read() {
        let Ok((position, noise)) = titans.get(ev.entity) else {
            continue;
        };

        noise_writer.send(NoiseEmittedEvent {
            source: ev.entity,
            position: *position,
            amount: noise.0,
        });
    }
}

fn apply_titan_damage_system(
    mut damage_events: EventReader<IncomingDamageEvent>,
    mut titans: Query<&mut Health, With<Titan>>,
    mut killed_writer: EventWriter<EntityKilledEvent>,
) {
    for ev in damage_events.read() {
        let Ok(mut health) = titans.get_mut(ev.target) else {
            continue;
        };

        if health.current <= I32F32::ZERO {
            continue;
        }

        let multiplier = match ev.damage_type {
            DamageType::Standard => TITAN_STANDARD_DAMAGE_MULTIPLIER,
            DamageType::Fire => TITAN_FIRE_DAMAGE_MULTIPLIER,
            DamageType::Venom => TITAN_VENOM_DAMAGE_MULTIPLIER,
        };

        let applied = ev.raw_amount * multiplier;
        if applied >= health.current {
            health.current = I32F32::ZERO;
            killed_writer.send(EntityKilledEvent {
                entity: ev.target,
                killer: ev.source,
                exp_reward: I32F32::ZERO,
                difficulty_tier: 0,
            });
        } else {
            health.current = health.current - applied;
        }
    }
}

fn titan_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    units: Query<
        (
            &ReplicatedUnitId,
            &SimPosition,
            &Health,
            &UnitStats,
            &ArmorReduction,
            &ArmorPoints,
            &AttackNoise,
            &MediumAreaOfEffect,
            &LargeCollisionSize,
            &BetterVsGiant,
            &WeakerVsVenom,
        ),
        With<Titan>,
    >,
) {
    for (
        replicated_id,
        pos,
        hp,
        stats,
        armor_reduction,
        armor_points,
        attack_noise,
        medium_aoe,
        large_collision,
        better_vs_giant,
        weaker_vs_venom,
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
        checksum.accumulate(armor_reduction.0.to_bits() as u64);
        checksum.accumulate(armor_points.0.to_bits() as u64);
        checksum.accumulate(attack_noise.0.to_bits() as u64);
        checksum.accumulate(u64::from(medium_aoe.0));
        checksum.accumulate(u64::from(large_collision.0));
        checksum.accumulate(u64::from(better_vs_giant.0));
        checksum.accumulate(u64::from(weaker_vs_venom.0));
    }
}

pub struct TitanPlugin;

impl Plugin for TitanPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NextReplicatedUnitId>()
            .add_event::<SpawnTitanEvent>()
            .add_event::<TitanFiredEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_titan_system,
                    titan_fire_noise_system,
                    apply_titan_damage_system,
                    titan_checksum_system,
                )
                    .chain(),
            );
    }
}
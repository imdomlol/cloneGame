// Sources: vault/units/soldier.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{
    DamageType, EntityKilledEvent, Health, IncomingDamageEvent, NoiseEmittedEvent, SimChecksumState,
    SimPosition, UnitStats,
};

const SOLDIER_HP: I32F32 = I32F32::lit("120");
const SOLDIER_MOVE_SPEED: I32F32 = I32F32::lit("2.4");
const SOLDIER_ATTACK_RANGE: I32F32 = I32F32::lit("5");
const SOLDIER_ATTACK_SPEED: I32F32 = I32F32::lit("2");
const SOLDIER_ATTACK_DAMAGE: I32F32 = I32F32::lit("16");
const SOLDIER_WATCH_RANGE: I32F32 = I32F32::lit("6");

const SOLDIER_STANDARD_DAMAGE_MULTIPLIER: I32F32 = I32F32::lit("0.60");
const SOLDIER_FIRE_DAMAGE_MULTIPLIER: I32F32 = I32F32::lit("0.50");
const SOLDIER_VENOM_DAMAGE_MULTIPLIER: I32F32 = I32F32::lit("0.50");
const SOLDIER_ATTACK_NOISE: I32F32 = I32F32::lit("3");

const SOLDIER_VETERAN_ATTACK_RANGE: I32F32 = I32F32::lit("5.5");
const SOLDIER_VETERAN_ATTACK_SPEED: I32F32 = I32F32::lit("2.5");
const SOLDIER_VETERAN_ATTACK_DAMAGE: I32F32 = I32F32::lit("26");
const SOLDIER_VETERAN_XP_THRESHOLD: u32 = 90;

#[derive(Component, Default)]
pub struct Soldier;

#[derive(Component, Clone, Copy, Default)]
pub struct ReplicatedUnitId(pub u64);

#[derive(Component, Clone, Copy, Default)]
pub struct AttackNoise(pub I32F32);

#[derive(Component, Clone, Copy, Default)]
pub struct Experience {
    pub value: u32,
}

#[derive(Component, Clone, Copy, Default)]
pub struct Veteran(pub bool);

#[derive(Bundle)]
pub struct SoldierBundle {
    pub unit: Soldier,
    pub replicated_id: ReplicatedUnitId,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub attack_noise: AttackNoise,
    pub experience: Experience,
    pub veteran: Veteran,
}

impl Default for SoldierBundle {
    fn default() -> Self {
        Self {
            unit: Soldier,
            replicated_id: ReplicatedUnitId::default(),
            position: SimPosition {
                x: I32F32::ZERO,
                y: I32F32::ZERO,
            },
            health: Health::full(SOLDIER_HP),
            stats: UnitStats {
                move_speed: SOLDIER_MOVE_SPEED,
                attack_range: SOLDIER_ATTACK_RANGE,
                attack_damage: SOLDIER_ATTACK_DAMAGE,
                attack_speed: SOLDIER_ATTACK_SPEED,
                watch_range: SOLDIER_WATCH_RANGE,
            },
            attack_noise: AttackNoise(SOLDIER_ATTACK_NOISE),
            experience: Experience::default(),
            veteran: Veteran::default(),
        }
    }
}

#[derive(Resource, Default)]
pub struct NextReplicatedUnitId(pub u64);

#[derive(Event, Clone, Copy)]
pub struct SpawnSoldierEvent {
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy)]
pub struct GainSoldierExperienceEvent {
    pub entity: Entity,
    pub amount: u32,
}

#[derive(Event, Clone, Copy)]
pub struct SoldierFiredEvent {
    pub entity: Entity,
}

fn spawn_soldier_system(
    mut commands: Commands,
    mut events: EventReader<SpawnSoldierEvent>,
    mut next_id: ResMut<NextReplicatedUnitId>,
) {
    for ev in events.read() {
        let mut bundle = SoldierBundle::default();
        bundle.position = ev.position;
        bundle.replicated_id = ReplicatedUnitId(next_id.0);
        next_id.0 = next_id.0.wrapping_add(1);
        commands.spawn(bundle);
    }
}

fn gain_soldier_experience_system(
    mut events: EventReader<GainSoldierExperienceEvent>,
    mut units: Query<(&mut Experience, &mut Veteran, &mut UnitStats), With<Soldier>>,
) {
    for ev in events.read() {
        let Ok((mut xp, mut veteran, mut stats)) = units.get_mut(ev.entity) else {
            continue;
        };

        xp.value = xp.value.saturating_add(ev.amount);

        if !veteran.0 && xp.value >= SOLDIER_VETERAN_XP_THRESHOLD {
            veteran.0 = true;
            stats.attack_range = SOLDIER_VETERAN_ATTACK_RANGE;
            stats.attack_speed = SOLDIER_VETERAN_ATTACK_SPEED;
            stats.attack_damage = SOLDIER_VETERAN_ATTACK_DAMAGE;
        }
    }
}

fn soldier_fire_noise_system(
    mut events: EventReader<SoldierFiredEvent>,
    soldiers: Query<(&SimPosition, &AttackNoise), With<Soldier>>,
    mut noise_writer: EventWriter<NoiseEmittedEvent>,
) {
    for ev in events.read() {
        let Ok((position, noise)) = soldiers.get(ev.entity) else {
            continue;
        };

        noise_writer.send(NoiseEmittedEvent {
            source: ev.entity,
            position: *position,
            amount: noise.0,
        });
    }
}

fn apply_soldier_damage_system(
    mut damage_events: EventReader<IncomingDamageEvent>,
    mut soldiers: Query<&mut Health, With<Soldier>>,
    mut killed_writer: EventWriter<EntityKilledEvent>,
) {
    for ev in damage_events.read() {
        let Ok(mut health) = soldiers.get_mut(ev.target) else {
            continue;
        };

        if health.current <= I32F32::ZERO {
            continue;
        }

        let multiplier = match ev.damage_type {
            DamageType::Standard => SOLDIER_STANDARD_DAMAGE_MULTIPLIER,
            DamageType::Fire => SOLDIER_FIRE_DAMAGE_MULTIPLIER,
            DamageType::Venom => SOLDIER_VENOM_DAMAGE_MULTIPLIER,
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

fn soldier_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    units: Query<
        (
            &ReplicatedUnitId,
            &SimPosition,
            &Health,
            &UnitStats,
            &AttackNoise,
            &Experience,
            &Veteran,
        ),
        With<Soldier>,
    >,
) {
    for (replicated_id, pos, hp, stats, noise, xp, veteran) in &units {
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
        checksum.accumulate(noise.0.to_bits() as u64);
        checksum.accumulate(xp.value as u64);
        checksum.accumulate(u64::from(veteran.0));
    }
}

pub struct SoldierPlugin;

impl Plugin for SoldierPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NextReplicatedUnitId>()
            .add_event::<SpawnSoldierEvent>()
            .add_event::<GainSoldierExperienceEvent>()
            .add_event::<SoldierFiredEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_soldier_system,
                    gain_soldier_experience_system,
                    soldier_fire_noise_system,
                    apply_soldier_damage_system,
                    soldier_checksum_system,
                )
                    .chain(),
            );
    }
}
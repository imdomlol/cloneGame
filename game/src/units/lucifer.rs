// Sources: vault/units/lucifer.md, vault/buildings/engineering_center.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{
    DamageType, EntityKilledEvent, Health, IncomingDamageEvent, NoiseEmittedEvent, SimChecksumState,
    SimHz, SimPosition, UnitStats,
};

const LUCIFER_HP: I32F32 = I32F32::lit("500");
const LUCIFER_MOVE_SPEED: I32F32 = I32F32::lit("1.8");
const LUCIFER_ATTACK_RANGE: I32F32 = I32F32::lit("3.5");
const LUCIFER_ATTACK_SPEED: I32F32 = I32F32::lit("2");
const LUCIFER_ATTACK_DAMAGE: I32F32 = I32F32::lit("24");
const LUCIFER_WATCH_RANGE: I32F32 = I32F32::lit("6");

const LUCIFER_BURN_DAMAGE: I32F32 = I32F32::lit("4");
const LUCIFER_ATTACK_NOISE: I32F32 = I32F32::lit("10");
const LUCIFER_PHYSICAL_DAMAGE_REDUCTION: I32F32 = I32F32::lit("0.25");
const LUCIFER_VENOM_RESISTANCE: I32F32 = I32F32::lit("0.65");
const LUCIFER_FIRE_RESISTANCE: I32F32 = I32F32::lit("1");
const LUCIFER_STANDARD_DAMAGE_MULTIPLIER: I32F32 = I32F32::lit("0.75");
const LUCIFER_VENOM_DAMAGE_MULTIPLIER: I32F32 = I32F32::lit("0.35");
const LUCIFER_FIRE_DAMAGE_MULTIPLIER: I32F32 = I32F32::ZERO;
const LUCIFER_REGEN_PER_SECOND: I32F32 = I32F32::lit("40");

#[derive(Component, Default)]
pub struct Lucifer;

#[derive(Component, Clone, Copy, Default)]
pub struct ReplicatedUnitId(pub u64);

#[derive(Component, Clone, Copy, Default)]
pub struct AttackNoise(pub I32F32);

#[derive(Component, Clone, Copy, Default)]
pub struct BurnDamagePerHit(pub I32F32);

#[derive(Component, Clone, Copy, Default)]
pub struct PhysicalDamageReduction(pub I32F32);

#[derive(Component, Clone, Copy, Default)]
pub struct VenomResistance(pub I32F32);

#[derive(Component, Clone, Copy, Default)]
pub struct FireResistance(pub I32F32);

#[derive(Component, Clone, Copy, Default)]
pub struct RegenerationPerSecond(pub I32F32);

#[derive(Component, Clone, Copy, Default)]
pub struct CarriesExplosiveBarrel(pub bool);

#[derive(Component, Clone, Copy, Default)]
pub struct FlameConeHitsMultipleTargets(pub bool);

#[derive(Component, Clone, Copy, Default)]
pub struct AttackStartupDelay(pub bool);

#[derive(Component, Clone, Copy, Default)]
pub struct InstantRetargetNoReload(pub bool);

#[derive(Bundle)]
pub struct LuciferBundle {
    pub unit: Lucifer,
    pub replicated_id: ReplicatedUnitId,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub attack_noise: AttackNoise,
    pub burn_damage_per_hit: BurnDamagePerHit,
    pub physical_damage_reduction: PhysicalDamageReduction,
    pub venom_resistance: VenomResistance,
    pub fire_resistance: FireResistance,
    pub regeneration_per_second: RegenerationPerSecond,
    pub carries_explosive_barrel: CarriesExplosiveBarrel,
    pub flame_cone_hits_multiple_targets: FlameConeHitsMultipleTargets,
    pub attack_startup_delay: AttackStartupDelay,
    pub instant_retarget_no_reload: InstantRetargetNoReload,
}

impl Default for LuciferBundle {
    fn default() -> Self {
        Self {
            unit: Lucifer,
            replicated_id: ReplicatedUnitId::default(),
            position: SimPosition {
                x: I32F32::ZERO,
                y: I32F32::ZERO,
            },
            health: Health::full(LUCIFER_HP),
            stats: UnitStats {
                move_speed: LUCIFER_MOVE_SPEED,
                attack_range: LUCIFER_ATTACK_RANGE,
                attack_damage: LUCIFER_ATTACK_DAMAGE,
                attack_speed: LUCIFER_ATTACK_SPEED,
                watch_range: LUCIFER_WATCH_RANGE,
            },
            attack_noise: AttackNoise(LUCIFER_ATTACK_NOISE),
            burn_damage_per_hit: BurnDamagePerHit(LUCIFER_BURN_DAMAGE),
            physical_damage_reduction: PhysicalDamageReduction(LUCIFER_PHYSICAL_DAMAGE_REDUCTION),
            venom_resistance: VenomResistance(LUCIFER_VENOM_RESISTANCE),
            fire_resistance: FireResistance(LUCIFER_FIRE_RESISTANCE),
            regeneration_per_second: RegenerationPerSecond(LUCIFER_REGEN_PER_SECOND),
            carries_explosive_barrel: CarriesExplosiveBarrel::default(),
            flame_cone_hits_multiple_targets: FlameConeHitsMultipleTargets(true),
            attack_startup_delay: AttackStartupDelay(true),
            instant_retarget_no_reload: InstantRetargetNoReload(true),
        }
    }
}

#[derive(Resource, Default)]
pub struct NextReplicatedUnitId(pub u64);

#[derive(Event, Clone, Copy)]
pub struct SpawnLuciferEvent {
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy)]
pub struct LuciferFiredEvent {
    pub entity: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct LuciferBurnAppliedEvent {
    pub source: Entity,
    pub target: Entity,
    pub amount: I32F32,
}

#[derive(Event, Clone, Copy)]
pub struct LuciferBarrelExplodedEvent {
    pub source: Entity,
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy)]
pub struct SetLuciferBarrelCarriedEvent {
    pub entity: Entity,
    pub carrying: bool,
}

fn spawn_lucifer_system(
    mut commands: Commands,
    mut events: EventReader<SpawnLuciferEvent>,
    mut next_id: ResMut<NextReplicatedUnitId>,
) {
    for ev in events.read() {
        let mut bundle = LuciferBundle::default();
        bundle.position = ev.position;
        bundle.replicated_id = ReplicatedUnitId(next_id.0);
        next_id.0 = next_id.0.wrapping_add(1);
        commands.spawn(bundle);
    }
}

fn lucifer_fire_noise_system(
    mut events: EventReader<LuciferFiredEvent>,
    lucifers: Query<(&SimPosition, &AttackNoise), With<Lucifer>>,
    mut noise_writer: EventWriter<NoiseEmittedEvent>,
) {
    for ev in events.read() {
        let Ok((position, noise)) = lucifers.get(ev.entity) else {
            continue;
        };

        noise_writer.send(NoiseEmittedEvent {
            source: ev.entity,
            position: *position,
            amount: noise.0,
        });
    }
}

fn apply_lucifer_damage_system(
    mut damage_events: EventReader<IncomingDamageEvent>,
    mut lucifers: Query<&mut Health, With<Lucifer>>,
    mut killed_writer: EventWriter<EntityKilledEvent>,
) {
    for ev in damage_events.read() {
        let Ok(mut health) = lucifers.get_mut(ev.target) else {
            continue;
        };

        if health.current <= I32F32::ZERO {
            continue;
        }

        let multiplier = match ev.damage_type {
            DamageType::Standard => LUCIFER_STANDARD_DAMAGE_MULTIPLIER,
            DamageType::Fire => LUCIFER_FIRE_DAMAGE_MULTIPLIER,
            DamageType::Venom => LUCIFER_VENOM_DAMAGE_MULTIPLIER,
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

fn lucifer_regeneration_system(
    sim_hz: Res<SimHz>,
    mut lucifers: Query<(&mut Health, &RegenerationPerSecond), With<Lucifer>>,
) {
    if sim_hz.0 <= I32F32::ZERO {
        return;
    }

    for (mut health, regen_per_second) in &mut lucifers {
        if health.current >= health.max {
            continue;
        }

        let regen_per_tick = regen_per_second.0 / sim_hz.0;
        let healed = health.current + regen_per_tick;
        health.current = if healed > health.max { health.max } else { healed };
    }
}

fn set_lucifer_barrel_state_system(
    mut events: EventReader<SetLuciferBarrelCarriedEvent>,
    mut lucifers: Query<&mut CarriesExplosiveBarrel, With<Lucifer>>,
) {
    for ev in events.read() {
        let Ok(mut carrying) = lucifers.get_mut(ev.entity) else {
            continue;
        };
        carrying.0 = ev.carrying;
    }
}

fn lucifer_death_barrel_explosion_system(
    mut death_events: EventReader<EntityKilledEvent>,
    lucifers: Query<(&SimPosition, &CarriesExplosiveBarrel), With<Lucifer>>,
    mut explosion_writer: EventWriter<LuciferBarrelExplodedEvent>,
) {
    for ev in death_events.read() {
        let Ok((position, carrying)) = lucifers.get(ev.entity) else {
            continue;
        };

        if carrying.0 {
            explosion_writer.send(LuciferBarrelExplodedEvent {
                source: ev.entity,
                position: *position,
            });
        }
    }
}

fn lucifer_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    units: Query<
        (
            &ReplicatedUnitId,
            &SimPosition,
            &Health,
            &UnitStats,
            &AttackNoise,
            &BurnDamagePerHit,
            &PhysicalDamageReduction,
            &VenomResistance,
            &FireResistance,
            &RegenerationPerSecond,
            &CarriesExplosiveBarrel,
            &FlameConeHitsMultipleTargets,
            &AttackStartupDelay,
            &InstantRetargetNoReload,
        ),
        With<Lucifer>,
    >,
) {
    for (
        replicated_id,
        pos,
        hp,
        stats,
        noise,
        burn_damage,
        physical_reduction,
        venom_resistance,
        fire_resistance,
        regen_per_second,
        carries_barrel,
        cone_hits_multiple,
        startup_delay,
        instant_retarget,
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
        checksum.accumulate(noise.0.to_bits() as u64);
        checksum.accumulate(burn_damage.0.to_bits() as u64);
        checksum.accumulate(physical_reduction.0.to_bits() as u64);
        checksum.accumulate(venom_resistance.0.to_bits() as u64);
        checksum.accumulate(fire_resistance.0.to_bits() as u64);
        checksum.accumulate(regen_per_second.0.to_bits() as u64);
        checksum.accumulate(u64::from(carries_barrel.0));
        checksum.accumulate(u64::from(cone_hits_multiple.0));
        checksum.accumulate(u64::from(startup_delay.0));
        checksum.accumulate(u64::from(instant_retarget.0));
    }
}

pub struct LuciferPlugin;

impl Plugin for LuciferPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NextReplicatedUnitId>()
            .add_event::<SpawnLuciferEvent>()
            .add_event::<LuciferFiredEvent>()
            .add_event::<LuciferBurnAppliedEvent>()
            .add_event::<LuciferBarrelExplodedEvent>()
            .add_event::<SetLuciferBarrelCarriedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_lucifer_system,
                    lucifer_fire_noise_system,
                    apply_lucifer_damage_system,
                    lucifer_regeneration_system,
                    set_lucifer_barrel_state_system,
                    lucifer_death_barrel_explosion_system,
                    lucifer_checksum_system,
                )
                    .chain(),
            );
    }
}
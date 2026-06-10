// Sources: vault/daytime_entity_pages/circuit_bee.md, vault/entity_pages/entity.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{
    DamageType, EntityKilledEvent, Health, IncomingDamageEvent, SimChecksumState, SimHz,
    SimPosition, SimTick, UnitStats,
};

pub const CIRCUIT_BEE_ID: &str = "circuit_bee";
pub const CIRCUIT_BEE_NAME: &str = "Circuit Bee";
pub const CIRCUIT_BEE_SCIENTIFIC_NAME: &str = "Crabro-coruscus";
pub const CIRCUIT_BEE_POWER_LEVEL: u8 = 1;
pub const CIRCUIT_BEE_MAX_SPAWNED: usize = 6;
pub const CIRCUIT_BEE_ATTACK_DAMAGE: I32F32 = I32F32::lit("10");
pub const CIRCUIT_BEE_ATTACK_SPEED: I32F32 = I32F32::lit("2.5");
pub const CIRCUIT_BEE_DPS: I32F32 = I32F32::lit("25");
pub const CIRCUIT_BEE_LEAVE_MINUTE_OF_DAY: u16 = 1437;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CircuitBeeOccurrence {
    pub moon: CircuitBeeMoon,
    pub base_spawn_chance: I32F32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CircuitBeeMoon {
    March,
    Assurance,
    Vow,
    Artifice,
    Experimentation,
    Adamance,
}

pub const CIRCUIT_BEE_OCCURRENCE: [CircuitBeeOccurrence; 6] = [
    CircuitBeeOccurrence {
        moon: CircuitBeeMoon::March,
        base_spawn_chance: I32F32::lit("36.27"),
    },
    CircuitBeeOccurrence {
        moon: CircuitBeeMoon::Assurance,
        base_spawn_chance: I32F32::lit("21.5"),
    },
    CircuitBeeOccurrence {
        moon: CircuitBeeMoon::Vow,
        base_spawn_chance: I32F32::lit("18.65"),
    },
    CircuitBeeOccurrence {
        moon: CircuitBeeMoon::Artifice,
        base_spawn_chance: I32F32::lit("15.63"),
    },
    CircuitBeeOccurrence {
        moon: CircuitBeeMoon::Experimentation,
        base_spawn_chance: I32F32::lit("14.86"),
    },
    CircuitBeeOccurrence {
        moon: CircuitBeeMoon::Adamance,
        base_spawn_chance: I32F32::lit("10.53"),
    },
];

pub struct CircuitBeePlugin;

impl Plugin for CircuitBeePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnCircuitBeeEvent>()
            .add_event::<CircuitBeeHivePickedUpEvent>()
            .add_event::<CircuitBeeIgnoredDamageEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_circuit_bees,
                    circuit_bee_detect_hive_threats,
                    circuit_bee_return_when_threat_far,
                    circuit_bee_continue_attacking_killed_players,
                    circuit_bee_hive_picked_up,
                    circuit_bee_attack_targets,
                    circuit_bee_ignore_damage,
                    circuit_bee_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CircuitBee;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CircuitBeeHive;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CircuitBeeThreat;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CircuitBeeKilledBody;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CircuitBeeImmune;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CircuitBeeHiveAnchor {
    pub position: SimPosition,
    pub guard_radius: I32F32,
    pub return_radius: I32F32,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CircuitBeeAttackClock {
    pub ticks_until_attack: u32,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum CircuitBeeState {
    Territorial,
    Roaming,
    Chasing,
    ReturningToHive,
    AttackingKilledBody,
}

impl Default for CircuitBeeState {
    fn default() -> Self {
        Self::Territorial
    }
}

#[derive(Bundle)]
pub struct CircuitBeeBundle {
    pub name: Name,
    pub bee: CircuitBee,
    pub immune: CircuitBeeImmune,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub hive_anchor: CircuitBeeHiveAnchor,
    pub state: CircuitBeeState,
    pub attack_clock: CircuitBeeAttackClock,
}

impl CircuitBeeBundle {
    pub fn new(event: SpawnCircuitBeeEvent) -> Self {
        Self {
            name: Name::new(CIRCUIT_BEE_NAME),
            bee: CircuitBee,
            immune: CircuitBeeImmune,
            position: event.position,
            health: Health::full(I32F32::lit("0")),
            stats: UnitStats {
                move_speed: event.move_speed,
                attack_range: event.attack_range,
                attack_damage: CIRCUIT_BEE_ATTACK_DAMAGE,
                attack_speed: CIRCUIT_BEE_ATTACK_SPEED,
                watch_range: event.watch_range,
            },
            hive_anchor: CircuitBeeHiveAnchor {
                position: event.hive_position,
                guard_radius: event.guard_radius,
                return_radius: event.return_radius,
            },
            state: CircuitBeeState::Territorial,
            attack_clock: CircuitBeeAttackClock {
                ticks_until_attack: 0,
            },
        }
    }
}

#[derive(Bundle)]
pub struct CircuitBeeHiveBundle {
    pub name: Name,
    pub hive: CircuitBeeHive,
    pub position: SimPosition,
}

impl CircuitBeeHiveBundle {
    pub fn new(position: SimPosition) -> Self {
        Self {
            name: Name::new("Circuit Bee Hive"),
            hive: CircuitBeeHive,
            position,
        }
    }
}

#[derive(Event, Clone, Copy, Debug)]
pub struct SpawnCircuitBeeEvent {
    pub position: SimPosition,
    pub hive_position: SimPosition,
    pub guard_radius: I32F32,
    pub return_radius: I32F32,
    pub attack_range: I32F32,
    pub watch_range: I32F32,
    pub move_speed: I32F32,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct CircuitBeeHivePickedUpEvent {
    pub hive_position: SimPosition,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct CircuitBeeIgnoredDamageEvent {
    pub bee: Entity,
    pub source: Entity,
    pub damage_type: DamageType,
    pub raw_amount: I32F32,
}

fn spawn_circuit_bees(
    mut commands: Commands,
    mut events: EventReader<SpawnCircuitBeeEvent>,
    bees: Query<(), With<CircuitBee>>,
) {
    let mut spawned_count = bees.iter().count();

    for event in events.read() {
        if spawned_count >= CIRCUIT_BEE_MAX_SPAWNED {
            break;
        }

        commands.spawn(CircuitBeeBundle::new(*event));
        spawned_count += 1;
    }
}

fn circuit_bee_detect_hive_threats(
    mut bees: Query<(&mut CircuitBeeState, &CircuitBeeHiveAnchor), With<CircuitBee>>,
    threats: Query<&SimPosition, With<CircuitBeeThreat>>,
) {
    for (mut state, hive_anchor) in bees.iter_mut() {
        if *state == CircuitBeeState::AttackingKilledBody {
            continue;
        }

        let guard_radius_squared = hive_anchor.guard_radius * hive_anchor.guard_radius;
        let mut threat_near_hive = false;

        for threat_position in threats.iter() {
            if distance_squared(*threat_position, hive_anchor.position) <= guard_radius_squared {
                threat_near_hive = true;
            }
        }

        if threat_near_hive {
            *state = CircuitBeeState::Chasing;
        }
    }
}

fn circuit_bee_return_when_threat_far(
    mut bees: Query<(&mut CircuitBeeState, &CircuitBeeHiveAnchor), With<CircuitBee>>,
    threats: Query<&SimPosition, With<CircuitBeeThreat>>,
) {
    for (mut state, hive_anchor) in bees.iter_mut() {
        if *state != CircuitBeeState::Chasing {
            continue;
        }

        let return_radius_squared = hive_anchor.return_radius * hive_anchor.return_radius;
        let mut threat_still_close = false;

        for threat_position in threats.iter() {
            if distance_squared(*threat_position, hive_anchor.position) <= return_radius_squared {
                threat_still_close = true;
            }
        }

        if !threat_still_close {
            *state = CircuitBeeState::ReturningToHive;
        }
    }
}

fn circuit_bee_continue_attacking_killed_players(
    mut killed_events: EventReader<EntityKilledEvent>,
    mut bees: Query<&mut CircuitBeeState, With<CircuitBee>>,
) {
    let mut bee_kill_happened = false;

    for killed in killed_events.read() {
        if killed.exp_reward == I32F32::lit("0") {
            bee_kill_happened = true;
        }
    }

    if !bee_kill_happened {
        return;
    }

    for mut state in bees.iter_mut() {
        *state = CircuitBeeState::AttackingKilledBody;
    }
}

fn circuit_bee_hive_picked_up(
    mut events: EventReader<CircuitBeeHivePickedUpEvent>,
    mut bees: Query<(&mut CircuitBeeState, &CircuitBeeHiveAnchor), With<CircuitBee>>,
) {
    for event in events.read() {
        for (mut state, hive_anchor) in bees.iter_mut() {
            if hive_anchor.position == event.hive_position {
                *state = CircuitBeeState::Roaming;
            }
        }
    }
}

fn circuit_bee_attack_targets(
    mut damage_events: EventWriter<IncomingDamageEvent>,
    sim_hz: Res<SimHz>,
    mut bees: Query<(
        Entity,
        &SimPosition,
        &UnitStats,
        &CircuitBeeState,
        &mut CircuitBeeAttackClock,
    ), With<CircuitBee>>,
    targets: Query<(Entity, &SimPosition), Or<(With<CircuitBeeThreat>, With<CircuitBeeKilledBody>)>>,
) {
    let attack_period = attack_period_ticks(sim_hz.0);

    for (bee_entity, bee_position, stats, state, mut attack_clock) in bees.iter_mut() {
        if *state != CircuitBeeState::Chasing && *state != CircuitBeeState::AttackingKilledBody {
            continue;
        }

        if attack_clock.ticks_until_attack > 0 {
            attack_clock.ticks_until_attack -= 1;
            continue;
        }

        let attack_range_squared = stats.attack_range * stats.attack_range;

        for (target_entity, target_position) in targets.iter() {
            if distance_squared(*bee_position, *target_position) <= attack_range_squared {
                damage_events.send(IncomingDamageEvent {
                    target: target_entity,
                    raw_amount: stats.attack_damage,
                    damage_type: DamageType::Standard,
                    source: bee_entity,
                });
                attack_clock.ticks_until_attack = attack_period;
                break;
            }
        }
    }
}

fn circuit_bee_ignore_damage(
    mut damage_events: EventReader<IncomingDamageEvent>,
    mut ignored_events: EventWriter<CircuitBeeIgnoredDamageEvent>,
    bees: Query<(), With<CircuitBeeImmune>>,
) {
    for event in damage_events.read() {
        if bees.get(event.target).is_ok() {
            ignored_events.send(CircuitBeeIgnoredDamageEvent {
                bee: event.target,
                source: event.source,
                damage_type: event.damage_type,
                raw_amount: event.raw_amount,
            });
        }
    }
}

fn circuit_bee_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    bees: Query<(
        &SimPosition,
        &Health,
        &UnitStats,
        &CircuitBeeHiveAnchor,
        &CircuitBeeState,
        &CircuitBeeAttackClock,
    ), With<CircuitBee>>,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(CIRCUIT_BEE_POWER_LEVEL as u64);
    checksum.accumulate(CIRCUIT_BEE_MAX_SPAWNED as u64);
    checksum.accumulate(CIRCUIT_BEE_ATTACK_DAMAGE.to_bits() as u64);
    checksum.accumulate(CIRCUIT_BEE_ATTACK_SPEED.to_bits() as u64);
    checksum.accumulate(CIRCUIT_BEE_DPS.to_bits() as u64);
    checksum.accumulate(CIRCUIT_BEE_LEAVE_MINUTE_OF_DAY as u64);

    for occurrence in CIRCUIT_BEE_OCCURRENCE {
        checksum.accumulate(occurrence.moon as u64);
        checksum.accumulate(occurrence.base_spawn_chance.to_bits() as u64);
    }

    for (position, health, stats, hive_anchor, state, attack_clock) in bees.iter() {
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);
        checksum.accumulate(hive_anchor.position.x.to_bits() as u64);
        checksum.accumulate(hive_anchor.position.y.to_bits() as u64);
        checksum.accumulate(hive_anchor.guard_radius.to_bits() as u64);
        checksum.accumulate(hive_anchor.return_radius.to_bits() as u64);
        checksum.accumulate(*state as u64);
        checksum.accumulate(attack_clock.ticks_until_attack as u64);
    }
}

fn distance_squared(a: SimPosition, b: SimPosition) -> I32F32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    dx * dx + dy * dy
}

fn attack_period_ticks(sim_hz: I32F32) -> u32 {
    let period = sim_hz / CIRCUIT_BEE_ATTACK_SPEED;
    let ticks = period.ceil().to_num::<u32>();

    if ticks == 0 {
        1
    } else {
        ticks
    }
}
// Sources: vault/outdoor_entity_pages/earth_leviathan.md
use bevy::prelude::*;

use crate::sim::{
    DamageType, Health, IncomingDamageEvent, SimChecksumState, SimHz, SimPosition, SimTick,
    UnitStats,
};

pub const EARTH_LEVIATHAN_ID: &str = "earth_leviathan";
pub const EARTH_LEVIATHAN_NAME: &str = "Earth Leviathan";
pub const EARTH_LEVIATHAN_TYPE: &str = "outdoor_entity_pages";
pub const EARTH_LEVIATHAN_SUBTYPE: &str = "creature";
pub const EARTH_LEVIATHAN_SOURCE_URL: &str =
    "https://lethal-company.fandom.com/wiki/Earth_Leviathan";
pub const EARTH_LEVIATHAN_SOURCE_REVISION: u32 = 21360;
pub const EARTH_LEVIATHAN_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const EARTH_LEVIATHAN_CONFIDENCE_BASIS_POINTS: u16 = 93;

pub const EARTH_LEVIATHAN_DWELLS: &str = "Outside (Night-time/Eclipse)";
pub const EARTH_LEVIATHAN_SCIENTIFIC_NAME: &str = "Hemibdella-gigantis";
pub const EARTH_LEVIATHAN_ATTACK_DAMAGE: &str = "Instant Kill";
pub const EARTH_LEVIATHAN_CONTACT_DAMAGE: &str = "Instant Kill";
pub const EARTH_LEVIATHAN_POWER_LEVEL: u32 = 2;
pub const EARTH_LEVIATHAN_MAX_SPAWNED: usize = 3;
pub const EARTH_LEVIATHAN_RADAR_PIP_SIZE: &str = "Colossal";
pub const EARTH_LEVIATHAN_SHOVEL_HP: &str = "Immune";
pub const EARTH_LEVIATHAN_SHOCK_RESPONSE: &str = "Immune";
pub const EARTH_LEVIATHAN_INTERNAL_NAME: &str = "Worm";

pub const EARTH_LEVIATHAN_MIN_WARNING_SECONDS_AFTER_VERSION_50: u32 = 1;
pub const EARTH_LEVIATHAN_TRACKING_SPEED: fixed::types::I32F32 = fixed::types::I32F32::lit("8");
pub const EARTH_LEVIATHAN_ROAM_SPEED: fixed::types::I32F32 = fixed::types::I32F32::lit("3");
pub const EARTH_LEVIATHAN_ATTACK_ZONE_RADIUS: fixed::types::I32F32 =
    fixed::types::I32F32::lit("6");
pub const EARTH_LEVIATHAN_WATCH_RANGE: fixed::types::I32F32 = fixed::types::I32F32::lit("80");
pub const EARTH_LEVIATHAN_NO_DAMAGE: fixed::types::I32F32 = fixed::types::I32F32::lit("0");

pub const EARTH_LEVIATHAN_FRONTMATTER_BEHAVIOR: [&str; 3] =
    ["Roaming", "Tracking", "Emerging"];

pub const EARTH_LEVIATHAN_BEHAVIORAL_MECHANICS: [EarthLeviathanBehaviorRule; 9] = [
    EarthLeviathanBehaviorRule {
        condition: "underground",
        outcome: "the Earth Leviathan roams the area searching for a target",
    },
    EarthLeviathanBehaviorRule {
        condition: "it locates a target",
        outcome: "it centers itself on that target before emerging",
    },
    EarthLeviathanBehaviorRule {
        condition: "the attack warning begins",
        outcome: "the creature enters a tracking phase before surfacing",
    },
    EarthLeviathanBehaviorRule {
        condition: "black particles and debris appear",
        outcome: "the emergence phase has started",
    },
    EarthLeviathanBehaviorRule {
        condition: "the emergence completes",
        outcome: "anything within the attack zone is killed instantly",
    },
    EarthLeviathanBehaviorRule {
        condition: "the warning-to-emergence delay occurs after Version 50",
        outcome: "the minimum delay is 1 second",
    },
    EarthLeviathanBehaviorRule {
        condition: "a player attempts to damage it with a shovel",
        outcome: "the attack is ineffective",
    },
    EarthLeviathanBehaviorRule {
        condition: "the Earth Leviathan is subjected to shock-based effects",
        outcome: "it remains immune",
    },
    EarthLeviathanBehaviorRule {
        condition: "the creature is active in a room",
        outcome: "no more than 3 can be spawned at once",
    },
];

pub struct EarthLeviathanPlugin;

impl Plugin for EarthLeviathanPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnEarthLeviathanEvent>()
            .add_event::<EarthLeviathanStateChangedEvent>()
            .add_event::<EarthLeviathanTargetLocatedEvent>()
            .add_event::<EarthLeviathanAttackWarningBegunEvent>()
            .add_event::<EarthLeviathanEmergenceStartedEvent>()
            .add_event::<EarthLeviathanInstantKillEvent>()
            .add_event::<EarthLeviathanShovelAttackIgnoredEvent>()
            .add_event::<EarthLeviathanShockIgnoredEvent>()
            .add_event::<EarthLeviathanSpawnLimitReachedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    earth_leviathan_spawn_from_events,
                    earth_leviathan_acquire_target,
                    earth_leviathan_begin_tracking_from_warning,
                    earth_leviathan_track_target,
                    earth_leviathan_start_emergence,
                    earth_leviathan_complete_emergence,
                    earth_leviathan_ignore_shovel_attacks,
                    earth_leviathan_ignore_shock,
                    earth_leviathan_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EarthLeviathanBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EarthLeviathan {
    pub stable_id: u64,
    pub active_in_room: bool,
    pub post_version_50: bool,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct EarthLeviathanTarget {
    pub has_target: bool,
    pub target_entity: Entity,
    pub target_stable_id: u64,
    pub centered_on_target: bool,
    pub target_position: SimPosition,
}

impl Default for EarthLeviathanTarget {
    fn default() -> Self {
        Self {
            has_target: false,
            target_entity: Entity::PLACEHOLDER,
            target_stable_id: 0,
            centered_on_target: false,
            target_position: SimPosition {
                x: fixed::types::I32F32::ZERO,
                y: fixed::types::I32F32::ZERO,
            },
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EarthLeviathanVictimSensor {
    pub stable_id: u64,
    pub is_alive: bool,
    pub detectable: bool,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct EarthLeviathanTimers {
    pub warning_ticks_remaining: u32,
    pub emergence_ticks_remaining: u32,
}

impl Default for EarthLeviathanTimers {
    fn default() -> Self {
        Self {
            warning_ticks_remaining: 0,
            emergence_ticks_remaining: 0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EarthLeviathanState {
    #[default]
    Roaming,
    Tracking,
    Emerging,
    Surfaced,
}

#[derive(Bundle)]
pub struct EarthLeviathanBundle {
    pub name: Name,
    pub leviathan: EarthLeviathan,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub state: EarthLeviathanState,
    pub target: EarthLeviathanTarget,
    pub timers: EarthLeviathanTimers,
}

impl EarthLeviathanBundle {
    pub fn new(event: SpawnEarthLeviathanEvent) -> Self {
        Self {
            name: Name::new(EARTH_LEVIATHAN_NAME),
            leviathan: EarthLeviathan {
                stable_id: event.stable_id,
                active_in_room: event.active_in_room,
                post_version_50: event.post_version_50,
            },
            position: event.position,
            health: Health::full(EARTH_LEVIATHAN_NO_DAMAGE),
            stats: UnitStats {
                move_speed: EARTH_LEVIATHAN_ROAM_SPEED,
                attack_range: EARTH_LEVIATHAN_ATTACK_ZONE_RADIUS,
                attack_damage: EARTH_LEVIATHAN_NO_DAMAGE,
                attack_speed: fixed::types::I32F32::ONE,
                watch_range: EARTH_LEVIATHAN_WATCH_RANGE,
            },
            state: EarthLeviathanState::Roaming,
            target: EarthLeviathanTarget {
                target_position: event.position,
                ..Default::default()
            },
            timers: EarthLeviathanTimers::default(),
        }
    }
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpawnEarthLeviathanEvent {
    pub stable_id: u64,
    pub position: SimPosition,
    pub active_in_room: bool,
    pub post_version_50: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct EarthLeviathanStateChangedEvent {
    pub leviathan: Entity,
    pub from: EarthLeviathanState,
    pub to: EarthLeviathanState,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct EarthLeviathanTargetLocatedEvent {
    pub leviathan: Entity,
    pub target: Entity,
    pub target_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct EarthLeviathanAttackWarningBegunEvent {
    pub leviathan: Entity,
    pub target: Entity,
    pub target_stable_id: u64,
    pub warning_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct EarthLeviathanEmergenceStartedEvent {
    pub leviathan: Entity,
    pub target: Entity,
    pub target_stable_id: u64,
    pub emergence_ticks: u32,
    pub black_particles_and_debris: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct EarthLeviathanInstantKillEvent {
    pub leviathan: Entity,
    pub victim: Entity,
    pub victim_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct EarthLeviathanShovelAttackIgnoredEvent {
    pub leviathan: Entity,
    pub source: Entity,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct EarthLeviathanShockIgnoredEvent {
    pub leviathan: Entity,
    pub source: Entity,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct EarthLeviathanSpawnLimitReachedEvent {
    pub requested_stable_id: u64,
    pub max_spawned: usize,
}

fn earth_leviathan_spawn_from_events(
    mut commands: Commands,
    mut spawn_events: EventReader<SpawnEarthLeviathanEvent>,
    mut limit_events: EventWriter<EarthLeviathanSpawnLimitReachedEvent>,
    leviathans: Query<(), With<EarthLeviathan>>,
) {
    let mut spawned_count = leviathans.iter().count();

    for event in spawn_events.read() {
        if spawned_count >= EARTH_LEVIATHAN_MAX_SPAWNED {
            limit_events.send(EarthLeviathanSpawnLimitReachedEvent {
                requested_stable_id: event.stable_id,
                max_spawned: EARTH_LEVIATHAN_MAX_SPAWNED,
            });
            continue;
        }

        commands.spawn(EarthLeviathanBundle::new(*event));
        spawned_count += 1;
    }
}

fn earth_leviathan_acquire_target(
    mut located_events: EventWriter<EarthLeviathanTargetLocatedEvent>,
    mut warning_events: EventWriter<EarthLeviathanAttackWarningBegunEvent>,
    mut leviathans: Query<
        (
            Entity,
            &EarthLeviathan,
            &SimPosition,
            &mut EarthLeviathanTarget,
            &mut EarthLeviathanTimers,
            &EarthLeviathanState,
        ),
        With<EarthLeviathan>,
    >,
    victims: Query<(Entity, &SimPosition, &EarthLeviathanVictimSensor), Without<EarthLeviathan>>,
    sim_hz: Res<SimHz>,
) {
    for (leviathan_entity, leviathan, position, mut target, mut timers, state) in
        leviathans.iter_mut()
    {
        if *state != EarthLeviathanState::Roaming || target.has_target {
            continue;
        }

        let Some((victim_entity, victim_position, sensor)) =
            nearest_detectable_victim(*position, EARTH_LEVIATHAN_WATCH_RANGE, &victims)
        else {
            continue;
        };

        target.has_target = true;
        target.target_entity = victim_entity;
        target.target_stable_id = sensor.stable_id;
        target.target_position = victim_position;
        target.centered_on_target = false;

        let minimum_warning_ticks = if leviathan.post_version_50 {
            fixed_seconds_to_ticks(
                fixed::types::I32F32::from_num(EARTH_LEVIATHAN_MIN_WARNING_SECONDS_AFTER_VERSION_50),
                sim_hz.0,
            )
        } else {
            0
        };
        timers.warning_ticks_remaining = minimum_warning_ticks;

        located_events.send(EarthLeviathanTargetLocatedEvent {
            leviathan: leviathan_entity,
            target: victim_entity,
            target_stable_id: sensor.stable_id,
        });

        warning_events.send(EarthLeviathanAttackWarningBegunEvent {
            leviathan: leviathan_entity,
            target: victim_entity,
            target_stable_id: sensor.stable_id,
            warning_ticks: timers.warning_ticks_remaining,
        });
    }
}

fn earth_leviathan_begin_tracking_from_warning(
    mut warning_events: EventReader<EarthLeviathanAttackWarningBegunEvent>,
    mut state_events: EventWriter<EarthLeviathanStateChangedEvent>,
    mut leviathans: Query<(Entity, &mut EarthLeviathanState), With<EarthLeviathan>>,
) {
    for event in warning_events.read() {
        let Ok((entity, mut state)) = leviathans.get_mut(event.leviathan) else {
            continue;
        };

        set_earth_leviathan_state(
            entity,
            &mut state,
            EarthLeviathanState::Tracking,
            &mut state_events,
        );
    }
}

fn earth_leviathan_track_target(
    mut leviathans: Query<
        (
            &mut SimPosition,
            &mut EarthLeviathanTarget,
            &mut UnitStats,
            &EarthLeviathanState,
        ),
        With<EarthLeviathan>,
    >,
    victims: Query<(Entity, &SimPosition, &EarthLeviathanVictimSensor), Without<EarthLeviathan>>,
    sim_hz: Res<SimHz>,
) {
    for (mut position, mut target, mut stats, state) in leviathans.iter_mut() {
        if *state != EarthLeviathanState::Tracking || !target.has_target {
            continue;
        }

        if let Some((victim_position, sensor)) =
            victim_position_by_stable_id(target.target_stable_id, &victims)
        {
            if sensor.is_alive && sensor.detectable {
                target.target_position = victim_position;
            }
        }

        stats.move_speed = EARTH_LEVIATHAN_TRACKING_SPEED;
        move_axis_toward(
            &mut position,
            target.target_position,
            EARTH_LEVIATHAN_TRACKING_SPEED / sim_hz.0,
        );

        target.centered_on_target =
            fixed_distance_sq(*position, target.target_position) <= fixed_square(stats.attack_range);
    }
}

fn earth_leviathan_start_emergence(
    mut emergence_events: EventWriter<EarthLeviathanEmergenceStartedEvent>,
    mut state_events: EventWriter<EarthLeviathanStateChangedEvent>,
    mut leviathans: Query<
        (
            Entity,
            &mut EarthLeviathanState,
            &EarthLeviathanTarget,
            &mut EarthLeviathanTimers,
        ),
        With<EarthLeviathan>,
    >,
    sim_hz: Res<SimHz>,
) {
    for (entity, mut state, target, mut timers) in leviathans.iter_mut() {
        if *state != EarthLeviathanState::Tracking || !target.has_target {
            continue;
        }

        if timers.warning_ticks_remaining > 0 {
            timers.warning_ticks_remaining -= 1;
            continue;
        }

        if !target.centered_on_target {
            continue;
        }

        timers.emergence_ticks_remaining = fixed_seconds_to_ticks(fixed::types::I32F32::ONE, sim_hz.0);

        emergence_events.send(EarthLeviathanEmergenceStartedEvent {
            leviathan: entity,
            target: target.target_entity,
            target_stable_id: target.target_stable_id,
            emergence_ticks: timers.emergence_ticks_remaining,
            black_particles_and_debris: true,
        });

        set_earth_leviathan_state(
            entity,
            &mut state,
            EarthLeviathanState::Emerging,
            &mut state_events,
        );
    }
}

fn earth_leviathan_complete_emergence(
    mut kill_events: EventWriter<EarthLeviathanInstantKillEvent>,
    mut damage_events: EventWriter<IncomingDamageEvent>,
    mut state_events: EventWriter<EarthLeviathanStateChangedEvent>,
    mut leviathans: Query<
        (
            Entity,
            &SimPosition,
            &mut EarthLeviathanState,
            &mut EarthLeviathanTimers,
            &UnitStats,
        ),
        With<EarthLeviathan>,
    >,
    victims: Query<
        (Entity, &SimPosition, &EarthLeviathanVictimSensor, &Health),
        Without<EarthLeviathan>,
    >,
) {
    for (leviathan_entity, position, mut state, mut timers, stats) in leviathans.iter_mut() {
        if *state != EarthLeviathanState::Emerging {
            continue;
        }

        if timers.emergence_ticks_remaining > 0 {
            timers.emergence_ticks_remaining -= 1;
            continue;
        }

        for (victim_entity, victim_position, sensor, health) in victims.iter() {
            if !sensor.is_alive {
                continue;
            }

            if fixed_distance_sq(*position, *victim_position) > fixed_square(stats.attack_range) {
                continue;
            }

            kill_events.send(EarthLeviathanInstantKillEvent {
                leviathan: leviathan_entity,
                victim: victim_entity,
                victim_stable_id: sensor.stable_id,
            });

            damage_events.send(IncomingDamageEvent {
                target: victim_entity,
                raw_amount: health.current,
                damage_type: DamageType::Standard,
                source: leviathan_entity,
            });
        }

        set_earth_leviathan_state(
            leviathan_entity,
            &mut state,
            EarthLeviathanState::Surfaced,
            &mut state_events,
        );
    }
}

fn earth_leviathan_ignore_shovel_attacks(
    mut damage_events: EventReader<IncomingDamageEvent>,
    mut ignored_events: EventWriter<EarthLeviathanShovelAttackIgnoredEvent>,
    leviathans: Query<(), With<EarthLeviathan>>,
) {
    for event in damage_events.read() {
        if leviathans.get(event.target).is_err() {
            continue;
        }

        ignored_events.send(EarthLeviathanShovelAttackIgnoredEvent {
            leviathan: event.target,
            source: event.source,
        });
    }
}

fn earth_leviathan_ignore_shock(
    mut shock_events: EventReader<EarthLeviathanShockIgnoredEvent>,
    leviathans: Query<(), With<EarthLeviathan>>,
) {
    for event in shock_events.read() {
        if leviathans.get(event.leviathan).is_err() {
            continue;
        }
    }
}

fn earth_leviathan_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    sim_hz: Res<SimHz>,
    leviathans: Query<
        (
            &EarthLeviathan,
            &SimPosition,
            &Health,
            &UnitStats,
            &EarthLeviathanState,
            &EarthLeviathanTarget,
            &EarthLeviathanTimers,
        ),
        With<EarthLeviathan>,
    >,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(sim_hz.0.to_bits() as u64);
    checksum.accumulate(EARTH_LEVIATHAN_SOURCE_REVISION as u64);
    checksum.accumulate(EARTH_LEVIATHAN_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(EARTH_LEVIATHAN_POWER_LEVEL as u64);
    checksum.accumulate(EARTH_LEVIATHAN_MAX_SPAWNED as u64);
    checksum.accumulate(EARTH_LEVIATHAN_MIN_WARNING_SECONDS_AFTER_VERSION_50 as u64);
    checksum.accumulate(EARTH_LEVIATHAN_TRACKING_SPEED.to_bits() as u64);
    checksum.accumulate(EARTH_LEVIATHAN_ROAM_SPEED.to_bits() as u64);
    checksum.accumulate(EARTH_LEVIATHAN_ATTACK_ZONE_RADIUS.to_bits() as u64);
    checksum.accumulate(EARTH_LEVIATHAN_WATCH_RANGE.to_bits() as u64);

    accumulate_str(&mut checksum, 0x1000, EARTH_LEVIATHAN_ID);
    accumulate_str(&mut checksum, 0x1001, EARTH_LEVIATHAN_NAME);
    accumulate_str(&mut checksum, 0x1002, EARTH_LEVIATHAN_TYPE);
    accumulate_str(&mut checksum, 0x1003, EARTH_LEVIATHAN_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, EARTH_LEVIATHAN_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1005, EARTH_LEVIATHAN_EXTRACTED_AT);
    accumulate_str(&mut checksum, 0x1006, EARTH_LEVIATHAN_DWELLS);
    accumulate_str(&mut checksum, 0x1007, EARTH_LEVIATHAN_SCIENTIFIC_NAME);
    accumulate_str(&mut checksum, 0x1008, EARTH_LEVIATHAN_ATTACK_DAMAGE);
    accumulate_str(&mut checksum, 0x1009, EARTH_LEVIATHAN_CONTACT_DAMAGE);
    accumulate_str(&mut checksum, 0x100A, EARTH_LEVIATHAN_RADAR_PIP_SIZE);
    accumulate_str(&mut checksum, 0x100B, EARTH_LEVIATHAN_SHOVEL_HP);
    accumulate_str(&mut checksum, 0x100C, EARTH_LEVIATHAN_SHOCK_RESPONSE);
    accumulate_str(&mut checksum, 0x100D, EARTH_LEVIATHAN_INTERNAL_NAME);

    for behavior in EARTH_LEVIATHAN_FRONTMATTER_BEHAVIOR {
        accumulate_str(&mut checksum, 0x2000, behavior);
    }

    for rule in EARTH_LEVIATHAN_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x3000, rule.condition);
        accumulate_str(&mut checksum, 0x3001, rule.outcome);
    }

    for (leviathan, position, health, stats, state, target, timers) in leviathans.iter() {
        checksum.accumulate(leviathan.stable_id);
        checksum.accumulate(leviathan.active_in_room as u64);
        checksum.accumulate(leviathan.post_version_50 as u64);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);
        checksum.accumulate(earth_leviathan_state_bits(*state));
        checksum.accumulate(target.has_target as u64);
        checksum.accumulate(target.target_stable_id);
        checksum.accumulate(target.centered_on_target as u64);
        checksum.accumulate(target.target_position.x.to_bits() as u64);
        checksum.accumulate(target.target_position.y.to_bits() as u64);
        checksum.accumulate(timers.warning_ticks_remaining as u64);
        checksum.accumulate(timers.emergence_ticks_remaining as u64);
    }
}

fn nearest_detectable_victim(
    leviathan_position: SimPosition,
    range: fixed::types::I32F32,
    victims: &Query<(Entity, &SimPosition, &EarthLeviathanVictimSensor), Without<EarthLeviathan>>,
) -> Option<(Entity, SimPosition, EarthLeviathanVictimSensor)> {
    let mut best: Option<(Entity, SimPosition, EarthLeviathanVictimSensor, fixed::types::I32F32)> =
        None;
    let range_sq = fixed_square(range);

    for (entity, position, sensor) in victims.iter() {
        if !sensor.is_alive || !sensor.detectable {
            continue;
        }

        let distance_sq = fixed_distance_sq(leviathan_position, *position);
        if distance_sq > range_sq {
            continue;
        }

        match best {
            Some((_entity, _position, _sensor, best_distance_sq))
                if distance_sq >= best_distance_sq => {}
            _ => {
                best = Some((entity, *position, *sensor, distance_sq));
            }
        }
    }

    best.map(|(entity, position, sensor, _distance_sq)| (entity, position, sensor))
}

fn victim_position_by_stable_id(
    stable_id: u64,
    victims: &Query<(Entity, &SimPosition, &EarthLeviathanVictimSensor), Without<EarthLeviathan>>,
) -> Option<(SimPosition, EarthLeviathanVictimSensor)> {
    for (_entity, position, sensor) in victims.iter() {
        if sensor.stable_id == stable_id {
            return Some((*position, *sensor));
        }
    }

    None
}

fn set_earth_leviathan_state(
    leviathan: Entity,
    state: &mut EarthLeviathanState,
    next: EarthLeviathanState,
    events: &mut EventWriter<EarthLeviathanStateChangedEvent>,
) {
    if *state == next {
        return;
    }

    let previous = *state;
    *state = next;
    events.send(EarthLeviathanStateChangedEvent {
        leviathan,
        from: previous,
        to: next,
    });
}

fn move_axis_toward(
    position: &mut SimPosition,
    target: SimPosition,
    max_step: fixed::types::I32F32,
) {
    position.x = move_scalar_toward(position.x, target.x, max_step);
    position.y = move_scalar_toward(position.y, target.y, max_step);
}

fn move_scalar_toward(
    current: fixed::types::I32F32,
    target: fixed::types::I32F32,
    max_step: fixed::types::I32F32,
) -> fixed::types::I32F32 {
    if current < target {
        let next = current + max_step;
        if next > target {
            target
        } else {
            next
        }
    } else if current > target {
        let next = current - max_step;
        if next < target {
            target
        } else {
            next
        }
    } else {
        current
    }
}

fn fixed_distance_sq(a: SimPosition, b: SimPosition) -> fixed::types::I32F32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    dx * dx + dy * dy
}

fn fixed_square(value: fixed::types::I32F32) -> fixed::types::I32F32 {
    value * value
}

fn fixed_seconds_to_ticks(seconds: fixed::types::I32F32, sim_hz: fixed::types::I32F32) -> u32 {
    let ticks = seconds * sim_hz;
    if ticks <= fixed::types::I32F32::ZERO {
        0
    } else {
        ticks.ceil().to_num::<u32>()
    }
}

fn earth_leviathan_state_bits(state: EarthLeviathanState) -> u64 {
    match state {
        EarthLeviathanState::Roaming => 0,
        EarthLeviathanState::Tracking => 1,
        EarthLeviathanState::Emerging => 2,
        EarthLeviathanState::Surfaced => 3,
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}
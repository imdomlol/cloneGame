// Sources: vault/harmless_entity_pages/roaming_locust.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{
    Health, IncomingDamageEvent, SimChecksumState, SimHz, SimPosition, SimTick, UnitStats,
};

pub const ROAMING_LOCUST_ID: &str = "roaming_locust";
pub const ROAMING_LOCUST_NAME: &str = "Roaming Locust";
pub const ROAMING_LOCUST_TYPE: &str = "harmless_entity_pages";
pub const ROAMING_LOCUST_SUBTYPE: &str = "creature";
pub const ROAMING_LOCUST_SOURCE_URL: &str =
    "https://lethal-company.fandom.com/wiki/Roaming_Locust";
pub const ROAMING_LOCUST_SOURCE_REVISION: u32 = 19424;
pub const ROAMING_LOCUST_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const ROAMING_LOCUST_CONFIDENCE_BASIS_POINTS: u16 = 96;

pub const ROAMING_LOCUST_DWELLS: &str = "Outside (Daytime)";
pub const ROAMING_LOCUST_DANGER: &str = "Harmless";
pub const ROAMING_LOCUST_POWER_LEVEL: I32F32 = I32F32::lit("1");
pub const ROAMING_LOCUST_MAX_SPAWNED: usize = 5;
pub const ROAMING_LOCUST_SHOVEL_HP: &str = "Immune";
pub const ROAMING_LOCUST_SHOCK_RESPONSE: &str = "Immune";
pub const ROAMING_LOCUST_CAN_SEE_THROUGH_FOG: bool = false;
pub const ROAMING_LOCUST_PIP_SIZE: &str = "Tiny";
pub const ROAMING_LOCUST_INTERNAL_NAME: &str = "Roaming";

pub const ROAMING_LOCUST_ALIASES: [&str; 2] = ["Locust Bees", "Docile Locusts"];
pub const ROAMING_LOCUST_FRONTMATTER_BEHAVIOR: [&str; 1] = ["Roaming"];

pub const ROAMING_LOCUST_GROUP_NEAR_RADIUS: I32F32 = I32F32::lit("12");
pub const ROAMING_LOCUST_DISPERSE_RADIUS: I32F32 = I32F32::lit("2");
pub const ROAMING_LOCUST_ROAM_SPEED: I32F32 = I32F32::lit("2.5");
pub const ROAMING_LOCUST_FLEE_SPEED: I32F32 = I32F32::lit("8");
pub const ROAMING_LOCUST_DISPERSE_SPEED: I32F32 = I32F32::lit("11");
pub const ROAMING_LOCUST_FORMATION_RADIUS: I32F32 = I32F32::lit("1.5");
pub const ROAMING_LOCUST_IMMUNE_HEALTH: I32F32 = I32F32::lit("0");

pub const ROAMING_LOCUST_SPAWN_CHANCE_BY_MOON: [RoamingLocustMoonSpawnChance; 6] = [
    RoamingLocustMoonSpawnChance {
        moon: "Experimentation",
        base_spawn_chance: I32F32::lit("35.14"),
    },
    RoamingLocustMoonSpawnChance {
        moon: "Artifice",
        base_spawn_chance: I32F32::lit("34.9"),
    },
    RoamingLocustMoonSpawnChance {
        moon: "Adamance",
        base_spawn_chance: I32F32::lit("24.56"),
    },
    RoamingLocustMoonSpawnChance {
        moon: "Assurance",
        base_spawn_chance: I32F32::lit("23.0"),
    },
    RoamingLocustMoonSpawnChance {
        moon: "Vow",
        base_spawn_chance: I32F32::lit("20.73"),
    },
    RoamingLocustMoonSpawnChance {
        moon: "March",
        base_spawn_chance: I32F32::lit("20.21"),
    },
];

pub const ROAMING_LOCUST_BEHAVIORAL_MECHANICS: [RoamingLocustBehaviorRule; 3] = [
    RoamingLocustBehaviorRule {
        condition: "a player is near the group",
        outcome: "the Roaming Locusts fly away",
    },
    RoamingLocustBehaviorRule {
        condition: "a player runs through the formation",
        outcome: "the group disperses",
    },
    RoamingLocustBehaviorRule {
        condition: "the entity is outdoors during daytime",
        outcome: "it can appear in the environment",
    },
];

pub struct RoamingLocustPlugin;

impl Plugin for RoamingLocustPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnRoamingLocustEvent>()
            .add_event::<RoamingLocustStateChangedEvent>()
            .add_event::<RoamingLocustIgnoredDamageEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_roaming_locust,
                    roaming_locust_daytime_presence,
                    roaming_locust_player_near_group,
                    roaming_locust_player_runs_through_formation,
                    roaming_locust_apply_state_movement,
                    roaming_locust_formation_spacing,
                    roaming_locust_ignore_damage,
                    roaming_locust_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoamingLocustBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoamingLocustMoonSpawnChance {
    pub moon: &'static str,
    pub base_spawn_chance: I32F32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RoamingLocust;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RoamingLocustEmployeeTarget {
    pub stable_id: u64,
    pub outdoors: bool,
    pub running: bool,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RoamingLocustDaytimeEnvironment {
    pub outdoors: bool,
    pub daytime: bool,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoamingLocustFormation {
    pub group_id: u64,
    pub slot: u8,
    pub center: SimPosition,
    pub velocity_x: I32F32,
    pub velocity_y: I32F32,
    pub target_employee_stable_id: u64,
}

impl Default for RoamingLocustFormation {
    fn default() -> Self {
        Self {
            group_id: 0,
            slot: 0,
            center: SimPosition {
                x: I32F32::lit("0"),
                y: I32F32::lit("0"),
            },
            velocity_x: I32F32::lit("0"),
            velocity_y: I32F32::lit("0"),
            target_employee_stable_id: 0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RoamingLocustState {
    #[default]
    Roaming,
    FlyingAway,
    Dispersed,
    Dormant,
}

#[derive(Bundle)]
pub struct RoamingLocustBundle {
    pub name: Name,
    pub locust: RoamingLocust,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub state: RoamingLocustState,
    pub formation: RoamingLocustFormation,
}

impl RoamingLocustBundle {
    pub fn new(event: SpawnRoamingLocustEvent) -> Self {
        Self {
            name: Name::new(ROAMING_LOCUST_NAME),
            locust: RoamingLocust,
            position: event.position,
            health: Health::full(ROAMING_LOCUST_IMMUNE_HEALTH),
            stats: UnitStats {
                move_speed: ROAMING_LOCUST_ROAM_SPEED,
                attack_range: I32F32::lit("0"),
                attack_damage: I32F32::lit("0"),
                attack_speed: I32F32::lit("0"),
                watch_range: ROAMING_LOCUST_GROUP_NEAR_RADIUS,
            },
            state: RoamingLocustState::Roaming,
            formation: RoamingLocustFormation {
                group_id: event.group_id,
                slot: event.slot,
                center: event.group_center,
                velocity_x: I32F32::lit("0"),
                velocity_y: I32F32::lit("0"),
                target_employee_stable_id: 0,
            },
        }
    }
}

#[derive(Event, Clone, Copy, Debug)]
pub struct SpawnRoamingLocustEvent {
    pub position: SimPosition,
    pub group_center: SimPosition,
    pub group_id: u64,
    pub slot: u8,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoamingLocustStateChangedEvent {
    pub locust: Entity,
    pub from: RoamingLocustState,
    pub to: RoamingLocustState,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoamingLocustIgnoredDamageEvent {
    pub locust: Entity,
    pub source: Option<Entity>,
}

fn spawn_roaming_locust(
    mut commands: Commands,
    mut events: EventReader<SpawnRoamingLocustEvent>,
    locusts: Query<(), With<RoamingLocust>>,
) {
    let mut spawned_count = locusts.iter().count();

    for event in events.read() {
        if spawned_count >= ROAMING_LOCUST_MAX_SPAWNED {
            break;
        }

        commands.spawn(RoamingLocustBundle::new(*event));
        spawned_count += 1;
    }
}

fn roaming_locust_daytime_presence(
    mut state_events: EventWriter<RoamingLocustStateChangedEvent>,
    environment: Query<&RoamingLocustDaytimeEnvironment>,
    mut locusts: Query<(Entity, &mut RoamingLocustState), With<RoamingLocust>>,
) {
    let can_appear = environment
        .iter()
        .any(|environment| environment.outdoors && environment.daytime);

    for (entity, mut state) in locusts.iter_mut() {
        if can_appear {
            if *state == RoamingLocustState::Dormant {
                set_state(
                    entity,
                    &mut state,
                    RoamingLocustState::Roaming,
                    &mut state_events,
                );
            }
        } else {
            set_state(
                entity,
                &mut state,
                RoamingLocustState::Dormant,
                &mut state_events,
            );
        }
    }
}

fn roaming_locust_player_near_group(
    mut state_events: EventWriter<RoamingLocustStateChangedEvent>,
    mut locusts: Query<
        (
            Entity,
            &SimPosition,
            &mut RoamingLocustState,
            &mut UnitStats,
            &mut RoamingLocustFormation,
        ),
        With<RoamingLocust>,
    >,
    employees: Query<(&SimPosition, &RoamingLocustEmployeeTarget)>,
) {
    let near_radius_squared = ROAMING_LOCUST_GROUP_NEAR_RADIUS * ROAMING_LOCUST_GROUP_NEAR_RADIUS;

    for (entity, position, mut state, mut stats, mut formation) in locusts.iter_mut() {
        if *state != RoamingLocustState::Roaming {
            continue;
        }

        let Some((employee_position, stable_id)) =
            closest_outdoor_employee(*position, near_radius_squared, &employees)
        else {
            continue;
        };

        formation.target_employee_stable_id = stable_id;
        set_away_velocity(
            &mut formation,
            *position,
            employee_position,
            ROAMING_LOCUST_FLEE_SPEED,
        );
        stats.move_speed = ROAMING_LOCUST_FLEE_SPEED;

        set_state(
            entity,
            &mut state,
            RoamingLocustState::FlyingAway,
            &mut state_events,
        );
    }
}

fn roaming_locust_player_runs_through_formation(
    mut state_events: EventWriter<RoamingLocustStateChangedEvent>,
    mut locusts: Query<
        (
            Entity,
            &SimPosition,
            &mut RoamingLocustState,
            &mut UnitStats,
            &mut RoamingLocustFormation,
        ),
        With<RoamingLocust>,
    >,
    employees: Query<(&SimPosition, &RoamingLocustEmployeeTarget)>,
) {
    let disperse_radius_squared = ROAMING_LOCUST_DISPERSE_RADIUS * ROAMING_LOCUST_DISPERSE_RADIUS;

    for (entity, position, mut state, mut stats, mut formation) in locusts.iter_mut() {
        if *state == RoamingLocustState::Dormant {
            continue;
        }

        let Some((employee_position, stable_id)) =
            closest_running_outdoor_employee(*position, disperse_radius_squared, &employees)
        else {
            continue;
        };

        formation.target_employee_stable_id = stable_id;
        set_away_velocity(
            &mut formation,
            *position,
            employee_position,
            ROAMING_LOCUST_DISPERSE_SPEED,
        );
        stats.move_speed = ROAMING_LOCUST_DISPERSE_SPEED;

        set_state(
            entity,
            &mut state,
            RoamingLocustState::Dispersed,
            &mut state_events,
        );
    }
}

fn roaming_locust_apply_state_movement(
    sim_hz: Res<SimHz>,
    mut locusts: Query<
        (
            &mut SimPosition,
            &RoamingLocustState,
            &UnitStats,
            &mut RoamingLocustFormation,
        ),
        With<RoamingLocust>,
    >,
) {
    let tick_scale = I32F32::lit("1") / sim_hz.0;

    for (mut position, state, stats, mut formation) in locusts.iter_mut() {
        if *state == RoamingLocustState::Dormant {
            continue;
        }

        if *state == RoamingLocustState::Roaming
            && formation.velocity_x == I32F32::lit("0")
            && formation.velocity_y == I32F32::lit("0")
        {
            formation.velocity_x = stats.move_speed;
            formation.velocity_y = I32F32::lit("0");
        }

        let velocity_x = formation.velocity_x;
        let velocity_y = formation.velocity_y;
        position.x += velocity_x * tick_scale;
        position.y += velocity_y * tick_scale;
        formation.center.x += velocity_x * tick_scale;
        formation.center.y += velocity_y * tick_scale;
    }
}

fn roaming_locust_formation_spacing(
    mut locusts: Query<
        (&mut SimPosition, &RoamingLocustState, &RoamingLocustFormation),
        With<RoamingLocust>,
    >,
) {
    for (mut position, state, formation) in locusts.iter_mut() {
        if *state != RoamingLocustState::Roaming && *state != RoamingLocustState::FlyingAway {
            continue;
        }

        let offset = formation_offset(formation.slot);
        position.x = formation.center.x + offset.x;
        position.y = formation.center.y + offset.y;
    }
}

fn roaming_locust_ignore_damage(
    mut damage_events: EventReader<IncomingDamageEvent>,
    mut ignored_events: EventWriter<RoamingLocustIgnoredDamageEvent>,
    locusts: Query<(), With<RoamingLocust>>,
) {
    for event in damage_events.read() {
        if locusts.get(event.target).is_ok() {
            ignored_events.send(RoamingLocustIgnoredDamageEvent {
                locust: event.target,
                source: Some(event.source),
            });
        }
    }
}

fn roaming_locust_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    locusts: Query<
        (
            &SimPosition,
            &Health,
            &UnitStats,
            &RoamingLocustState,
            &RoamingLocustFormation,
        ),
        With<RoamingLocust>,
    >,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(ROAMING_LOCUST_SOURCE_REVISION as u64);
    checksum.accumulate(ROAMING_LOCUST_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(ROAMING_LOCUST_POWER_LEVEL.to_bits() as u64);
    checksum.accumulate(ROAMING_LOCUST_MAX_SPAWNED as u64);
    checksum.accumulate(ROAMING_LOCUST_CAN_SEE_THROUGH_FOG as u64);
    checksum.accumulate(ROAMING_LOCUST_GROUP_NEAR_RADIUS.to_bits() as u64);
    checksum.accumulate(ROAMING_LOCUST_DISPERSE_RADIUS.to_bits() as u64);
    checksum.accumulate(ROAMING_LOCUST_ROAM_SPEED.to_bits() as u64);
    checksum.accumulate(ROAMING_LOCUST_FLEE_SPEED.to_bits() as u64);
    checksum.accumulate(ROAMING_LOCUST_DISPERSE_SPEED.to_bits() as u64);
    checksum.accumulate(ROAMING_LOCUST_FORMATION_RADIUS.to_bits() as u64);
    checksum.accumulate(ROAMING_LOCUST_IMMUNE_HEALTH.to_bits() as u64);

    accumulate_str(&mut checksum, 0x1000, ROAMING_LOCUST_ID);
    accumulate_str(&mut checksum, 0x1001, ROAMING_LOCUST_NAME);
    accumulate_str(&mut checksum, 0x1002, ROAMING_LOCUST_TYPE);
    accumulate_str(&mut checksum, 0x1003, ROAMING_LOCUST_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, ROAMING_LOCUST_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1005, ROAMING_LOCUST_EXTRACTED_AT);
    accumulate_str(&mut checksum, 0x1006, ROAMING_LOCUST_DWELLS);
    accumulate_str(&mut checksum, 0x1007, ROAMING_LOCUST_DANGER);
    accumulate_str(&mut checksum, 0x1008, ROAMING_LOCUST_SHOVEL_HP);
    accumulate_str(&mut checksum, 0x1009, ROAMING_LOCUST_SHOCK_RESPONSE);
    accumulate_str(&mut checksum, 0x100a, ROAMING_LOCUST_PIP_SIZE);
    accumulate_str(&mut checksum, 0x100b, ROAMING_LOCUST_INTERNAL_NAME);

    for alias in ROAMING_LOCUST_ALIASES {
        accumulate_str(&mut checksum, 0x2000, alias);
    }

    for behavior in ROAMING_LOCUST_FRONTMATTER_BEHAVIOR {
        accumulate_str(&mut checksum, 0x3000, behavior);
    }

    for spawn_chance in ROAMING_LOCUST_SPAWN_CHANCE_BY_MOON {
        accumulate_str(&mut checksum, 0x4000, spawn_chance.moon);
        checksum.accumulate(spawn_chance.base_spawn_chance.to_bits() as u64);
    }

    for rule in ROAMING_LOCUST_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x5000, rule.condition);
        accumulate_str(&mut checksum, 0x5001, rule.outcome);
    }

    for (position, health, stats, state, formation) in locusts.iter() {
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);
        checksum.accumulate(*state as u64);
        checksum.accumulate(formation.group_id);
        checksum.accumulate(formation.slot as u64);
        checksum.accumulate(formation.center.x.to_bits() as u64);
        checksum.accumulate(formation.center.y.to_bits() as u64);
        checksum.accumulate(formation.velocity_x.to_bits() as u64);
        checksum.accumulate(formation.velocity_y.to_bits() as u64);
        checksum.accumulate(formation.target_employee_stable_id);
    }
}

fn closest_outdoor_employee(
    origin: SimPosition,
    max_distance_squared: I32F32,
    employees: &Query<(&SimPosition, &RoamingLocustEmployeeTarget)>,
) -> Option<(SimPosition, u64)> {
    let mut selected: Option<(SimPosition, u64, I32F32)> = None;

    for (employee_position, employee) in employees.iter() {
        if !employee.outdoors {
            continue;
        }

        let distance = distance_squared(origin, *employee_position);
        if distance > max_distance_squared {
            continue;
        }

        selected = match selected {
            Some((selected_position, selected_id, selected_distance))
                if selected_distance < distance
                    || (selected_distance == distance && selected_id <= employee.stable_id) =>
            {
                Some((selected_position, selected_id, selected_distance))
            }
            _ => Some((*employee_position, employee.stable_id, distance)),
        };
    }

    selected.map(|(position, stable_id, _)| (position, stable_id))
}

fn closest_running_outdoor_employee(
    origin: SimPosition,
    max_distance_squared: I32F32,
    employees: &Query<(&SimPosition, &RoamingLocustEmployeeTarget)>,
) -> Option<(SimPosition, u64)> {
    let mut selected: Option<(SimPosition, u64, I32F32)> = None;

    for (employee_position, employee) in employees.iter() {
        if !employee.outdoors || !employee.running {
            continue;
        }

        let distance = distance_squared(origin, *employee_position);
        if distance > max_distance_squared {
            continue;
        }

        selected = match selected {
            Some((selected_position, selected_id, selected_distance))
                if selected_distance < distance
                    || (selected_distance == distance && selected_id <= employee.stable_id) =>
            {
                Some((selected_position, selected_id, selected_distance))
            }
            _ => Some((*employee_position, employee.stable_id, distance)),
        };
    }

    selected.map(|(position, stable_id, _)| (position, stable_id))
}

fn set_away_velocity(
    formation: &mut RoamingLocustFormation,
    locust_position: SimPosition,
    employee_position: SimPosition,
    speed: I32F32,
) {
    let dx = locust_position.x - employee_position.x;
    let dy = locust_position.y - employee_position.y;

    formation.velocity_x = signed_axis_speed(dx, speed);
    formation.velocity_y = signed_axis_speed(dy, speed);

    if formation.velocity_x == I32F32::lit("0") && formation.velocity_y == I32F32::lit("0") {
        formation.velocity_x = speed;
    }
}

fn signed_axis_speed(delta: I32F32, speed: I32F32) -> I32F32 {
    if delta > I32F32::lit("0") {
        speed
    } else if delta < I32F32::lit("0") {
        -speed
    } else {
        I32F32::lit("0")
    }
}

fn formation_offset(slot: u8) -> SimPosition {
    match slot % 5 {
        0 => SimPosition {
            x: I32F32::lit("0"),
            y: I32F32::lit("0"),
        },
        1 => SimPosition {
            x: ROAMING_LOCUST_FORMATION_RADIUS,
            y: I32F32::lit("0"),
        },
        2 => SimPosition {
            x: -ROAMING_LOCUST_FORMATION_RADIUS,
            y: I32F32::lit("0"),
        },
        3 => SimPosition {
            x: I32F32::lit("0"),
            y: ROAMING_LOCUST_FORMATION_RADIUS,
        },
        _ => SimPosition {
            x: I32F32::lit("0"),
            y: -ROAMING_LOCUST_FORMATION_RADIUS,
        },
    }
}

fn set_state(
    entity: Entity,
    state: &mut RoamingLocustState,
    to: RoamingLocustState,
    events: &mut EventWriter<RoamingLocustStateChangedEvent>,
) {
    if *state == to {
        return;
    }

    let from = *state;
    *state = to;
    events.send(RoamingLocustStateChangedEvent {
        locust: entity,
        from,
        to,
    });
}

fn distance_squared(a: SimPosition, b: SimPosition) -> I32F32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    dx * dx + dy * dy
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}
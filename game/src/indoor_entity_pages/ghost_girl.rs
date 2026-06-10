// Sources: vault/indoor_entity_pages/ghost_girl.md
use bevy::prelude::*;
use fixed::types::I32F32;
use rand_core::RngCore;

use crate::sim::{
    tick_rng, DamageType, GameSeed, Health, IncomingDamageEvent, SimChecksumState, SimHz,
    SimPosition, SimTick, UnitStats,
};

pub const GHOST_GIRL_ID: &str = "ghost_girl";
pub const GHOST_GIRL_NAME: &str = "Ghost Girl";
pub const GHOST_GIRL_TYPE: &str = "indoor_entity_pages";
pub const GHOST_GIRL_SUBTYPE: &str = "creature";
pub const GHOST_GIRL_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Ghost_Girl";
pub const GHOST_GIRL_SOURCE_REVISION: u32 = 21476;
pub const GHOST_GIRL_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const GHOST_GIRL_CONFIDENCE_BASIS_POINTS: u16 = 91;

pub const GHOST_GIRL_DWELLS: &str = "Inside and Outside";
pub const GHOST_GIRL_POWER_LEVEL: I32F32 = I32F32::lit("2");
pub const GHOST_GIRL_MAX_SPAWNED: usize = 1;
pub const GHOST_GIRL_ATTACK_DAMAGE_TEXT: &str = "Instant Kill";
pub const GHOST_GIRL_STUN_MULTIPLIER: I32F32 = I32F32::lit("0");
pub const GHOST_GIRL_STUN_GRENADE: &str = "Immune";
pub const GHOST_GIRL_SHOCK_RESPONSE: &str = "Immune";
pub const GHOST_GIRL_RADAR_PIP_SIZE: &str = "Invisible";
pub const GHOST_GIRL_SHOVEL_HP: &str = "Immune";
pub const GHOST_GIRL_CAN_SEE_THROUGH_FOG: bool = true;
pub const GHOST_GIRL_DOOR_OPEN_SPEED: I32F32 = I32F32::lit("1.5");
pub const GHOST_GIRL_CONTACT_DAMAGE: &str = "Instant Kill";

pub const GHOST_GIRL_INITIAL_ABSENT_DELAY_SECONDS: I32F32 = I32F32::lit("20");
pub const GHOST_GIRL_ABSENT_RETRY_SECONDS: I32F32 = I32F32::lit("4");
pub const GHOST_GIRL_INDOOR_STARE_SECONDS: I32F32 = I32F32::lit("15");
pub const GHOST_GIRL_OUTDOOR_STARE_SECONDS: I32F32 = I32F32::lit("25");
pub const GHOST_GIRL_LOS_LOST_STARE_RATE: I32F32 = I32F32::lit("3");
pub const GHOST_GIRL_LOOKING_STARE_RATE: I32F32 = I32F32::lit("1.25");
pub const GHOST_GIRL_LOOKING_CLOSE_RANGE: I32F32 = I32F32::lit("5");
pub const GHOST_GIRL_LOOKING_FAR_RANGE: I32F32 = I32F32::lit("100");
pub const GHOST_GIRL_STARE_CHASE_SEEN_THRESHOLD: u8 = 3;
pub const GHOST_GIRL_STARE_CHASE_UNSEEN_THRESHOLD: u8 = 3;
pub const GHOST_GIRL_STARE_CHASE_CHANCE_BASIS_POINTS: u32 = 8500;
pub const GHOST_GIRL_CLOSE_DISAPPEAR_RANGE: I32F32 = I32F32::lit("7");
pub const GHOST_GIRL_CLOSE_DISAPPEAR_SEEN_STARES_MAX: u8 = 1;
pub const GHOST_GIRL_CLOSE_DISAPPEAR_CHANCE_BASIS_POINTS: u32 = 2500;
pub const GHOST_GIRL_CLOSE_CHASE_RANGE: I32F32 = I32F32::lit("5");
pub const GHOST_GIRL_CLOSE_CHASE_SEEN_STARES_MIN: u8 = 2;
pub const GHOST_GIRL_CLOSE_CHASE_CHANCE_BASIS_POINTS: u32 = 6400;
pub const GHOST_GIRL_DISAPPEAR_TARGET_RANGE: I32F32 = I32F32::lit("4");
pub const GHOST_GIRL_DISAPPEAR_DESTINATION_RANGE: I32F32 = I32F32::lit("0.2");
pub const GHOST_GIRL_CHASE_SECONDS: I32F32 = I32F32::lit("20");
pub const GHOST_GIRL_CHASE_END_DISTANCE: I32F32 = I32F32::lit("50");
pub const GHOST_GIRL_CHASE_TELEPORT_RETRY_SECONDS: I32F32 = I32F32::lit("5");
pub const GHOST_GIRL_IMMUNE_HEALTH: I32F32 = I32F32::lit("0");
pub const GHOST_GIRL_STARE_SPEED: I32F32 = I32F32::lit("0");
pub const GHOST_GIRL_DISAPPEAR_SPEED: I32F32 = I32F32::lit("1");
pub const GHOST_GIRL_CHASE_SPEED: I32F32 = I32F32::lit("3");
pub const GHOST_GIRL_ATTACK_RANGE: I32F32 = I32F32::lit("0.5");
pub const GHOST_GIRL_INSTANT_KILL_DAMAGE: I32F32 = I32F32::lit("1000000");
pub const GHOST_GIRL_WATCH_RANGE: I32F32 = I32F32::lit("100");

pub const GHOST_GIRL_DEPENDS_ON: [&str; 2] = ["lethal_company", "employee"];

pub const GHOST_GIRL_FRONTMATTER_BEHAVIOR: [&str; 10] = [
    "IF the target is in the facility, THEN Absent-state spawn attempts search for a facility location every 4 seconds after an initial 20-second delay.",
    "IF the target is in the ship, THEN Absent-state spawn attempts search for a surface location.",
    "IF the target is outside both the ship and the facility, THEN the entity fails to appear.",
    "IF the entity is in Staring, THEN it faces the target and advances its timer at 3x speed while the target is out of line of sight and at 1.25x speed while the target is looking.",
    "IF the Staring timer expires, THEN the entity switches to Disappearing after 15 seconds indoors or 25 seconds outdoors.",
    "IF the target has at least 3 seen stares or at least 3 unseen stares, THEN each qualifying stare has an 85% chance to switch to Chasing.",
    "IF the target is within 7 units and the entity has 1 or fewer seen stares, THEN that stare has a 25% chance to switch to Disappearing.",
    "IF the target is within 5 units and the entity has at least 2 seen stares, THEN that stare also rolls an independent 64% chance to switch to Chasing.",
    "IF a non-target player touches the entity during Staring or Disappearing, THEN it switches to Vanishing.",
    "IF the entity is Chasing, THEN it skips toward the target, instantly kills on contact, lasts 20 seconds or until the target is more than 50 units away, and retries teleport selection every 5 seconds when the target is not looking.",
];

pub const GHOST_GIRL_BEHAVIORAL_MECHANICS: [GhostGirlBehaviorRule; 23] = [
    GhostGirlBehaviorRule {
        condition: "the target employee is assigned",
        outcome: "the entity remains in Absent until it begins spawn selection",
    },
    GhostGirlBehaviorRule {
        condition: "the entity is Absent for 20 seconds",
        outcome: "it starts attempting to spawn every 4 seconds until a valid location is found",
    },
    GhostGirlBehaviorRule {
        condition: "the target is in the facility",
        outcome: "the spawn location must be inside the facility and on valid AI navigation",
    },
    GhostGirlBehaviorRule {
        condition: "the target is in the ship",
        outcome: "the spawn location must be on the surface",
    },
    GhostGirlBehaviorRule {
        condition: "the target is neither in the ship nor in the facility",
        outcome: "the spawn attempt fails",
    },
    GhostGirlBehaviorRule {
        condition: "the entity enters Staring",
        outcome: "it rotates to face the target and checks whether the target is looking at it",
    },
    GhostGirlBehaviorRule {
        condition: "the target has line of sight and is within 5 units",
        outcome: "the target counts as looking in any facing direction",
    },
    GhostGirlBehaviorRule {
        condition: "the target has line of sight and is within 100 units",
        outcome: "the target counts as looking only while facing within 60 degrees of the entity",
    },
    GhostGirlBehaviorRule {
        condition: "the target loses line of sight",
        outcome: "the Staring timer advances at 3x speed",
    },
    GhostGirlBehaviorRule {
        condition: "the target is looking at the entity",
        outcome: "the Staring timer advances at 1.25x speed unless a chase starts",
    },
    GhostGirlBehaviorRule {
        condition: "the target accumulates at least 3 seen stares or 3 unseen stares",
        outcome: "that stare can trigger Chasing with an 85% chance",
    },
    GhostGirlBehaviorRule {
        condition: "the target enters within 7 units and the entity has 1 or fewer seen stares",
        outcome: "that stare can trigger Disappearing with a 25% chance",
    },
    GhostGirlBehaviorRule {
        condition: "the target enters within 5 units and the entity has at least 2 seen stares",
        outcome: "that stare also gets an independent 64% chance to trigger Chasing",
    },
    GhostGirlBehaviorRule {
        condition: "the entity does not start Chasing when the target reaches 5 units",
        outcome: "it switches to Vanishing",
    },
    GhostGirlBehaviorRule {
        condition: "a non-target player touches the entity during Staring",
        outcome: "it switches to Vanishing",
    },
    GhostGirlBehaviorRule {
        condition: "the entity enters Disappearing",
        outcome: "it walks toward an out-of-sight destination and becomes invisible once it leaves line of sight, gets within 4 units of the target, or reaches within 0.2 units of the destination",
    },
    GhostGirlBehaviorRule {
        condition: "the entity cannot find a valid Disappearing destination",
        outcome: "it switches to Vanishing",
    },
    GhostGirlBehaviorRule {
        condition: "the entity enters Vanishing",
        outcome: "it becomes invisible immediately and returns to Absent while causing short-lived light flicker",
    },
    GhostGirlBehaviorRule {
        condition: "the entity enters Chasing",
        outcome: "it skips toward the target and kills on contact",
    },
    GhostGirlBehaviorRule {
        condition: "the chase lasts 20 seconds",
        outcome: "it ends and the entity returns to Absent",
    },
    GhostGirlBehaviorRule {
        condition: "the target moves more than 50 units away during Chasing",
        outcome: "the chase ends immediately",
    },
    GhostGirlBehaviorRule {
        condition: "every 5 seconds during Chasing the target is not looking",
        outcome: "the entity attempts a teleport selection using Absent-state rules without requiring line of sight to the target",
    },
    GhostGirlBehaviorRule {
        condition: "the target dies",
        outcome: "the entity immediately selects a new target and resets that target's counters",
    },
];

pub struct GhostGirlPlugin;

impl Plugin for GhostGirlPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnGhostGirlEvent>()
            .add_event::<GhostGirlSpawnSelectionEvent>()
            .add_event::<GhostGirlSpawnSelectionResolvedEvent>()
            .add_event::<GhostGirlStateChangedEvent>()
            .add_event::<GhostGirlTargetChangedEvent>()
            .add_event::<GhostGirlStareRollEvent>()
            .add_event::<GhostGirlContactKillEvent>()
            .add_event::<GhostGirlLightFlickerEvent>()
            .add_event::<GhostGirlIgnoredDamageEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_ghost_girl,
                    ghost_girl_select_new_target,
                    ghost_girl_absent_spawn_attempts,
                    ghost_girl_apply_spawn_selection,
                    ghost_girl_update_staring,
                    ghost_girl_non_target_touch_vanish,
                    ghost_girl_disappearing_walk,
                    ghost_girl_vanishing_return_absent,
                    ghost_girl_chasing_update,
                    ghost_girl_contact_instant_kill,
                    ghost_girl_ignore_damage,
                    ghost_girl_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GhostGirlBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GhostGirl;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GhostGirlTargetSensor {
    pub stable_id: u64,
    pub in_facility: bool,
    pub in_ship: bool,
    pub has_line_of_sight: bool,
    pub facing_within_60_degrees: bool,
    pub touching_ghost_girl: bool,
    pub alive: bool,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct GhostGirlTarget {
    pub has_target: bool,
    pub stable_id: u64,
    pub seen_stares: u8,
    pub unseen_stares: u8,
}

impl Default for GhostGirlTarget {
    fn default() -> Self {
        Self {
            has_target: false,
            stable_id: 0,
            seen_stares: 0,
            unseen_stares: 0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct GhostGirlTimers {
    pub absent_ticks: u32,
    pub absent_retry_ticks: u32,
    pub stare_progress_ticks: u32,
    pub stare_required_ticks: u32,
    pub chase_ticks: u32,
    pub chase_teleport_retry_ticks: u32,
}

impl Default for GhostGirlTimers {
    fn default() -> Self {
        Self {
            absent_ticks: 0,
            absent_retry_ticks: 0,
            stare_progress_ticks: 0,
            stare_required_ticks: 0,
            chase_ticks: 0,
            chase_teleport_retry_ticks: 0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct GhostGirlDestination {
    pub has_destination: bool,
    pub position: SimPosition,
}

impl Default for GhostGirlDestination {
    fn default() -> Self {
        Self {
            has_destination: false,
            position: SimPosition {
                x: I32F32::lit("0"),
                y: I32F32::lit("0"),
            },
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GhostGirlState {
    #[default]
    Absent,
    Staring,
    Disappearing,
    Vanishing,
    Chasing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GhostGirlSpawnArea {
    Facility,
    Surface,
}

#[derive(Bundle)]
pub struct GhostGirlBundle {
    pub name: Name,
    pub ghost_girl: GhostGirl,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub state: GhostGirlState,
    pub target: GhostGirlTarget,
    pub timers: GhostGirlTimers,
    pub destination: GhostGirlDestination,
}

impl GhostGirlBundle {
    pub fn new(event: SpawnGhostGirlEvent) -> Self {
        Self {
            name: Name::new(GHOST_GIRL_NAME),
            ghost_girl: GhostGirl,
            position: event.position,
            health: Health::full(GHOST_GIRL_IMMUNE_HEALTH),
            stats: UnitStats {
                move_speed: GHOST_GIRL_STARE_SPEED,
                attack_range: GHOST_GIRL_ATTACK_RANGE,
                attack_damage: GHOST_GIRL_INSTANT_KILL_DAMAGE,
                attack_speed: I32F32::lit("0"),
                watch_range: GHOST_GIRL_WATCH_RANGE,
            },
            state: GhostGirlState::Absent,
            target: GhostGirlTarget {
                has_target: event.target_stable_id != 0,
                stable_id: event.target_stable_id,
                seen_stares: 0,
                unseen_stares: 0,
            },
            timers: GhostGirlTimers::default(),
            destination: GhostGirlDestination::default(),
        }
    }
}

#[derive(Event, Clone, Copy, Debug)]
pub struct SpawnGhostGirlEvent {
    pub position: SimPosition,
    pub target_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct GhostGirlSpawnSelectionEvent {
    pub ghost_girl: Entity,
    pub target_stable_id: u64,
    pub area: GhostGirlSpawnArea,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct GhostGirlSpawnSelectionResolvedEvent {
    pub ghost_girl: Entity,
    pub valid: bool,
    pub position: SimPosition,
    pub disappearing_destination_valid: bool,
    pub disappearing_destination: SimPosition,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct GhostGirlStateChangedEvent {
    pub ghost_girl: Entity,
    pub from: GhostGirlState,
    pub to: GhostGirlState,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct GhostGirlTargetChangedEvent {
    pub ghost_girl: Entity,
    pub previous_stable_id: u64,
    pub next_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct GhostGirlStareRollEvent {
    pub ghost_girl: Entity,
    pub target_stable_id: u64,
    pub chance_basis_points: u32,
    pub rolled_basis_points: u32,
    pub result: GhostGirlStareRollResult,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GhostGirlStareRollResult {
    Chase,
    Disappear,
    NoChange,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct GhostGirlContactKillEvent {
    pub ghost_girl: Entity,
    pub employee: Entity,
    pub employee_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct GhostGirlLightFlickerEvent {
    pub ghost_girl: Entity,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct GhostGirlIgnoredDamageEvent {
    pub ghost_girl: Entity,
    pub source: Entity,
}

fn spawn_ghost_girl(
    mut commands: Commands,
    mut events: EventReader<SpawnGhostGirlEvent>,
    ghost_girls: Query<(), With<GhostGirl>>,
) {
    let mut spawned_count = ghost_girls.iter().count();

    for event in events.read() {
        if spawned_count >= GHOST_GIRL_MAX_SPAWNED {
            break;
        }

        commands.spawn(GhostGirlBundle::new(*event));
        spawned_count += 1;
    }
}

fn ghost_girl_select_new_target(
    mut target_events: EventWriter<GhostGirlTargetChangedEvent>,
    mut ghost_girls: Query<(Entity, &mut GhostGirlTarget), With<GhostGirl>>,
    employees: Query<&GhostGirlTargetSensor, Without<GhostGirl>>,
) {
    for (ghost_girl_entity, mut target) in ghost_girls.iter_mut() {
        if target.has_target && target_is_alive(target.stable_id, &employees) {
            continue;
        }

        let previous = target.stable_id;
        let next = first_alive_employee(&employees);

        target.has_target = next != 0;
        target.stable_id = next;
        target.seen_stares = 0;
        target.unseen_stares = 0;

        if previous != next {
            target_events.send(GhostGirlTargetChangedEvent {
                ghost_girl: ghost_girl_entity,
                previous_stable_id: previous,
                next_stable_id: next,
            });
        }
    }
}

fn ghost_girl_absent_spawn_attempts(
    sim_hz: Res<SimHz>,
    mut selection_events: EventWriter<GhostGirlSpawnSelectionEvent>,
    mut ghost_girls: Query<(Entity, &GhostGirlState, &GhostGirlTarget, &mut GhostGirlTimers), With<GhostGirl>>,
    employees: Query<&GhostGirlTargetSensor, Without<GhostGirl>>,
) {
    let initial_delay_ticks =
        fixed_seconds_to_ticks(GHOST_GIRL_INITIAL_ABSENT_DELAY_SECONDS, sim_hz.0);
    let retry_ticks = fixed_seconds_to_ticks(GHOST_GIRL_ABSENT_RETRY_SECONDS, sim_hz.0);

    for (ghost_girl_entity, state, target, mut timers) in ghost_girls.iter_mut() {
        if *state != GhostGirlState::Absent || !target.has_target {
            timers.absent_ticks = 0;
            timers.absent_retry_ticks = 0;
            continue;
        }

        timers.absent_ticks = timers.absent_ticks.saturating_add(1);

        if timers.absent_ticks < initial_delay_ticks {
            continue;
        }

        if timers.absent_retry_ticks > 0 {
            timers.absent_retry_ticks -= 1;
            continue;
        }

        let Some(sensor) = employee_sensor_by_stable_id(target.stable_id, &employees) else {
            timers.absent_retry_ticks = retry_ticks;
            continue;
        };

        if sensor.in_facility {
            selection_events.send(GhostGirlSpawnSelectionEvent {
                ghost_girl: ghost_girl_entity,
                target_stable_id: target.stable_id,
                area: GhostGirlSpawnArea::Facility,
            });
        } else if sensor.in_ship {
            selection_events.send(GhostGirlSpawnSelectionEvent {
                ghost_girl: ghost_girl_entity,
                target_stable_id: target.stable_id,
                area: GhostGirlSpawnArea::Surface,
            });
        }

        timers.absent_retry_ticks = retry_ticks;
    }
}

fn ghost_girl_apply_spawn_selection(
    sim_hz: Res<SimHz>,
    mut events: EventReader<GhostGirlSpawnSelectionResolvedEvent>,
    mut state_events: EventWriter<GhostGirlStateChangedEvent>,
    mut ghost_girls: Query<(
        Entity,
        &mut SimPosition,
        &mut GhostGirlState,
        &mut UnitStats,
        &mut GhostGirlTimers,
        &mut GhostGirlDestination,
    ), With<GhostGirl>>,
    employees: Query<&GhostGirlTargetSensor, Without<GhostGirl>>,
) {
    for event in events.read() {
        let Ok((entity, mut position, mut state, mut stats, mut timers, mut destination)) =
            ghost_girls.get_mut(event.ghost_girl)
        else {
            continue;
        };

        if *state == GhostGirlState::Disappearing {
            if !event.disappearing_destination_valid {
                set_ghost_girl_state(entity, &mut state, GhostGirlState::Vanishing, &mut state_events);
                continue;
            }

            destination.has_destination = true;
            destination.position = event.disappearing_destination;
            continue;
        }

        if *state != GhostGirlState::Absent || !event.valid {
            continue;
        }

        *position = event.position;
        stats.move_speed = GHOST_GIRL_STARE_SPEED;
        timers.absent_ticks = 0;
        timers.absent_retry_ticks = 0;
        timers.stare_progress_ticks = 0;

        let required_seconds = if target_is_in_facility(&employees) {
            GHOST_GIRL_INDOOR_STARE_SECONDS
        } else {
            GHOST_GIRL_OUTDOOR_STARE_SECONDS
        };
        timers.stare_required_ticks = fixed_seconds_to_ticks(required_seconds, sim_hz.0);

        set_ghost_girl_state(entity, &mut state, GhostGirlState::Staring, &mut state_events);
    }
}

fn ghost_girl_update_staring(
    game_seed: Res<GameSeed>,
    sim_tick: Res<SimTick>,
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<GhostGirlStateChangedEvent>,
    mut roll_events: EventWriter<GhostGirlStareRollEvent>,
    mut ghost_girls: Query<(
        Entity,
        &SimPosition,
        &mut GhostGirlState,
        &mut UnitStats,
        &mut GhostGirlTarget,
        &mut GhostGirlTimers,
    ), With<GhostGirl>>,
    employees: Query<(&SimPosition, &GhostGirlTargetSensor), Without<GhostGirl>>,
) {
    for (entity, position, mut state, mut stats, mut target, mut timers) in ghost_girls.iter_mut() {
        if *state != GhostGirlState::Staring || !target.has_target {
            continue;
        }

        let Some((employee_position, sensor)) =
            employee_by_stable_id(target.stable_id, &employees)
        else {
            continue;
        };

        let distance = axis_distance(*position, employee_position);
        let looking = target_is_looking(distance, sensor);
        let unseen = !sensor.has_line_of_sight;

        if looking {
            target.seen_stares = target.seen_stares.saturating_add(1);
        } else if unseen {
            target.unseen_stares = target.unseen_stares.saturating_add(1);
        }

        if should_roll_chase(&target) {
            let rolled = roll_basis_points(game_seed.0, sim_tick.0, 0x6768_6f73_745f_0001 ^ target.stable_id);
            let chase = rolled < GHOST_GIRL_STARE_CHASE_CHANCE_BASIS_POINTS;
            roll_events.send(GhostGirlStareRollEvent {
                ghost_girl: entity,
                target_stable_id: target.stable_id,
                chance_basis_points: GHOST_GIRL_STARE_CHASE_CHANCE_BASIS_POINTS,
                rolled_basis_points: rolled,
                result: if chase {
                    GhostGirlStareRollResult::Chase
                } else {
                    GhostGirlStareRollResult::NoChange
                },
            });

            if chase {
                enter_chasing(entity, &mut state, &mut stats, &mut timers, sim_hz.0, &mut state_events);
                continue;
            }
        }

        if distance <= GHOST_GIRL_CLOSE_DISAPPEAR_RANGE
            && target.seen_stares <= GHOST_GIRL_CLOSE_DISAPPEAR_SEEN_STARES_MAX
        {
            let rolled = roll_basis_points(game_seed.0, sim_tick.0, 0x6768_6f73_745f_0002 ^ target.stable_id);
            let disappear = rolled < GHOST_GIRL_CLOSE_DISAPPEAR_CHANCE_BASIS_POINTS;
            roll_events.send(GhostGirlStareRollEvent {
                ghost_girl: entity,
                target_stable_id: target.stable_id,
                chance_basis_points: GHOST_GIRL_CLOSE_DISAPPEAR_CHANCE_BASIS_POINTS,
                rolled_basis_points: rolled,
                result: if disappear {
                    GhostGirlStareRollResult::Disappear
                } else {
                    GhostGirlStareRollResult::NoChange
                },
            });

            if disappear {
                enter_disappearing(entity, &mut state, &mut stats, &mut state_events);
                continue;
            }
        }

        if distance <= GHOST_GIRL_CLOSE_CHASE_RANGE
            && target.seen_stares >= GHOST_GIRL_CLOSE_CHASE_SEEN_STARES_MIN
        {
            let rolled = roll_basis_points(game_seed.0, sim_tick.0, 0x6768_6f73_745f_0003 ^ target.stable_id);
            let chase = rolled < GHOST_GIRL_CLOSE_CHASE_CHANCE_BASIS_POINTS;
            roll_events.send(GhostGirlStareRollEvent {
                ghost_girl: entity,
                target_stable_id: target.stable_id,
                chance_basis_points: GHOST_GIRL_CLOSE_CHASE_CHANCE_BASIS_POINTS,
                rolled_basis_points: rolled,
                result: if chase {
                    GhostGirlStareRollResult::Chase
                } else {
                    GhostGirlStareRollResult::NoChange
                },
            });

            if chase {
                enter_chasing(entity, &mut state, &mut stats, &mut timers, sim_hz.0, &mut state_events);
            } else {
                set_ghost_girl_state(entity, &mut state, GhostGirlState::Vanishing, &mut state_events);
            }
            continue;
        }

        let increment = if unseen {
            GHOST_GIRL_LOS_LOST_STARE_RATE
        } else if looking {
            GHOST_GIRL_LOOKING_STARE_RATE
        } else {
            I32F32::lit("1")
        };

        timers.stare_progress_ticks = timers
            .stare_progress_ticks
            .saturating_add(fixed_ticks_scaled(1, increment));

        if timers.stare_progress_ticks >= timers.stare_required_ticks {
            enter_disappearing(entity, &mut state, &mut stats, &mut state_events);
        }
    }
}

fn ghost_girl_non_target_touch_vanish(
    mut state_events: EventWriter<GhostGirlStateChangedEvent>,
    mut ghost_girls: Query<(Entity, &mut GhostGirlState, &GhostGirlTarget), With<GhostGirl>>,
    employees: Query<&GhostGirlTargetSensor, Without<GhostGirl>>,
) {
    for (entity, mut state, target) in ghost_girls.iter_mut() {
        if *state != GhostGirlState::Staring && *state != GhostGirlState::Disappearing {
            continue;
        }

        let touched_by_non_target = employees.iter().any(|sensor| {
            sensor.touching_ghost_girl && sensor.stable_id != target.stable_id
        });

        if touched_by_non_target {
            set_ghost_girl_state(entity, &mut state, GhostGirlState::Vanishing, &mut state_events);
        }
    }
}

fn ghost_girl_disappearing_walk(
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<GhostGirlStateChangedEvent>,
    mut ghost_girls: Query<(
        Entity,
        &mut SimPosition,
        &mut GhostGirlState,
        &GhostGirlTarget,
        &UnitStats,
        &GhostGirlDestination,
    ), With<GhostGirl>>,
    employees: Query<(&SimPosition, &GhostGirlTargetSensor), Without<GhostGirl>>,
) {
    for (entity, mut position, mut state, target, stats, destination) in ghost_girls.iter_mut() {
        if *state != GhostGirlState::Disappearing {
            continue;
        }

        if !destination.has_destination {
            set_ghost_girl_state(entity, &mut state, GhostGirlState::Vanishing, &mut state_events);
            continue;
        }

        move_axis_toward(&mut position, destination.position, stats.move_speed / sim_hz.0);

        let Some((employee_position, sensor)) =
            employee_by_stable_id(target.stable_id, &employees)
        else {
            set_ghost_girl_state(entity, &mut state, GhostGirlState::Vanishing, &mut state_events);
            continue;
        };

        if !sensor.has_line_of_sight
            || axis_distance(*position, employee_position) <= GHOST_GIRL_DISAPPEAR_TARGET_RANGE
            || axis_distance(*position, destination.position) <= GHOST_GIRL_DISAPPEAR_DESTINATION_RANGE
        {
            set_ghost_girl_state(entity, &mut state, GhostGirlState::Vanishing, &mut state_events);
        }
    }
}

fn ghost_girl_vanishing_return_absent(
    mut flicker_events: EventWriter<GhostGirlLightFlickerEvent>,
    mut state_events: EventWriter<GhostGirlStateChangedEvent>,
    mut ghost_girls: Query<(
        Entity,
        &mut GhostGirlState,
        &mut UnitStats,
        &mut GhostGirlTimers,
        &mut GhostGirlDestination,
    ), With<GhostGirl>>,
) {
    for (entity, mut state, mut stats, mut timers, mut destination) in ghost_girls.iter_mut() {
        if *state != GhostGirlState::Vanishing {
            continue;
        }

        flicker_events.send(GhostGirlLightFlickerEvent { ghost_girl: entity });
        stats.move_speed = GHOST_GIRL_STARE_SPEED;
        timers.absent_ticks = 0;
        timers.absent_retry_ticks = 0;
        timers.stare_progress_ticks = 0;
        timers.chase_ticks = 0;
        timers.chase_teleport_retry_ticks = 0;
        destination.has_destination = false;
        set_ghost_girl_state(entity, &mut state, GhostGirlState::Absent, &mut state_events);
    }
}

fn ghost_girl_chasing_update(
    sim_hz: Res<SimHz>,
    mut selection_events: EventWriter<GhostGirlSpawnSelectionEvent>,
    mut state_events: EventWriter<GhostGirlStateChangedEvent>,
    mut ghost_girls: Query<(
        Entity,
        &mut SimPosition,
        &mut GhostGirlState,
        &GhostGirlTarget,
        &UnitStats,
        &mut GhostGirlTimers,
    ), With<GhostGirl>>,
    employees: Query<(&SimPosition, &GhostGirlTargetSensor), Without<GhostGirl>>,
) {
    let chase_duration_ticks = fixed_seconds_to_ticks(GHOST_GIRL_CHASE_SECONDS, sim_hz.0);
    let teleport_retry_ticks =
        fixed_seconds_to_ticks(GHOST_GIRL_CHASE_TELEPORT_RETRY_SECONDS, sim_hz.0);

    for (entity, mut position, mut state, target, stats, mut timers) in ghost_girls.iter_mut() {
        if *state != GhostGirlState::Chasing || !target.has_target {
            continue;
        }

        let Some((employee_position, sensor)) =
            employee_by_stable_id(target.stable_id, &employees)
        else {
            set_ghost_girl_state(entity, &mut state, GhostGirlState::Absent, &mut state_events);
            continue;
        };

        timers.chase_ticks = timers.chase_ticks.saturating_add(1);
        move_axis_toward(&mut position, employee_position, stats.move_speed / sim_hz.0);

        if timers.chase_ticks >= chase_duration_ticks
            || axis_distance(*position, employee_position) > GHOST_GIRL_CHASE_END_DISTANCE
        {
            set_ghost_girl_state(entity, &mut state, GhostGirlState::Absent, &mut state_events);
            continue;
        }

        if timers.chase_teleport_retry_ticks > 0 {
            timers.chase_teleport_retry_ticks -= 1;
            continue;
        }

        let looking = target_is_looking(axis_distance(*position, employee_position), sensor);
        if !looking {
            let area = if sensor.in_facility {
                Some(GhostGirlSpawnArea::Facility)
            } else if sensor.in_ship {
                Some(GhostGirlSpawnArea::Surface)
            } else {
                None
            };

            if let Some(area) = area {
                selection_events.send(GhostGirlSpawnSelectionEvent {
                    ghost_girl: entity,
                    target_stable_id: target.stable_id,
                    area,
                });
            }
        }

        timers.chase_teleport_retry_ticks = teleport_retry_ticks;
    }
}

fn ghost_girl_contact_instant_kill(
    mut damage_events: EventWriter<IncomingDamageEvent>,
    mut contact_events: EventWriter<GhostGirlContactKillEvent>,
    ghost_girls: Query<(Entity, &GhostGirlState, &GhostGirlTarget), With<GhostGirl>>,
    employees: Query<(Entity, &GhostGirlTargetSensor), Without<GhostGirl>>,
) {
    for (ghost_girl_entity, state, target) in ghost_girls.iter() {
        if *state != GhostGirlState::Chasing || !target.has_target {
            continue;
        }

        for (employee_entity, sensor) in employees.iter() {
            if sensor.stable_id != target.stable_id || !sensor.touching_ghost_girl {
                continue;
            }

            contact_events.send(GhostGirlContactKillEvent {
                ghost_girl: ghost_girl_entity,
                employee: employee_entity,
                employee_stable_id: sensor.stable_id,
            });
            damage_events.send(IncomingDamageEvent {
                target: employee_entity,
                raw_amount: GHOST_GIRL_INSTANT_KILL_DAMAGE,
                damage_type: DamageType::Standard,
                source: ghost_girl_entity,
            });
        }
    }
}

fn ghost_girl_ignore_damage(
    mut damage_events: EventReader<IncomingDamageEvent>,
    mut ignored_events: EventWriter<GhostGirlIgnoredDamageEvent>,
    ghost_girls: Query<(), With<GhostGirl>>,
) {
    for event in damage_events.read() {
        if ghost_girls.get(event.target).is_err() {
            continue;
        }

        ignored_events.send(GhostGirlIgnoredDamageEvent {
            ghost_girl: event.target,
            source: event.source,
        });
    }
}

fn ghost_girl_checksum(
    mut checksum: ResMut<SimChecksumState>,
    ghost_girls: Query<(
        &SimPosition,
        &Health,
        &UnitStats,
        &GhostGirlState,
        &GhostGirlTarget,
        &GhostGirlTimers,
        &GhostGirlDestination,
    ), With<GhostGirl>>,
) {
    for (position, health, stats, state, target, timers, destination) in ghost_girls.iter() {
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);
        checksum.accumulate(ghost_girl_state_bits(*state));
        checksum.accumulate(target.has_target as u64);
        checksum.accumulate(target.stable_id);
        checksum.accumulate(target.seen_stares as u64);
        checksum.accumulate(target.unseen_stares as u64);
        checksum.accumulate(timers.absent_ticks as u64);
        checksum.accumulate(timers.absent_retry_ticks as u64);
        checksum.accumulate(timers.stare_progress_ticks as u64);
        checksum.accumulate(timers.stare_required_ticks as u64);
        checksum.accumulate(timers.chase_ticks as u64);
        checksum.accumulate(timers.chase_teleport_retry_ticks as u64);
        checksum.accumulate(destination.has_destination as u64);
        checksum.accumulate(destination.position.x.to_bits() as u64);
        checksum.accumulate(destination.position.y.to_bits() as u64);
    }
}

fn enter_disappearing(
    ghost_girl: Entity,
    state: &mut GhostGirlState,
    stats: &mut UnitStats,
    events: &mut EventWriter<GhostGirlStateChangedEvent>,
) {
    stats.move_speed = GHOST_GIRL_DISAPPEAR_SPEED;
    set_ghost_girl_state(ghost_girl, state, GhostGirlState::Disappearing, events);
}

fn enter_chasing(
    ghost_girl: Entity,
    state: &mut GhostGirlState,
    stats: &mut UnitStats,
    timers: &mut GhostGirlTimers,
    sim_hz: I32F32,
    events: &mut EventWriter<GhostGirlStateChangedEvent>,
) {
    stats.move_speed = GHOST_GIRL_CHASE_SPEED;
    timers.chase_ticks = 0;
    timers.chase_teleport_retry_ticks =
        fixed_seconds_to_ticks(GHOST_GIRL_CHASE_TELEPORT_RETRY_SECONDS, sim_hz);
    set_ghost_girl_state(ghost_girl, state, GhostGirlState::Chasing, events);
}

fn should_roll_chase(target: &GhostGirlTarget) -> bool {
    target.seen_stares >= GHOST_GIRL_STARE_CHASE_SEEN_THRESHOLD
        || target.unseen_stares >= GHOST_GIRL_STARE_CHASE_UNSEEN_THRESHOLD
}

fn target_is_looking(distance: I32F32, sensor: GhostGirlTargetSensor) -> bool {
    if !sensor.has_line_of_sight {
        return false;
    }

    if distance <= GHOST_GIRL_LOOKING_CLOSE_RANGE {
        return true;
    }

    distance <= GHOST_GIRL_LOOKING_FAR_RANGE && sensor.facing_within_60_degrees
}

fn first_alive_employee(employees: &Query<&GhostGirlTargetSensor, Without<GhostGirl>>) -> u64 {
    let mut best = 0;

    for sensor in employees.iter() {
        if !sensor.alive {
            continue;
        }

        if best == 0 || sensor.stable_id < best {
            best = sensor.stable_id;
        }
    }

    best
}

fn target_is_alive(stable_id: u64, employees: &Query<&GhostGirlTargetSensor, Without<GhostGirl>>) -> bool {
    employees
        .iter()
        .any(|sensor| sensor.stable_id == stable_id && sensor.alive)
}

fn employee_sensor_by_stable_id(
    stable_id: u64,
    employees: &Query<&GhostGirlTargetSensor, Without<GhostGirl>>,
) -> Option<GhostGirlTargetSensor> {
    for sensor in employees.iter() {
        if sensor.stable_id == stable_id {
            return Some(*sensor);
        }
    }

    None
}

fn employee_by_stable_id(
    stable_id: u64,
    employees: &Query<(&SimPosition, &GhostGirlTargetSensor), Without<GhostGirl>>,
) -> Option<(SimPosition, GhostGirlTargetSensor)> {
    for (position, sensor) in employees.iter() {
        if sensor.stable_id == stable_id {
            return Some((*position, *sensor));
        }
    }

    None
}

fn target_is_in_facility(employees: &Query<&GhostGirlTargetSensor, Without<GhostGirl>>) -> bool {
    employees.iter().any(|sensor| sensor.in_facility)
}

fn set_ghost_girl_state(
    ghost_girl: Entity,
    state: &mut GhostGirlState,
    next: GhostGirlState,
    events: &mut EventWriter<GhostGirlStateChangedEvent>,
) {
    if *state == next {
        return;
    }

    let previous = *state;
    *state = next;
    events.send(GhostGirlStateChangedEvent {
        ghost_girl,
        from: previous,
        to: next,
    });
}

fn move_axis_toward(position: &mut SimPosition, destination: SimPosition, max_step: I32F32) {
    let dx = destination.x - position.x;
    let dy = destination.y - position.y;

    if fixed_abs(dx) >= fixed_abs(dy) {
        position.x += fixed_clamp(dx, -max_step, max_step);
    } else {
        position.y += fixed_clamp(dy, -max_step, max_step);
    }
}

fn axis_distance(a: SimPosition, b: SimPosition) -> I32F32 {
    fixed_abs(a.x - b.x) + fixed_abs(a.y - b.y)
}

fn fixed_seconds_to_ticks(seconds: I32F32, sim_hz: I32F32) -> u32 {
    let ticks = seconds * sim_hz;
    let whole_ticks: u32 = ticks.to_num();

    if whole_ticks == 0 {
        1
    } else {
        whole_ticks
    }
}

fn fixed_ticks_scaled(ticks: u32, scale: I32F32) -> u32 {
    let scaled = I32F32::from_num(ticks) * scale;
    let whole_ticks: u32 = scaled.to_num();

    if whole_ticks == 0 {
        1
    } else {
        whole_ticks
    }
}

fn fixed_abs(value: I32F32) -> I32F32 {
    if value < I32F32::lit("0") {
        -value
    } else {
        value
    }
}

fn fixed_clamp(value: I32F32, min: I32F32, max: I32F32) -> I32F32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

fn roll_basis_points(game_seed: u64, tick: u64, salt: u64) -> u32 {
    let mut rng = tick_rng(game_seed, tick, salt);
    rng.next_u32() % 10000
}

fn ghost_girl_state_bits(state: GhostGirlState) -> u64 {
    match state {
        GhostGirlState::Absent => 0,
        GhostGirlState::Staring => 1,
        GhostGirlState::Disappearing => 2,
        GhostGirlState::Vanishing => 3,
        GhostGirlState::Chasing => 4,
    }
}
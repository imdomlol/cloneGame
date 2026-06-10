// Sources: vault/harmless_entity_pages/backwater_gunkfish.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{
    DamageType, EntityKilledEvent, Health, IncomingDamageEvent, SimChecksumState, SimHz,
    SimPosition, SimTick, UnitStats,
};

pub const BACKWATER_GUNKFISH_ID: &str = "backwater_gunkfish";
pub const BACKWATER_GUNKFISH_NAME: &str = "Backwater Gunkfish";
pub const BACKWATER_GUNKFISH_TYPE: &str = "harmless_entity_pages";
pub const BACKWATER_GUNKFISH_SUBTYPE: &str = "entity";
pub const BACKWATER_GUNKFISH_SOURCE_URL: &str =
    "https://lethal-company.fandom.com/wiki/Backwater_Gunkfish";
pub const BACKWATER_GUNKFISH_SOURCE_REVISION: u32 = 21453;
pub const BACKWATER_GUNKFISH_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const BACKWATER_GUNKFISH_CONFIDENCE_BASIS_POINTS: u16 = 94;

pub const BACKWATER_GUNKFISH_DWELLS: &str = "Inside";
pub const BACKWATER_GUNKFISH_DANGER: &str = "0%";
pub const BACKWATER_GUNKFISH_SCIENTIFIC_NAME: &str = "Infectum linteum";
pub const BACKWATER_GUNKFISH_HP: I32F32 = I32F32::lit("5");
pub const BACKWATER_GUNKFISH_POWER_LEVEL: I32F32 = I32F32::lit("0.5");
pub const BACKWATER_GUNKFISH_MAX_SPAWNED: usize = 6;
pub const BACKWATER_GUNKFISH_ATTACK_DAMAGE: I32F32 = I32F32::lit("0");
pub const BACKWATER_GUNKFISH_STUN_MULTIPLIER: I32F32 = I32F32::lit("1.0");
pub const BACKWATER_GUNKFISH_CAN_SEE_THROUGH_FOG: bool = false;
pub const BACKWATER_GUNKFISH_SPAWN_DELAY_SECONDS: u16 = 12;
pub const BACKWATER_GUNKFISH_DOOR_SPEED_MULTIPLIER: I32F32 = I32F32::lit("0.8");
pub const BACKWATER_GUNKFISH_ZAP_GUN_DIFFICULTY: I32F32 = I32F32::lit("0.5");
pub const BACKWATER_GUNKFISH_INTERNAL_NAME: &str = "Stingray";
pub const BACKWATER_GUNKFISH_PIP_SIZE: &str = "Small";

pub const BACKWATER_GUNKFISH_DEPENDS_ON: [&str; 2] = ["entity", "lethal_company"];
pub const BACKWATER_GUNKFISH_FRONTMATTER_BEHAVIOR: [&str; 3] =
    ["Cloak", "Cloaked", "Chase"];

pub const GUNKFISH_EMPLOYEE_AVOID_DISTANCE: I32F32 = I32F32::lit("8");
pub const GUNKFISH_SPOT_FOV_DEGREES: u16 = 110;
pub const GUNKFISH_SPOT_RANGE: I32F32 = I32F32::lit("25");
pub const GUNKFISH_SPOT_SECONDS: I32F32 = I32F32::lit("0.35");
pub const GUNKFISH_SEARCH_IGNORE_EMPLOYEE_RADIUS: I32F32 = I32F32::lit("4");
pub const GUNKFISH_FLEE_SPEED: I32F32 = I32F32::lit("11");
pub const GUNKFISH_SLIME_DECAL_DISTANCE: I32F32 = I32F32::lit("2.15");
pub const GUNKFISH_RETREAT_NODE_LIMIT: usize = 50;
pub const GUNKFISH_RETREAT_MIN_DISTANCE: I32F32 = I32F32::lit("40");
pub const GUNKFISH_MAX_FLEE_SECONDS: I32F32 = I32F32::lit("10");
pub const GUNKFISH_SAFE_AFTER_HIT_SECONDS: I32F32 = I32F32::lit("10");
pub const GUNKFISH_CLOAK_RESET_RADIUS: I32F32 = I32F32::lit("12");
pub const GUNKFISH_CLOAK_RELOCATE_SECONDS: I32F32 = I32F32::lit("30");
pub const GUNKFISH_CLOAK_CLOSEST_EMPLOYEE_RANGE: I32F32 = I32F32::lit("30");
pub const GUNKFISH_CLOAK_TIMER_RESET_SECONDS: I32F32 = I32F32::lit("15");
pub const GUNKFISH_LUNGE_WATCH_RADIUS: I32F32 = I32F32::lit("5");
pub const GUNKFISH_WATCH_TO_LUNGE_SECONDS: I32F32 = I32F32::lit("10");
pub const GUNKFISH_COLLISION_LUNGE_RADIUS: I32F32 = I32F32::lit("1.5");
pub const GUNKFISH_HIDING_REACHED_RADIUS: I32F32 = I32F32::lit("0.5");
pub const GUNKFISH_FIXED_FOV_DOT_SQUARED_MIN: I32F32 = I32F32::lit("0.329");

pub const BACKWATER_GUNKFISH_BEHAVIORAL_MECHANICS: [BackwaterGunkfishBehaviorRule; 15] = [
    BackwaterGunkfishBehaviorRule {
        condition: "the gunkfish is in state 0 while searching",
        outcome: "it looks for a suitable hiding spot by prioritizing the node closest to an employee while staying more than 8 units away, or by favoring positions near scrap if no player is detected, or by using a fallback position far from the main entrance",
    },
    BackwaterGunkfishBehaviorRule {
        condition: "the searching gunkfish reaches a hiding spot OR spots a player while moving to that spot within a 110 degree FOV and 25-unit range for at least 0.35 seconds",
        outcome: "it switches to Cloaked",
    },
    BackwaterGunkfishBehaviorRule {
        condition: "an employee is within 4 units during searching",
        outcome: "that employee is not detected",
    },
    BackwaterGunkfishBehaviorRule {
        condition: "the gunkfish enters lunge mode",
        outcome: "it charges toward the target employee at high speed, spits slime on them, and immediately enters fleeing",
    },
    BackwaterGunkfishBehaviorRule {
        condition: "the gunkfish is fleeing",
        outcome: "it whines, accelerates up to 11 speed, and leaves a slime decal every 2.15 units from the previous slime position",
    },
    BackwaterGunkfishBehaviorRule {
        condition: "the fleeing gunkfish is selecting a retreat point",
        outcome: "it chooses the furthest node from the player, up to 50 nodes away and at least 40 units distant",
    },
    BackwaterGunkfishBehaviorRule {
        condition: "the gunkfish has been fleeing for 10 seconds and has not been hit for 10 seconds",
        outcome: "it stops whining, returns to flopping, and slows down",
    },
    BackwaterGunkfishBehaviorRule {
        condition: "the fleeing gunkfish reaches its hiding spot",
        outcome: "it transitions back to Cloaked",
    },
    BackwaterGunkfishBehaviorRule {
        condition: "the gunkfish is Cloaked",
        outcome: "it is invisible, stationary, and tracks how long it has been hiding",
    },
    BackwaterGunkfishBehaviorRule {
        condition: "an employee is inside the facility within 12 units while the gunkfish is Cloaked",
        outcome: "the hide timer resets continuously and the gunkfish can remain hidden indefinitely while employees stay nearby",
    },
    BackwaterGunkfishBehaviorRule {
        condition: "no employee has been within 12 units for 30 seconds while the gunkfish is Cloaked",
        outcome: "it checks whether the closest employee is within 30 units; if not, it searches for a new hiding spot, and if so, it resets the timer to 15",
    },
    BackwaterGunkfishBehaviorRule {
        condition: "an employee comes within 5 units while the gunkfish is Cloaked and has been watched for more than 10 seconds",
        outcome: "it switches to lunge mode",
    },
    BackwaterGunkfishBehaviorRule {
        condition: "more employees are nearby while the gunkfish is Cloaked",
        outcome: "the watch timer accumulates faster",
    },
    BackwaterGunkfishBehaviorRule {
        condition: "an employee walks into the gunkfish within 1.5 units while it is Cloaked",
        outcome: "it switches to lunge mode",
    },
    BackwaterGunkfishBehaviorRule {
        condition: "the gunkfish is hit while Cloaked",
        outcome: "it switches to fleeing",
    },
];

pub struct BackwaterGunkfishPlugin;

impl Plugin for BackwaterGunkfishPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnBackwaterGunkfishEvent>()
            .add_event::<BackwaterGunkfishStateChangedEvent>()
            .add_event::<BackwaterGunkfishSlimeSpitEvent>()
            .add_event::<BackwaterGunkfishSlimeDecalEvent>()
            .add_event::<BackwaterGunkfishWhineEvent>()
            .add_event::<BackwaterGunkfishZapGunDifficultyEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_backwater_gunkfish,
                    backwater_gunkfish_select_search_hiding_spot,
                    backwater_gunkfish_search_to_cloaked,
                    backwater_gunkfish_lunge,
                    backwater_gunkfish_flee,
                    backwater_gunkfish_select_retreat_point,
                    backwater_gunkfish_stop_fleeing_when_safe,
                    backwater_gunkfish_flee_reaches_hiding_spot,
                    backwater_gunkfish_cloaked_hide_timer,
                    backwater_gunkfish_cloaked_relocation_check,
                    backwater_gunkfish_cloaked_watch_to_lunge,
                    backwater_gunkfish_cloaked_collision_to_lunge,
                    backwater_gunkfish_hit_while_cloaked,
                    backwater_gunkfish_apply_damage,
                    backwater_gunkfish_report_zap_gun_difficulty,
                    backwater_gunkfish_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BackwaterGunkfishBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BackwaterGunkfish;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BackwaterGunkfishEmployeeTarget {
    pub stable_id: u64,
    pub inside_facility: bool,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BackwaterGunkfishScrapAnchor;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BackwaterGunkfishMainEntrance;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BackwaterGunkfishInteriorNode {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BackwaterGunkfishFacing {
    pub x: I32F32,
    pub y: I32F32,
}

impl Default for BackwaterGunkfishFacing {
    fn default() -> Self {
        Self {
            x: I32F32::lit("1"),
            y: I32F32::lit("0"),
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BackwaterGunkfishTimers {
    pub spawn_delay_ticks: u32,
    pub spotting_ticks: u32,
    pub hide_ticks: u32,
    pub watch_ticks: u32,
    pub flee_ticks: u32,
    pub ticks_since_hit: u32,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BackwaterGunkfishMovement {
    pub hiding_spot: SimPosition,
    pub fallback_position: SimPosition,
    pub last_slime_position: SimPosition,
    pub target_employee: Option<Entity>,
    pub target_employee_stable_id: u64,
    pub whining: bool,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BackwaterGunkfishState {
    #[default]
    Searching,
    Cloaked,
    Lunge,
    Fleeing,
    Flopping,
}

#[derive(Bundle)]
pub struct BackwaterGunkfishBundle {
    pub name: Name,
    pub gunkfish: BackwaterGunkfish,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub facing: BackwaterGunkfishFacing,
    pub state: BackwaterGunkfishState,
    pub timers: BackwaterGunkfishTimers,
    pub movement: BackwaterGunkfishMovement,
}

impl BackwaterGunkfishBundle {
    pub fn new(event: SpawnBackwaterGunkfishEvent, sim_hz: I32F32) -> Self {
        Self {
            name: Name::new(BACKWATER_GUNKFISH_NAME),
            gunkfish: BackwaterGunkfish,
            position: event.position,
            health: Health::full(BACKWATER_GUNKFISH_HP),
            stats: UnitStats {
                move_speed: event.search_move_speed,
                attack_range: GUNKFISH_LUNGE_WATCH_RADIUS,
                attack_damage: BACKWATER_GUNKFISH_ATTACK_DAMAGE,
                attack_speed: I32F32::lit("0"),
                watch_range: GUNKFISH_CLOAK_RESET_RADIUS,
            },
            facing: event.facing,
            state: BackwaterGunkfishState::Searching,
            timers: BackwaterGunkfishTimers {
                spawn_delay_ticks: seconds_to_ticks(I32F32::from_num(
                    BACKWATER_GUNKFISH_SPAWN_DELAY_SECONDS,
                ), sim_hz),
                spotting_ticks: 0,
                hide_ticks: 0,
                watch_ticks: 0,
                flee_ticks: 0,
                ticks_since_hit: seconds_to_ticks(GUNKFISH_SAFE_AFTER_HIT_SECONDS, sim_hz),
            },
            movement: BackwaterGunkfishMovement {
                hiding_spot: event.initial_hiding_spot,
                fallback_position: event.fallback_position,
                last_slime_position: event.position,
                target_employee: None,
                target_employee_stable_id: 0,
                whining: false,
            },
        }
    }
}

#[derive(Event, Clone, Copy, Debug)]
pub struct SpawnBackwaterGunkfishEvent {
    pub position: SimPosition,
    pub initial_hiding_spot: SimPosition,
    pub fallback_position: SimPosition,
    pub facing: BackwaterGunkfishFacing,
    pub search_move_speed: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BackwaterGunkfishStateChangedEvent {
    pub gunkfish: Entity,
    pub from: BackwaterGunkfishState,
    pub to: BackwaterGunkfishState,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct BackwaterGunkfishSlimeSpitEvent {
    pub gunkfish: Entity,
    pub target: Entity,
    pub target_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct BackwaterGunkfishSlimeDecalEvent {
    pub gunkfish: Entity,
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BackwaterGunkfishWhineEvent {
    pub gunkfish: Entity,
    pub active: bool,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct BackwaterGunkfishZapGunDifficultyEvent {
    pub gunkfish: Entity,
    pub difficulty: I32F32,
}

fn spawn_backwater_gunkfish(
    mut commands: Commands,
    mut events: EventReader<SpawnBackwaterGunkfishEvent>,
    sim_hz: Res<SimHz>,
    gunkfish: Query<(), With<BackwaterGunkfish>>,
) {
    let mut spawned_count = gunkfish.iter().count();

    for event in events.read() {
        if spawned_count >= BACKWATER_GUNKFISH_MAX_SPAWNED {
            break;
        }

        commands.spawn(BackwaterGunkfishBundle::new(*event, sim_hz.0));
        spawned_count += 1;
    }
}

fn backwater_gunkfish_select_search_hiding_spot(
    mut gunkfish: Query<
        (
            &BackwaterGunkfishState,
            &mut BackwaterGunkfishMovement,
            &SimPosition,
        ),
        With<BackwaterGunkfish>,
    >,
    employees: Query<(&SimPosition, &BackwaterGunkfishEmployeeTarget)>,
    scrap: Query<&SimPosition, With<BackwaterGunkfishScrapAnchor>>,
    nodes: Query<(&SimPosition, &BackwaterGunkfishInteriorNode)>,
    entrances: Query<&SimPosition, With<BackwaterGunkfishMainEntrance>>,
) {
    for (state, mut movement, position) in gunkfish.iter_mut() {
        if *state != BackwaterGunkfishState::Searching {
            continue;
        }

        if let Some(hiding_spot) = hiding_spot_near_employee_far_enough(&employees, &nodes) {
            movement.hiding_spot = hiding_spot;
            continue;
        }

        if let Some(hiding_spot) = hiding_spot_near_scrap(&scrap, &nodes) {
            movement.hiding_spot = hiding_spot;
            continue;
        }

        movement.hiding_spot =
            fallback_far_from_entrance(position, &nodes, &entrances).unwrap_or(movement.fallback_position);
    }
}

fn backwater_gunkfish_search_to_cloaked(
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<BackwaterGunkfishStateChangedEvent>,
    mut gunkfish: Query<
        (
            Entity,
            &SimPosition,
            &BackwaterGunkfishFacing,
            &mut BackwaterGunkfishState,
            &mut BackwaterGunkfishTimers,
            &mut BackwaterGunkfishMovement,
        ),
        With<BackwaterGunkfish>,
    >,
    employees: Query<(Entity, &SimPosition, &BackwaterGunkfishEmployeeTarget)>,
) {
    let hiding_reached_radius_squared = GUNKFISH_HIDING_REACHED_RADIUS * GUNKFISH_HIDING_REACHED_RADIUS;
    let spotting_ticks_needed = seconds_to_ticks(GUNKFISH_SPOT_SECONDS, sim_hz.0);

    for (entity, position, facing, mut state, mut timers, mut movement) in gunkfish.iter_mut() {
        if *state != BackwaterGunkfishState::Searching {
            continue;
        }

        if timers.spawn_delay_ticks > 0 {
            timers.spawn_delay_ticks -= 1;
            continue;
        }

        if distance_squared(*position, movement.hiding_spot) <= hiding_reached_radius_squared {
            set_state(
                entity,
                &mut state,
                BackwaterGunkfishState::Cloaked,
                &mut state_events,
            );
            timers.hide_ticks = 0;
            timers.watch_ticks = 0;
            continue;
        }

        if let Some((target, stable_id)) = visible_employee(entity, *position, *facing, &employees) {
            timers.spotting_ticks = timers.spotting_ticks.saturating_add(1);
            movement.target_employee = Some(target);
            movement.target_employee_stable_id = stable_id;

            if timers.spotting_ticks >= spotting_ticks_needed {
                set_state(
                    entity,
                    &mut state,
                    BackwaterGunkfishState::Cloaked,
                    &mut state_events,
                );
                timers.hide_ticks = 0;
                timers.watch_ticks = 0;
            }
        } else {
            timers.spotting_ticks = 0;
            movement.target_employee = None;
            movement.target_employee_stable_id = 0;
        }
    }
}

fn backwater_gunkfish_lunge(
    mut state_events: EventWriter<BackwaterGunkfishStateChangedEvent>,
    mut spit_events: EventWriter<BackwaterGunkfishSlimeSpitEvent>,
    mut gunkfish: Query<
        (
            Entity,
            &mut BackwaterGunkfishState,
            &mut UnitStats,
            &mut BackwaterGunkfishMovement,
        ),
        With<BackwaterGunkfish>,
    >,
) {
    for (entity, mut state, mut stats, mut movement) in gunkfish.iter_mut() {
        if *state != BackwaterGunkfishState::Lunge {
            continue;
        }

        stats.move_speed = GUNKFISH_FLEE_SPEED;

        if let Some(target) = movement.target_employee {
            spit_events.send(BackwaterGunkfishSlimeSpitEvent {
                gunkfish: entity,
                target,
                target_stable_id: movement.target_employee_stable_id,
            });
        }

        movement.whining = false;
        set_state(
            entity,
            &mut state,
            BackwaterGunkfishState::Fleeing,
            &mut state_events,
        );
    }
}

fn backwater_gunkfish_flee(
    mut whine_events: EventWriter<BackwaterGunkfishWhineEvent>,
    mut slime_events: EventWriter<BackwaterGunkfishSlimeDecalEvent>,
    mut gunkfish: Query<
        (
            Entity,
            &BackwaterGunkfishState,
            &SimPosition,
            &mut UnitStats,
            &mut BackwaterGunkfishTimers,
            &mut BackwaterGunkfishMovement,
        ),
        With<BackwaterGunkfish>,
    >,
) {
    let slime_distance_squared = GUNKFISH_SLIME_DECAL_DISTANCE * GUNKFISH_SLIME_DECAL_DISTANCE;

    for (entity, state, position, mut stats, mut timers, mut movement) in gunkfish.iter_mut() {
        if *state != BackwaterGunkfishState::Fleeing {
            continue;
        }

        timers.flee_ticks = timers.flee_ticks.saturating_add(1);
        timers.ticks_since_hit = timers.ticks_since_hit.saturating_add(1);
        stats.move_speed = GUNKFISH_FLEE_SPEED;

        if !movement.whining {
            movement.whining = true;
            whine_events.send(BackwaterGunkfishWhineEvent {
                gunkfish: entity,
                active: true,
            });
        }

        if distance_squared(*position, movement.last_slime_position) >= slime_distance_squared {
            movement.last_slime_position = *position;
            slime_events.send(BackwaterGunkfishSlimeDecalEvent {
                gunkfish: entity,
                position: *position,
            });
        }
    }
}

fn backwater_gunkfish_select_retreat_point(
    mut gunkfish: Query<
        (
            &BackwaterGunkfishState,
            &mut BackwaterGunkfishMovement,
            &SimPosition,
        ),
        With<BackwaterGunkfish>,
    >,
    employees: Query<&SimPosition, With<BackwaterGunkfishEmployeeTarget>>,
    nodes: Query<(&SimPosition, &BackwaterGunkfishInteriorNode)>,
) {
    let min_distance_squared = GUNKFISH_RETREAT_MIN_DISTANCE * GUNKFISH_RETREAT_MIN_DISTANCE;

    for (state, mut movement, position) in gunkfish.iter_mut() {
        if *state != BackwaterGunkfishState::Fleeing {
            continue;
        }

        let Some(player_position) = closest_position(*position, &employees) else {
            continue;
        };

        let mut selected: Option<(u64, SimPosition, I32F32)> = None;

        for (index, (node_position, node)) in nodes.iter().enumerate() {
            if index >= GUNKFISH_RETREAT_NODE_LIMIT {
                break;
            }

            let distance_from_player = distance_squared(*node_position, player_position);
            if distance_from_player < min_distance_squared {
                continue;
            }

            selected = match selected {
                Some((selected_id, selected_position, selected_distance))
                    if selected_distance > distance_from_player
                        || (selected_distance == distance_from_player
                            && selected_id <= node.stable_id) =>
                {
                    Some((selected_id, selected_position, selected_distance))
                }
                _ => Some((node.stable_id, *node_position, distance_from_player)),
            };
        }

        if let Some((_, retreat_position, _)) = selected {
            movement.hiding_spot = retreat_position;
        }
    }
}

fn backwater_gunkfish_stop_fleeing_when_safe(
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<BackwaterGunkfishStateChangedEvent>,
    mut whine_events: EventWriter<BackwaterGunkfishWhineEvent>,
    mut gunkfish: Query<
        (
            Entity,
            &mut BackwaterGunkfishState,
            &mut UnitStats,
            &mut BackwaterGunkfishTimers,
            &mut BackwaterGunkfishMovement,
        ),
        With<BackwaterGunkfish>,
    >,
) {
    let flee_ticks_needed = seconds_to_ticks(GUNKFISH_MAX_FLEE_SECONDS, sim_hz.0);
    let hit_ticks_needed = seconds_to_ticks(GUNKFISH_SAFE_AFTER_HIT_SECONDS, sim_hz.0);

    for (entity, mut state, mut stats, mut timers, mut movement) in gunkfish.iter_mut() {
        if *state != BackwaterGunkfishState::Fleeing {
            continue;
        }

        if timers.flee_ticks >= flee_ticks_needed && timers.ticks_since_hit >= hit_ticks_needed {
            if movement.whining {
                movement.whining = false;
                whine_events.send(BackwaterGunkfishWhineEvent {
                    gunkfish: entity,
                    active: false,
                });
            }

            stats.move_speed = I32F32::lit("0");
            timers.flee_ticks = 0;
            set_state(
                entity,
                &mut state,
                BackwaterGunkfishState::Flopping,
                &mut state_events,
            );
        }
    }
}

fn backwater_gunkfish_flee_reaches_hiding_spot(
    mut state_events: EventWriter<BackwaterGunkfishStateChangedEvent>,
    mut gunkfish: Query<
        (
            Entity,
            &SimPosition,
            &mut BackwaterGunkfishState,
            &mut BackwaterGunkfishTimers,
            &BackwaterGunkfishMovement,
        ),
        With<BackwaterGunkfish>,
    >,
) {
    let hiding_reached_radius_squared = GUNKFISH_HIDING_REACHED_RADIUS * GUNKFISH_HIDING_REACHED_RADIUS;

    for (entity, position, mut state, mut timers, movement) in gunkfish.iter_mut() {
        if *state != BackwaterGunkfishState::Fleeing {
            continue;
        }

        if distance_squared(*position, movement.hiding_spot) <= hiding_reached_radius_squared {
            timers.hide_ticks = 0;
            timers.watch_ticks = 0;
            set_state(
                entity,
                &mut state,
                BackwaterGunkfishState::Cloaked,
                &mut state_events,
            );
        }
    }
}

fn backwater_gunkfish_cloaked_hide_timer(
    mut gunkfish: Query<
        (
            &SimPosition,
            &BackwaterGunkfishState,
            &mut BackwaterGunkfishTimers,
        ),
        With<BackwaterGunkfish>,
    >,
    employees: Query<(&SimPosition, &BackwaterGunkfishEmployeeTarget)>,
) {
    let reset_radius_squared = GUNKFISH_CLOAK_RESET_RADIUS * GUNKFISH_CLOAK_RESET_RADIUS;

    for (position, state, mut timers) in gunkfish.iter_mut() {
        if *state != BackwaterGunkfishState::Cloaked {
            continue;
        }

        let mut employee_nearby = false;

        for (employee_position, employee) in employees.iter() {
            if !employee.inside_facility {
                continue;
            }

            if distance_squared(*position, *employee_position) <= reset_radius_squared {
                employee_nearby = true;
            }
        }

        if employee_nearby {
            timers.hide_ticks = 0;
        } else {
            timers.hide_ticks = timers.hide_ticks.saturating_add(1);
        }
    }
}

fn backwater_gunkfish_cloaked_relocation_check(
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<BackwaterGunkfishStateChangedEvent>,
    mut gunkfish: Query<
        (
            Entity,
            &SimPosition,
            &mut BackwaterGunkfishState,
            &mut BackwaterGunkfishTimers,
            &mut BackwaterGunkfishMovement,
        ),
        With<BackwaterGunkfish>,
    >,
    employees: Query<&SimPosition, With<BackwaterGunkfishEmployeeTarget>>,
) {
    let relocate_ticks = seconds_to_ticks(GUNKFISH_CLOAK_RELOCATE_SECONDS, sim_hz.0);
    let reset_ticks = seconds_to_ticks(GUNKFISH_CLOAK_TIMER_RESET_SECONDS, sim_hz.0);
    let closest_range_squared =
        GUNKFISH_CLOAK_CLOSEST_EMPLOYEE_RANGE * GUNKFISH_CLOAK_CLOSEST_EMPLOYEE_RANGE;

    for (entity, position, mut state, mut timers, mut movement) in gunkfish.iter_mut() {
        if *state != BackwaterGunkfishState::Cloaked || timers.hide_ticks < relocate_ticks {
            continue;
        }

        if closest_position(*position, &employees)
            .map(|employee_position| distance_squared(*position, employee_position) <= closest_range_squared)
            .unwrap_or(false)
        {
            timers.hide_ticks = reset_ticks;
        } else {
            movement.target_employee = None;
            movement.target_employee_stable_id = 0;
            set_state(
                entity,
                &mut state,
                BackwaterGunkfishState::Searching,
                &mut state_events,
            );
        }
    }
}

fn backwater_gunkfish_cloaked_watch_to_lunge(
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<BackwaterGunkfishStateChangedEvent>,
    mut gunkfish: Query<
        (
            Entity,
            &SimPosition,
            &mut BackwaterGunkfishState,
            &mut BackwaterGunkfishTimers,
            &mut BackwaterGunkfishMovement,
        ),
        With<BackwaterGunkfish>,
    >,
    employees: Query<(Entity, &SimPosition, &BackwaterGunkfishEmployeeTarget)>,
) {
    let watch_radius_squared = GUNKFISH_LUNGE_WATCH_RADIUS * GUNKFISH_LUNGE_WATCH_RADIUS;
    let watch_ticks_needed = seconds_to_ticks(GUNKFISH_WATCH_TO_LUNGE_SECONDS, sim_hz.0);

    for (entity, position, mut state, mut timers, mut movement) in gunkfish.iter_mut() {
        if *state != BackwaterGunkfishState::Cloaked {
            continue;
        }

        let mut nearby_count = 0_u32;
        let mut selected: Option<(Entity, u64)> = None;

        for (employee_entity, employee_position, employee) in employees.iter() {
            if distance_squared(*position, *employee_position) <= watch_radius_squared {
                nearby_count = nearby_count.saturating_add(1);
                selected = match selected {
                    Some((selected_entity, selected_stable_id))
                        if selected_stable_id <= employee.stable_id =>
                    {
                        Some((selected_entity, selected_stable_id))
                    }
                    _ => Some((employee_entity, employee.stable_id)),
                };
            }
        }

        if nearby_count == 0 {
            timers.watch_ticks = 0;
            continue;
        }

        timers.watch_ticks = timers.watch_ticks.saturating_add(nearby_count);

        if timers.watch_ticks > watch_ticks_needed {
            if let Some((target, stable_id)) = selected {
                movement.target_employee = Some(target);
                movement.target_employee_stable_id = stable_id;
            }

            set_state(
                entity,
                &mut state,
                BackwaterGunkfishState::Lunge,
                &mut state_events,
            );
        }
    }
}

fn backwater_gunkfish_cloaked_collision_to_lunge(
    mut state_events: EventWriter<BackwaterGunkfishStateChangedEvent>,
    mut gunkfish: Query<
        (
            Entity,
            &SimPosition,
            &mut BackwaterGunkfishState,
            &mut BackwaterGunkfishMovement,
        ),
        With<BackwaterGunkfish>,
    >,
    employees: Query<(Entity, &SimPosition, &BackwaterGunkfishEmployeeTarget)>,
) {
    let collision_radius_squared = GUNKFISH_COLLISION_LUNGE_RADIUS * GUNKFISH_COLLISION_LUNGE_RADIUS;

    for (entity, position, mut state, mut movement) in gunkfish.iter_mut() {
        if *state != BackwaterGunkfishState::Cloaked {
            continue;
        }

        let mut selected: Option<(Entity, u64)> = None;

        for (employee_entity, employee_position, employee) in employees.iter() {
            if distance_squared(*position, *employee_position) <= collision_radius_squared {
                selected = match selected {
                    Some((selected_entity, selected_stable_id))
                        if selected_stable_id <= employee.stable_id =>
                    {
                        Some((selected_entity, selected_stable_id))
                    }
                    _ => Some((employee_entity, employee.stable_id)),
                };
            }
        }

        if let Some((target, stable_id)) = selected {
            movement.target_employee = Some(target);
            movement.target_employee_stable_id = stable_id;
            set_state(
                entity,
                &mut state,
                BackwaterGunkfishState::Lunge,
                &mut state_events,
            );
        }
    }
}

fn backwater_gunkfish_hit_while_cloaked(
    mut damage_events: EventReader<IncomingDamageEvent>,
    mut state_events: EventWriter<BackwaterGunkfishStateChangedEvent>,
    mut gunkfish: Query<
        (
            Entity,
            &mut BackwaterGunkfishState,
            &mut BackwaterGunkfishTimers,
        ),
        With<BackwaterGunkfish>,
    >,
) {
    for event in damage_events.read() {
        for (entity, mut state, mut timers) in gunkfish.iter_mut() {
            if event.target != entity || *state != BackwaterGunkfishState::Cloaked {
                continue;
            }

            timers.ticks_since_hit = 0;
            timers.flee_ticks = 0;
            set_state(
                entity,
                &mut state,
                BackwaterGunkfishState::Fleeing,
                &mut state_events,
            );
        }
    }
}

fn backwater_gunkfish_apply_damage(
    mut damage_events: EventReader<IncomingDamageEvent>,
    mut killed_events: EventWriter<EntityKilledEvent>,
    mut gunkfish: Query<(Entity, &mut Health), With<BackwaterGunkfish>>,
) {
    for event in damage_events.read() {
        for (entity, mut health) in gunkfish.iter_mut() {
            if event.target != entity {
                continue;
            }

            health.current -= event.raw_amount;

            if health.current <= I32F32::lit("0") {
                health.current = I32F32::lit("0");
                killed_events.send(EntityKilledEvent {
                    entity,
                    killer: event.source,
                    exp_reward: I32F32::lit("0"),
                    difficulty_tier: 0,
                });
            }
        }
    }
}

fn backwater_gunkfish_report_zap_gun_difficulty(
    mut targeted_events: EventReader<BackwaterGunkfishZapGunDifficultyEvent>,
    gunkfish: Query<(), With<BackwaterGunkfish>>,
) {
    for event in targeted_events.read() {
        let _ = gunkfish.get(event.gunkfish);
    }
}

fn backwater_gunkfish_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    gunkfish: Query<
        (
            &SimPosition,
            &Health,
            &UnitStats,
            &BackwaterGunkfishFacing,
            &BackwaterGunkfishState,
            &BackwaterGunkfishTimers,
            &BackwaterGunkfishMovement,
        ),
        With<BackwaterGunkfish>,
    >,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(BACKWATER_GUNKFISH_SOURCE_REVISION as u64);
    checksum.accumulate(BACKWATER_GUNKFISH_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(BACKWATER_GUNKFISH_HP.to_bits() as u64);
    checksum.accumulate(BACKWATER_GUNKFISH_POWER_LEVEL.to_bits() as u64);
    checksum.accumulate(BACKWATER_GUNKFISH_MAX_SPAWNED as u64);
    checksum.accumulate(BACKWATER_GUNKFISH_ATTACK_DAMAGE.to_bits() as u64);
    checksum.accumulate(BACKWATER_GUNKFISH_STUN_MULTIPLIER.to_bits() as u64);
    checksum.accumulate(BACKWATER_GUNKFISH_CAN_SEE_THROUGH_FOG as u64);
    checksum.accumulate(BACKWATER_GUNKFISH_SPAWN_DELAY_SECONDS as u64);
    checksum.accumulate(BACKWATER_GUNKFISH_DOOR_SPEED_MULTIPLIER.to_bits() as u64);
    checksum.accumulate(BACKWATER_GUNKFISH_ZAP_GUN_DIFFICULTY.to_bits() as u64);
    checksum.accumulate(GUNKFISH_SPOT_FOV_DEGREES as u64);

    accumulate_str(&mut checksum, 0x1000, BACKWATER_GUNKFISH_ID);
    accumulate_str(&mut checksum, 0x1001, BACKWATER_GUNKFISH_NAME);
    accumulate_str(&mut checksum, 0x1002, BACKWATER_GUNKFISH_TYPE);
    accumulate_str(&mut checksum, 0x1003, BACKWATER_GUNKFISH_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, BACKWATER_GUNKFISH_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1005, BACKWATER_GUNKFISH_EXTRACTED_AT);
    accumulate_str(&mut checksum, 0x1006, BACKWATER_GUNKFISH_DWELLS);
    accumulate_str(&mut checksum, 0x1007, BACKWATER_GUNKFISH_DANGER);
    accumulate_str(&mut checksum, 0x1008, BACKWATER_GUNKFISH_SCIENTIFIC_NAME);
    accumulate_str(&mut checksum, 0x1009, BACKWATER_GUNKFISH_INTERNAL_NAME);
    accumulate_str(&mut checksum, 0x100a, BACKWATER_GUNKFISH_PIP_SIZE);

    for dependency in BACKWATER_GUNKFISH_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for behavior in BACKWATER_GUNKFISH_FRONTMATTER_BEHAVIOR {
        accumulate_str(&mut checksum, 0x3000, behavior);
    }

    for rule in BACKWATER_GUNKFISH_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x4000, rule.condition);
        accumulate_str(&mut checksum, 0x4001, rule.outcome);
    }

    for (position, health, stats, facing, state, timers, movement) in gunkfish.iter() {
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);
        checksum.accumulate(facing.x.to_bits() as u64);
        checksum.accumulate(facing.y.to_bits() as u64);
        checksum.accumulate(*state as u64);
        checksum.accumulate(timers.spawn_delay_ticks as u64);
        checksum.accumulate(timers.spotting_ticks as u64);
        checksum.accumulate(timers.hide_ticks as u64);
        checksum.accumulate(timers.watch_ticks as u64);
        checksum.accumulate(timers.flee_ticks as u64);
        checksum.accumulate(timers.ticks_since_hit as u64);
        checksum.accumulate(movement.hiding_spot.x.to_bits() as u64);
        checksum.accumulate(movement.hiding_spot.y.to_bits() as u64);
        checksum.accumulate(movement.fallback_position.x.to_bits() as u64);
        checksum.accumulate(movement.fallback_position.y.to_bits() as u64);
        checksum.accumulate(movement.last_slime_position.x.to_bits() as u64);
        checksum.accumulate(movement.last_slime_position.y.to_bits() as u64);
        checksum.accumulate(movement.target_employee_stable_id);
        checksum.accumulate(movement.whining as u64);
    }
}

fn visible_employee(
    gunkfish_entity: Entity,
    position: SimPosition,
    facing: BackwaterGunkfishFacing,
    employees: &Query<(Entity, &SimPosition, &BackwaterGunkfishEmployeeTarget)>,
) -> Option<(Entity, u64)> {
    let spot_range_squared = GUNKFISH_SPOT_RANGE * GUNKFISH_SPOT_RANGE;
    let ignore_radius_squared =
        GUNKFISH_SEARCH_IGNORE_EMPLOYEE_RADIUS * GUNKFISH_SEARCH_IGNORE_EMPLOYEE_RADIUS;
    let mut selected: Option<(Entity, u64)> = None;

    for (employee_entity, employee_position, employee) in employees.iter() {
        let distance = distance_squared(position, *employee_position);

        if distance <= ignore_radius_squared || distance > spot_range_squared {
            continue;
        }

        let dx = employee_position.x - position.x;
        let dy = employee_position.y - position.y;
        let dot = dx * facing.x + dy * facing.y;
        if dot <= I32F32::lit("0") {
            continue;
        }

        if dot * dot < distance * GUNKFISH_FIXED_FOV_DOT_SQUARED_MIN {
            continue;
        }

        selected = match selected {
            Some((selected_entity, selected_stable_id)) if selected_stable_id <= employee.stable_id => {
                Some((selected_entity, selected_stable_id))
            }
            _ => Some((employee_entity, employee.stable_id)),
        };
    }

    let _ = gunkfish_entity;
    selected
}

fn hiding_spot_near_employee_far_enough(
    employees: &Query<(&SimPosition, &BackwaterGunkfishEmployeeTarget)>,
    nodes: &Query<(&SimPosition, &BackwaterGunkfishInteriorNode)>,
) -> Option<SimPosition> {
    let avoid_distance_squared = GUNKFISH_EMPLOYEE_AVOID_DISTANCE * GUNKFISH_EMPLOYEE_AVOID_DISTANCE;
    let mut selected: Option<(u64, SimPosition, I32F32)> = None;

    for (employee_position, _) in employees.iter() {
        for (node_position, node) in nodes.iter() {
            let distance = distance_squared(*node_position, *employee_position);

            if distance <= avoid_distance_squared {
                continue;
            }

            selected = match selected {
                Some((selected_id, selected_position, selected_distance))
                    if selected_distance < distance
                        || (selected_distance == distance && selected_id <= node.stable_id) =>
                {
                    Some((selected_id, selected_position, selected_distance))
                }
                _ => Some((node.stable_id, *node_position, distance)),
            };
        }
    }

    selected.map(|(_, position, _)| position)
}

fn hiding_spot_near_scrap(
    scrap: &Query<&SimPosition, With<BackwaterGunkfishScrapAnchor>>,
    nodes: &Query<(&SimPosition, &BackwaterGunkfishInteriorNode)>,
) -> Option<SimPosition> {
    let mut selected: Option<(u64, SimPosition, I32F32)> = None;

    for scrap_position in scrap.iter() {
        for (node_position, node) in nodes.iter() {
            let distance = distance_squared(*node_position, *scrap_position);

            selected = match selected {
                Some((selected_id, selected_position, selected_distance))
                    if selected_distance < distance
                        || (selected_distance == distance && selected_id <= node.stable_id) =>
                {
                    Some((selected_id, selected_position, selected_distance))
                }
                _ => Some((node.stable_id, *node_position, distance)),
            };
        }
    }

    selected.map(|(_, position, _)| position)
}

fn fallback_far_from_entrance(
    current_position: &SimPosition,
    nodes: &Query<(&SimPosition, &BackwaterGunkfishInteriorNode)>,
    entrances: &Query<&SimPosition, With<BackwaterGunkfishMainEntrance>>,
) -> Option<SimPosition> {
    let entrance_position = entrances.iter().next().copied().unwrap_or(*current_position);
    let mut selected: Option<(u64, SimPosition, I32F32)> = None;

    for (node_position, node) in nodes.iter() {
        let distance = distance_squared(*node_position, entrance_position);

        selected = match selected {
            Some((selected_id, selected_position, selected_distance))
                if selected_distance > distance
                    || (selected_distance == distance && selected_id <= node.stable_id) =>
            {
                Some((selected_id, selected_position, selected_distance))
            }
            _ => Some((node.stable_id, *node_position, distance)),
        };
    }

    selected.map(|(_, position, _)| position)
}

fn closest_position(
    origin: SimPosition,
    positions: &Query<&SimPosition, With<BackwaterGunkfishEmployeeTarget>>,
) -> Option<SimPosition> {
    let mut selected: Option<(SimPosition, I32F32)> = None;

    for position in positions.iter() {
        let distance = distance_squared(origin, *position);

        selected = match selected {
            Some((selected_position, selected_distance)) if selected_distance <= distance => {
                Some((selected_position, selected_distance))
            }
            _ => Some((*position, distance)),
        };
    }

    selected.map(|(position, _)| position)
}

fn set_state(
    entity: Entity,
    state: &mut BackwaterGunkfishState,
    to: BackwaterGunkfishState,
    events: &mut EventWriter<BackwaterGunkfishStateChangedEvent>,
) {
    if *state == to {
        return;
    }

    let from = *state;
    *state = to;
    events.send(BackwaterGunkfishStateChangedEvent {
        gunkfish: entity,
        from,
        to,
    });
}

fn distance_squared(a: SimPosition, b: SimPosition) -> I32F32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    dx * dx + dy * dy
}

fn seconds_to_ticks(seconds: I32F32, sim_hz: I32F32) -> u32 {
    let ticks = (seconds * sim_hz).ceil().to_num::<u32>();

    if ticks == 0 {
        1
    } else {
        ticks
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}

fn _damage_type_is_deterministic(value: DamageType) -> u64 {
    match value {
        DamageType::Standard => 1,
        DamageType::Fire => 2,
        DamageType::Venom => 3,
    }
}
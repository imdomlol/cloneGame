// Sources: vault/outdoor_entity_pages/forest_keeper.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{
    DamageType, Health, IncomingDamageEvent, SimChecksumState, SimHz, SimPosition, SimTick,
    UnitStats,
};

pub const FOREST_KEEPER_ID: &str = "forest_keeper";
pub const FOREST_KEEPER_NAME: &str = "Forest Keeper";
pub const FOREST_KEEPER_TYPE: &str = "outdoor_entity_pages";
pub const FOREST_KEEPER_SUBTYPE: &str = "giant";
pub const FOREST_KEEPER_SOURCE_URL: &str =
    "https://lethal-company.fandom.com/wiki/Forest_Keeper";
pub const FOREST_KEEPER_SOURCE_REVISION: u32 = 21307;
pub const FOREST_KEEPER_EXTRACTED_AT: &str = "2026-06-07";
pub const FOREST_KEEPER_CONFIDENCE_BASIS_POINTS: u16 = 97;

pub const FOREST_KEEPER_DWELLS: &str = "Outdoors";
pub const FOREST_KEEPER_DANGER: &str = "50%";
pub const FOREST_KEEPER_SCIENTIFIC_NAME: &str = "Satyrid-proceritas";
pub const FOREST_KEEPER_HP: I32F32 = I32F32::lit("38");
pub const FOREST_KEEPER_POWER_LEVEL: I32F32 = I32F32::lit("3");
pub const FOREST_KEEPER_MAX_SPAWNED: usize = 3;
pub const FOREST_KEEPER_CONTACT_DAMAGE: I32F32 = I32F32::lit("9999");
pub const FOREST_KEEPER_STUN_MULTIPLIER: I32F32 = I32F32::lit("1.2");
pub const FOREST_KEEPER_ZAP_GUN_DIFFICULTY: I32F32 = I32F32::lit("1.4");
pub const FOREST_KEEPER_INTERNAL_NAME: &str = "ForestGiant";
pub const FOREST_KEEPER_PIP_SIZE: &str = "Large";
pub const FOREST_KEEPER_DOOR_SPEED_MULTIPLIER: I32F32 = I32F32::lit("1");
pub const FOREST_KEEPER_CAN_SEE_THROUGH_FOG: bool = false;
pub const FOREST_KEEPER_SPAWN_DELAY_SECONDS: I32F32 = I32F32::lit("15");

pub const FOREST_KEEPER_RISE_SECONDS: I32F32 = I32F32::lit("9");
pub const FOREST_KEEPER_FIELD_OF_VIEW_DEGREES: I32F32 = I32F32::lit("50");
pub const FOREST_KEEPER_VIEW_DISTANCE: I32F32 = I32F32::lit("70");
pub const FOREST_KEEPER_FOG_VIEW_DISTANCE: I32F32 = I32F32::lit("30");
pub const FOREST_KEEPER_ROAM_SEARCH_PRECISION: I32F32 = I32F32::lit("8");
pub const FOREST_KEEPER_ROAM_SEARCH_WIDTH: I32F32 = I32F32::lit("200");
pub const FOREST_KEEPER_SEARCH_SECONDS: I32F32 = I32F32::lit("9");
pub const FOREST_KEEPER_LOST_LOS_SECONDS: I32F32 = I32F32::lit("3");
pub const FOREST_KEEPER_SEARCH_PRECISION: I32F32 = I32F32::lit("15");
pub const FOREST_KEEPER_SEARCH_WIDTH: I32F32 = I32F32::lit("25");
pub const FOREST_KEEPER_CROUCHING_SUSPICION_MULTIPLIER: I32F32 = I32F32::lit("1");
pub const FOREST_KEEPER_STANDING_SUSPICION_MULTIPLIER: I32F32 = I32F32::lit("2");
pub const FOREST_KEEPER_MOVING_SUSPICION_BONUS: I32F32 = I32F32::lit("1");
pub const FOREST_KEEPER_SUSPICION_GAIN_PER_SECOND: I32F32 = I32F32::lit("0.25");
pub const FOREST_KEEPER_SUSPICION_DROP_PER_SECOND: I32F32 = I32F32::lit("0.35");
pub const FOREST_KEEPER_ROAM_SPEED: I32F32 = I32F32::lit("4");
pub const FOREST_KEEPER_CHASE_SPEED: I32F32 = I32F32::lit("9");
pub const FOREST_KEEPER_ATTACK_RANGE: I32F32 = I32F32::lit("1");
pub const FOREST_KEEPER_ATTACK_SPEED_SECONDS: I32F32 = I32F32::lit("1");
pub const FOREST_KEEPER_CRUSH_RANGE: I32F32 = I32F32::lit("4");
pub const FOREST_KEEPER_BURN_SECONDS: I32F32 = I32F32::lit("3");
pub const FOREST_KEEPER_GRAVITY_CAUSE_ID: &str = "gravity";

pub const FOREST_KEEPER_DEPENDS_ON: [&str; 0] = [];
pub const FOREST_KEEPER_FRONTMATTER_BEHAVIOR: [&str; 2] = ["Roam", "Chase"];

pub const FOREST_KEEPER_BEHAVIORAL_MECHANICS: [ForestKeeperBehaviorRule; 20] = [
    ForestKeeperBehaviorRule {
        condition: "a Forest Keeper is spawning",
        outcome: "it rises from the ground for 9 seconds before it can make progress, and it cannot collide with or kill employees during that rise",
    },
    ForestKeeperBehaviorRule {
        condition: "the rising animation is still active",
        outcome: "the Forest Keeper can still take damage",
    },
    ForestKeeperBehaviorRule {
        condition: "an employee makes contact with a fully risen Forest Keeper",
        outcome: "the employee is grabbed and eaten instantly, and the body is not recoverable",
    },
    ForestKeeperBehaviorRule {
        condition: "an employee is eaten",
        outcome: "all items in that employee's inventory are deleted",
    },
    ForestKeeperBehaviorRule {
        condition: "a Forest Keeper is roaming",
        outcome: "it searches for employees with a 50 degree field of view, a 70 meter viewing distance, 30 meters in fog, an 8 meter search precision, and a 200 meter search width",
    },
    ForestKeeperBehaviorRule {
        condition: "a Forest Keeper sees an employee",
        outcome: "that employee's suspicion meter rises toward 1.0 until the Forest Keeper begins chasing them",
    },
    ForestKeeperBehaviorRule {
        condition: "the employee is crouching",
        outcome: "the suspicion multiplier is 1.0x",
    },
    ForestKeeperBehaviorRule {
        condition: "the employee is standing",
        outcome: "the suspicion multiplier is 2.0x",
    },
    ForestKeeperBehaviorRule {
        condition: "the employee is moving",
        outcome: "the total suspicion multiplier gains +1.0x",
    },
    ForestKeeperBehaviorRule {
        condition: "a Forest Keeper has line of sight on one employee",
        outcome: "every other employee's suspicion meter drops instead of rising",
    },
    ForestKeeperBehaviorRule {
        condition: "a suspicion meter reaches 1.0",
        outcome: "the Forest Keeper switches to chasing that employee",
    },
    ForestKeeperBehaviorRule {
        condition: "chasing",
        outcome: "the Forest Keeper sprints toward the target, continues raising suspicion, and can grab on contact to eat instantly",
    },
    ForestKeeperBehaviorRule {
        condition: "the target breaks line of sight for 3 seconds",
        outcome: "the Forest Keeper enters a search state for about 9 seconds",
    },
    ForestKeeperBehaviorRule {
        condition: "searching",
        outcome: "the Forest Keeper uses a 15 meter search precision and a 25 meter search width around the last known location",
    },
    ForestKeeperBehaviorRule {
        condition: "the search timer ends or the search area is exhausted",
        outcome: "the Forest Keeper returns to roaming",
    },
    ForestKeeperBehaviorRule {
        condition: "the Forest Keeper is stunned while attempting to eat an employee",
        outcome: "it drops that employee",
    },
    ForestKeeperBehaviorRule {
        condition: "the Forest Keeper is stunned",
        outcome: "stun effects are multiplied by 1.2x",
    },
    ForestKeeperBehaviorRule {
        condition: "the Forest Keeper is killed",
        outcome: "it falls backward and can kill nearby employees by crushing them",
    },
    ForestKeeperBehaviorRule {
        condition: "an employee is crushed by the death animation",
        outcome: "the death is attributed to Gravity and the body remains recoverable",
    },
    ForestKeeperBehaviorRule {
        condition: "a zap gun is used against the Forest Keeper",
        outcome: "the zap gun difficulty is 1.4",
    },
];

pub struct ForestKeeperPlugin;

impl Plugin for ForestKeeperPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnForestKeeperEvent>()
            .add_event::<ForestKeeperStateChangedEvent>()
            .add_event::<ForestKeeperSuspicionChangedEvent>()
            .add_event::<ForestKeeperEmployeeEatenEvent>()
            .add_event::<ForestKeeperInventoryDeletedEvent>()
            .add_event::<ForestKeeperStunAppliedEvent>()
            .add_event::<ForestKeeperStunAdjustedEvent>()
            .add_event::<ForestKeeperDroppedEmployeeEvent>()
            .add_event::<ForestKeeperCrushKillEvent>()
            .add_event::<ForestKeeperFireStartedEvent>()
            .add_event::<ForestKeeperBurnedDownEvent>()
            .add_event::<ForestKeeperZapGunDifficultyEvent>()
            .add_event::<ForestKeeperDamageTakenEvent>()
            .add_event::<ForestKeeperDefeatedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    forest_keeper_spawn_from_events,
                    forest_keeper_rise_from_ground,
                    forest_keeper_roam_suspicion,
                    forest_keeper_chase_target,
                    forest_keeper_search_after_lost_target,
                    forest_keeper_contact_eat,
                    forest_keeper_apply_stun,
                    forest_keeper_apply_fire,
                    forest_keeper_take_damage,
                    forest_keeper_crush_on_death,
                    forest_keeper_report_zap_gun_difficulty,
                    forest_keeper_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ForestKeeperBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ForestKeeper {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ForestKeeperEmployeeSensor {
    pub stable_id: u64,
    pub is_alive: bool,
    pub visible_to_keeper: bool,
    pub in_field_of_view: bool,
    pub in_fog: bool,
    pub crouching: bool,
    pub moving: bool,
    pub inventory_items: u32,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ForestKeeperTarget {
    pub has_target: bool,
    pub target_entity: Entity,
    pub target_stable_id: u64,
    pub last_known_position: SimPosition,
    pub target_visible: bool,
    pub lost_line_of_sight_ticks: u32,
}

impl Default for ForestKeeperTarget {
    fn default() -> Self {
        Self {
            has_target: false,
            target_entity: Entity::PLACEHOLDER,
            target_stable_id: 0,
            last_known_position: SimPosition {
                x: I32F32::ZERO,
                y: I32F32::ZERO,
            },
            target_visible: false,
            lost_line_of_sight_ticks: 0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ForestKeeperMovement {
    pub rise_timer_ticks: u32,
    pub search_timer_ticks: u32,
    pub stun_timer_ticks: u32,
    pub burn_timer_ticks: u32,
    pub current_speed: I32F32,
    pub search_precision: I32F32,
    pub search_width: I32F32,
}

impl Default for ForestKeeperMovement {
    fn default() -> Self {
        Self {
            rise_timer_ticks: 0,
            search_timer_ticks: 0,
            stun_timer_ticks: 0,
            burn_timer_ticks: 0,
            current_speed: FOREST_KEEPER_ROAM_SPEED,
            search_precision: FOREST_KEEPER_ROAM_SEARCH_PRECISION,
            search_width: FOREST_KEEPER_ROAM_SEARCH_WIDTH,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ForestKeeperSuspicion {
    pub employee_stable_id: u64,
    pub meter: I32F32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ForestKeeperState {
    #[default]
    Spawning,
    Roam,
    Chase,
    Search,
    Eating,
    Stunned,
    Burning,
    Dead,
}

#[derive(Bundle)]
pub struct ForestKeeperBundle {
    pub name: Name,
    pub keeper: ForestKeeper,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub state: ForestKeeperState,
    pub target: ForestKeeperTarget,
    pub movement: ForestKeeperMovement,
}

impl ForestKeeperBundle {
    pub fn new(event: SpawnForestKeeperEvent, sim_hz: I32F32) -> Self {
        Self {
            name: Name::new(FOREST_KEEPER_NAME),
            keeper: ForestKeeper {
                stable_id: event.stable_id,
            },
            position: event.position,
            health: Health::full(FOREST_KEEPER_HP),
            stats: UnitStats {
                move_speed: I32F32::ZERO,
                attack_range: FOREST_KEEPER_ATTACK_RANGE,
                attack_damage: FOREST_KEEPER_CONTACT_DAMAGE,
                attack_speed: FOREST_KEEPER_ATTACK_SPEED_SECONDS,
                watch_range: FOREST_KEEPER_VIEW_DISTANCE,
            },
            state: ForestKeeperState::Spawning,
            target: ForestKeeperTarget {
                last_known_position: event.position,
                ..Default::default()
            },
            movement: ForestKeeperMovement {
                rise_timer_ticks: fixed_seconds_to_ticks(FOREST_KEEPER_RISE_SECONDS, sim_hz),
                current_speed: I32F32::ZERO,
                ..Default::default()
            },
        }
    }
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpawnForestKeeperEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ForestKeeperStateChangedEvent {
    pub keeper: Entity,
    pub from: ForestKeeperState,
    pub to: ForestKeeperState,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ForestKeeperSuspicionChangedEvent {
    pub keeper: Entity,
    pub employee: Entity,
    pub employee_stable_id: u64,
    pub meter: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ForestKeeperEmployeeEatenEvent {
    pub keeper: Entity,
    pub employee: Entity,
    pub employee_stable_id: u64,
    pub body_recoverable: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ForestKeeperInventoryDeletedEvent {
    pub employee: Entity,
    pub employee_stable_id: u64,
    pub deleted_items: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ForestKeeperStunAppliedEvent {
    pub keeper: Entity,
    pub source: Entity,
    pub base_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ForestKeeperStunAdjustedEvent {
    pub keeper: Entity,
    pub source: Entity,
    pub adjusted_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ForestKeeperDroppedEmployeeEvent {
    pub keeper: Entity,
    pub employee: Entity,
    pub employee_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ForestKeeperCrushKillEvent {
    pub keeper: Entity,
    pub employee: Entity,
    pub employee_stable_id: u64,
    pub cause_id: &'static str,
    pub body_recoverable: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ForestKeeperFireStartedEvent {
    pub keeper: Entity,
    pub source: Entity,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ForestKeeperBurnedDownEvent {
    pub keeper: Entity,
    pub source: Entity,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ForestKeeperZapGunDifficultyEvent {
    pub keeper: Entity,
    pub difficulty: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ForestKeeperDamageTakenEvent {
    pub keeper: Entity,
    pub source: Entity,
    pub damage: I32F32,
    pub remaining_health: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ForestKeeperDefeatedEvent {
    pub keeper: Entity,
    pub source: Entity,
}

fn forest_keeper_spawn_from_events(
    mut commands: Commands,
    sim_hz: Res<SimHz>,
    mut events: EventReader<SpawnForestKeeperEvent>,
    keepers: Query<(), With<ForestKeeper>>,
) {
    let mut spawned_count = keepers.iter().count();

    for event in events.read() {
        if spawned_count >= FOREST_KEEPER_MAX_SPAWNED {
            break;
        }

        commands.spawn(ForestKeeperBundle::new(*event, sim_hz.0));
        spawned_count += 1;
    }
}

fn forest_keeper_rise_from_ground(
    mut state_events: EventWriter<ForestKeeperStateChangedEvent>,
    mut keepers: Query<
        (
            Entity,
            &mut ForestKeeperState,
            &mut ForestKeeperMovement,
            &mut UnitStats,
        ),
        With<ForestKeeper>,
    >,
) {
    for (entity, mut state, mut movement, mut stats) in keepers.iter_mut() {
        if *state != ForestKeeperState::Spawning {
            continue;
        }

        if movement.rise_timer_ticks > 0 {
            movement.rise_timer_ticks -= 1;
            stats.move_speed = I32F32::ZERO;
            continue;
        }

        stats.move_speed = FOREST_KEEPER_ROAM_SPEED;
        movement.current_speed = FOREST_KEEPER_ROAM_SPEED;
        set_forest_keeper_state(entity, &mut state, ForestKeeperState::Roam, &mut state_events);
    }
}

fn forest_keeper_roam_suspicion(
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<ForestKeeperStateChangedEvent>,
    mut suspicion_events: EventWriter<ForestKeeperSuspicionChangedEvent>,
    mut keepers: Query<
        (
            Entity,
            &SimPosition,
            &mut ForestKeeperState,
            &mut ForestKeeperTarget,
            &mut UnitStats,
        ),
        With<ForestKeeper>,
    >,
    mut employees: Query<
        (
            Entity,
            &SimPosition,
            &ForestKeeperEmployeeSensor,
            &mut ForestKeeperSuspicion,
        ),
        Without<ForestKeeper>,
    >,
) {
    for (keeper_entity, keeper_position, mut state, mut target, mut stats) in keepers.iter_mut() {
        if *state != ForestKeeperState::Roam {
            continue;
        }

        let visible_stable_id = first_visible_employee(*keeper_position, &employees);
        let mut chase_target: Option<(Entity, u64, SimPosition)> = None;

        for (employee_entity, employee_position, sensor, mut suspicion) in employees.iter_mut() {
            if !sensor.is_alive {
                continue;
            }

            suspicion.employee_stable_id = sensor.stable_id;
            let can_see = Some(sensor.stable_id) == visible_stable_id;

            if can_see {
                let posture_multiplier = if sensor.crouching {
                    FOREST_KEEPER_CROUCHING_SUSPICION_MULTIPLIER
                } else {
                    FOREST_KEEPER_STANDING_SUSPICION_MULTIPLIER
                };
                let moving_bonus = if sensor.moving {
                    FOREST_KEEPER_MOVING_SUSPICION_BONUS
                } else {
                    I32F32::ZERO
                };
                let gain = (FOREST_KEEPER_SUSPICION_GAIN_PER_SECOND
                    * (posture_multiplier + moving_bonus))
                    / sim_hz.0;
                suspicion.meter = fixed_clamp_unit(suspicion.meter + gain);
            } else if visible_stable_id.is_some() {
                suspicion.meter = fixed_clamp_unit(
                    suspicion.meter - (FOREST_KEEPER_SUSPICION_DROP_PER_SECOND / sim_hz.0),
                );
            }

            suspicion_events.send(ForestKeeperSuspicionChangedEvent {
                keeper: keeper_entity,
                employee: employee_entity,
                employee_stable_id: sensor.stable_id,
                meter: suspicion.meter,
            });

            if suspicion.meter >= I32F32::ONE {
                chase_target = Some((employee_entity, sensor.stable_id, *employee_position));
            }
        }

        if let Some((employee_entity, stable_id, position)) = chase_target {
            target.has_target = true;
            target.target_entity = employee_entity;
            target.target_stable_id = stable_id;
            target.last_known_position = position;
            target.target_visible = true;
            target.lost_line_of_sight_ticks = 0;
            stats.move_speed = FOREST_KEEPER_CHASE_SPEED;
            set_forest_keeper_state(
                keeper_entity,
                &mut state,
                ForestKeeperState::Chase,
                &mut state_events,
            );
        }
    }
}

fn forest_keeper_chase_target(
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<ForestKeeperStateChangedEvent>,
    mut keepers: Query<
        (
            Entity,
            &mut SimPosition,
            &mut ForestKeeperState,
            &mut ForestKeeperTarget,
            &mut ForestKeeperMovement,
            &mut UnitStats,
        ),
        With<ForestKeeper>,
    >,
    employees: Query<(Entity, &SimPosition, &ForestKeeperEmployeeSensor), Without<ForestKeeper>>,
) {
    for (keeper_entity, mut position, mut state, mut target, mut movement, mut stats) in
        keepers.iter_mut()
    {
        if *state != ForestKeeperState::Chase || !target.has_target {
            continue;
        }

        let Some((target_position, sensor)) =
            employee_position_by_stable_id(target.target_stable_id, &employees)
        else {
            start_forest_keeper_search(
                keeper_entity,
                &mut state,
                &mut target,
                &mut movement,
                &mut stats,
                sim_hz.0,
                &mut state_events,
            );
            continue;
        };

        let view_distance = if sensor.in_fog {
            FOREST_KEEPER_FOG_VIEW_DISTANCE
        } else {
            FOREST_KEEPER_VIEW_DISTANCE
        };

        target.target_visible = sensor.is_alive
            && sensor.visible_to_keeper
            && sensor.in_field_of_view
            && fixed_distance_sq(*position, target_position) <= fixed_square(view_distance);

        if target.target_visible {
            target.last_known_position = target_position;
            target.lost_line_of_sight_ticks = 0;
        } else {
            target.lost_line_of_sight_ticks += 1;
        }

        if target.lost_line_of_sight_ticks
            >= fixed_seconds_to_ticks(FOREST_KEEPER_LOST_LOS_SECONDS, sim_hz.0)
        {
            start_forest_keeper_search(
                keeper_entity,
                &mut state,
                &mut target,
                &mut movement,
                &mut stats,
                sim_hz.0,
                &mut state_events,
            );
            continue;
        }

        stats.move_speed = FOREST_KEEPER_CHASE_SPEED;
        movement.current_speed = FOREST_KEEPER_CHASE_SPEED;
        move_axis_toward(&mut position, target_position, FOREST_KEEPER_CHASE_SPEED / sim_hz.0);
    }
}

fn forest_keeper_search_after_lost_target(
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<ForestKeeperStateChangedEvent>,
    mut keepers: Query<
        (
            Entity,
            &mut SimPosition,
            &mut ForestKeeperState,
            &mut ForestKeeperTarget,
            &mut ForestKeeperMovement,
            &mut UnitStats,
        ),
        With<ForestKeeper>,
    >,
) {
    for (keeper_entity, mut position, mut state, mut target, mut movement, mut stats) in
        keepers.iter_mut()
    {
        if *state != ForestKeeperState::Search {
            continue;
        }

        movement.search_precision = FOREST_KEEPER_SEARCH_PRECISION;
        movement.search_width = FOREST_KEEPER_SEARCH_WIDTH;
        stats.move_speed = FOREST_KEEPER_ROAM_SPEED;
        move_axis_toward(&mut position, target.last_known_position, FOREST_KEEPER_ROAM_SPEED / sim_hz.0);

        if movement.search_timer_ticks > 0 {
            movement.search_timer_ticks -= 1;
            continue;
        }

        target.has_target = false;
        target.target_entity = Entity::PLACEHOLDER;
        target.target_stable_id = 0;
        target.target_visible = false;
        target.lost_line_of_sight_ticks = 0;
        movement.search_precision = FOREST_KEEPER_ROAM_SEARCH_PRECISION;
        movement.search_width = FOREST_KEEPER_ROAM_SEARCH_WIDTH;
        set_forest_keeper_state(
            keeper_entity,
            &mut state,
            ForestKeeperState::Roam,
            &mut state_events,
        );
    }
}

fn forest_keeper_contact_eat(
    mut damage_events: EventWriter<IncomingDamageEvent>,
    mut eaten_events: EventWriter<ForestKeeperEmployeeEatenEvent>,
    mut deleted_events: EventWriter<ForestKeeperInventoryDeletedEvent>,
    keepers: Query<
        (Entity, &SimPosition, &ForestKeeperState, &ForestKeeperTarget),
        With<ForestKeeper>,
    >,
    employees: Query<(Entity, &SimPosition, &ForestKeeperEmployeeSensor), Without<ForestKeeper>>,
) {
    for (keeper_entity, keeper_position, state, target) in keepers.iter() {
        if *state == ForestKeeperState::Spawning
            || *state == ForestKeeperState::Stunned
            || *state == ForestKeeperState::Dead
        {
            continue;
        }

        for (employee_entity, employee_position, sensor) in employees.iter() {
            if !sensor.is_alive {
                continue;
            }

            let targeted_or_roaming =
                !target.has_target || target.target_stable_id == sensor.stable_id;
            if !targeted_or_roaming {
                continue;
            }

            if fixed_distance_sq(*keeper_position, *employee_position)
                > fixed_square(FOREST_KEEPER_ATTACK_RANGE)
            {
                continue;
            }

            damage_events.send(IncomingDamageEvent {
                target: employee_entity,
                raw_amount: FOREST_KEEPER_CONTACT_DAMAGE,
                damage_type: DamageType::Standard,
                source: keeper_entity,
            });

            eaten_events.send(ForestKeeperEmployeeEatenEvent {
                keeper: keeper_entity,
                employee: employee_entity,
                employee_stable_id: sensor.stable_id,
                body_recoverable: false,
            });

            deleted_events.send(ForestKeeperInventoryDeletedEvent {
                employee: employee_entity,
                employee_stable_id: sensor.stable_id,
                deleted_items: sensor.inventory_items,
            });
        }
    }
}

fn forest_keeper_apply_stun(
    mut applied_events: EventReader<ForestKeeperStunAppliedEvent>,
    mut adjusted_events: EventWriter<ForestKeeperStunAdjustedEvent>,
    mut dropped_events: EventWriter<ForestKeeperDroppedEmployeeEvent>,
    mut state_events: EventWriter<ForestKeeperStateChangedEvent>,
    mut keepers: Query<
        (
            Entity,
            &mut ForestKeeperState,
            &mut ForestKeeperMovement,
            &mut UnitStats,
            &ForestKeeperTarget,
        ),
        With<ForestKeeper>,
    >,
) {
    for event in applied_events.read() {
        let Ok((keeper_entity, mut state, mut movement, mut stats, target)) =
            keepers.get_mut(event.keeper)
        else {
            continue;
        };

        if *state == ForestKeeperState::Dead {
            continue;
        }

        let adjusted_ticks = fixed_ticks_scaled(event.base_ticks, FOREST_KEEPER_STUN_MULTIPLIER);
        movement.stun_timer_ticks = adjusted_ticks;
        stats.move_speed = I32F32::ZERO;

        if *state == ForestKeeperState::Eating && target.has_target {
            dropped_events.send(ForestKeeperDroppedEmployeeEvent {
                keeper: keeper_entity,
                employee: target.target_entity,
                employee_stable_id: target.target_stable_id,
            });
        }

        set_forest_keeper_state(
            keeper_entity,
            &mut state,
            ForestKeeperState::Stunned,
            &mut state_events,
        );

        adjusted_events.send(ForestKeeperStunAdjustedEvent {
            keeper: keeper_entity,
            source: event.source,
            adjusted_ticks,
        });
    }

    for (keeper_entity, mut state, mut movement, mut stats, target) in keepers.iter_mut() {
        if *state != ForestKeeperState::Stunned {
            continue;
        }

        if movement.stun_timer_ticks > 0 {
            movement.stun_timer_ticks -= 1;
            continue;
        }

        let next_state = if target.has_target {
            ForestKeeperState::Chase
        } else {
            ForestKeeperState::Roam
        };
        stats.move_speed = if target.has_target {
            FOREST_KEEPER_CHASE_SPEED
        } else {
            FOREST_KEEPER_ROAM_SPEED
        };

        set_forest_keeper_state(keeper_entity, &mut state, next_state, &mut state_events);
    }
}

fn forest_keeper_apply_fire(
    sim_hz: Res<SimHz>,
    mut fire_events: EventReader<ForestKeeperFireStartedEvent>,
    mut burned_events: EventWriter<ForestKeeperBurnedDownEvent>,
    mut defeated_events: EventWriter<ForestKeeperDefeatedEvent>,
    mut state_events: EventWriter<ForestKeeperStateChangedEvent>,
    mut keepers: Query<(Entity, &mut ForestKeeperState, &mut ForestKeeperMovement), With<ForestKeeper>>,
) {
    for event in fire_events.read() {
        let Ok((keeper_entity, mut state, mut movement)) = keepers.get_mut(event.keeper) else {
            continue;
        };

        if *state == ForestKeeperState::Dead {
            continue;
        }

        movement.burn_timer_ticks = fixed_seconds_to_ticks(FOREST_KEEPER_BURN_SECONDS, sim_hz.0);
        set_forest_keeper_state(
            keeper_entity,
            &mut state,
            ForestKeeperState::Burning,
            &mut state_events,
        );
    }

    for (keeper_entity, mut state, mut movement) in keepers.iter_mut() {
        if *state != ForestKeeperState::Burning {
            continue;
        }

        if movement.burn_timer_ticks > 0 {
            movement.burn_timer_ticks -= 1;
            continue;
        }

        set_forest_keeper_state(
            keeper_entity,
            &mut state,
            ForestKeeperState::Dead,
            &mut state_events,
        );
        burned_events.send(ForestKeeperBurnedDownEvent {
            keeper: keeper_entity,
            source: Entity::PLACEHOLDER,
        });
        defeated_events.send(ForestKeeperDefeatedEvent {
            keeper: keeper_entity,
            source: Entity::PLACEHOLDER,
        });
    }
}

fn forest_keeper_take_damage(
    mut damage_events: EventReader<IncomingDamageEvent>,
    mut taken_events: EventWriter<ForestKeeperDamageTakenEvent>,
    mut defeated_events: EventWriter<ForestKeeperDefeatedEvent>,
    mut state_events: EventWriter<ForestKeeperStateChangedEvent>,
    mut keepers: Query<(Entity, &mut Health, &mut ForestKeeperState), With<ForestKeeper>>,
) {
    for event in damage_events.read() {
        let Ok((keeper_entity, mut health, mut state)) = keepers.get_mut(event.target) else {
            continue;
        };

        if *state == ForestKeeperState::Dead {
            continue;
        }

        health.current -= event.raw_amount;
        if health.current < I32F32::ZERO {
            health.current = I32F32::ZERO;
        }

        taken_events.send(ForestKeeperDamageTakenEvent {
            keeper: keeper_entity,
            source: event.source,
            damage: event.raw_amount,
            remaining_health: health.current,
        });

        if health.current <= I32F32::ZERO {
            set_forest_keeper_state(
                keeper_entity,
                &mut state,
                ForestKeeperState::Dead,
                &mut state_events,
            );
            defeated_events.send(ForestKeeperDefeatedEvent {
                keeper: keeper_entity,
                source: event.source,
            });
        }
    }
}

fn forest_keeper_crush_on_death(
    mut defeated_events: EventReader<ForestKeeperDefeatedEvent>,
    mut damage_events: EventWriter<IncomingDamageEvent>,
    mut crush_events: EventWriter<ForestKeeperCrushKillEvent>,
    keepers: Query<&SimPosition, With<ForestKeeper>>,
    employees: Query<(Entity, &SimPosition, &ForestKeeperEmployeeSensor), Without<ForestKeeper>>,
) {
    for event in defeated_events.read() {
        let Ok(keeper_position) = keepers.get(event.keeper) else {
            continue;
        };

        for (employee_entity, employee_position, sensor) in employees.iter() {
            if !sensor.is_alive {
                continue;
            }

            if fixed_distance_sq(*keeper_position, *employee_position)
                > fixed_square(FOREST_KEEPER_CRUSH_RANGE)
            {
                continue;
            }

            damage_events.send(IncomingDamageEvent {
                target: employee_entity,
                raw_amount: FOREST_KEEPER_CONTACT_DAMAGE,
                damage_type: DamageType::Standard,
                source: event.keeper,
            });

            crush_events.send(ForestKeeperCrushKillEvent {
                keeper: event.keeper,
                employee: employee_entity,
                employee_stable_id: sensor.stable_id,
                cause_id: FOREST_KEEPER_GRAVITY_CAUSE_ID,
                body_recoverable: true,
            });
        }
    }
}

fn forest_keeper_report_zap_gun_difficulty(
    mut events: EventWriter<ForestKeeperZapGunDifficultyEvent>,
    keepers: Query<Entity, With<ForestKeeper>>,
) {
    for keeper in keepers.iter() {
        events.send(ForestKeeperZapGunDifficultyEvent {
            keeper,
            difficulty: FOREST_KEEPER_ZAP_GUN_DIFFICULTY,
        });
    }
}

fn forest_keeper_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    sim_hz: Res<SimHz>,
    keepers: Query<
        (
            &ForestKeeper,
            &SimPosition,
            &Health,
            &UnitStats,
            &ForestKeeperState,
            &ForestKeeperTarget,
            &ForestKeeperMovement,
        ),
        With<ForestKeeper>,
    >,
    suspicions: Query<&ForestKeeperSuspicion>,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(sim_hz.0.to_bits() as u64);
    checksum.accumulate(FOREST_KEEPER_SOURCE_REVISION as u64);
    checksum.accumulate(FOREST_KEEPER_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(FOREST_KEEPER_HP.to_bits() as u64);
    checksum.accumulate(FOREST_KEEPER_POWER_LEVEL.to_bits() as u64);
    checksum.accumulate(FOREST_KEEPER_MAX_SPAWNED as u64);
    checksum.accumulate(FOREST_KEEPER_CONTACT_DAMAGE.to_bits() as u64);
    checksum.accumulate(FOREST_KEEPER_STUN_MULTIPLIER.to_bits() as u64);
    checksum.accumulate(FOREST_KEEPER_ZAP_GUN_DIFFICULTY.to_bits() as u64);
    checksum.accumulate(FOREST_KEEPER_DOOR_SPEED_MULTIPLIER.to_bits() as u64);
    checksum.accumulate(FOREST_KEEPER_CAN_SEE_THROUGH_FOG as u64);
    checksum.accumulate(FOREST_KEEPER_SPAWN_DELAY_SECONDS.to_bits() as u64);

    accumulate_str(&mut checksum, 0x1000, FOREST_KEEPER_ID);
    accumulate_str(&mut checksum, 0x1001, FOREST_KEEPER_NAME);
    accumulate_str(&mut checksum, 0x1002, FOREST_KEEPER_TYPE);
    accumulate_str(&mut checksum, 0x1003, FOREST_KEEPER_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, FOREST_KEEPER_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1005, FOREST_KEEPER_EXTRACTED_AT);
    accumulate_str(&mut checksum, 0x1006, FOREST_KEEPER_DWELLS);
    accumulate_str(&mut checksum, 0x1007, FOREST_KEEPER_DANGER);
    accumulate_str(&mut checksum, 0x1008, FOREST_KEEPER_SCIENTIFIC_NAME);
    accumulate_str(&mut checksum, 0x1009, FOREST_KEEPER_INTERNAL_NAME);
    accumulate_str(&mut checksum, 0x100A, FOREST_KEEPER_PIP_SIZE);

    for dependency in FOREST_KEEPER_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for behavior in FOREST_KEEPER_FRONTMATTER_BEHAVIOR {
        accumulate_str(&mut checksum, 0x3000, behavior);
    }

    for rule in FOREST_KEEPER_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x4000, rule.condition);
        accumulate_str(&mut checksum, 0x4001, rule.outcome);
    }

    for (keeper, position, health, stats, state, target, movement) in keepers.iter() {
        checksum.accumulate(keeper.stable_id);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);
        checksum.accumulate(forest_keeper_state_bits(*state));
        checksum.accumulate(target.has_target as u64);
        checksum.accumulate(target.target_stable_id);
        checksum.accumulate(target.last_known_position.x.to_bits() as u64);
        checksum.accumulate(target.last_known_position.y.to_bits() as u64);
        checksum.accumulate(target.target_visible as u64);
        checksum.accumulate(target.lost_line_of_sight_ticks as u64);
        checksum.accumulate(movement.rise_timer_ticks as u64);
        checksum.accumulate(movement.search_timer_ticks as u64);
        checksum.accumulate(movement.stun_timer_ticks as u64);
        checksum.accumulate(movement.burn_timer_ticks as u64);
        checksum.accumulate(movement.current_speed.to_bits() as u64);
        checksum.accumulate(movement.search_precision.to_bits() as u64);
        checksum.accumulate(movement.search_width.to_bits() as u64);
    }

    for suspicion in suspicions.iter() {
        checksum.accumulate(suspicion.employee_stable_id);
        checksum.accumulate(suspicion.meter.to_bits() as u64);
    }
}

fn first_visible_employee(
    keeper_position: SimPosition,
    employees: &Query<
        (
            Entity,
            &SimPosition,
            &ForestKeeperEmployeeSensor,
            &mut ForestKeeperSuspicion,
        ),
        Without<ForestKeeper>,
    >,
) -> Option<u64> {
    let mut best: Option<(u64, I32F32)> = None;

    for (_entity, position, sensor, _suspicion) in employees.iter() {
        if !sensor.is_alive || !sensor.visible_to_keeper || !sensor.in_field_of_view {
            continue;
        }

        let view_distance = if sensor.in_fog {
            FOREST_KEEPER_FOG_VIEW_DISTANCE
        } else {
            FOREST_KEEPER_VIEW_DISTANCE
        };

        let distance_sq = fixed_distance_sq(keeper_position, *position);
        if distance_sq > fixed_square(view_distance) {
            continue;
        }

        match best {
            Some((_stable_id, best_distance_sq)) if distance_sq >= best_distance_sq => {}
            _ => best = Some((sensor.stable_id, distance_sq)),
        }
    }

    best.map(|(stable_id, _distance_sq)| stable_id)
}

fn employee_position_by_stable_id(
    stable_id: u64,
    employees: &Query<(Entity, &SimPosition, &ForestKeeperEmployeeSensor), Without<ForestKeeper>>,
) -> Option<(SimPosition, ForestKeeperEmployeeSensor)> {
    for (_entity, position, sensor) in employees.iter() {
        if sensor.stable_id == stable_id {
            return Some((*position, *sensor));
        }
    }

    None
}

fn start_forest_keeper_search(
    keeper_entity: Entity,
    state: &mut ForestKeeperState,
    _target: &mut ForestKeeperTarget,
    movement: &mut ForestKeeperMovement,
    stats: &mut UnitStats,
    sim_hz: I32F32,
    state_events: &mut EventWriter<ForestKeeperStateChangedEvent>,
) {
    movement.search_timer_ticks = fixed_seconds_to_ticks(FOREST_KEEPER_SEARCH_SECONDS, sim_hz);
    movement.search_precision = FOREST_KEEPER_SEARCH_PRECISION;
    movement.search_width = FOREST_KEEPER_SEARCH_WIDTH;
    stats.move_speed = FOREST_KEEPER_ROAM_SPEED;
    set_forest_keeper_state(
        keeper_entity,
        state,
        ForestKeeperState::Search,
        state_events,
    );
}

fn set_forest_keeper_state(
    keeper: Entity,
    state: &mut ForestKeeperState,
    next: ForestKeeperState,
    events: &mut EventWriter<ForestKeeperStateChangedEvent>,
) {
    if *state == next {
        return;
    }

    let previous = *state;
    *state = next;
    events.send(ForestKeeperStateChangedEvent {
        keeper,
        from: previous,
        to: next,
    });
}

fn move_axis_toward(position: &mut SimPosition, target: SimPosition, max_step: I32F32) {
    position.x = move_scalar_toward(position.x, target.x, max_step);
    position.y = move_scalar_toward(position.y, target.y, max_step);
}

fn move_scalar_toward(current: I32F32, target: I32F32, max_step: I32F32) -> I32F32 {
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

fn fixed_distance_sq(a: SimPosition, b: SimPosition) -> I32F32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    dx * dx + dy * dy
}

fn fixed_square(value: I32F32) -> I32F32 {
    value * value
}

fn fixed_clamp_unit(value: I32F32) -> I32F32 {
    if value < I32F32::ZERO {
        I32F32::ZERO
    } else if value > I32F32::ONE {
        I32F32::ONE
    } else {
        value
    }
}

fn fixed_seconds_to_ticks(seconds: I32F32, sim_hz: I32F32) -> u32 {
    let ticks = seconds * sim_hz;
    if ticks <= I32F32::ZERO {
        0
    } else {
        ticks.ceil().to_num::<u32>()
    }
}

fn fixed_ticks_scaled(base_ticks: u32, multiplier: I32F32) -> u32 {
    let ticks = I32F32::from_num(base_ticks) * multiplier;
    if ticks <= I32F32::ONE {
        1
    } else {
        ticks.ceil().to_num::<u32>()
    }
}

fn forest_keeper_state_bits(state: ForestKeeperState) -> u64 {
    match state {
        ForestKeeperState::Spawning => 0,
        ForestKeeperState::Roam => 1,
        ForestKeeperState::Chase => 2,
        ForestKeeperState::Search => 3,
        ForestKeeperState::Eating => 4,
        ForestKeeperState::Stunned => 5,
        ForestKeeperState::Burning => 6,
        ForestKeeperState::Dead => 7,
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}
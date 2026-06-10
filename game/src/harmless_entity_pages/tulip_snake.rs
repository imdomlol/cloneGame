// Sources: vault/harmless_entity_pages/tulip_snake.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{
    EntityKilledEvent, Health, IncomingDamageEvent, NoiseEmittedEvent, SimChecksumState, SimHz,
    SimPosition, SimTick, UnitStats,
};

pub const TULIP_SNAKE_ID: &str = "tulip_snake";
pub const TULIP_SNAKE_NAME: &str = "Tulip Snake";
pub const TULIP_SNAKE_TYPE: &str = "harmless_entity_pages";
pub const TULIP_SNAKE_SUBTYPE: &str = "creature";
pub const TULIP_SNAKE_SOURCE_URL: &str =
    "https://lethal-company.fandom.com/wiki/Tulip_Snake";
pub const TULIP_SNAKE_SOURCE_REVISION: u32 = 21078;
pub const TULIP_SNAKE_EXTRACTED_AT: &str = "2026-06-07";
pub const TULIP_SNAKE_CONFIDENCE_BASIS_POINTS: u16 = 82;

pub const TULIP_SNAKE_DWELLS: &str = "Outside (Daytime)\nInside (Unnatural)";
pub const TULIP_SNAKE_SCIENTIFIC_NAME: &str = "Draco-tulipa";
pub const TULIP_SNAKE_POWER_LEVEL: I32F32 = I32F32::lit("0.5");
pub const TULIP_SNAKE_MAX_SPAWNED: usize = 12;
pub const TULIP_SNAKE_SHOVEL_HP: I32F32 = I32F32::lit("1");
pub const TULIP_SNAKE_SHOCK_RESPONSE: &str = "Susceptible";
pub const TULIP_SNAKE_RADAR_PIP_SIZE: &str = "Small";
pub const TULIP_SNAKE_DOOR_OPEN_SPEED: I32F32 = I32F32::lit("0.3");
pub const TULIP_SNAKE_CAN_SEE_THROUGH_FOG: bool = false;
pub const TULIP_SNAKE_STUN_MULTIPLIER: &str = "Immune";
pub const TULIP_SNAKE_LEAVE_TIME: &str = "3:21 PM";
pub const TULIP_SNAKE_LEAVE_TIME_MINUTES_AFTER_MIDNIGHT: u16 = 921;
pub const TULIP_SNAKE_IMAGE: &str = "TulipSnakeCel.png";

pub const TULIP_SNAKE_ONE_ATTACHED_WEIGHT_LB: I32F32 = I32F32::lit("3");
pub const TULIP_SNAKE_FIVE_ATTACHED_WEIGHT_LB: I32F32 = I32F32::lit("16");
pub const TULIP_SNAKE_MAX_ATTACHED_PER_EMPLOYEE: u8 = 5;
pub const TULIP_SNAKE_MULTI_ATTACH_FLIGHT_SECONDS: I32F32 = I32F32::lit("10");
pub const TULIP_SNAKE_DEFAULT_TIRE_ATTEMPTS: u8 = 3;
pub const TULIP_SNAKE_POUNCE_RANGE: I32F32 = I32F32::lit("3");
pub const TULIP_SNAKE_LATCH_RANGE: I32F32 = I32F32::lit("1");
pub const TULIP_SNAKE_WATCH_RANGE: I32F32 = I32F32::lit("18");
pub const TULIP_SNAKE_CHIRP_INTERVAL_SECONDS: I32F32 = I32F32::lit("3");

pub const TULIP_SNAKE_DEPENDS_ON: [&str; 25] = [
    "employee",
    "jetpack",
    "manticoil",
    "shovel",
    "stop_sign",
    "yield_sign",
    "interior",
    "fire_exit",
    "door",
    "monitors",
    "guide_camera_duty",
    "snare_flea",
    "hoarding_bug",
    "bracken",
    "baboon_hawk",
    "maneater",
    "teleporter",
    "old_bird",
    "weather",
    "items",
    "scrap",
    "eyeless_dog",
    "landmine",
    "spike_trap",
    "weapon",
];

pub const TULIP_SNAKE_FRONTMATTER_BEHAVIOR: [&str; 2] = ["Roaming", "Carrying"];

pub const TULIP_SNAKE_OCCURRENCE: [TulipSnakeOccurrence; 5] = [
    TulipSnakeOccurrence {
        moon: "Adamance",
        base_spawn_chance: I32F32::lit("6.14"),
    },
    TulipSnakeOccurrence {
        moon: "Artifice",
        base_spawn_chance: I32F32::lit("2.6"),
    },
    TulipSnakeOccurrence {
        moon: "Vow",
        base_spawn_chance: I32F32::lit("1.55"),
    },
    TulipSnakeOccurrence {
        moon: "Assurance",
        base_spawn_chance: I32F32::lit("1.5"),
    },
    TulipSnakeOccurrence {
        moon: "March",
        base_spawn_chance: I32F32::lit("0.52"),
    },
];

pub const TULIP_SNAKE_BEHAVIORAL_MECHANICS: [TulipSnakeBehaviorRule; 18] = [
    TulipSnakeBehaviorRule {
        condition: "roaming",
        outcome: "it moves similarly to manticoil and can sometimes fly from one area to another",
    },
    TulipSnakeBehaviorRule {
        condition: "it detects an employee",
        outcome: "it repeatedly attempts to pounce until it latches onto the employee's helmet",
    },
    TulipSnakeBehaviorRule {
        condition: "exactly 1 tulip snake is attached",
        outcome: "it adds about 3 lb of carried weight",
    },
    TulipSnakeBehaviorRule {
        condition: "5 tulip snakes are attached to the same employee",
        outcome: "they add roughly 16 lb total, and up to 5 snakes can carry the same employee at once",
    },
    TulipSnakeBehaviorRule {
        condition: "more than 1 tulip snake is attached",
        outcome: "the employee is forced into the sky for about 10 seconds before falling back down",
    },
    TulipSnakeBehaviorRule {
        condition: "a tulip snake is not trying to fly away with the employee",
        outcome: "it gazes downward and blocks a small part of the employee's vision",
    },
    TulipSnakeBehaviorRule {
        condition: "a tulip snake is latched onto an employee",
        outcome: "the employee can occasionally hear its chirping or chuckling call",
    },
    TulipSnakeBehaviorRule {
        condition: "the employee waves a shovel, stop sign, or yield sign",
        outcome: "all attached tulip snakes are killed immediately and the employee loses the flying effect",
    },
    TulipSnakeBehaviorRule {
        condition: "a tulip snake tires after several lift attempts",
        outcome: "it releases the employee and returns to roaming",
    },
    TulipSnakeBehaviorRule {
        condition: "the attached employee dies",
        outcome: "the tulip snake returns to its roaming phase",
    },
    TulipSnakeBehaviorRule {
        condition: "an attached employee enters the interior through the main entrance or a fire exit",
        outcome: "the snake can continue indoors and can open doors",
    },
    TulipSnakeBehaviorRule {
        condition: "a snake carries the employee into the main-room fan",
        outcome: "the employee and snake are both decapitated and killed",
    },
    TulipSnakeBehaviorRule {
        condition: "a tulip snake appears on ship duty",
        outcome: "its radar pip can be confused with a suffocated employee, a carried corpse, or an employee holding a maneater on the monitors",
    },
    TulipSnakeBehaviorRule {
        condition: "a crew needs a teleport decision",
        outcome: "the team can use a shared cue system with teleporter calls before the session starts",
    },
    TulipSnakeBehaviorRule {
        condition: "lightning strikes an employee carrying conductive items or scrap while a tulip snake is attached",
        outcome: "the snake can die",
    },
    TulipSnakeBehaviorRule {
        condition: "a tulip snake is in the path of an eyeless dog while it is attacking a sound",
        outcome: "the snake can die",
    },
    TulipSnakeBehaviorRule {
        condition: "a landmine or spike trap hits the snake",
        outcome: "the snake can die",
    },
    TulipSnakeBehaviorRule {
        condition: "the snake is attacked by baboon hawks or an old bird",
        outcome: "it can die",
    },
];

pub struct TulipSnakePlugin;

impl Plugin for TulipSnakePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnTulipSnakeEvent>()
            .add_event::<TulipSnakeStateChangedEvent>()
            .add_event::<TulipSnakeLatchedEvent>()
            .add_event::<TulipSnakeReleasedEvent>()
            .add_event::<TulipSnakeEmployeeCarryChangedEvent>()
            .add_event::<TulipSnakeWeaponWaveEvent>()
            .add_event::<TulipSnakeEmployeeDeathObservedEvent>()
            .add_event::<TulipSnakeEmployeeInteriorTransitionEvent>()
            .add_event::<TulipSnakeDoorOpenRequestEvent>()
            .add_event::<TulipSnakeMainRoomFanCollisionEvent>()
            .add_event::<TulipSnakeHazardHitEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_tulip_snake,
                    tulip_snake_detect_employee,
                    tulip_snake_latch_employee,
                    tulip_snake_update_carry_effect,
                    tulip_snake_chirp,
                    tulip_snake_release_when_tired,
                    tulip_snake_release_dead_employee,
                    tulip_snake_follow_employee_inside,
                    tulip_snake_open_doors,
                    tulip_snake_weapon_wave_kills_attached,
                    tulip_snake_main_room_fan_kills,
                    tulip_snake_hazard_kills,
                    tulip_snake_apply_damage,
                    tulip_snake_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TulipSnakeBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TulipSnakeOccurrence {
    pub moon: &'static str,
    pub base_spawn_chance: I32F32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TulipSnake;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TulipSnakeEmployeeTarget {
    pub stable_id: u64,
    pub alive: bool,
    pub inside_facility: bool,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TulipSnakeDoor {
    pub stable_id: u64,
    pub open: bool,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TulipSnakeMainRoomFan;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TulipSnakeConductiveCarrier;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TulipSnakeHazardKind {
    #[default]
    Lightning,
    EyelessDog,
    Landmine,
    SpikeTrap,
    BaboonHawk,
    OldBird,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TulipSnakeState {
    #[default]
    Roaming,
    Pouncing,
    AttachedViewing,
    AttachedLifting,
    FallingBack,
    IndoorAttached,
    Dead,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TulipSnakeTimers {
    pub pounce_ticks: u32,
    pub lift_ticks: u32,
    pub chirp_ticks: u32,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TulipSnakeAttachment {
    pub employee: Option<Entity>,
    pub employee_stable_id: u64,
    pub lift_attempts: u8,
    pub tire_after_attempts: u8,
    pub carried_weight_lb: I32F32,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TulipSnakeMovement {
    pub roaming_anchor: SimPosition,
    pub target_employee: Option<Entity>,
    pub target_employee_stable_id: u64,
}

#[derive(Bundle)]
pub struct TulipSnakeBundle {
    pub name: Name,
    pub snake: TulipSnake,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub state: TulipSnakeState,
    pub timers: TulipSnakeTimers,
    pub attachment: TulipSnakeAttachment,
    pub movement: TulipSnakeMovement,
}

impl TulipSnakeBundle {
    pub fn new(event: SpawnTulipSnakeEvent, sim_hz: I32F32) -> Self {
        Self {
            name: Name::new(TULIP_SNAKE_NAME),
            snake: TulipSnake,
            position: event.position,
            health: Health::full(TULIP_SNAKE_SHOVEL_HP),
            stats: UnitStats {
                move_speed: event.roam_move_speed,
                attack_range: TULIP_SNAKE_POUNCE_RANGE,
                attack_damage: I32F32::lit("0"),
                attack_speed: I32F32::lit("0"),
                watch_range: TULIP_SNAKE_WATCH_RANGE,
            },
            state: TulipSnakeState::Roaming,
            timers: TulipSnakeTimers {
                pounce_ticks: 0,
                lift_ticks: 0,
                chirp_ticks: seconds_to_ticks(TULIP_SNAKE_CHIRP_INTERVAL_SECONDS, sim_hz),
            },
            attachment: TulipSnakeAttachment {
                employee: None,
                employee_stable_id: 0,
                lift_attempts: 0,
                tire_after_attempts: event.tire_after_attempts,
                carried_weight_lb: I32F32::lit("0"),
            },
            movement: TulipSnakeMovement {
                roaming_anchor: event.position,
                target_employee: None,
                target_employee_stable_id: 0,
            },
        }
    }
}

#[derive(Event, Clone, Copy, Debug)]
pub struct SpawnTulipSnakeEvent {
    pub position: SimPosition,
    pub roam_move_speed: I32F32,
    pub tire_after_attempts: u8,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TulipSnakeStateChangedEvent {
    pub snake: Entity,
    pub from: TulipSnakeState,
    pub to: TulipSnakeState,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TulipSnakeLatchedEvent {
    pub snake: Entity,
    pub employee: Entity,
    pub employee_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TulipSnakeReleasedEvent {
    pub snake: Entity,
    pub employee: Entity,
    pub employee_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TulipSnakeEmployeeCarryChangedEvent {
    pub employee: Entity,
    pub employee_stable_id: u64,
    pub attached_count: u8,
    pub carried_weight_lb: I32F32,
    pub flying: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TulipSnakeWeaponWaveEvent {
    pub employee: Entity,
    pub employee_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TulipSnakeEmployeeDeathObservedEvent {
    pub employee: Entity,
    pub employee_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TulipSnakeEmployeeInteriorTransitionEvent {
    pub employee: Entity,
    pub employee_stable_id: u64,
    pub inside_facility: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TulipSnakeDoorOpenRequestEvent {
    pub snake: Entity,
    pub door: Entity,
    pub door_stable_id: u64,
    pub speed: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TulipSnakeMainRoomFanCollisionEvent {
    pub snake: Entity,
    pub employee: Entity,
    pub employee_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TulipSnakeHazardHitEvent {
    pub snake: Entity,
    pub hazard: TulipSnakeHazardKind,
}

fn spawn_tulip_snake(
    mut commands: Commands,
    mut events: EventReader<SpawnTulipSnakeEvent>,
    sim_hz: Res<SimHz>,
    snakes: Query<(), With<TulipSnake>>,
) {
    let mut spawned_count = snakes.iter().count();

    for event in events.read() {
        if spawned_count >= TULIP_SNAKE_MAX_SPAWNED {
            break;
        }

        commands.spawn(TulipSnakeBundle::new(*event, sim_hz.0));
        spawned_count += 1;
    }
}

fn tulip_snake_detect_employee(
    mut state_events: EventWriter<TulipSnakeStateChangedEvent>,
    mut snakes: Query<
        (
            Entity,
            &SimPosition,
            &mut TulipSnakeState,
            &mut TulipSnakeMovement,
        ),
        With<TulipSnake>,
    >,
    employees: Query<(Entity, &SimPosition, &TulipSnakeEmployeeTarget)>,
) {
    let detect_range_squared = TULIP_SNAKE_WATCH_RANGE * TULIP_SNAKE_WATCH_RANGE;

    for (snake_entity, snake_position, mut state, mut movement) in snakes.iter_mut() {
        if *state != TulipSnakeState::Roaming {
            continue;
        }

        let mut selected: Option<(Entity, u64, I32F32)> = None;

        for (employee_entity, employee_position, employee) in employees.iter() {
            if !employee.alive {
                continue;
            }

            let distance = distance_squared(*snake_position, *employee_position);
            if distance > detect_range_squared {
                continue;
            }

            selected = match selected {
                Some((selected_entity, selected_id, selected_distance))
                    if selected_distance < distance
                        || (selected_distance == distance && selected_id <= employee.stable_id) =>
                {
                    Some((selected_entity, selected_id, selected_distance))
                }
                _ => Some((employee_entity, employee.stable_id, distance)),
            };
        }

        if let Some((target, stable_id, _)) = selected {
            movement.target_employee = Some(target);
            movement.target_employee_stable_id = stable_id;
            set_state(
                snake_entity,
                &mut state,
                TulipSnakeState::Pouncing,
                &mut state_events,
            );
        }
    }
}

fn tulip_snake_latch_employee(
    mut latch_events: EventWriter<TulipSnakeLatchedEvent>,
    mut state_events: EventWriter<TulipSnakeStateChangedEvent>,
    mut snake_sets: ParamSet<(
        Query<
            (
                Entity,
                &SimPosition,
                &mut TulipSnakeState,
                &mut TulipSnakeTimers,
                &mut TulipSnakeAttachment,
                &mut TulipSnakeMovement,
            ),
            With<TulipSnake>,
        >,
        Query<&TulipSnakeAttachment, With<TulipSnake>>,
    )>,
    employees: Query<(Entity, &SimPosition, &TulipSnakeEmployeeTarget)>,
) {
    let latch_range_squared = TULIP_SNAKE_LATCH_RANGE * TULIP_SNAKE_LATCH_RANGE;
    let mut attached_counts = std::collections::BTreeMap::<u64, u8>::new();

    for attachment in snake_sets.p1().iter() {
        if attachment.employee_stable_id == 0 {
            continue;
        }

        let count = attached_counts
            .entry(attachment.employee_stable_id)
            .or_insert(0);
        *count = count.saturating_add(1);
    }

    for (snake_entity, snake_position, mut state, mut timers, mut attachment, mut movement) in
        snake_sets.p0().iter_mut()
    {
        if *state != TulipSnakeState::Pouncing {
            continue;
        }

        timers.pounce_ticks = timers.pounce_ticks.saturating_add(1);

        let Some(target_entity) = movement.target_employee else {
            set_state(
                snake_entity,
                &mut state,
                TulipSnakeState::Roaming,
                &mut state_events,
            );
            continue;
        };

        let Ok((employee_entity, employee_position, employee)) = employees.get(target_entity) else {
            set_state(
                snake_entity,
                &mut state,
                TulipSnakeState::Roaming,
                &mut state_events,
            );
            continue;
        };

        let attached_count = *attached_counts.get(&employee.stable_id).unwrap_or(&0);

        if !employee.alive
            || distance_squared(*snake_position, *employee_position) > latch_range_squared
            || attached_count >= TULIP_SNAKE_MAX_ATTACHED_PER_EMPLOYEE
        {
            continue;
        }

        attachment.employee = Some(employee_entity);
        attachment.employee_stable_id = employee.stable_id;
        attachment.carried_weight_lb = TULIP_SNAKE_ONE_ATTACHED_WEIGHT_LB;
        movement.target_employee = None;
        movement.target_employee_stable_id = 0;
        timers.pounce_ticks = 0;

        let count = attached_counts.entry(employee.stable_id).or_insert(0);
        *count = count.saturating_add(1);

        set_state(
            snake_entity,
            &mut state,
            TulipSnakeState::AttachedViewing,
            &mut state_events,
        );
        latch_events.send(TulipSnakeLatchedEvent {
            snake: snake_entity,
            employee: employee_entity,
            employee_stable_id: employee.stable_id,
        });
    }
}

fn tulip_snake_update_carry_effect(
    sim_hz: Res<SimHz>,
    mut carry_events: EventWriter<TulipSnakeEmployeeCarryChangedEvent>,
    mut state_events: EventWriter<TulipSnakeStateChangedEvent>,
    mut snakes: Query<
        (
            Entity,
            &mut TulipSnakeState,
            &mut TulipSnakeTimers,
            &mut TulipSnakeAttachment,
        ),
        With<TulipSnake>,
    >,
    employees: Query<(Entity, &TulipSnakeEmployeeTarget)>,
) {
    let lift_ticks_needed = seconds_to_ticks(TULIP_SNAKE_MULTI_ATTACH_FLIGHT_SECONDS, sim_hz.0);

    for (employee_entity, employee) in employees.iter() {
        let mut attached_count = 0_u8;
        for (_, _, _, attachment) in snakes.iter() {
            if attachment.employee_stable_id == employee.stable_id {
                attached_count = attached_count.saturating_add(1);
            }
        }

        if attached_count == 0 {
            continue;
        }

        let flying = attached_count > 1;
        let weight = carried_weight_for_count(attached_count);
        carry_events.send(TulipSnakeEmployeeCarryChangedEvent {
            employee: employee_entity,
            employee_stable_id: employee.stable_id,
            attached_count,
            carried_weight_lb: weight,
            flying,
        });

        for (snake_entity, mut state, mut timers, mut attachment) in snakes.iter_mut() {
            if attachment.employee_stable_id != employee.stable_id {
                continue;
            }

            attachment.carried_weight_lb = weight;

            if flying {
                timers.lift_ticks = timers.lift_ticks.saturating_add(1);
                set_state(
                    snake_entity,
                    &mut state,
                    TulipSnakeState::AttachedLifting,
                    &mut state_events,
                );

                if timers.lift_ticks >= lift_ticks_needed {
                    timers.lift_ticks = 0;
                    attachment.lift_attempts = attachment.lift_attempts.saturating_add(1);
                    set_state(
                        snake_entity,
                        &mut state,
                        TulipSnakeState::FallingBack,
                        &mut state_events,
                    );
                }
            } else {
                timers.lift_ticks = 0;
                set_state(
                    snake_entity,
                    &mut state,
                    TulipSnakeState::AttachedViewing,
                    &mut state_events,
                );
            }
        }
    }
}

fn tulip_snake_chirp(
    sim_hz: Res<SimHz>,
    mut noise_events: EventWriter<NoiseEmittedEvent>,
    mut snakes: Query<(Entity, &SimPosition, &TulipSnakeState, &mut TulipSnakeTimers), With<TulipSnake>>,
) {
    let chirp_interval_ticks = seconds_to_ticks(TULIP_SNAKE_CHIRP_INTERVAL_SECONDS, sim_hz.0);

    for (snake_entity, position, state, mut timers) in snakes.iter_mut() {
        if !is_attached_state(*state) {
            continue;
        }

        timers.chirp_ticks = timers.chirp_ticks.saturating_add(1);
        if timers.chirp_ticks < chirp_interval_ticks {
            continue;
        }

        timers.chirp_ticks = 0;
        noise_events.send(NoiseEmittedEvent {
            source: snake_entity,
            position: *position,
            amount: I32F32::lit("1"),
        });
    }
}

fn tulip_snake_release_when_tired(
    mut release_events: EventWriter<TulipSnakeReleasedEvent>,
    mut state_events: EventWriter<TulipSnakeStateChangedEvent>,
    mut snakes: Query<
        (
            Entity,
            &mut TulipSnakeState,
            &mut TulipSnakeAttachment,
            &mut TulipSnakeMovement,
        ),
        With<TulipSnake>,
    >,
) {
    for (snake_entity, mut state, mut attachment, mut movement) in snakes.iter_mut() {
        if !is_attached_state(*state) || attachment.lift_attempts < attachment.tire_after_attempts {
            continue;
        }

        release_attachment(
            snake_entity,
            &mut state,
            &mut attachment,
            &mut movement,
            &mut release_events,
            &mut state_events,
        );
    }
}

fn tulip_snake_release_dead_employee(
    mut death_events: EventReader<TulipSnakeEmployeeDeathObservedEvent>,
    mut release_events: EventWriter<TulipSnakeReleasedEvent>,
    mut state_events: EventWriter<TulipSnakeStateChangedEvent>,
    mut snakes: Query<
        (
            Entity,
            &mut TulipSnakeState,
            &mut TulipSnakeAttachment,
            &mut TulipSnakeMovement,
        ),
        With<TulipSnake>,
    >,
) {
    for event in death_events.read() {
        for (snake_entity, mut state, mut attachment, mut movement) in snakes.iter_mut() {
            if attachment.employee_stable_id != event.employee_stable_id {
                continue;
            }

            release_attachment(
                snake_entity,
                &mut state,
                &mut attachment,
                &mut movement,
                &mut release_events,
                &mut state_events,
            );
        }
    }
}

fn tulip_snake_follow_employee_inside(
    mut transition_events: EventReader<TulipSnakeEmployeeInteriorTransitionEvent>,
    mut state_events: EventWriter<TulipSnakeStateChangedEvent>,
    mut snakes: Query<(Entity, &mut TulipSnakeState, &TulipSnakeAttachment), With<TulipSnake>>,
) {
    for event in transition_events.read() {
        if !event.inside_facility {
            continue;
        }

        for (snake_entity, mut state, attachment) in snakes.iter_mut() {
            if attachment.employee_stable_id != event.employee_stable_id {
                continue;
            }

            set_state(
                snake_entity,
                &mut state,
                TulipSnakeState::IndoorAttached,
                &mut state_events,
            );
        }
    }
}

fn tulip_snake_open_doors(
    mut door_events: EventWriter<TulipSnakeDoorOpenRequestEvent>,
    snakes: Query<(Entity, &SimPosition, &TulipSnakeState), With<TulipSnake>>,
    doors: Query<(Entity, &SimPosition, &TulipSnakeDoor)>,
) {
    let door_range_squared = TULIP_SNAKE_LATCH_RANGE * TULIP_SNAKE_LATCH_RANGE;

    for (snake_entity, snake_position, state) in snakes.iter() {
        if *state != TulipSnakeState::IndoorAttached {
            continue;
        }

        let mut selected: Option<(Entity, u64, I32F32)> = None;

        for (door_entity, door_position, door) in doors.iter() {
            if door.open {
                continue;
            }

            let distance = distance_squared(*snake_position, *door_position);
            if distance > door_range_squared {
                continue;
            }

            selected = match selected {
                Some((selected_entity, selected_id, selected_distance))
                    if selected_distance < distance
                        || (selected_distance == distance && selected_id <= door.stable_id) =>
                {
                    Some((selected_entity, selected_id, selected_distance))
                }
                _ => Some((door_entity, door.stable_id, distance)),
            };
        }

        if let Some((door, door_stable_id, _)) = selected {
            door_events.send(TulipSnakeDoorOpenRequestEvent {
                snake: snake_entity,
                door,
                door_stable_id,
                speed: TULIP_SNAKE_DOOR_OPEN_SPEED,
            });
        }
    }
}

fn tulip_snake_weapon_wave_kills_attached(
    mut commands: Commands,
    mut wave_events: EventReader<TulipSnakeWeaponWaveEvent>,
    mut killed_events: EventWriter<EntityKilledEvent>,
    snakes: Query<(Entity, &TulipSnakeAttachment), With<TulipSnake>>,
) {
    for event in wave_events.read() {
        for (snake_entity, attachment) in snakes.iter() {
            if attachment.employee_stable_id != event.employee_stable_id {
                continue;
            }

            killed_events.send(EntityKilledEvent {
                entity: snake_entity,
                killer: event.employee,
                exp_reward: I32F32::lit("0"),
                difficulty_tier: 0,
            });
            commands.entity(snake_entity).despawn();
        }
    }
}

fn tulip_snake_main_room_fan_kills(
    mut commands: Commands,
    mut fan_events: EventReader<TulipSnakeMainRoomFanCollisionEvent>,
    mut killed_events: EventWriter<EntityKilledEvent>,
    snakes: Query<(), With<TulipSnake>>,
) {
    for event in fan_events.read() {
        if snakes.get(event.snake).is_err() {
            continue;
        }

        killed_events.send(EntityKilledEvent {
            entity: event.snake,
            killer: event.employee,
            exp_reward: I32F32::lit("0"),
            difficulty_tier: 0,
        });
        killed_events.send(EntityKilledEvent {
            entity: event.employee,
            killer: event.snake,
            exp_reward: I32F32::lit("0"),
            difficulty_tier: 0,
        });
        commands.entity(event.snake).despawn();
    }
}

fn tulip_snake_hazard_kills(
    mut commands: Commands,
    mut hazard_events: EventReader<TulipSnakeHazardHitEvent>,
    mut killed_events: EventWriter<EntityKilledEvent>,
    snakes: Query<(), With<TulipSnake>>,
) {
    for event in hazard_events.read() {
        if snakes.get(event.snake).is_err() {
            continue;
        }

        killed_events.send(EntityKilledEvent {
            entity: event.snake,
            killer: event.snake,
            exp_reward: I32F32::lit("0"),
            difficulty_tier: 0,
        });
        commands.entity(event.snake).despawn();
    }
}

fn tulip_snake_apply_damage(
    mut damage_events: EventReader<IncomingDamageEvent>,
    mut killed_events: EventWriter<EntityKilledEvent>,
    mut snakes: Query<(Entity, &mut Health), With<TulipSnake>>,
) {
    for event in damage_events.read() {
        for (snake_entity, mut health) in snakes.iter_mut() {
            if event.target != snake_entity {
                continue;
            }

            health.current -= event.raw_amount;

            if health.current <= I32F32::lit("0") {
                health.current = I32F32::lit("0");
                killed_events.send(EntityKilledEvent {
                    entity: snake_entity,
                    killer: event.source,
                    exp_reward: I32F32::lit("0"),
                    difficulty_tier: 0,
                });
            }
        }
    }
}

fn tulip_snake_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    snakes: Query<
        (
            &SimPosition,
            &Health,
            &UnitStats,
            &TulipSnakeState,
            &TulipSnakeTimers,
            &TulipSnakeAttachment,
            &TulipSnakeMovement,
        ),
        With<TulipSnake>,
    >,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(TULIP_SNAKE_SOURCE_REVISION as u64);
    checksum.accumulate(TULIP_SNAKE_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(TULIP_SNAKE_POWER_LEVEL.to_bits() as u64);
    checksum.accumulate(TULIP_SNAKE_MAX_SPAWNED as u64);
    checksum.accumulate(TULIP_SNAKE_SHOVEL_HP.to_bits() as u64);
    checksum.accumulate(TULIP_SNAKE_DOOR_OPEN_SPEED.to_bits() as u64);
    checksum.accumulate(TULIP_SNAKE_CAN_SEE_THROUGH_FOG as u64);
    checksum.accumulate(TULIP_SNAKE_LEAVE_TIME_MINUTES_AFTER_MIDNIGHT as u64);
    checksum.accumulate(TULIP_SNAKE_ONE_ATTACHED_WEIGHT_LB.to_bits() as u64);
    checksum.accumulate(TULIP_SNAKE_FIVE_ATTACHED_WEIGHT_LB.to_bits() as u64);
    checksum.accumulate(TULIP_SNAKE_MAX_ATTACHED_PER_EMPLOYEE as u64);
    checksum.accumulate(TULIP_SNAKE_MULTI_ATTACH_FLIGHT_SECONDS.to_bits() as u64);

    accumulate_str(&mut checksum, 0x1000, TULIP_SNAKE_ID);
    accumulate_str(&mut checksum, 0x1001, TULIP_SNAKE_NAME);
    accumulate_str(&mut checksum, 0x1002, TULIP_SNAKE_TYPE);
    accumulate_str(&mut checksum, 0x1003, TULIP_SNAKE_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, TULIP_SNAKE_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1005, TULIP_SNAKE_EXTRACTED_AT);
    accumulate_str(&mut checksum, 0x1006, TULIP_SNAKE_DWELLS);
    accumulate_str(&mut checksum, 0x1007, TULIP_SNAKE_SCIENTIFIC_NAME);
    accumulate_str(&mut checksum, 0x1008, TULIP_SNAKE_SHOCK_RESPONSE);
    accumulate_str(&mut checksum, 0x1009, TULIP_SNAKE_RADAR_PIP_SIZE);
    accumulate_str(&mut checksum, 0x100a, TULIP_SNAKE_STUN_MULTIPLIER);
    accumulate_str(&mut checksum, 0x100b, TULIP_SNAKE_LEAVE_TIME);
    accumulate_str(&mut checksum, 0x100c, TULIP_SNAKE_IMAGE);

    for dependency in TULIP_SNAKE_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for behavior in TULIP_SNAKE_FRONTMATTER_BEHAVIOR {
        accumulate_str(&mut checksum, 0x3000, behavior);
    }

    for occurrence in TULIP_SNAKE_OCCURRENCE {
        accumulate_str(&mut checksum, 0x3500, occurrence.moon);
        checksum.accumulate(occurrence.base_spawn_chance.to_bits() as u64);
    }

    for rule in TULIP_SNAKE_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x4000, rule.condition);
        accumulate_str(&mut checksum, 0x4001, rule.outcome);
    }

    for (position, health, stats, state, timers, attachment, movement) in snakes.iter() {
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
        checksum.accumulate(timers.pounce_ticks as u64);
        checksum.accumulate(timers.lift_ticks as u64);
        checksum.accumulate(timers.chirp_ticks as u64);
        checksum.accumulate(attachment.employee_stable_id);
        checksum.accumulate(attachment.lift_attempts as u64);
        checksum.accumulate(attachment.tire_after_attempts as u64);
        checksum.accumulate(attachment.carried_weight_lb.to_bits() as u64);
        checksum.accumulate(movement.roaming_anchor.x.to_bits() as u64);
        checksum.accumulate(movement.roaming_anchor.y.to_bits() as u64);
        checksum.accumulate(movement.target_employee_stable_id);
    }
}

fn release_attachment(
    snake_entity: Entity,
    state: &mut TulipSnakeState,
    attachment: &mut TulipSnakeAttachment,
    movement: &mut TulipSnakeMovement,
    release_events: &mut EventWriter<TulipSnakeReleasedEvent>,
    state_events: &mut EventWriter<TulipSnakeStateChangedEvent>,
) {
    if let Some(employee) = attachment.employee {
        release_events.send(TulipSnakeReleasedEvent {
            snake: snake_entity,
            employee,
            employee_stable_id: attachment.employee_stable_id,
        });
    }

    attachment.employee = None;
    attachment.employee_stable_id = 0;
    attachment.lift_attempts = 0;
    attachment.carried_weight_lb = I32F32::lit("0");
    movement.target_employee = None;
    movement.target_employee_stable_id = 0;
    set_state(
        snake_entity,
        state,
        TulipSnakeState::Roaming,
        state_events,
    );
}

fn carried_weight_for_count(attached_count: u8) -> I32F32 {
    if attached_count == 0 {
        I32F32::lit("0")
    } else if attached_count >= TULIP_SNAKE_MAX_ATTACHED_PER_EMPLOYEE {
        TULIP_SNAKE_FIVE_ATTACHED_WEIGHT_LB
    } else {
        TULIP_SNAKE_ONE_ATTACHED_WEIGHT_LB * I32F32::from_num(attached_count)
    }
}

fn is_attached_state(state: TulipSnakeState) -> bool {
    matches!(
        state,
        TulipSnakeState::AttachedViewing
            | TulipSnakeState::AttachedLifting
            | TulipSnakeState::FallingBack
            | TulipSnakeState::IndoorAttached
    )
}

fn set_state(
    entity: Entity,
    state: &mut TulipSnakeState,
    to: TulipSnakeState,
    events: &mut EventWriter<TulipSnakeStateChangedEvent>,
) {
    if *state == to {
        return;
    }

    let from = *state;
    *state = to;
    events.send(TulipSnakeStateChangedEvent {
        snake: entity,
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
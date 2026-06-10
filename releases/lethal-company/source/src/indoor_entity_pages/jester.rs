// Sources: vault/indoor_entity_pages/jester.md
use bevy::prelude::*;
use fixed::types::I32F32;
use rand_core::RngCore;

use crate::sim::{
    tick_rng, DamageType, GameSeed, Health, IncomingDamageEvent, SimChecksumState, SimHz,
    SimPosition, SimTick, UnitStats,
};

pub const JESTER_ID: &str = "jester";
pub const JESTER_NAME: &str = "Jester";
pub const JESTER_TYPE: &str = "indoor_entity_pages";
pub const JESTER_SUBTYPE: &str = "entity";
pub const JESTER_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Jester";
pub const JESTER_SOURCE_REVISION: u32 = 21327;
pub const JESTER_EXTRACTED_AT: &str = "2026-06-07";
pub const JESTER_CONFIDENCE_BASIS_POINTS: u16 = 98;

pub const JESTER_DWELLS: &str = "Inside";
pub const JESTER_DANGER: &str = "High";
pub const JESTER_PIP_SIZE: &str = "Medium";
pub const JESTER_HP: &str = "Immune";
pub const JESTER_SHOCK_RESPONSE: &str = "Susceptible";
pub const JESTER_POWER_LEVEL: I32F32 = I32F32::lit("3");
pub const JESTER_MAX_SPAWNED: usize = 1;
pub const JESTER_DOOR_OPEN_SPEED: I32F32 = I32F32::lit("0.5");
pub const JESTER_STUN_MULTIPLIER: I32F32 = I32F32::lit("0.6");
pub const JESTER_DOOR_SPEED_MULTIPLIER: I32F32 = I32F32::lit("0.5");
pub const JESTER_CONTACT_DAMAGE: &str = "Instant Kill";

pub const JESTER_FOLLOW_MIN_SECONDS: I32F32 = I32F32::lit("13");
pub const JESTER_FOLLOW_MAX_SECONDS: I32F32 = I32F32::lit("18");
pub const JESTER_WINDING_SECONDS: I32F32 = I32F32::lit("42");
pub const JESTER_MAX_WINDING_STALL_SECONDS: I32F32 = I32F32::lit("20");
pub const JESTER_BODY_HOLD_SECONDS: I32F32 = I32F32::lit("3");
pub const JESTER_ROAM_SPEED: I32F32 = I32F32::lit("1");
pub const JESTER_FOLLOW_SPEED: I32F32 = I32F32::lit("1.5");
pub const JESTER_CHASE_INITIAL_SPEED: I32F32 = I32F32::lit("2");
pub const JESTER_CHASE_ACCEL_PER_SECOND: I32F32 = I32F32::lit("0.5");
pub const JESTER_WATCH_RANGE: I32F32 = I32F32::lit("28");
pub const JESTER_ATTACK_RANGE: I32F32 = I32F32::lit("1");
pub const JESTER_IMMUNE_HEALTH: I32F32 = I32F32::lit("0");
pub const JESTER_INSTANT_KILL_DAMAGE: I32F32 = I32F32::lit("1000000");

pub const JESTER_RANDOM_ROAM_SALT: u64 = 0x6a65737465725f01;
pub const JESTER_FOLLOW_TIMER_SALT: u64 = 0x6a65737465725f02;

pub const JESTER_DEPENDS_ON: [&str; 0] = [];
pub const JESTER_FRONTMATTER_BEHAVIOR: [&str; 3] = ["Roaming", "Winding", "Chasing"];

pub const JESTER_BEHAVIORAL_MECHANICS: [JesterBehaviorRule; 9] = [
    JesterBehaviorRule {
        condition: "the Jester is in Roaming",
        outcome: "it moves randomly until it spots an employee, then follows that employee for about 13 to 18 seconds",
    },
    JesterBehaviorRule {
        condition: "the follow timer ends",
        outcome: "it enters Winding, cannot be interrupted, and spends about 42 seconds winding up",
    },
    JesterBehaviorRule {
        condition: "a player can perceive a winding Jester",
        outcome: "they should head for the nearest facility entrance immediately",
    },
    JesterBehaviorRule {
        condition: "winding completes",
        outcome: "the Jester enters Chasing, screams across the map, always knows the nearest player's position, and its speed increases continuously until it can outrun sprinting employees",
    },
    JesterBehaviorRule {
        condition: "a Chasing Jester reaches an employee",
        outcome: "it instantly kills them and holds the body for a few seconds before resuming the chase",
    },
    JesterBehaviorRule {
        condition: "there are no living employees inside the facility",
        outcome: "the Jester returns its skull to the box and re-enters Roaming",
    },
    JesterBehaviorRule {
        condition: "the Jester is subjected to a stun effect while winding",
        outcome: "the wind-up can be stalled for up to 20 seconds",
    },
    JesterBehaviorRule {
        condition: "the Jester is stunned before winding starts",
        outcome: "it resumes by immediately starting to wind once the stun ends",
    },
    JesterBehaviorRule {
        condition: "the Jester is hit by a stun effect or a flash effect while chasing",
        outcome: "its chase speed resets",
    },
];

pub struct JesterPlugin;

impl Plugin for JesterPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnJesterEvent>()
            .add_event::<JesterStateChangedEvent>()
            .add_event::<JesterWindingPerceivedEvent>()
            .add_event::<JesterMapScreamEvent>()
            .add_event::<JesterDoorAttemptEvent>()
            .add_event::<JesterDoorAttemptResolvedEvent>()
            .add_event::<JesterStunAppliedEvent>()
            .add_event::<JesterStunAdjustedEvent>()
            .add_event::<JesterFlashAppliedEvent>()
            .add_event::<JesterChaseSpeedResetEvent>()
            .add_event::<JesterContactKillEvent>()
            .add_event::<JesterIgnoredDamageEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_jester,
                    jester_roam_randomly_until_employee_spotted,
                    jester_follow_target_timer,
                    jester_start_winding_after_prestun,
                    jester_progress_winding,
                    jester_emit_winding_perceived,
                    jester_chase_nearest_employee,
                    jester_resume_after_body_hold,
                    jester_return_to_roaming_when_facility_empty,
                    jester_door_attempt_speed,
                    jester_apply_stun,
                    jester_apply_flash_reset,
                    jester_contact_instant_kill,
                    jester_ignore_damage,
                    jester_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JesterBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Jester;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct JesterEmployeeSensor {
    pub stable_id: u64,
    pub inside_facility: bool,
    pub alive: bool,
    pub can_be_spotted: bool,
    pub can_perceive_winding_jester: bool,
    pub touching_jester: bool,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct JesterTarget {
    pub has_target: bool,
    pub stable_id: u64,
    pub last_known_position: SimPosition,
}

impl Default for JesterTarget {
    fn default() -> Self {
        Self {
            has_target: false,
            stable_id: 0,
            last_known_position: SimPosition {
                x: I32F32::lit("0"),
                y: I32F32::lit("0"),
            },
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct JesterTiming {
    pub follow_ticks_remaining: u32,
    pub winding_ticks_remaining: u32,
    pub winding_stall_ticks_remaining: u32,
    pub body_hold_ticks_remaining: u32,
    pub prestun_ticks_remaining: u32,
}

impl Default for JesterTiming {
    fn default() -> Self {
        Self {
            follow_ticks_remaining: 0,
            winding_ticks_remaining: 0,
            winding_stall_ticks_remaining: 0,
            body_hold_ticks_remaining: 0,
            prestun_ticks_remaining: 0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct JesterChase {
    pub speed: I32F32,
    pub screamed: bool,
    pub holding_body: bool,
    pub held_employee_stable_id: u64,
}

impl Default for JesterChase {
    fn default() -> Self {
        Self {
            speed: JESTER_CHASE_INITIAL_SPEED,
            screamed: false,
            holding_body: false,
            held_employee_stable_id: 0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum JesterState {
    #[default]
    Roaming,
    Winding,
    Chasing,
}

#[derive(Bundle)]
pub struct JesterBundle {
    pub name: Name,
    pub jester: Jester,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub state: JesterState,
    pub target: JesterTarget,
    pub timing: JesterTiming,
    pub chase: JesterChase,
}

impl JesterBundle {
    pub fn new(event: SpawnJesterEvent) -> Self {
        Self {
            name: Name::new(JESTER_NAME),
            jester: Jester,
            position: event.position,
            health: Health::full(JESTER_IMMUNE_HEALTH),
            stats: UnitStats {
                move_speed: JESTER_ROAM_SPEED,
                attack_range: JESTER_ATTACK_RANGE,
                attack_damage: JESTER_INSTANT_KILL_DAMAGE,
                attack_speed: I32F32::lit("1"),
                watch_range: JESTER_WATCH_RANGE,
            },
            state: JesterState::Roaming,
            target: JesterTarget {
                has_target: false,
                stable_id: 0,
                last_known_position: event.position,
            },
            timing: JesterTiming::default(),
            chase: JesterChase::default(),
        }
    }
}

#[derive(Event, Clone, Copy, Debug)]
pub struct SpawnJesterEvent {
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct JesterStateChangedEvent {
    pub jester: Entity,
    pub from: JesterState,
    pub to: JesterState,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct JesterWindingPerceivedEvent {
    pub jester: Entity,
    pub employee: Entity,
    pub employee_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct JesterMapScreamEvent {
    pub jester: Entity,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct JesterDoorAttemptEvent {
    pub jester: Entity,
    pub door: Entity,
    pub base_open_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct JesterDoorAttemptResolvedEvent {
    pub jester: Entity,
    pub door: Entity,
    pub adjusted_open_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct JesterStunAppliedEvent {
    pub jester: Entity,
    pub base_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct JesterStunAdjustedEvent {
    pub jester: Entity,
    pub base_ticks: u32,
    pub adjusted_ticks: u32,
    pub stalled_winding_ticks: u32,
    pub chase_speed_reset: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct JesterFlashAppliedEvent {
    pub jester: Entity,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct JesterChaseSpeedResetEvent {
    pub jester: Entity,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct JesterContactKillEvent {
    pub jester: Entity,
    pub employee: Entity,
    pub employee_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct JesterIgnoredDamageEvent {
    pub jester: Entity,
    pub source: Entity,
}

fn spawn_jester(
    mut commands: Commands,
    mut events: EventReader<SpawnJesterEvent>,
    jesters: Query<(), With<Jester>>,
) {
    let mut spawned_count = jesters.iter().count();

    for event in events.read() {
        if spawned_count >= JESTER_MAX_SPAWNED {
            break;
        }

        commands.spawn(JesterBundle::new(*event));
        spawned_count += 1;
    }
}

fn jester_roam_randomly_until_employee_spotted(
    game_seed: Res<GameSeed>,
    sim_tick: Res<SimTick>,
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<JesterStateChangedEvent>,
    mut jesters: Query<
        (
            Entity,
            &mut SimPosition,
            &mut JesterState,
            &mut JesterTarget,
            &mut JesterTiming,
            &mut UnitStats,
        ),
        With<Jester>,
    >,
    employees: Query<(&SimPosition, &JesterEmployeeSensor), Without<Jester>>,
) {
    for (jester_entity, mut position, mut state, mut target, mut timing, mut stats) in
        jesters.iter_mut()
    {
        if *state != JesterState::Roaming {
            continue;
        }

        if let Some((employee_position, employee_sensor)) = spotted_employee(&employees) {
            target.has_target = true;
            target.stable_id = employee_sensor.stable_id;
            target.last_known_position = employee_position;
            timing.follow_ticks_remaining =
                random_follow_ticks(game_seed.0, sim_tick.0, jester_entity.index() as u64, sim_hz.0);
            stats.move_speed = JESTER_FOLLOW_SPEED;
            set_jester_state(
                jester_entity,
                &mut state,
                JesterState::Roaming,
                JesterState::Winding,
                false,
                &mut state_events,
            );
            *state = JesterState::Roaming;
            set_jester_state(
                jester_entity,
                &mut state,
                JesterState::Roaming,
                JesterState::Roaming,
                false,
                &mut state_events,
            );
            continue;
        }

        let step = stats.move_speed / sim_hz.0;
        let direction = random_axis_direction(game_seed.0, sim_tick.0, jester_entity.index() as u64);
        position.x += direction.x * step;
        position.y += direction.y * step;
    }
}

fn jester_follow_target_timer(
    mut state_events: EventWriter<JesterStateChangedEvent>,
    mut jesters: Query<
        (
            Entity,
            &mut SimPosition,
            &mut JesterState,
            &mut JesterTarget,
            &mut JesterTiming,
            &mut UnitStats,
        ),
        With<Jester>,
    >,
    employees: Query<(&SimPosition, &JesterEmployeeSensor), Without<Jester>>,
) {
    for (jester_entity, mut position, mut state, mut target, mut timing, mut stats) in
        jesters.iter_mut()
    {
        if *state != JesterState::Roaming || !target.has_target {
            continue;
        }

        if let Some(employee_position) = employee_position_by_stable_id(target.stable_id, &employees)
        {
            target.last_known_position = employee_position;
            move_axis_toward(&mut position, employee_position, stats.move_speed);
        }

        if timing.follow_ticks_remaining > 0 {
            timing.follow_ticks_remaining -= 1;
            continue;
        }

        timing.winding_ticks_remaining = fixed_seconds_to_ticks(JESTER_WINDING_SECONDS, I32F32::lit("1"));
        stats.move_speed = I32F32::lit("0");
        set_jester_state(
            jester_entity,
            &mut state,
            JesterState::Roaming,
            JesterState::Winding,
            true,
            &mut state_events,
        );
    }
}

fn jester_start_winding_after_prestun(
    mut state_events: EventWriter<JesterStateChangedEvent>,
    mut jesters: Query<(Entity, &mut JesterState, &mut JesterTiming, &mut UnitStats), With<Jester>>,
) {
    for (jester_entity, mut state, mut timing, mut stats) in jesters.iter_mut() {
        if *state != JesterState::Roaming || timing.prestun_ticks_remaining == 0 {
            continue;
        }

        timing.prestun_ticks_remaining -= 1;
        if timing.prestun_ticks_remaining > 0 {
            continue;
        }

        timing.winding_ticks_remaining = 1;
        stats.move_speed = I32F32::lit("0");
        set_jester_state(
            jester_entity,
            &mut state,
            JesterState::Roaming,
            JesterState::Winding,
            true,
            &mut state_events,
        );
    }
}

fn jester_progress_winding(
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<JesterStateChangedEvent>,
    mut scream_events: EventWriter<JesterMapScreamEvent>,
    mut jesters: Query<
        (
            Entity,
            &mut JesterState,
            &mut JesterTiming,
            &mut JesterChase,
            &mut UnitStats,
        ),
        With<Jester>,
    >,
) {
    for (jester_entity, mut state, mut timing, mut chase, mut stats) in jesters.iter_mut() {
        if *state != JesterState::Winding {
            continue;
        }

        if timing.winding_stall_ticks_remaining > 0 {
            timing.winding_stall_ticks_remaining -= 1;
            continue;
        }

        if timing.winding_ticks_remaining == 0 {
            chase.speed = JESTER_CHASE_INITIAL_SPEED;
            chase.screamed = true;
            chase.holding_body = false;
            chase.held_employee_stable_id = 0;
            stats.move_speed = chase.speed;
            scream_events.send(JesterMapScreamEvent {
                jester: jester_entity,
            });
            set_jester_state(
                jester_entity,
                &mut state,
                JesterState::Winding,
                JesterState::Chasing,
                true,
                &mut state_events,
            );
            continue;
        }

        let full_wind_ticks = fixed_seconds_to_ticks(JESTER_WINDING_SECONDS, sim_hz.0);
        if timing.winding_ticks_remaining > full_wind_ticks {
            timing.winding_ticks_remaining = full_wind_ticks;
        }
        timing.winding_ticks_remaining -= 1;
    }
}

fn jester_emit_winding_perceived(
    mut perceived_events: EventWriter<JesterWindingPerceivedEvent>,
    jesters: Query<(Entity, &JesterState), With<Jester>>,
    employees: Query<(Entity, &JesterEmployeeSensor)>,
) {
    for (jester_entity, state) in jesters.iter() {
        if *state != JesterState::Winding {
            continue;
        }

        for (employee_entity, sensor) in employees.iter() {
            if sensor.inside_facility && sensor.alive && sensor.can_perceive_winding_jester {
                perceived_events.send(JesterWindingPerceivedEvent {
                    jester: jester_entity,
                    employee: employee_entity,
                    employee_stable_id: sensor.stable_id,
                });
            }
        }
    }
}

fn jester_chase_nearest_employee(
    sim_hz: Res<SimHz>,
    mut jesters: Query<
        (
            &mut SimPosition,
            &JesterState,
            &mut JesterTarget,
            &mut JesterChase,
            &mut UnitStats,
        ),
        With<Jester>,
    >,
    employees: Query<(&SimPosition, &JesterEmployeeSensor), Without<Jester>>,
) {
    for (mut position, state, mut target, mut chase, mut stats) in jesters.iter_mut() {
        if *state != JesterState::Chasing || chase.holding_body {
            continue;
        }

        chase.speed += JESTER_CHASE_ACCEL_PER_SECOND / sim_hz.0;
        stats.move_speed = chase.speed;

        let Some((employee_position, employee_sensor)) = nearest_living_employee(*position, &employees)
        else {
            continue;
        };

        target.has_target = true;
        target.stable_id = employee_sensor.stable_id;
        target.last_known_position = employee_position;
        move_axis_toward(&mut position, employee_position, chase.speed / sim_hz.0);
    }
}

fn jester_resume_after_body_hold(mut jesters: Query<&mut JesterChase, With<Jester>>) {
    for mut chase in jesters.iter_mut() {
        if !chase.holding_body {
            continue;
        }

        if chase.held_employee_stable_id == 0 {
            chase.holding_body = false;
            continue;
        }
    }
}

fn jester_return_to_roaming_when_facility_empty(
    mut state_events: EventWriter<JesterStateChangedEvent>,
    mut jesters: Query<
        (
            Entity,
            &mut JesterState,
            &mut JesterTarget,
            &mut JesterTiming,
            &mut JesterChase,
            &mut UnitStats,
        ),
        With<Jester>,
    >,
    employees: Query<&JesterEmployeeSensor>,
) {
    if employees
        .iter()
        .any(|sensor| sensor.inside_facility && sensor.alive)
    {
        return;
    }

    for (jester_entity, mut state, mut target, mut timing, mut chase, mut stats) in
        jesters.iter_mut()
    {
        if *state == JesterState::Roaming {
            continue;
        }

        target.has_target = false;
        target.stable_id = 0;
        timing.follow_ticks_remaining = 0;
        timing.winding_ticks_remaining = 0;
        timing.winding_stall_ticks_remaining = 0;
        timing.body_hold_ticks_remaining = 0;
        chase.speed = JESTER_CHASE_INITIAL_SPEED;
        chase.screamed = false;
        chase.holding_body = false;
        chase.held_employee_stable_id = 0;
        stats.move_speed = JESTER_ROAM_SPEED;

        let previous = *state;
        set_jester_state(
            jester_entity,
            &mut state,
            previous,
            JesterState::Roaming,
            true,
            &mut state_events,
        );
    }
}

fn jester_door_attempt_speed(
    mut events: EventReader<JesterDoorAttemptEvent>,
    mut resolved_events: EventWriter<JesterDoorAttemptResolvedEvent>,
    jesters: Query<(), With<Jester>>,
) {
    for event in events.read() {
        if jesters.get(event.jester).is_err() {
            continue;
        }

        resolved_events.send(JesterDoorAttemptResolvedEvent {
            jester: event.jester,
            door: event.door,
            adjusted_open_ticks: fixed_ticks_scaled(event.base_open_ticks, JESTER_DOOR_SPEED_MULTIPLIER),
        });
    }
}

fn jester_apply_stun(
    sim_hz: Res<SimHz>,
    mut events: EventReader<JesterStunAppliedEvent>,
    mut adjusted_events: EventWriter<JesterStunAdjustedEvent>,
    mut reset_events: EventWriter<JesterChaseSpeedResetEvent>,
    mut jesters: Query<(&JesterState, &mut JesterTiming, &mut JesterChase, &mut UnitStats), With<Jester>>,
) {
    for event in events.read() {
        let Ok((state, mut timing, mut chase, mut stats)) = jesters.get_mut(event.jester) else {
            continue;
        };

        let adjusted_ticks = fixed_ticks_scaled(event.base_ticks, JESTER_STUN_MULTIPLIER);
        let mut stalled_winding_ticks = 0;
        let mut chase_speed_reset = false;

        match *state {
            JesterState::Roaming => {
                timing.prestun_ticks_remaining = adjusted_ticks;
            }
            JesterState::Winding => {
                let max_stall_ticks = fixed_seconds_to_ticks(JESTER_MAX_WINDING_STALL_SECONDS, sim_hz.0);
                let remaining_capacity = max_stall_ticks.saturating_sub(timing.winding_stall_ticks_remaining);
                stalled_winding_ticks = adjusted_ticks.min(remaining_capacity);
                timing.winding_stall_ticks_remaining += stalled_winding_ticks;
            }
            JesterState::Chasing => {
                chase.speed = JESTER_CHASE_INITIAL_SPEED;
                stats.move_speed = chase.speed;
                chase_speed_reset = true;
                reset_events.send(JesterChaseSpeedResetEvent {
                    jester: event.jester,
                });
            }
        }

        adjusted_events.send(JesterStunAdjustedEvent {
            jester: event.jester,
            base_ticks: event.base_ticks,
            adjusted_ticks,
            stalled_winding_ticks,
            chase_speed_reset,
        });
    }
}

fn jester_apply_flash_reset(
    mut events: EventReader<JesterFlashAppliedEvent>,
    mut reset_events: EventWriter<JesterChaseSpeedResetEvent>,
    mut jesters: Query<(&JesterState, &mut JesterChase, &mut UnitStats), With<Jester>>,
) {
    for event in events.read() {
        let Ok((state, mut chase, mut stats)) = jesters.get_mut(event.jester) else {
            continue;
        };

        if *state != JesterState::Chasing {
            continue;
        }

        chase.speed = JESTER_CHASE_INITIAL_SPEED;
        stats.move_speed = chase.speed;
        reset_events.send(JesterChaseSpeedResetEvent {
            jester: event.jester,
        });
    }
}

fn jester_contact_instant_kill(
    sim_hz: Res<SimHz>,
    mut damage_events: EventWriter<IncomingDamageEvent>,
    mut contact_events: EventWriter<JesterContactKillEvent>,
    mut jesters: Query<(Entity, &JesterState, &mut JesterChase, &mut JesterTiming), With<Jester>>,
    employees: Query<(Entity, &JesterEmployeeSensor)>,
) {
    for (jester_entity, state, mut chase, mut timing) in jesters.iter_mut() {
        if *state != JesterState::Chasing || chase.holding_body {
            continue;
        }

        for (employee_entity, sensor) in employees.iter() {
            if !sensor.inside_facility || !sensor.alive || !sensor.touching_jester {
                continue;
            }

            contact_events.send(JesterContactKillEvent {
                jester: jester_entity,
                employee: employee_entity,
                employee_stable_id: sensor.stable_id,
            });
            damage_events.send(IncomingDamageEvent {
                target: employee_entity,
                raw_amount: JESTER_INSTANT_KILL_DAMAGE,
                damage_type: DamageType::Standard,
                source: jester_entity,
            });

            chase.holding_body = true;
            chase.held_employee_stable_id = sensor.stable_id;
            timing.body_hold_ticks_remaining = fixed_seconds_to_ticks(JESTER_BODY_HOLD_SECONDS, sim_hz.0);
            break;
        }
    }
}

fn jester_ignore_damage(
    mut damage_events: EventReader<IncomingDamageEvent>,
    mut ignored_events: EventWriter<JesterIgnoredDamageEvent>,
    jesters: Query<(), With<Jester>>,
) {
    for event in damage_events.read() {
        if jesters.get(event.target).is_err() {
            continue;
        }

        ignored_events.send(JesterIgnoredDamageEvent {
            jester: event.target,
            source: event.source,
        });
    }
}

fn jester_checksum(
    mut checksum: ResMut<SimChecksumState>,
    jesters: Query<
        (
            &SimPosition,
            &Health,
            &UnitStats,
            &JesterState,
            &JesterTarget,
            &JesterTiming,
            &JesterChase,
        ),
        With<Jester>,
    >,
) {
    for (position, health, stats, state, target, timing, chase) in jesters.iter() {
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);
        checksum.accumulate(jester_state_bits(*state));
        checksum.accumulate(target.has_target as u64);
        checksum.accumulate(target.stable_id);
        checksum.accumulate(target.last_known_position.x.to_bits() as u64);
        checksum.accumulate(target.last_known_position.y.to_bits() as u64);
        checksum.accumulate(timing.follow_ticks_remaining as u64);
        checksum.accumulate(timing.winding_ticks_remaining as u64);
        checksum.accumulate(timing.winding_stall_ticks_remaining as u64);
        checksum.accumulate(timing.body_hold_ticks_remaining as u64);
        checksum.accumulate(timing.prestun_ticks_remaining as u64);
        checksum.accumulate(chase.speed.to_bits() as u64);
        checksum.accumulate(chase.screamed as u64);
        checksum.accumulate(chase.holding_body as u64);
        checksum.accumulate(chase.held_employee_stable_id);
    }
}

fn spotted_employee(
    employees: &Query<(&SimPosition, &JesterEmployeeSensor), Without<Jester>>,
) -> Option<(SimPosition, JesterEmployeeSensor)> {
    let mut best: Option<(SimPosition, JesterEmployeeSensor)> = None;

    for (position, sensor) in employees.iter() {
        if !sensor.inside_facility || !sensor.alive || !sensor.can_be_spotted {
            continue;
        }

        if let Some((_best_position, best_sensor)) = best {
            if sensor.stable_id >= best_sensor.stable_id {
                continue;
            }
        }

        best = Some((*position, *sensor));
    }

    best
}

fn nearest_living_employee(
    jester_position: SimPosition,
    employees: &Query<(&SimPosition, &JesterEmployeeSensor), Without<Jester>>,
) -> Option<(SimPosition, JesterEmployeeSensor)> {
    let mut best: Option<(SimPosition, JesterEmployeeSensor, I32F32)> = None;

    for (position, sensor) in employees.iter() {
        if !sensor.inside_facility || !sensor.alive {
            continue;
        }

        let distance = fixed_abs(position.x - jester_position.x) + fixed_abs(position.y - jester_position.y);

        if let Some((_best_position, best_sensor, best_distance)) = best {
            if distance > best_distance {
                continue;
            }

            if distance == best_distance && sensor.stable_id >= best_sensor.stable_id {
                continue;
            }
        }

        best = Some((*position, *sensor, distance));
    }

    best.map(|(position, sensor, _distance)| (position, sensor))
}

fn employee_position_by_stable_id(
    stable_id: u64,
    employees: &Query<(&SimPosition, &JesterEmployeeSensor), Without<Jester>>,
) -> Option<SimPosition> {
    for (position, sensor) in employees.iter() {
        if sensor.stable_id == stable_id && sensor.inside_facility && sensor.alive {
            return Some(*position);
        }
    }

    None
}

fn set_jester_state(
    jester: Entity,
    state: &mut JesterState,
    from: JesterState,
    to: JesterState,
    mutate: bool,
    events: &mut EventWriter<JesterStateChangedEvent>,
) {
    if from == to {
        return;
    }

    if mutate {
        *state = to;
    }

    events.send(JesterStateChangedEvent { jester, from, to });
}

fn random_follow_ticks(game_seed: u64, tick: u64, stable_salt: u64, sim_hz: I32F32) -> u32 {
    let mut rng = tick_rng(game_seed, tick, JESTER_FOLLOW_TIMER_SALT ^ stable_salt);
    let range_seconds = (JESTER_FOLLOW_MAX_SECONDS - JESTER_FOLLOW_MIN_SECONDS) + I32F32::lit("1");
    let range_whole: u32 = range_seconds.to_num();
    let offset = rng.next_u32() % range_whole;
    fixed_seconds_to_ticks(JESTER_FOLLOW_MIN_SECONDS + I32F32::from_num(offset), sim_hz)
}

fn random_axis_direction(game_seed: u64, tick: u64, stable_salt: u64) -> SimPosition {
    let mut rng = tick_rng(game_seed, tick, JESTER_RANDOM_ROAM_SALT ^ stable_salt);
    match rng.next_u32() % 4 {
        0 => SimPosition {
            x: I32F32::lit("1"),
            y: I32F32::lit("0"),
        },
        1 => SimPosition {
            x: I32F32::lit("-1"),
            y: I32F32::lit("0"),
        },
        2 => SimPosition {
            x: I32F32::lit("0"),
            y: I32F32::lit("1"),
        },
        _ => SimPosition {
            x: I32F32::lit("0"),
            y: I32F32::lit("-1"),
        },
    }
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

fn jester_state_bits(state: JesterState) -> u64 {
    match state {
        JesterState::Roaming => 0,
        JesterState::Winding => 1,
        JesterState::Chasing => 2,
    }
}
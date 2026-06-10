// Sources: vault/indoor_entity_pages/bracken.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{
    DamageType, Health, IncomingDamageEvent, SimChecksumState, SimHz, SimPosition, UnitStats,
};

pub const BRACKEN_ID: &str = "bracken";
pub const BRACKEN_NAME: &str = "Bracken";
pub const BRACKEN_TYPE: &str = "indoor_entity_pages";
pub const BRACKEN_SUBTYPE: &str = "hostile_creature";
pub const BRACKEN_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Bracken";
pub const BRACKEN_SOURCE_REVISION: u32 = 21471;
pub const BRACKEN_EXTRACTED_AT: &str = "2026-06-07";
pub const BRACKEN_CONFIDENCE_BASIS_POINTS: u16 = 93;

pub const BRACKEN_DWELLS: &str = "Indoors";
pub const BRACKEN_DANGER: &str = "Not mentioned by Sigurd";
pub const BRACKEN_SCIENTIFIC_NAME: &str = "Rapax-folium";
pub const BRACKEN_HP: I32F32 = I32F32::lit("5");
pub const BRACKEN_POWER_LEVEL: I32F32 = I32F32::lit("3");
pub const BRACKEN_MAX_SPAWNED: usize = 1;
pub const BRACKEN_DOOR_SPEED_MULTIPLIER: I32F32 = I32F32::lit("1.25");
pub const BRACKEN_CAN_SEE_THROUGH_FOG: bool = false;
pub const BRACKEN_SPAWN_DELAY_SECONDS: I32F32 = I32F32::lit("15");
pub const BRACKEN_STUN_MULTIPLIER: I32F32 = I32F32::lit("0.25");
pub const BRACKEN_ZAP_GUN_DIFFICULTY: I32F32 = I32F32::lit("0.8");
pub const BRACKEN_CONTACT_DAMAGE: &str = "Instant kill";
pub const BRACKEN_INTERNAL_NAME: &str = "Flowerman";
pub const BRACKEN_PIP_SIZE: &str = "Medium";

pub const BRACKEN_HUNT_SPEED: I32F32 = I32F32::lit("2");
pub const BRACKEN_RETREAT_SPEED: I32F32 = I32F32::lit("3");
pub const BRACKEN_ENRAGED_SPEED: I32F32 = I32F32::lit("4");
pub const BRACKEN_ATTACK_RANGE: I32F32 = I32F32::lit("0");
pub const BRACKEN_ATTACK_SPEED: I32F32 = I32F32::lit("1");
pub const BRACKEN_WATCH_RANGE: I32F32 = I32F32::lit("24");
pub const BRACKEN_INSTANT_KILL_DAMAGE: I32F32 = I32F32::lit("1000000");
pub const BRACKEN_RETREAT_SECONDS: I32F32 = I32F32::lit("24");
pub const BRACKEN_RETREAT_STARE_SAMPLE_SECONDS: I32F32 = I32F32::lit("0.2");
pub const BRACKEN_RETREAT_STARE_ANGER_SECONDS: I32F32 = I32F32::lit("0.2");
pub const BRACKEN_STUN_ANGER_SECONDS: I32F32 = I32F32::lit("12");
pub const BRACKEN_ENRAGED_STUN_ANGER_SECONDS: I32F32 = I32F32::lit("6");
pub const BRACKEN_WEAPON_DAMAGE_ANGER_SECONDS: I32F32 = I32F32::lit("12");

pub const BRACKEN_DEPENDS_ON: [&str; 0] = [];
pub const BRACKEN_FRONTMATTER_BEHAVIOR: [&str; 3] = ["Hunt", "Retreat", "Enrage"];

pub const BRACKEN_BEHAVIORAL_MECHANICS: [BrackenBehaviorRule; 17] = [
    BrackenBehaviorRule {
        condition: "the Bracken spawns",
        outcome: "it enters the Hunt phase and begins targeting the closest employee inside the facility",
    },
    BrackenBehaviorRule {
        condition: "the Bracken is hunting",
        outcome: "it tries to move behind the target and avoids crossing in front of them",
    },
    BrackenBehaviorRule {
        condition: "the Bracken is on a multi-level layout",
        outcome: "it may choose the opposite floor from the target to reduce the chance of being seen",
    },
    BrackenBehaviorRule {
        condition: "an employee makes physical contact with the Bracken",
        outcome: "the employee is killed instantly and the Bracken switches to Retreat",
    },
    BrackenBehaviorRule {
        condition: "there are no employees present inside the facility",
        outcome: "the Bracken returns to its preferred room and waits",
    },
    BrackenBehaviorRule {
        condition: "any employee spots the Bracken",
        outcome: "it enters Retreat, becomes audible, and runs backward toward its preferred room",
    },
    BrackenBehaviorRule {
        condition: "the Bracken is retreating",
        outcome: "the retreat state lasts 24 seconds and only counts down while it is not being watched",
    },
    BrackenBehaviorRule {
        condition: "employees maintain eye contact during Retreat",
        outcome: "the 24-second retreat timer resets to full",
    },
    BrackenBehaviorRule {
        condition: "the Bracken is being stared at while stopped during Retreat",
        outcome: "it gains 0.2 seconds of anger every 0.2 seconds",
    },
    BrackenBehaviorRule {
        condition: "the Bracken is stunned while not already enraged",
        outcome: "its anger is set to 12 seconds instantly",
    },
    BrackenBehaviorRule {
        condition: "the Bracken is stunned while already enraged",
        outcome: "its anger is set to 6 seconds instantly",
    },
    BrackenBehaviorRule {
        condition: "the Bracken takes weapon damage",
        outcome: "it becomes enraged immediately and its anger is set to 12 seconds",
    },
    BrackenBehaviorRule {
        condition: "the Bracken has stored anger",
        outcome: "its chance to enter the Enrage phase increases with the amount of anger stored; the exact probability formula is not specified",
    },
    BrackenBehaviorRule {
        condition: "the Bracken is enraged",
        outcome: "it chases the nearest employee and loses 1 second of anger every 1 second",
    },
    BrackenBehaviorRule {
        condition: "the Bracken's anger reaches 0 seconds",
        outcome: "it returns to the Retreat phase",
    },
    BrackenBehaviorRule {
        condition: "the Bracken opens doors",
        outcome: "its door speed is multiplied by 1.25 compared with baseline",
    },
    BrackenBehaviorRule {
        condition: "the Bracken is viewed through a closed or locked door",
        outcome: "it retreats even if it cannot be fully seen",
    },
];

pub struct BrackenPlugin;

impl Plugin for BrackenPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnBrackenEvent>()
            .add_event::<BrackenStateChangedEvent>()
            .add_event::<BrackenContactKillEvent>()
            .add_event::<BrackenSpottedEvent>()
            .add_event::<BrackenAudibleRetreatEvent>()
            .add_event::<BrackenDoorAttemptEvent>()
            .add_event::<BrackenDoorAttemptResolvedEvent>()
            .add_event::<BrackenStunAppliedEvent>()
            .add_event::<BrackenStunAdjustedEvent>()
            .add_event::<BrackenZapGunTargetedEvent>()
            .add_event::<BrackenZapGunDifficultyEvent>()
            .add_event::<BrackenAngerChanceIncreasedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_bracken,
                    bracken_select_closest_employee_or_wait,
                    bracken_hunt_behind_target,
                    bracken_choose_opposite_floor,
                    bracken_spot_or_door_view_retreat,
                    bracken_contact_instant_kill,
                    bracken_retreat_timer,
                    bracken_stare_anger_gain,
                    bracken_apply_stun,
                    bracken_weapon_damage_enrage,
                    bracken_report_anger_chance,
                    bracken_enraged_chase,
                    bracken_door_attempt_speed,
                    bracken_report_zap_gun_difficulty,
                    bracken_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BrackenBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Bracken;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BrackenEmployeeSensor {
    pub stable_id: u64,
    pub inside_facility: bool,
    pub can_spot_bracken: bool,
    pub watching_bracken: bool,
    pub staring_at_stopped_retreat_bracken: bool,
    pub touching_bracken: bool,
    pub viewed_through_closed_or_locked_door: bool,
    pub floor_index: i16,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BrackenPreferredRoom {
    pub position: SimPosition,
    pub floor_index: i16,
}

impl Default for BrackenPreferredRoom {
    fn default() -> Self {
        Self {
            position: SimPosition {
                x: I32F32::lit("0"),
                y: I32F32::lit("0"),
            },
            floor_index: 0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BrackenTarget {
    pub has_target: bool,
    pub stable_id: u64,
    pub position: SimPosition,
    pub floor_index: i16,
    pub wants_opposite_floor: bool,
}

impl Default for BrackenTarget {
    fn default() -> Self {
        Self {
            has_target: false,
            stable_id: 0,
            position: SimPosition {
                x: I32F32::lit("0"),
                y: I32F32::lit("0"),
            },
            floor_index: 0,
            wants_opposite_floor: false,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BrackenTimers {
    pub retreat_ticks_remaining: u32,
    pub anger_ticks: u32,
    pub stare_sample_ticks: u32,
    pub spawn_delay_ticks: u32,
}

impl Default for BrackenTimers {
    fn default() -> Self {
        Self {
            retreat_ticks_remaining: 720,
            anger_ticks: 0,
            stare_sample_ticks: 6,
            spawn_delay_ticks: 450,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BrackenState {
    #[default]
    Hunt,
    Retreat,
    Enraged,
    Waiting,
}

#[derive(Bundle)]
pub struct BrackenBundle {
    pub name: Name,
    pub bracken: Bracken,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub state: BrackenState,
    pub preferred_room: BrackenPreferredRoom,
    pub target: BrackenTarget,
    pub timers: BrackenTimers,
}

impl BrackenBundle {
    pub fn new(event: SpawnBrackenEvent, sim_hz: I32F32) -> Self {
        let retreat_ticks = fixed_seconds_to_ticks(BRACKEN_RETREAT_SECONDS, sim_hz);
        let stare_sample_ticks = fixed_seconds_to_ticks(BRACKEN_RETREAT_STARE_SAMPLE_SECONDS, sim_hz);
        let spawn_delay_ticks = fixed_seconds_to_ticks(BRACKEN_SPAWN_DELAY_SECONDS, sim_hz);

        Self {
            name: Name::new(BRACKEN_NAME),
            bracken: Bracken,
            position: event.position,
            health: Health::full(BRACKEN_HP),
            stats: UnitStats {
                move_speed: BRACKEN_HUNT_SPEED,
                attack_range: BRACKEN_ATTACK_RANGE,
                attack_damage: BRACKEN_INSTANT_KILL_DAMAGE,
                attack_speed: BRACKEN_ATTACK_SPEED,
                watch_range: BRACKEN_WATCH_RANGE,
            },
            state: BrackenState::Hunt,
            preferred_room: BrackenPreferredRoom {
                position: event.preferred_room_position,
                floor_index: event.preferred_room_floor_index,
            },
            target: BrackenTarget::default(),
            timers: BrackenTimers {
                retreat_ticks_remaining: retreat_ticks,
                anger_ticks: 0,
                stare_sample_ticks,
                spawn_delay_ticks,
            },
        }
    }
}

#[derive(Event, Clone, Copy, Debug)]
pub struct SpawnBrackenEvent {
    pub position: SimPosition,
    pub preferred_room_position: SimPosition,
    pub preferred_room_floor_index: i16,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BrackenStateChangedEvent {
    pub bracken: Entity,
    pub from: BrackenState,
    pub to: BrackenState,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BrackenContactKillEvent {
    pub bracken: Entity,
    pub employee: Entity,
    pub employee_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BrackenSpottedEvent {
    pub bracken: Entity,
    pub employee: Entity,
    pub employee_stable_id: u64,
    pub through_closed_or_locked_door: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BrackenAudibleRetreatEvent {
    pub bracken: Entity,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BrackenDoorAttemptEvent {
    pub bracken: Entity,
    pub door: Entity,
    pub base_open_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BrackenDoorAttemptResolvedEvent {
    pub bracken: Entity,
    pub door: Entity,
    pub adjusted_open_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BrackenStunAppliedEvent {
    pub bracken: Entity,
    pub base_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BrackenStunAdjustedEvent {
    pub bracken: Entity,
    pub base_ticks: u32,
    pub adjusted_ticks: u32,
    pub anger_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BrackenZapGunTargetedEvent {
    pub bracken: Entity,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BrackenZapGunDifficultyEvent {
    pub bracken: Entity,
    pub difficulty_modifier: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BrackenAngerChanceIncreasedEvent {
    pub bracken: Entity,
    pub anger_ticks: u32,
}

fn spawn_bracken(
    mut commands: Commands,
    sim_hz: Res<SimHz>,
    mut events: EventReader<SpawnBrackenEvent>,
    brackens: Query<(), With<Bracken>>,
) {
    let mut spawned_count = brackens.iter().count();

    for event in events.read() {
        if spawned_count >= BRACKEN_MAX_SPAWNED {
            break;
        }

        commands.spawn(BrackenBundle::new(*event, sim_hz.0));
        spawned_count += 1;
    }
}

fn bracken_select_closest_employee_or_wait(
    mut state_events: EventWriter<BrackenStateChangedEvent>,
    mut brackens: Query<
        (
            Entity,
            &mut SimPosition,
            &mut UnitStats,
            &mut BrackenState,
            &BrackenPreferredRoom,
            &mut BrackenTarget,
            &BrackenTimers,
        ),
        With<Bracken>,
    >,
    employees: Query<(&SimPosition, &BrackenEmployeeSensor), Without<Bracken>>,
) {
    for (bracken_entity, mut position, mut stats, mut state, room, mut target, timers) in
        brackens.iter_mut()
    {
        if timers.spawn_delay_ticks > 0 || *state == BrackenState::Retreat || *state == BrackenState::Enraged {
            continue;
        }

        let Some((employee_position, employee_sensor)) = closest_inside_employee(*position, &employees) else {
            target.has_target = false;
            target.stable_id = 0;
            target.position = room.position;
            target.floor_index = room.floor_index;
            stats.move_speed = BRACKEN_HUNT_SPEED;
            move_axis_toward(&mut position, room.position, BRACKEN_HUNT_SPEED);
            set_bracken_state(
                bracken_entity,
                &mut state,
                BrackenState::Waiting,
                &mut state_events,
            );
            continue;
        };

        target.has_target = true;
        target.stable_id = employee_sensor.stable_id;
        target.position = employee_position;
        target.floor_index = employee_sensor.floor_index;
        stats.move_speed = BRACKEN_HUNT_SPEED;

        set_bracken_state(
            bracken_entity,
            &mut state,
            BrackenState::Hunt,
            &mut state_events,
        );
    }
}

fn bracken_hunt_behind_target(
    sim_hz: Res<SimHz>,
    mut brackens: Query<(&mut SimPosition, &BrackenState, &UnitStats, &BrackenTarget), With<Bracken>>,
) {
    for (mut position, state, stats, target) in brackens.iter_mut() {
        if *state != BrackenState::Hunt || !target.has_target {
            continue;
        }

        let behind_position = SimPosition {
            x: target.position.x - I32F32::lit("1"),
            y: target.position.y,
        };
        move_axis_toward(&mut position, behind_position, stats.move_speed / sim_hz.0);
    }
}

fn bracken_choose_opposite_floor(
    mut brackens: Query<(&BrackenState, &mut BrackenTarget, &BrackenPreferredRoom), With<Bracken>>,
) {
    for (state, mut target, room) in brackens.iter_mut() {
        if *state != BrackenState::Hunt || !target.has_target {
            target.wants_opposite_floor = false;
            continue;
        }

        target.wants_opposite_floor = room.floor_index != target.floor_index;
    }
}

fn bracken_spot_or_door_view_retreat(
    mut spotted_events: EventWriter<BrackenSpottedEvent>,
    mut audible_events: EventWriter<BrackenAudibleRetreatEvent>,
    mut state_events: EventWriter<BrackenStateChangedEvent>,
    sim_hz: Res<SimHz>,
    mut brackens: Query<(Entity, &mut BrackenState, &mut UnitStats, &mut BrackenTimers), With<Bracken>>,
    employees: Query<(Entity, &BrackenEmployeeSensor), Without<Bracken>>,
) {
    let retreat_ticks = fixed_seconds_to_ticks(BRACKEN_RETREAT_SECONDS, sim_hz.0);

    for (bracken_entity, mut state, mut stats, mut timers) in brackens.iter_mut() {
        if *state == BrackenState::Enraged {
            continue;
        }

        let Some((employee_entity, sensor, through_door)) = first_spotting_employee(&employees) else {
            continue;
        };

        spotted_events.send(BrackenSpottedEvent {
            bracken: bracken_entity,
            employee: employee_entity,
            employee_stable_id: sensor.stable_id,
            through_closed_or_locked_door: through_door,
        });
        audible_events.send(BrackenAudibleRetreatEvent {
            bracken: bracken_entity,
        });
        timers.retreat_ticks_remaining = retreat_ticks;
        stats.move_speed = BRACKEN_RETREAT_SPEED;

        set_bracken_state(
            bracken_entity,
            &mut state,
            BrackenState::Retreat,
            &mut state_events,
        );
    }
}

fn bracken_contact_instant_kill(
    mut damage_events: EventWriter<IncomingDamageEvent>,
    mut contact_events: EventWriter<BrackenContactKillEvent>,
    mut state_events: EventWriter<BrackenStateChangedEvent>,
    sim_hz: Res<SimHz>,
    mut brackens: Query<(Entity, &mut BrackenState, &mut UnitStats, &mut BrackenTimers), With<Bracken>>,
    employees: Query<(Entity, &BrackenEmployeeSensor), Without<Bracken>>,
) {
    let retreat_ticks = fixed_seconds_to_ticks(BRACKEN_RETREAT_SECONDS, sim_hz.0);

    for (bracken_entity, mut state, mut stats, mut timers) in brackens.iter_mut() {
        for (employee_entity, employee_sensor) in employees.iter() {
            if !employee_sensor.touching_bracken {
                continue;
            }

            contact_events.send(BrackenContactKillEvent {
                bracken: bracken_entity,
                employee: employee_entity,
                employee_stable_id: employee_sensor.stable_id,
            });
            damage_events.send(IncomingDamageEvent {
                target: employee_entity,
                raw_amount: BRACKEN_INSTANT_KILL_DAMAGE,
                damage_type: DamageType::Standard,
                source: bracken_entity,
            });

            timers.retreat_ticks_remaining = retreat_ticks;
            stats.move_speed = BRACKEN_RETREAT_SPEED;
            set_bracken_state(
                bracken_entity,
                &mut state,
                BrackenState::Retreat,
                &mut state_events,
            );
        }
    }
}

fn bracken_retreat_timer(
    mut state_events: EventWriter<BrackenStateChangedEvent>,
    sim_hz: Res<SimHz>,
    mut brackens: Query<
        (
            Entity,
            &mut SimPosition,
            &mut UnitStats,
            &mut BrackenState,
            &BrackenPreferredRoom,
            &mut BrackenTimers,
        ),
        With<Bracken>,
    >,
    employees: Query<&BrackenEmployeeSensor, Without<Bracken>>,
) {
    let retreat_ticks = fixed_seconds_to_ticks(BRACKEN_RETREAT_SECONDS, sim_hz.0);

    for (bracken_entity, mut position, mut stats, mut state, room, mut timers) in brackens.iter_mut() {
        if *state != BrackenState::Retreat {
            continue;
        }

        move_axis_toward(&mut position, room.position, BRACKEN_RETREAT_SPEED / sim_hz.0);

        if any_employee_watching(&employees) {
            timers.retreat_ticks_remaining = retreat_ticks;
            continue;
        }

        if timers.retreat_ticks_remaining > 0 {
            timers.retreat_ticks_remaining -= 1;
            continue;
        }

        if timers.anger_ticks > 0 {
            stats.move_speed = BRACKEN_ENRAGED_SPEED;
            set_bracken_state(
                bracken_entity,
                &mut state,
                BrackenState::Enraged,
                &mut state_events,
            );
        } else {
            stats.move_speed = BRACKEN_HUNT_SPEED;
            set_bracken_state(
                bracken_entity,
                &mut state,
                BrackenState::Hunt,
                &mut state_events,
            );
        }
    }
}

fn bracken_stare_anger_gain(
    sim_hz: Res<SimHz>,
    mut brackens: Query<(&BrackenState, &mut BrackenTimers), With<Bracken>>,
    employees: Query<&BrackenEmployeeSensor, Without<Bracken>>,
) {
    let anger_gain_ticks = fixed_seconds_to_ticks(BRACKEN_RETREAT_STARE_ANGER_SECONDS, sim_hz.0);
    let sample_ticks = fixed_seconds_to_ticks(BRACKEN_RETREAT_STARE_SAMPLE_SECONDS, sim_hz.0);

    for (state, mut timers) in brackens.iter_mut() {
        if *state != BrackenState::Retreat {
            timers.stare_sample_ticks = sample_ticks;
            continue;
        }

        if !any_employee_staring_at_stopped_retreat(&employees) {
            timers.stare_sample_ticks = sample_ticks;
            continue;
        }

        if timers.stare_sample_ticks > 0 {
            timers.stare_sample_ticks -= 1;
            continue;
        }

        timers.anger_ticks = timers.anger_ticks.saturating_add(anger_gain_ticks);
        timers.stare_sample_ticks = sample_ticks;
    }
}

fn bracken_apply_stun(
    mut stun_events: EventReader<BrackenStunAppliedEvent>,
    mut adjusted_events: EventWriter<BrackenStunAdjustedEvent>,
    sim_hz: Res<SimHz>,
    mut brackens: Query<(&BrackenState, &mut BrackenTimers), With<Bracken>>,
) {
    let not_enraged_anger_ticks = fixed_seconds_to_ticks(BRACKEN_STUN_ANGER_SECONDS, sim_hz.0);
    let enraged_anger_ticks = fixed_seconds_to_ticks(BRACKEN_ENRAGED_STUN_ANGER_SECONDS, sim_hz.0);

    for event in stun_events.read() {
        let Ok((state, mut timers)) = brackens.get_mut(event.bracken) else {
            continue;
        };

        let anger_ticks = if *state == BrackenState::Enraged {
            enraged_anger_ticks
        } else {
            not_enraged_anger_ticks
        };
        timers.anger_ticks = anger_ticks;

        adjusted_events.send(BrackenStunAdjustedEvent {
            bracken: event.bracken,
            base_ticks: event.base_ticks,
            adjusted_ticks: fixed_ticks_scaled(event.base_ticks, BRACKEN_STUN_MULTIPLIER),
            anger_ticks,
        });
    }
}

fn bracken_weapon_damage_enrage(
    mut damage_events: EventReader<IncomingDamageEvent>,
    mut state_events: EventWriter<BrackenStateChangedEvent>,
    sim_hz: Res<SimHz>,
    mut brackens: Query<(Entity, &mut BrackenState, &mut UnitStats, &mut BrackenTimers), With<Bracken>>,
) {
    let anger_ticks = fixed_seconds_to_ticks(BRACKEN_WEAPON_DAMAGE_ANGER_SECONDS, sim_hz.0);

    for event in damage_events.read() {
        let Ok((bracken_entity, mut state, mut stats, mut timers)) = brackens.get_mut(event.target) else {
            continue;
        };

        timers.anger_ticks = anger_ticks;
        stats.move_speed = BRACKEN_ENRAGED_SPEED;
        set_bracken_state(
            bracken_entity,
            &mut state,
            BrackenState::Enraged,
            &mut state_events,
        );
    }
}

fn bracken_report_anger_chance(
    mut chance_events: EventWriter<BrackenAngerChanceIncreasedEvent>,
    brackens: Query<(Entity, &BrackenTimers), With<Bracken>>,
) {
    for (bracken_entity, timers) in brackens.iter() {
        if timers.anger_ticks == 0 {
            continue;
        }

        chance_events.send(BrackenAngerChanceIncreasedEvent {
            bracken: bracken_entity,
            anger_ticks: timers.anger_ticks,
        });
    }
}

fn bracken_enraged_chase(
    mut state_events: EventWriter<BrackenStateChangedEvent>,
    sim_hz: Res<SimHz>,
    mut brackens: Query<
        (Entity, &mut SimPosition, &mut UnitStats, &mut BrackenState, &mut BrackenTarget, &mut BrackenTimers),
        With<Bracken>,
    >,
    employees: Query<(&SimPosition, &BrackenEmployeeSensor), Without<Bracken>>,
) {
    let anger_loss_ticks = fixed_seconds_to_ticks(I32F32::lit("1"), sim_hz.0);

    for (bracken_entity, mut position, mut stats, mut state, mut target, mut timers) in brackens.iter_mut() {
        if *state != BrackenState::Enraged {
            continue;
        }

        if let Some((employee_position, employee_sensor)) = closest_inside_employee(*position, &employees) {
            target.has_target = true;
            target.stable_id = employee_sensor.stable_id;
            target.position = employee_position;
            target.floor_index = employee_sensor.floor_index;
            move_axis_toward(&mut position, employee_position, BRACKEN_ENRAGED_SPEED / sim_hz.0);
        }

        if timers.anger_ticks > anger_loss_ticks {
            timers.anger_ticks -= anger_loss_ticks;
        } else {
            timers.anger_ticks = 0;
            stats.move_speed = BRACKEN_RETREAT_SPEED;
            set_bracken_state(
                bracken_entity,
                &mut state,
                BrackenState::Retreat,
                &mut state_events,
            );
        }
    }
}

fn bracken_door_attempt_speed(
    mut events: EventReader<BrackenDoorAttemptEvent>,
    mut resolved_events: EventWriter<BrackenDoorAttemptResolvedEvent>,
    brackens: Query<(), With<Bracken>>,
) {
    for event in events.read() {
        if brackens.get(event.bracken).is_err() {
            continue;
        }

        resolved_events.send(BrackenDoorAttemptResolvedEvent {
            bracken: event.bracken,
            door: event.door,
            adjusted_open_ticks: fixed_ticks_scaled(event.base_open_ticks, BRACKEN_DOOR_SPEED_MULTIPLIER),
        });
    }
}

fn bracken_report_zap_gun_difficulty(
    mut events: EventReader<BrackenZapGunTargetedEvent>,
    mut difficulty_events: EventWriter<BrackenZapGunDifficultyEvent>,
    brackens: Query<(), With<Bracken>>,
) {
    for event in events.read() {
        if brackens.get(event.bracken).is_err() {
            continue;
        }

        difficulty_events.send(BrackenZapGunDifficultyEvent {
            bracken: event.bracken,
            difficulty_modifier: BRACKEN_ZAP_GUN_DIFFICULTY,
        });
    }
}

fn bracken_checksum(
    mut checksum: ResMut<SimChecksumState>,
    brackens: Query<
        (
            &SimPosition,
            &Health,
            &UnitStats,
            &BrackenState,
            &BrackenPreferredRoom,
            &BrackenTarget,
            &BrackenTimers,
        ),
        With<Bracken>,
    >,
) {
    for (position, health, stats, state, room, target, timers) in brackens.iter() {
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);
        checksum.accumulate(bracken_state_bits(*state));
        checksum.accumulate(room.position.x.to_bits() as u64);
        checksum.accumulate(room.position.y.to_bits() as u64);
        checksum.accumulate(room.floor_index as u64);
        checksum.accumulate(target.has_target as u64);
        checksum.accumulate(target.stable_id);
        checksum.accumulate(target.position.x.to_bits() as u64);
        checksum.accumulate(target.position.y.to_bits() as u64);
        checksum.accumulate(target.floor_index as u64);
        checksum.accumulate(target.wants_opposite_floor as u64);
        checksum.accumulate(timers.retreat_ticks_remaining as u64);
        checksum.accumulate(timers.anger_ticks as u64);
        checksum.accumulate(timers.stare_sample_ticks as u64);
        checksum.accumulate(timers.spawn_delay_ticks as u64);
    }
}

fn closest_inside_employee(
    origin: SimPosition,
    employees: &Query<(&SimPosition, &BrackenEmployeeSensor), Without<Bracken>>,
) -> Option<(SimPosition, BrackenEmployeeSensor)> {
    let mut best: Option<(SimPosition, BrackenEmployeeSensor, I32F32)> = None;

    for (position, sensor) in employees.iter() {
        if !sensor.inside_facility {
            continue;
        }

        let distance = fixed_abs(position.x - origin.x) + fixed_abs(position.y - origin.y);
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

fn first_spotting_employee(
    employees: &Query<(Entity, &BrackenEmployeeSensor), Without<Bracken>>,
) -> Option<(Entity, BrackenEmployeeSensor, bool)> {
    let mut best: Option<(Entity, BrackenEmployeeSensor, bool)> = None;

    for (entity, sensor) in employees.iter() {
        let through_door = sensor.viewed_through_closed_or_locked_door;
        if !sensor.can_spot_bracken && !through_door {
            continue;
        }

        if let Some((_best_entity, best_sensor, _best_through_door)) = best {
            if sensor.stable_id >= best_sensor.stable_id {
                continue;
            }
        }

        best = Some((entity, *sensor, through_door));
    }

    best
}

fn any_employee_watching(employees: &Query<&BrackenEmployeeSensor, Without<Bracken>>) -> bool {
    for employee in employees.iter() {
        if employee.watching_bracken {
            return true;
        }
    }

    false
}

fn any_employee_staring_at_stopped_retreat(
    employees: &Query<&BrackenEmployeeSensor, Without<Bracken>>,
) -> bool {
    for employee in employees.iter() {
        if employee.staring_at_stopped_retreat_bracken {
            return true;
        }
    }

    false
}

fn set_bracken_state(
    bracken: Entity,
    state: &mut BrackenState,
    next: BrackenState,
    events: &mut EventWriter<BrackenStateChangedEvent>,
) {
    if *state == next {
        return;
    }

    let previous = *state;
    *state = next;
    events.send(BrackenStateChangedEvent {
        bracken,
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

fn bracken_state_bits(state: BrackenState) -> u64 {
    match state {
        BrackenState::Hunt => 0,
        BrackenState::Retreat => 1,
        BrackenState::Enraged => 2,
        BrackenState::Waiting => 3,
    }
}
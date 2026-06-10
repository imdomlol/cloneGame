// Sources: vault/outdoor_entity_pages/eyeless_dog.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{
    DamageType, Health, IncomingDamageEvent, NoiseEmittedEvent, SimChecksumState, SimHz,
    SimPosition, SimTick, UnitStats,
};

pub const EYELESS_DOG_ID: &str = "eyeless_dog";
pub const EYELESS_DOG_NAME: &str = "Eyeless Dog";
pub const EYELESS_DOG_TYPE: &str = "outdoor_entity_pages";
pub const EYELESS_DOG_SUBTYPE: &str = "creature";
pub const EYELESS_DOG_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Eyeless_Dog";
pub const EYELESS_DOG_SOURCE_REVISION: u32 = 21379;
pub const EYELESS_DOG_EXTRACTED_AT: &str = "2026-06-07T00:00:00Z";
pub const EYELESS_DOG_CONFIDENCE_BASIS_POINTS: u16 = 90;

pub const EYELESS_DOG_IMAGE: &str = "EyelessdogQuan1.png";
pub const EYELESS_DOG_DWELLS: &str = "Outside (Night/Eclipse)";
pub const EYELESS_DOG_SCIENTIFIC_NAME: &str = "Leo Caecus";
pub const EYELESS_DOG_POWER_LEVEL: I32F32 = I32F32::lit("2");
pub const EYELESS_DOG_MAX_SPAWNED: usize = 8;
pub const EYELESS_DOG_ATTACK_DAMAGE_TEXT: &str =
    "Instant kill on employees; 3 damage to other entities";
pub const EYELESS_DOG_ENTITY_ATTACK_DAMAGE: I32F32 = I32F32::lit("3");
pub const EYELESS_DOG_EMPLOYEE_INSTANT_KILL_DAMAGE: I32F32 = I32F32::lit("9999");
pub const EYELESS_DOG_STUN_MULTIPLIER: I32F32 = I32F32::lit("0.7");
pub const EYELESS_DOG_RADAR_PIP_SIZE: &str = "Medium-large";
pub const EYELESS_DOG_SHOVEL_HP: I32F32 = I32F32::lit("12");
pub const EYELESS_DOG_CAN_SEE_THROUGH_FOG: bool = false;
pub const EYELESS_DOG_DOOR_OPEN_SPEED: I32F32 = I32F32::lit("0");
pub const EYELESS_DOG_INTERNAL_NAME: &str = "Dog";

pub const EYELESS_DOG_SUSPICIOUS_NOISE_SECONDS: I32F32 = I32F32::lit("3");
pub const EYELESS_DOG_CHASING_NOISE_SECONDS: I32F32 = I32F32::lit("4");
pub const EYELESS_DOG_SUSPICION_THRESHOLD_SECONDS: I32F32 = I32F32::lit("6");
pub const EYELESS_DOG_LAST_HEARD_LUNGE_RANGE: I32F32 = I32F32::lit("4");
pub const EYELESS_DOG_MIN_LUNGE_SECONDS: I32F32 = I32F32::lit("2.3");
pub const EYELESS_DOG_MAX_LUNGE_SECONDS: I32F32 = I32F32::lit("3.3");
pub const EYELESS_DOG_KILL_ANIMATION_DEAF_SECONDS: I32F32 = I32F32::lit("5");
pub const EYELESS_DOG_HEARING_RANGE: I32F32 = I32F32::lit("45");
pub const EYELESS_DOG_LOUD_NOISE_RANGE: I32F32 = I32F32::lit("12");
pub const EYELESS_DOG_CONTACT_RANGE: I32F32 = I32F32::lit("1");
pub const EYELESS_DOG_ROAM_SPEED: I32F32 = I32F32::lit("3");
pub const EYELESS_DOG_SUSPICIOUS_SPEED: I32F32 = I32F32::lit("5");
pub const EYELESS_DOG_CHASE_SPEED: I32F32 = I32F32::lit("8");
pub const EYELESS_DOG_LUNGE_SPEED: I32F32 = I32F32::lit("16");
pub const EYELESS_DOG_ATTACK_SPEED_SECONDS: I32F32 = I32F32::lit("1");

pub const EYELESS_DOG_FRONTMATTER_BEHAVIOR: [&str; 3] =
    ["Roaming", "Suspicious", "Charging"];

pub const EYELESS_DOG_BEHAVIORAL_MECHANICS: [EyelessDogBehaviorRule; 9] = [
    EyelessDogBehaviorRule {
        condition: "a sound occurs near it",
        outcome: "it becomes suspicious and raises its suspicion timer by 3 seconds or 4 seconds depending on its current state",
    },
    EyelessDogBehaviorRule {
        condition: "a sound is loud enough",
        outcome: "it immediately attacks",
    },
    EyelessDogBehaviorRule {
        condition: "the suspicion timer exceeds its threshold",
        outcome: "it switches to chasing and howls to alert nearby dogs",
    },
    EyelessDogBehaviorRule {
        condition: "it is chasing and gets within 4 meters of the last heard noise",
        outcome: "it accelerates and lunges",
    },
    EyelessDogBehaviorRule {
        condition: "it is lunging",
        outcome: "the lunge lasts 2.3 seconds to 3.3 seconds depending on speed and pauses the suspicion timer",
    },
    EyelessDogBehaviorRule {
        condition: "it loses suspicion while chasing",
        outcome: "it returns to suspicious",
    },
    EyelessDogBehaviorRule {
        condition: "it loses suspicion while suspicious",
        outcome: "it becomes calm",
    },
    EyelessDogBehaviorRule {
        condition: "it contacts an employee",
        outcome: "it kills instantly",
    },
    EyelessDogBehaviorRule {
        condition: "it is in a kill animation",
        outcome: "it does not hear noise for 5 seconds",
    },
];

pub struct EyelessDogPlugin;

impl Plugin for EyelessDogPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnEyelessDogEvent>()
            .add_event::<EyelessDogStateChangedEvent>()
            .add_event::<EyelessDogNoiseHeardEvent>()
            .add_event::<EyelessDogHowlEvent>()
            .add_event::<EyelessDogLungeStartedEvent>()
            .add_event::<EyelessDogContactKillEvent>()
            .add_event::<EyelessDogDamageTakenEvent>()
            .add_event::<EyelessDogDefeatedEvent>()
            .add_event::<EyelessDogStunAdjustedEvent>()
            .add_event::<EyelessDogDoorAttemptEvent>()
            .add_event::<EyelessDogDoorAttemptResolvedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    eyeless_dog_spawn_from_events,
                    eyeless_dog_hear_noise,
                    eyeless_dog_update_suspicion,
                    eyeless_dog_move,
                    eyeless_dog_contact_attack,
                    eyeless_dog_take_damage,
                    eyeless_dog_door_attempts,
                    eyeless_dog_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EyelessDogBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EyelessDog {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EyelessDogState {
    #[default]
    Roaming,
    Suspicious,
    Chasing,
    Lunging,
    KillAnimation,
    Dead,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct EyelessDogHearing {
    pub suspicion_ticks: u32,
    pub suspicion_threshold_ticks: u32,
    pub last_heard_position: SimPosition,
    pub last_noise_source: Entity,
    pub last_noise_source_stable_id: u64,
    pub deaf_ticks: u32,
}

impl Default for EyelessDogHearing {
    fn default() -> Self {
        Self {
            suspicion_ticks: 0,
            suspicion_threshold_ticks: 0,
            last_heard_position: SimPosition {
                x: I32F32::ZERO,
                y: I32F32::ZERO,
            },
            last_noise_source: Entity::PLACEHOLDER,
            last_noise_source_stable_id: 0,
            deaf_ticks: 0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EyelessDogMotion {
    pub lunge_ticks: u32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EyelessDogTargetSensor {
    pub stable_id: u64,
    pub is_employee: bool,
    pub is_alive: bool,
}

#[derive(Bundle)]
pub struct EyelessDogBundle {
    pub name: Name,
    pub dog: EyelessDog,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub state: EyelessDogState,
    pub hearing: EyelessDogHearing,
    pub motion: EyelessDogMotion,
}

impl EyelessDogBundle {
    pub fn new(event: SpawnEyelessDogEvent, sim_hz: I32F32) -> Self {
        Self {
            name: Name::new(EYELESS_DOG_NAME),
            dog: EyelessDog {
                stable_id: event.stable_id,
            },
            position: event.position,
            health: Health::full(EYELESS_DOG_SHOVEL_HP),
            stats: UnitStats {
                move_speed: EYELESS_DOG_ROAM_SPEED,
                attack_range: EYELESS_DOG_CONTACT_RANGE,
                attack_damage: EYELESS_DOG_ENTITY_ATTACK_DAMAGE,
                attack_speed: EYELESS_DOG_ATTACK_SPEED_SECONDS,
                watch_range: EYELESS_DOG_HEARING_RANGE,
            },
            state: EyelessDogState::Roaming,
            hearing: EyelessDogHearing {
                suspicion_threshold_ticks: fixed_seconds_to_ticks(
                    EYELESS_DOG_SUSPICION_THRESHOLD_SECONDS,
                    sim_hz,
                ),
                last_heard_position: event.position,
                ..Default::default()
            },
            motion: EyelessDogMotion::default(),
        }
    }
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpawnEyelessDogEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct EyelessDogStateChangedEvent {
    pub dog: Entity,
    pub from: EyelessDogState,
    pub to: EyelessDogState,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct EyelessDogNoiseHeardEvent {
    pub dog: Entity,
    pub noise_source: Entity,
    pub noise_position: SimPosition,
    pub added_suspicion_ticks: u32,
    pub immediate_attack: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct EyelessDogHowlEvent {
    pub dog: Entity,
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct EyelessDogLungeStartedEvent {
    pub dog: Entity,
    pub target_position: SimPosition,
    pub lunge_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct EyelessDogContactKillEvent {
    pub dog: Entity,
    pub target: Entity,
    pub target_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct EyelessDogDamageTakenEvent {
    pub dog: Entity,
    pub source: Entity,
    pub damage: I32F32,
    pub remaining_health: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct EyelessDogDefeatedEvent {
    pub dog: Entity,
    pub source: Entity,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct EyelessDogStunAdjustedEvent {
    pub dog: Entity,
    pub source: Entity,
    pub adjusted_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct EyelessDogDoorAttemptEvent {
    pub dog: Entity,
    pub door: Entity,
    pub base_open_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct EyelessDogDoorAttemptResolvedEvent {
    pub dog: Entity,
    pub door: Entity,
    pub adjusted_open_ticks: u32,
}

fn eyeless_dog_spawn_from_events(
    mut commands: Commands,
    sim_hz: Res<SimHz>,
    mut events: EventReader<SpawnEyelessDogEvent>,
    dogs: Query<(), With<EyelessDog>>,
) {
    let mut spawned_count = dogs.iter().count();

    for event in events.read() {
        if spawned_count >= EYELESS_DOG_MAX_SPAWNED {
            break;
        }

        commands.spawn(EyelessDogBundle::new(*event, sim_hz.0));
        spawned_count += 1;
    }
}

fn eyeless_dog_hear_noise(
    sim_hz: Res<SimHz>,
    mut noise_events: EventReader<NoiseEmittedEvent>,
    mut heard_events: EventWriter<EyelessDogNoiseHeardEvent>,
    mut state_events: EventWriter<EyelessDogStateChangedEvent>,
    mut howl_events: EventWriter<EyelessDogHowlEvent>,
    mut dogs: Query<
        (
            Entity,
            &SimPosition,
            &mut EyelessDogState,
            &mut EyelessDogHearing,
            &mut UnitStats,
        ),
        With<EyelessDog>,
    >,
) {
    for noise in noise_events.read() {
        for (dog_entity, dog_position, mut state, mut hearing, mut stats) in dogs.iter_mut() {
            if *state == EyelessDogState::Dead || *state == EyelessDogState::KillAnimation {
                continue;
            }

            if hearing.deaf_ticks > 0 {
                continue;
            }

            let distance_sq = fixed_distance_sq(*dog_position, noise.position);
            if distance_sq > fixed_square(EYELESS_DOG_HEARING_RANGE) {
                continue;
            }

            let immediate_attack = distance_sq <= fixed_square(EYELESS_DOG_LOUD_NOISE_RANGE);
            let added_seconds = if *state == EyelessDogState::Chasing {
                EYELESS_DOG_CHASING_NOISE_SECONDS
            } else {
                EYELESS_DOG_SUSPICIOUS_NOISE_SECONDS
            };
            let added_ticks = fixed_seconds_to_ticks(added_seconds, sim_hz.0);

            hearing.last_heard_position = noise.position;
            hearing.last_noise_source = noise.source;
            hearing.suspicion_ticks = hearing.suspicion_ticks.saturating_add(added_ticks);

            if immediate_attack {
                stats.move_speed = EYELESS_DOG_CHASE_SPEED;
                set_eyeless_dog_state(
                    dog_entity,
                    &mut state,
                    EyelessDogState::Chasing,
                    &mut state_events,
                );
                howl_events.send(EyelessDogHowlEvent {
                    dog: dog_entity,
                    position: *dog_position,
                });
            } else if *state == EyelessDogState::Roaming {
                stats.move_speed = EYELESS_DOG_SUSPICIOUS_SPEED;
                set_eyeless_dog_state(
                    dog_entity,
                    &mut state,
                    EyelessDogState::Suspicious,
                    &mut state_events,
                );
            }

            heard_events.send(EyelessDogNoiseHeardEvent {
                dog: dog_entity,
                noise_source: noise.source,
                noise_position: noise.position,
                added_suspicion_ticks: added_ticks,
                immediate_attack,
            });
        }
    }
}

fn eyeless_dog_update_suspicion(
    mut state_events: EventWriter<EyelessDogStateChangedEvent>,
    mut howl_events: EventWriter<EyelessDogHowlEvent>,
    mut dogs: Query<
        (
            Entity,
            &SimPosition,
            &mut EyelessDogState,
            &mut EyelessDogHearing,
            &mut UnitStats,
        ),
        With<EyelessDog>,
    >,
) {
    for (dog_entity, dog_position, mut state, mut hearing, mut stats) in dogs.iter_mut() {
        if hearing.deaf_ticks > 0 {
            hearing.deaf_ticks -= 1;
        }

        match *state {
            EyelessDogState::Suspicious => {
                if hearing.suspicion_ticks >= hearing.suspicion_threshold_ticks {
                    stats.move_speed = EYELESS_DOG_CHASE_SPEED;
                    set_eyeless_dog_state(
                        dog_entity,
                        &mut state,
                        EyelessDogState::Chasing,
                        &mut state_events,
                    );
                    howl_events.send(EyelessDogHowlEvent {
                        dog: dog_entity,
                        position: *dog_position,
                    });
                } else if hearing.suspicion_ticks > 0 {
                    hearing.suspicion_ticks -= 1;
                } else {
                    stats.move_speed = EYELESS_DOG_ROAM_SPEED;
                    set_eyeless_dog_state(
                        dog_entity,
                        &mut state,
                        EyelessDogState::Roaming,
                        &mut state_events,
                    );
                }
            }
            EyelessDogState::Chasing => {
                if hearing.suspicion_ticks > 0 {
                    hearing.suspicion_ticks -= 1;
                } else {
                    stats.move_speed = EYELESS_DOG_SUSPICIOUS_SPEED;
                    set_eyeless_dog_state(
                        dog_entity,
                        &mut state,
                        EyelessDogState::Suspicious,
                        &mut state_events,
                    );
                }
            }
            _ => {}
        }
    }
}

fn eyeless_dog_move(
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<EyelessDogStateChangedEvent>,
    mut lunge_events: EventWriter<EyelessDogLungeStartedEvent>,
    mut dogs: Query<
        (
            Entity,
            &mut SimPosition,
            &mut EyelessDogState,
            &EyelessDogHearing,
            &mut EyelessDogMotion,
            &mut UnitStats,
        ),
        With<EyelessDog>,
    >,
) {
    for (dog_entity, mut position, mut state, hearing, mut motion, mut stats) in dogs.iter_mut() {
        match *state {
            EyelessDogState::Suspicious | EyelessDogState::Chasing => {
                move_axis_toward(
                    &mut position,
                    hearing.last_heard_position,
                    stats.move_speed / sim_hz.0,
                );

                if *state == EyelessDogState::Chasing
                    && fixed_distance_sq(*position, hearing.last_heard_position)
                        <= fixed_square(EYELESS_DOG_LAST_HEARD_LUNGE_RANGE)
                {
                    stats.move_speed = EYELESS_DOG_LUNGE_SPEED;
                    motion.lunge_ticks =
                        lunge_ticks_for_speed(EYELESS_DOG_LUNGE_SPEED, sim_hz.0);
                    set_eyeless_dog_state(
                        dog_entity,
                        &mut state,
                        EyelessDogState::Lunging,
                        &mut state_events,
                    );
                    lunge_events.send(EyelessDogLungeStartedEvent {
                        dog: dog_entity,
                        target_position: hearing.last_heard_position,
                        lunge_ticks: motion.lunge_ticks,
                    });
                }
            }
            EyelessDogState::Lunging => {
                move_axis_toward(
                    &mut position,
                    hearing.last_heard_position,
                    EYELESS_DOG_LUNGE_SPEED / sim_hz.0,
                );

                if motion.lunge_ticks > 0 {
                    motion.lunge_ticks -= 1;
                } else {
                    stats.move_speed = EYELESS_DOG_CHASE_SPEED;
                    set_eyeless_dog_state(
                        dog_entity,
                        &mut state,
                        EyelessDogState::Chasing,
                        &mut state_events,
                    );
                }
            }
            _ => {}
        }
    }
}

fn eyeless_dog_contact_attack(
    sim_hz: Res<SimHz>,
    mut damage_events: EventWriter<IncomingDamageEvent>,
    mut contact_events: EventWriter<EyelessDogContactKillEvent>,
    mut state_events: EventWriter<EyelessDogStateChangedEvent>,
    mut dogs: Query<
        (
            Entity,
            &SimPosition,
            &mut EyelessDogState,
            &mut EyelessDogHearing,
            &mut UnitStats,
        ),
        With<EyelessDog>,
    >,
    targets: Query<(Entity, &SimPosition, &EyelessDogTargetSensor), Without<EyelessDog>>,
) {
    for (dog_entity, dog_position, mut state, mut hearing, mut stats) in dogs.iter_mut() {
        if *state == EyelessDogState::Dead || *state == EyelessDogState::KillAnimation {
            continue;
        }

        for (target_entity, target_position, sensor) in targets.iter() {
            if !sensor.is_alive {
                continue;
            }

            if fixed_distance_sq(*dog_position, *target_position)
                > fixed_square(EYELESS_DOG_CONTACT_RANGE)
            {
                continue;
            }

            let damage = if sensor.is_employee {
                EYELESS_DOG_EMPLOYEE_INSTANT_KILL_DAMAGE
            } else {
                EYELESS_DOG_ENTITY_ATTACK_DAMAGE
            };

            damage_events.send(IncomingDamageEvent {
                target: target_entity,
                raw_amount: damage,
                damage_type: DamageType::Standard,
                source: dog_entity,
            });

            if sensor.is_employee {
                contact_events.send(EyelessDogContactKillEvent {
                    dog: dog_entity,
                    target: target_entity,
                    target_stable_id: sensor.stable_id,
                });

                hearing.deaf_ticks =
                    fixed_seconds_to_ticks(EYELESS_DOG_KILL_ANIMATION_DEAF_SECONDS, sim_hz.0);
                stats.move_speed = I32F32::ZERO;
                set_eyeless_dog_state(
                    dog_entity,
                    &mut state,
                    EyelessDogState::KillAnimation,
                    &mut state_events,
                );
            }

            break;
        }
    }
}

fn eyeless_dog_take_damage(
    mut damage_events: EventReader<IncomingDamageEvent>,
    mut taken_events: EventWriter<EyelessDogDamageTakenEvent>,
    mut defeated_events: EventWriter<EyelessDogDefeatedEvent>,
    mut stun_events: EventWriter<EyelessDogStunAdjustedEvent>,
    mut dogs: Query<(Entity, &mut Health, &mut EyelessDogState), With<EyelessDog>>,
) {
    for event in damage_events.read() {
        let Ok((dog_entity, mut health, mut state)) = dogs.get_mut(event.target) else {
            continue;
        };

        if *state == EyelessDogState::Dead {
            continue;
        }

        health.current -= event.raw_amount;
        if health.current < I32F32::ZERO {
            health.current = I32F32::ZERO;
        }

        taken_events.send(EyelessDogDamageTakenEvent {
            dog: dog_entity,
            source: event.source,
            damage: event.raw_amount,
            remaining_health: health.current,
        });

        stun_events.send(EyelessDogStunAdjustedEvent {
            dog: dog_entity,
            source: event.source,
            adjusted_ticks: 0,
        });

        if health.current <= I32F32::ZERO {
            *state = EyelessDogState::Dead;
            defeated_events.send(EyelessDogDefeatedEvent {
                dog: dog_entity,
                source: event.source,
            });
        }
    }
}

fn eyeless_dog_door_attempts(
    mut events: EventReader<EyelessDogDoorAttemptEvent>,
    mut resolved_events: EventWriter<EyelessDogDoorAttemptResolvedEvent>,
    dogs: Query<(), With<EyelessDog>>,
) {
    for event in events.read() {
        if dogs.get(event.dog).is_err() {
            continue;
        }

        resolved_events.send(EyelessDogDoorAttemptResolvedEvent {
            dog: event.dog,
            door: event.door,
            adjusted_open_ticks: 0,
        });
    }
}

fn eyeless_dog_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    sim_hz: Res<SimHz>,
    dogs: Query<
        (
            &EyelessDog,
            &SimPosition,
            &Health,
            &UnitStats,
            &EyelessDogState,
            &EyelessDogHearing,
            &EyelessDogMotion,
        ),
        With<EyelessDog>,
    >,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(sim_hz.0.to_bits() as u64);
    checksum.accumulate(EYELESS_DOG_SOURCE_REVISION as u64);
    checksum.accumulate(EYELESS_DOG_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(EYELESS_DOG_POWER_LEVEL.to_bits() as u64);
    checksum.accumulate(EYELESS_DOG_MAX_SPAWNED as u64);
    checksum.accumulate(EYELESS_DOG_ENTITY_ATTACK_DAMAGE.to_bits() as u64);
    checksum.accumulate(EYELESS_DOG_EMPLOYEE_INSTANT_KILL_DAMAGE.to_bits() as u64);
    checksum.accumulate(EYELESS_DOG_STUN_MULTIPLIER.to_bits() as u64);
    checksum.accumulate(EYELESS_DOG_SHOVEL_HP.to_bits() as u64);
    checksum.accumulate(EYELESS_DOG_CAN_SEE_THROUGH_FOG as u64);
    checksum.accumulate(EYELESS_DOG_DOOR_OPEN_SPEED.to_bits() as u64);

    accumulate_str(&mut checksum, 0x1000, EYELESS_DOG_ID);
    accumulate_str(&mut checksum, 0x1001, EYELESS_DOG_NAME);
    accumulate_str(&mut checksum, 0x1002, EYELESS_DOG_TYPE);
    accumulate_str(&mut checksum, 0x1003, EYELESS_DOG_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, EYELESS_DOG_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1005, EYELESS_DOG_EXTRACTED_AT);
    accumulate_str(&mut checksum, 0x1006, EYELESS_DOG_IMAGE);
    accumulate_str(&mut checksum, 0x1007, EYELESS_DOG_DWELLS);
    accumulate_str(&mut checksum, 0x1008, EYELESS_DOG_SCIENTIFIC_NAME);
    accumulate_str(&mut checksum, 0x1009, EYELESS_DOG_ATTACK_DAMAGE_TEXT);
    accumulate_str(&mut checksum, 0x100A, EYELESS_DOG_RADAR_PIP_SIZE);
    accumulate_str(&mut checksum, 0x100B, EYELESS_DOG_INTERNAL_NAME);

    for behavior in EYELESS_DOG_FRONTMATTER_BEHAVIOR {
        accumulate_str(&mut checksum, 0x2000, behavior);
    }

    for rule in EYELESS_DOG_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x3000, rule.condition);
        accumulate_str(&mut checksum, 0x3001, rule.outcome);
    }

    for (dog, position, health, stats, state, hearing, motion) in dogs.iter() {
        checksum.accumulate(dog.stable_id);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);
        checksum.accumulate(eyeless_dog_state_bits(*state));
        checksum.accumulate(hearing.suspicion_ticks as u64);
        checksum.accumulate(hearing.suspicion_threshold_ticks as u64);
        checksum.accumulate(hearing.last_heard_position.x.to_bits() as u64);
        checksum.accumulate(hearing.last_heard_position.y.to_bits() as u64);
        checksum.accumulate(hearing.last_noise_source_stable_id);
        checksum.accumulate(hearing.deaf_ticks as u64);
        checksum.accumulate(motion.lunge_ticks as u64);
    }
}

fn set_eyeless_dog_state(
    dog: Entity,
    state: &mut EyelessDogState,
    next: EyelessDogState,
    events: &mut EventWriter<EyelessDogStateChangedEvent>,
) {
    if *state == next {
        return;
    }

    let previous = *state;
    *state = next;
    events.send(EyelessDogStateChangedEvent {
        dog,
        from: previous,
        to: next,
    });
}

fn lunge_ticks_for_speed(speed: I32F32, sim_hz: I32F32) -> u32 {
    let speed_ratio = if speed <= EYELESS_DOG_CHASE_SPEED {
        I32F32::ZERO
    } else {
        (speed - EYELESS_DOG_CHASE_SPEED) / (EYELESS_DOG_LUNGE_SPEED - EYELESS_DOG_CHASE_SPEED)
    };
    let seconds = EYELESS_DOG_MAX_LUNGE_SECONDS
        - ((EYELESS_DOG_MAX_LUNGE_SECONDS - EYELESS_DOG_MIN_LUNGE_SECONDS) * speed_ratio);
    fixed_seconds_to_ticks(seconds, sim_hz)
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

fn fixed_seconds_to_ticks(seconds: I32F32, sim_hz: I32F32) -> u32 {
    let ticks = seconds * sim_hz;
    if ticks <= I32F32::ZERO {
        0
    } else {
        ticks.ceil().to_num::<u32>()
    }
}

fn eyeless_dog_state_bits(state: EyelessDogState) -> u64 {
    match state {
        EyelessDogState::Roaming => 0,
        EyelessDogState::Suspicious => 1,
        EyelessDogState::Chasing => 2,
        EyelessDogState::Lunging => 3,
        EyelessDogState::KillAnimation => 4,
        EyelessDogState::Dead => 5,
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}
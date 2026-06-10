// Sources: vault/outdoor_entity_pages/old_bird.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{
    DamageType, Health, IncomingDamageEvent, SimChecksumState, SimHz, SimPosition, SimTick,
    UnitStats,
};

pub const OLD_BIRD_ID: &str = "old_bird";
pub const OLD_BIRD_NAME: &str = "Old Bird";
pub const OLD_BIRD_TYPE: &str = "outdoor_entity_pages";
pub const OLD_BIRD_SUBTYPE: &str = "creature";
pub const OLD_BIRD_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Old_Bird";
pub const OLD_BIRD_SOURCE_REVISION: u32 = 21354;
pub const OLD_BIRD_EXTRACTED_AT: &str = "2026-06-07T00:00:00Z";
pub const OLD_BIRD_CONFIDENCE_BASIS_POINTS: u16 = 94;

pub const OLD_BIRD_DWELLS: &str = "Outside";
pub const OLD_BIRD_SCIENTIFIC_NAME: &str = "RadMech";
pub const OLD_BIRD_HP_TEXT: &str = "∞";
pub const OLD_BIRD_HP_IS_INFINITE: bool = true;
pub const OLD_BIRD_EFFECTIVE_HEALTH: I32F32 = I32F32::lit("1000000000");
pub const OLD_BIRD_POWER_LEVEL: I32F32 = I32F32::lit("3");
pub const OLD_BIRD_MAX_SPAWNED: usize = 20;
pub const OLD_BIRD_ATTACK_DAMAGE_TEXT: &str = "30 - Instant Kill (Missile Attack); 30 (Stomp Attack); 70 (Squish Attack); 160 - Instant Kill (Grab Attack)";
pub const OLD_BIRD_DPS_TEXT: &str = "Varies by distance (Missile/Stomp/Squish Attack); 80 (Grab Attack)";
pub const OLD_BIRD_STUN_GRENADE: bool = true;
pub const OLD_BIRD_SHOCK_RESPONSE: &str = "Susceptible";
pub const OLD_BIRD_RADAR_PIP_SIZE: &str = "Large";
pub const OLD_BIRD_SHOVEL_HP_TEXT: &str = "∞";
pub const OLD_BIRD_INTERNAL_NAME: &str = "Radmech";

pub const OLD_BIRD_ROAM_SPEED: I32F32 = I32F32::lit("4");
pub const OLD_BIRD_CLOSE_CHASE_SPEED: I32F32 = I32F32::lit("8");
pub const OLD_BIRD_THRUSTER_RUSH_SPEED: I32F32 = I32F32::lit("16");
pub const OLD_BIRD_FLY_SPEED: I32F32 = I32F32::lit("20");
pub const OLD_BIRD_WATCH_RANGE: I32F32 = I32F32::lit("80");
pub const OLD_BIRD_FAR_RANGE: I32F32 = I32F32::lit("28");
pub const OLD_BIRD_CLOSE_RANGE: I32F32 = I32F32::lit("12");
pub const OLD_BIRD_EXTREME_CLOSE_RANGE: I32F32 = I32F32::lit("3");
pub const OLD_BIRD_MISSILE_RANGE: I32F32 = I32F32::lit("30");
pub const OLD_BIRD_MISSILE_BLAST_RANGE: I32F32 = I32F32::lit("5");
pub const OLD_BIRD_DIRECT_MISSILE_RANGE: I32F32 = I32F32::lit("1");
pub const OLD_BIRD_STOMP_RANGE: I32F32 = I32F32::lit("2");
pub const OLD_BIRD_SQUISH_RANGE: I32F32 = I32F32::lit("2");
pub const OLD_BIRD_GRAB_RANGE: I32F32 = I32F32::lit("1.5");
pub const OLD_BIRD_MISSILE_BLAST_DAMAGE: I32F32 = I32F32::lit("30");
pub const OLD_BIRD_STOMP_DAMAGE: I32F32 = I32F32::lit("30");
pub const OLD_BIRD_SQUISH_DAMAGE: I32F32 = I32F32::lit("70");
pub const OLD_BIRD_GRAB_DAMAGE: I32F32 = I32F32::lit("160");
pub const OLD_BIRD_ATTACK_SPEED_SECONDS: I32F32 = I32F32::lit("1");
pub const OLD_BIRD_GRAB_DPS: I32F32 = I32F32::lit("80");
pub const OLD_BIRD_STUN_SECONDS: I32F32 = I32F32::lit("5");
pub const OLD_BIRD_RADAR_BOOSTER_COOLDOWN_SECONDS: I32F32 = I32F32::lit("2.5");
pub const OLD_BIRD_ATTACK_PAUSE_SECONDS: I32F32 = I32F32::lit("1");
pub const OLD_BIRD_SEARCHLIGHT_ACTIVE: bool = true;

pub const OLD_BIRD_DEPENDS_ON: [&str; 7] = [
    "employee",
    "artifice",
    "radar_booster",
    "stun_grenade",
    "diy_flashbang",
    "tzp_inhalant",
    "curtained_door",
];

pub const OLD_BIRD_FRONTMATTER_BEHAVIOR: [&str; 3] = [
    "Roaming (Walking and Flying)",
    "Chasing (Sprinting and Flying)",
    "Attack (Missile, Stomping, Squishing, Grabbing)",
];

pub const OLD_BIRD_BEHAVIORAL_MECHANICS: [OldBirdBehaviorRule; 14] = [
    OldBirdBehaviorRule {
        condition: "the Old Bird is inactive",
        outcome: "it stays motionless outside, poses no threat, and only activates when selected for an entity spawn cycle",
    },
    OldBirdBehaviorRule {
        condition: "the Old Bird is roaming",
        outcome: "it walks on the ground, can fly to a new position, and its flight leaves black smoke that reveals its path",
    },
    OldBirdBehaviorRule {
        condition: "the Old Bird detects a target",
        outcome: "its searchlight turns on and it moves to the target's last seen location",
    },
    OldBirdBehaviorRule {
        condition: "the target is far away",
        outcome: "the Old Bird uses thrusters to rush toward the target until it reaches attack range",
    },
    OldBirdBehaviorRule {
        condition: "the target is close",
        outcome: "the Old Bird moves faster and may pause briefly before launching an attack",
    },
    OldBirdBehaviorRule {
        condition: "the target is extremely close",
        outcome: "the Old Bird engages in melee attacks instead of ranged attacks",
    },
    OldBirdBehaviorRule {
        condition: "a missile detonates directly under a target",
        outcome: "the missile attack can deal instant-kill damage, and the blast also inflicts 30 damage when it only reaches the blast radius",
    },
    OldBirdBehaviorRule {
        condition: "the Old Bird lands on a nearby target",
        outcome: "the stomp attack crushes the target for 30 damage",
    },
    OldBirdBehaviorRule {
        condition: "the Old Bird takes off and lands directly on a target",
        outcome: "the squish attack deals 70 damage",
    },
    OldBirdBehaviorRule {
        condition: "the Old Bird grabs an employee with its claw arm",
        outcome: "it torches the target to death and the grab attack deals 160 damage or instant-kill damage",
    },
    OldBirdBehaviorRule {
        condition: "the Old Bird is stunned by a stun_grenade or diy_flashbang",
        outcome: "the stun lasts about 5 seconds",
    },
    OldBirdBehaviorRule {
        condition: "an employee uses radar_booster pings at intervals of at least 5 seconds while watching an activated Old Bird from the terminal",
        outcome: "the Old Bird can be chained into repeated stuns because the booster cooldown is 2.5 seconds and the stun duration is about 5 seconds",
    },
    OldBirdBehaviorRule {
        condition: "an employee uses tzp_inhalant",
        outcome: "movement speed increases and missile evasion becomes easier",
    },
    OldBirdBehaviorRule {
        condition: "an active Old Bird is baited into a warehouse on artifice and sealed with a curtained_door",
        outcome: "the warehouse can contain the Old Bird",
    },
];

pub struct OldBirdPlugin;

impl Plugin for OldBirdPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnOldBirdEvent>()
            .add_event::<OldBirdStateChangedEvent>()
            .add_event::<OldBirdActivatedEvent>()
            .add_event::<OldBirdFlightSmokeEvent>()
            .add_event::<OldBirdSearchlightEvent>()
            .add_event::<OldBirdAttackPausedEvent>()
            .add_event::<OldBirdMissileAttackEvent>()
            .add_event::<OldBirdStompAttackEvent>()
            .add_event::<OldBirdSquishAttackEvent>()
            .add_event::<OldBirdGrabAttackEvent>()
            .add_event::<OldBirdInstantKillEvent>()
            .add_event::<OldBirdStunAppliedEvent>()
            .add_event::<OldBirdRadarBoosterPingEvent>()
            .add_event::<OldBirdRadarBoosterChainStunEvent>()
            .add_event::<OldBirdTzpEvasionEvent>()
            .add_event::<OldBirdContainedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    old_bird_spawn_from_events,
                    old_bird_activate_from_spawn_cycle,
                    old_bird_acquire_target,
                    old_bird_chase_target,
                    old_bird_attack_targets,
                    old_bird_apply_stuns,
                    old_bird_apply_radar_booster_pings,
                    old_bird_apply_artifice_containment,
                    old_bird_tick_timers,
                    old_bird_ignore_damage,
                    old_bird_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OldBirdBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OldBird {
    pub stable_id: u64,
    pub activated: bool,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OldBirdTargetSensor {
    pub stable_id: u64,
    pub is_employee: bool,
    pub is_alive: bool,
    pub visible_to_old_bird: bool,
    pub using_tzp_inhalant: bool,
    pub inside_company_cruiser: bool,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OldBirdContainmentSensor {
    pub on_artifice: bool,
    pub inside_warehouse: bool,
    pub curtained_door_sealed: bool,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct OldBirdTarget {
    pub has_target: bool,
    pub target_entity: Entity,
    pub target_stable_id: u64,
    pub last_seen_position: SimPosition,
    pub target_visible: bool,
}

impl Default for OldBirdTarget {
    fn default() -> Self {
        Self {
            has_target: false,
            target_entity: Entity::PLACEHOLDER,
            target_stable_id: 0,
            last_seen_position: SimPosition {
                x: I32F32::ZERO,
                y: I32F32::ZERO,
            },
            target_visible: false,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct OldBirdMovement {
    pub current_speed: I32F32,
    pub destination: SimPosition,
    pub flying: bool,
    pub smoke_trail_active: bool,
    pub attack_pause_ticks: u32,
    pub attack_cooldown_ticks: u32,
    pub stun_ticks: u32,
    pub searchlight_on: bool,
}

impl Default for OldBirdMovement {
    fn default() -> Self {
        Self {
            current_speed: I32F32::ZERO,
            destination: SimPosition {
                x: I32F32::ZERO,
                y: I32F32::ZERO,
            },
            flying: false,
            smoke_trail_active: false,
            attack_pause_ticks: 0,
            attack_cooldown_ticks: 0,
            stun_ticks: 0,
            searchlight_on: false,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OldBirdState {
    #[default]
    Inactive,
    Roaming,
    Chasing,
    AttackPaused,
    Stunned,
    Contained,
}

#[derive(Bundle)]
pub struct OldBirdBundle {
    pub name: Name,
    pub old_bird: OldBird,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub state: OldBirdState,
    pub target: OldBirdTarget,
    pub movement: OldBirdMovement,
    pub containment: OldBirdContainmentSensor,
}

impl OldBirdBundle {
    pub fn new(event: SpawnOldBirdEvent) -> Self {
        let state = if event.activated {
            OldBirdState::Roaming
        } else {
            OldBirdState::Inactive
        };

        let speed = if event.activated {
            OLD_BIRD_ROAM_SPEED
        } else {
            I32F32::ZERO
        };

        Self {
            name: Name::new(OLD_BIRD_NAME),
            old_bird: OldBird {
                stable_id: event.stable_id,
                activated: event.activated,
            },
            position: event.position,
            health: Health::full(OLD_BIRD_EFFECTIVE_HEALTH),
            stats: UnitStats {
                move_speed: speed,
                attack_range: OLD_BIRD_MISSILE_RANGE,
                attack_damage: OLD_BIRD_MISSILE_BLAST_DAMAGE,
                attack_speed: OLD_BIRD_ATTACK_SPEED_SECONDS,
                watch_range: OLD_BIRD_WATCH_RANGE,
            },
            state,
            target: OldBirdTarget {
                last_seen_position: event.position,
                ..Default::default()
            },
            movement: OldBirdMovement {
                current_speed: speed,
                destination: event.position,
                ..Default::default()
            },
            containment: OldBirdContainmentSensor::default(),
        }
    }
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpawnOldBirdEvent {
    pub stable_id: u64,
    pub position: SimPosition,
    pub activated: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct OldBirdStateChangedEvent {
    pub old_bird: Entity,
    pub from: OldBirdState,
    pub to: OldBirdState,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct OldBirdActivatedEvent {
    pub old_bird: Entity,
    pub stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct OldBirdFlightSmokeEvent {
    pub old_bird: Entity,
    pub position: SimPosition,
    pub active: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct OldBirdSearchlightEvent {
    pub old_bird: Entity,
    pub target: Entity,
    pub target_stable_id: u64,
    pub enabled: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct OldBirdAttackPausedEvent {
    pub old_bird: Entity,
    pub target: Entity,
    pub pause_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct OldBirdMissileAttackEvent {
    pub old_bird: Entity,
    pub target: Entity,
    pub target_stable_id: u64,
    pub direct_hit: bool,
    pub damage: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct OldBirdStompAttackEvent {
    pub old_bird: Entity,
    pub target: Entity,
    pub target_stable_id: u64,
    pub damage: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct OldBirdSquishAttackEvent {
    pub old_bird: Entity,
    pub target: Entity,
    pub target_stable_id: u64,
    pub damage: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct OldBirdGrabAttackEvent {
    pub old_bird: Entity,
    pub target: Entity,
    pub target_stable_id: u64,
    pub damage: I32F32,
    pub instant_kill: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct OldBirdInstantKillEvent {
    pub old_bird: Entity,
    pub target: Entity,
    pub target_stable_id: u64,
    pub cause_id: &'static str,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct OldBirdStunAppliedEvent {
    pub old_bird: Entity,
    pub source: Entity,
    pub source_id: &'static str,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct OldBirdRadarBoosterPingEvent {
    pub old_bird: Entity,
    pub booster: Entity,
    pub interval_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct OldBirdRadarBoosterChainStunEvent {
    pub old_bird: Entity,
    pub booster: Entity,
    pub stun_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct OldBirdTzpEvasionEvent {
    pub employee: Entity,
    pub employee_stable_id: u64,
    pub missile_evasion_easier: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct OldBirdContainedEvent {
    pub old_bird: Entity,
    pub on_artifice: bool,
    pub curtained_door_sealed: bool,
}

fn old_bird_spawn_from_events(
    mut commands: Commands,
    mut events: EventReader<SpawnOldBirdEvent>,
    birds: Query<(), With<OldBird>>,
) {
    let mut spawned_count = birds.iter().count();

    for event in events.read() {
        if spawned_count >= OLD_BIRD_MAX_SPAWNED {
            break;
        }

        commands.spawn(OldBirdBundle::new(*event));
        spawned_count += 1;
    }
}

fn old_bird_activate_from_spawn_cycle(
    mut activated_events: EventWriter<OldBirdActivatedEvent>,
    mut state_events: EventWriter<OldBirdStateChangedEvent>,
    mut birds: Query<(Entity, &mut OldBird, &mut OldBirdState, &mut UnitStats), With<OldBird>>,
) {
    for (entity, mut bird, mut state, mut stats) in birds.iter_mut() {
        if bird.activated || *state != OldBirdState::Inactive {
            continue;
        }

        bird.activated = true;
        stats.move_speed = OLD_BIRD_ROAM_SPEED;
        set_old_bird_state(entity, &mut state, OldBirdState::Roaming, &mut state_events);

        activated_events.send(OldBirdActivatedEvent {
            old_bird: entity,
            stable_id: bird.stable_id,
        });
    }
}

fn old_bird_acquire_target(
    mut state_events: EventWriter<OldBirdStateChangedEvent>,
    mut searchlight_events: EventWriter<OldBirdSearchlightEvent>,
    mut birds: Query<
        (
            Entity,
            &SimPosition,
            &OldBird,
            &mut OldBirdState,
            &mut OldBirdTarget,
            &mut OldBirdMovement,
            &mut UnitStats,
        ),
        With<OldBird>,
    >,
    targets: Query<(Entity, &SimPosition, &OldBirdTargetSensor), Without<OldBird>>,
) {
    for (bird_entity, bird_position, bird, mut state, mut target, mut movement, mut stats) in
        birds.iter_mut()
    {
        if !bird.activated || *state != OldBirdState::Roaming {
            continue;
        }

        let Some((target_entity, target_position, sensor)) =
            nearest_visible_target(*bird_position, OLD_BIRD_WATCH_RANGE, &targets)
        else {
            stats.move_speed = OLD_BIRD_ROAM_SPEED;
            movement.searchlight_on = false;
            continue;
        };

        target.has_target = true;
        target.target_entity = target_entity;
        target.target_stable_id = sensor.stable_id;
        target.last_seen_position = target_position;
        target.target_visible = true;
        movement.searchlight_on = OLD_BIRD_SEARCHLIGHT_ACTIVE;
        movement.destination = target_position;
        stats.move_speed = OLD_BIRD_THRUSTER_RUSH_SPEED;

        searchlight_events.send(OldBirdSearchlightEvent {
            old_bird: bird_entity,
            target: target_entity,
            target_stable_id: sensor.stable_id,
            enabled: true,
        });

        set_old_bird_state(
            bird_entity,
            &mut state,
            OldBirdState::Chasing,
            &mut state_events,
        );
    }
}

fn old_bird_chase_target(
    sim_hz: Res<SimHz>,
    mut smoke_events: EventWriter<OldBirdFlightSmokeEvent>,
    mut pause_events: EventWriter<OldBirdAttackPausedEvent>,
    mut state_events: EventWriter<OldBirdStateChangedEvent>,
    mut birds: Query<
        (
            Entity,
            &mut SimPosition,
            &mut OldBirdState,
            &mut OldBirdTarget,
            &mut OldBirdMovement,
            &mut UnitStats,
        ),
        With<OldBird>,
    >,
    targets: Query<(Entity, &SimPosition, &OldBirdTargetSensor), Without<OldBird>>,
) {
    for (bird_entity, mut bird_position, mut state, mut target, mut movement, mut stats) in
        birds.iter_mut()
    {
        if *state != OldBirdState::Chasing || !target.has_target {
            continue;
        }

        let Some((target_entity, target_position, sensor)) =
            target_by_stable_id(target.target_stable_id, &targets)
        else {
            target.has_target = false;
            movement.searchlight_on = false;
            stats.move_speed = OLD_BIRD_ROAM_SPEED;
            set_old_bird_state(
                bird_entity,
                &mut state,
                OldBirdState::Roaming,
                &mut state_events,
            );
            continue;
        };

        if !sensor.is_alive {
            target.has_target = false;
            movement.searchlight_on = false;
            stats.move_speed = OLD_BIRD_ROAM_SPEED;
            set_old_bird_state(
                bird_entity,
                &mut state,
                OldBirdState::Roaming,
                &mut state_events,
            );
            continue;
        }

        target.target_entity = target_entity;
        target.target_visible = sensor.visible_to_old_bird;
        if target.target_visible {
            target.last_seen_position = target_position;
        }

        let distance_sq = fixed_distance_sq(*bird_position, target.last_seen_position);

        if distance_sq > fixed_square(OLD_BIRD_FAR_RANGE) {
            stats.move_speed = OLD_BIRD_THRUSTER_RUSH_SPEED;
            movement.current_speed = OLD_BIRD_THRUSTER_RUSH_SPEED;
            movement.flying = true;
            movement.smoke_trail_active = true;
        } else if distance_sq <= fixed_square(OLD_BIRD_CLOSE_RANGE) {
            stats.move_speed = OLD_BIRD_CLOSE_CHASE_SPEED;
            movement.current_speed = OLD_BIRD_CLOSE_CHASE_SPEED;
            movement.flying = false;
            movement.smoke_trail_active = false;

            if movement.attack_pause_ticks == 0 && movement.attack_cooldown_ticks == 0 {
                movement.attack_pause_ticks =
                    fixed_seconds_to_ticks(OLD_BIRD_ATTACK_PAUSE_SECONDS, sim_hz.0);
                stats.move_speed = I32F32::ZERO;
                pause_events.send(OldBirdAttackPausedEvent {
                    old_bird: bird_entity,
                    target: target.target_entity,
                    pause_ticks: movement.attack_pause_ticks,
                });
                set_old_bird_state(
                    bird_entity,
                    &mut state,
                    OldBirdState::AttackPaused,
                    &mut state_events,
                );
                continue;
            }
        } else {
            stats.move_speed = OLD_BIRD_ROAM_SPEED;
            movement.current_speed = OLD_BIRD_ROAM_SPEED;
            movement.flying = true;
            movement.smoke_trail_active = true;
        }

        movement.destination = target.last_seen_position;
        move_axis_toward(
            &mut bird_position,
            movement.destination,
            stats.move_speed / sim_hz.0,
        );

        smoke_events.send(OldBirdFlightSmokeEvent {
            old_bird: bird_entity,
            position: *bird_position,
            active: movement.smoke_trail_active,
        });
    }
}

fn old_bird_attack_targets(
    sim_hz: Res<SimHz>,
    mut damage_events: EventWriter<IncomingDamageEvent>,
    mut missile_events: EventWriter<OldBirdMissileAttackEvent>,
    mut stomp_events: EventWriter<OldBirdStompAttackEvent>,
    mut squish_events: EventWriter<OldBirdSquishAttackEvent>,
    mut grab_events: EventWriter<OldBirdGrabAttackEvent>,
    mut instant_kill_events: EventWriter<OldBirdInstantKillEvent>,
    mut evasion_events: EventWriter<OldBirdTzpEvasionEvent>,
    mut state_events: EventWriter<OldBirdStateChangedEvent>,
    mut birds: Query<
        (
            Entity,
            &SimPosition,
            &mut OldBirdState,
            &mut OldBirdTarget,
            &mut OldBirdMovement,
            &mut UnitStats,
        ),
        With<OldBird>,
    >,
    targets: Query<(Entity, &SimPosition, &OldBirdTargetSensor), Without<OldBird>>,
) {
    for (bird_entity, bird_position, mut state, mut target, mut movement, mut stats) in
        birds.iter_mut()
    {
        if *state != OldBirdState::AttackPaused || !target.has_target {
            continue;
        }

        if movement.attack_pause_ticks > 0 {
            continue;
        }

        let Some((target_entity, target_position, sensor)) =
            target_by_stable_id(target.target_stable_id, &targets)
        else {
            target.has_target = false;
            set_old_bird_state(
                bird_entity,
                &mut state,
                OldBirdState::Roaming,
                &mut state_events,
            );
            continue;
        };

        if sensor.using_tzp_inhalant {
            evasion_events.send(OldBirdTzpEvasionEvent {
                employee: target_entity,
                employee_stable_id: sensor.stable_id,
                missile_evasion_easier: true,
            });
        }

        let distance_sq = fixed_distance_sq(*bird_position, target_position);

        if distance_sq <= fixed_square(OLD_BIRD_GRAB_RANGE) && sensor.is_employee {
            damage_events.send(IncomingDamageEvent {
                target: target_entity,
                raw_amount: OLD_BIRD_GRAB_DAMAGE,
                damage_type: DamageType::Standard,
                source: bird_entity,
            });
            grab_events.send(OldBirdGrabAttackEvent {
                old_bird: bird_entity,
                target: target_entity,
                target_stable_id: sensor.stable_id,
                damage: OLD_BIRD_GRAB_DAMAGE,
                instant_kill: true,
            });
            instant_kill_events.send(OldBirdInstantKillEvent {
                old_bird: bird_entity,
                target: target_entity,
                target_stable_id: sensor.stable_id,
                cause_id: OLD_BIRD_ID,
            });
        } else if distance_sq <= fixed_square(OLD_BIRD_STOMP_RANGE) {
            damage_events.send(IncomingDamageEvent {
                target: target_entity,
                raw_amount: OLD_BIRD_STOMP_DAMAGE,
                damage_type: DamageType::Standard,
                source: bird_entity,
            });
            stomp_events.send(OldBirdStompAttackEvent {
                old_bird: bird_entity,
                target: target_entity,
                target_stable_id: sensor.stable_id,
                damage: OLD_BIRD_STOMP_DAMAGE,
            });
        } else if distance_sq <= fixed_square(OLD_BIRD_SQUISH_RANGE) && movement.flying {
            damage_events.send(IncomingDamageEvent {
                target: target_entity,
                raw_amount: OLD_BIRD_SQUISH_DAMAGE,
                damage_type: DamageType::Standard,
                source: bird_entity,
            });
            squish_events.send(OldBirdSquishAttackEvent {
                old_bird: bird_entity,
                target: target_entity,
                target_stable_id: sensor.stable_id,
                damage: OLD_BIRD_SQUISH_DAMAGE,
            });
        } else if distance_sq <= fixed_square(OLD_BIRD_MISSILE_BLAST_RANGE) {
            let direct_hit = distance_sq <= fixed_square(OLD_BIRD_DIRECT_MISSILE_RANGE);
            damage_events.send(IncomingDamageEvent {
                target: target_entity,
                raw_amount: OLD_BIRD_MISSILE_BLAST_DAMAGE,
                damage_type: DamageType::Standard,
                source: bird_entity,
            });
            missile_events.send(OldBirdMissileAttackEvent {
                old_bird: bird_entity,
                target: target_entity,
                target_stable_id: sensor.stable_id,
                direct_hit,
                damage: OLD_BIRD_MISSILE_BLAST_DAMAGE,
            });

            if direct_hit {
                instant_kill_events.send(OldBirdInstantKillEvent {
                    old_bird: bird_entity,
                    target: target_entity,
                    target_stable_id: sensor.stable_id,
                    cause_id: OLD_BIRD_ID,
                });
            }
        }

        movement.attack_cooldown_ticks =
            fixed_seconds_to_ticks(OLD_BIRD_ATTACK_SPEED_SECONDS, sim_hz.0);
        stats.move_speed = OLD_BIRD_CLOSE_CHASE_SPEED;
        set_old_bird_state(
            bird_entity,
            &mut state,
            OldBirdState::Chasing,
            &mut state_events,
        );
    }
}

fn old_bird_apply_stuns(
    sim_hz: Res<SimHz>,
    mut stun_events: EventReader<OldBirdStunAppliedEvent>,
    mut state_events: EventWriter<OldBirdStateChangedEvent>,
    mut birds: Query<(Entity, &mut OldBirdState, &mut OldBirdMovement, &mut UnitStats), With<OldBird>>,
) {
    for event in stun_events.read() {
        if event.source_id != "stun_grenade" && event.source_id != "diy_flashbang" {
            continue;
        }

        let Ok((entity, mut state, mut movement, mut stats)) = birds.get_mut(event.old_bird) else {
            continue;
        };

        movement.stun_ticks = fixed_seconds_to_ticks(OLD_BIRD_STUN_SECONDS, sim_hz.0);
        stats.move_speed = I32F32::ZERO;
        set_old_bird_state(entity, &mut state, OldBirdState::Stunned, &mut state_events);
    }
}

fn old_bird_apply_radar_booster_pings(
    sim_hz: Res<SimHz>,
    mut ping_events: EventReader<OldBirdRadarBoosterPingEvent>,
    mut chain_events: EventWriter<OldBirdRadarBoosterChainStunEvent>,
    mut stun_events: EventWriter<OldBirdStunAppliedEvent>,
    birds: Query<(), With<OldBird>>,
) {
    let minimum_interval = fixed_seconds_to_ticks(OLD_BIRD_STUN_SECONDS, sim_hz.0);

    for event in ping_events.read() {
        if birds.get(event.old_bird).is_err() || event.interval_ticks < minimum_interval {
            continue;
        }

        let stun_ticks = fixed_seconds_to_ticks(OLD_BIRD_STUN_SECONDS, sim_hz.0);
        chain_events.send(OldBirdRadarBoosterChainStunEvent {
            old_bird: event.old_bird,
            booster: event.booster,
            stun_ticks,
        });
        stun_events.send(OldBirdStunAppliedEvent {
            old_bird: event.old_bird,
            source: event.booster,
            source_id: "radar_booster",
        });
    }
}

fn old_bird_apply_artifice_containment(
    mut contained_events: EventWriter<OldBirdContainedEvent>,
    mut state_events: EventWriter<OldBirdStateChangedEvent>,
    mut birds: Query<
        (
            Entity,
            &OldBirdContainmentSensor,
            &mut OldBirdState,
            &mut OldBirdMovement,
            &mut UnitStats,
        ),
        With<OldBird>,
    >,
) {
    for (entity, containment, mut state, mut movement, mut stats) in birds.iter_mut() {
        if !containment.on_artifice
            || !containment.inside_warehouse
            || !containment.curtained_door_sealed
            || *state == OldBirdState::Contained
        {
            continue;
        }

        movement.destination = SimPosition {
            x: I32F32::ZERO,
            y: I32F32::ZERO,
        };
        movement.searchlight_on = false;
        movement.smoke_trail_active = false;
        stats.move_speed = I32F32::ZERO;

        set_old_bird_state(entity, &mut state, OldBirdState::Contained, &mut state_events);
        contained_events.send(OldBirdContainedEvent {
            old_bird: entity,
            on_artifice: true,
            curtained_door_sealed: true,
        });
    }
}

fn old_bird_tick_timers(
    mut state_events: EventWriter<OldBirdStateChangedEvent>,
    mut birds: Query<(Entity, &mut OldBirdState, &mut OldBirdMovement, &mut UnitStats), With<OldBird>>,
) {
    for (entity, mut state, mut movement, mut stats) in birds.iter_mut() {
        if movement.attack_pause_ticks > 0 {
            movement.attack_pause_ticks -= 1;
        }

        if movement.attack_cooldown_ticks > 0 {
            movement.attack_cooldown_ticks -= 1;
        }

        if *state == OldBirdState::Stunned {
            if movement.stun_ticks > 0 {
                movement.stun_ticks -= 1;
                continue;
            }

            stats.move_speed = OLD_BIRD_ROAM_SPEED;
            set_old_bird_state(entity, &mut state, OldBirdState::Roaming, &mut state_events);
        }
    }
}

fn old_bird_ignore_damage(
    mut damage_events: EventReader<IncomingDamageEvent>,
    mut birds: Query<&mut Health, With<OldBird>>,
) {
    for event in damage_events.read() {
        let Ok(mut health) = birds.get_mut(event.target) else {
            continue;
        };

        if OLD_BIRD_HP_IS_INFINITE {
            health.current = OLD_BIRD_EFFECTIVE_HEALTH;
            health.max = OLD_BIRD_EFFECTIVE_HEALTH;
        }
    }
}

fn old_bird_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    sim_hz: Res<SimHz>,
    birds: Query<
        (
            &OldBird,
            &SimPosition,
            &Health,
            &UnitStats,
            &OldBirdState,
            &OldBirdTarget,
            &OldBirdMovement,
            &OldBirdContainmentSensor,
        ),
        With<OldBird>,
    >,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(sim_hz.0.to_bits() as u64);
    checksum.accumulate(OLD_BIRD_SOURCE_REVISION as u64);
    checksum.accumulate(OLD_BIRD_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(OLD_BIRD_HP_IS_INFINITE as u64);
    checksum.accumulate(OLD_BIRD_EFFECTIVE_HEALTH.to_bits() as u64);
    checksum.accumulate(OLD_BIRD_POWER_LEVEL.to_bits() as u64);
    checksum.accumulate(OLD_BIRD_MAX_SPAWNED as u64);
    checksum.accumulate(OLD_BIRD_STUN_GRENADE as u64);
    checksum.accumulate(OLD_BIRD_ROAM_SPEED.to_bits() as u64);
    checksum.accumulate(OLD_BIRD_CLOSE_CHASE_SPEED.to_bits() as u64);
    checksum.accumulate(OLD_BIRD_THRUSTER_RUSH_SPEED.to_bits() as u64);
    checksum.accumulate(OLD_BIRD_FLY_SPEED.to_bits() as u64);
    checksum.accumulate(OLD_BIRD_WATCH_RANGE.to_bits() as u64);
    checksum.accumulate(OLD_BIRD_FAR_RANGE.to_bits() as u64);
    checksum.accumulate(OLD_BIRD_CLOSE_RANGE.to_bits() as u64);
    checksum.accumulate(OLD_BIRD_EXTREME_CLOSE_RANGE.to_bits() as u64);
    checksum.accumulate(OLD_BIRD_MISSILE_RANGE.to_bits() as u64);
    checksum.accumulate(OLD_BIRD_MISSILE_BLAST_RANGE.to_bits() as u64);
    checksum.accumulate(OLD_BIRD_DIRECT_MISSILE_RANGE.to_bits() as u64);
    checksum.accumulate(OLD_BIRD_STOMP_RANGE.to_bits() as u64);
    checksum.accumulate(OLD_BIRD_SQUISH_RANGE.to_bits() as u64);
    checksum.accumulate(OLD_BIRD_GRAB_RANGE.to_bits() as u64);
    checksum.accumulate(OLD_BIRD_MISSILE_BLAST_DAMAGE.to_bits() as u64);
    checksum.accumulate(OLD_BIRD_STOMP_DAMAGE.to_bits() as u64);
    checksum.accumulate(OLD_BIRD_SQUISH_DAMAGE.to_bits() as u64);
    checksum.accumulate(OLD_BIRD_GRAB_DAMAGE.to_bits() as u64);
    checksum.accumulate(OLD_BIRD_ATTACK_SPEED_SECONDS.to_bits() as u64);
    checksum.accumulate(OLD_BIRD_GRAB_DPS.to_bits() as u64);
    checksum.accumulate(OLD_BIRD_STUN_SECONDS.to_bits() as u64);
    checksum.accumulate(OLD_BIRD_RADAR_BOOSTER_COOLDOWN_SECONDS.to_bits() as u64);
    checksum.accumulate(OLD_BIRD_ATTACK_PAUSE_SECONDS.to_bits() as u64);

    accumulate_str(&mut checksum, 0x1000, OLD_BIRD_ID);
    accumulate_str(&mut checksum, 0x1001, OLD_BIRD_NAME);
    accumulate_str(&mut checksum, 0x1002, OLD_BIRD_TYPE);
    accumulate_str(&mut checksum, 0x1003, OLD_BIRD_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, OLD_BIRD_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1005, OLD_BIRD_EXTRACTED_AT);
    accumulate_str(&mut checksum, 0x1006, OLD_BIRD_DWELLS);
    accumulate_str(&mut checksum, 0x1007, OLD_BIRD_SCIENTIFIC_NAME);
    accumulate_str(&mut checksum, 0x1008, OLD_BIRD_HP_TEXT);
    accumulate_str(&mut checksum, 0x1009, OLD_BIRD_ATTACK_DAMAGE_TEXT);
    accumulate_str(&mut checksum, 0x100A, OLD_BIRD_DPS_TEXT);
    accumulate_str(&mut checksum, 0x100B, OLD_BIRD_SHOCK_RESPONSE);
    accumulate_str(&mut checksum, 0x100C, OLD_BIRD_RADAR_PIP_SIZE);
    accumulate_str(&mut checksum, 0x100D, OLD_BIRD_SHOVEL_HP_TEXT);
    accumulate_str(&mut checksum, 0x100E, OLD_BIRD_INTERNAL_NAME);

    for dependency in OLD_BIRD_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for behavior in OLD_BIRD_FRONTMATTER_BEHAVIOR {
        accumulate_str(&mut checksum, 0x3000, behavior);
    }

    for rule in OLD_BIRD_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x4000, rule.condition);
        accumulate_str(&mut checksum, 0x4001, rule.outcome);
    }

    for (bird, position, health, stats, state, target, movement, containment) in birds.iter() {
        checksum.accumulate(bird.stable_id);
        checksum.accumulate(bird.activated as u64);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);
        checksum.accumulate(old_bird_state_bits(*state));
        checksum.accumulate(target.has_target as u64);
        checksum.accumulate(target.target_stable_id);
        checksum.accumulate(target.last_seen_position.x.to_bits() as u64);
        checksum.accumulate(target.last_seen_position.y.to_bits() as u64);
        checksum.accumulate(target.target_visible as u64);
        checksum.accumulate(movement.current_speed.to_bits() as u64);
        checksum.accumulate(movement.destination.x.to_bits() as u64);
        checksum.accumulate(movement.destination.y.to_bits() as u64);
        checksum.accumulate(movement.flying as u64);
        checksum.accumulate(movement.smoke_trail_active as u64);
        checksum.accumulate(movement.attack_pause_ticks as u64);
        checksum.accumulate(movement.attack_cooldown_ticks as u64);
        checksum.accumulate(movement.stun_ticks as u64);
        checksum.accumulate(movement.searchlight_on as u64);
        checksum.accumulate(containment.on_artifice as u64);
        checksum.accumulate(containment.inside_warehouse as u64);
        checksum.accumulate(containment.curtained_door_sealed as u64);
    }
}

fn nearest_visible_target(
    bird_position: SimPosition,
    range: I32F32,
    targets: &Query<(Entity, &SimPosition, &OldBirdTargetSensor), Without<OldBird>>,
) -> Option<(Entity, SimPosition, OldBirdTargetSensor)> {
    let mut best: Option<(Entity, SimPosition, OldBirdTargetSensor, I32F32)> = None;
    let range_sq = fixed_square(range);

    for (entity, position, sensor) in targets.iter() {
        if !sensor.is_alive || !sensor.visible_to_old_bird || sensor.inside_company_cruiser {
            continue;
        }

        let distance_sq = fixed_distance_sq(bird_position, *position);
        if distance_sq > range_sq {
            continue;
        }

        match best {
            Some((_best_entity, _best_position, _best_sensor, best_distance_sq))
                if distance_sq >= best_distance_sq => {}
            _ => {
                best = Some((entity, *position, *sensor, distance_sq));
            }
        }
    }

    best.map(|(entity, position, sensor, _distance_sq)| (entity, position, sensor))
}

fn target_by_stable_id(
    stable_id: u64,
    targets: &Query<(Entity, &SimPosition, &OldBirdTargetSensor), Without<OldBird>>,
) -> Option<(Entity, SimPosition, OldBirdTargetSensor)> {
    for (entity, position, sensor) in targets.iter() {
        if sensor.stable_id == stable_id {
            return Some((entity, *position, *sensor));
        }
    }

    None
}

fn set_old_bird_state(
    old_bird: Entity,
    state: &mut OldBirdState,
    next: OldBirdState,
    events: &mut EventWriter<OldBirdStateChangedEvent>,
) {
    if *state == next {
        return;
    }

    let previous = *state;
    *state = next;
    events.send(OldBirdStateChangedEvent {
        old_bird,
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

fn fixed_seconds_to_ticks(seconds: I32F32, sim_hz: I32F32) -> u32 {
    let ticks = seconds * sim_hz;
    if ticks <= I32F32::ZERO {
        0
    } else {
        ticks.ceil().to_num::<u32>()
    }
}

fn old_bird_state_bits(state: OldBirdState) -> u64 {
    match state {
        OldBirdState::Inactive => 0,
        OldBirdState::Roaming => 1,
        OldBirdState::Chasing => 2,
        OldBirdState::AttackPaused => 3,
        OldBirdState::Stunned => 4,
        OldBirdState::Contained => 5,
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}
// Sources: vault/outdoor_entity_pages/cadaver_bloom.md, vault/entity_pages/entity.md
use bevy::prelude::*;
use fixed::types::I32F32;
use rand_core::RngCore;

use crate::gameplay_mechanics::employee::EmployeeBodySpawnedEvent;
use crate::indoor_entity_pages::cadaver::CadaverBloomBurstRequestEvent;
use crate::sim::{
    tick_rng, DamageType, GameSeed, Health, IncomingDamageEvent, NoiseEmittedEvent,
    SimChecksumState, SimHz, SimPosition, SimTick, UnitStats,
};

pub const CADAVER_BLOOM_ID: &str = "cadaver_bloom";
pub const CADAVER_BLOOM_NAME: &str = "Cadaver Bloom";
pub const CADAVER_BLOOM_TYPE: &str = "outdoor_entity_pages";
pub const CADAVER_BLOOM_SUBTYPE: &str = "hostile_plant_entity";
pub const CADAVER_BLOOM_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Cadaver_Bloom";
pub const CADAVER_BLOOM_SOURCE_REVISION: u32 = 21456;
pub const CADAVER_BLOOM_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const CADAVER_BLOOM_CONFIDENCE_BASIS_POINTS: u16 = 87;

pub const CADAVER_BLOOM_DWELLS: &str = "Both";
pub const CADAVER_BLOOM_DANGER: &str = "100%";
pub const CADAVER_BLOOM_SCIENTIFIC_NAME: &str = "N/A";
pub const CADAVER_BLOOM_HP: I32F32 = I32F32::lit("4");
pub const CADAVER_BLOOM_POWER_LEVEL: I32F32 = I32F32::lit("3");
pub const CADAVER_BLOOM_MAX_SPAWNED: usize = 8;
pub const CADAVER_BLOOM_ATTACK_DAMAGE: I32F32 = I32F32::lit("30");
pub const CADAVER_BLOOM_STUN_MULTIPLIER: I32F32 = I32F32::lit("0.25");
pub const CADAVER_BLOOM_CAN_SEE_THROUGH_FOG: bool = false;
pub const CADAVER_BLOOM_DOOR_SPEED_MULTIPLIER: I32F32 = I32F32::lit("1.25");
pub const CADAVER_BLOOM_INTERNAL_NAME: &str = "Cadaver Bloom";
pub const CADAVER_BLOOM_PIP_SIZE: &str = "Medium (Employee Pip)";

pub const CADAVER_BLOOM_ROAM_SPEED: I32F32 = I32F32::lit("5");
pub const CADAVER_BLOOM_ROAM_SCAN_RANGE: I32F32 = I32F32::lit("80");
pub const CADAVER_BLOOM_CHARGE_SPEED: I32F32 = I32F32::lit("18");
pub const CADAVER_BLOOM_CHARGE_DISTANCE: I32F32 = I32F32::lit("18");
pub const CADAVER_BLOOM_ERRATIC_MIN_DISTANCE: I32F32 = I32F32::lit("6");
pub const CADAVER_BLOOM_ERRATIC_MAX_DISTANCE: I32F32 = I32F32::lit("18");
pub const CADAVER_BLOOM_ERRATIC_MIN_SPEED: I32F32 = I32F32::lit("7");
pub const CADAVER_BLOOM_ERRATIC_MAX_SPEED: I32F32 = I32F32::lit("18");
pub const CADAVER_BLOOM_ERRATIC_MIN_SECONDS: I32F32 = I32F32::lit("0.1");
pub const CADAVER_BLOOM_ERRATIC_MAX_SECONDS: I32F32 = I32F32::lit("0.3");
pub const CADAVER_BLOOM_CLOSE_RANGE: I32F32 = I32F32::lit("3.6");
pub const CADAVER_BLOOM_CLOSE_SPEED: I32F32 = I32F32::lit("7");
pub const CADAVER_BLOOM_SEARCH_RADIUS: I32F32 = I32F32::lit("18");
pub const CADAVER_BLOOM_SEARCH_SPEED: I32F32 = I32F32::lit("9");
pub const CADAVER_BLOOM_SEARCH_SECONDS: I32F32 = I32F32::lit("9");
pub const CADAVER_BLOOM_LOST_NOISE_RANGE: I32F32 = I32F32::lit("15");
pub const CADAVER_BLOOM_NOISE_TURN_SECONDS: I32F32 = I32F32::lit("1.5");
pub const CADAVER_BLOOM_HIT_STUN_SECONDS: I32F32 = I32F32::lit("0.75");
pub const CADAVER_BLOOM_NORMAL_SCAN_RANGE: I32F32 = I32F32::lit("18");
pub const CADAVER_BLOOM_FOG_SCAN_RANGE: I32F32 = I32F32::lit("8");
pub const CADAVER_BLOOM_ATTACK_RANGE: I32F32 = I32F32::lit("1");
pub const CADAVER_BLOOM_ATTACK_SPEED_SECONDS: I32F32 = I32F32::lit("1");
pub const CADAVER_BLOOM_WEAPON_HIT_DAMAGE: I32F32 = I32F32::lit("1");
pub const CADAVER_BLOOM_BODY_CAUSE_ID: &str = "cadaver_bloom";
pub const CADAVER_BLOOM_ERRATIC_SPEED_SALT: u64 = 0xCA_DA_B1_00_0000_0001;
pub const CADAVER_BLOOM_ERRATIC_TIMER_SALT: u64 = 0xCA_DA_B1_00_0000_0002;
pub const CADAVER_BLOOM_PERPENDICULAR_SALT: u64 = 0xCA_DA_B1_00_0000_0003;

pub const CADAVER_BLOOM_DEPENDS_ON: [&str; 7] = [
    "lethal_company",
    "cadaver",
    "masked",
    "maneater",
    "shovel",
    "kitchen_knife",
    "double_barrel",
];

pub const CADAVER_BLOOM_FRONTMATTER_BEHAVIOR: [&str; 2] = ["Roam", "Chase"];

pub const CADAVER_BLOOM_BEHAVIORAL_MECHANICS: [CadaverBloomBehaviorRule; 12] = [
    CadaverBloomBehaviorRule {
        condition: "it is in Roam state",
        outcome: "it wanders at 5 units/sec while searching for employees within an 80 unit radius",
    },
    CadaverBloomBehaviorRule {
        condition: "it spots an employee",
        outcome: "it switches from Roam to Chase",
    },
    CadaverBloomBehaviorRule {
        condition: "the target is beyond 18 units or behind an obstacle",
        outcome: "it charges toward the employee at high speed",
    },
    CadaverBloomBehaviorRule {
        condition: "the target is between 6 units and 18 units away",
        outcome: "it moves erratically and randomizes speed between 7 units/sec and 18 units/sec every 0.1 seconds to 0.3 seconds",
    },
    CadaverBloomBehaviorRule {
        condition: "the target is between 6 units and 18 units away",
        outcome: "it selects random perpendicular destinations relative to the employee",
    },
    CadaverBloomBehaviorRule {
        condition: "the target is within 3.6 units",
        outcome: "it slows to 7 units/sec and moves directly toward the employee",
    },
    CadaverBloomBehaviorRule {
        condition: "it loses the employee",
        outcome: "it moves to the last known position and searches within an 18 unit radius at 9 units/sec for 9 seconds before returning to Roam",
    },
    CadaverBloomBehaviorRule {
        condition: "it has lost the employee it was chasing",
        outcome: "it detects noise only within 15 units; when noise is detected, it stops and turns toward the sound source for 1.5 seconds before resuming",
    },
    CadaverBloomBehaviorRule {
        condition: "it is hit",
        outcome: "it is stunned for 0.75 seconds and turns toward the employee who hit it",
    },
    CadaverBloomBehaviorRule {
        condition: "the weather is foggy",
        outcome: "its scan range drops from 18 units to 8 units",
    },
    CadaverBloomBehaviorRule {
        condition: "it dies",
        outcome: "it spawns the body of the employee it bloomed from",
    },
    CadaverBloomBehaviorRule {
        condition: "an employee is transformed by this entity",
        outcome: "the ship monitor shows the employee and the target can be teleported back to the ship",
    },
];

pub struct CadaverBloomPlugin;

impl Plugin for CadaverBloomPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnCadaverBloomEvent>()
            .add_event::<CadaverBloomStateChangedEvent>()
            .add_event::<CadaverBloomErraticSpeedChangedEvent>()
            .add_event::<CadaverBloomPerpendicularDestinationSelectedEvent>()
            .add_event::<CadaverBloomSearchStartedEvent>()
            .add_event::<CadaverBloomNoiseInvestigatedEvent>()
            .add_event::<CadaverBloomStunAppliedEvent>()
            .add_event::<CadaverBloomStunAdjustedEvent>()
            .add_event::<CadaverBloomFogScanRangeChangedEvent>()
            .add_event::<CadaverBloomBodySpawnRequestedEvent>()
            .add_event::<CadaverBloomRadarPipEvent>()
            .add_event::<CadaverBloomTeleportEligibleEvent>()
            .add_event::<CadaverBloomDoorAttemptEvent>()
            .add_event::<CadaverBloomDoorAttemptResolvedEvent>()
            .add_event::<CadaverBloomDamageTakenEvent>()
            .add_event::<CadaverBloomDefeatedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    cadaver_bloom_spawn_from_events,
                    cadaver_bloom_spawn_from_cadaver_requests,
                    cadaver_bloom_update_fog_scan_range,
                    cadaver_bloom_acquire_target,
                    cadaver_bloom_chase_target,
                    cadaver_bloom_search_after_lost_target,
                    cadaver_bloom_investigate_lost_target_noise,
                    cadaver_bloom_attack_close_target,
                    cadaver_bloom_apply_stun,
                    cadaver_bloom_door_attempt_speed,
                    cadaver_bloom_take_damage,
                    cadaver_bloom_spawn_body_on_death,
                    cadaver_bloom_report_radar_and_teleport,
                    cadaver_bloom_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CadaverBloomBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CadaverBloom {
    pub stable_id: u64,
    pub bloomed_employee: Entity,
    pub bloomed_employee_stable_id: u64,
}

impl Default for CadaverBloom {
    fn default() -> Self {
        Self {
            stable_id: 0,
            bloomed_employee: Entity::PLACEHOLDER,
            bloomed_employee_stable_id: 0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CadaverBloomEmployeeSensor {
    pub stable_id: u64,
    pub is_alive: bool,
    pub visible_to_bloom: bool,
    pub behind_obstacle: bool,
    pub transformed_by_bloom: bool,
    pub teleported_back_to_ship: bool,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CadaverBloomWeatherSensor {
    pub foggy: bool,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CadaverBloomTarget {
    pub has_target: bool,
    pub target_entity: Entity,
    pub target_stable_id: u64,
    pub last_known_position: SimPosition,
    pub target_visible: bool,
    pub lost_target: bool,
}

impl Default for CadaverBloomTarget {
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
            lost_target: false,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CadaverBloomMovement {
    pub current_speed: I32F32,
    pub scan_range: I32F32,
    pub destination: SimPosition,
    pub erratic_timer_ticks: u32,
    pub search_timer_ticks: u32,
    pub noise_turn_timer_ticks: u32,
    pub stun_timer_ticks: u32,
}

impl Default for CadaverBloomMovement {
    fn default() -> Self {
        Self {
            current_speed: CADAVER_BLOOM_ROAM_SPEED,
            scan_range: CADAVER_BLOOM_NORMAL_SCAN_RANGE,
            destination: SimPosition {
                x: I32F32::ZERO,
                y: I32F32::ZERO,
            },
            erratic_timer_ticks: 0,
            search_timer_ticks: 0,
            noise_turn_timer_ticks: 0,
            stun_timer_ticks: 0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CadaverBloomState {
    #[default]
    Roam,
    Chase,
    Search,
    NoiseTurn,
    Stunned,
    Dead,
}

#[derive(Bundle)]
pub struct CadaverBloomBundle {
    pub name: Name,
    pub bloom: CadaverBloom,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub state: CadaverBloomState,
    pub target: CadaverBloomTarget,
    pub movement: CadaverBloomMovement,
    pub weather: CadaverBloomWeatherSensor,
}

impl CadaverBloomBundle {
    pub fn new(event: SpawnCadaverBloomEvent) -> Self {
        Self {
            name: Name::new(CADAVER_BLOOM_NAME),
            bloom: CadaverBloom {
                stable_id: event.stable_id,
                bloomed_employee: event.bloomed_employee,
                bloomed_employee_stable_id: event.bloomed_employee_stable_id,
            },
            position: event.position,
            health: Health::full(CADAVER_BLOOM_HP),
            stats: UnitStats {
                move_speed: CADAVER_BLOOM_ROAM_SPEED,
                attack_range: CADAVER_BLOOM_ATTACK_RANGE,
                attack_damage: CADAVER_BLOOM_ATTACK_DAMAGE,
                attack_speed: CADAVER_BLOOM_ATTACK_SPEED_SECONDS,
                watch_range: CADAVER_BLOOM_ROAM_SCAN_RANGE,
            },
            state: CadaverBloomState::Roam,
            target: CadaverBloomTarget {
                last_known_position: event.position,
                ..Default::default()
            },
            movement: CadaverBloomMovement {
                current_speed: CADAVER_BLOOM_ROAM_SPEED,
                scan_range: CADAVER_BLOOM_NORMAL_SCAN_RANGE,
                destination: event.position,
                ..Default::default()
            },
            weather: CadaverBloomWeatherSensor {
                foggy: event.weather_foggy,
            },
        }
    }
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpawnCadaverBloomEvent {
    pub stable_id: u64,
    pub position: SimPosition,
    pub bloomed_employee: Entity,
    pub bloomed_employee_stable_id: u64,
    pub weather_foggy: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CadaverBloomStateChangedEvent {
    pub bloom: Entity,
    pub from: CadaverBloomState,
    pub to: CadaverBloomState,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CadaverBloomErraticSpeedChangedEvent {
    pub bloom: Entity,
    pub target_stable_id: u64,
    pub speed: I32F32,
    pub next_change_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CadaverBloomPerpendicularDestinationSelectedEvent {
    pub bloom: Entity,
    pub target_stable_id: u64,
    pub destination: SimPosition,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CadaverBloomSearchStartedEvent {
    pub bloom: Entity,
    pub target_stable_id: u64,
    pub last_known_position: SimPosition,
    pub search_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CadaverBloomNoiseInvestigatedEvent {
    pub bloom: Entity,
    pub noise_source: Entity,
    pub sound_position: SimPosition,
    pub turn_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CadaverBloomStunAppliedEvent {
    pub bloom: Entity,
    pub source: Entity,
    pub base_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CadaverBloomStunAdjustedEvent {
    pub bloom: Entity,
    pub source: Entity,
    pub adjusted_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CadaverBloomFogScanRangeChangedEvent {
    pub bloom: Entity,
    pub foggy: bool,
    pub scan_range: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CadaverBloomBodySpawnRequestedEvent {
    pub bloom: Entity,
    pub employee: Entity,
    pub employee_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CadaverBloomRadarPipEvent {
    pub bloom: Entity,
    pub employee_stable_id: u64,
    pub employee_like_pip: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CadaverBloomTeleportEligibleEvent {
    pub bloom: Entity,
    pub employee: Entity,
    pub employee_stable_id: u64,
    pub can_teleport_back_to_ship: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CadaverBloomDoorAttemptEvent {
    pub bloom: Entity,
    pub door: Entity,
    pub base_open_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CadaverBloomDoorAttemptResolvedEvent {
    pub bloom: Entity,
    pub door: Entity,
    pub adjusted_open_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CadaverBloomDamageTakenEvent {
    pub bloom: Entity,
    pub source: Entity,
    pub damage: I32F32,
    pub remaining_health: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CadaverBloomDefeatedEvent {
    pub bloom: Entity,
    pub source: Entity,
    pub bloomed_employee: Entity,
    pub bloomed_employee_stable_id: u64,
}

fn cadaver_bloom_spawn_from_events(
    mut commands: Commands,
    mut events: EventReader<SpawnCadaverBloomEvent>,
    blooms: Query<(), With<CadaverBloom>>,
) {
    let mut spawned_count = blooms.iter().count();

    for event in events.read() {
        if spawned_count >= CADAVER_BLOOM_MAX_SPAWNED {
            break;
        }

        commands.spawn(CadaverBloomBundle::new(*event));
        spawned_count += 1;
    }
}

fn cadaver_bloom_spawn_from_cadaver_requests(
    mut spawn_events: EventWriter<SpawnCadaverBloomEvent>,
    mut requests: EventReader<CadaverBloomBurstRequestEvent>,
    employees: Query<&SimPosition>,
) {
    for request in requests.read() {
        if request.bloom_id != CADAVER_BLOOM_ID {
            continue;
        }

        let Ok(position) = employees.get(request.employee) else {
            continue;
        };

        spawn_events.send(SpawnCadaverBloomEvent {
            stable_id: request.employee_stable_id ^ 0xCA_DA_B1_00_0000_0000,
            position: *position,
            bloomed_employee: request.employee,
            bloomed_employee_stable_id: request.employee_stable_id,
            weather_foggy: false,
        });
    }
}

fn cadaver_bloom_update_fog_scan_range(
    mut events: EventWriter<CadaverBloomFogScanRangeChangedEvent>,
    mut blooms: Query<
        (
            Entity,
            &CadaverBloomWeatherSensor,
            &mut CadaverBloomMovement,
            &mut UnitStats,
        ),
        With<CadaverBloom>,
    >,
) {
    for (entity, weather, mut movement, mut stats) in blooms.iter_mut() {
        let next_range = if weather.foggy {
            CADAVER_BLOOM_FOG_SCAN_RANGE
        } else {
            CADAVER_BLOOM_NORMAL_SCAN_RANGE
        };

        if movement.scan_range == next_range {
            continue;
        }

        movement.scan_range = next_range;
        stats.watch_range = if weather.foggy {
            CADAVER_BLOOM_FOG_SCAN_RANGE
        } else {
            CADAVER_BLOOM_ROAM_SCAN_RANGE
        };

        events.send(CadaverBloomFogScanRangeChangedEvent {
            bloom: entity,
            foggy: weather.foggy,
            scan_range: next_range,
        });
    }
}

fn cadaver_bloom_acquire_target(
    mut state_events: EventWriter<CadaverBloomStateChangedEvent>,
    mut blooms: Query<
        (
            Entity,
            &SimPosition,
            &mut CadaverBloomState,
            &mut CadaverBloomTarget,
            &mut UnitStats,
        ),
        With<CadaverBloom>,
    >,
    employees: Query<(Entity, &SimPosition, &CadaverBloomEmployeeSensor), Without<CadaverBloom>>,
) {
    for (bloom_entity, bloom_position, mut state, mut target, mut stats) in blooms.iter_mut() {
        if *state != CadaverBloomState::Roam {
            continue;
        }

        let Some((employee_entity, employee_position, sensor)) =
            nearest_visible_employee(*bloom_position, CADAVER_BLOOM_ROAM_SCAN_RANGE, &employees)
        else {
            stats.move_speed = CADAVER_BLOOM_ROAM_SPEED;
            continue;
        };

        target.has_target = true;
        target.target_entity = employee_entity;
        target.target_stable_id = sensor.stable_id;
        target.last_known_position = employee_position;
        target.target_visible = true;
        target.lost_target = false;
        stats.move_speed = CADAVER_BLOOM_CHARGE_SPEED;

        set_cadaver_bloom_state(
            bloom_entity,
            &mut state,
            CadaverBloomState::Chase,
            &mut state_events,
        );
    }
}

fn cadaver_bloom_chase_target(
    game_seed: Res<GameSeed>,
    sim_tick: Res<SimTick>,
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<CadaverBloomStateChangedEvent>,
    mut speed_events: EventWriter<CadaverBloomErraticSpeedChangedEvent>,
    mut destination_events: EventWriter<CadaverBloomPerpendicularDestinationSelectedEvent>,
    mut search_events: EventWriter<CadaverBloomSearchStartedEvent>,
    mut blooms: Query<
        (
            Entity,
            &CadaverBloom,
            &mut SimPosition,
            &mut CadaverBloomState,
            &mut CadaverBloomTarget,
            &mut CadaverBloomMovement,
            &mut UnitStats,
        ),
        With<CadaverBloom>,
    >,
    employees: Query<(Entity, &SimPosition, &CadaverBloomEmployeeSensor), Without<CadaverBloom>>,
) {
    for (
        bloom_entity,
        bloom,
        mut bloom_position,
        mut state,
        mut target,
        mut movement,
        mut stats,
    ) in blooms.iter_mut()
    {
        if *state != CadaverBloomState::Chase || !target.has_target {
            continue;
        }

        let Some((employee_position, sensor)) =
            employee_position_by_stable_id(target.target_stable_id, &employees)
        else {
            start_cadaver_bloom_search(
                bloom_entity,
                &mut state,
                &mut target,
                &mut movement,
                &mut stats,
                sim_hz.0,
                &mut state_events,
                &mut search_events,
            );
            continue;
        };

        target.target_visible = sensor.visible_to_bloom && !sensor.behind_obstacle;
        if target.target_visible {
            target.last_known_position = employee_position;
        }

        if !target.target_visible {
            stats.move_speed = CADAVER_BLOOM_CHARGE_SPEED;
            move_axis_toward(
                &mut bloom_position,
                target.last_known_position,
                stats.move_speed / sim_hz.0,
            );

            if fixed_distance_sq(*bloom_position, target.last_known_position)
                <= fixed_square(I32F32::lit("1"))
            {
                start_cadaver_bloom_search(
                    bloom_entity,
                    &mut state,
                    &mut target,
                    &mut movement,
                    &mut stats,
                    sim_hz.0,
                    &mut state_events,
                    &mut search_events,
                );
            }
            continue;
        }

        let distance_sq = fixed_distance_sq(*bloom_position, employee_position);

        if distance_sq <= fixed_square(CADAVER_BLOOM_CLOSE_RANGE) {
            stats.move_speed = CADAVER_BLOOM_CLOSE_SPEED;
            movement.current_speed = CADAVER_BLOOM_CLOSE_SPEED;
            movement.destination = employee_position;
            move_axis_toward(&mut bloom_position, employee_position, stats.move_speed / sim_hz.0);
        } else if distance_sq <= fixed_square(CADAVER_BLOOM_ERRATIC_MAX_DISTANCE)
            && distance_sq >= fixed_square(CADAVER_BLOOM_ERRATIC_MIN_DISTANCE)
        {
            if movement.erratic_timer_ticks > 0 {
                movement.erratic_timer_ticks -= 1;
            } else {
                let salt = CADAVER_BLOOM_ERRATIC_SPEED_SALT ^ bloom.stable_id;
                let mut speed_rng = tick_rng(game_seed.0, sim_tick.0, salt);
                let speed_steps = (CADAVER_BLOOM_ERRATIC_MAX_SPEED - CADAVER_BLOOM_ERRATIC_MIN_SPEED)
                    .to_num::<u32>();
                let speed_offset = speed_rng.next_u32() % (speed_steps + 1);
                let next_speed =
                    CADAVER_BLOOM_ERRATIC_MIN_SPEED + I32F32::from_num(speed_offset);

                let timer_salt = CADAVER_BLOOM_ERRATIC_TIMER_SALT ^ bloom.stable_id;
                movement.erratic_timer_ticks = random_ticks_in_range(
                    &game_seed,
                    sim_tick.0,
                    timer_salt,
                    CADAVER_BLOOM_ERRATIC_MIN_SECONDS,
                    CADAVER_BLOOM_ERRATIC_MAX_SECONDS,
                    sim_hz.0,
                );
                movement.current_speed = next_speed;
                stats.move_speed = next_speed;

                speed_events.send(CadaverBloomErraticSpeedChangedEvent {
                    bloom: bloom_entity,
                    target_stable_id: target.target_stable_id,
                    speed: next_speed,
                    next_change_ticks: movement.erratic_timer_ticks,
                });

                movement.destination = perpendicular_destination(
                    *bloom_position,
                    employee_position,
                    &game_seed,
                    sim_tick.0,
                    CADAVER_BLOOM_PERPENDICULAR_SALT ^ bloom.stable_id,
                );

                destination_events.send(CadaverBloomPerpendicularDestinationSelectedEvent {
                    bloom: bloom_entity,
                    target_stable_id: target.target_stable_id,
                    destination: movement.destination,
                });
            }

            move_axis_toward(
                &mut bloom_position,
                movement.destination,
                stats.move_speed / sim_hz.0,
            );
        } else {
            stats.move_speed = CADAVER_BLOOM_CHARGE_SPEED;
            movement.current_speed = CADAVER_BLOOM_CHARGE_SPEED;
            movement.destination = employee_position;
            move_axis_toward(&mut bloom_position, employee_position, stats.move_speed / sim_hz.0);
        }
    }
}

fn cadaver_bloom_search_after_lost_target(
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<CadaverBloomStateChangedEvent>,
    mut blooms: Query<
        (
            Entity,
            &mut SimPosition,
            &mut CadaverBloomState,
            &mut CadaverBloomTarget,
            &mut CadaverBloomMovement,
            &mut UnitStats,
        ),
        With<CadaverBloom>,
    >,
) {
    for (bloom_entity, mut position, mut state, mut target, mut movement, mut stats) in
        blooms.iter_mut()
    {
        if *state != CadaverBloomState::Search {
            continue;
        }

        stats.move_speed = CADAVER_BLOOM_SEARCH_SPEED;
        movement.current_speed = CADAVER_BLOOM_SEARCH_SPEED;
        move_axis_toward(
            &mut position,
            target.last_known_position,
            CADAVER_BLOOM_SEARCH_SPEED / sim_hz.0,
        );

        if movement.search_timer_ticks > 0 {
            movement.search_timer_ticks -= 1;
            continue;
        }

        target.has_target = false;
        target.target_stable_id = 0;
        target.target_visible = false;
        target.lost_target = false;
        stats.move_speed = CADAVER_BLOOM_ROAM_SPEED;
        movement.current_speed = CADAVER_BLOOM_ROAM_SPEED;

        set_cadaver_bloom_state(
            bloom_entity,
            &mut state,
            CadaverBloomState::Roam,
            &mut state_events,
        );
    }
}

fn cadaver_bloom_investigate_lost_target_noise(
    sim_hz: Res<SimHz>,
    mut noise_events: EventReader<NoiseEmittedEvent>,
    mut investigated_events: EventWriter<CadaverBloomNoiseInvestigatedEvent>,
    mut state_events: EventWriter<CadaverBloomStateChangedEvent>,
    mut blooms: Query<
        (
            Entity,
            &SimPosition,
            &mut CadaverBloomState,
            &CadaverBloomTarget,
            &mut CadaverBloomMovement,
            &mut UnitStats,
        ),
        With<CadaverBloom>,
    >,
) {
    for event in noise_events.read() {
        for (bloom_entity, bloom_position, mut state, target, mut movement, mut stats) in
            blooms.iter_mut()
        {
            if !target.lost_target || *state == CadaverBloomState::Dead {
                continue;
            }

            if fixed_distance_sq(*bloom_position, event.position)
                > fixed_square(CADAVER_BLOOM_LOST_NOISE_RANGE)
            {
                continue;
            }

            movement.destination = event.position;
            movement.noise_turn_timer_ticks =
                fixed_seconds_to_ticks(CADAVER_BLOOM_NOISE_TURN_SECONDS, sim_hz.0);
            stats.move_speed = I32F32::ZERO;

            set_cadaver_bloom_state(
                bloom_entity,
                &mut state,
                CadaverBloomState::NoiseTurn,
                &mut state_events,
            );

            investigated_events.send(CadaverBloomNoiseInvestigatedEvent {
                bloom: bloom_entity,
                noise_source: event.source,
                sound_position: event.position,
                turn_ticks: movement.noise_turn_timer_ticks,
            });
        }
    }

    for (bloom_entity, _position, mut state, _target, mut movement, mut stats) in blooms.iter_mut() {
        if *state != CadaverBloomState::NoiseTurn {
            continue;
        }

        if movement.noise_turn_timer_ticks > 0 {
            movement.noise_turn_timer_ticks -= 1;
            continue;
        }

        stats.move_speed = CADAVER_BLOOM_SEARCH_SPEED;
        set_cadaver_bloom_state(
            bloom_entity,
            &mut state,
            CadaverBloomState::Search,
            &mut state_events,
        );
    }
}

fn cadaver_bloom_attack_close_target(
    mut damage_events: EventWriter<IncomingDamageEvent>,
    blooms: Query<(Entity, &SimPosition, &CadaverBloomState, &UnitStats, &CadaverBloomTarget), With<CadaverBloom>>,
    employees: Query<(Entity, &SimPosition, &CadaverBloomEmployeeSensor), Without<CadaverBloom>>,
) {
    for (bloom_entity, bloom_position, state, stats, target) in blooms.iter() {
        if *state != CadaverBloomState::Chase || !target.has_target {
            continue;
        }

        let Some((employee_position, sensor)) =
            employee_position_by_stable_id(target.target_stable_id, &employees)
        else {
            continue;
        };

        if !sensor.is_alive {
            continue;
        }

        if fixed_distance_sq(*bloom_position, employee_position) > fixed_square(stats.attack_range) {
            continue;
        }

        damage_events.send(IncomingDamageEvent {
            target: target.target_entity,
            raw_amount: CADAVER_BLOOM_ATTACK_DAMAGE,
            damage_type: DamageType::Standard,
            source: bloom_entity,
        });
    }
}

fn cadaver_bloom_apply_stun(
    sim_hz: Res<SimHz>,
    mut applied_events: EventReader<CadaverBloomStunAppliedEvent>,
    mut adjusted_events: EventWriter<CadaverBloomStunAdjustedEvent>,
    mut state_events: EventWriter<CadaverBloomStateChangedEvent>,
    mut blooms: Query<
        (
            Entity,
            &mut CadaverBloomState,
            &mut CadaverBloomMovement,
            &mut UnitStats,
            &mut CadaverBloomTarget,
        ),
        With<CadaverBloom>,
    >,
) {
    for event in applied_events.read() {
        let Ok((entity, mut state, mut movement, mut stats, mut target)) =
            blooms.get_mut(event.bloom)
        else {
            continue;
        };

        let adjusted_ticks = fixed_seconds_to_ticks(CADAVER_BLOOM_HIT_STUN_SECONDS, sim_hz.0);
        movement.stun_timer_ticks = adjusted_ticks;
        stats.move_speed = I32F32::ZERO;
        target.target_entity = event.source;

        set_cadaver_bloom_state(
            entity,
            &mut state,
            CadaverBloomState::Stunned,
            &mut state_events,
        );

        adjusted_events.send(CadaverBloomStunAdjustedEvent {
            bloom: event.bloom,
            source: event.source,
            adjusted_ticks,
        });
    }

    for (entity, mut state, mut movement, mut stats, _target) in blooms.iter_mut() {
        if *state != CadaverBloomState::Stunned {
            continue;
        }

        if movement.stun_timer_ticks > 0 {
            movement.stun_timer_ticks -= 1;
            continue;
        }

        stats.move_speed = movement.current_speed;
        set_cadaver_bloom_state(entity, &mut state, CadaverBloomState::Chase, &mut state_events);
    }
}

fn cadaver_bloom_door_attempt_speed(
    mut events: EventReader<CadaverBloomDoorAttemptEvent>,
    mut resolved_events: EventWriter<CadaverBloomDoorAttemptResolvedEvent>,
    blooms: Query<(), With<CadaverBloom>>,
) {
    for event in events.read() {
        if blooms.get(event.bloom).is_err() {
            continue;
        }

        resolved_events.send(CadaverBloomDoorAttemptResolvedEvent {
            bloom: event.bloom,
            door: event.door,
            adjusted_open_ticks: fixed_ticks_scaled(
                event.base_open_ticks,
                CADAVER_BLOOM_DOOR_SPEED_MULTIPLIER,
            ),
        });
    }
}

fn cadaver_bloom_take_damage(
    mut damage_events: EventReader<IncomingDamageEvent>,
    mut taken_events: EventWriter<CadaverBloomDamageTakenEvent>,
    mut defeated_events: EventWriter<CadaverBloomDefeatedEvent>,
    mut stun_events: EventWriter<CadaverBloomStunAppliedEvent>,
    mut blooms: Query<(Entity, &CadaverBloom, &mut Health, &mut CadaverBloomState), With<CadaverBloom>>,
) {
    for event in damage_events.read() {
        let Ok((bloom_entity, bloom, mut health, mut state)) = blooms.get_mut(event.target) else {
            continue;
        };

        if *state == CadaverBloomState::Dead {
            continue;
        }

        let damage = if event.raw_amount <= I32F32::ZERO {
            CADAVER_BLOOM_WEAPON_HIT_DAMAGE
        } else {
            event.raw_amount
        };

        health.current -= damage;
        if health.current < I32F32::ZERO {
            health.current = I32F32::ZERO;
        }

        taken_events.send(CadaverBloomDamageTakenEvent {
            bloom: bloom_entity,
            source: event.source,
            damage,
            remaining_health: health.current,
        });

        stun_events.send(CadaverBloomStunAppliedEvent {
            bloom: bloom_entity,
            source: event.source,
            base_ticks: 0,
        });

        if health.current <= I32F32::ZERO {
            *state = CadaverBloomState::Dead;
            defeated_events.send(CadaverBloomDefeatedEvent {
                bloom: bloom_entity,
                source: event.source,
                bloomed_employee: bloom.bloomed_employee,
                bloomed_employee_stable_id: bloom.bloomed_employee_stable_id,
            });
        }
    }
}

fn cadaver_bloom_spawn_body_on_death(
    mut defeated_events: EventReader<CadaverBloomDefeatedEvent>,
    mut body_events: EventWriter<EmployeeBodySpawnedEvent>,
    mut requested_events: EventWriter<CadaverBloomBodySpawnRequestedEvent>,
) {
    for event in defeated_events.read() {
        requested_events.send(CadaverBloomBodySpawnRequestedEvent {
            bloom: event.bloom,
            employee: event.bloomed_employee,
            employee_stable_id: event.bloomed_employee_stable_id,
        });

        body_events.send(EmployeeBodySpawnedEvent {
            employee: event.bloomed_employee,
            stable_id: event.bloomed_employee_stable_id,
            cause_id: CADAVER_BLOOM_BODY_CAUSE_ID,
        });
    }
}

fn cadaver_bloom_report_radar_and_teleport(
    mut radar_events: EventWriter<CadaverBloomRadarPipEvent>,
    mut teleport_events: EventWriter<CadaverBloomTeleportEligibleEvent>,
    blooms: Query<(Entity, &CadaverBloom), With<CadaverBloom>>,
    employees: Query<(Entity, &CadaverBloomEmployeeSensor), Without<CadaverBloom>>,
) {
    for (bloom_entity, bloom) in blooms.iter() {
        radar_events.send(CadaverBloomRadarPipEvent {
            bloom: bloom_entity,
            employee_stable_id: bloom.bloomed_employee_stable_id,
            employee_like_pip: true,
        });

        for (employee_entity, sensor) in employees.iter() {
            if !sensor.transformed_by_bloom {
                continue;
            }

            teleport_events.send(CadaverBloomTeleportEligibleEvent {
                bloom: bloom_entity,
                employee: employee_entity,
                employee_stable_id: sensor.stable_id,
                can_teleport_back_to_ship: !sensor.teleported_back_to_ship,
            });
        }
    }
}

fn cadaver_bloom_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    sim_hz: Res<SimHz>,
    blooms: Query<
        (
            &CadaverBloom,
            &SimPosition,
            &Health,
            &UnitStats,
            &CadaverBloomState,
            &CadaverBloomTarget,
            &CadaverBloomMovement,
            &CadaverBloomWeatherSensor,
        ),
        With<CadaverBloom>,
    >,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(sim_hz.0.to_bits() as u64);
    checksum.accumulate(CADAVER_BLOOM_SOURCE_REVISION as u64);
    checksum.accumulate(CADAVER_BLOOM_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(CADAVER_BLOOM_HP.to_bits() as u64);
    checksum.accumulate(CADAVER_BLOOM_POWER_LEVEL.to_bits() as u64);
    checksum.accumulate(CADAVER_BLOOM_MAX_SPAWNED as u64);
    checksum.accumulate(CADAVER_BLOOM_ATTACK_DAMAGE.to_bits() as u64);
    checksum.accumulate(CADAVER_BLOOM_STUN_MULTIPLIER.to_bits() as u64);
    checksum.accumulate(CADAVER_BLOOM_CAN_SEE_THROUGH_FOG as u64);
    checksum.accumulate(CADAVER_BLOOM_DOOR_SPEED_MULTIPLIER.to_bits() as u64);

    accumulate_str(&mut checksum, 0x1000, CADAVER_BLOOM_ID);
    accumulate_str(&mut checksum, 0x1001, CADAVER_BLOOM_NAME);
    accumulate_str(&mut checksum, 0x1002, CADAVER_BLOOM_TYPE);
    accumulate_str(&mut checksum, 0x1003, CADAVER_BLOOM_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, CADAVER_BLOOM_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1005, CADAVER_BLOOM_EXTRACTED_AT);
    accumulate_str(&mut checksum, 0x1006, CADAVER_BLOOM_DWELLS);
    accumulate_str(&mut checksum, 0x1007, CADAVER_BLOOM_DANGER);
    accumulate_str(&mut checksum, 0x1008, CADAVER_BLOOM_SCIENTIFIC_NAME);
    accumulate_str(&mut checksum, 0x1009, CADAVER_BLOOM_INTERNAL_NAME);
    accumulate_str(&mut checksum, 0x100A, CADAVER_BLOOM_PIP_SIZE);

    for dependency in CADAVER_BLOOM_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for behavior in CADAVER_BLOOM_FRONTMATTER_BEHAVIOR {
        accumulate_str(&mut checksum, 0x3000, behavior);
    }

    for rule in CADAVER_BLOOM_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x4000, rule.condition);
        accumulate_str(&mut checksum, 0x4001, rule.outcome);
    }

    for (bloom, position, health, stats, state, target, movement, weather) in blooms.iter() {
        checksum.accumulate(bloom.stable_id);
        checksum.accumulate(bloom.bloomed_employee_stable_id);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);
        checksum.accumulate(cadaver_bloom_state_bits(*state));
        checksum.accumulate(target.has_target as u64);
        checksum.accumulate(target.target_stable_id);
        checksum.accumulate(target.last_known_position.x.to_bits() as u64);
        checksum.accumulate(target.last_known_position.y.to_bits() as u64);
        checksum.accumulate(target.target_visible as u64);
        checksum.accumulate(target.lost_target as u64);
        checksum.accumulate(movement.current_speed.to_bits() as u64);
        checksum.accumulate(movement.scan_range.to_bits() as u64);
        checksum.accumulate(movement.destination.x.to_bits() as u64);
        checksum.accumulate(movement.destination.y.to_bits() as u64);
        checksum.accumulate(movement.erratic_timer_ticks as u64);
        checksum.accumulate(movement.search_timer_ticks as u64);
        checksum.accumulate(movement.noise_turn_timer_ticks as u64);
        checksum.accumulate(movement.stun_timer_ticks as u64);
        checksum.accumulate(weather.foggy as u64);
    }
}

fn start_cadaver_bloom_search(
    bloom_entity: Entity,
    state: &mut CadaverBloomState,
    target: &mut CadaverBloomTarget,
    movement: &mut CadaverBloomMovement,
    stats: &mut UnitStats,
    sim_hz: I32F32,
    state_events: &mut EventWriter<CadaverBloomStateChangedEvent>,
    search_events: &mut EventWriter<CadaverBloomSearchStartedEvent>,
) {
    target.lost_target = true;
    target.target_visible = false;
    movement.search_timer_ticks = fixed_seconds_to_ticks(CADAVER_BLOOM_SEARCH_SECONDS, sim_hz);
    stats.move_speed = CADAVER_BLOOM_SEARCH_SPEED;
    movement.current_speed = CADAVER_BLOOM_SEARCH_SPEED;

    search_events.send(CadaverBloomSearchStartedEvent {
        bloom: bloom_entity,
        target_stable_id: target.target_stable_id,
        last_known_position: target.last_known_position,
        search_ticks: movement.search_timer_ticks,
    });

    set_cadaver_bloom_state(
        bloom_entity,
        state,
        CadaverBloomState::Search,
        state_events,
    );
}

fn nearest_visible_employee(
    bloom_position: SimPosition,
    range: I32F32,
    employees: &Query<(Entity, &SimPosition, &CadaverBloomEmployeeSensor), Without<CadaverBloom>>,
) -> Option<(Entity, SimPosition, CadaverBloomEmployeeSensor)> {
    let mut best: Option<(Entity, SimPosition, CadaverBloomEmployeeSensor, I32F32)> = None;
    let range_sq = fixed_square(range);

    for (entity, position, sensor) in employees.iter() {
        if !sensor.is_alive || !sensor.visible_to_bloom {
            continue;
        }

        let distance_sq = fixed_distance_sq(bloom_position, *position);
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

fn employee_position_by_stable_id(
    stable_id: u64,
    employees: &Query<(Entity, &SimPosition, &CadaverBloomEmployeeSensor), Without<CadaverBloom>>,
) -> Option<(SimPosition, CadaverBloomEmployeeSensor)> {
    for (_entity, position, sensor) in employees.iter() {
        if sensor.stable_id == stable_id {
            return Some((*position, *sensor));
        }
    }

    None
}

fn perpendicular_destination(
    bloom_position: SimPosition,
    target_position: SimPosition,
    game_seed: &GameSeed,
    tick: u64,
    salt: u64,
) -> SimPosition {
    let dx = target_position.x - bloom_position.x;
    let dy = target_position.y - bloom_position.y;
    let mut rng = tick_rng(game_seed.0, tick, salt);
    let direction = if (rng.next_u32() % 2) == 0 {
        I32F32::lit("1")
    } else {
        I32F32::lit("-1")
    };

    SimPosition {
        x: target_position.x + (I32F32::ZERO - dy) * direction,
        y: target_position.y + dx * direction,
    }
}

fn set_cadaver_bloom_state(
    bloom: Entity,
    state: &mut CadaverBloomState,
    next: CadaverBloomState,
    events: &mut EventWriter<CadaverBloomStateChangedEvent>,
) {
    if *state == next {
        return;
    }

    let previous = *state;
    *state = next;
    events.send(CadaverBloomStateChangedEvent {
        bloom,
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

fn fixed_ticks_scaled(base_ticks: u32, multiplier: I32F32) -> u32 {
    let ticks = I32F32::from_num(base_ticks) / multiplier;
    if ticks <= I32F32::ONE {
        1
    } else {
        ticks.ceil().to_num::<u32>()
    }
}

fn random_ticks_in_range(
    game_seed: &GameSeed,
    tick: u64,
    salt: u64,
    min_seconds: I32F32,
    max_seconds: I32F32,
    sim_hz: I32F32,
) -> u32 {
    let min_ticks = fixed_seconds_to_ticks(min_seconds, sim_hz);
    let max_ticks = fixed_seconds_to_ticks(max_seconds, sim_hz);

    if max_ticks <= min_ticks {
        return min_ticks;
    }

    let mut rng = tick_rng(game_seed.0, tick, salt);
    min_ticks + (rng.next_u32() % (max_ticks - min_ticks + 1))
}

fn cadaver_bloom_state_bits(state: CadaverBloomState) -> u64 {
    match state {
        CadaverBloomState::Roam => 0,
        CadaverBloomState::Chase => 1,
        CadaverBloomState::Search => 2,
        CadaverBloomState::NoiseTurn => 3,
        CadaverBloomState::Stunned => 4,
        CadaverBloomState::Dead => 5,
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}
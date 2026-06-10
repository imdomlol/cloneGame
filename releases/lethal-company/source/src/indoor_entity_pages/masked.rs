// Sources: vault/indoor_entity_pages/masked.md
use bevy::prelude::*;
use fixed::types::I32F32;
use rand_core::RngCore;

use crate::sim::{
    tick_rng, DamageType, GameSeed, Health, IncomingDamageEvent, SimChecksumState, SimHz,
    SimPosition, SimTick, UnitStats,
};

pub const MASKED_ID: &str = "masked";
pub const MASKED_NAME: &str = "Masked";
pub const MASKED_TYPE: &str = "indoor_entity_pages";
pub const MASKED_SUBTYPE: &str = "enemy";
pub const MASKED_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Masked";
pub const MASKED_SOURCE_REVISION: u32 = 21253;
pub const MASKED_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const MASKED_CONFIDENCE_BASIS_POINTS: u16 = 97;

pub const MASKED_IMAGE: &str = "Masked2 Cel.png";
pub const MASKED_DWELLS: &str = "Indoors, Outdoors";
pub const MASKED_HP: I32F32 = I32F32::lit("4");
pub const MASKED_POWER_LEVEL: I32F32 = I32F32::lit("1");
pub const MASKED_MAX_SPAWNED: usize = 10;
pub const MASKED_STUN_MULTIPLIER: I32F32 = I32F32::lit("0.75");
pub const MASKED_CAN_SEE_THROUGH_FOG: bool = false;
pub const MASKED_SPAWN_DELAY_SECONDS: I32F32 = I32F32::lit("10");
pub const MASKED_DOOR_SPEED_MULTIPLIER: I32F32 = I32F32::lit("2");
pub const MASKED_ZAP_GUN_DIFFICULTY: I32F32 = I32F32::lit("1.3");
pub const MASKED_INTERNAL_NAME: &str = "MaskedPlayerEnemy";
pub const MASKED_PIP_SIZE: &str = "Medium (Employee Pip)";
pub const MASKED_CONTACT_DAMAGE: &str = "Instant kill";

pub const MASKED_ROAM_SPEED: I32F32 = I32F32::lit("1");
pub const MASKED_WALK_SPEED: I32F32 = I32F32::lit("2");
pub const MASKED_SPRINT_SPEED: I32F32 = I32F32::lit("5");
pub const MASKED_ATTACK_RANGE: I32F32 = I32F32::lit("1");
pub const MASKED_ATTACK_DAMAGE: I32F32 = I32F32::lit("1000000");
pub const MASKED_WATCH_RANGE: I32F32 = I32F32::lit("17");
pub const MASKED_GRAB_RANGE: I32F32 = I32F32::lit("6");
pub const MASKED_WALK_ARMS_DOWN_RANGE: I32F32 = I32F32::lit("12");
pub const MASKED_SPRINT_RANGE: I32F32 = I32F32::lit("17");
pub const MASKED_SHIP_HIDE_RANGE: I32F32 = I32F32::lit("22");
pub const MASKED_CHASE_LOSS_SECONDS: I32F32 = I32F32::lit("10");
pub const MASKED_STOP_AND_STARE_MIN_SECONDS: I32F32 = I32F32::lit("2");
pub const MASKED_STOP_AND_STARE_MAX_SECONDS: I32F32 = I32F32::lit("5");
pub const MASKED_PURSUIT_STARE_MIN_SECONDS: I32F32 = I32F32::lit("5");
pub const MASKED_PURSUIT_STARE_MAX_SECONDS: I32F32 = I32F32::lit("8");
pub const MASKED_PURSUIT_PAUSE_MAX_SECONDS: I32F32 = I32F32::lit("3");
pub const MASKED_RUN_RANDOMLY_MIN_SECONDS: I32F32 = I32F32::lit("0.7");
pub const MASKED_RUN_RANDOMLY_MAX_SECONDS: I32F32 = I32F32::lit("5");
pub const MASKED_SPRINT_TIMEOUT_SECONDS: I32F32 = I32F32::lit("6");
pub const MASKED_INSIDE_SPRINT_START_CHANCE_PERCENT: u32 = 20;
pub const MASKED_OUTSIDE_SPRINT_START_CHANCE_PERCENT: u32 = 35;
pub const MASKED_INSIDE_SPRINT_STOP_CHANCE_PERCENT: u32 = 30;
pub const MASKED_OUTSIDE_SPRINT_STOP_CHANCE_PERCENT: u32 = 80;
pub const MASKED_HIT_SPRINT_CHANCE_PERCENT: u32 = 40;
pub const MASKED_LOW_HP_HIT_SPRINT_CHANCE_PERCENT: u32 = 100;
pub const MASKED_BLUNT_STUN_SECONDS: I32F32 = I32F32::lit("0.5");

pub const MASKED_DEPENDS_ON: [&str; 0] = [];
pub const MASKED_ALIASES: [&str; 1] = ["Mimics"];
pub const MASKED_FRONTMATTER_BEHAVIOR: [&str; 3] = ["Roam", "Follow", "Hide"];

pub const MASKED_OCCURRENCE: [MaskedOccurrence; 4] = [
    MaskedOccurrence {
        moon: "artifice",
        base_spawn_chance: I32F32::lit("7.06"),
    },
    MaskedOccurrence {
        moon: "rend",
        base_spawn_chance: I32F32::lit("5.85"),
    },
    MaskedOccurrence {
        moon: "titan",
        base_spawn_chance: I32F32::lit("5.25"),
    },
    MaskedOccurrence {
        moon: "adamance",
        base_spawn_chance: I32F32::lit("0.38"),
    },
];

pub const MASKED_BEHAVIORAL_MECHANICS: [MaskedBehaviorRule; 30] = [
    MaskedBehaviorRule {
        condition: "it spawns",
        outcome: "it begins in Roaming",
    },
    MaskedBehaviorRule {
        condition: "roaming",
        outcome: "it searches for targets with a basic wandering routine and may randomly look around or sprint",
    },
    MaskedBehaviorRule {
        condition: "there are no living employees inside",
        outcome: "it may leave through the main entrance",
    },
    MaskedBehaviorRule {
        condition: "there are no living employees outside",
        outcome: "it may re-enter through the main entrance",
    },
    MaskedBehaviorRule {
        condition: "a fire exit would be the only available route",
        outcome: "it cannot use it",
    },
    MaskedBehaviorRule {
        condition: "it gains direct line of sight to an employee",
        outcome: "it stops and stares for 2 to 5 seconds before following",
    },
    MaskedBehaviorRule {
        condition: "chasing and within 6 meters of the target",
        outcome: "it walks with its arms out to grab",
    },
    MaskedBehaviorRule {
        condition: "chasing and between 6 and 12 meters from the target",
        outcome: "it walks with its arms down toward the target",
    },
    MaskedBehaviorRule {
        condition: "chasing and more than 17 meters from the target",
        outcome: "it sprints toward the target",
    },
    MaskedBehaviorRule {
        condition: "the StopAndStare timer runs for 5 to 8 seconds",
        outcome: "on expiry it pauses to stare for 0 to 3 seconds before resuming pursuit",
    },
    MaskedBehaviorRule {
        condition: "the target is within 17 meters",
        outcome: "the RunRandomly timer cycles every 0.7 to 5 seconds",
    },
    MaskedBehaviorRule {
        condition: "the RunRandomly timer expires while it is inside",
        outcome: "it has a 20% chance to start sprinting",
    },
    MaskedBehaviorRule {
        condition: "the RunRandomly timer expires while it is outside",
        outcome: "it has a 35% chance to start sprinting",
    },
    MaskedBehaviorRule {
        condition: "it is sprinting inside and the stop check succeeds",
        outcome: "it has a 30% chance to stop sprinting",
    },
    MaskedBehaviorRule {
        condition: "it is sprinting outside and the stop check succeeds",
        outcome: "it has an 80% chance to stop sprinting",
    },
    MaskedBehaviorRule {
        condition: "sprinting continues without a successful stop check",
        outcome: "it stops after 6 seconds",
    },
    MaskedBehaviorRule {
        condition: "it loses line of sight to its target",
        outcome: "a 10 second chase-loss timer starts",
    },
    MaskedBehaviorRule {
        condition: "the chase-loss timer runs",
        outcome: "it patrols the last known location",
    },
    MaskedBehaviorRule {
        condition: "it fails to recover the target before 10 seconds elapse",
        outcome: "it returns to Roaming",
    },
    MaskedBehaviorRule {
        condition: "it is outside, within 22 meters of the ship, and roaming",
        outcome: "it can enter Hiding",
    },
    MaskedBehaviorRule {
        condition: "it enters Hiding",
        outcome: "it boards the ship and waits in a hidden position",
    },
    MaskedBehaviorRule {
        condition: "it is spotted by an employee or comes within 6 meters of an employee",
        outcome: "it stands up and resumes pursuit",
    },
    MaskedBehaviorRule {
        condition: "it reaches an employee in direct contact",
        outcome: "the employee dies and begins conversion",
    },
    MaskedBehaviorRule {
        condition: "an employee is grabbed",
        outcome: "only a few seconds remain to kill, stun, or teleport the employee before conversion completes",
    },
    MaskedBehaviorRule {
        condition: "conversion completes",
        outcome: "the employee reanimates as a Masked",
    },
    MaskedBehaviorRule {
        condition: "it is hit",
        outcome: "there is a 40% chance it starts sprinting toward its target",
    },
    MaskedBehaviorRule {
        condition: "it has 1 HP remaining and is hit",
        outcome: "it starts sprinting toward its target with a 100% chance",
    },
    MaskedBehaviorRule {
        condition: "it is struck by a shovel or similar blunt weapon",
        outcome: "it is stunned for about 0.5 seconds",
    },
    MaskedBehaviorRule {
        condition: "it is shown on the ship radar",
        outcome: "it appears as an employee-like blue pip with facing direction",
    },
    MaskedBehaviorRule {
        condition: "it is scanned",
        outcome: "the scan provides no useful result because it cannot be scanned",
    },
];

pub struct MaskedPlugin;

impl Plugin for MaskedPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnMaskedEvent>()
            .add_event::<MaskedStateChangedEvent>()
            .add_event::<MaskedDoorAttemptEvent>()
            .add_event::<MaskedDoorAttemptResolvedEvent>()
            .add_event::<MaskedStunAppliedEvent>()
            .add_event::<MaskedStunAdjustedEvent>()
            .add_event::<MaskedZapGunTargetedEvent>()
            .add_event::<MaskedZapGunDifficultyEvent>()
            .add_event::<MaskedContactKillEvent>()
            .add_event::<MaskedConversionStartedEvent>()
            .add_event::<MaskedConversionCompletedEvent>()
            .add_event::<MaskedHitSprintRollEvent>()
            .add_event::<MaskedRadarPipEvent>()
            .add_event::<MaskedScanAttemptEvent>()
            .add_event::<MaskedNoScanResultEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_masked,
                    masked_find_line_of_sight_target,
                    masked_update_stop_and_stare,
                    masked_follow_target,
                    masked_update_run_randomly,
                    masked_update_chase_loss,
                    masked_use_main_entrance,
                    masked_enter_hiding_near_ship,
                    masked_leave_hiding_when_detected,
                    masked_contact_instant_kill,
                    masked_complete_conversion,
                    masked_door_attempt_speed,
                    masked_apply_stun_multiplier,
                    masked_report_zap_gun_difficulty,
                    masked_hit_sprint_roll,
                    masked_report_radar_pip,
                    masked_no_scan_result,
                    masked_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaskedOccurrence {
    pub moon: &'static str,
    pub base_spawn_chance: I32F32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaskedBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Masked;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MaskedEmployeeSensor {
    pub stable_id: u64,
    pub is_alive: bool,
    pub is_inside: bool,
    pub is_outside: bool,
    pub direct_line_of_sight_to_masked: bool,
    pub can_see_hidden_masked: bool,
    pub touching_masked: bool,
    pub teleported_after_grab: bool,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaskedLocation {
    pub is_inside: bool,
    pub is_outside: bool,
    pub near_main_entrance: bool,
    pub fire_exit_only_route: bool,
    pub within_ship_hide_range: bool,
    pub hidden_on_ship: bool,
}

impl Default for MaskedLocation {
    fn default() -> Self {
        Self {
            is_inside: true,
            is_outside: false,
            near_main_entrance: false,
            fire_exit_only_route: false,
            within_ship_hide_range: false,
            hidden_on_ship: false,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaskedTarget {
    pub has_target: bool,
    pub target_stable_id: u64,
    pub last_known_position: SimPosition,
    pub target_visible: bool,
}

impl Default for MaskedTarget {
    fn default() -> Self {
        Self {
            has_target: false,
            target_stable_id: 0,
            last_known_position: SimPosition {
                x: I32F32::lit("0"),
                y: I32F32::lit("0"),
            },
            target_visible: false,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaskedTimers {
    pub stop_and_stare_ticks: u32,
    pub pursuit_stare_ticks: u32,
    pub pursuit_pause_ticks: u32,
    pub run_randomly_ticks: u32,
    pub chase_loss_ticks: u32,
    pub sprint_ticks: u32,
    pub conversion_ticks: u32,
}

impl Default for MaskedTimers {
    fn default() -> Self {
        Self {
            stop_and_stare_ticks: 0,
            pursuit_stare_ticks: 0,
            pursuit_pause_ticks: 0,
            run_randomly_ticks: 0,
            chase_loss_ticks: 0,
            sprint_ticks: 0,
            conversion_ticks: 0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MaskedState {
    #[default]
    Roaming,
    StopAndStare,
    Following,
    Hiding,
    Converting,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MaskedMovementMode {
    #[default]
    Roam,
    WalkArmsOut,
    WalkArmsDown,
    Sprint,
}

#[derive(Bundle)]
pub struct MaskedBundle {
    pub name: Name,
    pub masked: Masked,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub state: MaskedState,
    pub movement_mode: MaskedMovementMode,
    pub location: MaskedLocation,
    pub target: MaskedTarget,
    pub timers: MaskedTimers,
}

impl MaskedBundle {
    pub fn new(event: SpawnMaskedEvent) -> Self {
        Self {
            name: Name::new(MASKED_NAME),
            masked: Masked,
            position: event.position,
            health: Health::full(MASKED_HP),
            stats: UnitStats {
                move_speed: MASKED_ROAM_SPEED,
                attack_range: MASKED_ATTACK_RANGE,
                attack_damage: MASKED_ATTACK_DAMAGE,
                attack_speed: I32F32::lit("1"),
                watch_range: MASKED_WATCH_RANGE,
            },
            state: MaskedState::Roaming,
            movement_mode: MaskedMovementMode::Roam,
            location: MaskedLocation {
                is_inside: event.starts_inside,
                is_outside: !event.starts_inside,
                near_main_entrance: false,
                fire_exit_only_route: false,
                within_ship_hide_range: false,
                hidden_on_ship: false,
            },
            target: MaskedTarget {
                has_target: false,
                target_stable_id: 0,
                last_known_position: event.position,
                target_visible: false,
            },
            timers: MaskedTimers::default(),
        }
    }
}

#[derive(Event, Clone, Copy, Debug)]
pub struct SpawnMaskedEvent {
    pub position: SimPosition,
    pub starts_inside: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaskedStateChangedEvent {
    pub masked: Entity,
    pub from: MaskedState,
    pub to: MaskedState,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaskedDoorAttemptEvent {
    pub masked: Entity,
    pub door: Entity,
    pub base_open_ticks: u32,
    pub is_fire_exit: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaskedDoorAttemptResolvedEvent {
    pub masked: Entity,
    pub door: Entity,
    pub adjusted_open_ticks: u32,
    pub can_use_route: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaskedStunAppliedEvent {
    pub masked: Entity,
    pub base_ticks: u32,
    pub is_blunt_weapon: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaskedStunAdjustedEvent {
    pub masked: Entity,
    pub base_ticks: u32,
    pub adjusted_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaskedZapGunTargetedEvent {
    pub masked: Entity,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaskedZapGunDifficultyEvent {
    pub masked: Entity,
    pub difficulty_modifier: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaskedContactKillEvent {
    pub masked: Entity,
    pub employee: Entity,
    pub employee_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaskedConversionStartedEvent {
    pub masked: Entity,
    pub employee: Entity,
    pub employee_stable_id: u64,
    pub conversion_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaskedConversionCompletedEvent {
    pub masked: Entity,
    pub employee: Entity,
    pub employee_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaskedHitSprintRollEvent {
    pub masked: Entity,
    pub source: Entity,
    pub chance_percent: u32,
    pub started_sprinting: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaskedRadarPipEvent {
    pub masked: Entity,
    pub employee_like_blue_pip: bool,
    pub has_facing_direction: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaskedScanAttemptEvent {
    pub masked: Entity,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaskedNoScanResultEvent {
    pub masked: Entity,
}

fn spawn_masked(
    mut commands: Commands,
    mut events: EventReader<SpawnMaskedEvent>,
    maskeds: Query<(), With<Masked>>,
) {
    let mut spawned_count = maskeds.iter().count();

    for event in events.read() {
        if spawned_count >= MASKED_MAX_SPAWNED {
            break;
        }

        commands.spawn(MaskedBundle::new(*event));
        spawned_count += 1;
    }
}

fn masked_find_line_of_sight_target(
    game_seed: Res<GameSeed>,
    sim_tick: Res<SimTick>,
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<MaskedStateChangedEvent>,
    mut maskeds: Query<
        (
            Entity,
            &mut MaskedState,
            &SimPosition,
            &mut UnitStats,
            &mut MaskedTarget,
            &mut MaskedTimers,
        ),
        With<Masked>,
    >,
    employees: Query<(&SimPosition, &MaskedEmployeeSensor), Without<Masked>>,
) {
    for (masked_entity, mut state, position, mut stats, mut target, mut timers) in maskeds.iter_mut()
    {
        if *state != MaskedState::Roaming {
            continue;
        }

        let Some((employee_position, employee_sensor)) =
            visible_employee_for_masked(*position, &employees)
        else {
            continue;
        };

        target.has_target = true;
        target.target_stable_id = employee_sensor.stable_id;
        target.last_known_position = employee_position;
        target.target_visible = true;
        stats.move_speed = I32F32::lit("0");
        timers.stop_and_stare_ticks = random_ticks_in_range(
            &game_seed,
            sim_tick.0,
            0x4d41534b45535a01u64 ^ employee_sensor.stable_id,
            MASKED_STOP_AND_STARE_MIN_SECONDS,
            MASKED_STOP_AND_STARE_MAX_SECONDS,
            sim_hz.0,
        );

        set_masked_state(
            masked_entity,
            &mut state,
            MaskedState::StopAndStare,
            &mut state_events,
        );
    }
}

fn masked_update_stop_and_stare(
    game_seed: Res<GameSeed>,
    sim_tick: Res<SimTick>,
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<MaskedStateChangedEvent>,
    mut maskeds: Query<
        (
            Entity,
            &mut MaskedState,
            &mut UnitStats,
            &mut MaskedMovementMode,
            &MaskedTarget,
            &mut MaskedTimers,
        ),
        With<Masked>,
    >,
) {
    for (masked_entity, mut state, mut stats, mut movement, target, mut timers) in
        maskeds.iter_mut()
    {
        if *state != MaskedState::StopAndStare {
            continue;
        }

        if timers.stop_and_stare_ticks > 0 {
            timers.stop_and_stare_ticks -= 1;
            continue;
        }

        stats.move_speed = MASKED_WALK_SPEED;
        *movement = MaskedMovementMode::WalkArmsDown;
        timers.pursuit_stare_ticks = random_ticks_in_range(
            &game_seed,
            sim_tick.0,
            0x4d41534b45535401u64 ^ target.target_stable_id,
            MASKED_PURSUIT_STARE_MIN_SECONDS,
            MASKED_PURSUIT_STARE_MAX_SECONDS,
            sim_hz.0,
        );
        timers.run_randomly_ticks = random_ticks_in_range(
            &game_seed,
            sim_tick.0,
            0x4d41534b45525201u64 ^ target.target_stable_id,
            MASKED_RUN_RANDOMLY_MIN_SECONDS,
            MASKED_RUN_RANDOMLY_MAX_SECONDS,
            sim_hz.0,
        );

        set_masked_state(
            masked_entity,
            &mut state,
            MaskedState::Following,
            &mut state_events,
        );
    }
}

fn masked_follow_target(
    game_seed: Res<GameSeed>,
    sim_tick: Res<SimTick>,
    sim_hz: Res<SimHz>,
    mut maskeds: Query<
        (
            &mut SimPosition,
            &MaskedState,
            &mut UnitStats,
            &mut MaskedMovementMode,
            &mut MaskedTarget,
            &mut MaskedTimers,
        ),
        With<Masked>,
    >,
    employees: Query<(&SimPosition, &MaskedEmployeeSensor), Without<Masked>>,
) {
    for (mut position, state, mut stats, mut movement, mut target, mut timers) in
        maskeds.iter_mut()
    {
        if *state != MaskedState::Following || !target.has_target {
            continue;
        }

        if timers.pursuit_pause_ticks > 0 {
            timers.pursuit_pause_ticks -= 1;
            stats.move_speed = I32F32::lit("0");
            continue;
        }

        if timers.pursuit_stare_ticks > 0 {
            timers.pursuit_stare_ticks -= 1;
        } else {
            timers.pursuit_pause_ticks = random_ticks_in_range(
                &game_seed,
                sim_tick.0,
                0x4d41534b45505001u64 ^ target.target_stable_id,
                I32F32::lit("0"),
                MASKED_PURSUIT_PAUSE_MAX_SECONDS,
                sim_hz.0,
            );
            timers.pursuit_stare_ticks = random_ticks_in_range(
                &game_seed,
                sim_tick.0,
                0x4d41534b45535402u64 ^ target.target_stable_id,
                MASKED_PURSUIT_STARE_MIN_SECONDS,
                MASKED_PURSUIT_STARE_MAX_SECONDS,
                sim_hz.0,
            );
        }

        let Some((target_position, sensor)) =
            employee_position_by_stable_id(target.target_stable_id, &employees)
        else {
            target.target_visible = false;
            continue;
        };

        target.target_visible = sensor.direct_line_of_sight_to_masked;
        if target.target_visible {
            target.last_known_position = target_position;
        }

        let distance_sq = fixed_distance_sq(*position, target_position);
        if distance_sq <= fixed_square(MASKED_GRAB_RANGE) {
            *movement = MaskedMovementMode::WalkArmsOut;
            stats.move_speed = MASKED_WALK_SPEED;
        } else if distance_sq <= fixed_square(MASKED_WALK_ARMS_DOWN_RANGE) {
            *movement = MaskedMovementMode::WalkArmsDown;
            stats.move_speed = MASKED_WALK_SPEED;
        } else if distance_sq > fixed_square(MASKED_SPRINT_RANGE) {
            *movement = MaskedMovementMode::Sprint;
            stats.move_speed = MASKED_SPRINT_SPEED;
            if timers.sprint_ticks == 0 {
                timers.sprint_ticks = fixed_seconds_to_ticks(MASKED_SPRINT_TIMEOUT_SECONDS, sim_hz.0);
            }
        }

        move_axis_toward(&mut position, target_position, stats.move_speed / sim_hz.0);
    }
}

fn masked_update_run_randomly(
    game_seed: Res<GameSeed>,
    sim_tick: Res<SimTick>,
    sim_hz: Res<SimHz>,
    mut maskeds: Query<
        (
            Entity,
            &MaskedState,
            &mut UnitStats,
            &mut MaskedMovementMode,
            &MaskedLocation,
            &MaskedTarget,
            &mut MaskedTimers,
        ),
        With<Masked>,
    >,
) {
    for (masked_entity, state, mut stats, mut movement, location, target, mut timers) in
        maskeds.iter_mut()
    {
        if *state != MaskedState::Following || !target.has_target {
            continue;
        }

        if timers.sprint_ticks > 0 {
            timers.sprint_ticks -= 1;
            let stop_chance = if location.is_inside {
                MASKED_INSIDE_SPRINT_STOP_CHANCE_PERCENT
            } else {
                MASKED_OUTSIDE_SPRINT_STOP_CHANCE_PERCENT
            };
            if percent_roll(
                &game_seed,
                sim_tick.0,
                0x4d41534b45535001u64 ^ masked_entity.to_bits(),
                stop_chance,
            ) || timers.sprint_ticks == 0
            {
                *movement = MaskedMovementMode::WalkArmsDown;
                stats.move_speed = MASKED_WALK_SPEED;
                timers.sprint_ticks = 0;
            }
        }

        if timers.run_randomly_ticks > 0 {
            timers.run_randomly_ticks -= 1;
            continue;
        }

        timers.run_randomly_ticks = random_ticks_in_range(
            &game_seed,
            sim_tick.0,
            0x4d41534b45525202u64 ^ target.target_stable_id,
            MASKED_RUN_RANDOMLY_MIN_SECONDS,
            MASKED_RUN_RANDOMLY_MAX_SECONDS,
            sim_hz.0,
        );

        let start_chance = if location.is_inside {
            MASKED_INSIDE_SPRINT_START_CHANCE_PERCENT
        } else {
            MASKED_OUTSIDE_SPRINT_START_CHANCE_PERCENT
        };
        if percent_roll(
            &game_seed,
            sim_tick.0,
            0x4d41534b45525301u64 ^ masked_entity.to_bits(),
            start_chance,
        ) {
            *movement = MaskedMovementMode::Sprint;
            stats.move_speed = MASKED_SPRINT_SPEED;
            timers.sprint_ticks = fixed_seconds_to_ticks(MASKED_SPRINT_TIMEOUT_SECONDS, sim_hz.0);
        }
    }
}

fn masked_update_chase_loss(
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<MaskedStateChangedEvent>,
    mut maskeds: Query<
        (
            Entity,
            &mut SimPosition,
            &mut MaskedState,
            &mut UnitStats,
            &mut MaskedMovementMode,
            &mut MaskedTarget,
            &mut MaskedTimers,
        ),
        With<Masked>,
    >,
) {
    for (masked_entity, mut position, mut state, mut stats, mut movement, mut target, mut timers) in
        maskeds.iter_mut()
    {
        if *state != MaskedState::Following || !target.has_target {
            continue;
        }

        if target.target_visible {
            timers.chase_loss_ticks = 0;
            continue;
        }

        if timers.chase_loss_ticks == 0 {
            timers.chase_loss_ticks = fixed_seconds_to_ticks(MASKED_CHASE_LOSS_SECONDS, sim_hz.0);
        }

        move_axis_toward(
            &mut position,
            target.last_known_position,
            MASKED_WALK_SPEED / sim_hz.0,
        );

        timers.chase_loss_ticks -= 1;
        if timers.chase_loss_ticks > 0 {
            continue;
        }

        target.has_target = false;
        target.target_stable_id = 0;
        target.target_visible = false;
        stats.move_speed = MASKED_ROAM_SPEED;
        *movement = MaskedMovementMode::Roam;

        set_masked_state(
            masked_entity,
            &mut state,
            MaskedState::Roaming,
            &mut state_events,
        );
    }
}

fn masked_use_main_entrance(
    mut maskeds: Query<(&MaskedState, &mut MaskedLocation), With<Masked>>,
    employees: Query<&MaskedEmployeeSensor, Without<Masked>>,
) {
    let living_inside = employees
        .iter()
        .any(|employee| employee.is_alive && employee.is_inside);
    let living_outside = employees
        .iter()
        .any(|employee| employee.is_alive && employee.is_outside);

    for (state, mut location) in maskeds.iter_mut() {
        if *state != MaskedState::Roaming || !location.near_main_entrance {
            continue;
        }

        if location.fire_exit_only_route {
            continue;
        }

        if location.is_inside && !living_inside {
            location.is_inside = false;
            location.is_outside = true;
        } else if location.is_outside && !living_outside {
            location.is_inside = true;
            location.is_outside = false;
        }
    }
}

fn masked_enter_hiding_near_ship(
    mut state_events: EventWriter<MaskedStateChangedEvent>,
    mut maskeds: Query<
        (
            Entity,
            &mut MaskedState,
            &mut UnitStats,
            &mut MaskedMovementMode,
            &mut MaskedLocation,
        ),
        With<Masked>,
    >,
) {
    for (masked_entity, mut state, mut stats, mut movement, mut location) in maskeds.iter_mut() {
        if *state != MaskedState::Roaming || !location.is_outside || !location.within_ship_hide_range
        {
            continue;
        }

        location.hidden_on_ship = true;
        stats.move_speed = I32F32::lit("0");
        *movement = MaskedMovementMode::Roam;

        set_masked_state(
            masked_entity,
            &mut state,
            MaskedState::Hiding,
            &mut state_events,
        );
    }
}

fn masked_leave_hiding_when_detected(
    mut state_events: EventWriter<MaskedStateChangedEvent>,
    mut maskeds: Query<
        (
            Entity,
            &SimPosition,
            &mut MaskedState,
            &mut UnitStats,
            &mut MaskedLocation,
            &mut MaskedTarget,
        ),
        With<Masked>,
    >,
    employees: Query<(&SimPosition, &MaskedEmployeeSensor), Without<Masked>>,
) {
    for (masked_entity, masked_position, mut state, mut stats, mut location, mut target) in
        maskeds.iter_mut()
    {
        if *state != MaskedState::Hiding {
            continue;
        }

        let Some((employee_position, sensor)) =
            hidden_detection_employee(*masked_position, &employees)
        else {
            continue;
        };

        location.hidden_on_ship = false;
        target.has_target = true;
        target.target_stable_id = sensor.stable_id;
        target.last_known_position = employee_position;
        target.target_visible = sensor.direct_line_of_sight_to_masked;
        stats.move_speed = MASKED_WALK_SPEED;

        set_masked_state(
            masked_entity,
            &mut state,
            MaskedState::Following,
            &mut state_events,
        );
    }
}

fn masked_contact_instant_kill(
    sim_hz: Res<SimHz>,
    mut damage_events: EventWriter<IncomingDamageEvent>,
    mut contact_events: EventWriter<MaskedContactKillEvent>,
    mut conversion_events: EventWriter<MaskedConversionStartedEvent>,
    mut maskeds: Query<(Entity, &mut MaskedState, &mut MaskedTarget, &mut MaskedTimers), With<Masked>>,
    employees: Query<(Entity, &MaskedEmployeeSensor), Without<Masked>>,
) {
    for (masked_entity, mut state, mut target, mut timers) in maskeds.iter_mut() {
        if *state == MaskedState::Converting {
            continue;
        }

        for (employee_entity, employee_sensor) in employees.iter() {
            if !employee_sensor.touching_masked || !employee_sensor.is_alive {
                continue;
            }

            target.has_target = true;
            target.target_stable_id = employee_sensor.stable_id;
            timers.conversion_ticks = fixed_seconds_to_ticks(I32F32::lit("3"), sim_hz.0);
            *state = MaskedState::Converting;

            contact_events.send(MaskedContactKillEvent {
                masked: masked_entity,
                employee: employee_entity,
                employee_stable_id: employee_sensor.stable_id,
            });
            conversion_events.send(MaskedConversionStartedEvent {
                masked: masked_entity,
                employee: employee_entity,
                employee_stable_id: employee_sensor.stable_id,
                conversion_ticks: timers.conversion_ticks,
            });
            damage_events.send(IncomingDamageEvent {
                target: employee_entity,
                raw_amount: MASKED_ATTACK_DAMAGE,
                damage_type: DamageType::Standard,
                source: masked_entity,
            });
        }
    }
}

fn masked_complete_conversion(
    mut spawn_events: EventWriter<SpawnMaskedEvent>,
    mut completed_events: EventWriter<MaskedConversionCompletedEvent>,
    mut state_events: EventWriter<MaskedStateChangedEvent>,
    mut maskeds: Query<
        (
            Entity,
            &SimPosition,
            &mut MaskedState,
            &mut UnitStats,
            &mut MaskedTarget,
            &mut MaskedTimers,
        ),
        With<Masked>,
    >,
    employees: Query<(Entity, &MaskedEmployeeSensor), Without<Masked>>,
) {
    for (masked_entity, position, mut state, mut stats, mut target, mut timers) in maskeds.iter_mut()
    {
        if *state != MaskedState::Converting {
            continue;
        }

        if timers.conversion_ticks > 0 {
            timers.conversion_ticks -= 1;
            continue;
        }

        let employee_entity = employee_entity_by_stable_id(target.target_stable_id, &employees);
        if let Some(employee) = employee_entity {
            completed_events.send(MaskedConversionCompletedEvent {
                masked: masked_entity,
                employee,
                employee_stable_id: target.target_stable_id,
            });
        }

        spawn_events.send(SpawnMaskedEvent {
            position: *position,
            starts_inside: true,
        });

        target.has_target = false;
        target.target_stable_id = 0;
        stats.move_speed = MASKED_ROAM_SPEED;

        set_masked_state(
            masked_entity,
            &mut state,
            MaskedState::Roaming,
            &mut state_events,
        );
    }
}

fn masked_door_attempt_speed(
    mut events: EventReader<MaskedDoorAttemptEvent>,
    mut resolved_events: EventWriter<MaskedDoorAttemptResolvedEvent>,
    maskeds: Query<&MaskedLocation, With<Masked>>,
) {
    for event in events.read() {
        let Ok(location) = maskeds.get(event.masked) else {
            continue;
        };

        let can_use_route = !event.is_fire_exit && !location.fire_exit_only_route;
        resolved_events.send(MaskedDoorAttemptResolvedEvent {
            masked: event.masked,
            door: event.door,
            adjusted_open_ticks: fixed_ticks_scaled(event.base_open_ticks, MASKED_DOOR_SPEED_MULTIPLIER),
            can_use_route,
        });
    }
}

fn masked_apply_stun_multiplier(
    sim_hz: Res<SimHz>,
    mut events: EventReader<MaskedStunAppliedEvent>,
    mut adjusted_events: EventWriter<MaskedStunAdjustedEvent>,
) {
    for event in events.read() {
        let adjusted_ticks = if event.is_blunt_weapon {
            fixed_seconds_to_ticks(MASKED_BLUNT_STUN_SECONDS, sim_hz.0)
        } else {
            fixed_ticks_scaled(event.base_ticks, MASKED_STUN_MULTIPLIER)
        };

        adjusted_events.send(MaskedStunAdjustedEvent {
            masked: event.masked,
            base_ticks: event.base_ticks,
            adjusted_ticks,
        });
    }
}

fn masked_report_zap_gun_difficulty(
    mut events: EventReader<MaskedZapGunTargetedEvent>,
    mut difficulty_events: EventWriter<MaskedZapGunDifficultyEvent>,
    maskeds: Query<(), With<Masked>>,
) {
    for event in events.read() {
        if maskeds.get(event.masked).is_err() {
            continue;
        }

        difficulty_events.send(MaskedZapGunDifficultyEvent {
            masked: event.masked,
            difficulty_modifier: MASKED_ZAP_GUN_DIFFICULTY,
        });
    }
}

fn masked_hit_sprint_roll(
    game_seed: Res<GameSeed>,
    sim_tick: Res<SimTick>,
    sim_hz: Res<SimHz>,
    mut damage_events: EventReader<IncomingDamageEvent>,
    mut roll_events: EventWriter<MaskedHitSprintRollEvent>,
    mut maskeds: Query<(&Health, &mut UnitStats, &mut MaskedMovementMode, &mut MaskedTimers), With<Masked>>,
) {
    for event in damage_events.read() {
        let Ok((health, mut stats, mut movement, mut timers)) = maskeds.get_mut(event.target) else {
            continue;
        };

        let chance = if health.current <= I32F32::lit("1") {
            MASKED_LOW_HP_HIT_SPRINT_CHANCE_PERCENT
        } else {
            MASKED_HIT_SPRINT_CHANCE_PERCENT
        };
        let started_sprinting = percent_roll(
            &game_seed,
            sim_tick.0,
            0x4d41534b45484901u64 ^ event.target.to_bits(),
            chance,
        );

        if started_sprinting {
            *movement = MaskedMovementMode::Sprint;
            stats.move_speed = MASKED_SPRINT_SPEED;
            timers.sprint_ticks = fixed_seconds_to_ticks(MASKED_SPRINT_TIMEOUT_SECONDS, sim_hz.0);
        }

        roll_events.send(MaskedHitSprintRollEvent {
            masked: event.target,
            source: event.source,
            chance_percent: chance,
            started_sprinting,
        });
    }
}

fn masked_report_radar_pip(
    mut radar_events: EventWriter<MaskedRadarPipEvent>,
    maskeds: Query<Entity, With<Masked>>,
) {
    for masked in maskeds.iter() {
        radar_events.send(MaskedRadarPipEvent {
            masked,
            employee_like_blue_pip: true,
            has_facing_direction: true,
        });
    }
}

fn masked_no_scan_result(
    mut events: EventReader<MaskedScanAttemptEvent>,
    mut no_result_events: EventWriter<MaskedNoScanResultEvent>,
    maskeds: Query<(), With<Masked>>,
) {
    for event in events.read() {
        if maskeds.get(event.masked).is_err() {
            continue;
        }

        no_result_events.send(MaskedNoScanResultEvent {
            masked: event.masked,
        });
    }
}

fn masked_checksum(
    mut checksum: ResMut<SimChecksumState>,
    maskeds: Query<
        (
            &SimPosition,
            &Health,
            &UnitStats,
            &MaskedState,
            &MaskedMovementMode,
            &MaskedLocation,
            &MaskedTarget,
            &MaskedTimers,
        ),
        With<Masked>,
    >,
) {
    for (position, health, stats, state, movement, location, target, timers) in maskeds.iter() {
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);
        checksum.accumulate(masked_state_bits(*state));
        checksum.accumulate(masked_movement_bits(*movement));
        checksum.accumulate(location.is_inside as u64);
        checksum.accumulate(location.is_outside as u64);
        checksum.accumulate(location.near_main_entrance as u64);
        checksum.accumulate(location.fire_exit_only_route as u64);
        checksum.accumulate(location.within_ship_hide_range as u64);
        checksum.accumulate(location.hidden_on_ship as u64);
        checksum.accumulate(target.has_target as u64);
        checksum.accumulate(target.target_stable_id);
        checksum.accumulate(target.last_known_position.x.to_bits() as u64);
        checksum.accumulate(target.last_known_position.y.to_bits() as u64);
        checksum.accumulate(target.target_visible as u64);
        checksum.accumulate(timers.stop_and_stare_ticks as u64);
        checksum.accumulate(timers.pursuit_stare_ticks as u64);
        checksum.accumulate(timers.pursuit_pause_ticks as u64);
        checksum.accumulate(timers.run_randomly_ticks as u64);
        checksum.accumulate(timers.chase_loss_ticks as u64);
        checksum.accumulate(timers.sprint_ticks as u64);
        checksum.accumulate(timers.conversion_ticks as u64);
    }
}

fn visible_employee_for_masked(
    masked_position: SimPosition,
    employees: &Query<(&SimPosition, &MaskedEmployeeSensor), Without<Masked>>,
) -> Option<(SimPosition, MaskedEmployeeSensor)> {
    let mut best: Option<(SimPosition, MaskedEmployeeSensor)> = None;

    for (position, sensor) in employees.iter() {
        if !sensor.is_alive || !sensor.direct_line_of_sight_to_masked {
            continue;
        }

        if fixed_distance_sq(masked_position, *position) > fixed_square(MASKED_WATCH_RANGE) {
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

fn hidden_detection_employee(
    masked_position: SimPosition,
    employees: &Query<(&SimPosition, &MaskedEmployeeSensor), Without<Masked>>,
) -> Option<(SimPosition, MaskedEmployeeSensor)> {
    let mut best: Option<(SimPosition, MaskedEmployeeSensor)> = None;

    for (position, sensor) in employees.iter() {
        if !sensor.is_alive {
            continue;
        }

        let close_enough = fixed_distance_sq(masked_position, *position) <= fixed_square(MASKED_GRAB_RANGE);
        if !sensor.can_see_hidden_masked && !close_enough {
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

fn employee_position_by_stable_id(
    stable_id: u64,
    employees: &Query<(&SimPosition, &MaskedEmployeeSensor), Without<Masked>>,
) -> Option<(SimPosition, MaskedEmployeeSensor)> {
    for (position, sensor) in employees.iter() {
        if sensor.stable_id == stable_id {
            return Some((*position, *sensor));
        }
    }

    None
}

fn employee_entity_by_stable_id(
    stable_id: u64,
    employees: &Query<(Entity, &MaskedEmployeeSensor), Without<Masked>>,
) -> Option<Entity> {
    for (entity, sensor) in employees.iter() {
        if sensor.stable_id == stable_id {
            return Some(entity);
        }
    }

    None
}

fn set_masked_state(
    masked: Entity,
    state: &mut MaskedState,
    next: MaskedState,
    events: &mut EventWriter<MaskedStateChangedEvent>,
) {
    if *state == next {
        return;
    }

    let previous = *state;
    *state = next;
    events.send(MaskedStateChangedEvent {
        masked,
        from: previous,
        to: next,
    });
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

fn percent_roll(game_seed: &GameSeed, tick: u64, salt: u64, chance_percent: u32) -> bool {
    if chance_percent >= 100 {
        return true;
    }

    let mut rng = tick_rng(game_seed.0, tick, salt);
    rng.next_u32() % 100 < chance_percent
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

fn masked_state_bits(state: MaskedState) -> u64 {
    match state {
        MaskedState::Roaming => 0,
        MaskedState::StopAndStare => 1,
        MaskedState::Following => 2,
        MaskedState::Hiding => 3,
        MaskedState::Converting => 4,
    }
}

fn masked_movement_bits(movement: MaskedMovementMode) -> u64 {
    match movement {
        MaskedMovementMode::Roam => 0,
        MaskedMovementMode::WalkArmsOut => 1,
        MaskedMovementMode::WalkArmsDown => 2,
        MaskedMovementMode::Sprint => 3,
    }
}
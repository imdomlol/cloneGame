// Sources: vault/indoor_entity_pages/coil_head.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{
    DamageType, Health, IncomingDamageEvent, SimChecksumState, SimHz, SimPosition, UnitStats,
};

pub const COIL_HEAD_ID: &str = "coil_head";
pub const COIL_HEAD_NAME: &str = "Coil-Head";
pub const COIL_HEAD_TYPE: &str = "indoor_entity_pages";
pub const COIL_HEAD_SUBTYPE: &str = "creature";
pub const COIL_HEAD_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Coil-Head";
pub const COIL_HEAD_SOURCE_REVISION: u32 = 21316;
pub const COIL_HEAD_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const COIL_HEAD_CONFIDENCE_BASIS_POINTS: u16 = 91;

pub const COIL_HEAD_DWELLS: &str = "Inside";
pub const COIL_HEAD_INTERNAL_NAME: &str = "Springman";
pub const COIL_HEAD_POWER_LEVEL: I32F32 = I32F32::lit("1");
pub const COIL_HEAD_MAX_SPAWNED: usize = 5;
pub const COIL_HEAD_ATTACK_DAMAGE: I32F32 = I32F32::lit("90");
pub const COIL_HEAD_ATTACK_SPEED_SECONDS: I32F32 = I32F32::lit("0.2");
pub const COIL_HEAD_DPS: I32F32 = I32F32::lit("450");
pub const COIL_HEAD_STUN_MULTIPLIER: I32F32 = I32F32::lit("3.25");
pub const COIL_HEAD_RADAR_PIP_SIZE: &str = "Medium-small";
pub const COIL_HEAD_SHOVEL_HP: &str = "Immune";
pub const COIL_HEAD_SHOCK_RESPONSE: &str = "Susceptible";
pub const COIL_HEAD_DOOR_OPEN_SPEED: I32F32 = I32F32::lit("0.06");

pub const COIL_HEAD_CHASE_SECONDS: I32F32 = I32F32::lit("35");
pub const COIL_HEAD_REST_SECONDS: I32F32 = I32F32::lit("15");
pub const COIL_HEAD_IMMUNE_HEALTH: I32F32 = I32F32::lit("0");
pub const COIL_HEAD_MOVE_STEP_PER_TICK: I32F32 = I32F32::lit("1");
pub const COIL_HEAD_WATCH_RANGE: I32F32 = I32F32::lit("24");

pub const COIL_HEAD_DEPENDS_ON: [&str; 7] = [
    "lethal_company",
    "employee",
    "stun_grenade",
    "diy_flashbang",
    "bracken",
    "secure_door",
    "the_mineshaft",
];

pub const COIL_HEAD_FRONTMATTER_BEHAVIOR: [&str; 2] = ["Roaming", "Stalking"];

pub const COIL_HEAD_BEHAVIORAL_MECHANICS: [CoilHeadBehaviorRule; 18] = [
    CoilHeadBehaviorRule {
        condition: "the Coil-Head spawns",
        outcome: "it enters Roaming state",
    },
    CoilHeadBehaviorRule {
        condition: "no player is within unobstructed detection range",
        outcome: "it stays in Roaming state and wanders between two randomly selected areas",
    },
    CoilHeadBehaviorRule {
        condition: "the Coil-Head has a clear line of sight to an Employee",
        outcome: "it enters Stalking state",
    },
    CoilHeadBehaviorRule {
        condition: "in Stalking state and not being looked at by any Employee",
        outcome: "it moves and attacks",
    },
    CoilHeadBehaviorRule {
        condition: "the Coil-Head is being looked at by an Employee",
        outcome: "it cannot move or attack",
    },
    CoilHeadBehaviorRule {
        condition: "the target Employee dies or leaves the facility",
        outcome: "the Coil-Head immediately retargets another player inside",
    },
    CoilHeadBehaviorRule {
        condition: "no players remain inside the facility",
        outcome: "the Coil-Head returns to Roaming state",
    },
    CoilHeadBehaviorRule {
        condition: "the Coil-Head chases a player for 35 seconds",
        outcome: "it enters Resting state",
    },
    CoilHeadBehaviorRule {
        condition: "the Coil-Head enters Resting state",
        outcome: "it stays dormant for about 15 seconds",
    },
    CoilHeadBehaviorRule {
        condition: "the 15-second rest ends and a player is in line of sight",
        outcome: "it becomes aggressive again",
    },
    CoilHeadBehaviorRule {
        condition: "a Stun grenade is used on the Coil-Head",
        outcome: "it is temporarily disabled",
    },
    CoilHeadBehaviorRule {
        condition: "a DIY Flashbang is used on the Coil-Head",
        outcome: "it is temporarily disabled and the user takes damage",
    },
    CoilHeadBehaviorRule {
        condition: "multiple Employees alternate eye contact",
        outcome: "one can reposition while another keeps the Coil-Head contained",
    },
    CoilHeadBehaviorRule {
        condition: "doors are placed between the player and the Coil-Head",
        outcome: "its 0.06 door open speed buys time for escape",
    },
    CoilHeadBehaviorRule {
        condition: "the player breaks line of sight while airborne across a door",
        outcome: "the Coil-Head can drop back to Roaming state",
    },
    CoilHeadBehaviorRule {
        condition: "the Coil-Head is trapped behind a Secure Door at a dead end",
        outcome: "it can be contained with terminal support",
    },
    CoilHeadBehaviorRule {
        condition: "fighting in The Mineshaft",
        outcome: "the Coil-Head can follow an Employee to the elevator and trap them",
    },
    CoilHeadBehaviorRule {
        condition: "the Coil-Head is the unobserved attacker",
        outcome: "it deals 90 attack damage with a 0.2 attack speed for 450 DPS",
    },
];

pub struct CoilHeadPlugin;

impl Plugin for CoilHeadPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnCoilHeadEvent>()
            .add_event::<CoilHeadStateChangedEvent>()
            .add_event::<CoilHeadRetargetedEvent>()
            .add_event::<CoilHeadAttackEvent>()
            .add_event::<CoilHeadStunAppliedEvent>()
            .add_event::<CoilHeadStunAdjustedEvent>()
            .add_event::<CoilHeadDiyFlashbangEvent>()
            .add_event::<CoilHeadDiyFlashbangUserDamagedEvent>()
            .add_event::<CoilHeadDoorAttemptEvent>()
            .add_event::<CoilHeadDoorAttemptResolvedEvent>()
            .add_event::<CoilHeadContainedBehindSecureDoorEvent>()
            .add_event::<CoilHeadMineshaftElevatorTrapEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_coil_head,
                    coil_head_tick_stun,
                    coil_head_roam_without_detection,
                    coil_head_enter_stalking_on_line_of_sight,
                    coil_head_freeze_when_observed,
                    coil_head_retarget_or_roam,
                    coil_head_stalk_when_unobserved,
                    coil_head_rest_after_chase,
                    coil_head_wake_after_rest,
                    coil_head_airborne_door_line_of_sight_break,
                    coil_head_apply_stun_multiplier,
                    coil_head_apply_diy_flashbang,
                    coil_head_door_attempt_speed,
                    coil_head_secure_door_containment,
                    coil_head_mineshaft_elevator_trap,
                    coil_head_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoilHeadBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CoilHead;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CoilHeadEmployeeSensor {
    pub stable_id: u64,
    pub clear_line_of_sight_to_coil_head: bool,
    pub looking_at_coil_head: bool,
    pub is_inside_facility: bool,
    pub is_alive: bool,
    pub airborne_across_door: bool,
    pub alternating_eye_contact: bool,
    pub in_mineshaft_elevator: bool,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoilHeadRoamRoute {
    pub area_a: SimPosition,
    pub area_b: SimPosition,
    pub destination_index: u8,
}

impl Default for CoilHeadRoamRoute {
    fn default() -> Self {
        Self {
            area_a: SimPosition {
                x: I32F32::lit("0"),
                y: I32F32::lit("0"),
            },
            area_b: SimPosition {
                x: I32F32::lit("0"),
                y: I32F32::lit("0"),
            },
            destination_index: 1,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoilHeadTarget {
    pub has_target: bool,
    pub target_stable_id: u64,
    pub chase_ticks: u32,
    pub attack_cooldown_ticks: u32,
}

impl Default for CoilHeadTarget {
    fn default() -> Self {
        Self {
            has_target: false,
            target_stable_id: 0,
            chase_ticks: 0,
            attack_cooldown_ticks: 0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoilHeadRestTimer {
    pub remaining_ticks: u32,
}

impl Default for CoilHeadRestTimer {
    fn default() -> Self {
        Self { remaining_ticks: 0 }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoilHeadStunTimer {
    pub remaining_ticks: u32,
}

impl Default for CoilHeadStunTimer {
    fn default() -> Self {
        Self { remaining_ticks: 0 }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CoilHeadState {
    #[default]
    Roaming,
    Stalking,
    Resting,
}

#[derive(Bundle)]
pub struct CoilHeadBundle {
    pub name: Name,
    pub coil_head: CoilHead,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub state: CoilHeadState,
    pub route: CoilHeadRoamRoute,
    pub target: CoilHeadTarget,
    pub rest_timer: CoilHeadRestTimer,
    pub stun_timer: CoilHeadStunTimer,
}

impl CoilHeadBundle {
    pub fn new(event: SpawnCoilHeadEvent) -> Self {
        Self {
            name: Name::new(COIL_HEAD_NAME),
            coil_head: CoilHead,
            position: event.position,
            health: Health::full(COIL_HEAD_IMMUNE_HEALTH),
            stats: UnitStats {
                move_speed: COIL_HEAD_MOVE_STEP_PER_TICK,
                attack_range: I32F32::lit("0"),
                attack_damage: COIL_HEAD_ATTACK_DAMAGE,
                attack_speed: COIL_HEAD_ATTACK_SPEED_SECONDS,
                watch_range: COIL_HEAD_WATCH_RANGE,
            },
            state: CoilHeadState::Roaming,
            route: CoilHeadRoamRoute {
                area_a: event.roam_area_a,
                area_b: event.roam_area_b,
                destination_index: 1,
            },
            target: CoilHeadTarget::default(),
            rest_timer: CoilHeadRestTimer::default(),
            stun_timer: CoilHeadStunTimer::default(),
        }
    }
}

#[derive(Event, Clone, Copy, Debug)]
pub struct SpawnCoilHeadEvent {
    pub position: SimPosition,
    pub roam_area_a: SimPosition,
    pub roam_area_b: SimPosition,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoilHeadStateChangedEvent {
    pub coil_head: Entity,
    pub from: CoilHeadState,
    pub to: CoilHeadState,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoilHeadRetargetedEvent {
    pub coil_head: Entity,
    pub target_employee: Entity,
    pub target_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoilHeadAttackEvent {
    pub coil_head: Entity,
    pub employee: Entity,
    pub employee_stable_id: u64,
    pub damage: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoilHeadStunAppliedEvent {
    pub coil_head: Entity,
    pub base_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoilHeadStunAdjustedEvent {
    pub coil_head: Entity,
    pub base_ticks: u32,
    pub adjusted_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoilHeadDiyFlashbangEvent {
    pub coil_head: Entity,
    pub user: Entity,
    pub base_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoilHeadDiyFlashbangUserDamagedEvent {
    pub coil_head: Entity,
    pub user: Entity,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoilHeadDoorAttemptEvent {
    pub coil_head: Entity,
    pub door: Entity,
    pub base_open_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoilHeadDoorAttemptResolvedEvent {
    pub coil_head: Entity,
    pub door: Entity,
    pub adjusted_open_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoilHeadContainedBehindSecureDoorEvent {
    pub coil_head: Entity,
    pub secure_door: Entity,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoilHeadMineshaftElevatorTrapEvent {
    pub coil_head: Entity,
    pub employee: Entity,
    pub employee_stable_id: u64,
}

fn spawn_coil_head(
    mut commands: Commands,
    mut events: EventReader<SpawnCoilHeadEvent>,
    coil_heads: Query<(), With<CoilHead>>,
) {
    let mut spawned_count = coil_heads.iter().count();

    for event in events.read() {
        if spawned_count >= COIL_HEAD_MAX_SPAWNED {
            break;
        }

        commands.spawn(CoilHeadBundle::new(*event));
        spawned_count += 1;
    }
}

fn coil_head_tick_stun(mut coil_heads: Query<&mut CoilHeadStunTimer, With<CoilHead>>) {
    for mut stun in coil_heads.iter_mut() {
        if stun.remaining_ticks > 0 {
            stun.remaining_ticks -= 1;
        }
    }
}

fn coil_head_roam_without_detection(
    mut coil_heads: Query<
        (
            &mut SimPosition,
            &CoilHeadState,
            &CoilHeadStunTimer,
            &mut CoilHeadRoamRoute,
        ),
        With<CoilHead>,
    >,
    employees: Query<&CoilHeadEmployeeSensor>,
) {
    let player_detected = employees.iter().any(|employee| {
        employee.is_alive
            && employee.is_inside_facility
            && employee.clear_line_of_sight_to_coil_head
    });

    if player_detected {
        return;
    }

    for (mut position, state, stun, mut route) in coil_heads.iter_mut() {
        if *state != CoilHeadState::Roaming || stun.remaining_ticks > 0 {
            continue;
        }

        let destination = if route.destination_index == 0 {
            route.area_a
        } else {
            route.area_b
        };

        move_axis_toward(&mut position, destination, COIL_HEAD_MOVE_STEP_PER_TICK);

        if *position == destination {
            route.destination_index = 1 - route.destination_index;
        }
    }
}

fn coil_head_enter_stalking_on_line_of_sight(
    mut state_events: EventWriter<CoilHeadStateChangedEvent>,
    mut retarget_events: EventWriter<CoilHeadRetargetedEvent>,
    mut coil_heads: Query<(Entity, &mut CoilHeadState, &mut CoilHeadTarget), With<CoilHead>>,
    employees: Query<(Entity, &CoilHeadEmployeeSensor)>,
) {
    let Some((employee_entity, employee_sensor)) = first_visible_employee(&employees) else {
        return;
    };

    for (coil_head_entity, mut state, mut target) in coil_heads.iter_mut() {
        if *state == CoilHeadState::Resting {
            continue;
        }

        target.has_target = true;
        target.target_stable_id = employee_sensor.stable_id;
        target.chase_ticks = 0;

        retarget_events.send(CoilHeadRetargetedEvent {
            coil_head: coil_head_entity,
            target_employee: employee_entity,
            target_stable_id: employee_sensor.stable_id,
        });

        set_coil_head_state(
            coil_head_entity,
            &mut state,
            CoilHeadState::Stalking,
            &mut state_events,
        );
    }
}

fn coil_head_freeze_when_observed(
    mut coil_heads: Query<&mut CoilHeadTarget, With<CoilHead>>,
    employees: Query<&CoilHeadEmployeeSensor>,
) {
    if !employees.iter().any(|employee| {
        employee.is_alive && employee.is_inside_facility && employee.looking_at_coil_head
    }) {
        return;
    }

    for mut target in coil_heads.iter_mut() {
        target.attack_cooldown_ticks = 0;
    }
}

fn coil_head_retarget_or_roam(
    mut state_events: EventWriter<CoilHeadStateChangedEvent>,
    mut retarget_events: EventWriter<CoilHeadRetargetedEvent>,
    mut coil_heads: Query<(Entity, &mut CoilHeadState, &mut CoilHeadTarget), With<CoilHead>>,
    employees: Query<(Entity, &CoilHeadEmployeeSensor)>,
) {
    let any_inside = employees
        .iter()
        .any(|(_entity, employee)| employee.is_alive && employee.is_inside_facility);

    for (coil_head_entity, mut state, mut target) in coil_heads.iter_mut() {
        if *state != CoilHeadState::Stalking {
            continue;
        }

        if !any_inside {
            target.has_target = false;
            target.target_stable_id = 0;
            target.chase_ticks = 0;
            set_coil_head_state(
                coil_head_entity,
                &mut state,
                CoilHeadState::Roaming,
                &mut state_events,
            );
            continue;
        }

        if target_is_valid(target.target_stable_id, &employees) {
            continue;
        }

        if let Some((employee_entity, employee_sensor)) = first_inside_employee(&employees) {
            target.has_target = true;
            target.target_stable_id = employee_sensor.stable_id;
            target.chase_ticks = 0;

            retarget_events.send(CoilHeadRetargetedEvent {
                coil_head: coil_head_entity,
                target_employee: employee_entity,
                target_stable_id: employee_sensor.stable_id,
            });
        }
    }
}

fn coil_head_stalk_when_unobserved(
    sim_hz: Res<SimHz>,
    mut damage_events: EventWriter<IncomingDamageEvent>,
    mut attack_events: EventWriter<CoilHeadAttackEvent>,
    mut coil_heads: Query<
        (
            Entity,
            &mut SimPosition,
            &CoilHeadState,
            &CoilHeadStunTimer,
            &mut CoilHeadTarget,
            &UnitStats,
        ),
        With<CoilHead>,
    >,
    employees: Query<(Entity, &SimPosition, &CoilHeadEmployeeSensor), Without<CoilHead>>,
) {
    let observed = employees.iter().any(|(_entity, _position, employee)| {
        employee.is_alive && employee.is_inside_facility && employee.looking_at_coil_head
    });

    if observed {
        return;
    }

    let attack_cooldown_ticks = fixed_seconds_to_ticks(COIL_HEAD_ATTACK_SPEED_SECONDS, sim_hz.0);

    for (coil_head_entity, mut position, state, stun, mut target, stats) in coil_heads.iter_mut() {
        if *state != CoilHeadState::Stalking || stun.remaining_ticks > 0 || !target.has_target {
            continue;
        }

        let Some((employee_entity, employee_position, employee_sensor)) =
            employee_by_stable_id(target.target_stable_id, &employees)
        else {
            continue;
        };

        move_axis_toward(&mut position, employee_position, stats.move_speed);
        target.chase_ticks += 1;

        if target.attack_cooldown_ticks > 0 {
            target.attack_cooldown_ticks -= 1;
            continue;
        }

        attack_events.send(CoilHeadAttackEvent {
            coil_head: coil_head_entity,
            employee: employee_entity,
            employee_stable_id: employee_sensor.stable_id,
            damage: COIL_HEAD_ATTACK_DAMAGE,
        });
        damage_events.send(IncomingDamageEvent {
            target: employee_entity,
            raw_amount: COIL_HEAD_ATTACK_DAMAGE,
            damage_type: DamageType::Standard,
            source: coil_head_entity,
        });
        target.attack_cooldown_ticks = attack_cooldown_ticks;
    }
}

fn coil_head_rest_after_chase(
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<CoilHeadStateChangedEvent>,
    mut coil_heads: Query<
        (
            Entity,
            &mut CoilHeadState,
            &mut CoilHeadTarget,
            &mut CoilHeadRestTimer,
        ),
        With<CoilHead>,
    >,
) {
    let chase_ticks = fixed_seconds_to_ticks(COIL_HEAD_CHASE_SECONDS, sim_hz.0);
    let rest_ticks = fixed_seconds_to_ticks(COIL_HEAD_REST_SECONDS, sim_hz.0);

    for (coil_head_entity, mut state, mut target, mut rest_timer) in coil_heads.iter_mut() {
        if *state != CoilHeadState::Stalking || target.chase_ticks < chase_ticks {
            continue;
        }

        target.has_target = false;
        target.target_stable_id = 0;
        target.chase_ticks = 0;
        target.attack_cooldown_ticks = 0;
        rest_timer.remaining_ticks = rest_ticks;

        set_coil_head_state(
            coil_head_entity,
            &mut state,
            CoilHeadState::Resting,
            &mut state_events,
        );
    }
}

fn coil_head_wake_after_rest(
    mut state_events: EventWriter<CoilHeadStateChangedEvent>,
    mut retarget_events: EventWriter<CoilHeadRetargetedEvent>,
    mut coil_heads: Query<
        (
            Entity,
            &mut CoilHeadState,
            &mut CoilHeadTarget,
            &mut CoilHeadRestTimer,
        ),
        With<CoilHead>,
    >,
    employees: Query<(Entity, &CoilHeadEmployeeSensor)>,
) {
    for (coil_head_entity, mut state, mut target, mut rest_timer) in coil_heads.iter_mut() {
        if *state != CoilHeadState::Resting {
            continue;
        }

        if rest_timer.remaining_ticks > 0 {
            rest_timer.remaining_ticks -= 1;
            continue;
        }

        if let Some((employee_entity, employee_sensor)) = first_visible_employee(&employees) {
            target.has_target = true;
            target.target_stable_id = employee_sensor.stable_id;
            target.chase_ticks = 0;

            retarget_events.send(CoilHeadRetargetedEvent {
                coil_head: coil_head_entity,
                target_employee: employee_entity,
                target_stable_id: employee_sensor.stable_id,
            });

            set_coil_head_state(
                coil_head_entity,
                &mut state,
                CoilHeadState::Stalking,
                &mut state_events,
            );
        } else {
            set_coil_head_state(
                coil_head_entity,
                &mut state,
                CoilHeadState::Roaming,
                &mut state_events,
            );
        }
    }
}

fn coil_head_airborne_door_line_of_sight_break(
    mut state_events: EventWriter<CoilHeadStateChangedEvent>,
    mut coil_heads: Query<(Entity, &mut CoilHeadState, &mut CoilHeadTarget), With<CoilHead>>,
    employees: Query<&CoilHeadEmployeeSensor>,
) {
    let airborne_break = employees.iter().any(|employee| {
        employee.is_alive
            && employee.is_inside_facility
            && employee.airborne_across_door
            && !employee.clear_line_of_sight_to_coil_head
    });

    if !airborne_break {
        return;
    }

    for (coil_head_entity, mut state, mut target) in coil_heads.iter_mut() {
        if *state != CoilHeadState::Stalking {
            continue;
        }

        target.has_target = false;
        target.target_stable_id = 0;
        target.chase_ticks = 0;
        target.attack_cooldown_ticks = 0;

        set_coil_head_state(
            coil_head_entity,
            &mut state,
            CoilHeadState::Roaming,
            &mut state_events,
        );
    }
}

fn coil_head_apply_stun_multiplier(
    mut events: EventReader<CoilHeadStunAppliedEvent>,
    mut adjusted_events: EventWriter<CoilHeadStunAdjustedEvent>,
    mut coil_heads: Query<&mut CoilHeadStunTimer, With<CoilHead>>,
) {
    for event in events.read() {
        let Ok(mut stun) = coil_heads.get_mut(event.coil_head) else {
            continue;
        };

        let adjusted_ticks = fixed_ticks_scaled(event.base_ticks, COIL_HEAD_STUN_MULTIPLIER);
        stun.remaining_ticks = adjusted_ticks;

        adjusted_events.send(CoilHeadStunAdjustedEvent {
            coil_head: event.coil_head,
            base_ticks: event.base_ticks,
            adjusted_ticks,
        });
    }
}

fn coil_head_apply_diy_flashbang(
    mut events: EventReader<CoilHeadDiyFlashbangEvent>,
    mut adjusted_events: EventWriter<CoilHeadStunAdjustedEvent>,
    mut user_damaged_events: EventWriter<CoilHeadDiyFlashbangUserDamagedEvent>,
    mut coil_heads: Query<&mut CoilHeadStunTimer, With<CoilHead>>,
) {
    for event in events.read() {
        let Ok(mut stun) = coil_heads.get_mut(event.coil_head) else {
            continue;
        };

        let adjusted_ticks = fixed_ticks_scaled(event.base_ticks, COIL_HEAD_STUN_MULTIPLIER);
        stun.remaining_ticks = adjusted_ticks;

        adjusted_events.send(CoilHeadStunAdjustedEvent {
            coil_head: event.coil_head,
            base_ticks: event.base_ticks,
            adjusted_ticks,
        });
        user_damaged_events.send(CoilHeadDiyFlashbangUserDamagedEvent {
            coil_head: event.coil_head,
            user: event.user,
        });
    }
}

fn coil_head_door_attempt_speed(
    mut events: EventReader<CoilHeadDoorAttemptEvent>,
    mut resolved_events: EventWriter<CoilHeadDoorAttemptResolvedEvent>,
    coil_heads: Query<(), With<CoilHead>>,
) {
    for event in events.read() {
        if coil_heads.get(event.coil_head).is_err() {
            continue;
        }

        resolved_events.send(CoilHeadDoorAttemptResolvedEvent {
            coil_head: event.coil_head,
            door: event.door,
            adjusted_open_ticks: fixed_ticks_scaled(event.base_open_ticks, COIL_HEAD_DOOR_OPEN_SPEED),
        });
    }
}

fn coil_head_secure_door_containment(
    mut events: EventReader<CoilHeadContainedBehindSecureDoorEvent>,
    mut coil_heads: Query<&mut CoilHeadTarget, With<CoilHead>>,
) {
    for event in events.read() {
        let Ok(mut target) = coil_heads.get_mut(event.coil_head) else {
            continue;
        };

        target.has_target = false;
        target.target_stable_id = 0;
        target.chase_ticks = 0;
        target.attack_cooldown_ticks = 0;
    }
}

fn coil_head_mineshaft_elevator_trap(
    mut trap_events: EventWriter<CoilHeadMineshaftElevatorTrapEvent>,
    coil_heads: Query<(Entity, &CoilHeadState, &CoilHeadTarget), With<CoilHead>>,
    employees: Query<(Entity, &CoilHeadEmployeeSensor)>,
) {
    for (coil_head_entity, state, target) in coil_heads.iter() {
        if *state != CoilHeadState::Stalking || !target.has_target {
            continue;
        }

        for (employee_entity, employee) in employees.iter() {
            if employee.stable_id != target.target_stable_id || !employee.in_mineshaft_elevator {
                continue;
            }

            trap_events.send(CoilHeadMineshaftElevatorTrapEvent {
                coil_head: coil_head_entity,
                employee: employee_entity,
                employee_stable_id: employee.stable_id,
            });
        }
    }
}

fn coil_head_checksum(
    mut checksum: ResMut<SimChecksumState>,
    coil_heads: Query<
        (
            &SimPosition,
            &Health,
            &UnitStats,
            &CoilHeadState,
            &CoilHeadRoamRoute,
            &CoilHeadTarget,
            &CoilHeadRestTimer,
            &CoilHeadStunTimer,
        ),
        With<CoilHead>,
    >,
) {
    for (position, health, stats, state, route, target, rest_timer, stun_timer) in coil_heads.iter() {
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);
        checksum.accumulate(coil_head_state_bits(*state));
        checksum.accumulate(route.area_a.x.to_bits() as u64);
        checksum.accumulate(route.area_a.y.to_bits() as u64);
        checksum.accumulate(route.area_b.x.to_bits() as u64);
        checksum.accumulate(route.area_b.y.to_bits() as u64);
        checksum.accumulate(route.destination_index as u64);
        checksum.accumulate(target.has_target as u64);
        checksum.accumulate(target.target_stable_id);
        checksum.accumulate(target.chase_ticks as u64);
        checksum.accumulate(target.attack_cooldown_ticks as u64);
        checksum.accumulate(rest_timer.remaining_ticks as u64);
        checksum.accumulate(stun_timer.remaining_ticks as u64);
    }
}

fn first_visible_employee(
    employees: &Query<(Entity, &CoilHeadEmployeeSensor)>,
) -> Option<(Entity, CoilHeadEmployeeSensor)> {
    let mut best: Option<(Entity, CoilHeadEmployeeSensor)> = None;

    for (entity, sensor) in employees.iter() {
        if !sensor.is_alive
            || !sensor.is_inside_facility
            || !sensor.clear_line_of_sight_to_coil_head
        {
            continue;
        }

        if let Some((_best_entity, best_sensor)) = best {
            if sensor.stable_id >= best_sensor.stable_id {
                continue;
            }
        }

        best = Some((entity, *sensor));
    }

    best
}

fn first_inside_employee(
    employees: &Query<(Entity, &CoilHeadEmployeeSensor)>,
) -> Option<(Entity, CoilHeadEmployeeSensor)> {
    let mut best: Option<(Entity, CoilHeadEmployeeSensor)> = None;

    for (entity, sensor) in employees.iter() {
        if !sensor.is_alive || !sensor.is_inside_facility {
            continue;
        }

        if let Some((_best_entity, best_sensor)) = best {
            if sensor.stable_id >= best_sensor.stable_id {
                continue;
            }
        }

        best = Some((entity, *sensor));
    }

    best
}

fn target_is_valid(
    stable_id: u64,
    employees: &Query<(Entity, &CoilHeadEmployeeSensor)>,
) -> bool {
    employees.iter().any(|(_entity, employee)| {
        employee.stable_id == stable_id && employee.is_alive && employee.is_inside_facility
    })
}

fn employee_by_stable_id(
    stable_id: u64,
    employees: &Query<(Entity, &SimPosition, &CoilHeadEmployeeSensor), Without<CoilHead>>,
) -> Option<(Entity, SimPosition, CoilHeadEmployeeSensor)> {
    for (entity, position, sensor) in employees.iter() {
        if sensor.stable_id == stable_id && sensor.is_alive && sensor.is_inside_facility {
            return Some((entity, *position, *sensor));
        }
    }

    None
}

fn set_coil_head_state(
    coil_head: Entity,
    state: &mut CoilHeadState,
    next: CoilHeadState,
    events: &mut EventWriter<CoilHeadStateChangedEvent>,
) {
    if *state == next {
        return;
    }

    let previous = *state;
    *state = next;
    events.send(CoilHeadStateChangedEvent {
        coil_head,
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

fn coil_head_state_bits(state: CoilHeadState) -> u64 {
    match state {
        CoilHeadState::Roaming => 0,
        CoilHeadState::Stalking => 1,
        CoilHeadState::Resting => 2,
    }
}
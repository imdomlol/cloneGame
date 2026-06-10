// Sources: vault/indoor_entity_pages/hoarding_bug.md, vault/version_pages/version_65.md, vault/version_pages/version_68.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{
    DamageType, Health, IncomingDamageEvent, SimChecksumState, SimHz, SimPosition, UnitStats,
};

pub const HOARDING_BUG_ID: &str = "hoarding_bug";
pub const HOARDING_BUG_NAME: &str = "Hoarding Bug";
pub const HOARDING_BUG_TYPE: &str = "indoor_entity_pages";
pub const HOARDING_BUG_SUBTYPE: &str = "Creature";
pub const HOARDING_BUG_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Hoarding_Bug";
pub const HOARDING_BUG_SOURCE_REVISION: u32 = 20525;
pub const HOARDING_BUG_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const HOARDING_BUG_CONFIDENCE_BASIS_POINTS: u16 = 88;

pub const VERSION_65_SOURCE_REVISION: u32 = 17356;
pub const VERSION_68_SOURCE_REVISION: u32 = 17417;
pub const VERSION_65_RELEASE_DATE: &str = "2024-10-23";
pub const VERSION_68_RELEASE_DATE: &str = "2024-11-02";

pub const HOARDING_BUG_DWELLS: &str = "Inside";
pub const HOARDING_BUG_SCIENTIFIC_NAME: &str = "Linepithema-crassus";
pub const HOARDING_BUG_ATTACK_DAMAGE: I32F32 = I32F32::lit("30");
pub const HOARDING_BUG_POWER_LEVEL: I32F32 = I32F32::lit("1");
pub const HOARDING_BUG_MAX_SPAWNED: usize = 10;
pub const HOARDING_BUG_DOOR_OPEN_SPEED: I32F32 = I32F32::lit("1.5");
pub const HOARDING_BUG_STUN_MULTIPLIER: I32F32 = I32F32::lit("0.3");
pub const HOARDING_BUG_SHOCK_RESPONSE: &str = "Susceptible";
pub const HOARDING_BUG_RADAR_PIP_SIZE: &str = "Small";
pub const HOARDING_BUG_SHOVEL_HP: I32F32 = I32F32::lit("3");
pub const HOARDING_BUG_INTERNAL_NAME: &str = "Lootbug";
pub const HOARDING_BUG_ZAP_GUN_DIFFICULTY: I32F32 = I32F32::lit("0.9");

pub const HOARDING_BUG_GUARD_TIMER_SECONDS: I32F32 = I32F32::lit("15");
pub const HOARDING_BUG_ROAMING_PROXIMITY_AGGRO_SECONDS: I32F32 = I32F32::lit("3");
pub const HOARDING_BUG_PROXIMITY_CHASE_SECONDS: I32F32 = I32F32::lit("4");
pub const HOARDING_BUG_HIT_LOS_BREAK_SECONDS: I32F32 = I32F32::lit("15");
pub const HOARDING_BUG_ROAM_SPEED: I32F32 = I32F32::lit("1");
pub const HOARDING_BUG_CHASE_SPEED: I32F32 = I32F32::lit("2");
pub const HOARDING_BUG_ATTACK_RANGE: I32F32 = I32F32::lit("1");
pub const HOARDING_BUG_ATTACK_SPEED: I32F32 = I32F32::lit("1");
pub const HOARDING_BUG_WATCH_RANGE: I32F32 = I32F32::lit("16");

pub const HOARDING_BUG_INFESTATION_CHANCE_BASIS_POINTS: u16 = 290;
pub const HOARDING_BUG_OCTOBER_23_INFESTATION_CHANCE_BASIS_POINTS: u16 = 150;
pub const VERSION_65_TEMPORARY_INFESTATION_CHANCE_BASIS_POINTS: u16 = 800;
pub const VERSION_65_ELIGIBLE_PLANET_INFESTATION_CHANCE_BASIS_POINTS: u16 = 900;
pub const VERSION_65_EQUALIZE_AFTER_SPAWNED: usize = 8;
pub const HOARDING_BUG_INFESTATION_POWER_LEVEL: I32F32 = I32F32::lit("30");
pub const HOARDING_BUG_INFESTATION_MIN_INDOOR_SPAWN_BONUS: u8 = 2;
pub const HOARDING_BUG_NUTCRACKER_REPLACEMENT_CHANCE_BASIS_POINTS: u16 = 2500;
pub const HOARDING_BUG_INFESTATION_FOG_CHANCE_BASIS_POINTS: u16 = 2000;
pub const HOARDING_BUG_NORMAL_FOG_CHANCE_BASIS_POINTS: u16 = 200;
pub const VERSION_65_TERMINAL_SALE_CHANCE_BASIS_POINTS: u16 = 6670;
pub const VERSION_65_OLD_TERMINAL_SALE_CHANCE_BASIS_POINTS: u16 = 2500;

pub const HOARDING_BUG_DEPENDS_ON: [&str; 3] = ["scrap", "employee", "nutcracker"];
pub const HOARDING_BUG_FRONTMATTER_BEHAVIOR: [&str; 2] = ["Looting", "Territorial"];

pub const HOARDING_BUG_BEHAVIORAL_MECHANICS: [HoardingBugBehaviorRule; 24] = [
    HoardingBugBehaviorRule {
        condition: "the hoarding bug is roaming",
        outcome: "it searches indoor rooms for scrap and may backtrack into rooms it already checked",
    },
    HoardingBugBehaviorRule {
        condition: "it finds an item, notices a player too close to its nest, or hears a player making noise near its nest",
        outcome: "it switches from roaming to guarding",
    },
    HoardingBugBehaviorRule {
        condition: "the hoarding bug is guarding",
        outcome: "it drops its carried item, stays on its nest, and follows a 15 second internal timer before returning to roaming",
    },
    HoardingBugBehaviorRule {
        condition: "it spots a player or hears noise while guarding",
        outcome: "the 15 second timer pauses until line of sight is broken or the noise stops",
    },
    HoardingBugBehaviorRule {
        condition: "a player comes too close or steals from the nest",
        outcome: "it enters aggro; while roaming this takes about 3 seconds, but while guarding it happens instantly",
    },
    HoardingBugBehaviorRule {
        condition: "proximity aggro is triggered",
        outcome: "it chases for a few seconds and then returns to guarding",
    },
    HoardingBugBehaviorRule {
        condition: "it is hit",
        outcome: "it enters a stronger aggro state and will chase until the player dies or the bug dies",
    },
    HoardingBugBehaviorRule {
        condition: "line of sight is broken for about 15 seconds after hit-based aggro",
        outcome: "it returns to guarding",
    },
    HoardingBugBehaviorRule {
        condition: "theft aggro is triggered by removing nest contents",
        outcome: "it remains aggressive until the player dies, the bug dies, or the same stolen scrap is returned",
    },
    HoardingBugBehaviorRule {
        condition: "each round on a moon that can spawn the entity rolls an infestation",
        outcome: "there is a 2.9% chance of a Hoarding Bug infestation",
    },
    HoardingBugBehaviorRule {
        condition: "the date is October 23 and the round rolls an infestation",
        outcome: "the infestation chance is 1.5%",
    },
    HoardingBugBehaviorRule {
        condition: "infestation is active",
        outcome: "indoor power level is set to 30 and the minimum indoor spawn amount increases by 2",
    },
    HoardingBugBehaviorRule {
        condition: "10 Hoarding Bugs have spawned",
        outcome: "remaining entity spawn weights are equalized because the max spawned count is 10",
    },
    HoardingBugBehaviorRule {
        condition: "an infestation is scheduled and nutcrackers can spawn on the moon",
        outcome: "there is a 25% chance the infestation becomes a nutcracker infestation instead",
    },
    HoardingBugBehaviorRule {
        condition: "the hoarding bug is attacking",
        outcome: "it deals 30 damage and its stun multiplier is 0.3",
    },
    HoardingBugBehaviorRule {
        condition: "the hoarding bug is being opened against a door",
        outcome: "its door_open_speed multiplier is 1.5",
    },
    HoardingBugBehaviorRule {
        condition: "the hoarding bug is affected by a zap gun",
        outcome: "its zap_gun_difficulty is 0.9",
    },
    HoardingBugBehaviorRule {
        condition: "a planet can spawn Hoarding Bugs in version 65",
        outcome: "each round has a 9% chance to become a Hoarding Bug infestation round",
    },
    HoardingBugBehaviorRule {
        condition: "the temporary anniversary summary is used in version 65",
        outcome: "there is an 8% chance for a hoarding bug infestation",
    },
    HoardingBugBehaviorRule {
        condition: "8 Hoarding Bugs have spawned in the version 65 rule text",
        outcome: "all other entities' spawn weights are equalized",
    },
    HoardingBugBehaviorRule {
        condition: "an infestation is happening in version 68",
        outcome: "the indoor fog chance is 20%",
    },
    HoardingBugBehaviorRule {
        condition: "an infestation is not happening in version 68",
        outcome: "the indoor fog chance is 2%",
    },
    HoardingBugBehaviorRule {
        condition: "the indoor fog system initializes in version 68",
        outcome: "it is no longer enabled by default",
    },
    HoardingBugBehaviorRule {
        condition: "Halloween occurs in version 68",
        outcome: "the hoarding bug receives its Halloween costume",
    },
];

pub struct HoardingBugPlugin;

impl Plugin for HoardingBugPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnHoardingBugEvent>()
            .add_event::<HoardingBugStateChangedEvent>()
            .add_event::<HoardingBugScrapFoundEvent>()
            .add_event::<HoardingBugNoiseNearNestEvent>()
            .add_event::<HoardingBugNestTheftEvent>()
            .add_event::<HoardingBugStolenScrapReturnedEvent>()
            .add_event::<HoardingBugDoorAttemptEvent>()
            .add_event::<HoardingBugDoorAttemptResolvedEvent>()
            .add_event::<HoardingBugStunAppliedEvent>()
            .add_event::<HoardingBugStunAdjustedEvent>()
            .add_event::<HoardingBugZapGunTargetedEvent>()
            .add_event::<HoardingBugZapGunDifficultyEvent>()
            .add_event::<HoardingBugAttackEvent>()
            .add_event::<HoardingBugInfestationResolvedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_hoarding_bug,
                    hoarding_bug_guard_from_scrap_found,
                    hoarding_bug_guard_from_noise,
                    hoarding_bug_guard_from_sensor,
                    hoarding_bug_tick_guard_timer,
                    hoarding_bug_start_proximity_aggro,
                    hoarding_bug_start_theft_aggro,
                    hoarding_bug_start_hit_aggro,
                    hoarding_bug_chase_target,
                    hoarding_bug_tick_proximity_aggro,
                    hoarding_bug_tick_hit_aggro_los,
                    hoarding_bug_return_stolen_scrap,
                    hoarding_bug_attack,
                    hoarding_bug_door_attempt_speed,
                    hoarding_bug_apply_stun_multiplier,
                    hoarding_bug_report_zap_gun_difficulty,
                    hoarding_bug_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HoardingBugBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HoardingBug;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HoardingBugNest {
    pub position: SimPosition,
    pub stored_scrap_count: u16,
}

impl Default for HoardingBugNest {
    fn default() -> Self {
        Self {
            position: SimPosition {
                x: I32F32::lit("0"),
                y: I32F32::lit("0"),
            },
            stored_scrap_count: 0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HoardingBugCarry {
    pub has_item: bool,
    pub scrap_stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HoardingBugTarget {
    pub has_target: bool,
    pub employee: Entity,
    pub employee_stable_id: u64,
    pub target_position: SimPosition,
}

impl Default for HoardingBugTarget {
    fn default() -> Self {
        Self {
            has_target: false,
            employee: Entity::PLACEHOLDER,
            employee_stable_id: 0,
            target_position: SimPosition {
                x: I32F32::lit("0"),
                y: I32F32::lit("0"),
            },
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HoardingBugEmployeeSensor {
    pub stable_id: u64,
    pub too_close_to_nest: bool,
    pub spotted_by_bug: bool,
    pub making_noise_near_nest: bool,
    pub line_of_sight_to_bug: bool,
    pub alive: bool,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HoardingBugTimers {
    pub guard_ticks_remaining: u32,
    pub roaming_proximity_ticks: u32,
    pub proximity_chase_ticks_remaining: u32,
    pub hit_los_break_ticks: u32,
}

impl Default for HoardingBugTimers {
    fn default() -> Self {
        Self {
            guard_ticks_remaining: 0,
            roaming_proximity_ticks: 0,
            proximity_chase_ticks_remaining: 0,
            hit_los_break_ticks: 0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HoardingBugInfestationState {
    pub active: bool,
    pub replacement_is_nutcracker: bool,
    pub indoor_power_level: I32F32,
    pub min_indoor_spawn_bonus: u8,
    pub fog_chance_basis_points: u16,
    pub spawned_count_before_equalized_weights: u8,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HoardingBugState {
    #[default]
    Roaming,
    Guarding,
    ProximityAggro,
    HitAggro,
    TheftAggro,
}

#[derive(Bundle)]
pub struct HoardingBugBundle {
    pub name: Name,
    pub hoarding_bug: HoardingBug,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub state: HoardingBugState,
    pub nest: HoardingBugNest,
    pub carry: HoardingBugCarry,
    pub target: HoardingBugTarget,
    pub timers: HoardingBugTimers,
}

impl HoardingBugBundle {
    pub fn new(event: SpawnHoardingBugEvent) -> Self {
        Self {
            name: Name::new(HOARDING_BUG_NAME),
            hoarding_bug: HoardingBug,
            position: event.position,
            health: Health::full(HOARDING_BUG_SHOVEL_HP),
            stats: UnitStats {
                move_speed: HOARDING_BUG_ROAM_SPEED,
                attack_range: HOARDING_BUG_ATTACK_RANGE,
                attack_damage: HOARDING_BUG_ATTACK_DAMAGE,
                attack_speed: HOARDING_BUG_ATTACK_SPEED,
                watch_range: HOARDING_BUG_WATCH_RANGE,
            },
            state: HoardingBugState::Roaming,
            nest: HoardingBugNest {
                position: event.nest_position,
                stored_scrap_count: event.initial_nest_scrap_count,
            },
            carry: HoardingBugCarry::default(),
            target: HoardingBugTarget::default(),
            timers: HoardingBugTimers::default(),
        }
    }
}

#[derive(Event, Clone, Copy, Debug)]
pub struct SpawnHoardingBugEvent {
    pub position: SimPosition,
    pub nest_position: SimPosition,
    pub initial_nest_scrap_count: u16,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HoardingBugStateChangedEvent {
    pub hoarding_bug: Entity,
    pub from: HoardingBugState,
    pub to: HoardingBugState,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HoardingBugScrapFoundEvent {
    pub hoarding_bug: Entity,
    pub scrap_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HoardingBugNoiseNearNestEvent {
    pub hoarding_bug: Entity,
    pub employee: Entity,
    pub employee_stable_id: u64,
    pub employee_position: SimPosition,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HoardingBugNestTheftEvent {
    pub hoarding_bug: Entity,
    pub employee: Entity,
    pub employee_stable_id: u64,
    pub scrap_stable_id: u64,
    pub employee_position: SimPosition,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HoardingBugStolenScrapReturnedEvent {
    pub hoarding_bug: Entity,
    pub scrap_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HoardingBugDoorAttemptEvent {
    pub hoarding_bug: Entity,
    pub door: Entity,
    pub base_open_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HoardingBugDoorAttemptResolvedEvent {
    pub hoarding_bug: Entity,
    pub door: Entity,
    pub adjusted_open_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HoardingBugStunAppliedEvent {
    pub hoarding_bug: Entity,
    pub base_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HoardingBugStunAdjustedEvent {
    pub hoarding_bug: Entity,
    pub base_ticks: u32,
    pub adjusted_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HoardingBugZapGunTargetedEvent {
    pub hoarding_bug: Entity,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HoardingBugZapGunDifficultyEvent {
    pub hoarding_bug: Entity,
    pub difficulty_modifier: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HoardingBugAttackEvent {
    pub hoarding_bug: Entity,
    pub employee: Entity,
    pub employee_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HoardingBugInfestationResolvedEvent {
    pub infestation_active: bool,
    pub replacement_is_nutcracker: bool,
    pub infestation_chance_basis_points: u16,
    pub indoor_power_level: I32F32,
    pub min_indoor_spawn_bonus: u8,
    pub fog_chance_basis_points: u16,
}

fn spawn_hoarding_bug(
    mut commands: Commands,
    mut events: EventReader<SpawnHoardingBugEvent>,
    hoarding_bugs: Query<(), With<HoardingBug>>,
) {
    let mut spawned_count = hoarding_bugs.iter().count();

    for event in events.read() {
        if spawned_count >= HOARDING_BUG_MAX_SPAWNED {
            break;
        }

        commands.spawn(HoardingBugBundle::new(*event));
        spawned_count += 1;
    }
}

fn hoarding_bug_guard_from_scrap_found(
    sim_hz: Res<SimHz>,
    mut events: EventReader<HoardingBugScrapFoundEvent>,
    mut state_events: EventWriter<HoardingBugStateChangedEvent>,
    mut bugs: Query<(
        Entity,
        &mut HoardingBugState,
        &mut HoardingBugCarry,
        &mut HoardingBugTimers,
    ), With<HoardingBug>>,
) {
    for event in events.read() {
        let Ok((bug_entity, mut state, mut carry, mut timers)) = bugs.get_mut(event.hoarding_bug) else {
            continue;
        };

        if *state != HoardingBugState::Roaming {
            continue;
        }

        carry.has_item = true;
        carry.scrap_stable_id = event.scrap_stable_id;
        timers.guard_ticks_remaining = fixed_seconds_to_ticks(HOARDING_BUG_GUARD_TIMER_SECONDS, sim_hz.0);
        set_hoarding_bug_state(
            bug_entity,
            &mut state,
            HoardingBugState::Guarding,
            &mut state_events,
        );
    }
}

fn hoarding_bug_guard_from_noise(
    sim_hz: Res<SimHz>,
    mut events: EventReader<HoardingBugNoiseNearNestEvent>,
    mut state_events: EventWriter<HoardingBugStateChangedEvent>,
    mut bugs: Query<(
        Entity,
        &mut HoardingBugState,
        &mut HoardingBugCarry,
        &mut HoardingBugTarget,
        &mut HoardingBugTimers,
    ), With<HoardingBug>>,
) {
    for event in events.read() {
        let Ok((bug_entity, mut state, mut carry, mut target, mut timers)) = bugs.get_mut(event.hoarding_bug) else {
            continue;
        };

        target.has_target = true;
        target.employee = event.employee;
        target.employee_stable_id = event.employee_stable_id;
        target.target_position = event.employee_position;

        if *state == HoardingBugState::Roaming {
            carry.has_item = false;
            carry.scrap_stable_id = 0;
            timers.guard_ticks_remaining = fixed_seconds_to_ticks(HOARDING_BUG_GUARD_TIMER_SECONDS, sim_hz.0);
            set_hoarding_bug_state(
                bug_entity,
                &mut state,
                HoardingBugState::Guarding,
                &mut state_events,
            );
        }
    }
}

fn hoarding_bug_guard_from_sensor(
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<HoardingBugStateChangedEvent>,
    mut bugs: Query<(
        Entity,
        &mut HoardingBugState,
        &mut HoardingBugCarry,
        &mut HoardingBugTarget,
        &mut HoardingBugTimers,
    ), With<HoardingBug>>,
    employees: Query<(Entity, &SimPosition, &HoardingBugEmployeeSensor), Without<HoardingBug>>,
) {
    let Some((employee_entity, employee_position, employee_sensor)) = nearest_sensor_employee(&employees) else {
        return;
    };

    for (bug_entity, mut state, mut carry, mut target, mut timers) in bugs.iter_mut() {
        target.has_target = true;
        target.employee = employee_entity;
        target.employee_stable_id = employee_sensor.stable_id;
        target.target_position = employee_position;

        if *state != HoardingBugState::Roaming {
            continue;
        }

        carry.has_item = false;
        carry.scrap_stable_id = 0;
        timers.guard_ticks_remaining = fixed_seconds_to_ticks(HOARDING_BUG_GUARD_TIMER_SECONDS, sim_hz.0);
        set_hoarding_bug_state(
            bug_entity,
            &mut state,
            HoardingBugState::Guarding,
            &mut state_events,
        );
    }
}

fn hoarding_bug_tick_guard_timer(
    mut state_events: EventWriter<HoardingBugStateChangedEvent>,
    mut bugs: Query<(Entity, &mut HoardingBugState, &mut HoardingBugTimers), With<HoardingBug>>,
    employees: Query<&HoardingBugEmployeeSensor, Without<HoardingBug>>,
) {
    let guard_paused = employees
        .iter()
        .any(|sensor| sensor.spotted_by_bug || sensor.making_noise_near_nest);

    for (bug_entity, mut state, mut timers) in bugs.iter_mut() {
        if *state != HoardingBugState::Guarding {
            continue;
        }

        if guard_paused {
            continue;
        }

        if timers.guard_ticks_remaining > 0 {
            timers.guard_ticks_remaining -= 1;
            continue;
        }

        set_hoarding_bug_state(
            bug_entity,
            &mut state,
            HoardingBugState::Roaming,
            &mut state_events,
        );
    }
}

fn hoarding_bug_start_proximity_aggro(
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<HoardingBugStateChangedEvent>,
    mut bugs: Query<(
        Entity,
        &mut HoardingBugState,
        &mut HoardingBugTarget,
        &mut HoardingBugTimers,
        &mut UnitStats,
    ), With<HoardingBug>>,
    employees: Query<(Entity, &SimPosition, &HoardingBugEmployeeSensor), Without<HoardingBug>>,
) {
    let Some((employee_entity, employee_position, employee_sensor)) = too_close_employee(&employees) else {
        return;
    };

    for (bug_entity, mut state, mut target, mut timers, mut stats) in bugs.iter_mut() {
        if *state != HoardingBugState::Roaming && *state != HoardingBugState::Guarding {
            continue;
        }

        target.has_target = true;
        target.employee = employee_entity;
        target.employee_stable_id = employee_sensor.stable_id;
        target.target_position = employee_position;

        if *state == HoardingBugState::Roaming {
            timers.roaming_proximity_ticks += 1;
            let needed_ticks = fixed_seconds_to_ticks(HOARDING_BUG_ROAMING_PROXIMITY_AGGRO_SECONDS, sim_hz.0);
            if timers.roaming_proximity_ticks < needed_ticks {
                continue;
            }
        }

        timers.proximity_chase_ticks_remaining =
            fixed_seconds_to_ticks(HOARDING_BUG_PROXIMITY_CHASE_SECONDS, sim_hz.0);
        stats.move_speed = HOARDING_BUG_CHASE_SPEED;
        set_hoarding_bug_state(
            bug_entity,
            &mut state,
            HoardingBugState::ProximityAggro,
            &mut state_events,
        );
    }
}

fn hoarding_bug_start_theft_aggro(
    mut events: EventReader<HoardingBugNestTheftEvent>,
    mut state_events: EventWriter<HoardingBugStateChangedEvent>,
    mut bugs: Query<(
        Entity,
        &mut HoardingBugState,
        &mut HoardingBugCarry,
        &mut HoardingBugTarget,
        &mut UnitStats,
    ), With<HoardingBug>>,
) {
    for event in events.read() {
        let Ok((bug_entity, mut state, mut carry, mut target, mut stats)) = bugs.get_mut(event.hoarding_bug) else {
            continue;
        };

        carry.has_item = true;
        carry.scrap_stable_id = event.scrap_stable_id;
        target.has_target = true;
        target.employee = event.employee;
        target.employee_stable_id = event.employee_stable_id;
        target.target_position = event.employee_position;
        stats.move_speed = HOARDING_BUG_CHASE_SPEED;

        set_hoarding_bug_state(
            bug_entity,
            &mut state,
            HoardingBugState::TheftAggro,
            &mut state_events,
        );
    }
}

fn hoarding_bug_start_hit_aggro(
    sim_hz: Res<SimHz>,
    mut damage_events: EventReader<IncomingDamageEvent>,
    mut state_events: EventWriter<HoardingBugStateChangedEvent>,
    mut bugs: Query<(
        Entity,
        &mut HoardingBugState,
        &mut HoardingBugTarget,
        &mut HoardingBugTimers,
        &mut UnitStats,
    ), With<HoardingBug>>,
    employees: Query<(Entity, &SimPosition, &HoardingBugEmployeeSensor), Without<HoardingBug>>,
) {
    for damage in damage_events.read() {
        let Ok((bug_entity, mut state, mut target, mut timers, mut stats)) = bugs.get_mut(damage.target) else {
            continue;
        };

        let Some((employee_entity, employee_position, employee_sensor)) =
            employee_by_entity(damage.source, &employees)
        else {
            continue;
        };

        target.has_target = true;
        target.employee = employee_entity;
        target.employee_stable_id = employee_sensor.stable_id;
        target.target_position = employee_position;
        timers.hit_los_break_ticks = fixed_seconds_to_ticks(HOARDING_BUG_HIT_LOS_BREAK_SECONDS, sim_hz.0);
        stats.move_speed = HOARDING_BUG_CHASE_SPEED;

        set_hoarding_bug_state(
            bug_entity,
            &mut state,
            HoardingBugState::HitAggro,
            &mut state_events,
        );
    }
}

fn hoarding_bug_chase_target(
    sim_hz: Res<SimHz>,
    mut bugs: Query<
        (&mut SimPosition, &HoardingBugState, &UnitStats, &mut HoardingBugTarget),
        With<HoardingBug>,
    >,
    employees: Query<(&SimPosition, &HoardingBugEmployeeSensor), Without<HoardingBug>>,
) {
    for (mut position, state, stats, mut target) in bugs.iter_mut() {
        if *state != HoardingBugState::ProximityAggro
            && *state != HoardingBugState::HitAggro
            && *state != HoardingBugState::TheftAggro
        {
            continue;
        }

        if !target.has_target {
            continue;
        }

        if let Some(employee_position) = employee_position_by_stable_id(target.employee_stable_id, &employees) {
            target.target_position = employee_position;
        }

        move_axis_toward(&mut position, target.target_position, stats.move_speed / sim_hz.0);
    }
}

fn hoarding_bug_tick_proximity_aggro(
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<HoardingBugStateChangedEvent>,
    mut bugs: Query<(
        Entity,
        &mut HoardingBugState,
        &mut HoardingBugTimers,
        &mut UnitStats,
    ), With<HoardingBug>>,
) {
    for (bug_entity, mut state, mut timers, mut stats) in bugs.iter_mut() {
        if *state != HoardingBugState::ProximityAggro {
            continue;
        }

        if timers.proximity_chase_ticks_remaining > 0 {
            timers.proximity_chase_ticks_remaining -= 1;
            continue;
        }

        timers.guard_ticks_remaining = fixed_seconds_to_ticks(HOARDING_BUG_GUARD_TIMER_SECONDS, sim_hz.0);
        stats.move_speed = HOARDING_BUG_ROAM_SPEED;
        set_hoarding_bug_state(
            bug_entity,
            &mut state,
            HoardingBugState::Guarding,
            &mut state_events,
        );
    }
}

fn hoarding_bug_tick_hit_aggro_los(
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<HoardingBugStateChangedEvent>,
    mut bugs: Query<(
        Entity,
        &mut HoardingBugState,
        &HoardingBugTarget,
        &mut HoardingBugTimers,
        &mut UnitStats,
    ), With<HoardingBug>>,
    employees: Query<&HoardingBugEmployeeSensor, Without<HoardingBug>>,
) {
    for (bug_entity, mut state, target, mut timers, mut stats) in bugs.iter_mut() {
        if *state != HoardingBugState::HitAggro || !target.has_target {
            continue;
        }

        let target_visible = employees
            .iter()
            .any(|sensor| sensor.stable_id == target.employee_stable_id && sensor.line_of_sight_to_bug);

        if target_visible {
            timers.hit_los_break_ticks = fixed_seconds_to_ticks(HOARDING_BUG_HIT_LOS_BREAK_SECONDS, sim_hz.0);
            continue;
        }

        if timers.hit_los_break_ticks > 0 {
            timers.hit_los_break_ticks -= 1;
            continue;
        }

        timers.guard_ticks_remaining = fixed_seconds_to_ticks(HOARDING_BUG_GUARD_TIMER_SECONDS, sim_hz.0);
        stats.move_speed = HOARDING_BUG_ROAM_SPEED;
        set_hoarding_bug_state(
            bug_entity,
            &mut state,
            HoardingBugState::Guarding,
            &mut state_events,
        );
    }
}

fn hoarding_bug_return_stolen_scrap(
    sim_hz: Res<SimHz>,
    mut events: EventReader<HoardingBugStolenScrapReturnedEvent>,
    mut state_events: EventWriter<HoardingBugStateChangedEvent>,
    mut bugs: Query<(
        Entity,
        &mut HoardingBugState,
        &mut HoardingBugCarry,
        &mut HoardingBugTimers,
        &mut UnitStats,
    ), With<HoardingBug>>,
) {
    for event in events.read() {
        let Ok((bug_entity, mut state, mut carry, mut timers, mut stats)) = bugs.get_mut(event.hoarding_bug) else {
            continue;
        };

        if *state != HoardingBugState::TheftAggro {
            continue;
        }

        if !carry.has_item || carry.scrap_stable_id != event.scrap_stable_id {
            continue;
        }

        carry.has_item = false;
        carry.scrap_stable_id = 0;
        timers.guard_ticks_remaining = fixed_seconds_to_ticks(HOARDING_BUG_GUARD_TIMER_SECONDS, sim_hz.0);
        stats.move_speed = HOARDING_BUG_ROAM_SPEED;

        set_hoarding_bug_state(
            bug_entity,
            &mut state,
            HoardingBugState::Guarding,
            &mut state_events,
        );
    }
}

fn hoarding_bug_attack(
    mut attack_events: EventWriter<HoardingBugAttackEvent>,
    mut damage_events: EventWriter<IncomingDamageEvent>,
    bugs: Query<(Entity, &HoardingBugState, &HoardingBugTarget), With<HoardingBug>>,
    employees: Query<&HoardingBugEmployeeSensor, Without<HoardingBug>>,
) {
    for (bug_entity, state, target) in bugs.iter() {
        if *state != HoardingBugState::ProximityAggro
            && *state != HoardingBugState::HitAggro
            && *state != HoardingBugState::TheftAggro
        {
            continue;
        }

        if !target.has_target {
            continue;
        }

        let target_alive = employees
            .iter()
            .any(|sensor| sensor.stable_id == target.employee_stable_id && sensor.alive);

        if !target_alive {
            continue;
        }

        attack_events.send(HoardingBugAttackEvent {
            hoarding_bug: bug_entity,
            employee: target.employee,
            employee_stable_id: target.employee_stable_id,
        });
        damage_events.send(IncomingDamageEvent {
            target: target.employee,
            raw_amount: HOARDING_BUG_ATTACK_DAMAGE,
            damage_type: DamageType::Standard,
            source: bug_entity,
        });
    }
}

fn hoarding_bug_door_attempt_speed(
    mut events: EventReader<HoardingBugDoorAttemptEvent>,
    mut resolved_events: EventWriter<HoardingBugDoorAttemptResolvedEvent>,
    bugs: Query<(), With<HoardingBug>>,
) {
    for event in events.read() {
        if bugs.get(event.hoarding_bug).is_err() {
            continue;
        }

        resolved_events.send(HoardingBugDoorAttemptResolvedEvent {
            hoarding_bug: event.hoarding_bug,
            door: event.door,
            adjusted_open_ticks: fixed_ticks_scaled(event.base_open_ticks, HOARDING_BUG_DOOR_OPEN_SPEED),
        });
    }
}

fn hoarding_bug_apply_stun_multiplier(
    mut events: EventReader<HoardingBugStunAppliedEvent>,
    mut adjusted_events: EventWriter<HoardingBugStunAdjustedEvent>,
) {
    for event in events.read() {
        adjusted_events.send(HoardingBugStunAdjustedEvent {
            hoarding_bug: event.hoarding_bug,
            base_ticks: event.base_ticks,
            adjusted_ticks: fixed_ticks_scaled(event.base_ticks, HOARDING_BUG_STUN_MULTIPLIER),
        });
    }
}

fn hoarding_bug_report_zap_gun_difficulty(
    mut events: EventReader<HoardingBugZapGunTargetedEvent>,
    mut difficulty_events: EventWriter<HoardingBugZapGunDifficultyEvent>,
    bugs: Query<(), With<HoardingBug>>,
) {
    for event in events.read() {
        if bugs.get(event.hoarding_bug).is_err() {
            continue;
        }

        difficulty_events.send(HoardingBugZapGunDifficultyEvent {
            hoarding_bug: event.hoarding_bug,
            difficulty_modifier: HOARDING_BUG_ZAP_GUN_DIFFICULTY,
        });
    }
}

fn hoarding_bug_checksum(
    mut checksum: ResMut<SimChecksumState>,
    bugs: Query<(
        &SimPosition,
        &Health,
        &UnitStats,
        &HoardingBugState,
        &HoardingBugNest,
        &HoardingBugCarry,
        &HoardingBugTarget,
        &HoardingBugTimers,
    ), With<HoardingBug>>,
) {
    for (position, health, stats, state, nest, carry, target, timers) in bugs.iter() {
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);
        checksum.accumulate(hoarding_bug_state_bits(*state));
        checksum.accumulate(nest.position.x.to_bits() as u64);
        checksum.accumulate(nest.position.y.to_bits() as u64);
        checksum.accumulate(nest.stored_scrap_count as u64);
        checksum.accumulate(carry.has_item as u64);
        checksum.accumulate(carry.scrap_stable_id);
        checksum.accumulate(target.has_target as u64);
        checksum.accumulate(target.employee_stable_id);
        checksum.accumulate(target.target_position.x.to_bits() as u64);
        checksum.accumulate(target.target_position.y.to_bits() as u64);
        checksum.accumulate(timers.guard_ticks_remaining as u64);
        checksum.accumulate(timers.roaming_proximity_ticks as u64);
        checksum.accumulate(timers.proximity_chase_ticks_remaining as u64);
        checksum.accumulate(timers.hit_los_break_ticks as u64);
    }
}

pub fn hoarding_bug_infestation_chance_basis_points(month: u8, day: u8) -> u16 {
    if month == 10 && day == 23 {
        HOARDING_BUG_OCTOBER_23_INFESTATION_CHANCE_BASIS_POINTS
    } else {
        HOARDING_BUG_INFESTATION_CHANCE_BASIS_POINTS
    }
}

pub fn hoarding_bug_fog_chance_basis_points(infestation_active: bool) -> u16 {
    if infestation_active {
        HOARDING_BUG_INFESTATION_FOG_CHANCE_BASIS_POINTS
    } else {
        HOARDING_BUG_NORMAL_FOG_CHANCE_BASIS_POINTS
    }
}

pub fn hoarding_bug_infestation_power_level(infestation_active: bool) -> I32F32 {
    if infestation_active {
        HOARDING_BUG_INFESTATION_POWER_LEVEL
    } else {
        HOARDING_BUG_POWER_LEVEL
    }
}

pub fn hoarding_bug_min_spawn_bonus(infestation_active: bool) -> u8 {
    if infestation_active {
        HOARDING_BUG_INFESTATION_MIN_INDOOR_SPAWN_BONUS
    } else {
        0
    }
}

pub fn hoarding_bug_should_equalize_spawn_weights(spawned_count: usize) -> bool {
    spawned_count >= HOARDING_BUG_MAX_SPAWNED
}

fn nearest_sensor_employee(
    employees: &Query<(Entity, &SimPosition, &HoardingBugEmployeeSensor), Without<HoardingBug>>,
) -> Option<(Entity, SimPosition, HoardingBugEmployeeSensor)> {
    let mut best: Option<(Entity, SimPosition, HoardingBugEmployeeSensor)> = None;

    for (entity, position, sensor) in employees.iter() {
        if !sensor.too_close_to_nest && !sensor.making_noise_near_nest {
            continue;
        }

        if let Some((_best_entity, _best_position, best_sensor)) = best {
            if sensor.stable_id >= best_sensor.stable_id {
                continue;
            }
        }

        best = Some((entity, *position, *sensor));
    }

    best
}

fn too_close_employee(
    employees: &Query<(Entity, &SimPosition, &HoardingBugEmployeeSensor), Without<HoardingBug>>,
) -> Option<(Entity, SimPosition, HoardingBugEmployeeSensor)> {
    let mut best: Option<(Entity, SimPosition, HoardingBugEmployeeSensor)> = None;

    for (entity, position, sensor) in employees.iter() {
        if !sensor.too_close_to_nest {
            continue;
        }

        if let Some((_best_entity, _best_position, best_sensor)) = best {
            if sensor.stable_id >= best_sensor.stable_id {
                continue;
            }
        }

        best = Some((entity, *position, *sensor));
    }

    best
}

fn employee_by_entity(
    entity: Entity,
    employees: &Query<(Entity, &SimPosition, &HoardingBugEmployeeSensor), Without<HoardingBug>>,
) -> Option<(Entity, SimPosition, HoardingBugEmployeeSensor)> {
    for (employee_entity, position, sensor) in employees.iter() {
        if employee_entity == entity {
            return Some((employee_entity, *position, *sensor));
        }
    }

    None
}

fn employee_position_by_stable_id(
    stable_id: u64,
    employees: &Query<(&SimPosition, &HoardingBugEmployeeSensor), Without<HoardingBug>>,
) -> Option<SimPosition> {
    for (position, sensor) in employees.iter() {
        if sensor.stable_id == stable_id {
            return Some(*position);
        }
    }

    None
}

fn set_hoarding_bug_state(
    hoarding_bug: Entity,
    state: &mut HoardingBugState,
    next: HoardingBugState,
    events: &mut EventWriter<HoardingBugStateChangedEvent>,
) {
    if *state == next {
        return;
    }

    let previous = *state;
    *state = next;
    events.send(HoardingBugStateChangedEvent {
        hoarding_bug,
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

fn hoarding_bug_state_bits(state: HoardingBugState) -> u64 {
    match state {
        HoardingBugState::Roaming => 0,
        HoardingBugState::Guarding => 1,
        HoardingBugState::ProximityAggro => 2,
        HoardingBugState::HitAggro => 3,
        HoardingBugState::TheftAggro => 4,
    }
}
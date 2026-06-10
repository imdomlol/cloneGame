// Sources: vault/outdoor_entity_pages/kidnapper_fox.md
use bevy::prelude::*;
use fixed::types::I32F32;
use rand_core::RngCore;

use crate::sim::{
    tick_rng, DamageType, GameSeed, Health, IncomingDamageEvent, SimChecksumState, SimHz,
    SimPosition, SimTick, UnitStats,
};

pub const KIDNAPPER_FOX_ID: &str = "kidnapper_fox";
pub const KIDNAPPER_FOX_NAME: &str = "Kidnapper Fox";
pub const KIDNAPPER_FOX_TYPE: &str = "outdoor_entity_pages";
pub const KIDNAPPER_FOX_SUBTYPE: &str = "entity";
pub const KIDNAPPER_FOX_SOURCE_URL: &str =
    "https://lethal-company.fandom.com/wiki/Kidnapper_Fox";
pub const KIDNAPPER_FOX_SOURCE_REVISION: u32 = 21469;
pub const KIDNAPPER_FOX_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const KIDNAPPER_FOX_CONFIDENCE_BASIS_POINTS: u16 = 88;

pub const KIDNAPPER_FOX_DWELLS: &str = "Outside";
pub const KIDNAPPER_FOX_DANGER: &str = "80%";
pub const KIDNAPPER_FOX_SCIENTIFIC_NAME: &str = "Vulpes raptor";
pub const KIDNAPPER_FOX_HP: I32F32 = I32F32::lit("7");
pub const KIDNAPPER_FOX_POWER_LEVEL: I32F32 = I32F32::lit("1");
pub const KIDNAPPER_FOX_MAX_SPAWNED: usize = 1;
pub const KIDNAPPER_FOX_STUN_MULTIPLIER: I32F32 = I32F32::lit("0.45");
pub const KIDNAPPER_FOX_ZAP_GUN_DIFFICULTY: I32F32 = I32F32::lit("0.8");
pub const KIDNAPPER_FOX_INTERNAL_NAME: &str = "BushWolf";
pub const KIDNAPPER_FOX_SPAWN_DELAY_HOURS: u32 = 2;
pub const KIDNAPPER_FOX_CONTACT_DAMAGE: I32F32 = I32F32::lit("0");

pub const KIDNAPPER_FOX_MIN_VAIN_SHROUDS_TO_SPAWN: u32 = 17;
pub const KIDNAPPER_FOX_SPAWN_CHANCE_DIVISOR: u32 = 50;
pub const KIDNAPPER_FOX_FIRST_RETRY_HOUR: u32 = 8;
pub const KIDNAPPER_FOX_HIDE_PATCH_SECONDS: I32F32 = I32F32::lit("80");
pub const KIDNAPPER_FOX_INDOOR_WAIT_SECONDS: I32F32 = I32F32::lit("18");
pub const KIDNAPPER_FOX_MEDIUM_STARE_RANGE: I32F32 = I32F32::lit("20");
pub const KIDNAPPER_FOX_NEST_SHRINK_PER_TICK: I32F32 = I32F32::lit("0.05");
pub const KIDNAPPER_FOX_DEFAULT_NEST_RADIUS: I32F32 = I32F32::lit("12");
pub const KIDNAPPER_FOX_MIN_NEST_RADIUS: I32F32 = I32F32::lit("2");
pub const KIDNAPPER_FOX_CHASE_KILL_AREA_MULTIPLIER: I32F32 = I32F32::lit("2.01");
pub const KIDNAPPER_FOX_HIDE_SPEED: I32F32 = I32F32::lit("0");
pub const KIDNAPPER_FOX_HUNT_SPEED: I32F32 = I32F32::lit("7");
pub const KIDNAPPER_FOX_DRAG_SPEED: I32F32 = I32F32::lit("4");
pub const KIDNAPPER_FOX_ATTACK_RANGE: I32F32 = I32F32::lit("2");
pub const KIDNAPPER_FOX_ATTACK_SPEED_SECONDS: I32F32 = I32F32::lit("1");
pub const KIDNAPPER_FOX_WATCH_RANGE: I32F32 = I32F32::lit("30");
pub const KIDNAPPER_FOX_SPAWN_SALT: u64 = 0xF0_7C_00_00_0000_0001;

pub const KIDNAPPER_FOX_DEPENDS_ON: [&str; 4] = [
    "employee",
    "vain_shroud",
    "weed_killer",
    "company_cruiser",
];

pub const KIDNAPPER_FOX_FRONTMATTER_BEHAVIOR: [&str; 4] = [
    "Hides in dense Vain Shroud patches before entering hunting behavior.",
    "Switches between hiding, hunting, and attacking states.",
    "Latches onto employees and drags them toward a kill area.",
    "Returns to hiding after being attacked or after killing an employee.",
];

pub const KIDNAPPER_FOX_BEHAVIORAL_MECHANICS: [KidnapperFoxBehaviorRule; 17] = [
    KidnapperFoxBehaviorRule {
        condition: "a moon has 17 or more vain_shrouds when the ship lands",
        outcome: "the fox can spawn, with spawn chance equal to vain_shroud_count / 50",
    },
    KidnapperFoxBehaviorRule {
        condition: "the ship has already landed",
        outcome: "the fox's spawn probability stays fixed for that run even if weed_killer removes vain_shrouds later",
    },
    KidnapperFoxBehaviorRule {
        condition: "the fox has not spawned yet",
        outcome: "the game retries the spawn every 2 in-game hours starting at 08:00",
    },
    KidnapperFoxBehaviorRule {
        condition: "one fox has already spawned in a run",
        outcome: "no second fox can spawn that day",
    },
    KidnapperFoxBehaviorRule {
        condition: "an employee enters the nest area",
        outcome: "the fox switches from hiding to hunting",
    },
    KidnapperFoxBehaviorRule {
        condition: "the fox is hiding",
        outcome: "it remains in the largest vain_shroud patch for 80 seconds before moving to the second-largest patch",
    },
    KidnapperFoxBehaviorRule {
        condition: "the fox is hunting an outdoor employee",
        outcome: "it tries to get behind the target and attack with its tongue",
    },
    KidnapperFoxBehaviorRule {
        condition: "the fox is hunting an indoor employee",
        outcome: "it waits 18 seconds outside before entering and latching on",
    },
    KidnapperFoxBehaviorRule {
        condition: "the fox latches onto an employee",
        outcome: "that employee drops all held items and the fox drags them toward the kill area",
    },
    KidnapperFoxBehaviorRule {
        condition: "the fox is being stared down at medium range",
        outcome: "the nest radius shrinks, and in multiplayer all but one employee must keep staring to reduce it",
    },
    KidnapperFoxBehaviorRule {
        condition: "2 or more employees are not staring at the fox",
        outcome: "staring no longer shrinks the nest area",
    },
    KidnapperFoxBehaviorRule {
        condition: "the fox is chasing a target",
        outcome: "the kill area radius more than doubles",
    },
    KidnapperFoxBehaviorRule {
        condition: "the fox is attacked or kills an employee",
        outcome: "it retreats to hiding",
    },
    KidnapperFoxBehaviorRule {
        condition: "the tongue is used during a latch",
        outcome: "it deals 0 damage",
    },
    KidnapperFoxBehaviorRule {
        condition: "a ship door closes on the fox's tongue while an employee is being dragged",
        outcome: "the fox lets go, provided another employee can operate the buttons",
    },
    KidnapperFoxBehaviorRule {
        condition: "an employee is seated inside the company_cruiser",
        outcome: "the fox cannot latch on",
    },
    KidnapperFoxBehaviorRule {
        condition: "the fox is killed",
        outcome: "it is removed as a threat for the rest of the day",
    },
];

pub struct KidnapperFoxPlugin;

impl Plugin for KidnapperFoxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<KidnapperFoxSpawnState>()
            .add_event::<SpawnKidnapperFoxEvent>()
            .add_event::<KidnapperFoxShipLandedEvent>()
            .add_event::<KidnapperFoxSpawnGateEvent>()
            .add_event::<KidnapperFoxStateChangedEvent>()
            .add_event::<KidnapperFoxLatchedEvent>()
            .add_event::<KidnapperFoxDraggedEvent>()
            .add_event::<KidnapperFoxEmployeeItemsDroppedEvent>()
            .add_event::<KidnapperFoxTongueReleasedEvent>()
            .add_event::<KidnapperFoxNestRadiusChangedEvent>()
            .add_event::<KidnapperFoxDamageTakenEvent>()
            .add_event::<KidnapperFoxDefeatedEvent>()
            .add_event::<KidnapperFoxLatchBlockedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    kidnapper_fox_lock_spawn_probability,
                    kidnapper_fox_retry_spawn,
                    kidnapper_fox_spawn_from_events,
                    kidnapper_fox_rotate_hiding_patch,
                    kidnapper_fox_detect_nest_intrusion,
                    kidnapper_fox_hunt_targets,
                    kidnapper_fox_latch_targets,
                    kidnapper_fox_drag_latched_employee,
                    kidnapper_fox_shrink_nest_when_stared_down,
                    kidnapper_fox_release_tongue_on_ship_door,
                    kidnapper_fox_take_damage,
                    kidnapper_fox_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KidnapperFoxBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct KidnapperFoxSpawnState {
    pub ship_landed: bool,
    pub fixed_vain_shroud_count: u32,
    pub spawn_probability_numerator: u32,
    pub spawn_probability_denominator: u32,
    pub next_retry_hour: u32,
    pub spawned_this_day: bool,
    pub removed_for_day: bool,
}

impl Default for KidnapperFoxSpawnState {
    fn default() -> Self {
        Self {
            ship_landed: false,
            fixed_vain_shroud_count: 0,
            spawn_probability_numerator: 0,
            spawn_probability_denominator: KIDNAPPER_FOX_SPAWN_CHANCE_DIVISOR,
            next_retry_hour: KIDNAPPER_FOX_FIRST_RETRY_HOUR,
            spawned_this_day: false,
            removed_for_day: false,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct KidnapperFox {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct KidnapperFoxNest {
    pub largest_patch: SimPosition,
    pub second_largest_patch: SimPosition,
    pub active_patch: SimPosition,
    pub kill_area: SimPosition,
    pub radius: I32F32,
    pub base_kill_area_radius: I32F32,
    pub kill_area_radius: I32F32,
    pub hide_timer_ticks: u32,
    pub using_second_largest_patch: bool,
}

impl Default for KidnapperFoxNest {
    fn default() -> Self {
        Self {
            largest_patch: SimPosition {
                x: I32F32::ZERO,
                y: I32F32::ZERO,
            },
            second_largest_patch: SimPosition {
                x: I32F32::ZERO,
                y: I32F32::ZERO,
            },
            active_patch: SimPosition {
                x: I32F32::ZERO,
                y: I32F32::ZERO,
            },
            kill_area: SimPosition {
                x: I32F32::ZERO,
                y: I32F32::ZERO,
            },
            radius: KIDNAPPER_FOX_DEFAULT_NEST_RADIUS,
            base_kill_area_radius: KIDNAPPER_FOX_DEFAULT_NEST_RADIUS,
            kill_area_radius: KIDNAPPER_FOX_DEFAULT_NEST_RADIUS,
            hide_timer_ticks: 0,
            using_second_largest_patch: false,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct KidnapperFoxTarget {
    pub has_target: bool,
    pub target_entity: Entity,
    pub target_stable_id: u64,
    pub target_position: SimPosition,
    pub target_indoor: bool,
    pub target_outdoor: bool,
    pub target_seated_in_company_cruiser: bool,
    pub target_on_ship_railings: bool,
    pub latch_wait_ticks: u32,
}

impl Default for KidnapperFoxTarget {
    fn default() -> Self {
        Self {
            has_target: false,
            target_entity: Entity::PLACEHOLDER,
            target_stable_id: 0,
            target_position: SimPosition {
                x: I32F32::ZERO,
                y: I32F32::ZERO,
            },
            target_indoor: false,
            target_outdoor: true,
            target_seated_in_company_cruiser: false,
            target_on_ship_railings: false,
            latch_wait_ticks: 0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct KidnapperFoxEmployeeSensor {
    pub stable_id: u64,
    pub is_alive: bool,
    pub is_indoor: bool,
    pub is_outdoor: bool,
    pub in_nest_area: bool,
    pub staring_at_fox: bool,
    pub seated_in_company_cruiser: bool,
    pub on_ship_railings: bool,
    pub can_operate_ship_buttons: bool,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct KidnapperFoxDrag {
    pub latched: bool,
    pub employee: Entity,
    pub employee_stable_id: u64,
    pub tongue_caught_in_ship_door: bool,
}

impl Default for KidnapperFoxDrag {
    fn default() -> Self {
        Self {
            latched: false,
            employee: Entity::PLACEHOLDER,
            employee_stable_id: 0,
            tongue_caught_in_ship_door: false,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum KidnapperFoxState {
    #[default]
    Hiding,
    Hunting,
    Attacking,
    Dragging,
    Retreating,
    Dead,
}

#[derive(Bundle)]
pub struct KidnapperFoxBundle {
    pub name: Name,
    pub fox: KidnapperFox,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub state: KidnapperFoxState,
    pub nest: KidnapperFoxNest,
    pub target: KidnapperFoxTarget,
    pub drag: KidnapperFoxDrag,
}

impl KidnapperFoxBundle {
    pub fn new(event: SpawnKidnapperFoxEvent, sim_hz: I32F32) -> Self {
        let hide_ticks = fixed_seconds_to_ticks(KIDNAPPER_FOX_HIDE_PATCH_SECONDS, sim_hz);

        Self {
            name: Name::new(KIDNAPPER_FOX_NAME),
            fox: KidnapperFox {
                stable_id: event.stable_id,
            },
            position: event.position,
            health: Health::full(KIDNAPPER_FOX_HP),
            stats: UnitStats {
                move_speed: KIDNAPPER_FOX_HIDE_SPEED,
                attack_range: KIDNAPPER_FOX_ATTACK_RANGE,
                attack_damage: KIDNAPPER_FOX_CONTACT_DAMAGE,
                attack_speed: KIDNAPPER_FOX_ATTACK_SPEED_SECONDS,
                watch_range: KIDNAPPER_FOX_WATCH_RANGE,
            },
            state: KidnapperFoxState::Hiding,
            nest: KidnapperFoxNest {
                largest_patch: event.largest_vain_shroud_patch,
                second_largest_patch: event.second_largest_vain_shroud_patch,
                active_patch: event.largest_vain_shroud_patch,
                kill_area: event.kill_area,
                radius: KIDNAPPER_FOX_DEFAULT_NEST_RADIUS,
                base_kill_area_radius: event.base_kill_area_radius,
                kill_area_radius: event.base_kill_area_radius,
                hide_timer_ticks: hide_ticks,
                using_second_largest_patch: false,
            },
            target: KidnapperFoxTarget::default(),
            drag: KidnapperFoxDrag::default(),
        }
    }
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpawnKidnapperFoxEvent {
    pub stable_id: u64,
    pub position: SimPosition,
    pub largest_vain_shroud_patch: SimPosition,
    pub second_largest_vain_shroud_patch: SimPosition,
    pub kill_area: SimPosition,
    pub base_kill_area_radius: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct KidnapperFoxShipLandedEvent {
    pub vain_shroud_count: u32,
    pub largest_vain_shroud_patch: SimPosition,
    pub second_largest_vain_shroud_patch: SimPosition,
    pub kill_area: SimPosition,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct KidnapperFoxSpawnGateEvent {
    pub vain_shroud_count: u32,
    pub spawn_probability_numerator: u32,
    pub spawn_probability_denominator: u32,
    pub can_spawn: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct KidnapperFoxStateChangedEvent {
    pub fox: Entity,
    pub from: KidnapperFoxState,
    pub to: KidnapperFoxState,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct KidnapperFoxLatchedEvent {
    pub fox: Entity,
    pub employee: Entity,
    pub employee_stable_id: u64,
    pub tongue_damage: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct KidnapperFoxDraggedEvent {
    pub fox: Entity,
    pub employee: Entity,
    pub employee_stable_id: u64,
    pub destination: SimPosition,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct KidnapperFoxEmployeeItemsDroppedEvent {
    pub employee: Entity,
    pub employee_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct KidnapperFoxTongueReleasedEvent {
    pub fox: Entity,
    pub employee: Entity,
    pub employee_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct KidnapperFoxNestRadiusChangedEvent {
    pub fox: Entity,
    pub radius: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct KidnapperFoxDamageTakenEvent {
    pub fox: Entity,
    pub source: Entity,
    pub damage: I32F32,
    pub remaining_health: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct KidnapperFoxDefeatedEvent {
    pub fox: Entity,
    pub source: Entity,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct KidnapperFoxLatchBlockedEvent {
    pub fox: Entity,
    pub employee: Entity,
    pub employee_stable_id: u64,
    pub reason: KidnapperFoxLatchBlockedReason,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KidnapperFoxLatchBlockedReason {
    CompanyCruiser,
    ShipRailings,
}

fn kidnapper_fox_lock_spawn_probability(
    mut landed_events: EventReader<KidnapperFoxShipLandedEvent>,
    mut gate_events: EventWriter<KidnapperFoxSpawnGateEvent>,
    mut spawn_state: ResMut<KidnapperFoxSpawnState>,
) {
    for event in landed_events.read() {
        if spawn_state.ship_landed {
            continue;
        }

        spawn_state.ship_landed = true;
        spawn_state.fixed_vain_shroud_count = event.vain_shroud_count;
        spawn_state.spawn_probability_numerator = event.vain_shroud_count;
        spawn_state.spawn_probability_denominator = KIDNAPPER_FOX_SPAWN_CHANCE_DIVISOR;

        gate_events.send(KidnapperFoxSpawnGateEvent {
            vain_shroud_count: event.vain_shroud_count,
            spawn_probability_numerator: spawn_state.spawn_probability_numerator,
            spawn_probability_denominator: spawn_state.spawn_probability_denominator,
            can_spawn: event.vain_shroud_count >= KIDNAPPER_FOX_MIN_VAIN_SHROUDS_TO_SPAWN,
        });
    }
}

fn kidnapper_fox_retry_spawn(
    game_seed: Res<GameSeed>,
    sim_tick: Res<SimTick>,
    sim_hz: Res<SimHz>,
    mut landed_events: EventReader<KidnapperFoxShipLandedEvent>,
    mut spawn_events: EventWriter<SpawnKidnapperFoxEvent>,
    mut spawn_state: ResMut<KidnapperFoxSpawnState>,
    foxes: Query<(), With<KidnapperFox>>,
) {
    let current_hour = simulated_hour(sim_tick.0, sim_hz.0);
    let latest_landing = landed_events.read().last().copied();

    if !spawn_state.ship_landed
        || spawn_state.spawned_this_day
        || spawn_state.removed_for_day
        || foxes.iter().count() >= KIDNAPPER_FOX_MAX_SPAWNED
        || current_hour < spawn_state.next_retry_hour
        || spawn_state.fixed_vain_shroud_count < KIDNAPPER_FOX_MIN_VAIN_SHROUDS_TO_SPAWN
    {
        return;
    }

    while current_hour >= spawn_state.next_retry_hour {
        spawn_state.next_retry_hour += KIDNAPPER_FOX_SPAWN_DELAY_HOURS;
    }

    let numerator = spawn_state.spawn_probability_numerator;
    let denominator = spawn_state.spawn_probability_denominator.max(1);
    let mut rng = tick_rng(game_seed.0, sim_tick.0, KIDNAPPER_FOX_SPAWN_SALT);
    let roll = rng.next_u32() % denominator;

    if roll >= numerator {
        return;
    }

    let event = latest_landing.unwrap_or(KidnapperFoxShipLandedEvent {
        vain_shroud_count: spawn_state.fixed_vain_shroud_count,
        largest_vain_shroud_patch: SimPosition {
            x: I32F32::ZERO,
            y: I32F32::ZERO,
        },
        second_largest_vain_shroud_patch: SimPosition {
            x: I32F32::ZERO,
            y: I32F32::ZERO,
        },
        kill_area: SimPosition {
            x: I32F32::ZERO,
            y: I32F32::ZERO,
        },
    });

    spawn_state.spawned_this_day = true;
    spawn_events.send(SpawnKidnapperFoxEvent {
        stable_id: KIDNAPPER_FOX_SPAWN_SALT ^ sim_tick.0,
        position: event.largest_vain_shroud_patch,
        largest_vain_shroud_patch: event.largest_vain_shroud_patch,
        second_largest_vain_shroud_patch: event.second_largest_vain_shroud_patch,
        kill_area: event.kill_area,
        base_kill_area_radius: KIDNAPPER_FOX_DEFAULT_NEST_RADIUS,
    });
}

fn kidnapper_fox_spawn_from_events(
    mut commands: Commands,
    sim_hz: Res<SimHz>,
    mut events: EventReader<SpawnKidnapperFoxEvent>,
    mut spawn_state: ResMut<KidnapperFoxSpawnState>,
    foxes: Query<(), With<KidnapperFox>>,
) {
    let mut spawned_count = foxes.iter().count();

    for event in events.read() {
        if spawned_count >= KIDNAPPER_FOX_MAX_SPAWNED || spawn_state.removed_for_day {
            break;
        }

        commands.spawn(KidnapperFoxBundle::new(*event, sim_hz.0));
        spawn_state.spawned_this_day = true;
        spawned_count += 1;
    }
}

fn kidnapper_fox_rotate_hiding_patch(
    sim_hz: Res<SimHz>,
    mut foxes: Query<(&mut SimPosition, &mut KidnapperFoxNest, &KidnapperFoxState), With<KidnapperFox>>,
) {
    for (mut position, mut nest, state) in foxes.iter_mut() {
        if *state != KidnapperFoxState::Hiding {
            continue;
        }

        if nest.hide_timer_ticks > 0 {
            nest.hide_timer_ticks -= 1;
            continue;
        }

        nest.using_second_largest_patch = !nest.using_second_largest_patch;
        nest.active_patch = if nest.using_second_largest_patch {
            nest.second_largest_patch
        } else {
            nest.largest_patch
        };
        *position = nest.active_patch;
        nest.hide_timer_ticks = fixed_seconds_to_ticks(KIDNAPPER_FOX_HIDE_PATCH_SECONDS, sim_hz.0);
    }
}

fn kidnapper_fox_detect_nest_intrusion(
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<KidnapperFoxStateChangedEvent>,
    mut foxes: Query<
        (
            Entity,
            &mut KidnapperFoxState,
            &mut KidnapperFoxTarget,
            &mut KidnapperFoxNest,
            &mut UnitStats,
        ),
        With<KidnapperFox>,
    >,
    employees: Query<(Entity, &SimPosition, &KidnapperFoxEmployeeSensor), Without<KidnapperFox>>,
) {
    for (fox_entity, mut state, mut target, mut nest, mut stats) in foxes.iter_mut() {
        if *state != KidnapperFoxState::Hiding {
            continue;
        }

        let Some((employee, position, sensor)) = first_employee_in_nest(&employees) else {
            continue;
        };

        target.has_target = true;
        target.target_entity = employee;
        target.target_stable_id = sensor.stable_id;
        target.target_position = position;
        target.target_indoor = sensor.is_indoor;
        target.target_outdoor = sensor.is_outdoor;
        target.target_seated_in_company_cruiser = sensor.seated_in_company_cruiser;
        target.target_on_ship_railings = sensor.on_ship_railings;
        target.latch_wait_ticks = if sensor.is_indoor {
            fixed_seconds_to_ticks(KIDNAPPER_FOX_INDOOR_WAIT_SECONDS, sim_hz.0)
        } else {
            0
        };

        stats.move_speed = KIDNAPPER_FOX_HUNT_SPEED;
        nest.kill_area_radius = nest.base_kill_area_radius * KIDNAPPER_FOX_CHASE_KILL_AREA_MULTIPLIER;

        set_kidnapper_fox_state(
            fox_entity,
            &mut state,
            KidnapperFoxState::Hunting,
            &mut state_events,
        );
    }
}

fn kidnapper_fox_hunt_targets(
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<KidnapperFoxStateChangedEvent>,
    mut foxes: Query<
        (
            Entity,
            &mut SimPosition,
            &mut KidnapperFoxState,
            &mut KidnapperFoxTarget,
            &mut UnitStats,
        ),
        With<KidnapperFox>,
    >,
    employees: Query<(Entity, &SimPosition, &KidnapperFoxEmployeeSensor), Without<KidnapperFox>>,
) {
    for (fox_entity, mut position, mut state, mut target, mut stats) in foxes.iter_mut() {
        if *state != KidnapperFoxState::Hunting || !target.has_target {
            continue;
        }

        let Some((employee, employee_position, sensor)) =
            employee_by_stable_id(target.target_stable_id, &employees)
        else {
            stats.move_speed = KIDNAPPER_FOX_HIDE_SPEED;
            target.has_target = false;
            set_kidnapper_fox_state(
                fox_entity,
                &mut state,
                KidnapperFoxState::Hiding,
                &mut state_events,
            );
            continue;
        };

        target.target_entity = employee;
        target.target_position = employee_position;
        target.target_indoor = sensor.is_indoor;
        target.target_outdoor = sensor.is_outdoor;
        target.target_seated_in_company_cruiser = sensor.seated_in_company_cruiser;
        target.target_on_ship_railings = sensor.on_ship_railings;

        if target.latch_wait_ticks > 0 {
            target.latch_wait_ticks -= 1;
            continue;
        }

        stats.move_speed = KIDNAPPER_FOX_HUNT_SPEED;
        move_axis_toward(&mut position, employee_position, stats.move_speed / sim_hz.0);

        if fixed_distance_sq(*position, employee_position) <= fixed_square(stats.attack_range) {
            set_kidnapper_fox_state(
                fox_entity,
                &mut state,
                KidnapperFoxState::Attacking,
                &mut state_events,
            );
        }
    }
}

fn kidnapper_fox_latch_targets(
    mut latched_events: EventWriter<KidnapperFoxLatchedEvent>,
    mut drop_events: EventWriter<KidnapperFoxEmployeeItemsDroppedEvent>,
    mut blocked_events: EventWriter<KidnapperFoxLatchBlockedEvent>,
    mut state_events: EventWriter<KidnapperFoxStateChangedEvent>,
    mut damage_events: EventWriter<IncomingDamageEvent>,
    mut foxes: Query<
        (
            Entity,
            &mut KidnapperFoxState,
            &KidnapperFoxTarget,
            &mut KidnapperFoxDrag,
        ),
        With<KidnapperFox>,
    >,
) {
    for (fox_entity, mut state, target, mut drag) in foxes.iter_mut() {
        if *state != KidnapperFoxState::Attacking || !target.has_target {
            continue;
        }

        if target.target_seated_in_company_cruiser {
            blocked_events.send(KidnapperFoxLatchBlockedEvent {
                fox: fox_entity,
                employee: target.target_entity,
                employee_stable_id: target.target_stable_id,
                reason: KidnapperFoxLatchBlockedReason::CompanyCruiser,
            });
            set_kidnapper_fox_state(
                fox_entity,
                &mut state,
                KidnapperFoxState::Hunting,
                &mut state_events,
            );
            continue;
        }

        if target.target_on_ship_railings {
            blocked_events.send(KidnapperFoxLatchBlockedEvent {
                fox: fox_entity,
                employee: target.target_entity,
                employee_stable_id: target.target_stable_id,
                reason: KidnapperFoxLatchBlockedReason::ShipRailings,
            });
            set_kidnapper_fox_state(
                fox_entity,
                &mut state,
                KidnapperFoxState::Hunting,
                &mut state_events,
            );
            continue;
        }

        drag.latched = true;
        drag.employee = target.target_entity;
        drag.employee_stable_id = target.target_stable_id;

        damage_events.send(IncomingDamageEvent {
            target: target.target_entity,
            raw_amount: KIDNAPPER_FOX_CONTACT_DAMAGE,
            damage_type: DamageType::Standard,
            source: fox_entity,
        });

        drop_events.send(KidnapperFoxEmployeeItemsDroppedEvent {
            employee: target.target_entity,
            employee_stable_id: target.target_stable_id,
        });

        latched_events.send(KidnapperFoxLatchedEvent {
            fox: fox_entity,
            employee: target.target_entity,
            employee_stable_id: target.target_stable_id,
            tongue_damage: KIDNAPPER_FOX_CONTACT_DAMAGE,
        });

        set_kidnapper_fox_state(
            fox_entity,
            &mut state,
            KidnapperFoxState::Dragging,
            &mut state_events,
        );
    }
}

fn kidnapper_fox_drag_latched_employee(
    sim_hz: Res<SimHz>,
    mut dragged_events: EventWriter<KidnapperFoxDraggedEvent>,
    mut state_events: EventWriter<KidnapperFoxStateChangedEvent>,
    mut foxes: Query<
        (
            Entity,
            &mut SimPosition,
            &mut KidnapperFoxState,
            &KidnapperFoxNest,
            &KidnapperFoxDrag,
            &mut UnitStats,
        ),
        With<KidnapperFox>,
    >,
) {
    for (fox_entity, mut position, mut state, nest, drag, mut stats) in foxes.iter_mut() {
        if *state != KidnapperFoxState::Dragging || !drag.latched {
            continue;
        }

        stats.move_speed = KIDNAPPER_FOX_DRAG_SPEED;
        move_axis_toward(&mut position, nest.kill_area, stats.move_speed / sim_hz.0);

        dragged_events.send(KidnapperFoxDraggedEvent {
            fox: fox_entity,
            employee: drag.employee,
            employee_stable_id: drag.employee_stable_id,
            destination: nest.kill_area,
        });

        if fixed_distance_sq(*position, nest.kill_area) <= fixed_square(I32F32::ONE) {
            set_kidnapper_fox_state(
                fox_entity,
                &mut state,
                KidnapperFoxState::Retreating,
                &mut state_events,
            );
        }
    }
}

fn kidnapper_fox_shrink_nest_when_stared_down(
    mut radius_events: EventWriter<KidnapperFoxNestRadiusChangedEvent>,
    mut foxes: Query<(Entity, &SimPosition, &mut KidnapperFoxNest), With<KidnapperFox>>,
    employees: Query<(&SimPosition, &KidnapperFoxEmployeeSensor), Without<KidnapperFox>>,
) {
    for (fox_entity, fox_position, mut nest) in foxes.iter_mut() {
        let mut employee_count = 0_u32;
        let mut staring_count = 0_u32;

        for (employee_position, sensor) in employees.iter() {
            if !sensor.is_alive {
                continue;
            }

            if fixed_distance_sq(*fox_position, *employee_position)
                > fixed_square(KIDNAPPER_FOX_MEDIUM_STARE_RANGE)
            {
                continue;
            }

            employee_count += 1;
            if sensor.staring_at_fox {
                staring_count += 1;
            }
        }

        let not_staring = employee_count.saturating_sub(staring_count);
        if employee_count == 0 || not_staring >= 2 {
            continue;
        }

        let required_staring = employee_count.saturating_sub(1).max(1);
        if staring_count < required_staring {
            continue;
        }

        let next_radius = if nest.radius <= KIDNAPPER_FOX_MIN_NEST_RADIUS + KIDNAPPER_FOX_NEST_SHRINK_PER_TICK {
            KIDNAPPER_FOX_MIN_NEST_RADIUS
        } else {
            nest.radius - KIDNAPPER_FOX_NEST_SHRINK_PER_TICK
        };

        if next_radius == nest.radius {
            continue;
        }

        nest.radius = next_radius;
        radius_events.send(KidnapperFoxNestRadiusChangedEvent {
            fox: fox_entity,
            radius: nest.radius,
        });
    }
}

fn kidnapper_fox_release_tongue_on_ship_door(
    mut release_events: EventWriter<KidnapperFoxTongueReleasedEvent>,
    mut state_events: EventWriter<KidnapperFoxStateChangedEvent>,
    mut foxes: Query<(Entity, &mut KidnapperFoxState, &mut KidnapperFoxDrag), With<KidnapperFox>>,
    employees: Query<&KidnapperFoxEmployeeSensor>,
) {
    let operator_available = employees.iter().any(|sensor| {
        sensor.is_alive && sensor.can_operate_ship_buttons
    });

    for (fox_entity, mut state, mut drag) in foxes.iter_mut() {
        if *state != KidnapperFoxState::Dragging
            || !drag.latched
            || !drag.tongue_caught_in_ship_door
            || !operator_available
        {
            continue;
        }

        release_events.send(KidnapperFoxTongueReleasedEvent {
            fox: fox_entity,
            employee: drag.employee,
            employee_stable_id: drag.employee_stable_id,
        });

        drag.latched = false;
        drag.employee = Entity::PLACEHOLDER;
        drag.employee_stable_id = 0;
        drag.tongue_caught_in_ship_door = false;

        set_kidnapper_fox_state(
            fox_entity,
            &mut state,
            KidnapperFoxState::Hiding,
            &mut state_events,
        );
    }
}

fn kidnapper_fox_take_damage(
    mut damage_events: EventReader<IncomingDamageEvent>,
    mut taken_events: EventWriter<KidnapperFoxDamageTakenEvent>,
    mut defeated_events: EventWriter<KidnapperFoxDefeatedEvent>,
    mut state_events: EventWriter<KidnapperFoxStateChangedEvent>,
    mut spawn_state: ResMut<KidnapperFoxSpawnState>,
    mut foxes: Query<(Entity, &mut Health, &mut KidnapperFoxState), With<KidnapperFox>>,
) {
    for event in damage_events.read() {
        let Ok((fox_entity, mut health, mut state)) = foxes.get_mut(event.target) else {
            continue;
        };

        if *state == KidnapperFoxState::Dead {
            continue;
        }

        let damage = event.raw_amount;
        health.current -= damage;
        if health.current < I32F32::ZERO {
            health.current = I32F32::ZERO;
        }

        taken_events.send(KidnapperFoxDamageTakenEvent {
            fox: fox_entity,
            source: event.source,
            damage,
            remaining_health: health.current,
        });

        if health.current <= I32F32::ZERO {
            spawn_state.removed_for_day = true;
            set_kidnapper_fox_state(
                fox_entity,
                &mut state,
                KidnapperFoxState::Dead,
                &mut state_events,
            );
            defeated_events.send(KidnapperFoxDefeatedEvent {
                fox: fox_entity,
                source: event.source,
            });
        } else {
            set_kidnapper_fox_state(
                fox_entity,
                &mut state,
                KidnapperFoxState::Hiding,
                &mut state_events,
            );
        }
    }
}

fn kidnapper_fox_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    sim_hz: Res<SimHz>,
    spawn_state: Res<KidnapperFoxSpawnState>,
    foxes: Query<
        (
            &KidnapperFox,
            &SimPosition,
            &Health,
            &UnitStats,
            &KidnapperFoxState,
            &KidnapperFoxNest,
            &KidnapperFoxTarget,
            &KidnapperFoxDrag,
        ),
        With<KidnapperFox>,
    >,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(sim_hz.0.to_bits() as u64);
    checksum.accumulate(KIDNAPPER_FOX_SOURCE_REVISION as u64);
    checksum.accumulate(KIDNAPPER_FOX_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(KIDNAPPER_FOX_HP.to_bits() as u64);
    checksum.accumulate(KIDNAPPER_FOX_POWER_LEVEL.to_bits() as u64);
    checksum.accumulate(KIDNAPPER_FOX_MAX_SPAWNED as u64);
    checksum.accumulate(KIDNAPPER_FOX_STUN_MULTIPLIER.to_bits() as u64);
    checksum.accumulate(KIDNAPPER_FOX_ZAP_GUN_DIFFICULTY.to_bits() as u64);
    checksum.accumulate(KIDNAPPER_FOX_SPAWN_DELAY_HOURS as u64);
    checksum.accumulate(KIDNAPPER_FOX_CONTACT_DAMAGE.to_bits() as u64);
    checksum.accumulate(spawn_state.ship_landed as u64);
    checksum.accumulate(spawn_state.fixed_vain_shroud_count as u64);
    checksum.accumulate(spawn_state.spawn_probability_numerator as u64);
    checksum.accumulate(spawn_state.spawn_probability_denominator as u64);
    checksum.accumulate(spawn_state.next_retry_hour as u64);
    checksum.accumulate(spawn_state.spawned_this_day as u64);
    checksum.accumulate(spawn_state.removed_for_day as u64);

    accumulate_str(&mut checksum, 0x1000, KIDNAPPER_FOX_ID);
    accumulate_str(&mut checksum, 0x1001, KIDNAPPER_FOX_NAME);
    accumulate_str(&mut checksum, 0x1002, KIDNAPPER_FOX_TYPE);
    accumulate_str(&mut checksum, 0x1003, KIDNAPPER_FOX_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, KIDNAPPER_FOX_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1005, KIDNAPPER_FOX_EXTRACTED_AT);
    accumulate_str(&mut checksum, 0x1006, KIDNAPPER_FOX_DWELLS);
    accumulate_str(&mut checksum, 0x1007, KIDNAPPER_FOX_DANGER);
    accumulate_str(&mut checksum, 0x1008, KIDNAPPER_FOX_SCIENTIFIC_NAME);
    accumulate_str(&mut checksum, 0x1009, KIDNAPPER_FOX_INTERNAL_NAME);

    for dependency in KIDNAPPER_FOX_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for behavior in KIDNAPPER_FOX_FRONTMATTER_BEHAVIOR {
        accumulate_str(&mut checksum, 0x3000, behavior);
    }

    for rule in KIDNAPPER_FOX_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x4000, rule.condition);
        accumulate_str(&mut checksum, 0x4001, rule.outcome);
    }

    for (fox, position, health, stats, state, nest, target, drag) in foxes.iter() {
        checksum.accumulate(fox.stable_id);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);
        checksum.accumulate(kidnapper_fox_state_bits(*state));
        checksum.accumulate(nest.largest_patch.x.to_bits() as u64);
        checksum.accumulate(nest.largest_patch.y.to_bits() as u64);
        checksum.accumulate(nest.second_largest_patch.x.to_bits() as u64);
        checksum.accumulate(nest.second_largest_patch.y.to_bits() as u64);
        checksum.accumulate(nest.active_patch.x.to_bits() as u64);
        checksum.accumulate(nest.active_patch.y.to_bits() as u64);
        checksum.accumulate(nest.kill_area.x.to_bits() as u64);
        checksum.accumulate(nest.kill_area.y.to_bits() as u64);
        checksum.accumulate(nest.radius.to_bits() as u64);
        checksum.accumulate(nest.base_kill_area_radius.to_bits() as u64);
        checksum.accumulate(nest.kill_area_radius.to_bits() as u64);
        checksum.accumulate(nest.hide_timer_ticks as u64);
        checksum.accumulate(nest.using_second_largest_patch as u64);
        checksum.accumulate(target.has_target as u64);
        checksum.accumulate(target.target_stable_id);
        checksum.accumulate(target.target_position.x.to_bits() as u64);
        checksum.accumulate(target.target_position.y.to_bits() as u64);
        checksum.accumulate(target.target_indoor as u64);
        checksum.accumulate(target.target_outdoor as u64);
        checksum.accumulate(target.target_seated_in_company_cruiser as u64);
        checksum.accumulate(target.target_on_ship_railings as u64);
        checksum.accumulate(target.latch_wait_ticks as u64);
        checksum.accumulate(drag.latched as u64);
        checksum.accumulate(drag.employee_stable_id);
        checksum.accumulate(drag.tongue_caught_in_ship_door as u64);
    }
}

fn first_employee_in_nest(
    employees: &Query<(Entity, &SimPosition, &KidnapperFoxEmployeeSensor), Without<KidnapperFox>>,
) -> Option<(Entity, SimPosition, KidnapperFoxEmployeeSensor)> {
    for (entity, position, sensor) in employees.iter() {
        if sensor.is_alive && sensor.in_nest_area {
            return Some((entity, *position, *sensor));
        }
    }

    None
}

fn employee_by_stable_id(
    stable_id: u64,
    employees: &Query<(Entity, &SimPosition, &KidnapperFoxEmployeeSensor), Without<KidnapperFox>>,
) -> Option<(Entity, SimPosition, KidnapperFoxEmployeeSensor)> {
    for (entity, position, sensor) in employees.iter() {
        if sensor.stable_id == stable_id {
            return Some((entity, *position, *sensor));
        }
    }

    None
}

fn set_kidnapper_fox_state(
    fox: Entity,
    state: &mut KidnapperFoxState,
    next: KidnapperFoxState,
    events: &mut EventWriter<KidnapperFoxStateChangedEvent>,
) {
    if *state == next {
        return;
    }

    let previous = *state;
    *state = next;
    events.send(KidnapperFoxStateChangedEvent {
        fox,
        from: previous,
        to: next,
    });
}

fn simulated_hour(tick: u64, sim_hz: I32F32) -> u32 {
    let seconds = I32F32::from_num(tick) / sim_hz;
    let hour = seconds / I32F32::lit("60");
    hour.to_num::<u32>()
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

fn kidnapper_fox_state_bits(state: KidnapperFoxState) -> u64 {
    match state {
        KidnapperFoxState::Hiding => 0,
        KidnapperFoxState::Hunting => 1,
        KidnapperFoxState::Attacking => 2,
        KidnapperFoxState::Dragging => 3,
        KidnapperFoxState::Retreating => 4,
        KidnapperFoxState::Dead => 5,
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}
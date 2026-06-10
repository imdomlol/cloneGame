// Sources: vault/scrap_items/egg.md
use bevy::prelude::*;
use fixed::types::I32F32;
use rand_core::RngCore;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::ItemBarPickupEvent;
use crate::sim::{tick_rng, GameSeed, NoiseEmittedEvent, SimChecksumState, SimPosition, SimTick};

pub const EGG_ID: &str = "egg";
pub const EGG_NAME: &str = "Egg";
pub const EGG_TYPE: &str = "scrap_items";
pub const EGG_SUBTYPE: &str = "special_scrap";
pub const EGG_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Egg";
pub const EGG_SOURCE_REVISION: u32 = 21292;
pub const EGG_EXTRACTED_AT: &str = "2026-06-07T00:00:00Z";
pub const EGG_CONFIDENCE_BASIS_POINTS: u16 = 96;

pub const EGG_EFFECTS: &str = "Can hatch into a baby Sapsucker that calls for a giant_sapsucker.";
pub const EGG_WEIGHT: I32F32 = I32F32::lit("19");
pub const EGG_CONDUCTIVE: bool = false;
pub const EGG_MIN_VALUE: I32F32 = I32F32::lit("40");
pub const EGG_MAX_VALUE: I32F32 = I32F32::lit("199");
pub const EGG_TWO_HANDED: bool = true;

pub const EGG_NEAR_SHIP_DISTANCE_METERS: I32F32 = I32F32::lit("100");
pub const EGG_NEAR_SHIP_DISTANCE_SQUARED: I32F32 = I32F32::lit("10000");
pub const EGG_NEAR_SHIP_MIN_VALUE: I32F32 = I32F32::lit("40");
pub const EGG_NEAR_SHIP_MAX_VALUE: I32F32 = I32F32::lit("69");
pub const EGG_FAR_SHIP_MIN_VALUE: I32F32 = I32F32::lit("70");
pub const EGG_FAR_SHIP_MAX_VALUE: I32F32 = I32F32::lit("119");
pub const EGG_FAR_SHIP_BONUS_MAX_VALUE: I32F32 = I32F32::lit("199");
pub const EGG_FAR_SHIP_BONUS_ROLL_PERCENT: u32 = 30;
pub const EGG_GUARANTEED_NEST_COUNT: u8 = 3;
pub const EGG_HATCH_SHORT_DURATION_TICKS: u16 = 60;
pub const EGG_SCREECH_SOUND_AMOUNT: I32F32 = I32F32::lit("80");
pub const EGG_CALL_SOUND_AMOUNT: I32F32 = I32F32::lit("120");
pub const EGG_VALUE_ROLL_SALT: u64 = 0x6567_675f_7661_6c75;
pub const EGG_BONUS_ROLL_SALT: u64 = 0x6567_675f_626f_6e75;

pub const EGG_DEPENDS_ON: [&str; 3] = ["giant_sapsucker", "the_ship", "the_company"];

pub const EGG_BEHAVIORAL_MECHANICS: [EggBehaviorRule; 12] = [
    EggBehaviorRule {
        condition: "giant_sapsucker spawns",
        outcome: "Eggs can appear, and three Eggs are guaranteed inside its nest",
    },
    EggBehaviorRule {
        condition: "an Egg is picked up and moved away from the nest",
        outcome: "it can hatch into a baby Sapsucker after a short duration",
    },
    EggBehaviorRule {
        condition: "a hatched Egg is being carried",
        outcome: "it will screech, briefly pause, stare at the carrier, and resume screeching until dropped",
    },
    EggBehaviorRule {
        condition: "a dropped Egg is present",
        outcome: "it calls the giant_sapsucker to its exact location",
    },
    EggBehaviorRule {
        condition: "an Egg is carried near the giant_sapsucker or into its nesting area",
        outcome: "the giant_sapsucker can attack nearby employees and entities",
    },
    EggBehaviorRule {
        condition: "an Egg is carried onto the_ship while the giant_sapsucker is alive",
        outcome: "the giant_sapsucker can break the hydraulic doors to retrieve it",
    },
    EggBehaviorRule {
        condition: "the giant_sapsucker dies before the Egg hatches",
        outcome: "the Egg hatches upon entering orbit and has no further effect",
    },
    EggBehaviorRule {
        condition: "the nest-to-ship distance is less than or equal to 100 meters",
        outcome: "each Egg value is randomly between 40 and 69 credits",
    },
    EggBehaviorRule {
        condition: "the nest-to-ship distance is greater than 100 meters",
        outcome: "each Egg value is randomly between 70 and 119 credits",
    },
    EggBehaviorRule {
        condition: "the nest-to-ship distance is greater than 100 meters and the 30% bonus roll succeeds",
        outcome: "each Egg value is randomly between 70 and 199 credits",
    },
    EggBehaviorRule {
        condition: "the Egg is carried",
        outcome: "it requires two hands",
    },
    EggBehaviorRule {
        condition: "the Egg is evaluated for conductance",
        outcome: "it is non-conductive",
    },
];

pub struct EggPlugin;

impl Plugin for EggPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnEggEvent>()
            .add_event::<EggPickedUpAndMovedEvent>()
            .add_event::<EggHatchedEvent>()
            .add_event::<EggScreechEvent>()
            .add_event::<EggDroppedCallEvent>()
            .add_event::<EggShipDoorThreatEvent>()
            .add_event::<EggSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_egg,
                    egg_pickup_item_bar_bridge,
                    egg_mark_moved_from_nest,
                    egg_hatch_after_moved,
                    egg_screech_while_carried,
                    egg_call_sapsucker_when_dropped,
                    egg_ship_door_threat,
                    egg_sell_bridge,
                    egg_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EggBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Egg {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct EggScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
    pub credit_value: I32F32,
}

impl Default for EggScrap {
    fn default() -> Self {
        Self {
            min_value: EGG_MIN_VALUE,
            max_value: EGG_MAX_VALUE,
            weight: EGG_WEIGHT,
            conductive: EGG_CONDUCTIVE,
            two_handed: EGG_TWO_HANDED,
            credit_value: EGG_MIN_VALUE,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EggHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct EggNestState {
    pub nest_id: u64,
    pub nest_position: SimPosition,
    pub ship_position: SimPosition,
    pub moved_from_nest: bool,
    pub carried_onto_ship: bool,
    pub giant_sapsucker_alive: bool,
}

impl Default for EggNestState {
    fn default() -> Self {
        Self {
            nest_id: 0,
            nest_position: zero_position(),
            ship_position: zero_position(),
            moved_from_nest: false,
            carried_onto_ship: false,
            giant_sapsucker_alive: true,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct EggHatchState {
    pub hatched: bool,
    pub has_no_further_effect: bool,
    pub hatch_tick: u64,
    pub calls_sapsucker: bool,
    pub screeching: bool,
}

impl Default for EggHatchState {
    fn default() -> Self {
        Self {
            hatched: false,
            has_no_further_effect: false,
            hatch_tick: 0,
            calls_sapsucker: false,
            screeching: false,
        }
    }
}

#[derive(Bundle)]
pub struct EggBundle {
    pub name: Name,
    pub egg: Egg,
    pub scrap: EggScrap,
    pub position: SimPosition,
    pub held_by: EggHeldBy,
    pub nest_state: EggNestState,
    pub hatch_state: EggHatchState,
}

impl EggBundle {
    pub fn new(event: SpawnEggEvent, seed: u64, tick: u64) -> Self {
        let distance_squared =
            distance_squared_between(event.nest_position, event.ship_position);
        let credit_value = egg_roll_credit_value(
            seed,
            tick,
            event.stable_id,
            distance_squared,
        );

        Self {
            name: Name::new(EGG_NAME),
            egg: Egg {
                stable_id: event.stable_id,
            },
            scrap: EggScrap {
                credit_value,
                ..EggScrap::default()
            },
            position: event.position,
            held_by: EggHeldBy::default(),
            nest_state: EggNestState {
                nest_id: event.nest_id,
                nest_position: event.nest_position,
                ship_position: event.ship_position,
                giant_sapsucker_alive: event.giant_sapsucker_alive,
                ..EggNestState::default()
            },
            hatch_state: EggHatchState::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnEggEvent {
    pub stable_id: u64,
    pub nest_id: u64,
    pub position: SimPosition,
    pub nest_position: SimPosition,
    pub ship_position: SimPosition,
    pub giant_sapsucker_alive: bool,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EggPickedUpAndMovedEvent {
    pub egg_entity: Entity,
    pub egg_stable_id: u64,
    pub employee_id: u64,
    pub nest_id: u64,
    pub hatch_tick: u64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EggHatchedEvent {
    pub egg_entity: Entity,
    pub egg_stable_id: u64,
    pub employee_id: u64,
    pub position: SimPosition,
    pub has_no_further_effect: bool,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EggScreechEvent {
    pub egg_entity: Entity,
    pub egg_stable_id: u64,
    pub employee_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EggDroppedCallEvent {
    pub egg_entity: Entity,
    pub egg_stable_id: u64,
    pub nest_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EggShipDoorThreatEvent {
    pub egg_entity: Entity,
    pub egg_stable_id: u64,
    pub nest_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EggSoldEvent {
    pub egg_stable_id: u64,
    pub credit_value: I32F32,
}

pub fn egg_value_range() -> (I32F32, I32F32) {
    (EGG_MIN_VALUE, EGG_MAX_VALUE)
}

pub fn egg_value_range_for_nest_distance_squared(distance_squared: I32F32) -> (I32F32, I32F32) {
    if distance_squared <= EGG_NEAR_SHIP_DISTANCE_SQUARED {
        (EGG_NEAR_SHIP_MIN_VALUE, EGG_NEAR_SHIP_MAX_VALUE)
    } else {
        (EGG_FAR_SHIP_MIN_VALUE, EGG_FAR_SHIP_MAX_VALUE)
    }
}

pub fn egg_guaranteed_nest_count() -> u8 {
    EGG_GUARANTEED_NEST_COUNT
}

fn spawn_egg(
    mut commands: Commands,
    mut events: EventReader<SpawnEggEvent>,
    seed: Res<GameSeed>,
    tick: Res<SimTick>,
) {
    for event in events.read() {
        commands.spawn(EggBundle::new(*event, seed.0, tick.0));
    }
}

fn egg_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    eggs: Query<(&Egg, &EggHeldBy), Changed<EggHeldBy>>,
) {
    for (egg, held_by) in &eggs {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: EGG_ID,
                two_handed: EGG_TWO_HANDED,
                functional: false,
                passive: true,
                from_store_or_valueless: false,
            });
        } else {
            let _ = egg.stable_id;
        }
    }
}

fn egg_mark_moved_from_nest(
    mut moved_events: EventWriter<EggPickedUpAndMovedEvent>,
    mut eggs: Query<(
        Entity,
        &Egg,
        &EggHeldBy,
        &SimPosition,
        &mut EggNestState,
        &mut EggHatchState,
    )>,
    tick: Res<SimTick>,
) {
    for (entity, egg, held_by, position, mut nest_state, mut hatch_state) in &mut eggs {
        if nest_state.moved_from_nest || !held_by.is_held {
            continue;
        }

        if position.x == nest_state.nest_position.x && position.y == nest_state.nest_position.y {
            continue;
        }

        nest_state.moved_from_nest = true;
        hatch_state.hatch_tick = tick.0 + EGG_HATCH_SHORT_DURATION_TICKS as u64;

        moved_events.send(EggPickedUpAndMovedEvent {
            egg_entity: entity,
            egg_stable_id: egg.stable_id,
            employee_id: held_by.employee_id,
            nest_id: nest_state.nest_id,
            hatch_tick: hatch_state.hatch_tick,
        });
    }
}

fn egg_hatch_after_moved(
    mut hatched_events: EventWriter<EggHatchedEvent>,
    mut eggs: Query<(
        Entity,
        &Egg,
        &EggHeldBy,
        &SimPosition,
        &EggNestState,
        &mut EggHatchState,
    )>,
    tick: Res<SimTick>,
) {
    for (entity, egg, held_by, position, nest_state, mut hatch_state) in &mut eggs {
        if hatch_state.hatched || !nest_state.moved_from_nest || tick.0 < hatch_state.hatch_tick {
            continue;
        }

        hatch_state.hatched = true;
        hatch_state.has_no_further_effect = !nest_state.giant_sapsucker_alive;
        hatch_state.screeching = held_by.is_held && nest_state.giant_sapsucker_alive;

        hatched_events.send(EggHatchedEvent {
            egg_entity: entity,
            egg_stable_id: egg.stable_id,
            employee_id: held_by.employee_id,
            position: *position,
            has_no_further_effect: hatch_state.has_no_further_effect,
        });
    }
}

fn egg_screech_while_carried(
    mut screech_events: EventWriter<EggScreechEvent>,
    mut noise_events: EventWriter<NoiseEmittedEvent>,
    eggs: Query<(Entity, &Egg, &EggHeldBy, &SimPosition, &EggHatchState)>,
) {
    for (entity, egg, held_by, position, hatch_state) in &eggs {
        if !hatch_state.hatched || hatch_state.has_no_further_effect || !held_by.is_held {
            continue;
        }

        screech_events.send(EggScreechEvent {
            egg_entity: entity,
            egg_stable_id: egg.stable_id,
            employee_id: held_by.employee_id,
            position: *position,
        });

        noise_events.send(NoiseEmittedEvent {
            source: entity,
            position: *position,
            amount: EGG_SCREECH_SOUND_AMOUNT,
        });
    }
}

fn egg_call_sapsucker_when_dropped(
    mut call_events: EventWriter<EggDroppedCallEvent>,
    mut noise_events: EventWriter<NoiseEmittedEvent>,
    mut eggs: Query<(
        Entity,
        &Egg,
        &EggHeldBy,
        &SimPosition,
        &EggNestState,
        &mut EggHatchState,
    ), Changed<EggHeldBy>>,
) {
    for (entity, egg, held_by, position, nest_state, mut hatch_state) in &mut eggs {
        if held_by.is_held || !nest_state.giant_sapsucker_alive || hatch_state.has_no_further_effect {
            continue;
        }

        hatch_state.calls_sapsucker = true;
        hatch_state.screeching = false;

        call_events.send(EggDroppedCallEvent {
            egg_entity: entity,
            egg_stable_id: egg.stable_id,
            nest_id: nest_state.nest_id,
            position: *position,
        });

        noise_events.send(NoiseEmittedEvent {
            source: entity,
            position: *position,
            amount: EGG_CALL_SOUND_AMOUNT,
        });
    }
}

fn egg_ship_door_threat(
    mut threat_events: EventWriter<EggShipDoorThreatEvent>,
    mut eggs: Query<(Entity, &Egg, &EggHeldBy, &SimPosition, &mut EggNestState)>,
) {
    for (entity, egg, held_by, position, mut nest_state) in &mut eggs {
        if !held_by.is_held || !nest_state.giant_sapsucker_alive || nest_state.carried_onto_ship {
            continue;
        }

        if position.x != nest_state.ship_position.x || position.y != nest_state.ship_position.y {
            continue;
        }

        nest_state.carried_onto_ship = true;

        threat_events.send(EggShipDoorThreatEvent {
            egg_entity: entity,
            egg_stable_id: egg.stable_id,
            nest_id: nest_state.nest_id,
            position: *position,
        });
    }
}

fn egg_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<EggSoldEvent>,
    eggs: Query<(&Egg, &EggScrap)>,
) {
    for event in sell_events.read() {
        for (egg, scrap) in &eggs {
            if egg.stable_id != event.scrap_entity_id {
                continue;
            }

            sold_events.send(EggSoldEvent {
                egg_stable_id: egg.stable_id,
                credit_value: scrap.credit_value,
            });
        }
    }
}

fn egg_roll_credit_value(seed: u64, tick: u64, stable_id: u64, distance_squared: I32F32) -> I32F32 {
    let (min_value, base_max_value) = egg_value_range_for_nest_distance_squared(distance_squared);
    let mut max_value = base_max_value;

    if distance_squared > EGG_NEAR_SHIP_DISTANCE_SQUARED {
        let mut bonus_rng = tick_rng(seed, tick, EGG_BONUS_ROLL_SALT ^ stable_id);
        if bonus_rng.next_u32() % 100 < EGG_FAR_SHIP_BONUS_ROLL_PERCENT {
            max_value = EGG_FAR_SHIP_BONUS_MAX_VALUE;
        }
    }

    let min_raw = min_value.to_num::<u32>();
    let max_raw = max_value.to_num::<u32>();
    let span = max_raw - min_raw + 1;
    let mut value_rng = tick_rng(seed, tick, EGG_VALUE_ROLL_SALT ^ stable_id);
    I32F32::from_num(min_raw + (value_rng.next_u32() % span))
}

fn distance_squared_between(a: SimPosition, b: SimPosition) -> I32F32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    dx * dx + dy * dy
}

fn zero_position() -> SimPosition {
    SimPosition {
        x: I32F32::lit("0"),
        y: I32F32::lit("0"),
    }
}

fn egg_checksum(
    mut checksum: ResMut<SimChecksumState>,
    eggs: Query<(&Egg, &EggScrap, &SimPosition, &EggHeldBy, &EggNestState, &EggHatchState)>,
) {
    accumulate_str(&mut checksum, 0x1000, EGG_ID);
    accumulate_str(&mut checksum, 0x1001, EGG_NAME);
    accumulate_str(&mut checksum, 0x1002, EGG_TYPE);
    accumulate_str(&mut checksum, 0x1003, EGG_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, EGG_EFFECTS);
    accumulate_str(&mut checksum, 0x1005, EGG_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1006, EGG_EXTRACTED_AT);

    checksum.accumulate(EGG_SOURCE_REVISION as u64);
    checksum.accumulate(EGG_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(EGG_WEIGHT.to_bits() as u64);
    checksum.accumulate(EGG_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(EGG_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(EGG_CONDUCTIVE as u64);
    checksum.accumulate(EGG_TWO_HANDED as u64);
    checksum.accumulate(EGG_NEAR_SHIP_DISTANCE_METERS.to_bits() as u64);
    checksum.accumulate(EGG_NEAR_SHIP_DISTANCE_SQUARED.to_bits() as u64);
    checksum.accumulate(EGG_NEAR_SHIP_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(EGG_NEAR_SHIP_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(EGG_FAR_SHIP_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(EGG_FAR_SHIP_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(EGG_FAR_SHIP_BONUS_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(EGG_FAR_SHIP_BONUS_ROLL_PERCENT as u64);
    checksum.accumulate(EGG_GUARANTEED_NEST_COUNT as u64);
    checksum.accumulate(EGG_HATCH_SHORT_DURATION_TICKS as u64);
    checksum.accumulate(EGG_SCREECH_SOUND_AMOUNT.to_bits() as u64);
    checksum.accumulate(EGG_CALL_SOUND_AMOUNT.to_bits() as u64);

    for dependency in EGG_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for rule in EGG_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x3000, rule.condition);
        accumulate_str(&mut checksum, 0x3001, rule.outcome);
    }

    for (egg, scrap, position, held_by, nest_state, hatch_state) in &eggs {
        checksum.accumulate(egg.stable_id);
        checksum.accumulate(scrap.min_value.to_bits() as u64);
        checksum.accumulate(scrap.max_value.to_bits() as u64);
        checksum.accumulate(scrap.weight.to_bits() as u64);
        checksum.accumulate(scrap.conductive as u64);
        checksum.accumulate(scrap.two_handed as u64);
        checksum.accumulate(scrap.credit_value.to_bits() as u64);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(held_by.employee_id);
        checksum.accumulate(held_by.is_held as u64);
        checksum.accumulate(nest_state.nest_id);
        checksum.accumulate(nest_state.nest_position.x.to_bits() as u64);
        checksum.accumulate(nest_state.nest_position.y.to_bits() as u64);
        checksum.accumulate(nest_state.ship_position.x.to_bits() as u64);
        checksum.accumulate(nest_state.ship_position.y.to_bits() as u64);
        checksum.accumulate(nest_state.moved_from_nest as u64);
        checksum.accumulate(nest_state.carried_onto_ship as u64);
        checksum.accumulate(nest_state.giant_sapsucker_alive as u64);
        checksum.accumulate(hatch_state.hatched as u64);
        checksum.accumulate(hatch_state.has_no_further_effect as u64);
        checksum.accumulate(hatch_state.hatch_tick);
        checksum.accumulate(hatch_state.calls_sapsucker as u64);
        checksum.accumulate(hatch_state.screeching as u64);
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt);
    for byte in value.as_bytes() {
        checksum.accumulate(*byte as u64);
    }
}
// Sources: vault/scrap_items/teeth.md
use bevy::prelude::*;
use fixed::types::I32F32;
use rand_core::RngCore;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::ItemBarPickupEvent;
use crate::sim::{tick_rng, GameSeed, NoiseEmittedEvent, SimChecksumState, SimPosition, SimTick};

pub const TEETH_ID: &str = "teeth";
pub const TEETH_NAME: &str = "Teeth";
pub const TEETH_TYPE: &str = "scrap_items";
pub const TEETH_SUBTYPE: &str = "scrap";
pub const TEETH_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Teeth";
pub const TEETH_SOURCE_REVISION: u32 = 20418;
pub const TEETH_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const TEETH_CONFIDENCE_BASIS_POINTS: u16 = 96;

pub const TEETH_EFFECTS: &str = "Can make chattering noises when picked up";
pub const TEETH_WEIGHT: I32F32 = I32F32::lit("0");
pub const TEETH_CONDUCTIVE: bool = false;
pub const TEETH_MIN_VALUE: I32F32 = I32F32::lit("60");
pub const TEETH_MAX_VALUE: I32F32 = I32F32::lit("83");
pub const TEETH_TWO_HANDED: bool = false;
pub const TEETH_CHATTER_NOISE_AMOUNT: I32F32 = I32F32::lit("40");
pub const TEETH_CHATTER_NOISE_INTERVAL_TICKS: u64 = 15;
// 50% chance to begin chattering on pickup ("may begin")
pub const TEETH_CHATTER_START_CHANCE_DENOM: u32 = 2;
// after this many pickup/drop cycles, chatter can stop
pub const TEETH_CHATTER_STOP_CYCLE_THRESHOLD: u32 = 3;

pub const TEETH_CHATTER_START_SALT: u64 = 0x7465_6574_6873_7461;
pub const TEETH_CHATTER_STOP_SALT: u64 = 0x7465_6574_6873_746f;

pub const TEETH_DEPENDS_ON: [&str; 5] = [
    "scrap",
    "lethal_company",
    "the_company",
    "eyeless_dog",
    "audible_sounds",
];

pub const TEETH_BEHAVIORAL_MECHANICS: [TeethBehaviorRule; 4] = [
    TeethBehaviorRule {
        condition: "picked up",
        outcome: "it may begin chattering and making noise",
    },
    TeethBehaviorRule {
        condition: "dropped while chattering",
        outcome: "it may move in a small circular motion",
    },
    TeethBehaviorRule {
        condition: "repeatedly dropped and picked up",
        outcome: "the chatter can stop",
    },
    TeethBehaviorRule {
        condition: "pocketed",
        outcome: "its noise does not alert Eyeless Dogs or other entities capable of hearing",
    },
];

pub struct TeethPlugin;

impl Plugin for TeethPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnTeethEvent>()
            .add_event::<TeethChatterStartedEvent>()
            .add_event::<TeethChatterStoppedEvent>()
            .add_event::<TeethAudibleAlertEvent>()
            .add_event::<TeethCircularMotionEvent>()
            .add_event::<TeethSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_teeth,
                    teeth_pickup_item_bar_bridge,
                    teeth_chatter_on_held_changed,
                    teeth_noise_while_chattering,
                    teeth_sell_bridge,
                    teeth_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TeethBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Teeth {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TeethScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for TeethScrap {
    fn default() -> Self {
        Self {
            min_value: TEETH_MIN_VALUE,
            max_value: TEETH_MAX_VALUE,
            weight: TEETH_WEIGHT,
            conductive: TEETH_CONDUCTIVE,
            two_handed: TEETH_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TeethHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
    pub is_pocketed: bool,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TeethChatterState {
    pub is_chattering: bool,
    pub pickup_drop_cycles: u32,
    pub chatter_tick_started: u64,
    pub noise_tick_last: u64,
}

impl Default for TeethChatterState {
    fn default() -> Self {
        Self {
            is_chattering: false,
            pickup_drop_cycles: 0,
            chatter_tick_started: 0,
            noise_tick_last: 0,
        }
    }
}

#[derive(Bundle)]
pub struct TeethBundle {
    pub name: Name,
    pub teeth: Teeth,
    pub scrap: TeethScrap,
    pub position: SimPosition,
    pub held_by: TeethHeldBy,
    pub chatter_state: TeethChatterState,
}

impl TeethBundle {
    pub fn new(event: SpawnTeethEvent) -> Self {
        Self {
            name: Name::new(TEETH_NAME),
            teeth: Teeth {
                stable_id: event.stable_id,
            },
            scrap: TeethScrap::default(),
            position: event.position,
            held_by: TeethHeldBy::default(),
            chatter_state: TeethChatterState::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnTeethEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TeethChatterStartedEvent {
    pub teeth_stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TeethChatterStoppedEvent {
    pub teeth_stable_id: u64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TeethAudibleAlertEvent {
    pub source: Entity,
    pub position: SimPosition,
    pub amount: I32F32,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TeethCircularMotionEvent {
    pub teeth_stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TeethSoldEvent {
    pub teeth_stable_id: u64,
    pub credit_value: I32F32,
}

pub fn teeth_value_range() -> (I32F32, I32F32) {
    (TEETH_MIN_VALUE, TEETH_MAX_VALUE)
}

fn spawn_teeth(mut commands: Commands, mut events: EventReader<SpawnTeethEvent>) {
    for event in events.read() {
        commands.spawn(TeethBundle::new(*event));
    }
}

fn teeth_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    teeth_query: Query<(&Teeth, &TeethHeldBy), Changed<TeethHeldBy>>,
) {
    for (teeth, held_by) in &teeth_query {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: TEETH_ID,
                two_handed: TEETH_TWO_HANDED,
                functional: false,
                passive: true,
                from_store_or_valueless: false,
            });
        } else {
            let _ = teeth.stable_id;
        }
    }
}

fn teeth_chatter_on_held_changed(
    mut teeth_query: Query<
        (&Teeth, &TeethHeldBy, &SimPosition, &mut TeethChatterState),
        Changed<TeethHeldBy>,
    >,
    mut chatter_started: EventWriter<TeethChatterStartedEvent>,
    mut chatter_stopped: EventWriter<TeethChatterStoppedEvent>,
    mut circular_motion: EventWriter<TeethCircularMotionEvent>,
    seed: Res<GameSeed>,
    tick: Res<SimTick>,
) {
    for (teeth, held_by, position, mut state) in &mut teeth_query {
        if held_by.is_held {
            let salt = TEETH_CHATTER_START_SALT ^ teeth.stable_id;
            let mut rng = tick_rng(seed.0, tick.0, salt);
            let roll = rng.next_u32() % TEETH_CHATTER_START_CHANCE_DENOM;
            if roll == 0 && !state.is_chattering {
                state.is_chattering = true;
                state.chatter_tick_started = tick.0;
                chatter_started.send(TeethChatterStartedEvent {
                    teeth_stable_id: teeth.stable_id,
                    position: *position,
                });
            } else if state.is_chattering
                && state.pickup_drop_cycles >= TEETH_CHATTER_STOP_CYCLE_THRESHOLD
            {
                let stop_salt = TEETH_CHATTER_STOP_SALT ^ teeth.stable_id;
                let mut stop_rng = tick_rng(seed.0, tick.0, stop_salt);
                if stop_rng.next_u32() % 2 == 0 {
                    state.is_chattering = false;
                    chatter_stopped.send(TeethChatterStoppedEvent {
                        teeth_stable_id: teeth.stable_id,
                    });
                }
            }
        } else {
            if state.is_chattering {
                circular_motion.send(TeethCircularMotionEvent {
                    teeth_stable_id: teeth.stable_id,
                    position: *position,
                });
            }
            state.pickup_drop_cycles = state.pickup_drop_cycles.saturating_add(1);
        }
    }
}

fn teeth_noise_while_chattering(
    mut teeth_query: Query<(Entity, &TeethHeldBy, &SimPosition, &mut TeethChatterState)>,
    mut noise_events: EventWriter<NoiseEmittedEvent>,
    mut alert_events: EventWriter<TeethAudibleAlertEvent>,
    tick: Res<SimTick>,
) {
    for (entity, held_by, position, mut state) in &mut teeth_query {
        if !state.is_chattering {
            continue;
        }
        if tick.0.wrapping_sub(state.noise_tick_last) < TEETH_CHATTER_NOISE_INTERVAL_TICKS {
            continue;
        }
        state.noise_tick_last = tick.0;
        if held_by.is_pocketed {
            continue;
        }
        noise_events.send(NoiseEmittedEvent {
            source: entity,
            position: *position,
            amount: TEETH_CHATTER_NOISE_AMOUNT,
        });
        alert_events.send(TeethAudibleAlertEvent {
            source: entity,
            position: *position,
            amount: TEETH_CHATTER_NOISE_AMOUNT,
        });
    }
}

fn teeth_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<TeethSoldEvent>,
    teeth_query: Query<(&Teeth, &TeethScrap)>,
) {
    for event in sell_events.read() {
        for (teeth, scrap) in &teeth_query {
            if teeth.stable_id != event.scrap_entity_id {
                continue;
            }
            sold_events.send(TeethSoldEvent {
                teeth_stable_id: teeth.stable_id,
                credit_value: scrap.max_value,
            });
        }
    }
}

fn teeth_checksum(
    mut checksum: ResMut<SimChecksumState>,
    teeth_query: Query<(&Teeth, &TeethScrap, &SimPosition, &TeethHeldBy, &TeethChatterState)>,
) {
    accumulate_str(&mut checksum, 0x1000, TEETH_ID);
    accumulate_str(&mut checksum, 0x1001, TEETH_NAME);
    accumulate_str(&mut checksum, 0x1002, TEETH_TYPE);
    accumulate_str(&mut checksum, 0x1003, TEETH_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, TEETH_EFFECTS);
    checksum.accumulate(TEETH_SOURCE_REVISION as u64);
    checksum.accumulate(TEETH_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(TEETH_WEIGHT.to_bits() as u64);
    checksum.accumulate(TEETH_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(TEETH_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(TEETH_CONDUCTIVE as u64);
    checksum.accumulate(TEETH_TWO_HANDED as u64);
    checksum.accumulate(TEETH_CHATTER_NOISE_AMOUNT.to_bits() as u64);
    checksum.accumulate(TEETH_CHATTER_NOISE_INTERVAL_TICKS);
    checksum.accumulate(TEETH_CHATTER_START_CHANCE_DENOM as u64);
    checksum.accumulate(TEETH_CHATTER_STOP_CYCLE_THRESHOLD as u64);
    for dep in TEETH_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dep);
    }
    for rule in TEETH_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x5000, rule.condition);
        accumulate_str(&mut checksum, 0x5001, rule.outcome);
    }
    for (teeth, scrap, position, held_by, state) in &teeth_query {
        checksum.accumulate(teeth.stable_id);
        checksum.accumulate(scrap.min_value.to_bits() as u64);
        checksum.accumulate(scrap.max_value.to_bits() as u64);
        checksum.accumulate(scrap.weight.to_bits() as u64);
        checksum.accumulate(scrap.conductive as u64);
        checksum.accumulate(scrap.two_handed as u64);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(held_by.employee_id);
        checksum.accumulate(held_by.is_held as u64);
        checksum.accumulate(held_by.is_pocketed as u64);
        checksum.accumulate(state.is_chattering as u64);
        checksum.accumulate(state.pickup_drop_cycles as u64);
        checksum.accumulate(state.chatter_tick_started);
        checksum.accumulate(state.noise_tick_last);
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt);
    for byte in value.as_bytes() {
        checksum.accumulate(*byte as u64);
    }
}
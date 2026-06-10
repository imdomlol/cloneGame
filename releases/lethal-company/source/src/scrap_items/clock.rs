// Sources: vault/scrap_items/clock.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::ItemBarPickupEvent;
use crate::sim::{SimChecksumState, SimHz, SimPosition};

pub const CLOCK_ID: &str = "clock";
pub const CLOCK_NAME: &str = "Clock";
pub const CLOCK_TYPE: &str = "scrap_items";
pub const CLOCK_SUBTYPE: &str = "scrap";
pub const CLOCK_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Clock";
pub const CLOCK_SOURCE_REVISION: u32 = 20388;
pub const CLOCK_EXTRACTED_AT: &str = "2026-06-07";
pub const CLOCK_CONFIDENCE_BASIS_POINTS: u16 = 97;

pub const CLOCK_EFFECTS: &str = "Makes ticking sounds every second";
pub const CLOCK_WEIGHT: I32F32 = I32F32::lit("26");
pub const CLOCK_CONDUCTIVE: bool = false;
pub const CLOCK_MIN_VALUE: I32F32 = I32F32::lit("44");
pub const CLOCK_MAX_VALUE: I32F32 = I32F32::lit("55");
pub const CLOCK_TWO_HANDED: bool = false;
pub const CLOCK_TICK_INTERVAL_SECONDS: u16 = 1;
pub const CLOCK_AUDIBLE_ALERTS_ENTITIES: bool = false;

pub const CLOCK_DEPENDS_ON: [&str; 6] = [
    "scrap",
    "lethal_company",
    "the_company",
    "credits",
    "eyeless_dog",
    "audible_sounds",
];

pub const CLOCK_SPAWN_CHANCES: [ClockSpawnChance; 3] = [
    ClockSpawnChance {
        moon: "rend",
        chance: I32F32::lit("3.24"),
    },
    ClockSpawnChance {
        moon: "artifice",
        chance: I32F32::lit("2.88"),
    },
    ClockSpawnChance {
        moon: "offense",
        chance: I32F32::lit("0.72"),
    },
];

pub const CLOCK_BEHAVIORAL_MECHANICS: [ClockBehaviorRule; 3] = [
    ClockBehaviorRule {
        condition: "the clock is present",
        outcome: "it emits a tick-tock sound once every 1 second",
    },
    ClockBehaviorRule {
        condition: "the ticking is heard while searching for loot",
        outcome: "it can help locate the item by audio",
    },
    ClockBehaviorRule {
        condition: "eyeless_dog or another entity that reacts to audible_sounds is nearby",
        outcome: "the ticking does not alert it",
    },
];

pub struct ClockPlugin;

impl Plugin for ClockPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnClockEvent>()
            .add_event::<ClockTickedEvent>()
            .add_event::<ClockSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_clock,
                    clock_pickup_item_bar_bridge,
                    clock_tick_sound,
                    clock_sell_bridge,
                    clock_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClockBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClockSpawnChance {
    pub moon: &'static str,
    pub chance: I32F32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Clock {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClockScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for ClockScrap {
    fn default() -> Self {
        Self {
            min_value: CLOCK_MIN_VALUE,
            max_value: CLOCK_MAX_VALUE,
            weight: CLOCK_WEIGHT,
            conductive: CLOCK_CONDUCTIVE,
            two_handed: CLOCK_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ClockHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClockTickState {
    pub ticks_until_next_sound: u16,
    pub total_ticks_emitted: u64,
    pub last_ticked_at_sim_tick: u64,
}

impl Default for ClockTickState {
    fn default() -> Self {
        Self {
            ticks_until_next_sound: 0,
            total_ticks_emitted: 0,
            last_ticked_at_sim_tick: 0,
        }
    }
}

#[derive(Bundle)]
pub struct ClockBundle {
    pub name: Name,
    pub clock: Clock,
    pub scrap: ClockScrap,
    pub position: SimPosition,
    pub held_by: ClockHeldBy,
    pub tick_state: ClockTickState,
}

impl ClockBundle {
    pub fn new(event: SpawnClockEvent) -> Self {
        Self {
            name: Name::new(CLOCK_NAME),
            clock: Clock {
                stable_id: event.stable_id,
            },
            scrap: ClockScrap::default(),
            position: event.position,
            held_by: ClockHeldBy::default(),
            tick_state: ClockTickState::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnClockEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClockTickedEvent {
    pub clock_entity: Entity,
    pub clock_stable_id: u64,
    pub position: SimPosition,
    pub alerts_audible_sound_reactors: bool,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClockSoldEvent {
    pub clock_stable_id: u64,
    pub credit_value: I32F32,
}

pub fn clock_value_range() -> (I32F32, I32F32) {
    (CLOCK_MIN_VALUE, CLOCK_MAX_VALUE)
}

pub fn clock_spawn_chance_for_moon(moon: &str) -> Option<I32F32> {
    CLOCK_SPAWN_CHANCES
        .iter()
        .find(|spawn_chance| spawn_chance.moon == moon)
        .map(|spawn_chance| spawn_chance.chance)
}

fn spawn_clock(mut commands: Commands, mut events: EventReader<SpawnClockEvent>) {
    for event in events.read() {
        commands.spawn(ClockBundle::new(*event));
    }
}

fn clock_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    clocks: Query<(&Clock, &ClockHeldBy), Changed<ClockHeldBy>>,
) {
    for (clock, held_by) in &clocks {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: CLOCK_ID,
                two_handed: CLOCK_TWO_HANDED,
                functional: false,
                passive: true,
                from_store_or_valueless: false,
            });
        } else {
            let _ = clock.stable_id;
        }
    }
}

fn clock_tick_sound(
    mut ticked_events: EventWriter<ClockTickedEvent>,
    mut clocks: Query<(Entity, &Clock, &SimPosition, &mut ClockTickState)>,
    sim_hz: Res<SimHz>,
    sim_tick: Res<crate::sim::SimTick>,
) {
    let one_second_ticks = sim_hz
        .0
        .to_num::<u16>()
        .saturating_mul(CLOCK_TICK_INTERVAL_SECONDS);

    for (entity, clock, position, mut tick_state) in &mut clocks {
        if tick_state.ticks_until_next_sound > 0 {
            tick_state.ticks_until_next_sound -= 1;
            continue;
        }

        tick_state.total_ticks_emitted = tick_state.total_ticks_emitted.wrapping_add(1);
        tick_state.last_ticked_at_sim_tick = sim_tick.0;
        tick_state.ticks_until_next_sound = one_second_ticks;

        ticked_events.send(ClockTickedEvent {
            clock_entity: entity,
            clock_stable_id: clock.stable_id,
            position: *position,
            alerts_audible_sound_reactors: CLOCK_AUDIBLE_ALERTS_ENTITIES,
        });
    }
}

fn clock_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<ClockSoldEvent>,
    clocks: Query<(&Clock, &ClockScrap)>,
) {
    for event in sell_events.read() {
        for (clock, scrap) in &clocks {
            if clock.stable_id != event.scrap_entity_id {
                continue;
            }

            sold_events.send(ClockSoldEvent {
                clock_stable_id: clock.stable_id,
                credit_value: scrap.max_value,
            });
        }
    }
}

fn clock_checksum(
    mut checksum: ResMut<SimChecksumState>,
    clocks: Query<(&Clock, &ClockScrap, &SimPosition, &ClockHeldBy, &ClockTickState)>,
) {
    accumulate_str(&mut checksum, 0x1000, CLOCK_ID);
    accumulate_str(&mut checksum, 0x1001, CLOCK_NAME);
    accumulate_str(&mut checksum, 0x1002, CLOCK_TYPE);
    accumulate_str(&mut checksum, 0x1003, CLOCK_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, CLOCK_EFFECTS);

    checksum.accumulate(CLOCK_SOURCE_REVISION as u64);
    checksum.accumulate(CLOCK_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(CLOCK_WEIGHT.to_bits() as u64);
    checksum.accumulate(CLOCK_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(CLOCK_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(CLOCK_CONDUCTIVE as u64);
    checksum.accumulate(CLOCK_TWO_HANDED as u64);
    checksum.accumulate(CLOCK_TICK_INTERVAL_SECONDS as u64);
    checksum.accumulate(CLOCK_AUDIBLE_ALERTS_ENTITIES as u64);

    for dependency in CLOCK_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for spawn_chance in CLOCK_SPAWN_CHANCES {
        accumulate_str(&mut checksum, 0x3000, spawn_chance.moon);
        checksum.accumulate(spawn_chance.chance.to_bits() as u64);
    }

    for rule in CLOCK_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x4000, rule.condition);
        accumulate_str(&mut checksum, 0x4001, rule.outcome);
    }

    for (clock, scrap, position, held_by, tick_state) in &clocks {
        checksum.accumulate(clock.stable_id);
        checksum.accumulate(scrap.min_value.to_bits() as u64);
        checksum.accumulate(scrap.max_value.to_bits() as u64);
        checksum.accumulate(scrap.weight.to_bits() as u64);
        checksum.accumulate(scrap.conductive as u64);
        checksum.accumulate(scrap.two_handed as u64);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(held_by.employee_id);
        checksum.accumulate(held_by.is_held as u64);
        checksum.accumulate(tick_state.ticks_until_next_sound as u64);
        checksum.accumulate(tick_state.total_ticks_emitted);
        checksum.accumulate(tick_state.last_ticked_at_sim_tick);
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt);
    for byte in value.as_bytes() {
        checksum.accumulate(*byte as u64);
    }
}
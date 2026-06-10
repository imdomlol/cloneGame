// Sources: vault/scrap_items/fancy_lamp.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::ItemBarPickupEvent;
use crate::sim::{SimChecksumState, SimPosition};

pub const FANCY_LAMP_ID: &str = "fancy_lamp";
pub const FANCY_LAMP_NAME: &str = "Fancy lamp";
pub const FANCY_LAMP_TYPE: &str = "scrap_items";
pub const FANCY_LAMP_SUBTYPE: &str = "scrap";
pub const FANCY_LAMP_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Fancy_Lamp";
pub const FANCY_LAMP_SOURCE_REVISION: u32 = 20211;
pub const FANCY_LAMP_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const FANCY_LAMP_CONFIDENCE_BASIS_POINTS: u16 = 95;

pub const FANCY_LAMP_EFFECTS: &str = "Emits constant light";
pub const FANCY_LAMP_WEIGHT: I32F32 = I32F32::lit("21");
pub const FANCY_LAMP_CONDUCTIVE: bool = true;
pub const FANCY_LAMP_SELL_VALUE: I32F32 = I32F32::lit("60");
pub const FANCY_LAMP_MIN_VALUE: I32F32 = I32F32::lit("60");
pub const FANCY_LAMP_MAX_VALUE: I32F32 = I32F32::lit("127");
pub const FANCY_LAMP_TWO_HANDED: bool = true;
pub const FANCY_LAMP_LIGHT_AMOUNT: I32F32 = I32F32::lit("1");
pub const FANCY_LAMP_CAN_TURN_OFF: bool = false;

pub const FANCY_LAMP_DEPENDS_ON: [&str; 8] = [
    "scrap",
    "lethal_company",
    "the_company",
    "credits",
    "rend",
    "artifice",
    "titan",
    "fancy_lamp",
];

pub const FANCY_LAMP_SPAWN_CHANCES: [FancyLampSpawnChance; 3] = [
    FancyLampSpawnChance {
        moon: "rend",
        chance: I32F32::lit("4.54"),
    },
    FancyLampSpawnChance {
        moon: "artifice",
        chance: I32F32::lit("3.91"),
    },
    FancyLampSpawnChance {
        moon: "titan",
        chance: I32F32::lit("2.91"),
    },
];

pub const FANCY_LAMP_BEHAVIORAL_MECHANICS: [FancyLampBehaviorRule; 9] = [
    FancyLampBehaviorRule {
        condition: "the fancy_lamp is present in play",
        outcome: "it emits constant light continuously",
    },
    FancyLampBehaviorRule {
        condition: "the fancy_lamp is active",
        outcome: "it cannot be turned off",
    },
    FancyLampBehaviorRule {
        condition: "the fancy_lamp is handled",
        outcome: "its weight is 21",
    },
    FancyLampBehaviorRule {
        condition: "the fancy_lamp is checked for electrical interaction",
        outcome: "it is conductive",
    },
    FancyLampBehaviorRule {
        condition: "the fancy_lamp is carried",
        outcome: "it requires two hands",
    },
    FancyLampBehaviorRule {
        condition: "the fancy_lamp is sold to the_company",
        outcome: "it can be exchanged for credits",
    },
    FancyLampBehaviorRule {
        condition: "the fancy_lamp spawns on rend",
        outcome: "its spawn chance is 4.54%",
    },
    FancyLampBehaviorRule {
        condition: "the fancy_lamp spawns on artifice",
        outcome: "its spawn chance is 3.91%",
    },
    FancyLampBehaviorRule {
        condition: "the fancy_lamp spawns on titan",
        outcome: "its spawn chance is 2.91%",
    },
];

pub struct FancyLampPlugin;

impl Plugin for FancyLampPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnFancyLampEvent>()
            .add_event::<FancyLampLightEmittedEvent>()
            .add_event::<FancyLampSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_fancy_lamp,
                    fancy_lamp_pickup_item_bar_bridge,
                    fancy_lamp_emit_constant_light,
                    fancy_lamp_sell_bridge,
                    fancy_lamp_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FancyLampBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FancyLampSpawnChance {
    pub moon: &'static str,
    pub chance: I32F32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FancyLamp {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FancyLampScrap {
    pub sell_value: I32F32,
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for FancyLampScrap {
    fn default() -> Self {
        Self {
            sell_value: FANCY_LAMP_SELL_VALUE,
            min_value: FANCY_LAMP_MIN_VALUE,
            max_value: FANCY_LAMP_MAX_VALUE,
            weight: FANCY_LAMP_WEIGHT,
            conductive: FANCY_LAMP_CONDUCTIVE,
            two_handed: FANCY_LAMP_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FancyLampHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FancyLampLightState {
    pub active: bool,
    pub can_turn_off: bool,
    pub light_amount: I32F32,
    pub emitted_ticks: u64,
}

impl Default for FancyLampLightState {
    fn default() -> Self {
        Self {
            active: true,
            can_turn_off: FANCY_LAMP_CAN_TURN_OFF,
            light_amount: FANCY_LAMP_LIGHT_AMOUNT,
            emitted_ticks: 0,
        }
    }
}

#[derive(Bundle)]
pub struct FancyLampBundle {
    pub name: Name,
    pub fancy_lamp: FancyLamp,
    pub scrap: FancyLampScrap,
    pub position: SimPosition,
    pub held_by: FancyLampHeldBy,
    pub light_state: FancyLampLightState,
}

impl FancyLampBundle {
    pub fn new(event: SpawnFancyLampEvent) -> Self {
        Self {
            name: Name::new(FANCY_LAMP_NAME),
            fancy_lamp: FancyLamp {
                stable_id: event.stable_id,
            },
            scrap: FancyLampScrap::default(),
            position: event.position,
            held_by: FancyLampHeldBy::default(),
            light_state: FancyLampLightState::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnFancyLampEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FancyLampLightEmittedEvent {
    pub source: Entity,
    pub fancy_lamp_stable_id: u64,
    pub position: SimPosition,
    pub amount: I32F32,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FancyLampSoldEvent {
    pub fancy_lamp_stable_id: u64,
    pub credit_value: I32F32,
}

pub fn fancy_lamp_value_range() -> (I32F32, I32F32) {
    (FANCY_LAMP_MIN_VALUE, FANCY_LAMP_MAX_VALUE)
}

pub fn fancy_lamp_sell_value() -> I32F32 {
    FANCY_LAMP_SELL_VALUE
}

pub fn fancy_lamp_spawn_chance_for_moon(moon: &str) -> Option<I32F32> {
    FANCY_LAMP_SPAWN_CHANCES
        .iter()
        .find(|spawn_chance| spawn_chance.moon == moon)
        .map(|spawn_chance| spawn_chance.chance)
}

fn spawn_fancy_lamp(mut commands: Commands, mut events: EventReader<SpawnFancyLampEvent>) {
    for event in events.read() {
        commands.spawn(FancyLampBundle::new(*event));
    }
}

fn fancy_lamp_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    fancy_lamps: Query<(&FancyLamp, &FancyLampHeldBy), Changed<FancyLampHeldBy>>,
) {
    for (fancy_lamp, held_by) in &fancy_lamps {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: FANCY_LAMP_ID,
                two_handed: FANCY_LAMP_TWO_HANDED,
                functional: false,
                passive: true,
                from_store_or_valueless: false,
            });
        } else {
            let _ = fancy_lamp.stable_id;
        }
    }
}

fn fancy_lamp_emit_constant_light(
    mut light_events: EventWriter<FancyLampLightEmittedEvent>,
    mut fancy_lamps: Query<(Entity, &FancyLamp, &SimPosition, &mut FancyLampLightState)>,
) {
    for (entity, fancy_lamp, position, mut light_state) in &mut fancy_lamps {
        if !light_state.active {
            light_state.active = true;
        }

        light_state.can_turn_off = FANCY_LAMP_CAN_TURN_OFF;
        light_state.emitted_ticks = light_state.emitted_ticks.wrapping_add(1);

        light_events.send(FancyLampLightEmittedEvent {
            source: entity,
            fancy_lamp_stable_id: fancy_lamp.stable_id,
            position: *position,
            amount: light_state.light_amount,
        });
    }
}

fn fancy_lamp_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<FancyLampSoldEvent>,
    fancy_lamps: Query<(&FancyLamp, &FancyLampScrap)>,
) {
    for event in sell_events.read() {
        for (fancy_lamp, scrap) in &fancy_lamps {
            if fancy_lamp.stable_id != event.scrap_entity_id {
                continue;
            }

            sold_events.send(FancyLampSoldEvent {
                fancy_lamp_stable_id: fancy_lamp.stable_id,
                credit_value: scrap.sell_value,
            });
        }
    }
}

fn fancy_lamp_checksum(
    mut checksum: ResMut<SimChecksumState>,
    fancy_lamps: Query<(
        &FancyLamp,
        &FancyLampScrap,
        &SimPosition,
        &FancyLampHeldBy,
        &FancyLampLightState,
    )>,
) {
    accumulate_str(&mut checksum, 0x1000, FANCY_LAMP_ID);
    accumulate_str(&mut checksum, 0x1001, FANCY_LAMP_NAME);
    accumulate_str(&mut checksum, 0x1002, FANCY_LAMP_TYPE);
    accumulate_str(&mut checksum, 0x1003, FANCY_LAMP_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, FANCY_LAMP_EFFECTS);

    checksum.accumulate(FANCY_LAMP_SOURCE_REVISION as u64);
    checksum.accumulate(FANCY_LAMP_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(FANCY_LAMP_WEIGHT.to_bits() as u64);
    checksum.accumulate(FANCY_LAMP_SELL_VALUE.to_bits() as u64);
    checksum.accumulate(FANCY_LAMP_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(FANCY_LAMP_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(FANCY_LAMP_CONDUCTIVE as u64);
    checksum.accumulate(FANCY_LAMP_TWO_HANDED as u64);
    checksum.accumulate(FANCY_LAMP_LIGHT_AMOUNT.to_bits() as u64);
    checksum.accumulate(FANCY_LAMP_CAN_TURN_OFF as u64);

    for dependency in FANCY_LAMP_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for spawn_chance in FANCY_LAMP_SPAWN_CHANCES {
        accumulate_str(&mut checksum, 0x3000, spawn_chance.moon);
        checksum.accumulate(spawn_chance.chance.to_bits() as u64);
    }

    for rule in FANCY_LAMP_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x4000, rule.condition);
        accumulate_str(&mut checksum, 0x4001, rule.outcome);
    }

    for (fancy_lamp, scrap, position, held_by, light_state) in &fancy_lamps {
        checksum.accumulate(fancy_lamp.stable_id);
        checksum.accumulate(scrap.sell_value.to_bits() as u64);
        checksum.accumulate(scrap.min_value.to_bits() as u64);
        checksum.accumulate(scrap.max_value.to_bits() as u64);
        checksum.accumulate(scrap.weight.to_bits() as u64);
        checksum.accumulate(scrap.conductive as u64);
        checksum.accumulate(scrap.two_handed as u64);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(held_by.employee_id);
        checksum.accumulate(held_by.is_held as u64);
        checksum.accumulate(light_state.active as u64);
        checksum.accumulate(light_state.can_turn_off as u64);
        checksum.accumulate(light_state.light_amount.to_bits() as u64);
        checksum.accumulate(light_state.emitted_ticks);
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt);
    for byte in value.as_bytes() {
        checksum.accumulate(*byte as u64);
    }
}
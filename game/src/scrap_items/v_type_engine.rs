// Sources: vault/scrap_items/v_type_engine.md, vault/item_index_pages/scrap.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::ItemBarPickupEvent;
use crate::sim::{SimChecksumState, SimPosition};

pub const V_TYPE_ENGINE_ID: &str = "v_type_engine";
pub const V_TYPE_ENGINE_NAME: &str = "V-type engine";
pub const V_TYPE_ENGINE_TYPE: &str = "scrap_items";
pub const V_TYPE_ENGINE_SUBTYPE: &str = "scrap";
pub const V_TYPE_ENGINE_SOURCE_URL: &str =
    "https://lethal-company.fandom.com/wiki/V-type_Engine";
pub const V_TYPE_ENGINE_SOURCE_REVISION: u32 = 20436;
pub const V_TYPE_ENGINE_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const V_TYPE_ENGINE_CONFIDENCE_BASIS_POINTS: u16 = 98;

pub const V_TYPE_ENGINE_WEIGHT: I32F32 = I32F32::lit("16");
pub const V_TYPE_ENGINE_CONDUCTIVE: bool = true;
pub const V_TYPE_ENGINE_TWO_HANDED: bool = true;
pub const V_TYPE_ENGINE_MIN_VALUE: I32F32 = I32F32::lit("20");
pub const V_TYPE_ENGINE_MAX_VALUE: I32F32 = I32F32::lit("55");

pub const V_TYPE_ENGINE_DEPENDS_ON: [&str; 0] = [];

pub const V_TYPE_ENGINE_SPAWN_CHANCES: [VTypeEngineSpawnChance; 10] = [
    VTypeEngineSpawnChance {
        moon: "experimentation",
        chance: I32F32::lit("15.82"),
    },
    VTypeEngineSpawnChance {
        moon: "offense",
        chance: I32F32::lit("9.6"),
    },
    VTypeEngineSpawnChance {
        moon: "embrion",
        chance: I32F32::lit("9.26"),
    },
    VTypeEngineSpawnChance {
        moon: "march",
        chance: I32F32::lit("8.22"),
    },
    VTypeEngineSpawnChance {
        moon: "assurance",
        chance: I32F32::lit("4.7"),
    },
    VTypeEngineSpawnChance {
        moon: "titan",
        chance: I32F32::lit("4.31"),
    },
    VTypeEngineSpawnChance {
        moon: "adamance",
        chance: I32F32::lit("4.22"),
    },
    VTypeEngineSpawnChance {
        moon: "vow",
        chance: I32F32::lit("2.57"),
    },
    VTypeEngineSpawnChance {
        moon: "artifice",
        chance: I32F32::lit("2.22"),
    },
    VTypeEngineSpawnChance {
        moon: "rend",
        chance: I32F32::lit("0.19"),
    },
];

pub const V_TYPE_ENGINE_BEHAVIORAL_MECHANICS: [VTypeEngineBehaviorRule; 4] = [
    VTypeEngineBehaviorRule {
        condition: "sold to the Company",
        outcome: "it yields 20 to 55 credits",
    },
    VTypeEngineBehaviorRule {
        condition: "carried",
        outcome: "it occupies two hands",
    },
    VTypeEngineBehaviorRule {
        condition: "checked for mass",
        outcome: "its weight is 16",
    },
    VTypeEngineBehaviorRule {
        condition: "checked for conductivity",
        outcome: "conductive is true",
    },
];

pub struct VTypeEnginePlugin;

impl Plugin for VTypeEnginePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnVTypeEngineEvent>()
            .add_event::<VTypeEngineSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_v_type_engine,
                    v_type_engine_pickup_item_bar_bridge,
                    v_type_engine_sell_bridge,
                    v_type_engine_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VTypeEngineSpawnChance {
    pub moon: &'static str,
    pub chance: I32F32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VTypeEngineBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VTypeEngine {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct VTypeEngineScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for VTypeEngineScrap {
    fn default() -> Self {
        Self {
            min_value: V_TYPE_ENGINE_MIN_VALUE,
            max_value: V_TYPE_ENGINE_MAX_VALUE,
            weight: V_TYPE_ENGINE_WEIGHT,
            conductive: V_TYPE_ENGINE_CONDUCTIVE,
            two_handed: V_TYPE_ENGINE_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VTypeEngineHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Bundle)]
pub struct VTypeEngineBundle {
    pub name: Name,
    pub v_type_engine: VTypeEngine,
    pub scrap: VTypeEngineScrap,
    pub position: SimPosition,
    pub held_by: VTypeEngineHeldBy,
}

impl VTypeEngineBundle {
    pub fn new(event: SpawnVTypeEngineEvent) -> Self {
        Self {
            name: Name::new(V_TYPE_ENGINE_NAME),
            v_type_engine: VTypeEngine {
                stable_id: event.stable_id,
            },
            scrap: VTypeEngineScrap::default(),
            position: event.position,
            held_by: VTypeEngineHeldBy::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnVTypeEngineEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct VTypeEngineSoldEvent {
    pub stable_id: u64,
    pub credit_value: I32F32,
}

pub fn v_type_engine_value_range() -> (I32F32, I32F32) {
    (V_TYPE_ENGINE_MIN_VALUE, V_TYPE_ENGINE_MAX_VALUE)
}

pub fn v_type_engine_spawn_chance_for_moon(moon: &str) -> Option<I32F32> {
    V_TYPE_ENGINE_SPAWN_CHANCES
        .iter()
        .find(|sc| sc.moon == moon)
        .map(|sc| sc.chance)
}

fn spawn_v_type_engine(
    mut commands: Commands,
    mut events: EventReader<SpawnVTypeEngineEvent>,
) {
    for event in events.read() {
        commands.spawn(VTypeEngineBundle::new(*event));
    }
}

fn v_type_engine_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    engines: Query<(&VTypeEngine, &VTypeEngineHeldBy), Changed<VTypeEngineHeldBy>>,
) {
    for (engine, held_by) in &engines {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: V_TYPE_ENGINE_ID,
                two_handed: V_TYPE_ENGINE_TWO_HANDED,
                functional: false,
                passive: false,
                from_store_or_valueless: false,
            });
        } else {
            let _ = engine.stable_id;
        }
    }
}

fn v_type_engine_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<VTypeEngineSoldEvent>,
    engines: Query<(&VTypeEngine, &VTypeEngineScrap)>,
) {
    for event in sell_events.read() {
        for (engine, scrap) in &engines {
            if engine.stable_id != event.scrap_entity_id {
                continue;
            }
            sold_events.send(VTypeEngineSoldEvent {
                stable_id: engine.stable_id,
                credit_value: scrap.max_value,
            });
        }
    }
}

fn v_type_engine_checksum(
    mut checksum: ResMut<SimChecksumState>,
    engines: Query<(&VTypeEngine, &VTypeEngineScrap, &SimPosition, &VTypeEngineHeldBy)>,
) {
    accumulate_str(&mut checksum, 0x1000, V_TYPE_ENGINE_ID);
    accumulate_str(&mut checksum, 0x1001, V_TYPE_ENGINE_NAME);
    accumulate_str(&mut checksum, 0x1002, V_TYPE_ENGINE_TYPE);
    accumulate_str(&mut checksum, 0x1003, V_TYPE_ENGINE_SUBTYPE);

    checksum.accumulate(V_TYPE_ENGINE_SOURCE_REVISION as u64);
    checksum.accumulate(V_TYPE_ENGINE_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(V_TYPE_ENGINE_WEIGHT.to_bits() as u64);
    checksum.accumulate(V_TYPE_ENGINE_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(V_TYPE_ENGINE_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(V_TYPE_ENGINE_CONDUCTIVE as u64);
    checksum.accumulate(V_TYPE_ENGINE_TWO_HANDED as u64);

    for sc in V_TYPE_ENGINE_SPAWN_CHANCES {
        accumulate_str(&mut checksum, 0x4000, sc.moon);
        checksum.accumulate(sc.chance.to_bits() as u64);
    }

    for rule in V_TYPE_ENGINE_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x5000, rule.condition);
        accumulate_str(&mut checksum, 0x5001, rule.outcome);
    }

    for (engine, scrap, position, held_by) in &engines {
        checksum.accumulate(engine.stable_id);
        checksum.accumulate(scrap.min_value.to_bits() as u64);
        checksum.accumulate(scrap.max_value.to_bits() as u64);
        checksum.accumulate(scrap.weight.to_bits() as u64);
        checksum.accumulate(scrap.conductive as u64);
        checksum.accumulate(scrap.two_handed as u64);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(held_by.employee_id);
        checksum.accumulate(held_by.is_held as u64);
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt);
    for byte in value.as_bytes() {
        checksum.accumulate(*byte as u64);
    }
}
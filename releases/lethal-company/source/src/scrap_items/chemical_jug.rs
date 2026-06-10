// Sources: vault/scrap_items/chemical_jug.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::ItemBarPickupEvent;
use crate::sim::{SimChecksumState, SimPosition};

pub const CHEMICAL_JUG_ID: &str = "chemical_jug";
pub const CHEMICAL_JUG_NAME: &str = "Chemical jug";
pub const CHEMICAL_JUG_TYPE: &str = "scrap_items";
pub const CHEMICAL_JUG_SUBTYPE: &str = "scrap";
pub const CHEMICAL_JUG_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Chemical_Jug";
pub const CHEMICAL_JUG_SOURCE_REVISION: u32 = 20214;
pub const CHEMICAL_JUG_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const CHEMICAL_JUG_CONFIDENCE_BASIS_POINTS: u16 = 96;

pub const CHEMICAL_JUG_WEIGHT: I32F32 = I32F32::lit("31");
pub const CHEMICAL_JUG_CONDUCTIVE: bool = false;
pub const CHEMICAL_JUG_MIN_VALUE: I32F32 = I32F32::lit("32");
pub const CHEMICAL_JUG_MAX_VALUE: I32F32 = I32F32::lit("83");
pub const CHEMICAL_JUG_TWO_HANDED: bool = true;
pub const CHEMICAL_JUG_USAGE_NOTE: &str =
    "It has no documented use beyond being sold to the_company.";

pub const CHEMICAL_JUG_DEPENDS_ON: [&str; 5] = [
    "lethal_company",
    "scrap",
    "the_company",
    "vow",
    "adamance",
];

pub const CHEMICAL_JUG_SPAWN_CHANCES: [ChemicalJugSpawnChance; 2] = [
    ChemicalJugSpawnChance {
        moon: "vow",
        chance: I32F32::lit("4.73"),
    },
    ChemicalJugSpawnChance {
        moon: "adamance",
        chance: I32F32::lit("4.33"),
    },
];

pub const CHEMICAL_JUG_BEHAVIORAL_MECHANICS: [ChemicalJugBehaviorRule; 6] = [
    ChemicalJugBehaviorRule {
        condition: "the Chemical jug is carried",
        outcome: "it weighs 31",
    },
    ChemicalJugBehaviorRule {
        condition: "the Chemical jug is checked for material behavior",
        outcome: "it is conductive: false",
    },
    ChemicalJugBehaviorRule {
        condition: "the Chemical jug is handled",
        outcome: "it is two_handed: true",
    },
    ChemicalJugBehaviorRule {
        condition: "the Chemical jug is sold",
        outcome: "its only documented use is sale to the_company",
    },
    ChemicalJugBehaviorRule {
        condition: "the Chemical jug spawns on vow",
        outcome: "spawn chance is 4.73%",
    },
    ChemicalJugBehaviorRule {
        condition: "the Chemical jug spawns on adamance",
        outcome: "spawn chance is 4.33%",
    },
];

pub struct ChemicalJugPlugin;

impl Plugin for ChemicalJugPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnChemicalJugEvent>()
            .add_event::<ChemicalJugSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_chemical_jug,
                    chemical_jug_pickup_item_bar_bridge,
                    chemical_jug_sell_bridge,
                    chemical_jug_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChemicalJugBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChemicalJugSpawnChance {
    pub moon: &'static str,
    pub chance: I32F32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ChemicalJug {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChemicalJugScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for ChemicalJugScrap {
    fn default() -> Self {
        Self {
            min_value: CHEMICAL_JUG_MIN_VALUE,
            max_value: CHEMICAL_JUG_MAX_VALUE,
            weight: CHEMICAL_JUG_WEIGHT,
            conductive: CHEMICAL_JUG_CONDUCTIVE,
            two_handed: CHEMICAL_JUG_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ChemicalJugHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Bundle)]
pub struct ChemicalJugBundle {
    pub name: Name,
    pub chemical_jug: ChemicalJug,
    pub scrap: ChemicalJugScrap,
    pub position: SimPosition,
    pub held_by: ChemicalJugHeldBy,
}

impl ChemicalJugBundle {
    pub fn new(event: SpawnChemicalJugEvent) -> Self {
        Self {
            name: Name::new(CHEMICAL_JUG_NAME),
            chemical_jug: ChemicalJug {
                stable_id: event.stable_id,
            },
            scrap: ChemicalJugScrap::default(),
            position: event.position,
            held_by: ChemicalJugHeldBy::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnChemicalJugEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChemicalJugSoldEvent {
    pub chemical_jug_stable_id: u64,
    pub credit_value: I32F32,
}

pub fn chemical_jug_value_range() -> (I32F32, I32F32) {
    (CHEMICAL_JUG_MIN_VALUE, CHEMICAL_JUG_MAX_VALUE)
}

pub fn chemical_jug_spawn_chance_for_moon(moon: &str) -> Option<I32F32> {
    CHEMICAL_JUG_SPAWN_CHANCES
        .iter()
        .find(|spawn_chance| spawn_chance.moon == moon)
        .map(|spawn_chance| spawn_chance.chance)
}

fn spawn_chemical_jug(mut commands: Commands, mut events: EventReader<SpawnChemicalJugEvent>) {
    for event in events.read() {
        commands.spawn(ChemicalJugBundle::new(*event));
    }
}

fn chemical_jug_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    chemical_jugs: Query<(&ChemicalJug, &ChemicalJugHeldBy), Changed<ChemicalJugHeldBy>>,
) {
    for (chemical_jug, held_by) in &chemical_jugs {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: CHEMICAL_JUG_ID,
                two_handed: CHEMICAL_JUG_TWO_HANDED,
                functional: false,
                passive: true,
                from_store_or_valueless: false,
            });
        } else {
            let _ = chemical_jug.stable_id;
        }
    }
}

fn chemical_jug_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<ChemicalJugSoldEvent>,
    chemical_jugs: Query<(&ChemicalJug, &ChemicalJugScrap)>,
) {
    for event in sell_events.read() {
        for (chemical_jug, scrap) in &chemical_jugs {
            if chemical_jug.stable_id != event.scrap_entity_id {
                continue;
            }

            sold_events.send(ChemicalJugSoldEvent {
                chemical_jug_stable_id: chemical_jug.stable_id,
                credit_value: scrap.max_value,
            });
        }
    }
}

fn chemical_jug_checksum(
    mut checksum: ResMut<SimChecksumState>,
    chemical_jugs: Query<(&ChemicalJug, &ChemicalJugScrap, &SimPosition, &ChemicalJugHeldBy)>,
) {
    accumulate_str(&mut checksum, 0x1000, CHEMICAL_JUG_ID);
    accumulate_str(&mut checksum, 0x1001, CHEMICAL_JUG_NAME);
    accumulate_str(&mut checksum, 0x1002, CHEMICAL_JUG_TYPE);
    accumulate_str(&mut checksum, 0x1003, CHEMICAL_JUG_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, CHEMICAL_JUG_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1005, CHEMICAL_JUG_EXTRACTED_AT);
    accumulate_str(&mut checksum, 0x1006, CHEMICAL_JUG_USAGE_NOTE);

    checksum.accumulate(CHEMICAL_JUG_SOURCE_REVISION as u64);
    checksum.accumulate(CHEMICAL_JUG_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(CHEMICAL_JUG_WEIGHT.to_bits() as u64);
    checksum.accumulate(CHEMICAL_JUG_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(CHEMICAL_JUG_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(CHEMICAL_JUG_CONDUCTIVE as u64);
    checksum.accumulate(CHEMICAL_JUG_TWO_HANDED as u64);

    for dependency in CHEMICAL_JUG_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for spawn_chance in CHEMICAL_JUG_SPAWN_CHANCES {
        accumulate_str(&mut checksum, 0x3000, spawn_chance.moon);
        checksum.accumulate(spawn_chance.chance.to_bits() as u64);
    }

    for rule in CHEMICAL_JUG_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x4000, rule.condition);
        accumulate_str(&mut checksum, 0x4001, rule.outcome);
    }

    for (chemical_jug, scrap, position, held_by) in &chemical_jugs {
        checksum.accumulate(chemical_jug.stable_id);
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
// Sources: vault/scrap_items/tattered_metal_sheet.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::ItemBarPickupEvent;
use crate::sim::{SimChecksumState, SimPosition};

pub const TATTERED_METAL_SHEET_ID: &str = "tattered_metal_sheet";
pub const TATTERED_METAL_SHEET_NAME: &str = "Tattered metal sheet";
pub const TATTERED_METAL_SHEET_TYPE: &str = "scrap_items";
pub const TATTERED_METAL_SHEET_SUBTYPE: &str = "scrap";
pub const TATTERED_METAL_SHEET_SOURCE_URL: &str =
    "https://lethal-company.fandom.com/wiki/Tattered_Metal_Sheet";
pub const TATTERED_METAL_SHEET_SOURCE_REVISION: u32 = 21176;
pub const TATTERED_METAL_SHEET_EXTRACTED_AT: &str = "2026-06-07";
pub const TATTERED_METAL_SHEET_CONFIDENCE_BASIS_POINTS: u16 = 91;

pub const TATTERED_METAL_SHEET_EFFECTS: &str = "No functional use; sold for credits.";
pub const TATTERED_METAL_SHEET_WEIGHT: I32F32 = I32F32::lit("26");
pub const TATTERED_METAL_SHEET_CONDUCTIVE: bool = true;
pub const TATTERED_METAL_SHEET_TWO_HANDED: bool = false;
pub const TATTERED_METAL_SHEET_MIN_VALUE: I32F32 = I32F32::lit("10");
pub const TATTERED_METAL_SHEET_MAX_VALUE: I32F32 = I32F32::lit("21");

// Label shown when scanned vs when collected or sold.
pub const TATTERED_METAL_SHEET_SCAN_LABEL: &str = "Tattered metal sheet";
pub const TATTERED_METAL_SHEET_COLLECTION_LABEL: &str = "Metal sheet";

pub const TATTERED_METAL_SHEET_DEPENDS_ON: [&str; 12] = [
    "scrap",
    "lethal_company",
    "experimentation",
    "employee",
    "embrion",
    "march",
    "offense",
    "assurance",
    "adamance",
    "vow",
    "the_company",
    "credits",
];

pub const TATTERED_METAL_SHEET_SPAWN_CHANCES: [TatteredMetalSheetSpawnChance; 7] = [
    TatteredMetalSheetSpawnChance {
        moon: "experimentation",
        chance: I32F32::lit("15.47"),
    },
    TatteredMetalSheetSpawnChance {
        moon: "embrion",
        chance: I32F32::lit("11.57"),
    },
    TatteredMetalSheetSpawnChance {
        moon: "march",
        chance: I32F32::lit("9.47"),
    },
    TatteredMetalSheetSpawnChance {
        moon: "offense",
        chance: I32F32::lit("7.8"),
    },
    TatteredMetalSheetSpawnChance {
        moon: "assurance",
        chance: I32F32::lit("2.7"),
    },
    TatteredMetalSheetSpawnChance {
        moon: "adamance",
        chance: I32F32::lit("1.69"),
    },
    TatteredMetalSheetSpawnChance {
        moon: "vow",
        chance: I32F32::lit("1.64"),
    },
];

pub const TATTERED_METAL_SHEET_BEHAVIORAL_MECHANICS: [TatteredMetalSheetBehaviorRule; 10] = [
    TatteredMetalSheetBehaviorRule {
        condition: "the item is scanned",
        outcome: "it is labeled Tattered metal sheet",
    },
    TatteredMetalSheetBehaviorRule {
        condition: "the item is collected or sold",
        outcome: "it is labeled Metal sheet",
    },
    TatteredMetalSheetBehaviorRule {
        condition: "the item spawns on experimentation",
        outcome: "the spawn chance is 15.47%",
    },
    TatteredMetalSheetBehaviorRule {
        condition: "the item spawns on embrion",
        outcome: "the spawn chance is 11.57%",
    },
    TatteredMetalSheetBehaviorRule {
        condition: "the item spawns on march",
        outcome: "the spawn chance is 9.47%",
    },
    TatteredMetalSheetBehaviorRule {
        condition: "the item spawns on offense",
        outcome: "the spawn chance is 7.8%",
    },
    TatteredMetalSheetBehaviorRule {
        condition: "the item spawns on assurance",
        outcome: "the spawn chance is 2.7%",
    },
    TatteredMetalSheetBehaviorRule {
        condition: "the item spawns on adamance",
        outcome: "the spawn chance is 1.69%",
    },
    TatteredMetalSheetBehaviorRule {
        condition: "the item spawns on vow",
        outcome: "the spawn chance is 1.64%",
    },
    TatteredMetalSheetBehaviorRule {
        condition: "the item appears as either the 2-hole variant or the 4-hole variant",
        outcome: "the appearance has 0 gameplay effect",
    },
];

pub struct TatteredMetalSheetPlugin;

impl Plugin for TatteredMetalSheetPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnTatteredMetalSheetEvent>()
            .add_event::<TatteredMetalSheetSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_tattered_metal_sheet,
                    tattered_metal_sheet_pickup_item_bar_bridge,
                    tattered_metal_sheet_sell_bridge,
                    tattered_metal_sheet_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TatteredMetalSheetBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TatteredMetalSheetSpawnChance {
    pub moon: &'static str,
    pub chance: I32F32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TatteredMetalSheet {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TatteredMetalSheetScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for TatteredMetalSheetScrap {
    fn default() -> Self {
        Self {
            min_value: TATTERED_METAL_SHEET_MIN_VALUE,
            max_value: TATTERED_METAL_SHEET_MAX_VALUE,
            weight: TATTERED_METAL_SHEET_WEIGHT,
            conductive: TATTERED_METAL_SHEET_CONDUCTIVE,
            two_handed: TATTERED_METAL_SHEET_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TatteredMetalSheetHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Bundle)]
pub struct TatteredMetalSheetBundle {
    pub name: Name,
    pub tattered_metal_sheet: TatteredMetalSheet,
    pub scrap: TatteredMetalSheetScrap,
    pub position: SimPosition,
    pub held_by: TatteredMetalSheetHeldBy,
}

impl TatteredMetalSheetBundle {
    pub fn new(event: SpawnTatteredMetalSheetEvent) -> Self {
        Self {
            name: Name::new(TATTERED_METAL_SHEET_NAME),
            tattered_metal_sheet: TatteredMetalSheet {
                stable_id: event.stable_id,
            },
            scrap: TatteredMetalSheetScrap::default(),
            position: event.position,
            held_by: TatteredMetalSheetHeldBy::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnTatteredMetalSheetEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TatteredMetalSheetSoldEvent {
    pub stable_id: u64,
    pub credit_value: I32F32,
}

pub fn tattered_metal_sheet_value_range() -> (I32F32, I32F32) {
    (TATTERED_METAL_SHEET_MIN_VALUE, TATTERED_METAL_SHEET_MAX_VALUE)
}

pub fn tattered_metal_sheet_spawn_chance_for_moon(moon: &str) -> Option<I32F32> {
    TATTERED_METAL_SHEET_SPAWN_CHANCES
        .iter()
        .find(|s| s.moon == moon)
        .map(|s| s.chance)
}

fn spawn_tattered_metal_sheet(
    mut commands: Commands,
    mut events: EventReader<SpawnTatteredMetalSheetEvent>,
) {
    for event in events.read() {
        commands.spawn(TatteredMetalSheetBundle::new(*event));
    }
}

fn tattered_metal_sheet_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    items: Query<(&TatteredMetalSheet, &TatteredMetalSheetHeldBy), Changed<TatteredMetalSheetHeldBy>>,
) {
    for (item, held_by) in &items {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: TATTERED_METAL_SHEET_ID,
                two_handed: TATTERED_METAL_SHEET_TWO_HANDED,
                functional: false,
                passive: false,
                from_store_or_valueless: false,
            });
        } else {
            let _ = item.stable_id;
        }
    }
}

fn tattered_metal_sheet_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<TatteredMetalSheetSoldEvent>,
    items: Query<(&TatteredMetalSheet, &TatteredMetalSheetScrap)>,
) {
    for event in sell_events.read() {
        for (item, scrap) in &items {
            if item.stable_id != event.scrap_entity_id {
                continue;
            }
            sold_events.send(TatteredMetalSheetSoldEvent {
                stable_id: item.stable_id,
                credit_value: scrap.max_value,
            });
        }
    }
}

fn tattered_metal_sheet_checksum(
    mut checksum: ResMut<SimChecksumState>,
    items: Query<(
        &TatteredMetalSheet,
        &TatteredMetalSheetScrap,
        &SimPosition,
        &TatteredMetalSheetHeldBy,
    )>,
) {
    accumulate_str(&mut checksum, 0x1000, TATTERED_METAL_SHEET_ID);
    accumulate_str(&mut checksum, 0x1001, TATTERED_METAL_SHEET_NAME);
    accumulate_str(&mut checksum, 0x1002, TATTERED_METAL_SHEET_TYPE);
    accumulate_str(&mut checksum, 0x1003, TATTERED_METAL_SHEET_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, TATTERED_METAL_SHEET_EFFECTS);
    accumulate_str(&mut checksum, 0x1005, TATTERED_METAL_SHEET_SCAN_LABEL);
    accumulate_str(&mut checksum, 0x1006, TATTERED_METAL_SHEET_COLLECTION_LABEL);

    checksum.accumulate(TATTERED_METAL_SHEET_SOURCE_REVISION as u64);
    checksum.accumulate(TATTERED_METAL_SHEET_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(TATTERED_METAL_SHEET_WEIGHT.to_bits() as u64);
    checksum.accumulate(TATTERED_METAL_SHEET_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(TATTERED_METAL_SHEET_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(TATTERED_METAL_SHEET_CONDUCTIVE as u64);
    checksum.accumulate(TATTERED_METAL_SHEET_TWO_HANDED as u64);

    for dependency in TATTERED_METAL_SHEET_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for spawn_chance in TATTERED_METAL_SHEET_SPAWN_CHANCES {
        accumulate_str(&mut checksum, 0x4000, spawn_chance.moon);
        checksum.accumulate(spawn_chance.chance.to_bits() as u64);
    }

    for rule in TATTERED_METAL_SHEET_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x5000, rule.condition);
        accumulate_str(&mut checksum, 0x5001, rule.outcome);
    }

    for (item, scrap, position, held_by) in &items {
        checksum.accumulate(item.stable_id);
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
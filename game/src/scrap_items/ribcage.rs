// Sources: vault/scrap_items/ribcage.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::ItemBarPickupEvent;
use crate::sim::{SimChecksumState, SimPosition};

pub const RIBCAGE_ID: &str = "ribcage";
pub const RIBCAGE_NAME: &str = "Ribcage";
pub const RIBCAGE_TYPE: &str = "scrap_items";
pub const RIBCAGE_SUBTYPE: &str = "scrap";
pub const RIBCAGE_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Ribcage";
pub const RIBCAGE_SOURCE_REVISION: u32 = 20362;
pub const RIBCAGE_EXTRACTED_AT: &str = "2026-06-07T00:00:00Z";
pub const RIBCAGE_CONFIDENCE_BASIS_POINTS: u16 = 96;

pub const RIBCAGE_WEIGHT: I32F32 = I32F32::lit("0");
pub const RIBCAGE_CONDUCTIVE: bool = false;
pub const RIBCAGE_TWO_HANDED: bool = true;
pub const RIBCAGE_MIN_VALUE: I32F32 = I32F32::lit("8");
pub const RIBCAGE_MAX_VALUE: I32F32 = I32F32::lit("25");

pub const RIBCAGE_DEPENDS_ON: [&str; 5] = [
    "scrap",
    "lethal_company",
    "dine",
    "the_company",
    "credits",
];

pub const RIBCAGE_SPAWN_CHANCES: [RibcageSpawnChance; 1] = [RibcageSpawnChance {
    moon: "dine",
    chance: I32F32::lit("15.99"),
}];

pub const RIBCAGE_BEHAVIORAL_MECHANICS: [RibcageBehaviorRule; 4] = [
    RibcageBehaviorRule {
        condition: "the item is carried",
        outcome: "its weight is 0 and two_handed is true",
    },
    RibcageBehaviorRule {
        condition: "the item is checked for conductivity",
        outcome: "conductive is false",
    },
    RibcageBehaviorRule {
        condition: "the item spawns",
        outcome: "it can appear only on dine with a spawn chance of 15.99%",
    },
    RibcageBehaviorRule {
        condition: "the item is sold",
        outcome: "no special use is documented beyond conversion to credits at the_company",
    },
];

pub struct RibcagePlugin;

impl Plugin for RibcagePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnRibcageEvent>()
            .add_event::<RibcageSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_ribcage,
                    ribcage_pickup_item_bar_bridge,
                    ribcage_sell_bridge,
                    ribcage_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RibcageBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RibcageSpawnChance {
    pub moon: &'static str,
    pub chance: I32F32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Ribcage {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct RibcageScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for RibcageScrap {
    fn default() -> Self {
        Self {
            min_value: RIBCAGE_MIN_VALUE,
            max_value: RIBCAGE_MAX_VALUE,
            weight: RIBCAGE_WEIGHT,
            conductive: RIBCAGE_CONDUCTIVE,
            two_handed: RIBCAGE_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RibcageHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Bundle)]
pub struct RibcageBundle {
    pub name: Name,
    pub ribcage: Ribcage,
    pub scrap: RibcageScrap,
    pub position: SimPosition,
    pub held_by: RibcageHeldBy,
}

impl RibcageBundle {
    pub fn new(event: SpawnRibcageEvent) -> Self {
        Self {
            name: Name::new(RIBCAGE_NAME),
            ribcage: Ribcage {
                stable_id: event.stable_id,
            },
            scrap: RibcageScrap::default(),
            position: event.position,
            held_by: RibcageHeldBy::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnRibcageEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RibcageSoldEvent {
    pub ribcage_stable_id: u64,
    pub credit_value: I32F32,
}

pub fn ribcage_value_range() -> (I32F32, I32F32) {
    (RIBCAGE_MIN_VALUE, RIBCAGE_MAX_VALUE)
}

pub fn ribcage_spawn_chance_for_moon(moon: &str) -> Option<I32F32> {
    RIBCAGE_SPAWN_CHANCES
        .iter()
        .find(|sc| sc.moon == moon)
        .map(|sc| sc.chance)
}

fn spawn_ribcage(mut commands: Commands, mut events: EventReader<SpawnRibcageEvent>) {
    for event in events.read() {
        commands.spawn(RibcageBundle::new(*event));
    }
}

fn ribcage_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    ribcages: Query<(&Ribcage, &RibcageHeldBy), Changed<RibcageHeldBy>>,
) {
    for (ribcage, held_by) in &ribcages {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: RIBCAGE_ID,
                two_handed: RIBCAGE_TWO_HANDED,
                functional: false,
                passive: false,
                from_store_or_valueless: false,
            });
        } else {
            let _ = ribcage.stable_id;
        }
    }
}

fn ribcage_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<RibcageSoldEvent>,
    ribcages: Query<(&Ribcage, &RibcageScrap)>,
) {
    for event in sell_events.read() {
        for (ribcage, scrap) in &ribcages {
            if ribcage.stable_id != event.scrap_entity_id {
                continue;
            }
            sold_events.send(RibcageSoldEvent {
                ribcage_stable_id: ribcage.stable_id,
                credit_value: scrap.max_value,
            });
        }
    }
}

fn ribcage_checksum(
    mut checksum: ResMut<SimChecksumState>,
    ribcages: Query<(&Ribcage, &RibcageScrap, &SimPosition, &RibcageHeldBy)>,
) {
    accumulate_str(&mut checksum, 0x1000, RIBCAGE_ID);
    accumulate_str(&mut checksum, 0x1001, RIBCAGE_NAME);
    accumulate_str(&mut checksum, 0x1002, RIBCAGE_TYPE);
    accumulate_str(&mut checksum, 0x1003, RIBCAGE_SUBTYPE);

    checksum.accumulate(RIBCAGE_SOURCE_REVISION as u64);
    checksum.accumulate(RIBCAGE_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(RIBCAGE_WEIGHT.to_bits() as u64);
    checksum.accumulate(RIBCAGE_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(RIBCAGE_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(RIBCAGE_CONDUCTIVE as u64);
    checksum.accumulate(RIBCAGE_TWO_HANDED as u64);

    for dependency in RIBCAGE_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for sc in RIBCAGE_SPAWN_CHANCES {
        accumulate_str(&mut checksum, 0x4000, sc.moon);
        checksum.accumulate(sc.chance.to_bits() as u64);
    }

    for rule in RIBCAGE_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x5000, rule.condition);
        accumulate_str(&mut checksum, 0x5001, rule.outcome);
    }

    for (ribcage, scrap, position, held_by) in &ribcages {
        checksum.accumulate(ribcage.stable_id);
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
// Sources: vault/scrap_items/ear.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::ItemBarPickupEvent;
use crate::sim::{SimChecksumState, SimPosition};

pub const EAR_ID: &str = "ear";
pub const EAR_NAME: &str = "Ear";
pub const EAR_TYPE: &str = "scrap_items";
pub const EAR_SUBTYPE: &str = "scrap";
pub const EAR_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Ear";
pub const EAR_SOURCE_REVISION: u32 = 20287;
pub const EAR_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const EAR_CONFIDENCE_BASIS_POINTS: u16 = 96;

pub const EAR_EFFECTS: &str = "No use beyond being sold for credits.";
pub const EAR_WEIGHT: I32F32 = I32F32::lit("0");
pub const EAR_CONDUCTIVE: bool = false;
pub const EAR_MIN_VALUE: I32F32 = I32F32::lit("2");
pub const EAR_MAX_VALUE: I32F32 = I32F32::lit("13");
pub const EAR_TWO_HANDED: bool = false;

pub const EAR_DEPENDS_ON: [&str; 2] = ["dine", "the_company"];

pub const EAR_SPAWN_CHANCES: [EarSpawnChance; 1] = [EarSpawnChance {
    moon: "dine",
    chance: I32F32::lit("8.3"),
}];

pub const EAR_BEHAVIORAL_MECHANICS: [EarBehaviorRule; 4] = [
    EarBehaviorRule {
        condition: "the item spawns",
        outcome: "it can appear only on dine with an 8.3% spawn chance",
    },
    EarBehaviorRule {
        condition: "the item is carried",
        outcome: "it weighs 0 and is not conductive",
    },
    EarBehaviorRule {
        condition: "the item is handled as loot",
        outcome: "it has no use beyond being sold to the_company for credits",
    },
    EarBehaviorRule {
        condition: "the item is checked for carrying requirements",
        outcome: "it is not two-handed",
    },
];

pub struct EarPlugin;

impl Plugin for EarPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnEarEvent>()
            .add_event::<EarSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_ear,
                    ear_pickup_item_bar_bridge,
                    ear_sell_bridge,
                    ear_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EarBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EarSpawnChance {
    pub moon: &'static str,
    pub chance: I32F32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Ear {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct EarScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for EarScrap {
    fn default() -> Self {
        Self {
            min_value: EAR_MIN_VALUE,
            max_value: EAR_MAX_VALUE,
            weight: EAR_WEIGHT,
            conductive: EAR_CONDUCTIVE,
            two_handed: EAR_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EarHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Bundle)]
pub struct EarBundle {
    pub name: Name,
    pub ear: Ear,
    pub scrap: EarScrap,
    pub position: SimPosition,
    pub held_by: EarHeldBy,
}

impl EarBundle {
    pub fn new(event: SpawnEarEvent) -> Self {
        Self {
            name: Name::new(EAR_NAME),
            ear: Ear {
                stable_id: event.stable_id,
            },
            scrap: EarScrap::default(),
            position: event.position,
            held_by: EarHeldBy::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnEarEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EarSoldEvent {
    pub ear_stable_id: u64,
    pub credit_value: I32F32,
}

pub fn ear_value_range() -> (I32F32, I32F32) {
    (EAR_MIN_VALUE, EAR_MAX_VALUE)
}

pub fn ear_spawn_chance_for_moon(moon: &str) -> Option<I32F32> {
    EAR_SPAWN_CHANCES
        .iter()
        .find(|spawn_chance| spawn_chance.moon == moon)
        .map(|spawn_chance| spawn_chance.chance)
}

fn spawn_ear(mut commands: Commands, mut events: EventReader<SpawnEarEvent>) {
    for event in events.read() {
        commands.spawn(EarBundle::new(*event));
    }
}

fn ear_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    ears: Query<(&Ear, &EarHeldBy), Changed<EarHeldBy>>,
) {
    for (ear, held_by) in &ears {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: EAR_ID,
                two_handed: EAR_TWO_HANDED,
                functional: false,
                passive: true,
                from_store_or_valueless: false,
            });
        } else {
            let _ = ear.stable_id;
        }
    }
}

fn ear_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<EarSoldEvent>,
    ears: Query<(&Ear, &EarScrap)>,
) {
    for event in sell_events.read() {
        for (ear, scrap) in &ears {
            if ear.stable_id != event.scrap_entity_id {
                continue;
            }

            sold_events.send(EarSoldEvent {
                ear_stable_id: ear.stable_id,
                credit_value: scrap.max_value,
            });
        }
    }
}

fn ear_checksum(
    mut checksum: ResMut<SimChecksumState>,
    ears: Query<(&Ear, &EarScrap, &SimPosition, &EarHeldBy)>,
) {
    accumulate_str(&mut checksum, 0x1000, EAR_ID);
    accumulate_str(&mut checksum, 0x1001, EAR_NAME);
    accumulate_str(&mut checksum, 0x1002, EAR_TYPE);
    accumulate_str(&mut checksum, 0x1003, EAR_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, EAR_EFFECTS);
    accumulate_str(&mut checksum, 0x1005, EAR_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1006, EAR_EXTRACTED_AT);

    checksum.accumulate(EAR_SOURCE_REVISION as u64);
    checksum.accumulate(EAR_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(EAR_WEIGHT.to_bits() as u64);
    checksum.accumulate(EAR_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(EAR_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(EAR_CONDUCTIVE as u64);
    checksum.accumulate(EAR_TWO_HANDED as u64);

    for dependency in EAR_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for spawn_chance in EAR_SPAWN_CHANCES {
        accumulate_str(&mut checksum, 0x4000, spawn_chance.moon);
        checksum.accumulate(spawn_chance.chance.to_bits() as u64);
    }

    for rule in EAR_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x5000, rule.condition);
        accumulate_str(&mut checksum, 0x5001, rule.outcome);
    }

    for (ear, scrap, position, held_by) in &ears {
        checksum.accumulate(ear.stable_id);
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
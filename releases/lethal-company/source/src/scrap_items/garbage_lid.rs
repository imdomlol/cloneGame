// Sources: vault/scrap_items/garbage_lid.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::ItemBarPickupEvent;
use crate::sim::{SimChecksumState, SimPosition};

pub const GARBAGE_LID_ID: &str = "garbage_lid";
pub const GARBAGE_LID_NAME: &str = "Garbage lid";
pub const GARBAGE_LID_TYPE: &str = "scrap_items";
pub const GARBAGE_LID_SUBTYPE: &str = "scrap";
pub const GARBAGE_LID_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Garbage_Lid";
pub const GARBAGE_LID_SOURCE_REVISION: u32 = 20216;
pub const GARBAGE_LID_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const GARBAGE_LID_CONFIDENCE_BASIS_POINTS: u16 = 97;

pub const GARBAGE_LID_WEIGHT: I32F32 = I32F32::lit("0");
pub const GARBAGE_LID_CONDUCTIVE: bool = true;
pub const GARBAGE_LID_TWO_HANDED: bool = true;
pub const GARBAGE_LID_MIN_VALUE: I32F32 = I32F32::lit("20");
pub const GARBAGE_LID_MAX_VALUE: I32F32 = I32F32::lit("43");

pub const GARBAGE_LID_DEPENDS_ON: [&str; 10] = [
    "scrap",
    "landmine",
    "the_company",
    "credits",
    "offense",
    "assurance",
    "vow",
    "march",
    "adamance",
    "artifice",
];

pub const GARBAGE_LID_SPAWN_CHANCES: [GarbageLidSpawnChance; 7] = [
    GarbageLidSpawnChance {
        moon: "offense",
        chance: I32F32::lit("3.72"),
    },
    GarbageLidSpawnChance {
        moon: "assurance",
        chance: I32F32::lit("3.64"),
    },
    GarbageLidSpawnChance {
        moon: "vow",
        chance: I32F32::lit("2.77"),
    },
    GarbageLidSpawnChance {
        moon: "march",
        chance: I32F32::lit("2.28"),
    },
    GarbageLidSpawnChance {
        moon: "adamance",
        chance: I32F32::lit("2.01"),
    },
    GarbageLidSpawnChance {
        moon: "artifice",
        chance: I32F32::lit("0.81"),
    },
    GarbageLidSpawnChance {
        moon: "rend",
        chance: I32F32::lit("0.65"),
    },
];

pub const GARBAGE_LID_BEHAVIORAL_MECHANICS: [GarbageLidBehaviorRule; 11] = [
    GarbageLidBehaviorRule {
        condition: "carried",
        outcome: "weight is 0",
    },
    GarbageLidBehaviorRule {
        condition: "checked for electrical interaction",
        outcome: "conductive is true",
    },
    GarbageLidBehaviorRule {
        condition: "held",
        outcome: "it requires two hands",
    },
    GarbageLidBehaviorRule {
        condition: "the moon is offense",
        outcome: "the spawn chance is 3.72%",
    },
    GarbageLidBehaviorRule {
        condition: "the moon is assurance",
        outcome: "the spawn chance is 3.64%",
    },
    GarbageLidBehaviorRule {
        condition: "the moon is vow",
        outcome: "the spawn chance is 2.77%",
    },
    GarbageLidBehaviorRule {
        condition: "the moon is march",
        outcome: "the spawn chance is 2.28%",
    },
    GarbageLidBehaviorRule {
        condition: "the moon is adamance",
        outcome: "the spawn chance is 2.01%",
    },
    GarbageLidBehaviorRule {
        condition: "the moon is artifice",
        outcome: "the spawn chance is 0.81%",
    },
    GarbageLidBehaviorRule {
        condition: "the moon is rend",
        outcome: "the spawn chance is 0.65%",
    },
    GarbageLidBehaviorRule {
        condition: "sold to the_company",
        outcome: "it can be exchanged for credits",
    },
];

pub struct GarbageLidPlugin;

impl Plugin for GarbageLidPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnGarbageLidEvent>()
            .add_event::<GarbageLidSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_garbage_lid,
                    garbage_lid_pickup_item_bar_bridge,
                    garbage_lid_sell_bridge,
                    garbage_lid_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GarbageLidBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GarbageLidSpawnChance {
    pub moon: &'static str,
    pub chance: I32F32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GarbageLid {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct GarbageLidScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for GarbageLidScrap {
    fn default() -> Self {
        Self {
            min_value: GARBAGE_LID_MIN_VALUE,
            max_value: GARBAGE_LID_MAX_VALUE,
            weight: GARBAGE_LID_WEIGHT,
            conductive: GARBAGE_LID_CONDUCTIVE,
            two_handed: GARBAGE_LID_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GarbageLidHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Bundle)]
pub struct GarbageLidBundle {
    pub name: Name,
    pub garbage_lid: GarbageLid,
    pub scrap: GarbageLidScrap,
    pub position: SimPosition,
    pub held_by: GarbageLidHeldBy,
}

impl GarbageLidBundle {
    pub fn new(event: SpawnGarbageLidEvent) -> Self {
        Self {
            name: Name::new(GARBAGE_LID_NAME),
            garbage_lid: GarbageLid {
                stable_id: event.stable_id,
            },
            scrap: GarbageLidScrap::default(),
            position: event.position,
            held_by: GarbageLidHeldBy::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnGarbageLidEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GarbageLidSoldEvent {
    pub garbage_lid_stable_id: u64,
    pub credit_value: I32F32,
}

pub fn garbage_lid_value_range() -> (I32F32, I32F32) {
    (GARBAGE_LID_MIN_VALUE, GARBAGE_LID_MAX_VALUE)
}

pub fn garbage_lid_spawn_chance_for_moon(moon: &str) -> Option<I32F32> {
    GARBAGE_LID_SPAWN_CHANCES
        .iter()
        .find(|spawn_chance| spawn_chance.moon == moon)
        .map(|spawn_chance| spawn_chance.chance)
}

fn spawn_garbage_lid(mut commands: Commands, mut events: EventReader<SpawnGarbageLidEvent>) {
    for event in events.read() {
        commands.spawn(GarbageLidBundle::new(*event));
    }
}

fn garbage_lid_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    garbage_lids: Query<(&GarbageLid, &GarbageLidHeldBy), Changed<GarbageLidHeldBy>>,
) {
    for (garbage_lid, held_by) in &garbage_lids {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: GARBAGE_LID_ID,
                two_handed: GARBAGE_LID_TWO_HANDED,
                functional: false,
                passive: false,
                from_store_or_valueless: false,
            });
        } else {
            let _ = garbage_lid.stable_id;
        }
    }
}

fn garbage_lid_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<GarbageLidSoldEvent>,
    garbage_lids: Query<(&GarbageLid, &GarbageLidScrap)>,
) {
    for event in sell_events.read() {
        for (garbage_lid, scrap) in &garbage_lids {
            if garbage_lid.stable_id != event.scrap_entity_id {
                continue;
            }

            sold_events.send(GarbageLidSoldEvent {
                garbage_lid_stable_id: garbage_lid.stable_id,
                credit_value: scrap.max_value,
            });
        }
    }
}

fn garbage_lid_checksum(
    mut checksum: ResMut<SimChecksumState>,
    garbage_lids: Query<(&GarbageLid, &GarbageLidScrap, &SimPosition, &GarbageLidHeldBy)>,
) {
    accumulate_str(&mut checksum, 0x1000, GARBAGE_LID_ID);
    accumulate_str(&mut checksum, 0x1001, GARBAGE_LID_NAME);
    accumulate_str(&mut checksum, 0x1002, GARBAGE_LID_TYPE);
    accumulate_str(&mut checksum, 0x1003, GARBAGE_LID_SUBTYPE);

    checksum.accumulate(GARBAGE_LID_SOURCE_REVISION as u64);
    checksum.accumulate(GARBAGE_LID_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(GARBAGE_LID_WEIGHT.to_bits() as u64);
    checksum.accumulate(GARBAGE_LID_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(GARBAGE_LID_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(GARBAGE_LID_CONDUCTIVE as u64);
    checksum.accumulate(GARBAGE_LID_TWO_HANDED as u64);

    for dependency in GARBAGE_LID_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for spawn_chance in GARBAGE_LID_SPAWN_CHANCES {
        accumulate_str(&mut checksum, 0x4000, spawn_chance.moon);
        checksum.accumulate(spawn_chance.chance.to_bits() as u64);
    }

    for rule in GARBAGE_LID_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x5000, rule.condition);
        accumulate_str(&mut checksum, 0x5001, rule.outcome);
    }

    for (garbage_lid, scrap, position, held_by) in &garbage_lids {
        checksum.accumulate(garbage_lid.stable_id);
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
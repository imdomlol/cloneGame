// Sources: vault/scrap_items/foot.md, vault/item_index_pages/scrap.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::ItemBarPickupEvent;
use crate::sim::{SimChecksumState, SimPosition};

pub const FOOT_ID: &str = "foot";
pub const FOOT_NAME: &str = "Foot";
pub const FOOT_TYPE: &str = "scrap_items";
pub const FOOT_SUBTYPE: &str = "scrap";
pub const FOOT_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Foot";
pub const FOOT_SOURCE_REVISION: u32 = 20323;
pub const FOOT_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const FOOT_CONFIDENCE_BASIS_POINTS: u16 = 93;

pub const FOOT_EFFECTS: &str = "No direct use.";
pub const FOOT_WEIGHT: I32F32 = I32F32::lit("0");
pub const FOOT_CONDUCTIVE: bool = false;
pub const FOOT_TWO_HANDED: bool = false;
pub const FOOT_MIN_VALUE: I32F32 = I32F32::lit("6");
pub const FOOT_MAX_VALUE: I32F32 = I32F32::lit("21");

pub const FOOT_DEPENDS_ON: [&str; 7] = [
    "scrap",
    "lethal_company",
    "the_company",
    "credits",
    "dine",
    "rend",
    "scrap",
];

pub const FOOT_SPAWN_CHANCES: [FootSpawnChance; 2] = [
    FootSpawnChance {
        moon: "dine",
        chance: I32F32::lit("20.24"),
    },
    FootSpawnChance {
        moon: "rend",
        chance: I32F32::lit("0.19"),
    },
];

pub const FOOT_BEHAVIORAL_MECHANICS: [FootBehaviorRule; 5] = [
    FootBehaviorRule {
        condition: "the item is evaluated for carry load",
        outcome: "its weight is 0",
    },
    FootBehaviorRule {
        condition: "the item is checked for conductivity",
        outcome: "it is non-conductive",
    },
    FootBehaviorRule {
        condition: "the item is checked for hand usage",
        outcome: "it is not two-handed",
    },
    FootBehaviorRule {
        condition: "dine",
        outcome: "the spawn chance is 20.24%",
    },
    FootBehaviorRule {
        condition: "rend",
        outcome: "the spawn chance is 0.19%",
    },
];

pub struct FootPlugin;

impl Plugin for FootPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnFootEvent>()
            .add_event::<FootSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_foot,
                    foot_pickup_item_bar_bridge,
                    foot_sell_bridge,
                    foot_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FootBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FootSpawnChance {
    pub moon: &'static str,
    pub chance: I32F32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Foot {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FootScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for FootScrap {
    fn default() -> Self {
        Self {
            min_value: FOOT_MIN_VALUE,
            max_value: FOOT_MAX_VALUE,
            weight: FOOT_WEIGHT,
            conductive: FOOT_CONDUCTIVE,
            two_handed: FOOT_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FootHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Bundle)]
pub struct FootBundle {
    pub name: Name,
    pub foot: Foot,
    pub scrap: FootScrap,
    pub position: SimPosition,
    pub held_by: FootHeldBy,
}

impl FootBundle {
    pub fn new(event: SpawnFootEvent) -> Self {
        Self {
            name: Name::new(FOOT_NAME),
            foot: Foot {
                stable_id: event.stable_id,
            },
            scrap: FootScrap::default(),
            position: event.position,
            held_by: FootHeldBy::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnFootEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FootSoldEvent {
    pub foot_stable_id: u64,
    pub credit_value: I32F32,
}

pub fn foot_value_range() -> (I32F32, I32F32) {
    (FOOT_MIN_VALUE, FOOT_MAX_VALUE)
}

pub fn foot_spawn_chance_for_moon(moon: &str) -> Option<I32F32> {
    FOOT_SPAWN_CHANCES
        .iter()
        .find(|spawn_chance| spawn_chance.moon == moon)
        .map(|spawn_chance| spawn_chance.chance)
}

fn spawn_foot(mut commands: Commands, mut events: EventReader<SpawnFootEvent>) {
    for event in events.read() {
        commands.spawn(FootBundle::new(*event));
    }
}

fn foot_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    feet: Query<(&Foot, &FootHeldBy), Changed<FootHeldBy>>,
) {
    for (foot, held_by) in &feet {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: FOOT_ID,
                two_handed: FOOT_TWO_HANDED,
                functional: false,
                passive: false,
                from_store_or_valueless: false,
            });
        } else {
            let _ = foot.stable_id;
        }
    }
}

fn foot_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<FootSoldEvent>,
    feet: Query<(&Foot, &FootScrap)>,
) {
    for event in sell_events.read() {
        for (foot, scrap) in &feet {
            if foot.stable_id != event.scrap_entity_id {
                continue;
            }

            sold_events.send(FootSoldEvent {
                foot_stable_id: foot.stable_id,
                credit_value: scrap.max_value,
            });
        }
    }
}

fn foot_checksum(
    mut checksum: ResMut<SimChecksumState>,
    feet: Query<(&Foot, &FootScrap, &SimPosition, &FootHeldBy)>,
) {
    accumulate_str(&mut checksum, 0x1000, FOOT_ID);
    accumulate_str(&mut checksum, 0x1001, FOOT_NAME);
    accumulate_str(&mut checksum, 0x1002, FOOT_TYPE);
    accumulate_str(&mut checksum, 0x1003, FOOT_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, FOOT_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1005, FOOT_EXTRACTED_AT);
    accumulate_str(&mut checksum, 0x1006, FOOT_EFFECTS);

    checksum.accumulate(FOOT_SOURCE_REVISION as u64);
    checksum.accumulate(FOOT_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(FOOT_WEIGHT.to_bits() as u64);
    checksum.accumulate(FOOT_CONDUCTIVE as u64);
    checksum.accumulate(FOOT_TWO_HANDED as u64);
    checksum.accumulate(FOOT_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(FOOT_MAX_VALUE.to_bits() as u64);

    for dependency in FOOT_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for spawn_chance in FOOT_SPAWN_CHANCES {
        accumulate_str(&mut checksum, 0x3000, spawn_chance.moon);
        checksum.accumulate(spawn_chance.chance.to_bits() as u64);
    }

    for rule in FOOT_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x4000, rule.condition);
        accumulate_str(&mut checksum, 0x4001, rule.outcome);
    }

    for (foot, scrap, position, held_by) in &feet {
        checksum.accumulate(foot.stable_id);
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
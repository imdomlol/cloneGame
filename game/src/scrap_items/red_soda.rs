// Sources: vault/scrap_items/red_soda.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::{ItemBarPickupEvent};
use crate::sim::{SimChecksumState, SimPosition};

pub const RED_SODA_ID: &str = "red_soda";
pub const RED_SODA_NAME: &str = "Red soda";
pub const RED_SODA_TYPE: &str = "scrap_items";
pub const RED_SODA_SUBTYPE: &str = "scrap";
pub const RED_SODA_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Red_Soda";
pub const RED_SODA_SOURCE_REVISION: u32 = 20332;
pub const RED_SODA_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const RED_SODA_CONFIDENCE_BASIS_POINTS: u16 = 9300;

pub const RED_SODA_EFFECTS: &str = "No direct use; can be sold to the_company for credits.";
pub const RED_SODA_WEIGHT: I32F32 = I32F32::lit("7");
pub const RED_SODA_CONDUCTIVE: bool = true;
pub const RED_SODA_TWO_HANDED: bool = false;
pub const RED_SODA_MIN_VALUE: I32F32 = I32F32::lit("18");
pub const RED_SODA_MAX_VALUE: I32F32 = I32F32::lit("89");

pub const RED_SODA_DEPENDS_ON: [&str; 11] = [
    "scrap",
    "lethal_company",
    "the_company",
    "credits",
    "titan",
    "rend",
    "artifice",
    "assurance",
    "adamance",
    "vow",
    "march",
];

pub const RED_SODA_SPAWN_CHANCES: [RedSodaSpawnChance; 7] = [
    RedSodaSpawnChance {
        moon: "titan",
        chance: I32F32::lit("4.09"),
    },
    RedSodaSpawnChance {
        moon: "rend",
        chance: I32F32::lit("2.41"),
    },
    RedSodaSpawnChance {
        moon: "artifice",
        chance: I32F32::lit("1.48"),
    },
    RedSodaSpawnChance {
        moon: "assurance",
        chance: I32F32::lit("1.41"),
    },
    RedSodaSpawnChance {
        moon: "adamance",
        chance: I32F32::lit("1.27"),
    },
    RedSodaSpawnChance {
        moon: "vow",
        chance: I32F32::lit("1.23"),
    },
    RedSodaSpawnChance {
        moon: "march",
        chance: I32F32::lit("0.23"),
    },
];

pub const RED_SODA_BEHAVIORAL_MECHANICS: [RedSodaBehaviorRule; 7] = [
    RedSodaBehaviorRule {
        condition: "spawn",
        outcome: "the item always lies on its side",
    },
    RedSodaBehaviorRule {
        condition: "sold to the_company",
        outcome: "its value ranges from 18 to 89 credits",
    },
    RedSodaBehaviorRule {
        condition: "carried",
        outcome: "weight is 7",
    },
    RedSodaBehaviorRule {
        condition: "checked for conductivity",
        outcome: "conductive is true",
    },
    RedSodaBehaviorRule {
        condition: "checked for hand use",
        outcome: "two_handed is false",
    },
    RedSodaBehaviorRule {
        condition: "on titan",
        outcome: "spawn chance is 4.09%",
    },
    RedSodaBehaviorRule {
        condition: "on march",
        outcome: "spawn chance is 0.23%",
    },
];

pub struct RedSodaPlugin;

impl Plugin for RedSodaPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnRedSodaEvent>()
            .add_event::<RedSodaSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_red_soda,
                    red_soda_pickup_item_bar_bridge,
                    red_soda_sell_bridge,
                    red_soda_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RedSodaBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RedSodaSpawnChance {
    pub moon: &'static str,
    pub chance: I32F32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RedSoda {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct RedSodaScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for RedSodaScrap {
    fn default() -> Self {
        Self {
            min_value: RED_SODA_MIN_VALUE,
            max_value: RED_SODA_MAX_VALUE,
            weight: RED_SODA_WEIGHT,
            conductive: RED_SODA_CONDUCTIVE,
            two_handed: RED_SODA_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RedSodaHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Bundle)]
pub struct RedSodaBundle {
    pub name: Name,
    pub red_soda: RedSoda,
    pub scrap: RedSodaScrap,
    pub position: SimPosition,
    pub held_by: RedSodaHeldBy,
}

impl RedSodaBundle {
    pub fn new(event: SpawnRedSodaEvent) -> Self {
        Self {
            name: Name::new(RED_SODA_NAME),
            red_soda: RedSoda {
                stable_id: event.stable_id,
            },
            scrap: RedSodaScrap::default(),
            position: event.position,
            held_by: RedSodaHeldBy::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnRedSodaEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RedSodaSoldEvent {
    pub red_soda_stable_id: u64,
    pub credit_value: I32F32,
}

pub fn red_soda_value_range() -> (I32F32, I32F32) {
    (RED_SODA_MIN_VALUE, RED_SODA_MAX_VALUE)
}

pub fn red_soda_spawn_chance_for_moon(moon: &str) -> Option<I32F32> {
    RED_SODA_SPAWN_CHANCES
        .iter()
        .find(|sc| sc.moon == moon)
        .map(|sc| sc.chance)
}

fn spawn_red_soda(mut commands: Commands, mut events: EventReader<SpawnRedSodaEvent>) {
    for event in events.read() {
        commands.spawn(RedSodaBundle::new(*event));
    }
}

fn red_soda_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    items: Query<(&RedSoda, &RedSodaHeldBy), Changed<RedSodaHeldBy>>,
) {
    for (red_soda, held_by) in &items {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: RED_SODA_ID,
                two_handed: RED_SODA_TWO_HANDED,
                functional: false,
                passive: false,
                from_store_or_valueless: false,
            });
        } else {
            let _ = red_soda.stable_id;
        }
    }
}

fn red_soda_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<RedSodaSoldEvent>,
    items: Query<(&RedSoda, &RedSodaScrap)>,
) {
    for event in sell_events.read() {
        for (red_soda, scrap) in &items {
            if red_soda.stable_id != event.scrap_entity_id {
                continue;
            }
            sold_events.send(RedSodaSoldEvent {
                red_soda_stable_id: red_soda.stable_id,
                credit_value: scrap.max_value,
            });
        }
    }
}

fn red_soda_checksum(
    mut checksum: ResMut<SimChecksumState>,
    items: Query<(&RedSoda, &RedSodaScrap, &SimPosition, &RedSodaHeldBy)>,
) {
    accumulate_str(&mut checksum, 0x1000, RED_SODA_ID);
    accumulate_str(&mut checksum, 0x1001, RED_SODA_NAME);
    accumulate_str(&mut checksum, 0x1002, RED_SODA_TYPE);
    accumulate_str(&mut checksum, 0x1003, RED_SODA_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, RED_SODA_EFFECTS);

    checksum.accumulate(RED_SODA_SOURCE_REVISION as u64);
    checksum.accumulate(RED_SODA_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(RED_SODA_WEIGHT.to_bits() as u64);
    checksum.accumulate(RED_SODA_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(RED_SODA_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(RED_SODA_CONDUCTIVE as u64);
    checksum.accumulate(RED_SODA_TWO_HANDED as u64);

    for dependency in RED_SODA_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for spawn_chance in RED_SODA_SPAWN_CHANCES {
        accumulate_str(&mut checksum, 0x4000, spawn_chance.moon);
        checksum.accumulate(spawn_chance.chance.to_bits() as u64);
    }

    for rule in RED_SODA_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x5000, rule.condition);
        accumulate_str(&mut checksum, 0x5001, rule.outcome);
    }

    for (red_soda, scrap, position, held_by) in &items {
        checksum.accumulate(red_soda.stable_id);
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
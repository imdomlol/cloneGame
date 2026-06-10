// Sources: vault/scrap_items/egg_beater.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::ItemBarPickupEvent;
use crate::sim::{SimChecksumState, SimPosition};

pub const EGG_BEATER_ID: &str = "egg_beater";
pub const EGG_BEATER_NAME: &str = "Egg beater";
pub const EGG_BEATER_TYPE: &str = "scrap_items";
pub const EGG_BEATER_SUBTYPE: &str = "scrap";
pub const EGG_BEATER_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Egg_Beater";
pub const EGG_BEATER_SOURCE_REVISION: u32 = 20209;
pub const EGG_BEATER_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const EGG_BEATER_CONFIDENCE_BASIS_POINTS: u16 = 88;

pub const EGG_BEATER_WEIGHT: I32F32 = I32F32::lit("11");
pub const EGG_BEATER_CONDUCTIVE: bool = true;
pub const EGG_BEATER_TWO_HANDED: bool = false;
pub const EGG_BEATER_MIN_VALUE: I32F32 = I32F32::lit("12");
pub const EGG_BEATER_MAX_VALUE: I32F32 = I32F32::lit("43");

pub const EGG_BEATER_DEPENDS_ON: [&str; 7] = [
    "scrap",
    "the_company",
    "credits",
    "vow",
    "adamance",
    "assurance",
    "experimentation",
];

pub const EGG_BEATER_SPAWN_CHANCES: [EggBeaterSpawnChance; 4] = [
    EggBeaterSpawnChance {
        moon: "vow",
        chance: I32F32::lit("5.76"),
    },
    EggBeaterSpawnChance {
        moon: "adamance",
        chance: I32F32::lit("5.28"),
    },
    EggBeaterSpawnChance {
        moon: "assurance",
        chance: I32F32::lit("4"),
    },
    EggBeaterSpawnChance {
        moon: "experimentation",
        chance: I32F32::lit("1.76"),
    },
];

pub const EGG_BEATER_BEHAVIORAL_MECHANICS: [EggBeaterBehaviorRule; 5] = [
    EggBeaterBehaviorRule {
        condition: "it is sold to the_company",
        outcome: "its only stated use is being exchanged for credits",
    },
    EggBeaterBehaviorRule {
        condition: "vow",
        outcome: "spawn chance is 5.76%",
    },
    EggBeaterBehaviorRule {
        condition: "adamance",
        outcome: "spawn chance is 5.28%",
    },
    EggBeaterBehaviorRule {
        condition: "assurance",
        outcome: "spawn chance is 4%",
    },
    EggBeaterBehaviorRule {
        condition: "experimentation",
        outcome: "spawn chance is 1.76%",
    },
];

pub struct EggBeaterPlugin;

impl Plugin for EggBeaterPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnEggBeaterEvent>()
            .add_event::<EggBeaterSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_egg_beater,
                    egg_beater_pickup_item_bar_bridge,
                    egg_beater_sell_bridge,
                    egg_beater_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EggBeaterBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EggBeaterSpawnChance {
    pub moon: &'static str,
    pub chance: I32F32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EggBeater {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct EggBeaterScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for EggBeaterScrap {
    fn default() -> Self {
        Self {
            min_value: EGG_BEATER_MIN_VALUE,
            max_value: EGG_BEATER_MAX_VALUE,
            weight: EGG_BEATER_WEIGHT,
            conductive: EGG_BEATER_CONDUCTIVE,
            two_handed: EGG_BEATER_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EggBeaterHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Bundle)]
pub struct EggBeaterBundle {
    pub name: Name,
    pub egg_beater: EggBeater,
    pub scrap: EggBeaterScrap,
    pub position: SimPosition,
    pub held_by: EggBeaterHeldBy,
}

impl EggBeaterBundle {
    pub fn new(event: SpawnEggBeaterEvent) -> Self {
        Self {
            name: Name::new(EGG_BEATER_NAME),
            egg_beater: EggBeater {
                stable_id: event.stable_id,
            },
            scrap: EggBeaterScrap::default(),
            position: event.position,
            held_by: EggBeaterHeldBy::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnEggBeaterEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EggBeaterSoldEvent {
    pub egg_beater_stable_id: u64,
    pub credit_value: I32F32,
}

pub fn egg_beater_value_range() -> (I32F32, I32F32) {
    (EGG_BEATER_MIN_VALUE, EGG_BEATER_MAX_VALUE)
}

pub fn egg_beater_spawn_chance_for_moon(moon: &str) -> Option<I32F32> {
    EGG_BEATER_SPAWN_CHANCES
        .iter()
        .find(|spawn_chance| spawn_chance.moon == moon)
        .map(|spawn_chance| spawn_chance.chance)
}

fn spawn_egg_beater(mut commands: Commands, mut events: EventReader<SpawnEggBeaterEvent>) {
    for event in events.read() {
        commands.spawn(EggBeaterBundle::new(*event));
    }
}

fn egg_beater_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    egg_beaters: Query<(&EggBeater, &EggBeaterHeldBy), Changed<EggBeaterHeldBy>>,
) {
    for (egg_beater, held_by) in &egg_beaters {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: EGG_BEATER_ID,
                two_handed: EGG_BEATER_TWO_HANDED,
                functional: false,
                passive: false,
                from_store_or_valueless: false,
            });
        } else {
            let _ = egg_beater.stable_id;
        }
    }
}

fn egg_beater_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<EggBeaterSoldEvent>,
    egg_beaters: Query<(&EggBeater, &EggBeaterScrap)>,
) {
    for event in sell_events.read() {
        for (egg_beater, scrap) in &egg_beaters {
            if egg_beater.stable_id != event.scrap_entity_id {
                continue;
            }

            sold_events.send(EggBeaterSoldEvent {
                egg_beater_stable_id: egg_beater.stable_id,
                credit_value: scrap.max_value,
            });
        }
    }
}

fn egg_beater_checksum(
    mut checksum: ResMut<SimChecksumState>,
    egg_beaters: Query<(&EggBeater, &EggBeaterScrap, &SimPosition, &EggBeaterHeldBy)>,
) {
    accumulate_str(&mut checksum, 0x1000, EGG_BEATER_ID);
    accumulate_str(&mut checksum, 0x1001, EGG_BEATER_NAME);
    accumulate_str(&mut checksum, 0x1002, EGG_BEATER_TYPE);
    accumulate_str(&mut checksum, 0x1003, EGG_BEATER_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, EGG_BEATER_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1005, EGG_BEATER_EXTRACTED_AT);

    checksum.accumulate(EGG_BEATER_SOURCE_REVISION as u64);
    checksum.accumulate(EGG_BEATER_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(EGG_BEATER_WEIGHT.to_bits() as u64);
    checksum.accumulate(EGG_BEATER_CONDUCTIVE as u64);
    checksum.accumulate(EGG_BEATER_TWO_HANDED as u64);
    checksum.accumulate(EGG_BEATER_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(EGG_BEATER_MAX_VALUE.to_bits() as u64);

    for dependency in EGG_BEATER_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for spawn_chance in EGG_BEATER_SPAWN_CHANCES {
        accumulate_str(&mut checksum, 0x3000, spawn_chance.moon);
        checksum.accumulate(spawn_chance.chance.to_bits() as u64);
    }

    for rule in EGG_BEATER_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x4000, rule.condition);
        accumulate_str(&mut checksum, 0x4001, rule.outcome);
    }

    for (egg_beater, scrap, position, held_by) in &egg_beaters {
        checksum.accumulate(egg_beater.stable_id);
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
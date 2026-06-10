// Sources: vault/scrap_items/knee.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::ItemBarPickupEvent;
use crate::sim::{SimChecksumState, SimPosition};

pub const KNEE_ID: &str = "knee";
pub const KNEE_NAME: &str = "Knee";
pub const KNEE_TYPE: &str = "scrap_items";
pub const KNEE_SUBTYPE: &str = "scrap";
pub const KNEE_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Knee";
pub const KNEE_SOURCE_REVISION: u32 = 20326;
pub const KNEE_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const KNEE_CONFIDENCE_BASIS_POINTS: u16 = 95;

pub const KNEE_EFFECTS: &str = "No special use; can be sold for credits.";
pub const KNEE_WEIGHT: I32F32 = I32F32::lit("5");
pub const KNEE_CONDUCTIVE: bool = false;
pub const KNEE_TWO_HANDED: bool = false;
pub const KNEE_MIN_VALUE: I32F32 = I32F32::lit("8");
pub const KNEE_MAX_VALUE: I32F32 = I32F32::lit("17");

pub const KNEE_DEPENDS_ON: [&str; 5] = ["scrap", "lethal_company", "dine", "the_company", "credits"];

pub const KNEE_SPAWN_CHANCES: [KneeSpawnChance; 1] = [KneeSpawnChance {
    moon: "dine",
    chance: I32F32::lit("11.13"),
}];

pub const KNEE_BEHAVIORAL_MECHANICS: [KneeBehaviorRule; 6] = [
    KneeBehaviorRule {
        condition: "the item is collected as scrap",
        outcome: "its weight is 5",
    },
    KneeBehaviorRule {
        condition: "the item is checked for electrical properties",
        outcome: "it is non-conductive",
    },
    KneeBehaviorRule {
        condition: "the item is checked for handling",
        outcome: "it is not two-handed",
    },
    KneeBehaviorRule {
        condition: "the item is sold to the_company",
        outcome: "it is converted into credits",
    },
    KneeBehaviorRule {
        condition: "the item is found on a moon",
        outcome: "the spawn chance on dine is 11.13%",
    },
    KneeBehaviorRule {
        condition: "the item's value is rolled",
        outcome: "the sell range is 8 to 17",
    },
];

pub struct KneePlugin;

impl Plugin for KneePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnKneeEvent>()
            .add_event::<KneeSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_knee,
                    knee_pickup_item_bar_bridge,
                    knee_sell_bridge,
                    knee_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KneeBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KneeSpawnChance {
    pub moon: &'static str,
    pub chance: I32F32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Knee {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct KneeScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for KneeScrap {
    fn default() -> Self {
        Self {
            min_value: KNEE_MIN_VALUE,
            max_value: KNEE_MAX_VALUE,
            weight: KNEE_WEIGHT,
            conductive: KNEE_CONDUCTIVE,
            two_handed: KNEE_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct KneeHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Bundle)]
pub struct KneeBundle {
    pub name: Name,
    pub knee: Knee,
    pub scrap: KneeScrap,
    pub position: SimPosition,
    pub held_by: KneeHeldBy,
}

impl KneeBundle {
    pub fn new(event: SpawnKneeEvent) -> Self {
        Self {
            name: Name::new(KNEE_NAME),
            knee: Knee {
                stable_id: event.stable_id,
            },
            scrap: KneeScrap::default(),
            position: event.position,
            held_by: KneeHeldBy::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnKneeEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct KneeSoldEvent {
    pub knee_stable_id: u64,
    pub credit_value: I32F32,
}

pub fn knee_value_range() -> (I32F32, I32F32) {
    (KNEE_MIN_VALUE, KNEE_MAX_VALUE)
}

pub fn knee_spawn_chance_for_moon(moon: &str) -> Option<I32F32> {
    KNEE_SPAWN_CHANCES
        .iter()
        .find(|spawn_chance| spawn_chance.moon == moon)
        .map(|spawn_chance| spawn_chance.chance)
}

fn spawn_knee(mut commands: Commands, mut events: EventReader<SpawnKneeEvent>) {
    for event in events.read() {
        commands.spawn(KneeBundle::new(*event));
    }
}

fn knee_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    knees: Query<(&Knee, &KneeHeldBy), Changed<KneeHeldBy>>,
) {
    for (knee, held_by) in &knees {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: KNEE_ID,
                two_handed: KNEE_TWO_HANDED,
                functional: false,
                passive: false,
                from_store_or_valueless: false,
            });
        } else {
            let _ = knee.stable_id;
        }
    }
}

fn knee_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<KneeSoldEvent>,
    knees: Query<(&Knee, &KneeScrap)>,
) {
    for event in sell_events.read() {
        for (knee, scrap) in &knees {
            if knee.stable_id != event.scrap_entity_id {
                continue;
            }

            sold_events.send(KneeSoldEvent {
                knee_stable_id: knee.stable_id,
                credit_value: scrap.max_value,
            });
        }
    }
}

fn knee_checksum(
    mut checksum: ResMut<SimChecksumState>,
    knees: Query<(&Knee, &KneeScrap, &SimPosition, &KneeHeldBy)>,
) {
    accumulate_str(&mut checksum, 0x1000, KNEE_ID);
    accumulate_str(&mut checksum, 0x1001, KNEE_NAME);
    accumulate_str(&mut checksum, 0x1002, KNEE_TYPE);
    accumulate_str(&mut checksum, 0x1003, KNEE_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, KNEE_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1005, KNEE_EXTRACTED_AT);
    accumulate_str(&mut checksum, 0x1006, KNEE_EFFECTS);

    checksum.accumulate(KNEE_SOURCE_REVISION as u64);
    checksum.accumulate(KNEE_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(KNEE_WEIGHT.to_bits() as u64);
    checksum.accumulate(KNEE_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(KNEE_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(KNEE_CONDUCTIVE as u64);
    checksum.accumulate(KNEE_TWO_HANDED as u64);

    for dependency in KNEE_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for spawn_chance in KNEE_SPAWN_CHANCES {
        accumulate_str(&mut checksum, 0x3000, spawn_chance.moon);
        checksum.accumulate(spawn_chance.chance.to_bits() as u64);
    }

    for rule in KNEE_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x4000, rule.condition);
        accumulate_str(&mut checksum, 0x4001, rule.outcome);
    }

    for (knee, scrap, position, held_by) in &knees {
        checksum.accumulate(knee.stable_id);
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
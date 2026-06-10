// Sources: vault/scrap_items/hand.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::ItemBarPickupEvent;
use crate::sim::{SimChecksumState, SimPosition};

pub const HAND_ID: &str = "hand";
pub const HAND_NAME: &str = "Hand";
pub const HAND_TYPE: &str = "scrap_items";
pub const HAND_SUBTYPE: &str = "scrap";
pub const HAND_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Hand";
pub const HAND_SOURCE_REVISION: u32 = 20324;
pub const HAND_EXTRACTED_AT: &str = "2026-06-07";
pub const HAND_CONFIDENCE_BASIS_POINTS: u16 = 93;

pub const HAND_WEIGHT: I32F32 = I32F32::lit("0");
pub const HAND_CONDUCTIVE: bool = false;
pub const HAND_TWO_HANDED: bool = false;
pub const HAND_MIN_VALUE: I32F32 = I32F32::lit("4");
pub const HAND_MAX_VALUE: I32F32 = I32F32::lit("11");

pub const HAND_DEPENDS_ON: [&str; 3] = ["scrap", "dine", "the_company"];

pub const HAND_SPAWN_CHANCES: [HandSpawnChance; 1] = [HandSpawnChance {
    moon: "dine",
    chance: I32F32::lit("20.24"),
}];

pub const HAND_BEHAVIORAL_MECHANICS: [HandBehaviorRule; 5] = [
    HandBehaviorRule {
        condition: "the item is sold to the_company",
        outcome: "it can be converted into credits",
    },
    HandBehaviorRule {
        condition: "the item is spawned on dine",
        outcome: "its reported spawn chance is 20.24%",
    },
    HandBehaviorRule {
        condition: "the item is evaluated for carry handling",
        outcome: "it has a weight of 0 and is not two-handed",
    },
    HandBehaviorRule {
        condition: "the item is evaluated for electrical interaction",
        outcome: "it is not conductive",
    },
    HandBehaviorRule {
        condition: "the item is evaluated for value generation",
        outcome: "its sell range is 4 to 11",
    },
];

pub struct HandPlugin;

impl Plugin for HandPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnHandEvent>()
            .add_event::<HandSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_hand,
                    hand_pickup_item_bar_bridge,
                    hand_sell_bridge,
                    hand_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HandBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HandSpawnChance {
    pub moon: &'static str,
    pub chance: I32F32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Hand {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HandScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for HandScrap {
    fn default() -> Self {
        Self {
            min_value: HAND_MIN_VALUE,
            max_value: HAND_MAX_VALUE,
            weight: HAND_WEIGHT,
            conductive: HAND_CONDUCTIVE,
            two_handed: HAND_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HandHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Bundle)]
pub struct HandBundle {
    pub name: Name,
    pub hand: Hand,
    pub scrap: HandScrap,
    pub position: SimPosition,
    pub held_by: HandHeldBy,
}

impl HandBundle {
    pub fn new(event: SpawnHandEvent) -> Self {
        Self {
            name: Name::new(HAND_NAME),
            hand: Hand {
                stable_id: event.stable_id,
            },
            scrap: HandScrap::default(),
            position: event.position,
            held_by: HandHeldBy::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnHandEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandSoldEvent {
    pub hand_stable_id: u64,
    pub credit_value: I32F32,
}

pub fn hand_value_range() -> (I32F32, I32F32) {
    (HAND_MIN_VALUE, HAND_MAX_VALUE)
}

pub fn hand_spawn_chance_for_moon(moon: &str) -> Option<I32F32> {
    HAND_SPAWN_CHANCES
        .iter()
        .find(|spawn_chance| spawn_chance.moon == moon)
        .map(|spawn_chance| spawn_chance.chance)
}

fn spawn_hand(mut commands: Commands, mut events: EventReader<SpawnHandEvent>) {
    for event in events.read() {
        commands.spawn(HandBundle::new(*event));
    }
}

fn hand_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    hands: Query<(&Hand, &HandHeldBy), Changed<HandHeldBy>>,
) {
    for (hand, held_by) in &hands {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: HAND_ID,
                two_handed: HAND_TWO_HANDED,
                functional: false,
                passive: true,
                from_store_or_valueless: false,
            });
        } else {
            let _ = hand.stable_id;
        }
    }
}

fn hand_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<HandSoldEvent>,
    hands: Query<(&Hand, &HandScrap)>,
) {
    for event in sell_events.read() {
        for (hand, scrap) in &hands {
            if hand.stable_id != event.scrap_entity_id {
                continue;
            }

            sold_events.send(HandSoldEvent {
                hand_stable_id: hand.stable_id,
                credit_value: scrap.max_value,
            });
        }
    }
}

fn hand_checksum(
    mut checksum: ResMut<SimChecksumState>,
    hands: Query<(&Hand, &HandScrap, &SimPosition, &HandHeldBy)>,
) {
    accumulate_str(&mut checksum, 0x1000, HAND_ID);
    accumulate_str(&mut checksum, 0x1001, HAND_NAME);
    accumulate_str(&mut checksum, 0x1002, HAND_TYPE);
    accumulate_str(&mut checksum, 0x1003, HAND_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, HAND_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1005, HAND_EXTRACTED_AT);

    checksum.accumulate(HAND_SOURCE_REVISION as u64);
    checksum.accumulate(HAND_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(HAND_WEIGHT.to_bits() as u64);
    checksum.accumulate(HAND_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(HAND_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(HAND_CONDUCTIVE as u64);
    checksum.accumulate(HAND_TWO_HANDED as u64);

    for dependency in HAND_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for spawn_chance in HAND_SPAWN_CHANCES {
        accumulate_str(&mut checksum, 0x3000, spawn_chance.moon);
        checksum.accumulate(spawn_chance.chance.to_bits() as u64);
    }

    for rule in HAND_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x4000, rule.condition);
        accumulate_str(&mut checksum, 0x4001, rule.outcome);
    }

    for (hand, scrap, position, held_by) in &hands {
        checksum.accumulate(hand.stable_id);
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
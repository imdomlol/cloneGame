// Sources: vault/scrap_items/golden_cup.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::ItemBarPickupEvent;
use crate::sim::{SimChecksumState, SimPosition};

pub const GOLDEN_CUP_ID: &str = "golden_cup";
pub const GOLDEN_CUP_NAME: &str = "Golden cup";
pub const GOLDEN_CUP_TYPE: &str = "scrap_items";
pub const GOLDEN_CUP_SUBTYPE: &str = "scrap";
pub const GOLDEN_CUP_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Golden_Cup";
pub const GOLDEN_CUP_SOURCE_REVISION: u32 = 20224;
pub const GOLDEN_CUP_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const GOLDEN_CUP_CONFIDENCE_BASIS_POINTS: u16 = 94;

pub const GOLDEN_CUP_WEIGHT: I32F32 = I32F32::lit("16");
pub const GOLDEN_CUP_CONDUCTIVE: bool = false;
pub const GOLDEN_CUP_MIN_VALUE: I32F32 = I32F32::lit("40");
pub const GOLDEN_CUP_MAX_VALUE: I32F32 = I32F32::lit("79");
pub const GOLDEN_CUP_TWO_HANDED: bool = false;

pub const GOLDEN_CUP_DEPENDS_ON: [&str; 3] = ["lethal_company", "the_company", "credits"];

pub const GOLDEN_CUP_BEHAVIORAL_MECHANICS: [GoldenCupBehaviorRule; 4] = [
    GoldenCupBehaviorRule {
        condition: "sold to the_company",
        outcome: "its value is between 40 and 79 credits",
    },
    GoldenCupBehaviorRule {
        condition: "carried",
        outcome: "its weight is 16",
    },
    GoldenCupBehaviorRule {
        condition: "evaluated for properties",
        outcome: "it is not conductive",
    },
    GoldenCupBehaviorRule {
        condition: "evaluated for handling",
        outcome: "it is not two-handed",
    },
];

pub struct GoldenCupPlugin;

impl Plugin for GoldenCupPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnGoldenCupEvent>()
            .add_event::<GoldenCupSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_golden_cup,
                    golden_cup_pickup_item_bar_bridge,
                    golden_cup_sell_bridge,
                    golden_cup_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GoldenCupBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GoldenCup {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct GoldenCupScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for GoldenCupScrap {
    fn default() -> Self {
        Self {
            min_value: GOLDEN_CUP_MIN_VALUE,
            max_value: GOLDEN_CUP_MAX_VALUE,
            weight: GOLDEN_CUP_WEIGHT,
            conductive: GOLDEN_CUP_CONDUCTIVE,
            two_handed: GOLDEN_CUP_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GoldenCupHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Bundle)]
pub struct GoldenCupBundle {
    pub name: Name,
    pub golden_cup: GoldenCup,
    pub scrap: GoldenCupScrap,
    pub position: SimPosition,
    pub held_by: GoldenCupHeldBy,
}

impl GoldenCupBundle {
    pub fn new(event: SpawnGoldenCupEvent) -> Self {
        Self {
            name: Name::new(GOLDEN_CUP_NAME),
            golden_cup: GoldenCup {
                stable_id: event.stable_id,
            },
            scrap: GoldenCupScrap::default(),
            position: event.position,
            held_by: GoldenCupHeldBy::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnGoldenCupEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GoldenCupSoldEvent {
    pub golden_cup_stable_id: u64,
    pub credit_value: I32F32,
}

pub fn golden_cup_value_range() -> (I32F32, I32F32) {
    (GOLDEN_CUP_MIN_VALUE, GOLDEN_CUP_MAX_VALUE)
}

fn spawn_golden_cup(mut commands: Commands, mut events: EventReader<SpawnGoldenCupEvent>) {
    for event in events.read() {
        commands.spawn(GoldenCupBundle::new(*event));
    }
}

fn golden_cup_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    golden_cups: Query<(&GoldenCup, &GoldenCupHeldBy), Changed<GoldenCupHeldBy>>,
) {
    for (golden_cup, held_by) in &golden_cups {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: GOLDEN_CUP_ID,
                two_handed: GOLDEN_CUP_TWO_HANDED,
                functional: false,
                passive: false,
                from_store_or_valueless: false,
            });
        } else {
            let _ = golden_cup.stable_id;
        }
    }
}

fn golden_cup_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<GoldenCupSoldEvent>,
    golden_cups: Query<(&GoldenCup, &GoldenCupScrap)>,
) {
    for event in sell_events.read() {
        for (golden_cup, scrap) in &golden_cups {
            if golden_cup.stable_id != event.scrap_entity_id {
                continue;
            }

            sold_events.send(GoldenCupSoldEvent {
                golden_cup_stable_id: golden_cup.stable_id,
                credit_value: scrap.max_value,
            });
        }
    }
}

fn golden_cup_checksum(
    mut checksum: ResMut<SimChecksumState>,
    golden_cups: Query<(&GoldenCup, &GoldenCupScrap, &SimPosition, &GoldenCupHeldBy)>,
) {
    accumulate_str(&mut checksum, 0x1000, GOLDEN_CUP_ID);
    accumulate_str(&mut checksum, 0x1001, GOLDEN_CUP_NAME);
    accumulate_str(&mut checksum, 0x1002, GOLDEN_CUP_TYPE);
    accumulate_str(&mut checksum, 0x1003, GOLDEN_CUP_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, GOLDEN_CUP_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1005, GOLDEN_CUP_EXTRACTED_AT);

    checksum.accumulate(GOLDEN_CUP_SOURCE_REVISION as u64);
    checksum.accumulate(GOLDEN_CUP_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(GOLDEN_CUP_WEIGHT.to_bits() as u64);
    checksum.accumulate(GOLDEN_CUP_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(GOLDEN_CUP_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(GOLDEN_CUP_CONDUCTIVE as u64);
    checksum.accumulate(GOLDEN_CUP_TWO_HANDED as u64);

    for dependency in GOLDEN_CUP_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for rule in GOLDEN_CUP_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x3000, rule.condition);
        accumulate_str(&mut checksum, 0x3001, rule.outcome);
    }

    for (golden_cup, scrap, position, held_by) in &golden_cups {
        checksum.accumulate(golden_cup.stable_id);
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
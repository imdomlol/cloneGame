// Sources: vault/scrap_items/toy_cube.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::ItemBarPickupEvent;
use crate::sim::{SimChecksumState, SimPosition};

pub const TOY_CUBE_ID: &str = "toy_cube";
pub const TOY_CUBE_NAME: &str = "Toy cube";
pub const TOY_CUBE_TYPE: &str = "scrap_items";
pub const TOY_CUBE_SUBTYPE: &str = "scrap";
pub const TOY_CUBE_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Toy_Cube";
pub const TOY_CUBE_SOURCE_REVISION: u32 = 20429;
pub const TOY_CUBE_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const TOY_CUBE_CONFIDENCE_BASIS_POINTS: u16 = 96;

pub const TOY_CUBE_EFFECTS: &str =
    "No functional use; can be sold to the_company for credits.";
pub const TOY_CUBE_WEIGHT: I32F32 = I32F32::lit("0");
pub const TOY_CUBE_CONDUCTIVE: bool = false;
pub const TOY_CUBE_TWO_HANDED: bool = false;
pub const TOY_CUBE_MIN_VALUE: I32F32 = I32F32::lit("24");
pub const TOY_CUBE_MAX_VALUE: I32F32 = I32F32::lit("43");

pub const TOY_CUBE_DEPENDS_ON: [&str; 3] = ["lethal_company", "the_company", "credits"];

pub const TOY_CUBE_BEHAVIORAL_MECHANICS: [ToyCubeBehaviorRule; 4] = [
    ToyCubeBehaviorRule {
        condition: "the item is carried",
        outcome: "its weight is 0",
    },
    ToyCubeBehaviorRule {
        condition: "the item is evaluated for conductivity",
        outcome: "conductive is false",
    },
    ToyCubeBehaviorRule {
        condition: "the item is handled",
        outcome: "two_handed is false",
    },
    ToyCubeBehaviorRule {
        condition: "the item is sold to the_company",
        outcome: "its value is between 24 and 43 credits",
    },
];

pub struct ToyCubePlugin;

impl Plugin for ToyCubePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnToyCubeEvent>()
            .add_event::<ToyCubeSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_toy_cube,
                    toy_cube_pickup_item_bar_bridge,
                    toy_cube_sell_bridge,
                    toy_cube_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToyCubeBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ToyCube {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToyCubeScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for ToyCubeScrap {
    fn default() -> Self {
        Self {
            min_value: TOY_CUBE_MIN_VALUE,
            max_value: TOY_CUBE_MAX_VALUE,
            weight: TOY_CUBE_WEIGHT,
            conductive: TOY_CUBE_CONDUCTIVE,
            two_handed: TOY_CUBE_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ToyCubeHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Bundle)]
pub struct ToyCubeBundle {
    pub name: Name,
    pub toy_cube: ToyCube,
    pub scrap: ToyCubeScrap,
    pub position: SimPosition,
    pub held_by: ToyCubeHeldBy,
}

impl ToyCubeBundle {
    pub fn new(event: SpawnToyCubeEvent) -> Self {
        Self {
            name: Name::new(TOY_CUBE_NAME),
            toy_cube: ToyCube {
                stable_id: event.stable_id,
            },
            scrap: ToyCubeScrap::default(),
            position: event.position,
            held_by: ToyCubeHeldBy::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnToyCubeEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToyCubeSoldEvent {
    pub toy_cube_stable_id: u64,
    pub credit_value: I32F32,
}

pub fn toy_cube_value_range() -> (I32F32, I32F32) {
    (TOY_CUBE_MIN_VALUE, TOY_CUBE_MAX_VALUE)
}

fn spawn_toy_cube(mut commands: Commands, mut events: EventReader<SpawnToyCubeEvent>) {
    for event in events.read() {
        commands.spawn(ToyCubeBundle::new(*event));
    }
}

fn toy_cube_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    toy_cubes: Query<(&ToyCube, &ToyCubeHeldBy), Changed<ToyCubeHeldBy>>,
) {
    for (toy_cube, held_by) in &toy_cubes {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: TOY_CUBE_ID,
                two_handed: TOY_CUBE_TWO_HANDED,
                functional: false,
                passive: false,
                from_store_or_valueless: false,
            });
        } else {
            let _ = toy_cube.stable_id;
        }
    }
}

fn toy_cube_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<ToyCubeSoldEvent>,
    toy_cubes: Query<(&ToyCube, &ToyCubeScrap)>,
) {
    for event in sell_events.read() {
        for (toy_cube, scrap) in &toy_cubes {
            if toy_cube.stable_id != event.scrap_entity_id {
                continue;
            }

            sold_events.send(ToyCubeSoldEvent {
                toy_cube_stable_id: toy_cube.stable_id,
                credit_value: scrap.max_value,
            });
        }
    }
}

fn toy_cube_checksum(
    mut checksum: ResMut<SimChecksumState>,
    toy_cubes: Query<(&ToyCube, &ToyCubeScrap, &SimPosition, &ToyCubeHeldBy)>,
) {
    accumulate_str(&mut checksum, 0x1000, TOY_CUBE_ID);
    accumulate_str(&mut checksum, 0x1001, TOY_CUBE_NAME);
    accumulate_str(&mut checksum, 0x1002, TOY_CUBE_TYPE);
    accumulate_str(&mut checksum, 0x1003, TOY_CUBE_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, TOY_CUBE_EFFECTS);

    checksum.accumulate(TOY_CUBE_SOURCE_REVISION as u64);
    checksum.accumulate(TOY_CUBE_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(TOY_CUBE_WEIGHT.to_bits() as u64);
    checksum.accumulate(TOY_CUBE_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(TOY_CUBE_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(TOY_CUBE_CONDUCTIVE as u64);
    checksum.accumulate(TOY_CUBE_TWO_HANDED as u64);

    for dependency in TOY_CUBE_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for rule in TOY_CUBE_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x5000, rule.condition);
        accumulate_str(&mut checksum, 0x5001, rule.outcome);
    }

    for (toy_cube, scrap, position, held_by) in &toy_cubes {
        checksum.accumulate(toy_cube.stable_id);
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
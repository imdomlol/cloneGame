// Sources: vault/scrap_items/steering_wheel.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::ItemBarPickupEvent;
use crate::sim::{SimChecksumState, SimPosition};

pub const STEERING_WHEEL_ID: &str = "steering_wheel";
pub const STEERING_WHEEL_NAME: &str = "Steering wheel";
pub const STEERING_WHEEL_TYPE: &str = "scrap_items";
pub const STEERING_WHEEL_SUBTYPE: &str = "scrap";
pub const STEERING_WHEEL_SOURCE_URL: &str =
    "https://lethal-company.fandom.com/wiki/Steering_Wheel";
pub const STEERING_WHEEL_SOURCE_REVISION: u32 = 20339;
pub const STEERING_WHEEL_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const STEERING_WHEEL_CONFIDENCE_BASIS_POINTS: u16 = 96;

pub const STEERING_WHEEL_EFFECTS: &str = "No known functional use beyond sale.";
pub const STEERING_WHEEL_WEIGHT: I32F32 = I32F32::lit("16");
pub const STEERING_WHEEL_CONDUCTIVE: bool = false;
pub const STEERING_WHEEL_MIN_VALUE: I32F32 = I32F32::lit("16");
pub const STEERING_WHEEL_MAX_VALUE: I32F32 = I32F32::lit("31");
pub const STEERING_WHEEL_TWO_HANDED: bool = false;

pub const STEERING_WHEEL_DEPENDS_ON: [&str; 4] =
    ["experimentation", "assurance", "vow", "adamance"];

pub const STEERING_WHEEL_SPAWN_CHANCES: [SteeringWheelSpawnChance; 4] = [
    SteeringWheelSpawnChance {
        moon: "experimentation",
        chance: I32F32::lit("5.62"),
    },
    SteeringWheelSpawnChance {
        moon: "assurance",
        chance: I32F32::lit("2.23"),
    },
    SteeringWheelSpawnChance {
        moon: "vow",
        chance: I32F32::lit("1.75"),
    },
    SteeringWheelSpawnChance {
        moon: "adamance",
        chance: I32F32::lit("1.27"),
    },
];

pub const STEERING_WHEEL_BEHAVIORAL_MECHANICS: [SteeringWheelBehaviorRule; 4] = [
    SteeringWheelBehaviorRule {
        condition: "found on experimentation",
        outcome: "spawn chance is 5.62%",
    },
    SteeringWheelBehaviorRule {
        condition: "found on assurance",
        outcome: "spawn chance is 2.23%",
    },
    SteeringWheelBehaviorRule {
        condition: "found on vow",
        outcome: "spawn chance is 1.75%",
    },
    SteeringWheelBehaviorRule {
        condition: "found on adamance",
        outcome: "spawn chance is 1.27%",
    },
];

pub struct SteeringWheelPlugin;

impl Plugin for SteeringWheelPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnSteeringWheelEvent>()
            .add_event::<SteeringWheelSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_steering_wheel,
                    steering_wheel_pickup_item_bar_bridge,
                    steering_wheel_sell_bridge,
                    steering_wheel_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SteeringWheelBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SteeringWheelSpawnChance {
    pub moon: &'static str,
    pub chance: I32F32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SteeringWheel {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SteeringWheelScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for SteeringWheelScrap {
    fn default() -> Self {
        Self {
            min_value: STEERING_WHEEL_MIN_VALUE,
            max_value: STEERING_WHEEL_MAX_VALUE,
            weight: STEERING_WHEEL_WEIGHT,
            conductive: STEERING_WHEEL_CONDUCTIVE,
            two_handed: STEERING_WHEEL_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SteeringWheelHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Bundle)]
pub struct SteeringWheelBundle {
    pub name: Name,
    pub steering_wheel: SteeringWheel,
    pub scrap: SteeringWheelScrap,
    pub position: SimPosition,
    pub held_by: SteeringWheelHeldBy,
}

impl SteeringWheelBundle {
    pub fn new(event: SpawnSteeringWheelEvent) -> Self {
        Self {
            name: Name::new(STEERING_WHEEL_NAME),
            steering_wheel: SteeringWheel {
                stable_id: event.stable_id,
            },
            scrap: SteeringWheelScrap::default(),
            position: event.position,
            held_by: SteeringWheelHeldBy::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnSteeringWheelEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SteeringWheelSoldEvent {
    pub steering_wheel_stable_id: u64,
    pub credit_value: I32F32,
}

pub fn steering_wheel_value_range() -> (I32F32, I32F32) {
    (STEERING_WHEEL_MIN_VALUE, STEERING_WHEEL_MAX_VALUE)
}

pub fn steering_wheel_spawn_chance_for_moon(moon: &str) -> Option<I32F32> {
    STEERING_WHEEL_SPAWN_CHANCES
        .iter()
        .find(|s| s.moon == moon)
        .map(|s| s.chance)
}

fn spawn_steering_wheel(
    mut commands: Commands,
    mut events: EventReader<SpawnSteeringWheelEvent>,
) {
    for event in events.read() {
        commands.spawn(SteeringWheelBundle::new(*event));
    }
}

fn steering_wheel_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    wheels: Query<(&SteeringWheel, &SteeringWheelHeldBy), Changed<SteeringWheelHeldBy>>,
) {
    for (wheel, held_by) in &wheels {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: STEERING_WHEEL_ID,
                two_handed: STEERING_WHEEL_TWO_HANDED,
                functional: false,
                passive: false,
                from_store_or_valueless: false,
            });
        } else {
            let _ = wheel.stable_id;
        }
    }
}

fn steering_wheel_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<SteeringWheelSoldEvent>,
    wheels: Query<(&SteeringWheel, &SteeringWheelScrap)>,
) {
    for event in sell_events.read() {
        for (wheel, scrap) in &wheels {
            if wheel.stable_id != event.scrap_entity_id {
                continue;
            }
            sold_events.send(SteeringWheelSoldEvent {
                steering_wheel_stable_id: wheel.stable_id,
                credit_value: scrap.max_value,
            });
        }
    }
}

fn steering_wheel_checksum(
    mut checksum: ResMut<SimChecksumState>,
    wheels: Query<(&SteeringWheel, &SteeringWheelScrap, &SimPosition, &SteeringWheelHeldBy)>,
) {
    accumulate_str(&mut checksum, 0x1000, STEERING_WHEEL_ID);
    accumulate_str(&mut checksum, 0x1001, STEERING_WHEEL_NAME);
    accumulate_str(&mut checksum, 0x1002, STEERING_WHEEL_TYPE);
    accumulate_str(&mut checksum, 0x1003, STEERING_WHEEL_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, STEERING_WHEEL_EFFECTS);

    checksum.accumulate(STEERING_WHEEL_SOURCE_REVISION as u64);
    checksum.accumulate(STEERING_WHEEL_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(STEERING_WHEEL_WEIGHT.to_bits() as u64);
    checksum.accumulate(STEERING_WHEEL_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(STEERING_WHEEL_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(STEERING_WHEEL_CONDUCTIVE as u64);
    checksum.accumulate(STEERING_WHEEL_TWO_HANDED as u64);

    for dependency in STEERING_WHEEL_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for spawn_chance in STEERING_WHEEL_SPAWN_CHANCES {
        accumulate_str(&mut checksum, 0x4000, spawn_chance.moon);
        checksum.accumulate(spawn_chance.chance.to_bits() as u64);
    }

    for rule in STEERING_WHEEL_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x5000, rule.condition);
        accumulate_str(&mut checksum, 0x5001, rule.outcome);
    }

    for (wheel, scrap, position, held_by) in &wheels {
        checksum.accumulate(wheel.stable_id);
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
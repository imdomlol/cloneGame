// Sources: vault/scrap_items/hairdryer.md
use bevy::prelude::*;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::{
    ItemBarItemEffect, ItemBarItemEffectEvent, ItemBarPickupEvent,
};
use crate::sim::{NoiseEmittedEvent, SimChecksumState, SimPosition};

use fixed::types::I32F32;

pub const HAIRDRYER_ID: &str = "hairdryer";
pub const HAIRDRYER_NAME: &str = "Hairdryer";
pub const HAIRDRYER_TYPE: &str = "scrap_items";
pub const HAIRDRYER_SUBTYPE: &str = "scrap";
pub const HAIRDRYER_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Hairdryer";
pub const HAIRDRYER_SOURCE_REVISION: u32 = 20299;
pub const HAIRDRYER_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const HAIRDRYER_CONFIDENCE_BASIS_POINTS: u16 = 92;

pub const HAIRDRYER_EFFECTS: &str = "Makes hairdryer sounds";
pub const HAIRDRYER_WEIGHT: I32F32 = I32F32::lit("7");
pub const HAIRDRYER_CONDUCTIVE: bool = false;
pub const HAIRDRYER_MIN_VALUE: I32F32 = I32F32::lit("60");
pub const HAIRDRYER_MAX_VALUE: I32F32 = I32F32::lit("99");
pub const HAIRDRYER_TWO_HANDED: bool = false;
pub const HAIRDRYER_BATTERY_LIFE_USES: u16 = 10;
pub const HAIRDRYER_PAGE_ID: u32 = 37;
pub const HAIRDRYER_SOUND_AMOUNT: I32F32 = I32F32::lit("80");

pub const HAIRDRYER_DEPENDS_ON: [&str; 5] = [
    "electric_coil",
    "the_ship",
    "eyeless_dog",
    "audible_sounds",
    "the_company",
];

pub const HAIRDRYER_SPAWN_CHANCES: [HairdryerSpawnChance; 4] = [
    HairdryerSpawnChance {
        moon: "artifice",
        chance: I32F32::lit("4.06"),
    },
    HairdryerSpawnChance {
        moon: "rend",
        chance: I32F32::lit("3.99"),
    },
    HairdryerSpawnChance {
        moon: "titan",
        chance: I32F32::lit("3.23"),
    },
    HairdryerSpawnChance {
        moon: "assurance",
        chance: I32F32::lit("0.47"),
    },
];

pub const HAIRDRYER_BEHAVIORAL_MECHANICS: [HairdryerBehaviorRule; 8] = [
    HairdryerBehaviorRule {
        condition: "left mouse button is pressed",
        outcome: "the Hairdryer is used",
    },
    HairdryerBehaviorRule {
        condition: "the Hairdryer is activated",
        outcome: "it produces a loud blowing sound",
    },
    HairdryerBehaviorRule {
        condition: "the Hairdryer is fully charged",
        outcome: "it supports 10 uses before depletion",
    },
    HairdryerBehaviorRule {
        condition: "it is recharged using electric_coil in the_ship",
        outcome: "its battery uses can be restored",
    },
    HairdryerBehaviorRule {
        condition: "the sound is emitted",
        outcome: "it can alert eyeless_dog and other entities capable of audible_sounds",
    },
    HairdryerBehaviorRule {
        condition: "it is sold to the_company",
        outcome: "it can be exchanged for currency",
    },
    HairdryerBehaviorRule {
        condition: "artifice",
        outcome: "spawn chance is 4.06%",
    },
    HairdryerBehaviorRule {
        condition: "rend",
        outcome: "spawn chance is 3.99%",
    },
];

pub struct HairdryerPlugin;

impl Plugin for HairdryerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnHairdryerEvent>()
            .add_event::<HairdryerUsedEvent>()
            .add_event::<HairdryerAudibleAlertEvent>()
            .add_event::<HairdryerRechargedEvent>()
            .add_event::<HairdryerBatteryDepletedEvent>()
            .add_event::<HairdryerSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_hairdryer,
                    hairdryer_pickup_item_bar_bridge,
                    hairdryer_use_from_item_bar,
                    hairdryer_emit_noise,
                    hairdryer_recharge,
                    hairdryer_sell_bridge,
                    hairdryer_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HairdryerBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HairdryerSpawnChance {
    pub moon: &'static str,
    pub chance: I32F32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Hairdryer {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HairdryerScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for HairdryerScrap {
    fn default() -> Self {
        Self {
            min_value: HAIRDRYER_MIN_VALUE,
            max_value: HAIRDRYER_MAX_VALUE,
            weight: HAIRDRYER_WEIGHT,
            conductive: HAIRDRYER_CONDUCTIVE,
            two_handed: HAIRDRYER_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HairdryerHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HairdryerBatteryState {
    pub uses_remaining: u16,
    pub total_uses: u64,
    pub last_used_tick: u64,
    pub is_active: bool,
}

impl Default for HairdryerBatteryState {
    fn default() -> Self {
        Self {
            uses_remaining: HAIRDRYER_BATTERY_LIFE_USES,
            total_uses: 0,
            last_used_tick: 0,
            is_active: false,
        }
    }
}

#[derive(Bundle)]
pub struct HairdryerBundle {
    pub name: Name,
    pub hairdryer: Hairdryer,
    pub scrap: HairdryerScrap,
    pub position: SimPosition,
    pub held_by: HairdryerHeldBy,
    pub battery: HairdryerBatteryState,
}

impl HairdryerBundle {
    pub fn new(event: SpawnHairdryerEvent) -> Self {
        Self {
            name: Name::new(HAIRDRYER_NAME),
            hairdryer: Hairdryer {
                stable_id: event.stable_id,
            },
            scrap: HairdryerScrap::default(),
            position: event.position,
            held_by: HairdryerHeldBy::default(),
            battery: HairdryerBatteryState::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnHairdryerEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HairdryerUsedEvent {
    pub hairdryer_entity: Entity,
    pub hairdryer_stable_id: u64,
    pub employee_id: u64,
    pub position: SimPosition,
    pub uses_remaining: u16,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HairdryerAudibleAlertEvent {
    pub source: Entity,
    pub employee_id: u64,
    pub position: SimPosition,
    pub amount: I32F32,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HairdryerRechargedEvent {
    pub hairdryer_stable_id: u64,
    pub restored_uses: u16,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HairdryerBatteryDepletedEvent {
    pub hairdryer_entity: Entity,
    pub hairdryer_stable_id: u64,
    pub employee_id: u64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HairdryerSoldEvent {
    pub hairdryer_stable_id: u64,
    pub credit_value: I32F32,
}

pub fn hairdryer_value_range() -> (I32F32, I32F32) {
    (HAIRDRYER_MIN_VALUE, HAIRDRYER_MAX_VALUE)
}

pub fn hairdryer_spawn_chance_for_moon(moon: &str) -> Option<I32F32> {
    HAIRDRYER_SPAWN_CHANCES
        .iter()
        .find(|spawn_chance| spawn_chance.moon == moon)
        .map(|spawn_chance| spawn_chance.chance)
}

fn spawn_hairdryer(mut commands: Commands, mut events: EventReader<SpawnHairdryerEvent>) {
    for event in events.read() {
        commands.spawn(HairdryerBundle::new(*event));
    }
}

fn hairdryer_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    hairdryers: Query<(&Hairdryer, &HairdryerHeldBy), Changed<HairdryerHeldBy>>,
) {
    for (hairdryer, held_by) in &hairdryers {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: HAIRDRYER_ID,
                two_handed: HAIRDRYER_TWO_HANDED,
                functional: true,
                passive: false,
                from_store_or_valueless: false,
            });
        } else {
            let _ = hairdryer.stable_id;
        }
    }
}

fn hairdryer_use_from_item_bar(
    mut item_events: EventReader<ItemBarItemEffectEvent>,
    mut used_events: EventWriter<HairdryerUsedEvent>,
    mut depleted_events: EventWriter<HairdryerBatteryDepletedEvent>,
    mut hairdryers: Query<(
        Entity,
        &Hairdryer,
        &HairdryerHeldBy,
        &SimPosition,
        &mut HairdryerBatteryState,
    )>,
    tick: Res<crate::sim::SimTick>,
) {
    for event in item_events.read() {
        if event.item_id != HAIRDRYER_ID || event.effect != ItemBarItemEffect::FunctionalActivated {
            continue;
        }

        for (entity, hairdryer, held_by, position, mut battery) in &mut hairdryers {
            if !held_by.is_held || held_by.employee_id != event.employee_id {
                continue;
            }

            if battery.uses_remaining == 0 {
                battery.is_active = false;
                depleted_events.send(HairdryerBatteryDepletedEvent {
                    hairdryer_entity: entity,
                    hairdryer_stable_id: hairdryer.stable_id,
                    employee_id: event.employee_id,
                });
                continue;
            }

            battery.uses_remaining -= 1;
            battery.total_uses = battery.total_uses.wrapping_add(1);
            battery.last_used_tick = tick.0;
            battery.is_active = true;

            used_events.send(HairdryerUsedEvent {
                hairdryer_entity: entity,
                hairdryer_stable_id: hairdryer.stable_id,
                employee_id: event.employee_id,
                position: *position,
                uses_remaining: battery.uses_remaining,
            });

            if battery.uses_remaining == 0 {
                depleted_events.send(HairdryerBatteryDepletedEvent {
                    hairdryer_entity: entity,
                    hairdryer_stable_id: hairdryer.stable_id,
                    employee_id: event.employee_id,
                });
            }
        }
    }
}

fn hairdryer_emit_noise(
    mut used_events: EventReader<HairdryerUsedEvent>,
    mut noise_events: EventWriter<NoiseEmittedEvent>,
    mut alert_events: EventWriter<HairdryerAudibleAlertEvent>,
) {
    for event in used_events.read() {
        noise_events.send(NoiseEmittedEvent {
            source: event.hairdryer_entity,
            position: event.position,
            amount: HAIRDRYER_SOUND_AMOUNT,
        });

        alert_events.send(HairdryerAudibleAlertEvent {
            source: event.hairdryer_entity,
            employee_id: event.employee_id,
            position: event.position,
            amount: HAIRDRYER_SOUND_AMOUNT,
        });
    }
}

fn hairdryer_recharge(
    mut recharge_events: EventReader<HairdryerRechargedEvent>,
    mut hairdryers: Query<(&Hairdryer, &mut HairdryerBatteryState)>,
) {
    for event in recharge_events.read() {
        for (hairdryer, mut battery) in &mut hairdryers {
            if hairdryer.stable_id != event.hairdryer_stable_id {
                continue;
            }

            battery.uses_remaining = event.restored_uses.min(HAIRDRYER_BATTERY_LIFE_USES);
            battery.is_active = false;
        }
    }
}

fn hairdryer_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<HairdryerSoldEvent>,
    hairdryers: Query<(&Hairdryer, &HairdryerScrap)>,
) {
    for event in sell_events.read() {
        for (hairdryer, scrap) in &hairdryers {
            if hairdryer.stable_id != event.scrap_entity_id {
                continue;
            }

            sold_events.send(HairdryerSoldEvent {
                hairdryer_stable_id: hairdryer.stable_id,
                credit_value: scrap.max_value,
            });
        }
    }
}

fn hairdryer_checksum(
    mut checksum: ResMut<SimChecksumState>,
    hairdryers: Query<(
        &Hairdryer,
        &HairdryerScrap,
        &SimPosition,
        &HairdryerHeldBy,
        &HairdryerBatteryState,
    )>,
) {
    accumulate_str(&mut checksum, 0x1000, HAIRDRYER_ID);
    accumulate_str(&mut checksum, 0x1001, HAIRDRYER_NAME);
    accumulate_str(&mut checksum, 0x1002, HAIRDRYER_TYPE);
    accumulate_str(&mut checksum, 0x1003, HAIRDRYER_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, HAIRDRYER_EFFECTS);
    accumulate_str(&mut checksum, 0x1005, HAIRDRYER_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1006, HAIRDRYER_EXTRACTED_AT);

    checksum.accumulate(HAIRDRYER_SOURCE_REVISION as u64);
    checksum.accumulate(HAIRDRYER_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(HAIRDRYER_WEIGHT.to_bits() as u64);
    checksum.accumulate(HAIRDRYER_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(HAIRDRYER_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(HAIRDRYER_CONDUCTIVE as u64);
    checksum.accumulate(HAIRDRYER_TWO_HANDED as u64);
    checksum.accumulate(HAIRDRYER_BATTERY_LIFE_USES as u64);
    checksum.accumulate(HAIRDRYER_PAGE_ID as u64);
    checksum.accumulate(HAIRDRYER_SOUND_AMOUNT.to_bits() as u64);

    for dependency in HAIRDRYER_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for spawn_chance in HAIRDRYER_SPAWN_CHANCES {
        accumulate_str(&mut checksum, 0x3000, spawn_chance.moon);
        checksum.accumulate(spawn_chance.chance.to_bits() as u64);
    }

    for rule in HAIRDRYER_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x4000, rule.condition);
        accumulate_str(&mut checksum, 0x4001, rule.outcome);
    }

    for (hairdryer, scrap, position, held_by, battery) in &hairdryers {
        checksum.accumulate(hairdryer.stable_id);
        checksum.accumulate(scrap.min_value.to_bits() as u64);
        checksum.accumulate(scrap.max_value.to_bits() as u64);
        checksum.accumulate(scrap.weight.to_bits() as u64);
        checksum.accumulate(scrap.conductive as u64);
        checksum.accumulate(scrap.two_handed as u64);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(held_by.employee_id);
        checksum.accumulate(held_by.is_held as u64);
        checksum.accumulate(battery.uses_remaining as u64);
        checksum.accumulate(battery.total_uses);
        checksum.accumulate(battery.last_used_tick);
        checksum.accumulate(battery.is_active as u64);
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt);
    for byte in value.as_bytes() {
        checksum.accumulate(*byte as u64);
    }
}
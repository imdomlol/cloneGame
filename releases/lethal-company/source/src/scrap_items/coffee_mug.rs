// Sources: vault/scrap_items/coffee_mug.md
use bevy::prelude::*;
use fixed::types::I32F32;
use rand_core::RngCore;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::{
    ItemBarItemEffect, ItemBarItemEffectEvent, ItemBarPickupEvent,
};
use crate::sim::{tick_rng, GameSeed, SimChecksumState, SimPosition, SimTick};

pub const COFFEE_MUG_ID: &str = "coffee_mug";
pub const COFFEE_MUG_NAME: &str = "Coffee mug";
pub const COFFEE_MUG_TYPE: &str = "scrap_items";
pub const COFFEE_MUG_SUBTYPE: &str = "scrap";
pub const COFFEE_MUG_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Coffee_Mug";
pub const COFFEE_MUG_SOURCE_REVISION: u32 = 20392;
pub const COFFEE_MUG_EXTRACTED_AT: &str = "2026-06-07T00:00:00Z";
pub const COFFEE_MUG_CONFIDENCE_BASIS_POINTS: u16 = 84;

pub const COFFEE_MUG_EFFECTS: &str = "Inspectable while held; can be sold to the company.";
pub const COFFEE_MUG_WEIGHT: I32F32 = I32F32::lit("5");
pub const COFFEE_MUG_CONDUCTIVE: bool = false;
pub const COFFEE_MUG_MIN_VALUE: I32F32 = I32F32::lit("24");
pub const COFFEE_MUG_MAX_VALUE: I32F32 = I32F32::lit("67");
pub const COFFEE_MUG_TWO_HANDED: bool = false;
pub const COFFEE_MUG_DESIGN_VARIANT_SALT: u64 = 0x636f_6666_6565_6d75;

pub const COFFEE_MUG_DEPENDS_ON: [&str; 4] = [
    "lethal_company",
    "clipboard",
    "employee",
    "the_company",
];

pub const COFFEE_MUG_SPAWN_CHANCES: [CoffeeMugSpawnChance; 6] = [
    CoffeeMugSpawnChance {
        moon: "rend",
        chance: I32F32::lit("4.08"),
    },
    CoffeeMugSpawnChance {
        moon: "titan",
        chance: I32F32::lit("2.26"),
    },
    CoffeeMugSpawnChance {
        moon: "vow",
        chance: I32F32::lit("1.85"),
    },
    CoffeeMugSpawnChance {
        moon: "assurance",
        chance: I32F32::lit("1.41"),
    },
    CoffeeMugSpawnChance {
        moon: "adamance",
        chance: I32F32::lit("1.27"),
    },
    CoffeeMugSpawnChance {
        moon: "artifice",
        chance: I32F32::lit("0.74"),
    },
];

pub const COFFEE_MUG_BEHAVIORAL_MECHANICS: [CoffeeMugBehaviorRule; 3] = [
    CoffeeMugBehaviorRule {
        condition: "the mug is held",
        outcome: "an employee can inspect it similarly to the clipboard",
    },
    CoffeeMugBehaviorRule {
        condition: "the mug is generated",
        outcome: "it can receive a randomly assigned design variant",
    },
    CoffeeMugBehaviorRule {
        condition: "the mug is sold",
        outcome: "its practical effect is conversion into value at the_company",
    },
];

pub struct CoffeeMugPlugin;

impl Plugin for CoffeeMugPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnCoffeeMugEvent>()
            .add_event::<CoffeeMugInspectedEvent>()
            .add_event::<CoffeeMugSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_coffee_mug,
                    coffee_mug_pickup_item_bar_bridge,
                    coffee_mug_inspect_from_item_bar,
                    coffee_mug_sell_bridge,
                    coffee_mug_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoffeeMugBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoffeeMugSpawnChance {
    pub moon: &'static str,
    pub chance: I32F32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CoffeeMug {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoffeeMugScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for CoffeeMugScrap {
    fn default() -> Self {
        Self {
            min_value: COFFEE_MUG_MIN_VALUE,
            max_value: COFFEE_MUG_MAX_VALUE,
            weight: COFFEE_MUG_WEIGHT,
            conductive: COFFEE_MUG_CONDUCTIVE,
            two_handed: COFFEE_MUG_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CoffeeMugHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CoffeeMugInspectionState {
    pub design_variant: u32,
    pub inspections: u64,
    pub last_inspected_tick: u64,
}

#[derive(Bundle)]
pub struct CoffeeMugBundle {
    pub name: Name,
    pub coffee_mug: CoffeeMug,
    pub scrap: CoffeeMugScrap,
    pub position: SimPosition,
    pub held_by: CoffeeMugHeldBy,
    pub inspection_state: CoffeeMugInspectionState,
}

impl CoffeeMugBundle {
    pub fn new(event: SpawnCoffeeMugEvent, design_variant: u32) -> Self {
        Self {
            name: Name::new(COFFEE_MUG_NAME),
            coffee_mug: CoffeeMug {
                stable_id: event.stable_id,
            },
            scrap: CoffeeMugScrap::default(),
            position: event.position,
            held_by: CoffeeMugHeldBy::default(),
            inspection_state: CoffeeMugInspectionState {
                design_variant,
                inspections: 0,
                last_inspected_tick: 0,
            },
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnCoffeeMugEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoffeeMugInspectedEvent {
    pub coffee_mug_entity: Entity,
    pub coffee_mug_stable_id: u64,
    pub employee_id: u64,
    pub design_variant: u32,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoffeeMugSoldEvent {
    pub coffee_mug_stable_id: u64,
    pub credit_value: I32F32,
}

pub fn coffee_mug_value_range() -> (I32F32, I32F32) {
    (COFFEE_MUG_MIN_VALUE, COFFEE_MUG_MAX_VALUE)
}

pub fn coffee_mug_spawn_chance_for_moon(moon: &str) -> Option<I32F32> {
    COFFEE_MUG_SPAWN_CHANCES
        .iter()
        .find(|spawn_chance| spawn_chance.moon == moon)
        .map(|spawn_chance| spawn_chance.chance)
}

fn spawn_coffee_mug(
    mut commands: Commands,
    mut events: EventReader<SpawnCoffeeMugEvent>,
    seed: Res<GameSeed>,
    tick: Res<SimTick>,
) {
    for event in events.read() {
        let salt = COFFEE_MUG_DESIGN_VARIANT_SALT ^ event.stable_id;
        let mut rng = tick_rng(seed.0, tick.0, salt);
        let design_variant = rng.next_u32();

        commands.spawn(CoffeeMugBundle::new(*event, design_variant));
    }
}

fn coffee_mug_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    coffee_mugs: Query<(&CoffeeMug, &CoffeeMugHeldBy), Changed<CoffeeMugHeldBy>>,
) {
    for (coffee_mug, held_by) in &coffee_mugs {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: COFFEE_MUG_ID,
                two_handed: COFFEE_MUG_TWO_HANDED,
                functional: true,
                passive: false,
                from_store_or_valueless: false,
            });
        } else {
            let _ = coffee_mug.stable_id;
        }
    }
}

fn coffee_mug_inspect_from_item_bar(
    mut item_events: EventReader<ItemBarItemEffectEvent>,
    mut inspected_events: EventWriter<CoffeeMugInspectedEvent>,
    mut coffee_mugs: Query<(
        Entity,
        &CoffeeMug,
        &CoffeeMugHeldBy,
        &mut CoffeeMugInspectionState,
    )>,
    tick: Res<SimTick>,
) {
    for event in item_events.read() {
        if event.item_id != COFFEE_MUG_ID || event.effect != ItemBarItemEffect::FunctionalActivated {
            continue;
        }

        for (entity, coffee_mug, held_by, mut inspection_state) in &mut coffee_mugs {
            if !held_by.is_held || held_by.employee_id != event.employee_id {
                continue;
            }

            inspection_state.inspections = inspection_state.inspections.wrapping_add(1);
            inspection_state.last_inspected_tick = tick.0;

            inspected_events.send(CoffeeMugInspectedEvent {
                coffee_mug_entity: entity,
                coffee_mug_stable_id: coffee_mug.stable_id,
                employee_id: event.employee_id,
                design_variant: inspection_state.design_variant,
            });
        }
    }
}

fn coffee_mug_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<CoffeeMugSoldEvent>,
    coffee_mugs: Query<(&CoffeeMug, &CoffeeMugScrap)>,
) {
    for event in sell_events.read() {
        for (coffee_mug, scrap) in &coffee_mugs {
            if coffee_mug.stable_id != event.scrap_entity_id {
                continue;
            }

            sold_events.send(CoffeeMugSoldEvent {
                coffee_mug_stable_id: coffee_mug.stable_id,
                credit_value: scrap.max_value,
            });
        }
    }
}

fn coffee_mug_checksum(
    mut checksum: ResMut<SimChecksumState>,
    coffee_mugs: Query<(
        &CoffeeMug,
        &CoffeeMugScrap,
        &SimPosition,
        &CoffeeMugHeldBy,
        &CoffeeMugInspectionState,
    )>,
) {
    accumulate_str(&mut checksum, 0x1000, COFFEE_MUG_ID);
    accumulate_str(&mut checksum, 0x1001, COFFEE_MUG_NAME);
    accumulate_str(&mut checksum, 0x1002, COFFEE_MUG_TYPE);
    accumulate_str(&mut checksum, 0x1003, COFFEE_MUG_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, COFFEE_MUG_EFFECTS);
    accumulate_str(&mut checksum, 0x1005, COFFEE_MUG_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1006, COFFEE_MUG_EXTRACTED_AT);

    checksum.accumulate(COFFEE_MUG_SOURCE_REVISION as u64);
    checksum.accumulate(COFFEE_MUG_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(COFFEE_MUG_WEIGHT.to_bits() as u64);
    checksum.accumulate(COFFEE_MUG_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(COFFEE_MUG_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(COFFEE_MUG_CONDUCTIVE as u64);
    checksum.accumulate(COFFEE_MUG_TWO_HANDED as u64);

    for dependency in COFFEE_MUG_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for spawn_chance in COFFEE_MUG_SPAWN_CHANCES {
        accumulate_str(&mut checksum, 0x3000, spawn_chance.moon);
        checksum.accumulate(spawn_chance.chance.to_bits() as u64);
    }

    for rule in COFFEE_MUG_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x4000, rule.condition);
        accumulate_str(&mut checksum, 0x4001, rule.outcome);
    }

    for (coffee_mug, scrap, position, held_by, inspection_state) in &coffee_mugs {
        checksum.accumulate(coffee_mug.stable_id);
        checksum.accumulate(scrap.min_value.to_bits() as u64);
        checksum.accumulate(scrap.max_value.to_bits() as u64);
        checksum.accumulate(scrap.weight.to_bits() as u64);
        checksum.accumulate(scrap.conductive as u64);
        checksum.accumulate(scrap.two_handed as u64);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(held_by.employee_id);
        checksum.accumulate(held_by.is_held as u64);
        checksum.accumulate(inspection_state.design_variant as u64);
        checksum.accumulate(inspection_state.inspections);
        checksum.accumulate(inspection_state.last_inspected_tick);
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt);
    for byte in value.as_bytes() {
        checksum.accumulate(*byte as u64);
    }
}
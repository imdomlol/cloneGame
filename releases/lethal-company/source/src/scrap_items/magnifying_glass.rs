// Sources: vault/scrap_items/magnifying_glass.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::{
    ItemBarItemEffect, ItemBarItemEffectEvent, ItemBarPickupEvent,
};
use crate::sim::{SimChecksumState, SimPosition};

pub const MAGNIFYING_GLASS_ID: &str = "magnifying_glass";
pub const MAGNIFYING_GLASS_NAME: &str = "Magnifying glass";
pub const MAGNIFYING_GLASS_TYPE: &str = "scrap_items";
pub const MAGNIFYING_GLASS_SUBTYPE: &str = "scrap";
pub const MAGNIFYING_GLASS_SOURCE_URL: &str =
    "https://lethal-company.fandom.com/wiki/Magnifying_Glass";
pub const MAGNIFYING_GLASS_SOURCE_REVISION: u32 = 20245;
pub const MAGNIFYING_GLASS_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const MAGNIFYING_GLASS_CONFIDENCE_BASIS_POINTS: u16 = 92;

pub const MAGNIFYING_GLASS_EFFECTS: &str = "Inspectable while held; no other known use.";
pub const MAGNIFYING_GLASS_WEIGHT: I32F32 = I32F32::lit("11");
pub const MAGNIFYING_GLASS_CONDUCTIVE: bool = false;
pub const MAGNIFYING_GLASS_TWO_HANDED: bool = false;
pub const MAGNIFYING_GLASS_MIN_VALUE: I32F32 = I32F32::lit("44");
pub const MAGNIFYING_GLASS_MAX_VALUE: I32F32 = I32F32::lit("59");

pub const MAGNIFYING_GLASS_DEPENDS_ON: [&str; 4] = [
    "lethal_company",
    "employee",
    "clipboard",
    "the_company",
];

pub const MAGNIFYING_GLASS_BEHAVIORAL_MECHANICS: [MagnifyingGlassBehaviorRule; 2] = [
    MagnifyingGlassBehaviorRule {
        condition: "held",
        outcome: "pressing Z inspects the item similarly to clipboard",
    },
    MagnifyingGlassBehaviorRule {
        condition: "sold to the_company",
        outcome: "it yields credits in the range from 44 to 59",
    },
];

pub struct MagnifyingGlassPlugin;

impl Plugin for MagnifyingGlassPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnMagnifyingGlassEvent>()
            .add_event::<MagnifyingGlassInspectedEvent>()
            .add_event::<MagnifyingGlassSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_magnifying_glass,
                    magnifying_glass_pickup_item_bar_bridge,
                    magnifying_glass_inspect_from_item_bar,
                    magnifying_glass_sell_bridge,
                    magnifying_glass_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MagnifyingGlassBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MagnifyingGlass {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MagnifyingGlassScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for MagnifyingGlassScrap {
    fn default() -> Self {
        Self {
            min_value: MAGNIFYING_GLASS_MIN_VALUE,
            max_value: MAGNIFYING_GLASS_MAX_VALUE,
            weight: MAGNIFYING_GLASS_WEIGHT,
            conductive: MAGNIFYING_GLASS_CONDUCTIVE,
            two_handed: MAGNIFYING_GLASS_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MagnifyingGlassHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MagnifyingGlassInspectState {
    pub inspections: u64,
    pub last_inspected_tick: u64,
}

#[derive(Bundle)]
pub struct MagnifyingGlassBundle {
    pub name: Name,
    pub magnifying_glass: MagnifyingGlass,
    pub scrap: MagnifyingGlassScrap,
    pub position: SimPosition,
    pub held_by: MagnifyingGlassHeldBy,
    pub inspect_state: MagnifyingGlassInspectState,
}

impl MagnifyingGlassBundle {
    pub fn new(event: SpawnMagnifyingGlassEvent) -> Self {
        Self {
            name: Name::new(MAGNIFYING_GLASS_NAME),
            magnifying_glass: MagnifyingGlass {
                stable_id: event.stable_id,
            },
            scrap: MagnifyingGlassScrap::default(),
            position: event.position,
            held_by: MagnifyingGlassHeldBy::default(),
            inspect_state: MagnifyingGlassInspectState::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnMagnifyingGlassEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MagnifyingGlassInspectedEvent {
    pub magnifying_glass_entity: Entity,
    pub magnifying_glass_stable_id: u64,
    pub employee_id: u64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MagnifyingGlassSoldEvent {
    pub magnifying_glass_stable_id: u64,
    pub credit_value: I32F32,
}

pub fn magnifying_glass_value_range() -> (I32F32, I32F32) {
    (MAGNIFYING_GLASS_MIN_VALUE, MAGNIFYING_GLASS_MAX_VALUE)
}

fn spawn_magnifying_glass(
    mut commands: Commands,
    mut events: EventReader<SpawnMagnifyingGlassEvent>,
) {
    for event in events.read() {
        commands.spawn(MagnifyingGlassBundle::new(*event));
    }
}

fn magnifying_glass_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    magnifying_glasses: Query<(&MagnifyingGlass, &MagnifyingGlassHeldBy), Changed<MagnifyingGlassHeldBy>>,
) {
    for (magnifying_glass, held_by) in &magnifying_glasses {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: MAGNIFYING_GLASS_ID,
                two_handed: MAGNIFYING_GLASS_TWO_HANDED,
                functional: true,
                passive: false,
                from_store_or_valueless: false,
            });
        } else {
            let _ = magnifying_glass.stable_id;
        }
    }
}

fn magnifying_glass_inspect_from_item_bar(
    mut item_events: EventReader<ItemBarItemEffectEvent>,
    mut inspected_events: EventWriter<MagnifyingGlassInspectedEvent>,
    mut magnifying_glasses: Query<(
        Entity,
        &MagnifyingGlass,
        &MagnifyingGlassHeldBy,
        &mut MagnifyingGlassInspectState,
    )>,
    tick: Res<crate::sim::SimTick>,
) {
    for event in item_events.read() {
        if event.item_id != MAGNIFYING_GLASS_ID || event.effect != ItemBarItemEffect::FunctionalActivated {
            continue;
        }

        for (entity, magnifying_glass, held_by, mut inspect_state) in &mut magnifying_glasses {
            if !held_by.is_held || held_by.employee_id != event.employee_id {
                continue;
            }

            inspect_state.inspections = inspect_state.inspections.wrapping_add(1);
            inspect_state.last_inspected_tick = tick.0;

            inspected_events.send(MagnifyingGlassInspectedEvent {
                magnifying_glass_entity: entity,
                magnifying_glass_stable_id: magnifying_glass.stable_id,
                employee_id: event.employee_id,
            });
        }
    }
}

fn magnifying_glass_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<MagnifyingGlassSoldEvent>,
    magnifying_glasses: Query<(&MagnifyingGlass, &MagnifyingGlassScrap)>,
) {
    for event in sell_events.read() {
        for (magnifying_glass, scrap) in &magnifying_glasses {
            if magnifying_glass.stable_id != event.scrap_entity_id {
                continue;
            }

            sold_events.send(MagnifyingGlassSoldEvent {
                magnifying_glass_stable_id: magnifying_glass.stable_id,
                credit_value: scrap.max_value,
            });
        }
    }
}

fn magnifying_glass_checksum(
    mut checksum: ResMut<SimChecksumState>,
    magnifying_glasses: Query<(
        &MagnifyingGlass,
        &MagnifyingGlassScrap,
        &SimPosition,
        &MagnifyingGlassHeldBy,
        &MagnifyingGlassInspectState,
    )>,
) {
    accumulate_str(&mut checksum, 0x1000, MAGNIFYING_GLASS_ID);
    accumulate_str(&mut checksum, 0x1001, MAGNIFYING_GLASS_NAME);
    accumulate_str(&mut checksum, 0x1002, MAGNIFYING_GLASS_TYPE);
    accumulate_str(&mut checksum, 0x1003, MAGNIFYING_GLASS_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, MAGNIFYING_GLASS_EFFECTS);

    checksum.accumulate(MAGNIFYING_GLASS_SOURCE_REVISION as u64);
    checksum.accumulate(MAGNIFYING_GLASS_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(MAGNIFYING_GLASS_WEIGHT.to_bits() as u64);
    checksum.accumulate(MAGNIFYING_GLASS_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(MAGNIFYING_GLASS_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(MAGNIFYING_GLASS_CONDUCTIVE as u64);
    checksum.accumulate(MAGNIFYING_GLASS_TWO_HANDED as u64);

    for dependency in MAGNIFYING_GLASS_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for rule in MAGNIFYING_GLASS_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x5000, rule.condition);
        accumulate_str(&mut checksum, 0x5001, rule.outcome);
    }

    for (magnifying_glass, scrap, position, held_by, inspect_state) in &magnifying_glasses {
        checksum.accumulate(magnifying_glass.stable_id);
        checksum.accumulate(scrap.min_value.to_bits() as u64);
        checksum.accumulate(scrap.max_value.to_bits() as u64);
        checksum.accumulate(scrap.weight.to_bits() as u64);
        checksum.accumulate(scrap.conductive as u64);
        checksum.accumulate(scrap.two_handed as u64);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(held_by.employee_id);
        checksum.accumulate(held_by.is_held as u64);
        checksum.accumulate(inspect_state.inspections);
        checksum.accumulate(inspect_state.last_inspected_tick);
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt);
    for byte in value.as_bytes() {
        checksum.accumulate(*byte as u64);
    }
}
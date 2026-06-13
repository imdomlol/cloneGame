// Sources: vault/scrap_items/toilet_paper.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::ItemBarPickupEvent;
use crate::sim::{SimChecksumState, SimPosition};

pub const TOILET_PAPER_ID: &str = "toilet_paper";
pub const TOILET_PAPER_NAME: &str = "Toilet Paper";
pub const TOILET_PAPER_TYPE: &str = "scrap_items";
pub const TOILET_PAPER_SUBTYPE: &str = "scrap";
pub const TOILET_PAPER_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Toilet_Paper";
pub const TOILET_PAPER_SOURCE_REVISION: u32 = 21155;
pub const TOILET_PAPER_EXTRACTED_AT: &str = "2026-06-07T00:00:00Z";
pub const TOILET_PAPER_CONFIDENCE_BASIS_POINTS: u16 = 97;

pub const TOILET_PAPER_EFFECTS: &str =
    "Obstructs the carrier's view while carried; has no use beyond sale.";
pub const TOILET_PAPER_WEIGHT: I32F32 = I32F32::lit("5");
pub const TOILET_PAPER_CONDUCTIVE: bool = false;
pub const TOILET_PAPER_MIN_VALUE: I32F32 = I32F32::lit("60");
pub const TOILET_PAPER_MAX_VALUE: I32F32 = I32F32::lit("87");
pub const TOILET_PAPER_TWO_HANDED: bool = true;

pub const TOILET_PAPER_DEPENDS_ON: [&str; 10] = [
    "employee", "embrion", "adamance", "march", "vow", "assurance", "offense", "artifice", "rend",
    "titan",
];

pub const TOILET_PAPER_SPAWN_CHANCES: [ToiletPaperSpawnChance; 9] = [
    ToiletPaperSpawnChance { moon: "embrion", chance: I32F32::lit("5.21") },
    ToiletPaperSpawnChance { moon: "adamance", chance: I32F32::lit("4.22") },
    ToiletPaperSpawnChance { moon: "march", chance: I32F32::lit("3.65") },
    ToiletPaperSpawnChance { moon: "vow", chance: I32F32::lit("3.49") },
    ToiletPaperSpawnChance { moon: "assurance", chance: I32F32::lit("2.23") },
    ToiletPaperSpawnChance { moon: "offense", chance: I32F32::lit("2.16") },
    ToiletPaperSpawnChance { moon: "artifice", chance: I32F32::lit("1.48") },
    ToiletPaperSpawnChance { moon: "rend", chance: I32F32::lit("1.2") },
    ToiletPaperSpawnChance { moon: "titan", chance: I32F32::lit("0.86") },
];

pub const TOILET_PAPER_BEHAVIORAL_MECHANICS: [ToiletPaperBehaviorRule; 2] = [
    ToiletPaperBehaviorRule {
        condition: "carried by an employee",
        outcome: "it obstructs that employee's view",
    },
    ToiletPaperBehaviorRule {
        condition: "sold to the_company",
        outcome: "it can be exchanged for credits",
    },
];

pub struct ToiletPaperPlugin;

impl Plugin for ToiletPaperPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnToiletPaperEvent>()
            .add_event::<ToiletPaperViewObstructionEvent>()
            .add_event::<ToiletPaperSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_toilet_paper,
                    toilet_paper_pickup_item_bar_bridge,
                    toilet_paper_view_obstruction,
                    toilet_paper_sell_bridge,
                    toilet_paper_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToiletPaperBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToiletPaperSpawnChance {
    pub moon: &'static str,
    pub chance: I32F32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ToiletPaper {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToiletPaperScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for ToiletPaperScrap {
    fn default() -> Self {
        Self {
            min_value: TOILET_PAPER_MIN_VALUE,
            max_value: TOILET_PAPER_MAX_VALUE,
            weight: TOILET_PAPER_WEIGHT,
            conductive: TOILET_PAPER_CONDUCTIVE,
            two_handed: TOILET_PAPER_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ToiletPaperHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ToiletPaperViewState {
    pub view_obstructed: bool,
}

#[derive(Bundle)]
pub struct ToiletPaperBundle {
    pub name: Name,
    pub toilet_paper: ToiletPaper,
    pub scrap: ToiletPaperScrap,
    pub position: SimPosition,
    pub held_by: ToiletPaperHeldBy,
    pub view_state: ToiletPaperViewState,
}

impl ToiletPaperBundle {
    pub fn new(event: SpawnToiletPaperEvent) -> Self {
        Self {
            name: Name::new(TOILET_PAPER_NAME),
            toilet_paper: ToiletPaper { stable_id: event.stable_id },
            scrap: ToiletPaperScrap::default(),
            position: event.position,
            held_by: ToiletPaperHeldBy::default(),
            view_state: ToiletPaperViewState::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnToiletPaperEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToiletPaperViewObstructionEvent {
    pub toilet_paper_entity: Entity,
    pub toilet_paper_stable_id: u64,
    pub employee_id: u64,
    pub obstructed: bool,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToiletPaperSoldEvent {
    pub toilet_paper_stable_id: u64,
    pub credit_value: I32F32,
}

pub fn toilet_paper_value_range() -> (I32F32, I32F32) {
    (TOILET_PAPER_MIN_VALUE, TOILET_PAPER_MAX_VALUE)
}

pub fn toilet_paper_spawn_chance_for_moon(moon: &str) -> Option<I32F32> {
    TOILET_PAPER_SPAWN_CHANCES
        .iter()
        .find(|sc| sc.moon == moon)
        .map(|sc| sc.chance)
}

fn spawn_toilet_paper(mut commands: Commands, mut events: EventReader<SpawnToiletPaperEvent>) {
    for event in events.read() {
        commands.spawn(ToiletPaperBundle::new(*event));
    }
}

fn toilet_paper_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    items: Query<(&ToiletPaper, &ToiletPaperHeldBy), Changed<ToiletPaperHeldBy>>,
) {
    for (toilet_paper, held_by) in &items {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: TOILET_PAPER_ID,
                two_handed: TOILET_PAPER_TWO_HANDED,
                functional: false,
                passive: true,
                from_store_or_valueless: false,
            });
        } else {
            let _ = toilet_paper.stable_id;
        }
    }
}

fn toilet_paper_view_obstruction(
    mut obstruction_events: EventWriter<ToiletPaperViewObstructionEvent>,
    mut items: Query<
        (Entity, &ToiletPaper, &ToiletPaperHeldBy, &mut ToiletPaperViewState),
        Changed<ToiletPaperHeldBy>,
    >,
) {
    for (entity, toilet_paper, held_by, mut view_state) in &mut items {
        let now_obstructed = held_by.is_held;
        if view_state.view_obstructed != now_obstructed {
            view_state.view_obstructed = now_obstructed;
            obstruction_events.send(ToiletPaperViewObstructionEvent {
                toilet_paper_entity: entity,
                toilet_paper_stable_id: toilet_paper.stable_id,
                employee_id: held_by.employee_id,
                obstructed: now_obstructed,
            });
        }
    }
}

fn toilet_paper_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<ToiletPaperSoldEvent>,
    items: Query<(&ToiletPaper, &ToiletPaperScrap)>,
) {
    for event in sell_events.read() {
        for (toilet_paper, scrap) in &items {
            if toilet_paper.stable_id != event.scrap_entity_id {
                continue;
            }
            sold_events.send(ToiletPaperSoldEvent {
                toilet_paper_stable_id: toilet_paper.stable_id,
                credit_value: scrap.max_value,
            });
        }
    }
}

fn toilet_paper_checksum(
    mut checksum: ResMut<SimChecksumState>,
    items: Query<(
        &ToiletPaper,
        &ToiletPaperScrap,
        &SimPosition,
        &ToiletPaperHeldBy,
        &ToiletPaperViewState,
    )>,
) {
    accumulate_str(&mut checksum, 0x1000, TOILET_PAPER_ID);
    accumulate_str(&mut checksum, 0x1001, TOILET_PAPER_NAME);
    accumulate_str(&mut checksum, 0x1002, TOILET_PAPER_TYPE);
    accumulate_str(&mut checksum, 0x1003, TOILET_PAPER_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, TOILET_PAPER_EFFECTS);

    checksum.accumulate(TOILET_PAPER_SOURCE_REVISION as u64);
    checksum.accumulate(TOILET_PAPER_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(TOILET_PAPER_WEIGHT.to_bits() as u64);
    checksum.accumulate(TOILET_PAPER_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(TOILET_PAPER_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(TOILET_PAPER_CONDUCTIVE as u64);
    checksum.accumulate(TOILET_PAPER_TWO_HANDED as u64);

    for dependency in TOILET_PAPER_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for spawn_chance in TOILET_PAPER_SPAWN_CHANCES {
        accumulate_str(&mut checksum, 0x4000, spawn_chance.moon);
        checksum.accumulate(spawn_chance.chance.to_bits() as u64);
    }

    for rule in TOILET_PAPER_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x5000, rule.condition);
        accumulate_str(&mut checksum, 0x5001, rule.outcome);
    }

    for (toilet_paper, scrap, position, held_by, view_state) in &items {
        checksum.accumulate(toilet_paper.stable_id);
        checksum.accumulate(scrap.min_value.to_bits() as u64);
        checksum.accumulate(scrap.max_value.to_bits() as u64);
        checksum.accumulate(scrap.weight.to_bits() as u64);
        checksum.accumulate(scrap.conductive as u64);
        checksum.accumulate(scrap.two_handed as u64);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(held_by.employee_id);
        checksum.accumulate(held_by.is_held as u64);
        checksum.accumulate(view_state.view_obstructed as u64);
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt);
    for byte in value.as_bytes() {
        checksum.accumulate(*byte as u64);
    }
}
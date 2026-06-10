// Sources: vault/scrap_items/control_pad.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::ItemBarPickupEvent;
use crate::sim::{SimChecksumState, SimPosition};

pub const CONTROL_PAD_ID: &str = "control_pad";
pub const CONTROL_PAD_NAME: &str = "Control Pad";
pub const CONTROL_PAD_TYPE: &str = "scrap_items";
pub const CONTROL_PAD_SUBTYPE: &str = "scrap";
pub const CONTROL_PAD_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Control_Pad";
pub const CONTROL_PAD_SOURCE_REVISION: u32 = 20196;
pub const CONTROL_PAD_EXTRACTED_AT: &str = "2026-06-07T00:00:00Z";
pub const CONTROL_PAD_CONFIDENCE_BASIS_POINTS: u16 = 98;

pub const CONTROL_PAD_EFFECTS: &str = "No use.";
pub const CONTROL_PAD_WEIGHT: I32F32 = I32F32::lit("16");
pub const CONTROL_PAD_CONDUCTIVE: bool = true;
pub const CONTROL_PAD_TWO_HANDED: bool = true;
pub const CONTROL_PAD_MIN_VALUE: I32F32 = I32F32::lit("34");
pub const CONTROL_PAD_MAX_VALUE: I32F32 = I32F32::lit("63");
pub const CONTROL_PAD_PAGE_ID: u32 = 72;

pub const CONTROL_PAD_DEPENDS_ON: [&str; 8] = [
    "the_company",
    "credits",
    "offense",
    "rend",
    "assurance",
    "adamance",
    "embrion",
    "march",
];

pub const CONTROL_PAD_SPAWN_CHANCES: [ControlPadSpawnChance; 7] = [
    ControlPadSpawnChance {
        moon: "offense",
        chance: I32F32::lit("4.8"),
    },
    ControlPadSpawnChance {
        moon: "rend",
        chance: I32F32::lit("3.06"),
    },
    ControlPadSpawnChance {
        moon: "assurance",
        chance: I32F32::lit("1.76"),
    },
    ControlPadSpawnChance {
        moon: "adamance",
        chance: I32F32::lit("1.69"),
    },
    ControlPadSpawnChance {
        moon: "embrion",
        chance: I32F32::lit("1.62"),
    },
    ControlPadSpawnChance {
        moon: "march",
        chance: I32F32::lit("1.26"),
    },
    ControlPadSpawnChance {
        moon: "vow",
        chance: I32F32::lit("1.23"),
    },
];

pub const CONTROL_PAD_BEHAVIORAL_MECHANICS: [ControlPadBehaviorRule; 8] = [
    ControlPadBehaviorRule {
        condition: "sold",
        outcome: "it can be exchanged with the_company for credits",
    },
    ControlPadBehaviorRule {
        condition: "spawned on offense",
        outcome: "the spawn chance is 4.8%",
    },
    ControlPadBehaviorRule {
        condition: "spawned on rend",
        outcome: "the spawn chance is 3.06%",
    },
    ControlPadBehaviorRule {
        condition: "spawned on assurance",
        outcome: "the spawn chance is 1.76%",
    },
    ControlPadBehaviorRule {
        condition: "spawned on adamance",
        outcome: "the spawn chance is 1.69%",
    },
    ControlPadBehaviorRule {
        condition: "spawned on embrion",
        outcome: "the spawn chance is 1.62%",
    },
    ControlPadBehaviorRule {
        condition: "spawned on march",
        outcome: "the spawn chance is 1.26%",
    },
    ControlPadBehaviorRule {
        condition: "spawned on vow",
        outcome: "the spawn chance is 1.23%",
    },
];

pub struct ControlPadPlugin;

impl Plugin for ControlPadPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnControlPadEvent>()
            .add_event::<ControlPadSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_control_pad,
                    control_pad_pickup_item_bar_bridge,
                    control_pad_sell_bridge,
                    control_pad_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ControlPadBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ControlPadSpawnChance {
    pub moon: &'static str,
    pub chance: I32F32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ControlPad {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ControlPadScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for ControlPadScrap {
    fn default() -> Self {
        Self {
            min_value: CONTROL_PAD_MIN_VALUE,
            max_value: CONTROL_PAD_MAX_VALUE,
            weight: CONTROL_PAD_WEIGHT,
            conductive: CONTROL_PAD_CONDUCTIVE,
            two_handed: CONTROL_PAD_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ControlPadHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Bundle)]
pub struct ControlPadBundle {
    pub name: Name,
    pub control_pad: ControlPad,
    pub scrap: ControlPadScrap,
    pub position: SimPosition,
    pub held_by: ControlPadHeldBy,
}

impl ControlPadBundle {
    pub fn new(event: SpawnControlPadEvent) -> Self {
        Self {
            name: Name::new(CONTROL_PAD_NAME),
            control_pad: ControlPad {
                stable_id: event.stable_id,
            },
            scrap: ControlPadScrap::default(),
            position: event.position,
            held_by: ControlPadHeldBy::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnControlPadEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControlPadSoldEvent {
    pub control_pad_stable_id: u64,
    pub credit_value: I32F32,
}

pub fn control_pad_value_range() -> (I32F32, I32F32) {
    (CONTROL_PAD_MIN_VALUE, CONTROL_PAD_MAX_VALUE)
}

pub fn control_pad_spawn_chance_for_moon(moon: &str) -> Option<I32F32> {
    CONTROL_PAD_SPAWN_CHANCES
        .iter()
        .find(|spawn_chance| spawn_chance.moon == moon)
        .map(|spawn_chance| spawn_chance.chance)
}

fn spawn_control_pad(mut commands: Commands, mut events: EventReader<SpawnControlPadEvent>) {
    for event in events.read() {
        commands.spawn(ControlPadBundle::new(*event));
    }
}

fn control_pad_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    control_pads: Query<(&ControlPad, &ControlPadHeldBy), Changed<ControlPadHeldBy>>,
) {
    for (control_pad, held_by) in &control_pads {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: CONTROL_PAD_ID,
                two_handed: CONTROL_PAD_TWO_HANDED,
                functional: false,
                passive: false,
                from_store_or_valueless: false,
            });
        } else {
            let _ = control_pad.stable_id;
        }
    }
}

fn control_pad_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<ControlPadSoldEvent>,
    control_pads: Query<(&ControlPad, &ControlPadScrap)>,
) {
    for event in sell_events.read() {
        for (control_pad, scrap) in &control_pads {
            if control_pad.stable_id != event.scrap_entity_id {
                continue;
            }

            sold_events.send(ControlPadSoldEvent {
                control_pad_stable_id: control_pad.stable_id,
                credit_value: scrap.max_value,
            });
        }
    }
}

fn control_pad_checksum(
    mut checksum: ResMut<SimChecksumState>,
    control_pads: Query<(&ControlPad, &ControlPadScrap, &SimPosition, &ControlPadHeldBy)>,
) {
    accumulate_str(&mut checksum, 0x1000, CONTROL_PAD_ID);
    accumulate_str(&mut checksum, 0x1001, CONTROL_PAD_NAME);
    accumulate_str(&mut checksum, 0x1002, CONTROL_PAD_TYPE);
    accumulate_str(&mut checksum, 0x1003, CONTROL_PAD_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, CONTROL_PAD_EFFECTS);
    accumulate_str(&mut checksum, 0x1005, CONTROL_PAD_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1006, CONTROL_PAD_EXTRACTED_AT);

    checksum.accumulate(CONTROL_PAD_SOURCE_REVISION as u64);
    checksum.accumulate(CONTROL_PAD_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(CONTROL_PAD_WEIGHT.to_bits() as u64);
    checksum.accumulate(CONTROL_PAD_CONDUCTIVE as u64);
    checksum.accumulate(CONTROL_PAD_TWO_HANDED as u64);
    checksum.accumulate(CONTROL_PAD_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(CONTROL_PAD_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(CONTROL_PAD_PAGE_ID as u64);

    for dependency in CONTROL_PAD_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for spawn_chance in CONTROL_PAD_SPAWN_CHANCES {
        accumulate_str(&mut checksum, 0x3000, spawn_chance.moon);
        checksum.accumulate(spawn_chance.chance.to_bits() as u64);
    }

    for rule in CONTROL_PAD_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x4000, rule.condition);
        accumulate_str(&mut checksum, 0x4001, rule.outcome);
    }

    for (control_pad, scrap, position, held_by) in &control_pads {
        checksum.accumulate(control_pad.stable_id);
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
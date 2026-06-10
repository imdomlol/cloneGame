// Sources: vault/scrap_items/dust_pan.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::ItemBarPickupEvent;
use crate::sim::{SimChecksumState, SimPosition};

pub const DUST_PAN_ID: &str = "dust_pan";
pub const DUST_PAN_NAME: &str = "Dust pan";
pub const DUST_PAN_TYPE: &str = "scrap_items";
pub const DUST_PAN_SUBTYPE: &str = "scrap";
pub const DUST_PAN_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Dust_Pan";
pub const DUST_PAN_SOURCE_REVISION: u32 = 20188;
pub const DUST_PAN_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const DUST_PAN_CONFIDENCE_BASIS_POINTS: u16 = 94;

pub const DUST_PAN_WEIGHT: I32F32 = I32F32::lit("0");
pub const DUST_PAN_CONDUCTIVE: bool = false;
pub const DUST_PAN_TWO_HANDED: bool = false;
pub const DUST_PAN_MIN_VALUE: I32F32 = I32F32::lit("12");
pub const DUST_PAN_MAX_VALUE: I32F32 = I32F32::lit("31");

pub const DUST_PAN_DEPENDS_ON: [&str; 5] = [
    "scrap",
    "lethal_company",
    "experimentation",
    "the_company",
    "credits",
];

pub const DUST_PAN_SPAWN_CHANCES: [DustPanSpawnChance; 1] = [DustPanSpawnChance {
    moon: "experimentation",
    chance: I32F32::lit("5.62"),
}];

pub const DUST_PAN_BEHAVIORAL_MECHANICS: [DustPanBehaviorRule; 3] = [
    DustPanBehaviorRule {
        condition: "the item spawns",
        outcome: "it can appear only in experimentation with a spawn chance of 5.62%",
    },
    DustPanBehaviorRule {
        condition: "the item is carried",
        outcome: "its weight is 0, it is non-conductive, and it is not two-handed",
    },
    DustPanBehaviorRule {
        condition: "the item is sold",
        outcome: "the listed value is 12 to 31 credits",
    },
];

pub struct DustPanPlugin;

impl Plugin for DustPanPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnDustPanEvent>()
            .add_event::<DustPanSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_dust_pan,
                    dust_pan_pickup_item_bar_bridge,
                    dust_pan_sell_bridge,
                    dust_pan_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DustPanBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DustPanSpawnChance {
    pub moon: &'static str,
    pub chance: I32F32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DustPan {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DustPanScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for DustPanScrap {
    fn default() -> Self {
        Self {
            min_value: DUST_PAN_MIN_VALUE,
            max_value: DUST_PAN_MAX_VALUE,
            weight: DUST_PAN_WEIGHT,
            conductive: DUST_PAN_CONDUCTIVE,
            two_handed: DUST_PAN_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DustPanHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Bundle)]
pub struct DustPanBundle {
    pub name: Name,
    pub dust_pan: DustPan,
    pub scrap: DustPanScrap,
    pub position: SimPosition,
    pub held_by: DustPanHeldBy,
}

impl DustPanBundle {
    pub fn new(event: SpawnDustPanEvent) -> Self {
        Self {
            name: Name::new(DUST_PAN_NAME),
            dust_pan: DustPan {
                stable_id: event.stable_id,
            },
            scrap: DustPanScrap::default(),
            position: event.position,
            held_by: DustPanHeldBy::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnDustPanEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DustPanSoldEvent {
    pub dust_pan_stable_id: u64,
    pub credit_value: I32F32,
}

pub fn dust_pan_value_range() -> (I32F32, I32F32) {
    (DUST_PAN_MIN_VALUE, DUST_PAN_MAX_VALUE)
}

pub fn dust_pan_spawn_chance_for_moon(moon: &str) -> Option<I32F32> {
    DUST_PAN_SPAWN_CHANCES
        .iter()
        .find(|spawn_chance| spawn_chance.moon == moon)
        .map(|spawn_chance| spawn_chance.chance)
}

fn spawn_dust_pan(mut commands: Commands, mut events: EventReader<SpawnDustPanEvent>) {
    for event in events.read() {
        commands.spawn(DustPanBundle::new(*event));
    }
}

fn dust_pan_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    dust_pans: Query<&DustPanHeldBy, (With<DustPan>, Changed<DustPanHeldBy>)>,
) {
    for held_by in &dust_pans {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: DUST_PAN_ID,
                two_handed: DUST_PAN_TWO_HANDED,
                functional: false,
                passive: true,
                from_store_or_valueless: false,
            });
        }
    }
}

fn dust_pan_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<DustPanSoldEvent>,
    dust_pans: Query<(&DustPan, &DustPanScrap)>,
) {
    for event in sell_events.read() {
        for (dust_pan, scrap) in &dust_pans {
            if dust_pan.stable_id != event.scrap_entity_id {
                continue;
            }

            sold_events.send(DustPanSoldEvent {
                dust_pan_stable_id: dust_pan.stable_id,
                credit_value: scrap.max_value,
            });
        }
    }
}

fn dust_pan_checksum(
    mut checksum: ResMut<SimChecksumState>,
    dust_pans: Query<(&DustPan, &DustPanScrap, &SimPosition, &DustPanHeldBy)>,
) {
    accumulate_str(&mut checksum, 0x1000, DUST_PAN_ID);
    accumulate_str(&mut checksum, 0x1001, DUST_PAN_NAME);
    accumulate_str(&mut checksum, 0x1002, DUST_PAN_TYPE);
    accumulate_str(&mut checksum, 0x1003, DUST_PAN_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, DUST_PAN_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1005, DUST_PAN_EXTRACTED_AT);

    checksum.accumulate(DUST_PAN_SOURCE_REVISION as u64);
    checksum.accumulate(DUST_PAN_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(DUST_PAN_WEIGHT.to_bits() as u64);
    checksum.accumulate(DUST_PAN_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(DUST_PAN_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(DUST_PAN_CONDUCTIVE as u64);
    checksum.accumulate(DUST_PAN_TWO_HANDED as u64);

    for dependency in DUST_PAN_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for spawn_chance in DUST_PAN_SPAWN_CHANCES {
        accumulate_str(&mut checksum, 0x3000, spawn_chance.moon);
        checksum.accumulate(spawn_chance.chance.to_bits() as u64);
    }

    for rule in DUST_PAN_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x4000, rule.condition);
        accumulate_str(&mut checksum, 0x4001, rule.outcome);
    }

    for (dust_pan, scrap, position, held_by) in &dust_pans {
        checksum.accumulate(dust_pan.stable_id);
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
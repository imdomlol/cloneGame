// Sources: vault/scrap_items/wedding_ring.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::ItemBarPickupEvent;
use crate::sim::{SimChecksumState, SimPosition};

pub const WEDDING_RING_ID: &str = "wedding_ring";
pub const WEDDING_RING_NAME: &str = "Wedding ring";
pub const WEDDING_RING_TYPE: &str = "scrap_items";
pub const WEDDING_RING_SUBTYPE: &str = "scrap";
pub const WEDDING_RING_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Ring";
pub const WEDDING_RING_SOURCE_REVISION: u32 = 20962;
pub const WEDDING_RING_EXTRACTED_AT: &str = "2026-06-07T00:00:00Z";
pub const WEDDING_RING_CONFIDENCE_BASIS_POINTS: u16 = 93;

pub const WEDDING_RING_EFFECTS: &str =
    "No functional use; can be sold to the_company for credits.";
pub const WEDDING_RING_WEIGHT: I32F32 = I32F32::lit("16");
pub const WEDDING_RING_CONDUCTIVE: bool = true;
pub const WEDDING_RING_MIN_VALUE: I32F32 = I32F32::lit("52");
pub const WEDDING_RING_MAX_VALUE: I32F32 = I32F32::lit("79");
pub const WEDDING_RING_TWO_HANDED: bool = false;

pub const WEDDING_RING_DEPENDS_ON: [&str; 1] = ["the_company"];

pub const WEDDING_RING_BEHAVIORAL_MECHANICS: [WeddingRingBehaviorRule; 2] = [
    WeddingRingBehaviorRule {
        condition: "collected",
        outcome: "it provides no gameplay effect",
    },
    WeddingRingBehaviorRule {
        condition: "sold to the_company",
        outcome: "it converts into credits",
    },
];

pub struct WeddingRingPlugin;

impl Plugin for WeddingRingPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnWeddingRingEvent>()
            .add_event::<WeddingRingSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_wedding_ring,
                    wedding_ring_pickup_item_bar_bridge,
                    wedding_ring_sell_bridge,
                    wedding_ring_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WeddingRingBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WeddingRing {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct WeddingRingScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for WeddingRingScrap {
    fn default() -> Self {
        Self {
            min_value: WEDDING_RING_MIN_VALUE,
            max_value: WEDDING_RING_MAX_VALUE,
            weight: WEDDING_RING_WEIGHT,
            conductive: WEDDING_RING_CONDUCTIVE,
            two_handed: WEDDING_RING_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WeddingRingHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Bundle)]
pub struct WeddingRingBundle {
    pub name: Name,
    pub wedding_ring: WeddingRing,
    pub scrap: WeddingRingScrap,
    pub position: SimPosition,
    pub held_by: WeddingRingHeldBy,
}

impl WeddingRingBundle {
    pub fn new(event: SpawnWeddingRingEvent) -> Self {
        Self {
            name: Name::new(WEDDING_RING_NAME),
            wedding_ring: WeddingRing {
                stable_id: event.stable_id,
            },
            scrap: WeddingRingScrap::default(),
            position: event.position,
            held_by: WeddingRingHeldBy::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnWeddingRingEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeddingRingSoldEvent {
    pub wedding_ring_stable_id: u64,
    pub credit_value: I32F32,
}

pub fn wedding_ring_value_range() -> (I32F32, I32F32) {
    (WEDDING_RING_MIN_VALUE, WEDDING_RING_MAX_VALUE)
}

fn spawn_wedding_ring(
    mut commands: Commands,
    mut events: EventReader<SpawnWeddingRingEvent>,
) {
    for event in events.read() {
        commands.spawn(WeddingRingBundle::new(*event));
    }
}

fn wedding_ring_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    rings: Query<(&WeddingRing, &WeddingRingHeldBy), Changed<WeddingRingHeldBy>>,
) {
    for (ring, held_by) in &rings {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: WEDDING_RING_ID,
                two_handed: WEDDING_RING_TWO_HANDED,
                functional: false,
                passive: true,
                from_store_or_valueless: false,
            });
        } else {
            let _ = ring.stable_id;
        }
    }
}

fn wedding_ring_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<WeddingRingSoldEvent>,
    rings: Query<(&WeddingRing, &WeddingRingScrap)>,
) {
    for event in sell_events.read() {
        for (ring, scrap) in &rings {
            if ring.stable_id != event.scrap_entity_id {
                continue;
            }
            sold_events.send(WeddingRingSoldEvent {
                wedding_ring_stable_id: ring.stable_id,
                credit_value: scrap.max_value,
            });
        }
    }
}

fn wedding_ring_checksum(
    mut checksum: ResMut<SimChecksumState>,
    rings: Query<(&WeddingRing, &WeddingRingScrap, &SimPosition, &WeddingRingHeldBy)>,
) {
    accumulate_str(&mut checksum, 0x1000, WEDDING_RING_ID);
    accumulate_str(&mut checksum, 0x1001, WEDDING_RING_NAME);
    accumulate_str(&mut checksum, 0x1002, WEDDING_RING_TYPE);
    accumulate_str(&mut checksum, 0x1003, WEDDING_RING_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, WEDDING_RING_EFFECTS);

    checksum.accumulate(WEDDING_RING_SOURCE_REVISION as u64);
    checksum.accumulate(WEDDING_RING_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(WEDDING_RING_WEIGHT.to_bits() as u64);
    checksum.accumulate(WEDDING_RING_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(WEDDING_RING_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(WEDDING_RING_CONDUCTIVE as u64);
    checksum.accumulate(WEDDING_RING_TWO_HANDED as u64);

    for dependency in WEDDING_RING_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for rule in WEDDING_RING_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x5000, rule.condition);
        accumulate_str(&mut checksum, 0x5001, rule.outcome);
    }

    for (ring, scrap, position, held_by) in &rings {
        checksum.accumulate(ring.stable_id);
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
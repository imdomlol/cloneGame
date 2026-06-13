// Sources: vault/scrap_items/painting.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::ItemBarPickupEvent;
use crate::sim::{SimChecksumState, SimPosition};

pub const PAINTING_ID: &str = "painting";
pub const PAINTING_NAME: &str = "Painting";
pub const PAINTING_TYPE: &str = "scrap_items";
pub const PAINTING_SUBTYPE: &str = "painting";
pub const PAINTING_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Painting";
pub const PAINTING_SOURCE_REVISION: u32 = 20344;
pub const PAINTING_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const PAINTING_CONFIDENCE_BASIS_POINTS: u16 = 98;

pub const PAINTING_DESCRIPTION: &str = "The Painting is a wooden-framed scrap item with two visual variants: one shows a river and forest, and the other shows clouds in the sky. It has no functional use beyond being sold.";
pub const PAINTING_WEIGHT: I32F32 = I32F32::lit("31");
pub const PAINTING_CONDUCTIVE: bool = false;
pub const PAINTING_TWO_HANDED: bool = true;
pub const PAINTING_MIN_VALUE: I32F32 = I32F32::lit("60");
pub const PAINTING_MAX_VALUE: I32F32 = I32F32::lit("123");
pub const PAINTING_PAGE_ID: u32 = 33;

pub const PAINTING_DEPENDS_ON: [&str; 5] = ["the_company", "credits", "rend", "artifice", "titan"];

pub const PAINTING_VISUAL_VARIANTS: [&str; 2] = ["river_and_forest", "clouds_in_the_sky"];

pub const PAINTING_SPAWN_CHANCES: [PaintingSpawnChance; 3] = [
    PaintingSpawnChance {
        moon: "rend",
        chance: I32F32::lit("5.19"),
    },
    PaintingSpawnChance {
        moon: "artifice",
        chance: I32F32::lit("4.58"),
    },
    PaintingSpawnChance {
        moon: "titan",
        chance: I32F32::lit("3.66"),
    },
];

pub const PAINTING_BEHAVIORAL_MECHANICS: [PaintingBehaviorRule; 7] = [
    PaintingBehaviorRule {
        condition: "carried",
        outcome: "it has a weight of 31",
    },
    PaintingBehaviorRule {
        condition: "handled",
        outcome: "it is not conductive",
    },
    PaintingBehaviorRule {
        condition: "equipped",
        outcome: "it requires two hands",
    },
    PaintingBehaviorRule {
        condition: "sold to the_company",
        outcome: "it can be exchanged for credits with a value between 60 and 123 credits",
    },
    PaintingBehaviorRule {
        condition: "spawn on rend",
        outcome: "the spawn chance is 5.19%",
    },
    PaintingBehaviorRule {
        condition: "spawn on artifice",
        outcome: "the spawn chance is 4.58%",
    },
    PaintingBehaviorRule {
        condition: "spawn on titan",
        outcome: "the spawn chance is 3.66%",
    },
];

pub struct PaintingPlugin;

impl Plugin for PaintingPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnPaintingEvent>()
            .add_event::<PaintingSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_painting,
                    painting_pickup_item_bar_bridge,
                    painting_sell_bridge,
                    painting_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PaintingBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PaintingSpawnChance {
    pub moon: &'static str,
    pub chance: I32F32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Painting {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PaintingScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for PaintingScrap {
    fn default() -> Self {
        Self {
            min_value: PAINTING_MIN_VALUE,
            max_value: PAINTING_MAX_VALUE,
            weight: PAINTING_WEIGHT,
            conductive: PAINTING_CONDUCTIVE,
            two_handed: PAINTING_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PaintingHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PaintingVisualState {
    pub variant_index: u8,
}

#[derive(Bundle)]
pub struct PaintingBundle {
    pub name: Name,
    pub painting: Painting,
    pub scrap: PaintingScrap,
    pub position: SimPosition,
    pub held_by: PaintingHeldBy,
    pub visual_state: PaintingVisualState,
}

impl PaintingBundle {
    pub fn new(event: SpawnPaintingEvent) -> Self {
        Self {
            name: Name::new(PAINTING_NAME),
            painting: Painting {
                stable_id: event.stable_id,
            },
            scrap: PaintingScrap::default(),
            position: event.position,
            held_by: PaintingHeldBy::default(),
            visual_state: PaintingVisualState {
                variant_index: event.variant_index,
            },
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnPaintingEvent {
    pub stable_id: u64,
    pub position: SimPosition,
    pub variant_index: u8,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaintingSoldEvent {
    pub painting_stable_id: u64,
    pub credit_value: I32F32,
}

pub fn painting_value_range() -> (I32F32, I32F32) {
    (PAINTING_MIN_VALUE, PAINTING_MAX_VALUE)
}

pub fn painting_spawn_chance_for_moon(moon: &str) -> Option<I32F32> {
    PAINTING_SPAWN_CHANCES
        .iter()
        .find(|spawn_chance| spawn_chance.moon == moon)
        .map(|spawn_chance| spawn_chance.chance)
}

pub fn painting_visual_variant_name(variant_index: u8) -> Option<&'static str> {
    PAINTING_VISUAL_VARIANTS
        .get(variant_index as usize)
        .copied()
}

fn spawn_painting(mut commands: Commands, mut events: EventReader<SpawnPaintingEvent>) {
    for event in events.read() {
        commands.spawn(PaintingBundle::new(*event));
    }
}

fn painting_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    paintings: Query<(&Painting, &PaintingHeldBy), Changed<PaintingHeldBy>>,
) {
    for (painting, held_by) in &paintings {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: PAINTING_ID,
                two_handed: PAINTING_TWO_HANDED,
                functional: false,
                passive: false,
                from_store_or_valueless: false,
            });
        } else {
            let _ = painting.stable_id;
        }
    }
}

fn painting_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<PaintingSoldEvent>,
    paintings: Query<(&Painting, &PaintingScrap)>,
) {
    for event in sell_events.read() {
        for (painting, scrap) in &paintings {
            if painting.stable_id != event.scrap_entity_id {
                continue;
            }

            sold_events.send(PaintingSoldEvent {
                painting_stable_id: painting.stable_id,
                credit_value: scrap.max_value,
            });
        }
    }
}

fn painting_checksum(
    mut checksum: ResMut<SimChecksumState>,
    paintings: Query<(&Painting, &PaintingScrap, &SimPosition, &PaintingHeldBy, &PaintingVisualState)>,
) {
    accumulate_str(&mut checksum, 0x1000, PAINTING_ID);
    accumulate_str(&mut checksum, 0x1001, PAINTING_NAME);
    accumulate_str(&mut checksum, 0x1002, PAINTING_TYPE);
    accumulate_str(&mut checksum, 0x1003, PAINTING_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, PAINTING_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1005, PAINTING_EXTRACTED_AT);
    accumulate_str(&mut checksum, 0x1006, PAINTING_DESCRIPTION);

    checksum.accumulate(PAINTING_SOURCE_REVISION as u64);
    checksum.accumulate(PAINTING_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(PAINTING_WEIGHT.to_bits() as u64);
    checksum.accumulate(PAINTING_CONDUCTIVE as u64);
    checksum.accumulate(PAINTING_TWO_HANDED as u64);
    checksum.accumulate(PAINTING_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(PAINTING_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(PAINTING_PAGE_ID as u64);

    for dependency in PAINTING_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for visual_variant in PAINTING_VISUAL_VARIANTS {
        accumulate_str(&mut checksum, 0x3000, visual_variant);
    }

    for spawn_chance in PAINTING_SPAWN_CHANCES {
        accumulate_str(&mut checksum, 0x4000, spawn_chance.moon);
        checksum.accumulate(spawn_chance.chance.to_bits() as u64);
    }

    for rule in PAINTING_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x5000, rule.condition);
        accumulate_str(&mut checksum, 0x5001, rule.outcome);
    }

    for (painting, scrap, position, held_by, visual_state) in &paintings {
        checksum.accumulate(painting.stable_id);
        checksum.accumulate(scrap.min_value.to_bits() as u64);
        checksum.accumulate(scrap.max_value.to_bits() as u64);
        checksum.accumulate(scrap.weight.to_bits() as u64);
        checksum.accumulate(scrap.conductive as u64);
        checksum.accumulate(scrap.two_handed as u64);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(held_by.employee_id);
        checksum.accumulate(held_by.is_held as u64);
        checksum.accumulate(visual_state.variant_index as u64);
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt);
    for byte in value.as_bytes() {
        checksum.accumulate(*byte as u64);
    }
}
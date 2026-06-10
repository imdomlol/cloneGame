// Sources: vault/scrap_items/cookie_pan.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::ItemBarPickupEvent;
use crate::sim::{SimChecksumState, SimPosition};

pub const COOKIE_PAN_ID: &str = "cookie_pan";
pub const COOKIE_PAN_NAME: &str = "Cookie Pan";
pub const COOKIE_PAN_TYPE: &str = "scrap_items";
pub const COOKIE_PAN_SUBTYPE: &str = "scrap";
pub const COOKIE_PAN_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Cookie_Pan";
pub const COOKIE_PAN_SOURCE_REVISION: u32 = 20198;
pub const COOKIE_PAN_EXTRACTED_AT: &str = "2026-06-07T00:00:00Z";
pub const COOKIE_PAN_CONFIDENCE_BASIS_POINTS: u16 = 84;

pub const COOKIE_PAN_WEIGHT: I32F32 = I32F32::lit("16");
pub const COOKIE_PAN_CONDUCTIVE: bool = true;
pub const COOKIE_PAN_MIN_VALUE: I32F32 = I32F32::lit("12");
pub const COOKIE_PAN_MAX_VALUE: I32F32 = I32F32::lit("39");
pub const COOKIE_PAN_TWO_HANDED: bool = false;

pub const COOKIE_PAN_DEPENDS_ON: [&str; 1] = ["the_company"];

pub const COOKIE_PAN_SPAWN_CHANCES: [CookiePanSpawnChance; 7] = [
    CookiePanSpawnChance {
        moon: "embrion",
        chance: I32F32::lit("6.02"),
    },
    CookiePanSpawnChance {
        moon: "assurance",
        chance: I32F32::lit("5.76"),
    },
    CookiePanSpawnChance {
        moon: "vow",
        chance: I32F32::lit("5.04"),
    },
    CookiePanSpawnChance {
        moon: "adamance",
        chance: I32F32::lit("4.22"),
    },
    CookiePanSpawnChance {
        moon: "march",
        chance: I32F32::lit("2.74"),
    },
    CookiePanSpawnChance {
        moon: "experimentation",
        chance: I32F32::lit("0.88"),
    },
    CookiePanSpawnChance {
        moon: "offense",
        chance: I32F32::lit("0.72"),
    },
];

pub const COOKIE_PAN_BEHAVIORAL_MECHANICS: [CookiePanBehaviorRule; 5] = [
    CookiePanBehaviorRule {
        condition: "sold to the_company",
        outcome: "it is treated as scrap with a value range of min: 12 to max: 39",
    },
    CookiePanBehaviorRule {
        condition: "checked for conductivity",
        outcome: "conductive: true",
    },
    CookiePanBehaviorRule {
        condition: "carried as an item",
        outcome: "weight: 16",
    },
    CookiePanBehaviorRule {
        condition: "checked for hand use",
        outcome: "two_handed: false",
    },
    CookiePanBehaviorRule {
        condition: "spawn rates are needed",
        outcome: "use the moon-specific chances in the frontmatter's spawn_chance_by_moon field",
    },
];

pub struct CookiePanPlugin;

impl Plugin for CookiePanPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnCookiePanEvent>()
            .add_event::<CookiePanSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_cookie_pan,
                    cookie_pan_pickup_item_bar_bridge,
                    cookie_pan_sell_bridge,
                    cookie_pan_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CookiePanBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CookiePanSpawnChance {
    pub moon: &'static str,
    pub chance: I32F32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CookiePan {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CookiePanScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for CookiePanScrap {
    fn default() -> Self {
        Self {
            min_value: COOKIE_PAN_MIN_VALUE,
            max_value: COOKIE_PAN_MAX_VALUE,
            weight: COOKIE_PAN_WEIGHT,
            conductive: COOKIE_PAN_CONDUCTIVE,
            two_handed: COOKIE_PAN_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CookiePanHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Bundle)]
pub struct CookiePanBundle {
    pub name: Name,
    pub cookie_pan: CookiePan,
    pub scrap: CookiePanScrap,
    pub position: SimPosition,
    pub held_by: CookiePanHeldBy,
}

impl CookiePanBundle {
    pub fn new(event: SpawnCookiePanEvent) -> Self {
        Self {
            name: Name::new(COOKIE_PAN_NAME),
            cookie_pan: CookiePan {
                stable_id: event.stable_id,
            },
            scrap: CookiePanScrap::default(),
            position: event.position,
            held_by: CookiePanHeldBy::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnCookiePanEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CookiePanSoldEvent {
    pub cookie_pan_stable_id: u64,
    pub credit_value: I32F32,
}

pub fn cookie_pan_value_range() -> (I32F32, I32F32) {
    (COOKIE_PAN_MIN_VALUE, COOKIE_PAN_MAX_VALUE)
}

pub fn cookie_pan_spawn_chance_for_moon(moon: &str) -> Option<I32F32> {
    COOKIE_PAN_SPAWN_CHANCES
        .iter()
        .find(|spawn_chance| spawn_chance.moon == moon)
        .map(|spawn_chance| spawn_chance.chance)
}

fn spawn_cookie_pan(mut commands: Commands, mut events: EventReader<SpawnCookiePanEvent>) {
    for event in events.read() {
        commands.spawn(CookiePanBundle::new(*event));
    }
}

fn cookie_pan_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    cookie_pans: Query<(&CookiePan, &CookiePanHeldBy), Changed<CookiePanHeldBy>>,
) {
    for (cookie_pan, held_by) in &cookie_pans {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: COOKIE_PAN_ID,
                two_handed: COOKIE_PAN_TWO_HANDED,
                functional: false,
                passive: false,
                from_store_or_valueless: false,
            });
        } else {
            let _ = cookie_pan.stable_id;
        }
    }
}

fn cookie_pan_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<CookiePanSoldEvent>,
    cookie_pans: Query<(&CookiePan, &CookiePanScrap)>,
) {
    for event in sell_events.read() {
        for (cookie_pan, scrap) in &cookie_pans {
            if cookie_pan.stable_id != event.scrap_entity_id {
                continue;
            }

            sold_events.send(CookiePanSoldEvent {
                cookie_pan_stable_id: cookie_pan.stable_id,
                credit_value: scrap.max_value,
            });
        }
    }
}

fn cookie_pan_checksum(
    mut checksum: ResMut<SimChecksumState>,
    cookie_pans: Query<(&CookiePan, &CookiePanScrap, &SimPosition, &CookiePanHeldBy)>,
) {
    accumulate_str(&mut checksum, 0x1000, COOKIE_PAN_ID);
    accumulate_str(&mut checksum, 0x1001, COOKIE_PAN_NAME);
    accumulate_str(&mut checksum, 0x1002, COOKIE_PAN_TYPE);
    accumulate_str(&mut checksum, 0x1003, COOKIE_PAN_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, COOKIE_PAN_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1005, COOKIE_PAN_EXTRACTED_AT);

    checksum.accumulate(COOKIE_PAN_SOURCE_REVISION as u64);
    checksum.accumulate(COOKIE_PAN_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(COOKIE_PAN_WEIGHT.to_bits() as u64);
    checksum.accumulate(COOKIE_PAN_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(COOKIE_PAN_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(COOKIE_PAN_CONDUCTIVE as u64);
    checksum.accumulate(COOKIE_PAN_TWO_HANDED as u64);

    for dependency in COOKIE_PAN_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for spawn_chance in COOKIE_PAN_SPAWN_CHANCES {
        accumulate_str(&mut checksum, 0x3000, spawn_chance.moon);
        checksum.accumulate(spawn_chance.chance.to_bits() as u64);
    }

    for rule in COOKIE_PAN_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x4000, rule.condition);
        accumulate_str(&mut checksum, 0x4001, rule.outcome);
    }

    for (cookie_pan, scrap, position, held_by) in &cookie_pans {
        checksum.accumulate(cookie_pan.stable_id);
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
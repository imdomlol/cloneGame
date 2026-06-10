// Sources: vault/scrap_items/bee_hive.md, vault/item_index_pages/scrap.md
use bevy::prelude::*;
use fixed::types::I32F32;
use rand_core::RngCore;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::ItemBarPickupEvent;
use crate::sim::{tick_rng, GameSeed, SimChecksumState, SimPosition};

pub const BEE_HIVE_ID: &str = "bee_hive";
pub const BEE_HIVE_NAME: &str = "Bee hive";
pub const BEE_HIVE_TYPE: &str = "scrap_items";
pub const BEE_HIVE_SUBTYPE: &str = "special_scrap";
pub const BEE_HIVE_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Bee_Hive";
pub const BEE_HIVE_SOURCE_REVISION: u32 = 21210;
pub const BEE_HIVE_EXTRACTED_AT: &str = "2026-06-07T00:00:00Z";
pub const BEE_HIVE_CONFIDENCE_BASIS_POINTS: u16 = 96;

pub const BEE_HIVE_EFFECTS: &str = "Houses Circuit Bees";
pub const BEE_HIVE_WEIGHT: I32F32 = I32F32::lit("0");
pub const BEE_HIVE_CONDUCTIVE: bool = true;
pub const BEE_HIVE_MIN_VALUE: I32F32 = I32F32::lit("40");
pub const BEE_HIVE_MAX_VALUE: I32F32 = I32F32::lit("149");
pub const BEE_HIVE_TWO_HANDED: bool = true;
pub const BEE_HIVE_NEAR_SHIP_DISTANCE: I32F32 = I32F32::lit("100");
pub const BEE_HIVE_NEAR_MIN_VALUE: u32 = 40;
pub const BEE_HIVE_NEAR_MAX_VALUE: u32 = 99;
pub const BEE_HIVE_FAR_MIN_VALUE: u32 = 50;
pub const BEE_HIVE_FAR_MAX_VALUE: u32 = 149;
pub const BEE_HIVE_EXPERIMENTATION_MAX_COUNT: u8 = 5;
pub const BEE_HIVE_OTHER_MOON_MAX_COUNT: u8 = 6;
pub const BEE_HIVE_VALUE_ROLL_SALT: u64 = 0x6265_655f_6869_7665;

pub const BEE_HIVE_DEPENDS_ON: [&str; 13] = [
    "circuit_bee",
    "the_ship",
    "the_company",
    "credits",
    "march",
    "assurance",
    "vow",
    "artifice",
    "experimentation",
    "adamance",
    "employee",
    "version_80",
    "scrap",
];

pub const BEE_HIVE_SPAWN_CHANCES: [BeeHiveSpawnChance; 6] = [
    BeeHiveSpawnChance {
        moon: "march",
        chance: I32F32::lit("36.27"),
    },
    BeeHiveSpawnChance {
        moon: "assurance",
        chance: I32F32::lit("21.5"),
    },
    BeeHiveSpawnChance {
        moon: "vow",
        chance: I32F32::lit("18.65"),
    },
    BeeHiveSpawnChance {
        moon: "artifice",
        chance: I32F32::lit("15.63"),
    },
    BeeHiveSpawnChance {
        moon: "experimentation",
        chance: I32F32::lit("14.86"),
    },
    BeeHiveSpawnChance {
        moon: "adamance",
        chance: I32F32::lit("10.53"),
    },
];

pub const BEE_HIVE_BEHAVIORAL_MECHANICS: [BeeHiveBehaviorRule; 14] = [
    BeeHiveBehaviorRule {
        condition: "an employee comes too close to the hive",
        outcome: "circuit_bees chase the closest employee regardless of who is holding it",
    },
    BeeHiveBehaviorRule {
        condition: "an occupied hive is grabbed",
        outcome: "circuit_bees chase the closest employee regardless of who is holding it",
    },
    BeeHiveBehaviorRule {
        condition: "the hive is brought to the_ship",
        outcome: "it can be sold to the_company for credits",
    },
    BeeHiveBehaviorRule {
        condition: "the hive spawn location is smaller than 100 units from the ship landing area",
        outcome: "the value is randomly chosen between 40 and 99 credits",
    },
    BeeHiveBehaviorRule {
        condition: "the hive spawn location is greater than 100 units from the ship landing area",
        outcome: "the value is randomly chosen between 50 and 149 credits",
    },
    BeeHiveBehaviorRule {
        condition: "the moon is march",
        outcome: "the spawn chance is 36.27 percent",
    },
    BeeHiveBehaviorRule {
        condition: "the moon is assurance",
        outcome: "the spawn chance is 21.5 percent",
    },
    BeeHiveBehaviorRule {
        condition: "the moon is vow",
        outcome: "the spawn chance is 18.65 percent",
    },
    BeeHiveBehaviorRule {
        condition: "the moon is artifice",
        outcome: "the spawn chance is 15.63 percent",
    },
    BeeHiveBehaviorRule {
        condition: "the moon is experimentation",
        outcome: "the spawn chance is 14.86 percent",
    },
    BeeHiveBehaviorRule {
        condition: "the moon is adamance",
        outcome: "the spawn chance is 10.53 percent",
    },
    BeeHiveBehaviorRule {
        condition: "two bee hives are in the same distance category on the same day",
        outcome: "the random value is consistent across them",
    },
    BeeHiveBehaviorRule {
        condition: "the moon is experimentation",
        outcome: "the maximum number of bee hives on a single day is 5",
    },
    BeeHiveBehaviorRule {
        condition: "the moon is any other moon",
        outcome: "the maximum number of bee hives on a single day is 6",
    },
];

pub struct BeeHivePlugin;

impl Plugin for BeeHivePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnBeeHiveEvent>()
            .add_event::<BeeHiveClosestEmployeeChaseEvent>()
            .add_event::<BeeHiveSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_bee_hive,
                    bee_hive_pickup_item_bar_bridge,
                    bee_hive_proximity_chase_bridge,
                    bee_hive_sell_bridge,
                    bee_hive_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BeeHiveBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BeeHiveSpawnChance {
    pub moon: &'static str,
    pub chance: I32F32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BeeHiveDistanceCategory {
    #[default]
    NearShip,
    FarFromShip,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BeeHive {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BeeHiveScrap {
    pub value: I32F32,
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for BeeHiveScrap {
    fn default() -> Self {
        Self {
            value: BEE_HIVE_MIN_VALUE,
            min_value: BEE_HIVE_MIN_VALUE,
            max_value: BEE_HIVE_MAX_VALUE,
            weight: BEE_HIVE_WEIGHT,
            conductive: BEE_HIVE_CONDUCTIVE,
            two_handed: BEE_HIVE_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BeeHiveHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BeeHiveOccupancy {
    pub houses_circuit_bees: bool,
    pub closest_employee_id: u64,
    pub bees_are_chasing: bool,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BeeHiveSpawnContext {
    pub moon: &'static str,
    pub day_index: u64,
    pub distance_from_ship_landing_area: I32F32,
    pub distance_category: BeeHiveDistanceCategory,
    pub spawned_underwater: bool,
}

impl Default for BeeHiveSpawnContext {
    fn default() -> Self {
        Self {
            moon: "",
            day_index: 0,
            distance_from_ship_landing_area: I32F32::lit("0"),
            distance_category: BeeHiveDistanceCategory::NearShip,
            spawned_underwater: false,
        }
    }
}

#[derive(Bundle)]
pub struct BeeHiveBundle {
    pub name: Name,
    pub bee_hive: BeeHive,
    pub scrap: BeeHiveScrap,
    pub position: SimPosition,
    pub held_by: BeeHiveHeldBy,
    pub occupancy: BeeHiveOccupancy,
    pub spawn_context: BeeHiveSpawnContext,
}

impl BeeHiveBundle {
    pub fn new(event: SpawnBeeHiveEvent, value: I32F32) -> Self {
        let (min_value, max_value) = bee_hive_value_range_for_category(event.distance_category);

        Self {
            name: Name::new(BEE_HIVE_NAME),
            bee_hive: BeeHive {
                stable_id: event.stable_id,
            },
            scrap: BeeHiveScrap {
                value,
                min_value,
                max_value,
                weight: BEE_HIVE_WEIGHT,
                conductive: BEE_HIVE_CONDUCTIVE,
                two_handed: BEE_HIVE_TWO_HANDED,
            },
            position: event.position,
            held_by: BeeHiveHeldBy::default(),
            occupancy: BeeHiveOccupancy {
                houses_circuit_bees: true,
                closest_employee_id: 0,
                bees_are_chasing: false,
            },
            spawn_context: BeeHiveSpawnContext {
                moon: event.moon,
                day_index: event.day_index,
                distance_from_ship_landing_area: event.distance_from_ship_landing_area,
                distance_category: event.distance_category,
                spawned_underwater: event.spawned_underwater,
            },
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnBeeHiveEvent {
    pub stable_id: u64,
    pub position: SimPosition,
    pub moon: &'static str,
    pub day_index: u64,
    pub distance_from_ship_landing_area: I32F32,
    pub distance_category: BeeHiveDistanceCategory,
    pub spawned_underwater: bool,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BeeHiveClosestEmployeeChaseEvent {
    pub bee_hive_entity: Entity,
    pub bee_hive_stable_id: u64,
    pub closest_employee_id: u64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BeeHiveSoldEvent {
    pub bee_hive_stable_id: u64,
    pub credit_value: I32F32,
}

pub fn bee_hive_value_range() -> (I32F32, I32F32) {
    (BEE_HIVE_MIN_VALUE, BEE_HIVE_MAX_VALUE)
}

pub fn bee_hive_value_range_for_category(
    category: BeeHiveDistanceCategory,
) -> (I32F32, I32F32) {
    match category {
        BeeHiveDistanceCategory::NearShip => (
            I32F32::from_num(BEE_HIVE_NEAR_MIN_VALUE),
            I32F32::from_num(BEE_HIVE_NEAR_MAX_VALUE),
        ),
        BeeHiveDistanceCategory::FarFromShip => (
            I32F32::from_num(BEE_HIVE_FAR_MIN_VALUE),
            I32F32::from_num(BEE_HIVE_FAR_MAX_VALUE),
        ),
    }
}

pub fn bee_hive_spawn_chance_for_moon(moon: &str) -> Option<I32F32> {
    BEE_HIVE_SPAWN_CHANCES
        .iter()
        .find(|spawn_chance| spawn_chance.moon == moon)
        .map(|spawn_chance| spawn_chance.chance)
}

pub fn bee_hive_max_count_for_moon(moon: &str) -> u8 {
    if moon == "experimentation" {
        BEE_HIVE_EXPERIMENTATION_MAX_COUNT
    } else {
        BEE_HIVE_OTHER_MOON_MAX_COUNT
    }
}

pub fn bee_hive_distance_category(distance_from_ship_landing_area: I32F32) -> BeeHiveDistanceCategory {
    if distance_from_ship_landing_area < BEE_HIVE_NEAR_SHIP_DISTANCE {
        BeeHiveDistanceCategory::NearShip
    } else {
        BeeHiveDistanceCategory::FarFromShip
    }
}

fn spawn_bee_hive(
    mut commands: Commands,
    mut events: EventReader<SpawnBeeHiveEvent>,
    seed: Res<GameSeed>,
) {
    for event in events.read() {
        if event.spawned_underwater {
            continue;
        }

        let value = roll_bee_hive_value(seed.0, event.day_index, event.distance_category);
        commands.spawn(BeeHiveBundle::new(*event, value));
    }
}

fn bee_hive_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    mut chase_events: EventWriter<BeeHiveClosestEmployeeChaseEvent>,
    mut bee_hives: Query<(Entity, &BeeHive, &BeeHiveHeldBy, &mut BeeHiveOccupancy), Changed<BeeHiveHeldBy>>,
) {
    for (entity, bee_hive, held_by, mut occupancy) in &mut bee_hives {
        if !held_by.is_held {
            continue;
        }

        pickup_events.send(ItemBarPickupEvent {
            employee_id: held_by.employee_id,
            item_id: BEE_HIVE_ID,
            two_handed: BEE_HIVE_TWO_HANDED,
            functional: false,
            passive: true,
            from_store_or_valueless: false,
        });

        if occupancy.houses_circuit_bees {
            occupancy.closest_employee_id = held_by.employee_id;
            occupancy.bees_are_chasing = true;

            chase_events.send(BeeHiveClosestEmployeeChaseEvent {
                bee_hive_entity: entity,
                bee_hive_stable_id: bee_hive.stable_id,
                closest_employee_id: held_by.employee_id,
            });
        }
    }
}

fn bee_hive_proximity_chase_bridge(
    mut chase_events: EventWriter<BeeHiveClosestEmployeeChaseEvent>,
    mut bee_hives: Query<(Entity, &BeeHive, &mut BeeHiveOccupancy), Changed<BeeHiveOccupancy>>,
) {
    for (entity, bee_hive, mut occupancy) in &mut bee_hives {
        if !occupancy.houses_circuit_bees || occupancy.closest_employee_id == 0 {
            continue;
        }

        occupancy.bees_are_chasing = true;
        chase_events.send(BeeHiveClosestEmployeeChaseEvent {
            bee_hive_entity: entity,
            bee_hive_stable_id: bee_hive.stable_id,
            closest_employee_id: occupancy.closest_employee_id,
        });
    }
}

fn bee_hive_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<BeeHiveSoldEvent>,
    bee_hives: Query<(&BeeHive, &BeeHiveScrap)>,
) {
    for event in sell_events.read() {
        for (bee_hive, scrap) in &bee_hives {
            if bee_hive.stable_id != event.scrap_entity_id {
                continue;
            }

            sold_events.send(BeeHiveSoldEvent {
                bee_hive_stable_id: bee_hive.stable_id,
                credit_value: scrap.value,
            });
        }
    }
}

fn roll_bee_hive_value(
    game_seed: u64,
    day_index: u64,
    category: BeeHiveDistanceCategory,
) -> I32F32 {
    let (min_value, max_value) = match category {
        BeeHiveDistanceCategory::NearShip => (BEE_HIVE_NEAR_MIN_VALUE, BEE_HIVE_NEAR_MAX_VALUE),
        BeeHiveDistanceCategory::FarFromShip => (BEE_HIVE_FAR_MIN_VALUE, BEE_HIVE_FAR_MAX_VALUE),
    };

    let category_salt = match category {
        BeeHiveDistanceCategory::NearShip => 0x1000,
        BeeHiveDistanceCategory::FarFromShip => 0x2000,
    };
    let mut rng = tick_rng(game_seed, day_index, BEE_HIVE_VALUE_ROLL_SALT ^ category_salt);
    let span = max_value - min_value + 1;
    I32F32::from_num(min_value + (rng.next_u32() % span))
}

fn bee_hive_checksum(
    mut checksum: ResMut<SimChecksumState>,
    bee_hives: Query<(
        &BeeHive,
        &BeeHiveScrap,
        &SimPosition,
        &BeeHiveHeldBy,
        &BeeHiveOccupancy,
        &BeeHiveSpawnContext,
    )>,
) {
    accumulate_str(&mut checksum, 0x1000, BEE_HIVE_ID);
    accumulate_str(&mut checksum, 0x1001, BEE_HIVE_NAME);
    accumulate_str(&mut checksum, 0x1002, BEE_HIVE_TYPE);
    accumulate_str(&mut checksum, 0x1003, BEE_HIVE_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, BEE_HIVE_EFFECTS);

    checksum.accumulate(BEE_HIVE_SOURCE_REVISION as u64);
    checksum.accumulate(BEE_HIVE_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(BEE_HIVE_WEIGHT.to_bits() as u64);
    checksum.accumulate(BEE_HIVE_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(BEE_HIVE_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(BEE_HIVE_CONDUCTIVE as u64);
    checksum.accumulate(BEE_HIVE_TWO_HANDED as u64);
    checksum.accumulate(BEE_HIVE_NEAR_SHIP_DISTANCE.to_bits() as u64);
    checksum.accumulate(BEE_HIVE_NEAR_MIN_VALUE as u64);
    checksum.accumulate(BEE_HIVE_NEAR_MAX_VALUE as u64);
    checksum.accumulate(BEE_HIVE_FAR_MIN_VALUE as u64);
    checksum.accumulate(BEE_HIVE_FAR_MAX_VALUE as u64);
    checksum.accumulate(BEE_HIVE_EXPERIMENTATION_MAX_COUNT as u64);
    checksum.accumulate(BEE_HIVE_OTHER_MOON_MAX_COUNT as u64);

    for dependency in BEE_HIVE_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for spawn_chance in BEE_HIVE_SPAWN_CHANCES {
        accumulate_str(&mut checksum, 0x3000, spawn_chance.moon);
        checksum.accumulate(spawn_chance.chance.to_bits() as u64);
    }

    for rule in BEE_HIVE_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x4000, rule.condition);
        accumulate_str(&mut checksum, 0x4001, rule.outcome);
    }

    for (bee_hive, scrap, position, held_by, occupancy, spawn_context) in &bee_hives {
        checksum.accumulate(bee_hive.stable_id);
        checksum.accumulate(scrap.value.to_bits() as u64);
        checksum.accumulate(scrap.min_value.to_bits() as u64);
        checksum.accumulate(scrap.max_value.to_bits() as u64);
        checksum.accumulate(scrap.weight.to_bits() as u64);
        checksum.accumulate(scrap.conductive as u64);
        checksum.accumulate(scrap.two_handed as u64);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(held_by.employee_id);
        checksum.accumulate(held_by.is_held as u64);
        checksum.accumulate(occupancy.houses_circuit_bees as u64);
        checksum.accumulate(occupancy.closest_employee_id);
        checksum.accumulate(occupancy.bees_are_chasing as u64);
        accumulate_str(&mut checksum, 0x5000, spawn_context.moon);
        checksum.accumulate(spawn_context.day_index);
        checksum.accumulate(spawn_context.distance_from_ship_landing_area.to_bits() as u64);
        checksum.accumulate(match spawn_context.distance_category {
            BeeHiveDistanceCategory::NearShip => 1,
            BeeHiveDistanceCategory::FarFromShip => 2,
        });
        checksum.accumulate(spawn_context.spawned_underwater as u64);
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt);
    for byte in value.as_bytes() {
        checksum.accumulate(*byte as u64);
    }
}
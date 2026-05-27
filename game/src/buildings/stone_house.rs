// Sources: vault/buildings/stone_house.md, vault/buildings/cottage.md

use std::collections::BTreeMap;

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, SimChecksumState};

const STONE_HOUSE_HP: I32F32 = I32F32::lit("500");
const STONE_HOUSE_DEFENSES_LIFE: I32F32 = I32F32::lit("125");
const STONE_HOUSE_WATCH_RANGE: I32F32 = I32F32::lit("6");

const STONE_HOUSE_ENERGY_COST: i32 = 10;
const STONE_HOUSE_WOOD_COST: i32 = 5;
const STONE_HOUSE_STONE_COST: i32 = 10;
const STONE_HOUSE_IRON_COST: i32 = 0;
const STONE_HOUSE_OIL_COST: i32 = 0;
const STONE_HOUSE_GOLD_COST: i32 = 300;
const STONE_HOUSE_BUILD_TIME_SECONDS: i32 = 48;

const STONE_HOUSE_UPGRADE_WOOD_COST: i32 = 0;
const STONE_HOUSE_UPGRADE_STONE_COST: i32 = 10;
const STONE_HOUSE_UPGRADE_IRON_COST: i32 = 0;
const STONE_HOUSE_UPGRADE_OIL_COST: i32 = 0;
const STONE_HOUSE_UPGRADE_GOLD_COST: i32 = 180;
const STONE_HOUSE_UPGRADE_TIME_SECONDS: i32 = 21;

const STONE_HOUSE_SIZE_TILES: i32 = 2;
const STONE_HOUSE_WORKERS: i32 = 8;
const STONE_HOUSE_COLONISTS: i32 = 16;
const STONE_HOUSE_FOOD_UPKEEP: i32 = 16;
const STONE_HOUSE_GOLD_PER_CYCLE: i32 = 40;

const STONE_HOUSE_VS_TENT_WORKERS_MULTIPLIER: i32 = 4;
const STONE_HOUSE_VS_TENT_COLONISTS_MULTIPLIER: i32 = 4;
const STONE_HOUSE_VS_TENT_GOLD_MULTIPLIER: i32 = 5;
const STONE_HOUSE_VS_TENT_FOOD_MULTIPLIER: i32 = 4;
const STONE_HOUSE_VS_TENT_ENERGY_MULTIPLIER: i32 = 10;

const SIM_HZ: i32 = 25;
const GOLD_CYCLE_HOURS: i32 = 8;
const SECONDS_PER_HOUR: i32 = 3600;

#[derive(Component, Default)]
pub struct Cottage;

#[derive(Component, Default)]
pub struct StoneHouse;

#[derive(Component, Clone, Copy)]
pub struct BuildingAnchor {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Copy)]
pub struct BuildingDefenses {
    pub defenses_life: I32F32,
}

impl Default for BuildingDefenses {
    fn default() -> Self {
        Self {
            defenses_life: STONE_HOUSE_DEFENSES_LIFE,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct StoneHouseBuildState {
    pub build_ticks_remaining: i32,
    pub upgrading_from_cottage: bool,
    pub completed: bool,
}

#[derive(Component, Clone, Copy)]
pub struct StoneHouseEconomy {
    pub wood_cost: i32,
    pub stone_cost: i32,
    pub iron_cost: i32,
    pub oil_cost: i32,
    pub gold_cost: i32,
    pub upgrade_wood_cost: i32,
    pub upgrade_stone_cost: i32,
    pub upgrade_iron_cost: i32,
    pub upgrade_oil_cost: i32,
    pub upgrade_gold_cost: i32,
}

impl Default for StoneHouseEconomy {
    fn default() -> Self {
        Self {
            wood_cost: STONE_HOUSE_WOOD_COST,
            stone_cost: STONE_HOUSE_STONE_COST,
            iron_cost: STONE_HOUSE_IRON_COST,
            oil_cost: STONE_HOUSE_OIL_COST,
            gold_cost: STONE_HOUSE_GOLD_COST,
            upgrade_wood_cost: STONE_HOUSE_UPGRADE_WOOD_COST,
            upgrade_stone_cost: STONE_HOUSE_UPGRADE_STONE_COST,
            upgrade_iron_cost: STONE_HOUSE_UPGRADE_IRON_COST,
            upgrade_oil_cost: STONE_HOUSE_UPGRADE_OIL_COST,
            upgrade_gold_cost: STONE_HOUSE_UPGRADE_GOLD_COST,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct StoneHouseFootprint {
    pub size_tiles: i32,
    pub watch_range: I32F32,
}

impl Default for StoneHouseFootprint {
    fn default() -> Self {
        Self {
            size_tiles: STONE_HOUSE_SIZE_TILES,
            watch_range: STONE_HOUSE_WATCH_RANGE,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct StoneHousePopulation {
    pub workers: i32,
    pub colonists: i32,
    pub food_upkeep: i32,
    pub energy_upkeep: i32,
}

impl Default for StoneHousePopulation {
    fn default() -> Self {
        Self {
            workers: STONE_HOUSE_WORKERS,
            colonists: STONE_HOUSE_COLONISTS,
            food_upkeep: STONE_HOUSE_FOOD_UPKEEP,
            energy_upkeep: STONE_HOUSE_ENERGY_COST,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct StoneHouseIncome {
    pub gold_per_cycle: i32,
    pub cycle_ticks: i32,
    pub ticks_until_next_cycle: i32,
    pub stored_gold: i64,
}

impl Default for StoneHouseIncome {
    fn default() -> Self {
        let cycle_ticks = hours_to_ticks(GOLD_CYCLE_HOURS);
        Self {
            gold_per_cycle: STONE_HOUSE_GOLD_PER_CYCLE,
            cycle_ticks,
            ticks_until_next_cycle: cycle_ticks,
            stored_gold: 0,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct StoneHouseVsTentMultipliers {
    pub workers: i32,
    pub colonists: i32,
    pub gold_income: i32,
    pub food_upkeep: i32,
    pub energy_upkeep: i32,
}

impl Default for StoneHouseVsTentMultipliers {
    fn default() -> Self {
        Self {
            workers: STONE_HOUSE_VS_TENT_WORKERS_MULTIPLIER,
            colonists: STONE_HOUSE_VS_TENT_COLONISTS_MULTIPLIER,
            gold_income: STONE_HOUSE_VS_TENT_GOLD_MULTIPLIER,
            food_upkeep: STONE_HOUSE_VS_TENT_FOOD_MULTIPLIER,
            energy_upkeep: STONE_HOUSE_VS_TENT_ENERGY_MULTIPLIER,
        }
    }
}

#[derive(Resource, Default, Clone)]
pub struct StoneHousePlacementClaims {
    pub claims: BTreeMap<Entity, BuildingAnchor>,
}

#[derive(Event, Clone, Copy)]
pub struct PlaceStoneHouseEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct StoneHousePlacementRejectedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct UpgradeCottageToStoneHouseEvent {
    pub cottage_entity: Entity,
}

fn seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn hours_to_ticks(hours: i32) -> i32 {
    hours * SECONDS_PER_HOUR * SIM_HZ
}

fn place_stone_house_system(
    mut commands: Commands,
    mut events: EventReader<PlaceStoneHouseEvent>,
    mut rejected: EventWriter<StoneHousePlacementRejectedEvent>,
    mut claims: ResMut<StoneHousePlacementClaims>,
) {
    for ev in events.read() {
        let anchor = BuildingAnchor {
            x: ev.tile_x,
            y: ev.tile_y,
        };

        if claims
            .claims
            .values()
            .any(|existing| existing.x == anchor.x && existing.y == anchor.y)
        {
            rejected.send(StoneHousePlacementRejectedEvent {
                tile_x: ev.tile_x,
                tile_y: ev.tile_y,
            });
            continue;
        }

        let entity = commands
            .spawn((
                StoneHouse,
                anchor,
                Health::full(STONE_HOUSE_HP),
                BuildingDefenses::default(),
                StoneHouseBuildState {
                    build_ticks_remaining: seconds_to_ticks(STONE_HOUSE_BUILD_TIME_SECONDS),
                    upgrading_from_cottage: false,
                    completed: false,
                },
                StoneHouseEconomy::default(),
                StoneHouseFootprint::default(),
                StoneHousePopulation::default(),
                StoneHouseIncome::default(),
                StoneHouseVsTentMultipliers::default(),
            ))
            .id();

        claims.claims.insert(entity, anchor);
    }
}

fn upgrade_cottage_to_stone_house_system(
    mut commands: Commands,
    mut events: EventReader<UpgradeCottageToStoneHouseEvent>,
    cottages: Query<(Entity, &BuildingAnchor), With<Cottage>>,
    mut claims: ResMut<StoneHousePlacementClaims>,
) {
    for ev in events.read() {
        let Ok((cottage_entity, anchor)) = cottages.get(ev.cottage_entity) else {
            continue;
        };

        commands.entity(cottage_entity).remove::<Cottage>();
        commands.entity(cottage_entity).insert((
            StoneHouse,
            *anchor,
            Health::full(STONE_HOUSE_HP),
            BuildingDefenses::default(),
            StoneHouseBuildState {
                build_ticks_remaining: seconds_to_ticks(STONE_HOUSE_UPGRADE_TIME_SECONDS),
                upgrading_from_cottage: true,
                completed: false,
            },
            StoneHouseEconomy::default(),
            StoneHouseFootprint::default(),
            StoneHousePopulation::default(),
            StoneHouseIncome::default(),
            StoneHouseVsTentMultipliers::default(),
        ));

        claims.claims.insert(cottage_entity, *anchor);
    }
}

fn stone_house_build_tick_system(mut houses: Query<&mut StoneHouseBuildState, With<StoneHouse>>) {
    for mut state in &mut houses {
        if state.completed {
            continue;
        }

        if state.build_ticks_remaining > 0 {
            state.build_ticks_remaining -= 1;
        }

        if state.build_ticks_remaining <= 0 {
            state.build_ticks_remaining = 0;
            state.completed = true;
        }
    }
}

fn stone_house_income_tick_system(
    mut houses: Query<(&StoneHouseBuildState, &mut StoneHouseIncome), With<StoneHouse>>,
) {
    for (build, mut income) in &mut houses {
        if !build.completed {
            income.ticks_until_next_cycle = income.cycle_ticks;
            continue;
        }

        if income.ticks_until_next_cycle > 0 {
            income.ticks_until_next_cycle -= 1;
        }

        if income.ticks_until_next_cycle <= 0 {
            income.stored_gold += i64::from(income.gold_per_cycle);
            income.ticks_until_next_cycle = income.cycle_ticks;
        }
    }
}

fn stone_house_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    houses: Query<
        (
            Entity,
            &BuildingAnchor,
            &Health,
            &BuildingDefenses,
            &StoneHouseBuildState,
            &StoneHouseEconomy,
            &StoneHouseFootprint,
            &StoneHousePopulation,
            &StoneHouseIncome,
            &StoneHouseVsTentMultipliers,
        ),
        With<StoneHouse>,
    >,
    claims: Res<StoneHousePlacementClaims>,
) {
    for (
        entity,
        anchor,
        health,
        defenses,
        build,
        economy,
        footprint,
        population,
        income,
        multipliers,
    ) in &houses
    {
        checksum.accumulate(entity.to_bits() as u64);

        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);

        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(defenses.defenses_life.to_bits() as u64);

        checksum.accumulate(build.build_ticks_remaining as u64);
        checksum.accumulate(u64::from(build.upgrading_from_cottage));
        checksum.accumulate(u64::from(build.completed));

        checksum.accumulate(economy.wood_cost as u64);
        checksum.accumulate(economy.stone_cost as u64);
        checksum.accumulate(economy.iron_cost as u64);
        checksum.accumulate(economy.oil_cost as u64);
        checksum.accumulate(economy.gold_cost as u64);
        checksum.accumulate(economy.upgrade_wood_cost as u64);
        checksum.accumulate(economy.upgrade_stone_cost as u64);
        checksum.accumulate(economy.upgrade_iron_cost as u64);
        checksum.accumulate(economy.upgrade_oil_cost as u64);
        checksum.accumulate(economy.upgrade_gold_cost as u64);

        checksum.accumulate(footprint.size_tiles as u64);
        checksum.accumulate(footprint.watch_range.to_bits() as u64);

        checksum.accumulate(population.workers as u64);
        checksum.accumulate(population.colonists as u64);
        checksum.accumulate(population.food_upkeep as u64);
        checksum.accumulate(population.energy_upkeep as u64);

        checksum.accumulate(income.gold_per_cycle as u64);
        checksum.accumulate(income.cycle_ticks as u64);
        checksum.accumulate(income.ticks_until_next_cycle as u64);
        checksum.accumulate(income.stored_gold as u64);

        checksum.accumulate(multipliers.workers as u64);
        checksum.accumulate(multipliers.colonists as u64);
        checksum.accumulate(multipliers.gold_income as u64);
        checksum.accumulate(multipliers.food_upkeep as u64);
        checksum.accumulate(multipliers.energy_upkeep as u64);
    }

    checksum.accumulate(claims.claims.len() as u64);
    for (entity, anchor) in &claims.claims {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);
    }
}

pub struct StoneHousePlugin;

impl Plugin for StoneHousePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StoneHousePlacementClaims>()
            .add_event::<PlaceStoneHouseEvent>()
            .add_event::<StoneHousePlacementRejectedEvent>()
            .add_event::<UpgradeCottageToStoneHouseEvent>()
            .add_systems(
                FixedUpdate,
                (
                    place_stone_house_system,
                    upgrade_cottage_to_stone_house_system,
                    stone_house_build_tick_system,
                    stone_house_income_tick_system,
                    stone_house_checksum_system,
                )
                    .chain(),
            );
    }
}
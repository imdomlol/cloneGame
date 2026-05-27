// Sources: vault/buildings/power_plant.md, vault/buildings/buildings.md

use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::SimChecksumState;

const POWER_PLANT_HP: I32F32 = I32F32::lit("800");
const POWER_PLANT_DEFENSES_LIFE: I32F32 = I32F32::lit("200");
const POWER_PLANT_WATCH_RANGE: I32F32 = I32F32::lit("7");
const POWER_PLANT_WOOD_COST: i32 = 50;
const POWER_PLANT_STONE_COST: i32 = 30;
const POWER_PLANT_GOLD_COST: i32 = 800;
const POWER_PLANT_WORKERS: i32 = 8;
const POWER_PLANT_BUILD_TIME_SECONDS: i32 = 70;
const POWER_PLANT_BUILDING_SIZE_TILES: i32 = 3;
const POWER_PLANT_ENERGY_PRODUCTION: I32F32 = I32F32::lit("160");
const POWER_PLANT_WOOD_INCOME_DELTA: I32F32 = I32F32::lit("-10");
const SIM_HZ: i32 = 25;

#[derive(Component, Default)]
pub struct PowerPlant;

#[derive(Component, Clone, Copy)]
pub struct BuildingAnchor {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Copy)]
pub struct BuildingHealth {
    pub hp: I32F32,
    pub defenses_life: I32F32,
}

impl Default for BuildingHealth {
    fn default() -> Self {
        Self {
            hp: POWER_PLANT_HP,
            defenses_life: POWER_PLANT_DEFENSES_LIFE,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct PowerPlantBuildState {
    pub build_ticks_remaining: i32,
    pub completed: bool,
}

#[derive(Component, Clone, Copy)]
pub struct PowerPlantEconomy {
    pub wood_cost: i32,
    pub stone_cost: i32,
    pub gold_cost: i32,
    pub workers: i32,
}

impl Default for PowerPlantEconomy {
    fn default() -> Self {
        Self {
            wood_cost: POWER_PLANT_WOOD_COST,
            stone_cost: POWER_PLANT_STONE_COST,
            gold_cost: POWER_PLANT_GOLD_COST,
            workers: POWER_PLANT_WORKERS,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct PowerPlantOutput {
    pub energy_production: I32F32,
    pub wood_income_delta: I32F32,
    pub active_energy_output: I32F32,
    pub active_wood_income_delta: I32F32,
}

impl Default for PowerPlantOutput {
    fn default() -> Self {
        Self {
            energy_production: POWER_PLANT_ENERGY_PRODUCTION,
            wood_income_delta: POWER_PLANT_WOOD_INCOME_DELTA,
            active_energy_output: I32F32::ZERO,
            active_wood_income_delta: I32F32::ZERO,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct PowerPlantFootprint {
    pub size_tiles: i32,
    pub watch_range: I32F32,
}

impl Default for PowerPlantFootprint {
    fn default() -> Self {
        Self {
            size_tiles: POWER_PLANT_BUILDING_SIZE_TILES,
            watch_range: POWER_PLANT_WATCH_RANGE,
        }
    }
}

#[derive(Resource, Default, Clone)]
pub struct TileOccupancy {
    pub blocked_tiles: BTreeSet<(i32, i32)>,
}

#[derive(Resource, Default, Clone)]
pub struct PowerPlantPlacementClaims {
    pub claims: BTreeMap<Entity, BuildingAnchor>,
}

#[derive(Event, Clone, Copy)]
pub struct SetTileBlockedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
    pub blocked: bool,
}

#[derive(Event, Clone, Copy)]
pub struct PlacePowerPlantEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct PowerPlantPlacementRejectedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

fn build_seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn footprint_tiles(anchor: BuildingAnchor) -> impl Iterator<Item = (i32, i32)> {
    let min_x = anchor.x;
    let min_y = anchor.y;
    let max_x = anchor.x + POWER_PLANT_BUILDING_SIZE_TILES - 1;
    let max_y = anchor.y + POWER_PLANT_BUILDING_SIZE_TILES - 1;

    (min_x..=max_x).flat_map(move |x| (min_y..=max_y).map(move |y| (x, y)))
}

fn footprint_is_unblocked(anchor: BuildingAnchor, occupancy: &TileOccupancy) -> bool {
    for tile in footprint_tiles(anchor) {
        if occupancy.blocked_tiles.contains(&tile) {
            return false;
        }
    }
    true
}

fn apply_tile_block_events_system(
    mut events: EventReader<SetTileBlockedEvent>,
    mut occupancy: ResMut<TileOccupancy>,
) {
    for ev in events.read() {
        let tile = (ev.tile_x, ev.tile_y);
        if ev.blocked {
            occupancy.blocked_tiles.insert(tile);
        } else {
            occupancy.blocked_tiles.remove(&tile);
        }
    }
}

fn place_power_plant_system(
    mut commands: Commands,
    mut events: EventReader<PlacePowerPlantEvent>,
    mut rejected: EventWriter<PowerPlantPlacementRejectedEvent>,
    mut claims: ResMut<PowerPlantPlacementClaims>,
    occupancy: Res<TileOccupancy>,
) {
    for ev in events.read() {
        let anchor = BuildingAnchor {
            x: ev.tile_x,
            y: ev.tile_y,
        };

        if !footprint_is_unblocked(anchor, &occupancy) {
            rejected.send(PowerPlantPlacementRejectedEvent {
                tile_x: ev.tile_x,
                tile_y: ev.tile_y,
            });
            continue;
        }

        let entity = commands
            .spawn((
                PowerPlant,
                anchor,
                BuildingHealth::default(),
                PowerPlantBuildState {
                    build_ticks_remaining: build_seconds_to_ticks(POWER_PLANT_BUILD_TIME_SECONDS),
                    completed: false,
                },
                PowerPlantEconomy::default(),
                PowerPlantOutput::default(),
                PowerPlantFootprint::default(),
            ))
            .id();

        claims.claims.insert(entity, anchor);
    }
}

fn power_plant_build_tick_system(
    mut plants: Query<&mut PowerPlantBuildState, With<PowerPlant>>,
) {
    for mut state in &mut plants {
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

fn power_plant_output_system(
    mut plants: Query<(&PowerPlantBuildState, &mut PowerPlantOutput), With<PowerPlant>>,
) {
    for (build_state, mut output) in &mut plants {
        if build_state.completed {
            output.active_energy_output = output.energy_production;
            output.active_wood_income_delta = output.wood_income_delta;
        } else {
            output.active_energy_output = I32F32::ZERO;
            output.active_wood_income_delta = I32F32::ZERO;
        }
    }
}

fn power_plant_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    plants: Query<
        (
            Entity,
            &BuildingAnchor,
            &BuildingHealth,
            &PowerPlantBuildState,
            &PowerPlantEconomy,
            &PowerPlantOutput,
            &PowerPlantFootprint,
        ),
        With<PowerPlant>,
    >,
    claims: Res<PowerPlantPlacementClaims>,
) {
    for (entity, anchor, hp, build, eco, out, footprint) in &plants {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);

        checksum.accumulate(hp.hp.to_bits() as u64);
        checksum.accumulate(hp.defenses_life.to_bits() as u64);

        checksum.accumulate(build.build_ticks_remaining as u64);
        checksum.accumulate(u64::from(build.completed));

        checksum.accumulate(eco.wood_cost as u64);
        checksum.accumulate(eco.stone_cost as u64);
        checksum.accumulate(eco.gold_cost as u64);
        checksum.accumulate(eco.workers as u64);

        checksum.accumulate(out.energy_production.to_bits() as u64);
        checksum.accumulate(out.wood_income_delta.to_bits() as u64);
        checksum.accumulate(out.active_energy_output.to_bits() as u64);
        checksum.accumulate(out.active_wood_income_delta.to_bits() as u64);

        checksum.accumulate(footprint.size_tiles as u64);
        checksum.accumulate(footprint.watch_range.to_bits() as u64);
    }

    checksum.accumulate(claims.claims.len() as u64);
    for (entity, anchor) in &claims.claims {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);
    }
}

pub struct PowerPlantPlugin;

impl Plugin for PowerPlantPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileOccupancy>()
            .init_resource::<PowerPlantPlacementClaims>()
            .add_event::<SetTileBlockedEvent>()
            .add_event::<PlacePowerPlantEvent>()
            .add_event::<PowerPlantPlacementRejectedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    apply_tile_block_events_system,
                    place_power_plant_system,
                    power_plant_build_tick_system,
                    power_plant_output_system,
                    power_plant_checksum_system,
                )
                    .chain(),
            );
    }
}
// Sources: vault/buildings/fisherman_cottage.md

use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, SimChecksumState};

const FISHERMAN_COTTAGE_HP: I32F32 = I32F32::lit("160");
const FISHERMAN_COTTAGE_DEFENSES_LIFE: I32F32 = I32F32::lit("40");
const FISHERMAN_COTTAGE_WATCH_RANGE: I32F32 = I32F32::lit("6");

const FISHERMAN_COTTAGE_ENERGY_COST: i32 = 1;
const FISHERMAN_COTTAGE_GOLD_COST: i32 = 80;
const FISHERMAN_COTTAGE_BUILD_TIME_SECONDS: i32 = 30;
const FISHERMAN_COTTAGE_SIZE_TILES: i32 = 1;
const FISHERMAN_COTTAGE_PFOOD_MIN: i32 = 1;
const FISHERMAN_COTTAGE_PFOOD_MAX: i32 = 20;
const FISHERMAN_COTTAGE_PCOLONISTS: i32 = 2;

const GATHER_RADIUS: i32 = 3;
const SIM_HZ: i32 = 25;

#[derive(Component, Default)]
pub struct FishermanCottage;

#[derive(Component, Clone, Copy)]
pub struct BuildingAnchor {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Copy)]
pub struct BuildingDurability {
    pub defenses_life: I32F32,
}

impl Default for BuildingDurability {
    fn default() -> Self {
        Self {
            defenses_life: FISHERMAN_COTTAGE_DEFENSES_LIFE,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct FishermanCottageBuildState {
    pub build_ticks_remaining: i32,
    pub completed: bool,
}

impl Default for FishermanCottageBuildState {
    fn default() -> Self {
        Self {
            build_ticks_remaining: seconds_to_ticks(FISHERMAN_COTTAGE_BUILD_TIME_SECONDS),
            completed: false,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct FishermanCottageEconomy {
    pub energy_cost: i32,
    pub gold_cost: i32,
    pub pfood_min: i32,
    pub pfood_max: i32,
    pub pcolonists: i32,
}

impl Default for FishermanCottageEconomy {
    fn default() -> Self {
        Self {
            energy_cost: FISHERMAN_COTTAGE_ENERGY_COST,
            gold_cost: FISHERMAN_COTTAGE_GOLD_COST,
            pfood_min: FISHERMAN_COTTAGE_PFOOD_MIN,
            pfood_max: FISHERMAN_COTTAGE_PFOOD_MAX,
            pcolonists: FISHERMAN_COTTAGE_PCOLONISTS,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct FishermanCottageFootprint {
    pub size_tiles: i32,
    pub watch_range: I32F32,
}

impl Default for FishermanCottageFootprint {
    fn default() -> Self {
        Self {
            size_tiles: FISHERMAN_COTTAGE_SIZE_TILES,
            watch_range: FISHERMAN_COTTAGE_WATCH_RANGE,
        }
    }
}

#[derive(Component, Default, Clone)]
pub struct FishermanCottageCatchment {
    pub claimed_water_tiles: BTreeSet<(i32, i32)>,
}

#[derive(Component, Clone, Copy, Default)]
pub struct FishermanCottageProductionState {
    pub current_food_output: i32,
}

#[derive(Resource, Default, Clone)]
pub struct WaterTiles {
    pub tiles: BTreeSet<(i32, i32)>,
}

#[derive(Resource, Default, Clone)]
pub struct WaterTileClaims {
    pub claimed_by: BTreeMap<(i32, i32), Entity>,
}

#[derive(Resource, Default, Clone)]
pub struct FishermanCottagePlacementClaims {
    pub claims: BTreeMap<Entity, BuildingAnchor>,
}

#[derive(Event, Clone, Copy)]
pub struct SetWaterTileEvent {
    pub tile_x: i32,
    pub tile_y: i32,
    pub is_water: bool,
}

#[derive(Event, Clone, Copy)]
pub struct PlaceFishermanCottageEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct FishermanCottagePlacementRejectedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

fn seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn is_adjacent_to_water(anchor: BuildingAnchor, water_tiles: &WaterTiles) -> bool {
    for dx in -1..=1 {
        for dy in -1..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }

            let tile = (anchor.x + dx, anchor.y + dy);
            if water_tiles.tiles.contains(&tile) {
                return true;
            }
        }
    }

    false
}

fn gather_zone_tiles(anchor: BuildingAnchor) -> impl Iterator<Item = (i32, i32)> {
    let min_x = anchor.x - GATHER_RADIUS;
    let max_x = anchor.x + GATHER_RADIUS;
    let min_y = anchor.y - GATHER_RADIUS;
    let max_y = anchor.y + GATHER_RADIUS;

    (min_x..=max_x).flat_map(move |x| (min_y..=max_y).map(move |y| (x, y)))
}

fn apply_water_tile_events_system(
    mut events: EventReader<SetWaterTileEvent>,
    mut water_tiles: ResMut<WaterTiles>,
    mut claims: ResMut<WaterTileClaims>,
) {
    for ev in events.read() {
        let tile = (ev.tile_x, ev.tile_y);
        if ev.is_water {
            water_tiles.tiles.insert(tile);
        } else {
            water_tiles.tiles.remove(&tile);
            claims.claimed_by.remove(&tile);
        }
    }
}

fn place_fisherman_cottage_system(
    mut commands: Commands,
    mut events: EventReader<PlaceFishermanCottageEvent>,
    mut rejected: EventWriter<FishermanCottagePlacementRejectedEvent>,
    water_tiles: Res<WaterTiles>,
    mut water_claims: ResMut<WaterTileClaims>,
    mut placement_claims: ResMut<FishermanCottagePlacementClaims>,
) {
    for ev in events.read() {
        let anchor = BuildingAnchor {
            x: ev.tile_x,
            y: ev.tile_y,
        };

        if placement_claims
            .claims
            .values()
            .any(|existing| existing.x == anchor.x && existing.y == anchor.y)
        {
            rejected.send(FishermanCottagePlacementRejectedEvent {
                tile_x: ev.tile_x,
                tile_y: ev.tile_y,
            });
            continue;
        }

        if !is_adjacent_to_water(anchor, &water_tiles) {
            rejected.send(FishermanCottagePlacementRejectedEvent {
                tile_x: ev.tile_x,
                tile_y: ev.tile_y,
            });
            continue;
        }

        let entity = commands
            .spawn((
                FishermanCottage,
                anchor,
                Health::full(FISHERMAN_COTTAGE_HP),
                BuildingDurability::default(),
                FishermanCottageBuildState::default(),
                FishermanCottageEconomy::default(),
                FishermanCottageFootprint::default(),
                FishermanCottageCatchment::default(),
                FishermanCottageProductionState::default(),
            ))
            .id();

        let mut claimed_tiles = BTreeSet::new();
        for tile in gather_zone_tiles(anchor) {
            if !water_tiles.tiles.contains(&tile) {
                continue;
            }
            if water_claims.claimed_by.contains_key(&tile) {
                continue;
            }

            water_claims.claimed_by.insert(tile, entity);
            claimed_tiles.insert(tile);
        }

        commands.entity(entity).insert(FishermanCottageCatchment {
            claimed_water_tiles: claimed_tiles,
        });
        placement_claims.claims.insert(entity, anchor);
    }
}

fn fisherman_cottage_build_tick_system(
    mut cottages: Query<&mut FishermanCottageBuildState, With<FishermanCottage>>,
) {
    for mut state in &mut cottages {
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

fn fisherman_cottage_food_production_system(
    mut cottages: Query<
        (
            &FishermanCottageBuildState,
            &FishermanCottageEconomy,
            &FishermanCottageCatchment,
            &mut FishermanCottageProductionState,
        ),
        With<FishermanCottage>,
    >,
) {
    for (build, economy, catchment, mut production) in &mut cottages {
        if !build.completed {
            production.current_food_output = 0;
            continue;
        }

        let mut output = catchment.claimed_water_tiles.len() as i32;
        if output < economy.pfood_min {
            output = economy.pfood_min;
        }
        if output > economy.pfood_max {
            output = economy.pfood_max;
        }

        production.current_food_output = output;
    }
}

fn fisherman_cottage_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    cottages: Query<
        (
            Entity,
            &BuildingAnchor,
            &Health,
            &BuildingDurability,
            &FishermanCottageBuildState,
            &FishermanCottageEconomy,
            &FishermanCottageFootprint,
            &FishermanCottageCatchment,
            &FishermanCottageProductionState,
        ),
        With<FishermanCottage>,
    >,
    water_tiles: Res<WaterTiles>,
    water_claims: Res<WaterTileClaims>,
    placement_claims: Res<FishermanCottagePlacementClaims>,
) {
    for (
        entity,
        anchor,
        health,
        durability,
        build,
        economy,
        footprint,
        catchment,
        production,
    ) in &cottages
    {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);

        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(durability.defenses_life.to_bits() as u64);

        checksum.accumulate(build.build_ticks_remaining as u64);
        checksum.accumulate(u64::from(build.completed));

        checksum.accumulate(economy.energy_cost as u64);
        checksum.accumulate(economy.gold_cost as u64);
        checksum.accumulate(economy.pfood_min as u64);
        checksum.accumulate(economy.pfood_max as u64);
        checksum.accumulate(economy.pcolonists as u64);

        checksum.accumulate(footprint.size_tiles as u64);
        checksum.accumulate(footprint.watch_range.to_bits() as u64);

        checksum.accumulate(catchment.claimed_water_tiles.len() as u64);
        for tile in &catchment.claimed_water_tiles {
            checksum.accumulate(tile.0 as u64);
            checksum.accumulate(tile.1 as u64);
        }

        checksum.accumulate(production.current_food_output as u64);
    }

    checksum.accumulate(water_tiles.tiles.len() as u64);
    for tile in &water_tiles.tiles {
        checksum.accumulate(tile.0 as u64);
        checksum.accumulate(tile.1 as u64);
    }

    checksum.accumulate(water_claims.claimed_by.len() as u64);
    for (tile, entity) in &water_claims.claimed_by {
        checksum.accumulate(tile.0 as u64);
        checksum.accumulate(tile.1 as u64);
        checksum.accumulate(entity.to_bits() as u64);
    }

    checksum.accumulate(placement_claims.claims.len() as u64);
    for (entity, anchor) in &placement_claims.claims {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);
    }
}

pub struct FishermanCottagePlugin;

impl Plugin for FishermanCottagePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaterTiles>()
            .init_resource::<WaterTileClaims>()
            .init_resource::<FishermanCottagePlacementClaims>()
            .add_event::<SetWaterTileEvent>()
            .add_event::<PlaceFishermanCottageEvent>()
            .add_event::<FishermanCottagePlacementRejectedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    apply_water_tile_events_system,
                    place_fisherman_cottage_system,
                    fisherman_cottage_build_tick_system,
                    fisherman_cottage_food_production_system,
                    fisherman_cottage_checksum_system,
                )
                    .chain(),
            );
    }
}
// Sources: vault/buildings/advanced_farm.md, vault/buildings/farm.md

use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::SimChecksumState;

const ADVANCED_FARM_HP: I32F32 = I32F32::lit("500");
const ADVANCED_FARM_DEFENSES_LIFE: I32F32 = I32F32::lit("125");
const ADVANCED_FARM_WATCH_RANGE: I32F32 = I32F32::lit("7");
const ADVANCED_FARM_ENERGY_COST: i32 = 30;
const ADVANCED_FARM_WOOD_COST: i32 = 30;
const ADVANCED_FARM_STONE_COST: i32 = 20;
const ADVANCED_FARM_IRON_COST: i32 = 20;
const ADVANCED_FARM_OIL_COST: i32 = 20;
const ADVANCED_FARM_GOLD_COST: i32 = 1200;
const ADVANCED_FARM_BUILD_TIME_SECONDS: i32 = 60;
const ADVANCED_FARM_UPGRADE_TIME_SECONDS: i32 = 26;
const ADVANCED_FARM_BUILDING_SIZE_TILES: i32 = 2;
const ADVANCED_FARM_PLOT_RADIUS_TILES: i32 = 2;
const ADVANCED_FARM_MIN_SPACING_TILES: i32 = 4;
const ADVANCED_FARM_FOOD_PER_PLOT: I32F32 = I32F32::lit("4");
const ADVANCED_FARM_PFOOD_MIN: I32F32 = I32F32::lit("4");
const ADVANCED_FARM_PFOOD_MAX: I32F32 = I32F32::lit("128");
const ADVANCED_FARM_PCOLONISTS: i32 = 24;
const SIM_HZ: i32 = 25;

#[derive(Component, Default)]
pub struct Farm;

#[derive(Component, Default)]
pub struct AdvancedFarm;

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
            hp: ADVANCED_FARM_HP,
            defenses_life: ADVANCED_FARM_DEFENSES_LIFE,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct AdvancedFarmBuildState {
    pub build_ticks_remaining: i32,
    pub upgrading_from_farm: bool,
    pub completed: bool,
}

#[derive(Component, Clone, Copy)]
pub struct AdvancedFarmEconomy {
    pub energy_cost: i32,
    pub wood_cost: i32,
    pub stone_cost: i32,
    pub iron_cost: i32,
    pub oil_cost: i32,
    pub gold_cost: i32,
    pub pcolonists: i32,
}

impl Default for AdvancedFarmEconomy {
    fn default() -> Self {
        Self {
            energy_cost: ADVANCED_FARM_ENERGY_COST,
            wood_cost: ADVANCED_FARM_WOOD_COST,
            stone_cost: ADVANCED_FARM_STONE_COST,
            iron_cost: ADVANCED_FARM_IRON_COST,
            oil_cost: ADVANCED_FARM_OIL_COST,
            gold_cost: ADVANCED_FARM_GOLD_COST,
            pcolonists: ADVANCED_FARM_PCOLONISTS,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct AdvancedFarmOutput {
    pub food: I32F32,
    pub min_food: I32F32,
    pub max_food: I32F32,
}

impl Default for AdvancedFarmOutput {
    fn default() -> Self {
        Self {
            food: I32F32::ZERO,
            min_food: ADVANCED_FARM_PFOOD_MIN,
            max_food: ADVANCED_FARM_PFOOD_MAX,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct AdvancedFarmFootprint {
    pub size_tiles: i32,
    pub watch_range: I32F32,
}

impl Default for AdvancedFarmFootprint {
    fn default() -> Self {
        Self {
            size_tiles: ADVANCED_FARM_BUILDING_SIZE_TILES,
            watch_range: ADVANCED_FARM_WATCH_RANGE,
        }
    }
}

#[derive(Component, Clone, Default)]
pub struct AdvancedFarmPlots {
    pub wheat_tiles: BTreeSet<(i32, i32)>,
}

#[derive(Resource, Default, Clone)]
pub struct GrassTiles {
    pub tiles: BTreeSet<(i32, i32)>,
}

#[derive(Resource, Default, Clone)]
pub struct TileOccupancy {
    pub blocked_tiles: BTreeSet<(i32, i32)>,
}

#[derive(Resource, Default, Clone)]
pub struct AdvancedFarmPlacementClaims {
    pub claims: BTreeMap<Entity, BuildingAnchor>,
}

#[derive(Event, Clone, Copy)]
pub struct SetGrassTileEvent {
    pub tile_x: i32,
    pub tile_y: i32,
    pub is_grass: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetTileBlockedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
    pub blocked: bool,
}

#[derive(Event, Clone, Copy)]
pub struct PlaceAdvancedFarmEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct UpgradeFarmToAdvancedFarmEvent {
    pub farm_entity: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct AdvancedFarmPlacementRejectedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

fn build_seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn footprint_tiles(anchor: BuildingAnchor) -> impl Iterator<Item = (i32, i32)> {
    let min_x = anchor.x;
    let min_y = anchor.y;
    let max_x = anchor.x + ADVANCED_FARM_BUILDING_SIZE_TILES - 1;
    let max_y = anchor.y + ADVANCED_FARM_BUILDING_SIZE_TILES - 1;

    (min_x..=max_x).flat_map(move |x| (min_y..=max_y).map(move |y| (x, y)))
}

fn max_axis_delta(a: BuildingAnchor, b: BuildingAnchor) -> i32 {
    let dx = (a.x - b.x).abs();
    let dy = (a.y - b.y).abs();
    dx.max(dy)
}

fn is_valid_spacing(anchor: BuildingAnchor, claims: &AdvancedFarmPlacementClaims) -> bool {
    for existing in claims.claims.values() {
        if max_axis_delta(anchor, *existing) < ADVANCED_FARM_MIN_SPACING_TILES {
            return false;
        }
    }
    true
}

fn footprint_is_unblocked(anchor: BuildingAnchor, occupancy: &TileOccupancy) -> bool {
    for tile in footprint_tiles(anchor) {
        if occupancy.blocked_tiles.contains(&tile) {
            return false;
        }
    }
    true
}

fn apply_grass_tile_events_system(
    mut events: EventReader<SetGrassTileEvent>,
    mut grass: ResMut<GrassTiles>,
) {
    for ev in events.read() {
        let tile = (ev.tile_x, ev.tile_y);
        if ev.is_grass {
            grass.tiles.insert(tile);
        } else {
            grass.tiles.remove(&tile);
        }
    }
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

fn place_advanced_farm_system(
    mut commands: Commands,
    mut events: EventReader<PlaceAdvancedFarmEvent>,
    mut rejected: EventWriter<AdvancedFarmPlacementRejectedEvent>,
    mut claims: ResMut<AdvancedFarmPlacementClaims>,
    occupancy: Res<TileOccupancy>,
) {
    for ev in events.read() {
        let anchor = BuildingAnchor {
            x: ev.tile_x,
            y: ev.tile_y,
        };

        if !is_valid_spacing(anchor, &claims) || !footprint_is_unblocked(anchor, &occupancy) {
            rejected.send(AdvancedFarmPlacementRejectedEvent {
                tile_x: ev.tile_x,
                tile_y: ev.tile_y,
            });
            continue;
        }

        let entity = commands
            .spawn((
                AdvancedFarm,
                anchor,
                BuildingHealth::default(),
                AdvancedFarmBuildState {
                    build_ticks_remaining: build_seconds_to_ticks(ADVANCED_FARM_BUILD_TIME_SECONDS),
                    upgrading_from_farm: false,
                    completed: false,
                },
                AdvancedFarmEconomy::default(),
                AdvancedFarmOutput::default(),
                AdvancedFarmFootprint::default(),
                AdvancedFarmPlots::default(),
            ))
            .id();

        claims.claims.insert(entity, anchor);
    }
}

fn upgrade_farm_to_advanced_farm_system(
    mut commands: Commands,
    mut events: EventReader<UpgradeFarmToAdvancedFarmEvent>,
    farms: Query<(Entity, &BuildingAnchor), With<Farm>>,
    mut claims: ResMut<AdvancedFarmPlacementClaims>,
) {
    for ev in events.read() {
        let Ok((farm_entity, farm_anchor)) = farms.get(ev.farm_entity) else {
            continue;
        };

        if !is_valid_spacing(*farm_anchor, &claims) {
            continue;
        }

        commands.entity(farm_entity).remove::<Farm>();
        commands.entity(farm_entity).insert((
            AdvancedFarm,
            BuildingHealth::default(),
            AdvancedFarmBuildState {
                build_ticks_remaining: build_seconds_to_ticks(ADVANCED_FARM_UPGRADE_TIME_SECONDS),
                upgrading_from_farm: true,
                completed: false,
            },
            AdvancedFarmEconomy::default(),
            AdvancedFarmOutput::default(),
            AdvancedFarmFootprint::default(),
            AdvancedFarmPlots::default(),
        ));

        claims.claims.insert(farm_entity, *farm_anchor);
    }
}

fn advanced_farm_build_tick_system(
    mut farms: Query<&mut AdvancedFarmBuildState, With<AdvancedFarm>>,
) {
    for mut state in &mut farms {
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

fn advanced_farm_wheat_plot_and_output_system(
    mut farms: Query<
        (&BuildingAnchor, &AdvancedFarmBuildState, &mut AdvancedFarmPlots, &mut AdvancedFarmOutput),
        With<AdvancedFarm>,
    >,
    grass: Res<GrassTiles>,
    occupancy: Res<TileOccupancy>,
) {
    for (anchor, build_state, mut plots, mut output) in &mut farms {
        if !build_state.completed {
            plots.wheat_tiles.clear();
            output.food = I32F32::ZERO;
            continue;
        }

        let mut assigned: BTreeSet<(i32, i32)> = BTreeSet::new();
        let min_x = anchor.x - ADVANCED_FARM_PLOT_RADIUS_TILES;
        let max_x = anchor.x + ADVANCED_FARM_BUILDING_SIZE_TILES - 1 + ADVANCED_FARM_PLOT_RADIUS_TILES;
        let min_y = anchor.y - ADVANCED_FARM_PLOT_RADIUS_TILES;
        let max_y = anchor.y + ADVANCED_FARM_BUILDING_SIZE_TILES - 1 + ADVANCED_FARM_PLOT_RADIUS_TILES;

        for x in min_x..=max_x {
            for y in min_y..=max_y {
                let tile = (x, y);
                if !grass.tiles.contains(&tile) {
                    continue;
                }

                let adjacent_to_footprint = x >= anchor.x - 1
                    && x <= anchor.x + ADVANCED_FARM_BUILDING_SIZE_TILES
                    && y >= anchor.y - 1
                    && y <= anchor.y + ADVANCED_FARM_BUILDING_SIZE_TILES;

                if !adjacent_to_footprint && occupancy.blocked_tiles.contains(&tile) {
                    continue;
                }

                assigned.insert(tile);
            }
        }

        plots.wheat_tiles = assigned;
        let food = ADVANCED_FARM_FOOD_PER_PLOT * I32F32::from_num(plots.wheat_tiles.len() as i32);
        output.food = food.min(output.max_food).max(output.min_food);
    }
}

fn advanced_farm_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    farms: Query<
        (
            Entity,
            &BuildingAnchor,
            &BuildingHealth,
            &AdvancedFarmBuildState,
            &AdvancedFarmEconomy,
            &AdvancedFarmOutput,
            &AdvancedFarmFootprint,
            &AdvancedFarmPlots,
        ),
        With<AdvancedFarm>,
    >,
    claims: Res<AdvancedFarmPlacementClaims>,
) {
    for (entity, anchor, hp, build, eco, out, footprint, plots) in &farms {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);

        checksum.accumulate(hp.hp.to_bits() as u64);
        checksum.accumulate(hp.defenses_life.to_bits() as u64);

        checksum.accumulate(build.build_ticks_remaining as u64);
        checksum.accumulate(u64::from(build.upgrading_from_farm));
        checksum.accumulate(u64::from(build.completed));

        checksum.accumulate(eco.energy_cost as u64);
        checksum.accumulate(eco.wood_cost as u64);
        checksum.accumulate(eco.stone_cost as u64);
        checksum.accumulate(eco.iron_cost as u64);
        checksum.accumulate(eco.oil_cost as u64);
        checksum.accumulate(eco.gold_cost as u64);
        checksum.accumulate(eco.pcolonists as u64);

        checksum.accumulate(out.food.to_bits() as u64);
        checksum.accumulate(out.min_food.to_bits() as u64);
        checksum.accumulate(out.max_food.to_bits() as u64);

        checksum.accumulate(footprint.size_tiles as u64);
        checksum.accumulate(footprint.watch_range.to_bits() as u64);

        checksum.accumulate(plots.wheat_tiles.len() as u64);
        for (x, y) in &plots.wheat_tiles {
            checksum.accumulate(*x as u64);
            checksum.accumulate(*y as u64);
        }
    }

    checksum.accumulate(claims.claims.len() as u64);
    for (entity, anchor) in &claims.claims {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);
    }
}

pub struct AdvancedFarmPlugin;

impl Plugin for AdvancedFarmPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GrassTiles>()
            .init_resource::<TileOccupancy>()
            .init_resource::<AdvancedFarmPlacementClaims>()
            .add_event::<SetGrassTileEvent>()
            .add_event::<SetTileBlockedEvent>()
            .add_event::<PlaceAdvancedFarmEvent>()
            .add_event::<UpgradeFarmToAdvancedFarmEvent>()
            .add_event::<AdvancedFarmPlacementRejectedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    apply_grass_tile_events_system,
                    apply_tile_block_events_system,
                    place_advanced_farm_system,
                    upgrade_farm_to_advanced_farm_system,
                    advanced_farm_build_tick_system,
                    advanced_farm_wheat_plot_and_output_system,
                    advanced_farm_checksum_system,
                )
                    .chain(),
            );
    }
}
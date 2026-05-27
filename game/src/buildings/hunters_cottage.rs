// Sources: vault/buildings/hunters_cottage.md

use std::collections::BTreeMap;

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, SimChecksumState};

const HUNTERS_COTTAGE_HP: I32F32 = I32F32::lit("160");
const HUNTERS_COTTAGE_DEFENSES_LIFE: I32F32 = I32F32::lit("40");
const HUNTERS_COTTAGE_WATCH_RANGE: I32F32 = I32F32::lit("6");

const HUNTERS_COTTAGE_ENERGY_COST: i32 = 1;
const HUNTERS_COTTAGE_GOLD_COST: i32 = 80;
const HUNTERS_COTTAGE_BUILD_TIME_SECONDS: i32 = 30;
const HUNTERS_COTTAGE_SIZE_TILES: i32 = 1;
const HUNTERS_COTTAGE_WORKERS_USED: i32 = 0;

const HUNTERS_COTTAGE_PFOOD_MIN: i32 = 1;
const HUNTERS_COTTAGE_PFOOD_MAX: i32 = 20;
const HUNTERS_COTTAGE_PGOLD: i32 = 2;
const HUNTERS_COTTAGE_PENERGY: i32 = 1;
const HUNTERS_COTTAGE_PCOLONISTS: i32 = 2;

const HUNTING_SCALE: I32F32 = I32F32::lit("0.8");
const DARK_MOORLAND_DIRT: I32F32 = I32F32::lit("0.15");
const DARK_MOORLAND_GRASS: I32F32 = I32F32::lit("0.30");
const DARK_MOORLAND_TREE: I32F32 = I32F32::lit("0.60");

const PEACEFUL_LOWLANDS_DIRT: I32F32 = I32F32::lit("0.10");
const PEACEFUL_LOWLANDS_GRASS: I32F32 = I32F32::lit("0.30");
const PEACEFUL_LOWLANDS_TREE: I32F32 = I32F32::lit("0.50");

const FROZEN_HIGHLANDS_DIRT: I32F32 = I32F32::lit("0.00");
const FROZEN_HIGHLANDS_GRASS: I32F32 = I32F32::lit("0.40");
const FROZEN_HIGHLANDS_TREE: I32F32 = I32F32::lit("0.60");

const DESOLATED_WASTELAND_DIRT: I32F32 = I32F32::lit("0.00");
const DESOLATED_WASTELAND_GRASS: I32F32 = I32F32::lit("0.30");
const DESOLATED_WASTELAND_TREE: I32F32 = I32F32::lit("0.60");

const SIM_HZ: i32 = 25;

#[derive(Component, Default)]
pub struct HuntersCottage;

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
            defenses_life: HUNTERS_COTTAGE_DEFENSES_LIFE,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct HuntersCottageBuildState {
    pub build_ticks_remaining: i32,
    pub completed: bool,
}

impl Default for HuntersCottageBuildState {
    fn default() -> Self {
        Self {
            build_ticks_remaining: seconds_to_ticks(HUNTERS_COTTAGE_BUILD_TIME_SECONDS),
            completed: false,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct HuntersCottageEconomy {
    pub energy_cost: i32,
    pub gold_cost: i32,
    pub workers_used: i32,
    pub pfood_min: i32,
    pub pfood_max: i32,
    pub pgold: i32,
    pub penergy: i32,
    pub pcolonists: i32,
}

impl Default for HuntersCottageEconomy {
    fn default() -> Self {
        Self {
            energy_cost: HUNTERS_COTTAGE_ENERGY_COST,
            gold_cost: HUNTERS_COTTAGE_GOLD_COST,
            workers_used: HUNTERS_COTTAGE_WORKERS_USED,
            pfood_min: HUNTERS_COTTAGE_PFOOD_MIN,
            pfood_max: HUNTERS_COTTAGE_PFOOD_MAX,
            pgold: HUNTERS_COTTAGE_PGOLD,
            penergy: HUNTERS_COTTAGE_PENERGY,
            pcolonists: HUNTERS_COTTAGE_PCOLONISTS,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct HuntersCottageFootprint {
    pub size_tiles: i32,
    pub watch_range: I32F32,
}

impl Default for HuntersCottageFootprint {
    fn default() -> Self {
        Self {
            size_tiles: HUNTERS_COTTAGE_SIZE_TILES,
            watch_range: HUNTERS_COTTAGE_WATCH_RANGE,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct HuntersCottageTerrainSamples {
    pub dirt_cells: i32,
    pub grass_cells: i32,
    pub tree_cells: i32,
    pub obstruction_penalty_food: I32F32,
}

#[derive(Component, Clone, Copy, Default)]
pub struct HuntersCottageProductionState {
    pub current_food_output: i32,
    pub raw_food_output: I32F32,
}

#[derive(Component, Clone, Copy)]
pub enum MapTheme {
    DarkMoorland,
    PeacefulLowlands,
    FrozenHighlands,
    DesolatedWasteland,
}

impl Default for MapTheme {
    fn default() -> Self {
        Self::PeacefulLowlands
    }
}

#[derive(Resource, Default, Clone)]
pub struct HuntersCottagePlacementClaims {
    pub claims: BTreeMap<Entity, BuildingAnchor>,
}

#[derive(Event, Clone, Copy)]
pub struct PlaceHuntersCottageEvent {
    pub tile_x: i32,
    pub tile_y: i32,
    pub map_theme: MapTheme,
}

#[derive(Event, Clone, Copy)]
pub struct HuntersCottagePlacementRejectedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct SetHuntersCottageTerrainSamplesEvent {
    pub cottage_entity: Entity,
    pub dirt_cells: i32,
    pub grass_cells: i32,
    pub tree_cells: i32,
    pub obstruction_penalty_food: I32F32,
}

fn seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn theme_modifiers(theme: MapTheme) -> (I32F32, I32F32, I32F32) {
    match theme {
        MapTheme::DarkMoorland => (DARK_MOORLAND_DIRT, DARK_MOORLAND_GRASS, DARK_MOORLAND_TREE),
        MapTheme::PeacefulLowlands => (
            PEACEFUL_LOWLANDS_DIRT,
            PEACEFUL_LOWLANDS_GRASS,
            PEACEFUL_LOWLANDS_TREE,
        ),
        MapTheme::FrozenHighlands => (
            FROZEN_HIGHLANDS_DIRT,
            FROZEN_HIGHLANDS_GRASS,
            FROZEN_HIGHLANDS_TREE,
        ),
        MapTheme::DesolatedWasteland => (
            DESOLATED_WASTELAND_DIRT,
            DESOLATED_WASTELAND_GRASS,
            DESOLATED_WASTELAND_TREE,
        ),
    }
}

fn place_hunters_cottage_system(
    mut commands: Commands,
    mut events: EventReader<PlaceHuntersCottageEvent>,
    mut rejected: EventWriter<HuntersCottagePlacementRejectedEvent>,
    mut claims: ResMut<HuntersCottagePlacementClaims>,
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
            rejected.send(HuntersCottagePlacementRejectedEvent {
                tile_x: ev.tile_x,
                tile_y: ev.tile_y,
            });
            continue;
        }

        let entity = commands
            .spawn((
                HuntersCottage,
                anchor,
                Health::full(HUNTERS_COTTAGE_HP),
                BuildingDurability::default(),
                HuntersCottageBuildState::default(),
                HuntersCottageEconomy::default(),
                HuntersCottageFootprint::default(),
                HuntersCottageTerrainSamples::default(),
                HuntersCottageProductionState::default(),
                ev.map_theme,
            ))
            .id();

        claims.claims.insert(entity, anchor);
    }
}

fn set_hunters_cottage_terrain_samples_system(
    mut events: EventReader<SetHuntersCottageTerrainSamplesEvent>,
    mut cottages: Query<&mut HuntersCottageTerrainSamples, With<HuntersCottage>>,
) {
    for ev in events.read() {
        let Ok(mut samples) = cottages.get_mut(ev.cottage_entity) else {
            continue;
        };

        samples.dirt_cells = ev.dirt_cells.max(0);
        samples.grass_cells = ev.grass_cells.max(0);
        samples.tree_cells = ev.tree_cells.max(0);
        samples.obstruction_penalty_food = if ev.obstruction_penalty_food >= I32F32::ZERO {
            ev.obstruction_penalty_food
        } else {
            I32F32::ZERO
        };
    }
}

fn hunters_cottage_build_tick_system(
    mut cottages: Query<&mut HuntersCottageBuildState, With<HuntersCottage>>,
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

fn hunters_cottage_food_production_system(
    mut cottages: Query<
        (
            &HuntersCottageBuildState,
            &HuntersCottageEconomy,
            &HuntersCottageTerrainSamples,
            &MapTheme,
            &mut HuntersCottageProductionState,
        ),
        With<HuntersCottage>,
    >,
) {
    for (build, economy, terrain, map_theme, mut production) in &mut cottages {
        if !build.completed {
            production.current_food_output = 0;
            production.raw_food_output = I32F32::ZERO;
            continue;
        }

        let (dirt_modifier, grass_modifier, tree_modifier) = theme_modifiers(*map_theme);

        let weighted_cells = I32F32::from_num(terrain.dirt_cells) * dirt_modifier
            + I32F32::from_num(terrain.grass_cells) * grass_modifier
            + I32F32::from_num(terrain.tree_cells) * tree_modifier;

        let mut raw_food = HUNTING_SCALE * weighted_cells - terrain.obstruction_penalty_food;
        if raw_food < I32F32::ZERO {
            raw_food = I32F32::ZERO;
        }

        let mut food_output = raw_food.to_num::<i32>();
        if food_output < economy.pfood_min {
            food_output = economy.pfood_min;
        }
        if food_output > economy.pfood_max {
            food_output = economy.pfood_max;
        }

        production.current_food_output = food_output;
        production.raw_food_output = raw_food;
    }
}

fn hunters_cottage_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    cottages: Query<
        (
            Entity,
            &BuildingAnchor,
            &Health,
            &BuildingDurability,
            &HuntersCottageBuildState,
            &HuntersCottageEconomy,
            &HuntersCottageFootprint,
            &HuntersCottageTerrainSamples,
            &HuntersCottageProductionState,
            &MapTheme,
        ),
        With<HuntersCottage>,
    >,
    claims: Res<HuntersCottagePlacementClaims>,
) {
    for (
        entity,
        anchor,
        health,
        durability,
        build,
        economy,
        footprint,
        terrain,
        production,
        map_theme,
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
        checksum.accumulate(economy.workers_used as u64);
        checksum.accumulate(economy.pfood_min as u64);
        checksum.accumulate(economy.pfood_max as u64);
        checksum.accumulate(economy.pgold as u64);
        checksum.accumulate(economy.penergy as u64);
        checksum.accumulate(economy.pcolonists as u64);

        checksum.accumulate(footprint.size_tiles as u64);
        checksum.accumulate(footprint.watch_range.to_bits() as u64);

        checksum.accumulate(terrain.dirt_cells as u64);
        checksum.accumulate(terrain.grass_cells as u64);
        checksum.accumulate(terrain.tree_cells as u64);
        checksum.accumulate(terrain.obstruction_penalty_food.to_bits() as u64);

        checksum.accumulate(production.current_food_output as u64);
        checksum.accumulate(production.raw_food_output.to_bits() as u64);

        let map_theme_value = match map_theme {
            MapTheme::DarkMoorland => 0_u64,
            MapTheme::PeacefulLowlands => 1_u64,
            MapTheme::FrozenHighlands => 2_u64,
            MapTheme::DesolatedWasteland => 3_u64,
        };
        checksum.accumulate(map_theme_value);
    }

    checksum.accumulate(claims.claims.len() as u64);
    for (entity, anchor) in &claims.claims {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);
    }
}

pub struct HuntersCottagePlugin;

impl Plugin for HuntersCottagePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HuntersCottagePlacementClaims>()
            .add_event::<PlaceHuntersCottageEvent>()
            .add_event::<HuntersCottagePlacementRejectedEvent>()
            .add_event::<SetHuntersCottageTerrainSamplesEvent>()
            .add_systems(
                FixedUpdate,
                (
                    place_hunters_cottage_system,
                    set_hunters_cottage_terrain_samples_system,
                    hunters_cottage_build_tick_system,
                    hunters_cottage_food_production_system,
                    hunters_cottage_checksum_system,
                )
                    .chain(),
            );
    }
}
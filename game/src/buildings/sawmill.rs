// Sources: vault/buildings/sawmill.md

use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, SimChecksumState};

const SAWMILL_HP: I32F32 = I32F32::lit("200");
const SAWMILL_DEFENSES_LIFE: I32F32 = I32F32::lit("50");
const SAWMILL_WATCH_RANGE: I32F32 = I32F32::lit("6");

const SAWMILL_ENERGY_COST: i32 = 4;
const SAWMILL_WOOD_COST: i32 = 0;
const SAWMILL_STONE_COST: i32 = 0;
const SAWMILL_IRON_COST: i32 = 0;
const SAWMILL_OIL_COST: i32 = 0;
const SAWMILL_GOLD_COST: i32 = 300;
const SAWMILL_BUILD_TIME_SECONDS: i32 = 39;
const SAWMILL_WORKERS: i32 = 4;

const SAWMILL_PFOOD: i32 = 0;
const SAWMILL_PWOOD: i32 = 0;
const SAWMILL_PSTONE: i32 = 0;
const SAWMILL_PIRON: i32 = 0;
const SAWMILL_POIL: i32 = 0;
const SAWMILL_PGOLD: i32 = 0;
const SAWMILL_PENERGY: i32 = 0;
const SAWMILL_PCOLONISTS: i32 = 0;
const SAWMILL_MAINTENANCE_GOLD: i32 = 4;

const SAWMILL_WOOD_MIN: i32 = 1;
const SAWMILL_WOOD_MAX: i32 = 29;
const SAWMILL_SIZE_TILES: i32 = 2;
const SAWMILL_GATHER_RADIUS_TILES: i32 = 3;
const SIM_HZ: i32 = 25;

#[derive(Component, Default)]
pub struct Sawmill;

#[derive(Component, Clone, Copy, Default)]
pub struct BuildingAnchor {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Copy)]
pub struct SawmillCoreStats {
    pub defenses_life: I32F32,
    pub watch_range: I32F32,
    pub health: Health,
}

impl Default for SawmillCoreStats {
    fn default() -> Self {
        Self {
            defenses_life: SAWMILL_DEFENSES_LIFE,
            watch_range: SAWMILL_WATCH_RANGE,
            health: Health::full(SAWMILL_HP),
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct SawmillBuildState {
    pub build_ticks_remaining: i32,
    pub completed: bool,
}

#[derive(Component, Clone, Copy)]
pub struct SawmillEconomy {
    pub energy_cost: i32,
    pub wood_cost: i32,
    pub stone_cost: i32,
    pub iron_cost: i32,
    pub oil_cost: i32,
    pub gold_cost: i32,
    pub workers: i32,
    pub pfood: i32,
    pub pwood: i32,
    pub pstone: i32,
    pub piron: i32,
    pub poil: i32,
    pub pgold: i32,
    pub penergy: i32,
    pub pcolonists: i32,
    pub maintenance_gold: i32,
}

impl Default for SawmillEconomy {
    fn default() -> Self {
        Self {
            energy_cost: SAWMILL_ENERGY_COST,
            wood_cost: SAWMILL_WOOD_COST,
            stone_cost: SAWMILL_STONE_COST,
            iron_cost: SAWMILL_IRON_COST,
            oil_cost: SAWMILL_OIL_COST,
            gold_cost: SAWMILL_GOLD_COST,
            workers: SAWMILL_WORKERS,
            pfood: SAWMILL_PFOOD,
            pwood: SAWMILL_PWOOD,
            pstone: SAWMILL_PSTONE,
            piron: SAWMILL_PIRON,
            poil: SAWMILL_POIL,
            pgold: SAWMILL_PGOLD,
            penergy: SAWMILL_PENERGY,
            pcolonists: SAWMILL_PCOLONISTS,
            maintenance_gold: SAWMILL_MAINTENANCE_GOLD,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct SawmillOutput {
    pub wood_per_cycle: i32,
}

impl Default for SawmillOutput {
    fn default() -> Self {
        Self { wood_per_cycle: 0 }
    }
}

#[derive(Component, Clone, Copy)]
pub struct SawmillFootprint {
    pub size_tiles: i32,
    pub gather_radius_tiles: i32,
}

impl Default for SawmillFootprint {
    fn default() -> Self {
        Self {
            size_tiles: SAWMILL_SIZE_TILES,
            gather_radius_tiles: SAWMILL_GATHER_RADIUS_TILES,
        }
    }
}

#[derive(Resource, Default, Clone)]
pub struct SawmillForestTiles {
    pub tiles: BTreeSet<(i32, i32)>,
}

#[derive(Resource, Default, Clone)]
pub struct SawmillPlacementClaims {
    pub claims: BTreeMap<Entity, BuildingAnchor>,
}

#[derive(Event, Clone, Copy)]
pub struct SetSawmillForestTileEvent {
    pub tile_x: i32,
    pub tile_y: i32,
    pub is_forest: bool,
}

#[derive(Event, Clone, Copy)]
pub struct PlaceSawmillEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct RemoveSawmillEvent {
    pub entity: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct SawmillPlacementRejectedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

fn seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn gather_area_min(anchor: BuildingAnchor) -> (i32, i32) {
    (
        anchor.x - SAWMILL_GATHER_RADIUS_TILES,
        anchor.y - SAWMILL_GATHER_RADIUS_TILES,
    )
}

fn gather_area_max(anchor: BuildingAnchor) -> (i32, i32) {
    (
        anchor.x + SAWMILL_SIZE_TILES - 1 + SAWMILL_GATHER_RADIUS_TILES,
        anchor.y + SAWMILL_SIZE_TILES - 1 + SAWMILL_GATHER_RADIUS_TILES,
    )
}

fn gather_areas_overlap(a: BuildingAnchor, b: BuildingAnchor) -> bool {
    let (a_min_x, a_min_y) = gather_area_min(a);
    let (a_max_x, a_max_y) = gather_area_max(a);
    let (b_min_x, b_min_y) = gather_area_min(b);
    let (b_max_x, b_max_y) = gather_area_max(b);

    !(a_max_x < b_min_x || b_max_x < a_min_x || a_max_y < b_min_y || b_max_y < a_min_y)
}

fn valid_sawmill_placement(anchor: BuildingAnchor, claims: &SawmillPlacementClaims) -> bool {
    for existing in claims.claims.values() {
        if gather_areas_overlap(anchor, *existing) {
            return false;
        }
    }
    true
}

fn apply_sawmill_forest_tile_events_system(
    mut events: EventReader<SetSawmillForestTileEvent>,
    mut forest: ResMut<SawmillForestTiles>,
) {
    for ev in events.read() {
        let tile = (ev.tile_x, ev.tile_y);
        if ev.is_forest {
            forest.tiles.insert(tile);
        } else {
            forest.tiles.remove(&tile);
        }
    }
}

fn place_sawmill_system(
    mut commands: Commands,
    mut events: EventReader<PlaceSawmillEvent>,
    mut rejected: EventWriter<SawmillPlacementRejectedEvent>,
    mut claims: ResMut<SawmillPlacementClaims>,
) {
    for ev in events.read() {
        let anchor = BuildingAnchor {
            x: ev.tile_x,
            y: ev.tile_y,
        };

        if !valid_sawmill_placement(anchor, &claims) {
            rejected.send(SawmillPlacementRejectedEvent {
                tile_x: ev.tile_x,
                tile_y: ev.tile_y,
            });
            continue;
        }

        let entity = commands
            .spawn((
                Sawmill,
                anchor,
                SawmillCoreStats::default(),
                SawmillBuildState {
                    build_ticks_remaining: seconds_to_ticks(SAWMILL_BUILD_TIME_SECONDS),
                    completed: false,
                },
                SawmillEconomy::default(),
                SawmillOutput::default(),
                SawmillFootprint::default(),
            ))
            .id();

        claims.claims.insert(entity, anchor);
    }
}

fn remove_sawmill_system(
    mut commands: Commands,
    mut events: EventReader<RemoveSawmillEvent>,
    mills: Query<(), With<Sawmill>>,
    mut claims: ResMut<SawmillPlacementClaims>,
) {
    for ev in events.read() {
        if mills.get(ev.entity).is_err() {
            continue;
        }

        claims.claims.remove(&ev.entity);
        commands.entity(ev.entity).despawn();
    }
}

fn sawmill_build_tick_system(mut mills: Query<&mut SawmillBuildState, With<Sawmill>>) {
    for mut state in &mut mills {
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

fn sawmill_output_system(
    mut mills: Query<(&BuildingAnchor, &SawmillBuildState, &mut SawmillOutput), With<Sawmill>>,
    forest: Res<SawmillForestTiles>,
) {
    for (anchor, build, mut output) in &mut mills {
        if !build.completed {
            output.wood_per_cycle = 0;
            continue;
        }

        let (min_x, min_y) = gather_area_min(*anchor);
        let (max_x, max_y) = gather_area_max(*anchor);

        let mut tree_tiles = 0;
        for x in min_x..=max_x {
            for y in min_y..=max_y {
                if forest.tiles.contains(&(x, y)) {
                    tree_tiles += 1;
                }
            }
        }

        let produced = (tree_tiles + 1) / 2;
        output.wood_per_cycle = produced.clamp(SAWMILL_WOOD_MIN, SAWMILL_WOOD_MAX);
    }
}

fn sawmill_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    mills: Query<
        (
            Entity,
            &BuildingAnchor,
            &SawmillCoreStats,
            &SawmillBuildState,
            &SawmillEconomy,
            &SawmillOutput,
            &SawmillFootprint,
        ),
        With<Sawmill>,
    >,
    forest: Res<SawmillForestTiles>,
    claims: Res<SawmillPlacementClaims>,
) {
    for (entity, anchor, core, build, economy, output, footprint) in &mills {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);

        checksum.accumulate(core.defenses_life.to_bits() as u64);
        checksum.accumulate(core.watch_range.to_bits() as u64);
        checksum.accumulate(core.health.current.to_bits() as u64);
        checksum.accumulate(core.health.max.to_bits() as u64);

        checksum.accumulate(build.build_ticks_remaining as u64);
        checksum.accumulate(u64::from(build.completed));

        checksum.accumulate(economy.energy_cost as u64);
        checksum.accumulate(economy.wood_cost as u64);
        checksum.accumulate(economy.stone_cost as u64);
        checksum.accumulate(economy.iron_cost as u64);
        checksum.accumulate(economy.oil_cost as u64);
        checksum.accumulate(economy.gold_cost as u64);
        checksum.accumulate(economy.workers as u64);
        checksum.accumulate(economy.pfood as u64);
        checksum.accumulate(economy.pwood as u64);
        checksum.accumulate(economy.pstone as u64);
        checksum.accumulate(economy.piron as u64);
        checksum.accumulate(economy.poil as u64);
        checksum.accumulate(economy.pgold as u64);
        checksum.accumulate(economy.penergy as u64);
        checksum.accumulate(economy.pcolonists as u64);
        checksum.accumulate(economy.maintenance_gold as u64);

        checksum.accumulate(output.wood_per_cycle as u64);

        checksum.accumulate(footprint.size_tiles as u64);
        checksum.accumulate(footprint.gather_radius_tiles as u64);
    }

    checksum.accumulate(forest.tiles.len() as u64);
    for (x, y) in &forest.tiles {
        checksum.accumulate(*x as u64);
        checksum.accumulate(*y as u64);
    }

    checksum.accumulate(claims.claims.len() as u64);
    for (entity, anchor) in &claims.claims {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);
    }
}

pub struct SawmillPlugin;

impl Plugin for SawmillPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SawmillForestTiles>()
            .init_resource::<SawmillPlacementClaims>()
            .add_event::<SetSawmillForestTileEvent>()
            .add_event::<PlaceSawmillEvent>()
            .add_event::<RemoveSawmillEvent>()
            .add_event::<SawmillPlacementRejectedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    apply_sawmill_forest_tile_events_system,
                    place_sawmill_system,
                    remove_sawmill_system,
                    sawmill_build_tick_system,
                    sawmill_output_system,
                    sawmill_checksum_system,
                )
                    .chain(),
            );
    }
}
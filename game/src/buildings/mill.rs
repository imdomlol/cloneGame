// Sources: vault/buildings/mill.md, vault/buildings/advanced_mill.md

use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, SimChecksumState};

const MILL_HP: I32F32 = I32F32::lit("200");
const MILL_DEFENSES_LIFE: I32F32 = I32F32::lit("50");
const MILL_WATCH_RANGE: I32F32 = I32F32::lit("6");

const MILL_ENERGY_COST: i32 = 0;
const MILL_WOOD_COST: i32 = 20;
const MILL_STONE_COST: i32 = 0;
const MILL_IRON_COST: i32 = 0;
const MILL_OIL_COST: i32 = 0;
const MILL_GOLD_COST: i32 = 300;
const MILL_BUILD_TIME_SECONDS: i32 = 40;
const MILL_WORKERS: i32 = 4;

const MILL_PFOOD: i32 = 0;
const MILL_PWOOD: i32 = 0;
const MILL_PSTONE: i32 = 0;
const MILL_PIRON: i32 = 0;
const MILL_POIL: i32 = 0;
const MILL_PGOLD: i32 = 0;
const MILL_PENERGY: i32 = 30;
const MILL_PCOLONISTS: i32 = 0;
const MILL_GOLD_MAINTENANCE: i32 = 6;

const MILL_SIZE_TILES: i32 = 2;
const MILL_MIN_SPACING_TILES: i32 = 4;

const SIM_HZ: i32 = 25;

#[derive(Component, Default)]
pub struct Mill;

#[derive(Component, Clone, Copy)]
pub struct BuildingAnchor {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Copy)]
pub struct MillCoreStats {
    pub defenses_life: I32F32,
    pub watch_range: I32F32,
    pub health: Health,
}

impl Default for MillCoreStats {
    fn default() -> Self {
        Self {
            defenses_life: MILL_DEFENSES_LIFE,
            watch_range: MILL_WATCH_RANGE,
            health: Health::full(MILL_HP),
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct MillBuildState {
    pub build_ticks_remaining: i32,
    pub completed: bool,
}

#[derive(Component, Clone, Copy)]
pub struct MillEconomy {
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
    pub gold_maintenance: i32,
}

impl Default for MillEconomy {
    fn default() -> Self {
        Self {
            energy_cost: MILL_ENERGY_COST,
            wood_cost: MILL_WOOD_COST,
            stone_cost: MILL_STONE_COST,
            iron_cost: MILL_IRON_COST,
            oil_cost: MILL_OIL_COST,
            gold_cost: MILL_GOLD_COST,
            workers: MILL_WORKERS,
            pfood: MILL_PFOOD,
            pwood: MILL_PWOOD,
            pstone: MILL_PSTONE,
            piron: MILL_PIRON,
            poil: MILL_POIL,
            pgold: MILL_PGOLD,
            penergy: MILL_PENERGY,
            pcolonists: MILL_PCOLONISTS,
            gold_maintenance: MILL_GOLD_MAINTENANCE,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct MillOutput {
    pub penergy: i32,
}

impl Default for MillOutput {
    fn default() -> Self {
        Self { penergy: 0 }
    }
}

#[derive(Component, Clone, Copy)]
pub struct MillFootprint {
    pub size_tiles: i32,
}

impl Default for MillFootprint {
    fn default() -> Self {
        Self {
            size_tiles: MILL_SIZE_TILES,
        }
    }
}

#[derive(Resource, Default, Clone)]
pub struct TileOccupancy {
    pub blocked_tiles: BTreeSet<(i32, i32)>,
}

#[derive(Resource, Default, Clone)]
pub struct MillPlacementClaims {
    pub claims: BTreeMap<Entity, BuildingAnchor>,
}

#[derive(Event, Clone, Copy)]
pub struct SetTileBlockedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
    pub blocked: bool,
}

#[derive(Event, Clone, Copy)]
pub struct PlaceMillEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct RemoveMillEvent {
    pub entity: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct MillPlacementRejectedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

fn seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn footprint_tiles(anchor: BuildingAnchor) -> impl Iterator<Item = (i32, i32)> {
    let min_x = anchor.x;
    let min_y = anchor.y;
    let max_x = anchor.x + MILL_SIZE_TILES - 1;
    let max_y = anchor.y + MILL_SIZE_TILES - 1;
    (min_x..=max_x).flat_map(move |x| (min_y..=max_y).map(move |y| (x, y)))
}

fn max_axis_delta(a: BuildingAnchor, b: BuildingAnchor) -> i32 {
    let dx = (a.x - b.x).abs();
    let dy = (a.y - b.y).abs();
    dx.max(dy)
}

fn is_valid_spacing(anchor: BuildingAnchor, claims: &MillPlacementClaims) -> bool {
    for existing in claims.claims.values() {
        if max_axis_delta(anchor, *existing) < MILL_MIN_SPACING_TILES {
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

fn place_mill_system(
    mut commands: Commands,
    mut events: EventReader<PlaceMillEvent>,
    mut rejected: EventWriter<MillPlacementRejectedEvent>,
    mut claims: ResMut<MillPlacementClaims>,
    occupancy: Res<TileOccupancy>,
) {
    for ev in events.read() {
        let anchor = BuildingAnchor {
            x: ev.tile_x,
            y: ev.tile_y,
        };

        if !is_valid_spacing(anchor, &claims) || !footprint_is_unblocked(anchor, &occupancy) {
            rejected.send(MillPlacementRejectedEvent {
                tile_x: ev.tile_x,
                tile_y: ev.tile_y,
            });
            continue;
        }

        let entity = commands
            .spawn((
                Mill,
                anchor,
                MillCoreStats::default(),
                MillBuildState {
                    build_ticks_remaining: seconds_to_ticks(MILL_BUILD_TIME_SECONDS),
                    completed: false,
                },
                MillEconomy::default(),
                MillOutput::default(),
                MillFootprint::default(),
            ))
            .id();

        claims.claims.insert(entity, anchor);
    }
}

fn remove_mill_system(
    mut commands: Commands,
    mut events: EventReader<RemoveMillEvent>,
    mills: Query<(), With<Mill>>,
    mut claims: ResMut<MillPlacementClaims>,
) {
    for ev in events.read() {
        if mills.get(ev.entity).is_err() {
            continue;
        }

        claims.claims.remove(&ev.entity);
        commands.entity(ev.entity).despawn();
    }
}

fn mill_build_tick_system(mut mills: Query<(&mut MillBuildState, &mut MillOutput), With<Mill>>) {
    for (mut build, mut output) in &mut mills {
        if build.completed {
            output.penergy = MILL_PENERGY;
            continue;
        }

        if build.build_ticks_remaining > 0 {
            build.build_ticks_remaining -= 1;
        }

        if build.build_ticks_remaining <= 0 {
            build.build_ticks_remaining = 0;
            build.completed = true;
            output.penergy = MILL_PENERGY;
        } else {
            output.penergy = 0;
        }
    }
}

fn mill_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    mills: Query<
        (
            Entity,
            &BuildingAnchor,
            &MillCoreStats,
            &MillBuildState,
            &MillEconomy,
            &MillOutput,
            &MillFootprint,
        ),
        With<Mill>,
    >,
    claims: Res<MillPlacementClaims>,
    occupancy: Res<TileOccupancy>,
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
        checksum.accumulate(economy.gold_maintenance as u64);

        checksum.accumulate(output.penergy as u64);

        checksum.accumulate(footprint.size_tiles as u64);
    }

    checksum.accumulate(claims.claims.len() as u64);
    for (entity, anchor) in &claims.claims {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);
    }

    checksum.accumulate(occupancy.blocked_tiles.len() as u64);
    for (x, y) in &occupancy.blocked_tiles {
        checksum.accumulate(*x as u64);
        checksum.accumulate(*y as u64);
    }
}

pub struct MillPlugin;

impl Plugin for MillPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileOccupancy>()
            .init_resource::<MillPlacementClaims>()
            .add_event::<SetTileBlockedEvent>()
            .add_event::<PlaceMillEvent>()
            .add_event::<RemoveMillEvent>()
            .add_event::<MillPlacementRejectedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    apply_tile_block_events_system,
                    place_mill_system,
                    remove_mill_system,
                    mill_build_tick_system,
                    mill_checksum_system,
                )
                    .chain(),
            );
    }
}
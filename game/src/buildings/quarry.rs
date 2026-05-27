// Sources: vault/buildings/quarry.md, vault/buildings/advanced_quarry.md

use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::SimChecksumState;

const QUARRY_HP: I32F32 = I32F32::lit("200");
const QUARRY_DEFENSES_LIFE: I32F32 = I32F32::lit("50");
const QUARRY_WATCH_RANGE: I32F32 = I32F32::lit("6");
const QUARRY_ENERGY_COST: i32 = 4;
const QUARRY_WOOD_COST: i32 = 30;
const QUARRY_STONE_COST: i32 = 0;
const QUARRY_IRON_COST: i32 = 0;
const QUARRY_OIL_COST: i32 = 0;
const QUARRY_GOLD_COST: i32 = 300;
const QUARRY_BUILD_TIME_SECONDS: i32 = 39;
const QUARRY_WORKERS: i32 = 4;
const QUARRY_BUILDING_SIZE_TILES: i32 = 2;
const QUARRY_FIELD_SIDE_TILES: i32 = 6;
const QUARRY_STONE_IRON_PER_TILE: I32F32 = I32F32::lit("0.5");
const QUARRY_GOLD_PER_TILE: i32 = 10;
const SIM_HZ: i32 = 25;

#[derive(Component, Default)]
pub struct Quarry;

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
            hp: QUARRY_HP,
            defenses_life: QUARRY_DEFENSES_LIFE,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct QuarryBuildState {
    pub build_ticks_remaining: i32,
    pub completed: bool,
}

#[derive(Component, Clone, Copy)]
pub struct QuarryEconomy {
    pub energy_cost: i32,
    pub wood_cost: i32,
    pub stone_cost: i32,
    pub iron_cost: i32,
    pub oil_cost: i32,
    pub gold_cost: i32,
    pub workers: i32,
}

impl Default for QuarryEconomy {
    fn default() -> Self {
        Self {
            energy_cost: QUARRY_ENERGY_COST,
            wood_cost: QUARRY_WOOD_COST,
            stone_cost: QUARRY_STONE_COST,
            iron_cost: QUARRY_IRON_COST,
            oil_cost: QUARRY_OIL_COST,
            gold_cost: QUARRY_GOLD_COST,
            workers: QUARRY_WORKERS,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct QuarryOutput {
    pub pstone: i32,
    pub piron: i32,
    pub pgold: i32,
}

impl Default for QuarryOutput {
    fn default() -> Self {
        Self {
            pstone: 0,
            piron: 0,
            pgold: 0,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct QuarryFootprint {
    pub size_tiles: i32,
    pub watch_range: I32F32,
    pub field_side_tiles: i32,
}

impl Default for QuarryFootprint {
    fn default() -> Self {
        Self {
            size_tiles: QUARRY_BUILDING_SIZE_TILES,
            watch_range: QUARRY_WATCH_RANGE,
            field_side_tiles: QUARRY_FIELD_SIDE_TILES,
        }
    }
}

#[derive(Component, Clone, Default)]
pub struct QuarryFieldAssignment {
    pub stone_tiles: BTreeSet<(i32, i32)>,
    pub iron_tiles: BTreeSet<(i32, i32)>,
    pub gold_tiles: BTreeSet<(i32, i32)>,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MineralTileKind {
    Stone,
    Iron,
    Gold,
}

#[derive(Resource, Default, Clone)]
pub struct MineralTiles {
    pub by_tile: BTreeMap<(i32, i32), MineralTileKind>,
}

#[derive(Resource, Default, Clone)]
pub struct TileOccupancy {
    pub blocked_tiles: BTreeSet<(i32, i32)>,
}

#[derive(Resource, Default, Clone)]
pub struct QuarryPlacementClaims {
    pub claims: BTreeMap<Entity, BuildingAnchor>,
}

#[derive(Event, Clone, Copy)]
pub struct SetTileBlockedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
    pub blocked: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetMineralTileEvent {
    pub tile_x: i32,
    pub tile_y: i32,
    pub mineral_kind: i32,
    pub present: bool,
}

#[derive(Event, Clone, Copy)]
pub struct PlaceQuarryEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct RemoveQuarryEvent {
    pub entity: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct QuarryPlacementRejectedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

fn build_seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn footprint_tiles(anchor: BuildingAnchor) -> impl Iterator<Item = (i32, i32)> {
    let min_x = anchor.x;
    let min_y = anchor.y;
    let max_x = anchor.x + QUARRY_BUILDING_SIZE_TILES - 1;
    let max_y = anchor.y + QUARRY_BUILDING_SIZE_TILES - 1;

    (min_x..=max_x).flat_map(move |x| (min_y..=max_y).map(move |y| (x, y)))
}

fn field_tiles(anchor: BuildingAnchor) -> impl Iterator<Item = (i32, i32)> {
    let half = QUARRY_FIELD_SIDE_TILES / 2;
    let center_x = anchor.x + (QUARRY_BUILDING_SIZE_TILES / 2);
    let center_y = anchor.y + (QUARRY_BUILDING_SIZE_TILES / 2);

    let min_x = center_x - half;
    let min_y = center_y - half;
    let max_x = min_x + QUARRY_FIELD_SIDE_TILES - 1;
    let max_y = min_y + QUARRY_FIELD_SIDE_TILES - 1;

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

fn field_has_no_overlap(anchor: BuildingAnchor, claims: &QuarryPlacementClaims) -> bool {
    let mut existing_field_tiles = BTreeSet::new();
    for existing_anchor in claims.claims.values() {
        for tile in field_tiles(*existing_anchor) {
            existing_field_tiles.insert(tile);
        }
    }

    for tile in field_tiles(anchor) {
        if existing_field_tiles.contains(&tile) {
            return false;
        }
    }

    true
}

fn field_has_any_resource(anchor: BuildingAnchor, minerals: &MineralTiles) -> bool {
    for tile in field_tiles(anchor) {
        if minerals.by_tile.contains_key(&tile) {
            return true;
        }
    }
    false
}

fn decode_mineral_kind(raw: i32) -> Option<MineralTileKind> {
    match raw {
        0 => Some(MineralTileKind::Stone),
        1 => Some(MineralTileKind::Iron),
        2 => Some(MineralTileKind::Gold),
        _ => None,
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

fn apply_mineral_tile_events_system(
    mut events: EventReader<SetMineralTileEvent>,
    mut minerals: ResMut<MineralTiles>,
) {
    for ev in events.read() {
        let tile = (ev.tile_x, ev.tile_y);

        if !ev.present {
            minerals.by_tile.remove(&tile);
            continue;
        }

        let Some(kind) = decode_mineral_kind(ev.mineral_kind) else {
            continue;
        };

        minerals.by_tile.insert(tile, kind);
    }
}

fn place_quarry_system(
    mut commands: Commands,
    mut events: EventReader<PlaceQuarryEvent>,
    mut rejected: EventWriter<QuarryPlacementRejectedEvent>,
    mut claims: ResMut<QuarryPlacementClaims>,
    occupancy: Res<TileOccupancy>,
    minerals: Res<MineralTiles>,
) {
    for ev in events.read() {
        let anchor = BuildingAnchor {
            x: ev.tile_x,
            y: ev.tile_y,
        };

        let valid = footprint_is_unblocked(anchor, &occupancy)
            && field_has_no_overlap(anchor, &claims)
            && field_has_any_resource(anchor, &minerals);

        if !valid {
            rejected.send(QuarryPlacementRejectedEvent {
                tile_x: ev.tile_x,
                tile_y: ev.tile_y,
            });
            continue;
        }

        let entity = commands
            .spawn((
                Quarry,
                anchor,
                BuildingHealth::default(),
                QuarryBuildState {
                    build_ticks_remaining: build_seconds_to_ticks(QUARRY_BUILD_TIME_SECONDS),
                    completed: false,
                },
                QuarryEconomy::default(),
                QuarryOutput::default(),
                QuarryFootprint::default(),
                QuarryFieldAssignment::default(),
            ))
            .id();

        claims.claims.insert(entity, anchor);
    }
}

fn remove_quarry_system(
    mut commands: Commands,
    mut events: EventReader<RemoveQuarryEvent>,
    quarries: Query<(), With<Quarry>>,
    mut claims: ResMut<QuarryPlacementClaims>,
) {
    for ev in events.read() {
        if quarries.get(ev.entity).is_err() {
            continue;
        }

        claims.claims.remove(&ev.entity);
        commands.entity(ev.entity).despawn();
    }
}

fn quarry_build_tick_system(
    mut quarries: Query<(&mut QuarryBuildState, &mut QuarryOutput), With<Quarry>>,
) {
    for (mut build, mut output) in &mut quarries {
        if build.completed {
            continue;
        }

        output.pstone = 0;
        output.piron = 0;
        output.pgold = 0;

        if build.build_ticks_remaining > 0 {
            build.build_ticks_remaining -= 1;
        }

        if build.build_ticks_remaining <= 0 {
            build.build_ticks_remaining = 0;
            build.completed = true;
        }
    }
}

fn quarry_field_and_output_system(
    mut quarries: Query<
        (
            &BuildingAnchor,
            &QuarryBuildState,
            &mut QuarryFieldAssignment,
            &mut QuarryOutput,
        ),
        With<Quarry>,
    >,
    minerals: Res<MineralTiles>,
) {
    for (anchor, build, mut assigned, mut output) in &mut quarries {
        if !build.completed {
            assigned.stone_tiles.clear();
            assigned.iron_tiles.clear();
            assigned.gold_tiles.clear();
            output.pstone = 0;
            output.piron = 0;
            output.pgold = 0;
            continue;
        }

        let mut stone_count = 0;
        let mut iron_count = 0;
        let mut gold_count = 0;

        let mut stone_tiles = BTreeSet::new();
        let mut iron_tiles = BTreeSet::new();
        let mut gold_tiles = BTreeSet::new();

        for tile in field_tiles(*anchor) {
            let Some(kind) = minerals.by_tile.get(&tile) else {
                continue;
            };

            match kind {
                MineralTileKind::Stone => {
                    stone_count += 1;
                    stone_tiles.insert(tile);
                }
                MineralTileKind::Iron => {
                    iron_count += 1;
                    iron_tiles.insert(tile);
                }
                MineralTileKind::Gold => {
                    gold_count += 1;
                    gold_tiles.insert(tile);
                }
            }
        }

        assigned.stone_tiles = stone_tiles;
        assigned.iron_tiles = iron_tiles;
        assigned.gold_tiles = gold_tiles;

        let stone_fp = QUARRY_STONE_IRON_PER_TILE * I32F32::from_num(stone_count);
        let iron_fp = QUARRY_STONE_IRON_PER_TILE * I32F32::from_num(iron_count);
        output.pstone = stone_fp.ceil().to_num::<i32>();
        output.piron = iron_fp.ceil().to_num::<i32>();
        output.pgold = gold_count * QUARRY_GOLD_PER_TILE;
    }
}

fn quarry_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    quarries: Query<
        (
            Entity,
            &BuildingAnchor,
            &BuildingHealth,
            &QuarryBuildState,
            &QuarryEconomy,
            &QuarryOutput,
            &QuarryFootprint,
            &QuarryFieldAssignment,
        ),
        With<Quarry>,
    >,
    claims: Res<QuarryPlacementClaims>,
    occupancy: Res<TileOccupancy>,
    minerals: Res<MineralTiles>,
) {
    for (entity, anchor, hp, build, eco, output, footprint, assigned) in &quarries {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);

        checksum.accumulate(hp.hp.to_bits() as u64);
        checksum.accumulate(hp.defenses_life.to_bits() as u64);

        checksum.accumulate(build.build_ticks_remaining as u64);
        checksum.accumulate(u64::from(build.completed));

        checksum.accumulate(eco.energy_cost as u64);
        checksum.accumulate(eco.wood_cost as u64);
        checksum.accumulate(eco.stone_cost as u64);
        checksum.accumulate(eco.iron_cost as u64);
        checksum.accumulate(eco.oil_cost as u64);
        checksum.accumulate(eco.gold_cost as u64);
        checksum.accumulate(eco.workers as u64);

        checksum.accumulate(output.pstone as u64);
        checksum.accumulate(output.piron as u64);
        checksum.accumulate(output.pgold as u64);

        checksum.accumulate(footprint.size_tiles as u64);
        checksum.accumulate(footprint.watch_range.to_bits() as u64);
        checksum.accumulate(footprint.field_side_tiles as u64);

        checksum.accumulate(assigned.stone_tiles.len() as u64);
        for (x, y) in &assigned.stone_tiles {
            checksum.accumulate(*x as u64);
            checksum.accumulate(*y as u64);
        }

        checksum.accumulate(assigned.iron_tiles.len() as u64);
        for (x, y) in &assigned.iron_tiles {
            checksum.accumulate(*x as u64);
            checksum.accumulate(*y as u64);
        }

        checksum.accumulate(assigned.gold_tiles.len() as u64);
        for (x, y) in &assigned.gold_tiles {
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

    checksum.accumulate(occupancy.blocked_tiles.len() as u64);
    for (x, y) in &occupancy.blocked_tiles {
        checksum.accumulate(*x as u64);
        checksum.accumulate(*y as u64);
    }

    checksum.accumulate(minerals.by_tile.len() as u64);
    for ((x, y), kind) in &minerals.by_tile {
        checksum.accumulate(*x as u64);
        checksum.accumulate(*y as u64);
        let value = match kind {
            MineralTileKind::Stone => 0_u64,
            MineralTileKind::Iron => 1_u64,
            MineralTileKind::Gold => 2_u64,
        };
        checksum.accumulate(value);
    }
}

pub struct QuarryPlugin;

impl Plugin for QuarryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MineralTiles>()
            .init_resource::<TileOccupancy>()
            .init_resource::<QuarryPlacementClaims>()
            .add_event::<SetTileBlockedEvent>()
            .add_event::<SetMineralTileEvent>()
            .add_event::<PlaceQuarryEvent>()
            .add_event::<RemoveQuarryEvent>()
            .add_event::<QuarryPlacementRejectedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    apply_tile_block_events_system,
                    apply_mineral_tile_events_system,
                    place_quarry_system,
                    remove_quarry_system,
                    quarry_build_tick_system,
                    quarry_field_and_output_system,
                    quarry_checksum_system,
                )
                    .chain(),
            );
    }
}
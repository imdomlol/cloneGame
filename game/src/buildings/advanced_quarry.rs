// Sources: vault/buildings/advanced_quarry.md, vault/buildings/quarry.md

use std::collections::BTreeSet;

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::buildings::quarry::{
    BuildingAnchor, MineralTileKind, MineralTiles, Quarry, QuarryPlacementClaims, SetMineralTileEvent,
    SetTileBlockedEvent, TileOccupancy,
};
use crate::sim::{Health, SimChecksumState};

const ADVANCED_QUARRY_HP: I32F32 = I32F32::lit("600");
const ADVANCED_QUARRY_DEFENSES_LIFE: I32F32 = I32F32::lit("150");
const ADVANCED_QUARRY_WATCH_RANGE: I32F32 = I32F32::lit("7");
const ADVANCED_QUARRY_ENERGY_COST: i32 = 15;
const ADVANCED_QUARRY_WOOD_COST: i32 = 30;
const ADVANCED_QUARRY_STONE_COST: i32 = 0;
const ADVANCED_QUARRY_IRON_COST: i32 = 20;
const ADVANCED_QUARRY_OIL_COST: i32 = 20;
const ADVANCED_QUARRY_GOLD_COST: i32 = 1200;
const ADVANCED_QUARRY_BUILD_TIME_SECONDS: i32 = 45;
const ADVANCED_QUARRY_UPGRADE_WOOD_COST: i32 = 0;
const ADVANCED_QUARRY_UPGRADE_STONE_COST: i32 = 0;
const ADVANCED_QUARRY_UPGRADE_IRON_COST: i32 = 0;
const ADVANCED_QUARRY_UPGRADE_OIL_COST: i32 = 0;
const ADVANCED_QUARRY_UPGRADE_GOLD_COST: i32 = 900;
const ADVANCED_QUARRY_UPGRADE_TIME_SECONDS: i32 = 21;
const ADVANCED_QUARRY_WORKERS: i32 = 6;
const ADVANCED_QUARRY_GOLD_MAINTENANCE: i32 = 30;
const ADVANCED_QUARRY_PGOLD: i32 = 30;
const ADVANCED_QUARRY_SIZE_TILES: i32 = 2;
const ADVANCED_QUARRY_FIELD_SIDE_TILES: i32 = 6;
const ADVANCED_QUARRY_STONE_IRON_PER_TILE: i32 = 1;
const ADVANCED_QUARRY_GOLD_PER_TILE: i32 = 20;
const SIM_HZ: i32 = 25;

#[derive(Component, Default)]
pub struct AdvancedQuarry;

#[derive(Component, Clone, Copy)]
pub struct AdvancedQuarryCoreStats {
    pub defenses_life: I32F32,
    pub watch_range: I32F32,
    pub health: Health,
}

impl Default for AdvancedQuarryCoreStats {
    fn default() -> Self {
        Self {
            defenses_life: ADVANCED_QUARRY_DEFENSES_LIFE,
            watch_range: ADVANCED_QUARRY_WATCH_RANGE,
            health: Health::full(ADVANCED_QUARRY_HP),
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct AdvancedQuarryBuildState {
    pub build_ticks_remaining: i32,
    pub upgrading_from_quarry: bool,
    pub completed: bool,
}

#[derive(Component, Clone, Copy)]
pub struct AdvancedQuarryEconomy {
    pub energy_cost: i32,
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
    pub workers: i32,
    pub gold_maintenance: i32,
}

impl Default for AdvancedQuarryEconomy {
    fn default() -> Self {
        Self {
            energy_cost: ADVANCED_QUARRY_ENERGY_COST,
            wood_cost: ADVANCED_QUARRY_WOOD_COST,
            stone_cost: ADVANCED_QUARRY_STONE_COST,
            iron_cost: ADVANCED_QUARRY_IRON_COST,
            oil_cost: ADVANCED_QUARRY_OIL_COST,
            gold_cost: ADVANCED_QUARRY_GOLD_COST,
            upgrade_wood_cost: ADVANCED_QUARRY_UPGRADE_WOOD_COST,
            upgrade_stone_cost: ADVANCED_QUARRY_UPGRADE_STONE_COST,
            upgrade_iron_cost: ADVANCED_QUARRY_UPGRADE_IRON_COST,
            upgrade_oil_cost: ADVANCED_QUARRY_UPGRADE_OIL_COST,
            upgrade_gold_cost: ADVANCED_QUARRY_UPGRADE_GOLD_COST,
            workers: ADVANCED_QUARRY_WORKERS,
            gold_maintenance: ADVANCED_QUARRY_GOLD_MAINTENANCE,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct AdvancedQuarryOutput {
    pub pstone: i32,
    pub piron: i32,
    pub pgold: i32,
}

#[derive(Component, Clone, Copy)]
pub struct AdvancedQuarryFootprint {
    pub size_tiles: i32,
    pub field_side_tiles: i32,
}

impl Default for AdvancedQuarryFootprint {
    fn default() -> Self {
        Self {
            size_tiles: ADVANCED_QUARRY_SIZE_TILES,
            field_side_tiles: ADVANCED_QUARRY_FIELD_SIDE_TILES,
        }
    }
}

#[derive(Component, Clone, Default)]
pub struct AdvancedQuarryFieldAssignment {
    pub stone_tiles: BTreeSet<(i32, i32)>,
    pub iron_tiles: BTreeSet<(i32, i32)>,
    pub gold_tiles: BTreeSet<(i32, i32)>,
}

#[derive(Event, Clone, Copy)]
pub struct PlaceAdvancedQuarryEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct UpgradeQuarryToAdvancedQuarryEvent {
    pub quarry_entity: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct AdvancedQuarryPlacementRejectedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

fn seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn footprint_tiles(anchor: BuildingAnchor) -> impl Iterator<Item = (i32, i32)> {
    let min_x = anchor.x;
    let min_y = anchor.y;
    let max_x = anchor.x + ADVANCED_QUARRY_SIZE_TILES - 1;
    let max_y = anchor.y + ADVANCED_QUARRY_SIZE_TILES - 1;
    (min_x..=max_x).flat_map(move |x| (min_y..=max_y).map(move |y| (x, y)))
}

fn field_tiles(anchor: BuildingAnchor) -> impl Iterator<Item = (i32, i32)> {
    let half = ADVANCED_QUARRY_FIELD_SIDE_TILES / 2;
    let center_x = anchor.x + (ADVANCED_QUARRY_SIZE_TILES / 2);
    let center_y = anchor.y + (ADVANCED_QUARRY_SIZE_TILES / 2);
    let min_x = center_x - half;
    let min_y = center_y - half;
    let max_x = min_x + ADVANCED_QUARRY_FIELD_SIDE_TILES - 1;
    let max_y = min_y + ADVANCED_QUARRY_FIELD_SIDE_TILES - 1;
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

fn field_has_no_overlap(
    anchor: BuildingAnchor,
    ignore_entity: Option<Entity>,
    claims: &QuarryPlacementClaims,
) -> bool {
    let mut existing = BTreeSet::new();

    for (entity, existing_anchor) in &claims.claims {
        if ignore_entity.is_some_and(|ignored| *entity == ignored) {
            continue;
        }

        for tile in field_tiles(*existing_anchor) {
            existing.insert(tile);
        }
    }

    for tile in field_tiles(anchor) {
        if existing.contains(&tile) {
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

fn place_advanced_quarry_system(
    mut commands: Commands,
    mut events: EventReader<PlaceAdvancedQuarryEvent>,
    mut rejected: EventWriter<AdvancedQuarryPlacementRejectedEvent>,
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
            && field_has_no_overlap(anchor, None, &claims)
            && field_has_any_resource(anchor, &minerals);

        if !valid {
            rejected.send(AdvancedQuarryPlacementRejectedEvent {
                tile_x: ev.tile_x,
                tile_y: ev.tile_y,
            });
            continue;
        }

        let entity = commands
            .spawn((
                AdvancedQuarry,
                anchor,
                AdvancedQuarryCoreStats::default(),
                AdvancedQuarryBuildState {
                    build_ticks_remaining: seconds_to_ticks(ADVANCED_QUARRY_BUILD_TIME_SECONDS),
                    upgrading_from_quarry: false,
                    completed: false,
                },
                AdvancedQuarryEconomy::default(),
                AdvancedQuarryOutput::default(),
                AdvancedQuarryFootprint::default(),
                AdvancedQuarryFieldAssignment::default(),
            ))
            .id();

        claims.claims.insert(entity, anchor);
    }
}

fn upgrade_quarry_to_advanced_quarry_system(
    mut commands: Commands,
    mut events: EventReader<UpgradeQuarryToAdvancedQuarryEvent>,
    quarries: Query<(Entity, &BuildingAnchor), With<Quarry>>,
    mut claims: ResMut<QuarryPlacementClaims>,
    occupancy: Res<TileOccupancy>,
    minerals: Res<MineralTiles>,
) {
    for ev in events.read() {
        let Ok((quarry_entity, anchor)) = quarries.get(ev.quarry_entity) else {
            continue;
        };

        let valid = footprint_is_unblocked(*anchor, &occupancy)
            && field_has_no_overlap(*anchor, Some(quarry_entity), &claims)
            && field_has_any_resource(*anchor, &minerals);

        if !valid {
            continue;
        }

        commands.entity(quarry_entity).remove::<Quarry>();
        commands.entity(quarry_entity).insert((
            AdvancedQuarry,
            AdvancedQuarryCoreStats::default(),
            AdvancedQuarryBuildState {
                build_ticks_remaining: seconds_to_ticks(ADVANCED_QUARRY_UPGRADE_TIME_SECONDS),
                upgrading_from_quarry: true,
                completed: false,
            },
            AdvancedQuarryEconomy {
                energy_cost: ADVANCED_QUARRY_ENERGY_COST,
                wood_cost: ADVANCED_QUARRY_UPGRADE_WOOD_COST,
                stone_cost: ADVANCED_QUARRY_STONE_COST,
                iron_cost: ADVANCED_QUARRY_UPGRADE_IRON_COST,
                oil_cost: ADVANCED_QUARRY_UPGRADE_OIL_COST,
                gold_cost: ADVANCED_QUARRY_UPGRADE_GOLD_COST,
                upgrade_wood_cost: ADVANCED_QUARRY_UPGRADE_WOOD_COST,
                upgrade_stone_cost: ADVANCED_QUARRY_UPGRADE_STONE_COST,
                upgrade_iron_cost: ADVANCED_QUARRY_UPGRADE_IRON_COST,
                upgrade_oil_cost: ADVANCED_QUARRY_UPGRADE_OIL_COST,
                upgrade_gold_cost: ADVANCED_QUARRY_UPGRADE_GOLD_COST,
                workers: ADVANCED_QUARRY_WORKERS,
                gold_maintenance: ADVANCED_QUARRY_GOLD_MAINTENANCE,
            },
            AdvancedQuarryOutput::default(),
            AdvancedQuarryFootprint::default(),
            AdvancedQuarryFieldAssignment::default(),
        ));

        claims.claims.insert(quarry_entity, *anchor);
    }
}

fn advanced_quarry_build_tick_system(
    mut quarries: Query<&mut AdvancedQuarryBuildState, With<AdvancedQuarry>>,
) {
    for mut state in &mut quarries {
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

fn advanced_quarry_field_and_output_system(
    mut quarries: Query<
        (
            &BuildingAnchor,
            &AdvancedQuarryBuildState,
            &mut AdvancedQuarryFieldAssignment,
            &mut AdvancedQuarryOutput,
        ),
        With<AdvancedQuarry>,
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

        output.pstone = stone_count * ADVANCED_QUARRY_STONE_IRON_PER_TILE;
        output.piron = iron_count * ADVANCED_QUARRY_STONE_IRON_PER_TILE;
        output.pgold = (gold_count * ADVANCED_QUARRY_GOLD_PER_TILE) + ADVANCED_QUARRY_PGOLD;
    }
}

fn advanced_quarry_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    quarries: Query<
        (
            Entity,
            &BuildingAnchor,
            &AdvancedQuarryCoreStats,
            &AdvancedQuarryBuildState,
            &AdvancedQuarryEconomy,
            &AdvancedQuarryOutput,
            &AdvancedQuarryFootprint,
            &AdvancedQuarryFieldAssignment,
        ),
        With<AdvancedQuarry>,
    >,
    claims: Res<QuarryPlacementClaims>,
    occupancy: Res<TileOccupancy>,
    minerals: Res<MineralTiles>,
) {
    for (entity, anchor, core, build, eco, output, footprint, assigned) in &quarries {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);

        checksum.accumulate(core.defenses_life.to_bits() as u64);
        checksum.accumulate(core.watch_range.to_bits() as u64);
        checksum.accumulate(core.health.current.to_bits() as u64);
        checksum.accumulate(core.health.max.to_bits() as u64);

        checksum.accumulate(build.build_ticks_remaining as u64);
        checksum.accumulate(u64::from(build.upgrading_from_quarry));
        checksum.accumulate(u64::from(build.completed));

        checksum.accumulate(eco.energy_cost as u64);
        checksum.accumulate(eco.wood_cost as u64);
        checksum.accumulate(eco.stone_cost as u64);
        checksum.accumulate(eco.iron_cost as u64);
        checksum.accumulate(eco.oil_cost as u64);
        checksum.accumulate(eco.gold_cost as u64);
        checksum.accumulate(eco.upgrade_wood_cost as u64);
        checksum.accumulate(eco.upgrade_stone_cost as u64);
        checksum.accumulate(eco.upgrade_iron_cost as u64);
        checksum.accumulate(eco.upgrade_oil_cost as u64);
        checksum.accumulate(eco.upgrade_gold_cost as u64);
        checksum.accumulate(eco.workers as u64);
        checksum.accumulate(eco.gold_maintenance as u64);

        checksum.accumulate(output.pstone as u64);
        checksum.accumulate(output.piron as u64);
        checksum.accumulate(output.pgold as u64);

        checksum.accumulate(footprint.size_tiles as u64);
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

pub struct AdvancedQuarryPlugin;

impl Plugin for AdvancedQuarryPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SetTileBlockedEvent>()
            .add_event::<SetMineralTileEvent>()
            .add_event::<PlaceAdvancedQuarryEvent>()
            .add_event::<UpgradeQuarryToAdvancedQuarryEvent>()
            .add_event::<AdvancedQuarryPlacementRejectedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    place_advanced_quarry_system,
                    upgrade_quarry_to_advanced_quarry_system,
                    advanced_quarry_build_tick_system,
                    advanced_quarry_field_and_output_system,
                    advanced_quarry_checksum_system,
                )
                    .chain(),
            );
    }
}
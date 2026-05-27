// Sources: vault/buildings/advanced_mill.md, vault/buildings/mill.md

use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, SimChecksumState};

const ADVANCED_MILL_HP: I32F32 = I32F32::lit("600");
const ADVANCED_MILL_DEFENSES_LIFE: I32F32 = I32F32::lit("150");
const ADVANCED_MILL_WATCH_RANGE: I32F32 = I32F32::lit("7");
const ADVANCED_MILL_ENERGY_COST: i32 = 0;
const ADVANCED_MILL_WOOD_COST: i32 = 20;
const ADVANCED_MILL_STONE_COST: i32 = 0;
const ADVANCED_MILL_IRON_COST: i32 = 10;
const ADVANCED_MILL_OIL_COST: i32 = 0;
const ADVANCED_MILL_GOLD_COST: i32 = 800;
const ADVANCED_MILL_BUILD_TIME_SECONDS: i32 = 46;
const ADVANCED_MILL_UPGRADE_WOOD_COST: i32 = 0;
const ADVANCED_MILL_UPGRADE_STONE_COST: i32 = 0;
const ADVANCED_MILL_UPGRADE_IRON_COST: i32 = 10;
const ADVANCED_MILL_UPGRADE_OIL_COST: i32 = 0;
const ADVANCED_MILL_UPGRADE_GOLD_COST: i32 = 500;
const ADVANCED_MILL_UPGRADE_TIME_SECONDS: i32 = 21;
const ADVANCED_MILL_WORKERS: i32 = 4;
const ADVANCED_MILL_GOLD_MAINTENANCE: i32 = 40;
const ADVANCED_MILL_PENERGY: i32 = 60;
const ADVANCED_MILL_SIZE_TILES: i32 = 2;
const ADVANCED_MILL_MIN_SPACING_TILES: i32 = 4;
const SIM_HZ: i32 = 25;

#[derive(Component, Default)]
pub struct Mill;

#[derive(Component, Default)]
pub struct AdvancedMill;

#[derive(Component, Clone, Copy)]
pub struct BuildingAnchor {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Copy)]
pub struct AdvancedMillCoreStats {
    pub defenses_life: I32F32,
    pub watch_range: I32F32,
    pub health: Health,
}

impl Default for AdvancedMillCoreStats {
    fn default() -> Self {
        Self {
            defenses_life: ADVANCED_MILL_DEFENSES_LIFE,
            watch_range: ADVANCED_MILL_WATCH_RANGE,
            health: Health::full(ADVANCED_MILL_HP),
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct AdvancedMillBuildState {
    pub build_ticks_remaining: i32,
    pub upgrading_from_mill: bool,
    pub completed: bool,
}

#[derive(Component, Clone, Copy)]
pub struct AdvancedMillEconomy {
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

impl Default for AdvancedMillEconomy {
    fn default() -> Self {
        Self {
            energy_cost: ADVANCED_MILL_ENERGY_COST,
            wood_cost: ADVANCED_MILL_WOOD_COST,
            stone_cost: ADVANCED_MILL_STONE_COST,
            iron_cost: ADVANCED_MILL_IRON_COST,
            oil_cost: ADVANCED_MILL_OIL_COST,
            gold_cost: ADVANCED_MILL_GOLD_COST,
            upgrade_wood_cost: ADVANCED_MILL_UPGRADE_WOOD_COST,
            upgrade_stone_cost: ADVANCED_MILL_UPGRADE_STONE_COST,
            upgrade_iron_cost: ADVANCED_MILL_UPGRADE_IRON_COST,
            upgrade_oil_cost: ADVANCED_MILL_UPGRADE_OIL_COST,
            upgrade_gold_cost: ADVANCED_MILL_UPGRADE_GOLD_COST,
            workers: ADVANCED_MILL_WORKERS,
            gold_maintenance: ADVANCED_MILL_GOLD_MAINTENANCE,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct AdvancedMillOutput {
    pub penergy: i32,
}

impl Default for AdvancedMillOutput {
    fn default() -> Self {
        Self { penergy: 0 }
    }
}

#[derive(Component, Clone, Copy)]
pub struct AdvancedMillFootprint {
    pub size_tiles: i32,
    pub watch_range: I32F32,
}

impl Default for AdvancedMillFootprint {
    fn default() -> Self {
        Self {
            size_tiles: ADVANCED_MILL_SIZE_TILES,
            watch_range: ADVANCED_MILL_WATCH_RANGE,
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
pub struct PlaceAdvancedMillEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct UpgradeMillToAdvancedMillEvent {
    pub mill_entity: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct RegisterMillStructureEvent {
    pub entity: Entity,
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct UnregisterMillStructureEvent {
    pub entity: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct AdvancedMillPlacementRejectedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

fn seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn footprint_tiles(anchor: BuildingAnchor) -> impl Iterator<Item = (i32, i32)> {
    let min_x = anchor.x;
    let min_y = anchor.y;
    let max_x = anchor.x + ADVANCED_MILL_SIZE_TILES - 1;
    let max_y = anchor.y + ADVANCED_MILL_SIZE_TILES - 1;
    (min_x..=max_x).flat_map(move |x| (min_y..=max_y).map(move |y| (x, y)))
}

fn max_axis_delta(a: BuildingAnchor, b: BuildingAnchor) -> i32 {
    let dx = (a.x - b.x).abs();
    let dy = (a.y - b.y).abs();
    dx.max(dy)
}

fn has_valid_spacing(
    anchor: BuildingAnchor,
    ignore_entity: Option<Entity>,
    claims: &MillPlacementClaims,
) -> bool {
    for (entity, existing) in &claims.claims {
        if ignore_entity.is_some_and(|ignored| *entity == ignored) {
            continue;
        }

        if max_axis_delta(anchor, *existing) <= ADVANCED_MILL_MIN_SPACING_TILES {
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

fn register_mill_structure_claims_system(
    mut register_events: EventReader<RegisterMillStructureEvent>,
    mut unregister_events: EventReader<UnregisterMillStructureEvent>,
    mut claims: ResMut<MillPlacementClaims>,
) {
    for ev in register_events.read() {
        claims.claims.insert(
            ev.entity,
            BuildingAnchor {
                x: ev.tile_x,
                y: ev.tile_y,
            },
        );
    }

    for ev in unregister_events.read() {
        claims.claims.remove(&ev.entity);
    }
}

fn place_advanced_mill_system(
    mut commands: Commands,
    mut events: EventReader<PlaceAdvancedMillEvent>,
    mut rejected: EventWriter<AdvancedMillPlacementRejectedEvent>,
    mut claims: ResMut<MillPlacementClaims>,
    occupancy: Res<TileOccupancy>,
) {
    for ev in events.read() {
        let anchor = BuildingAnchor {
            x: ev.tile_x,
            y: ev.tile_y,
        };

        if !has_valid_spacing(anchor, None, &claims) || !footprint_is_unblocked(anchor, &occupancy) {
            rejected.send(AdvancedMillPlacementRejectedEvent {
                tile_x: ev.tile_x,
                tile_y: ev.tile_y,
            });
            continue;
        }

        let entity = commands
            .spawn((
                AdvancedMill,
                anchor,
                AdvancedMillCoreStats::default(),
                AdvancedMillBuildState {
                    build_ticks_remaining: seconds_to_ticks(ADVANCED_MILL_BUILD_TIME_SECONDS),
                    upgrading_from_mill: false,
                    completed: false,
                },
                AdvancedMillEconomy::default(),
                AdvancedMillOutput::default(),
                AdvancedMillFootprint::default(),
            ))
            .id();

        claims.claims.insert(entity, anchor);
    }
}

fn upgrade_mill_to_advanced_mill_system(
    mut commands: Commands,
    mut events: EventReader<UpgradeMillToAdvancedMillEvent>,
    mills: Query<(Entity, &BuildingAnchor), With<Mill>>,
    mut claims: ResMut<MillPlacementClaims>,
) {
    for ev in events.read() {
        let Ok((mill_entity, mill_anchor)) = mills.get(ev.mill_entity) else {
            continue;
        };

        if !has_valid_spacing(*mill_anchor, Some(mill_entity), &claims) {
            continue;
        }

        commands.entity(mill_entity).remove::<Mill>();
        commands.entity(mill_entity).insert((
            AdvancedMill,
            AdvancedMillCoreStats::default(),
            AdvancedMillBuildState {
                build_ticks_remaining: seconds_to_ticks(ADVANCED_MILL_UPGRADE_TIME_SECONDS),
                upgrading_from_mill: true,
                completed: false,
            },
            AdvancedMillEconomy::default(),
            AdvancedMillOutput::default(),
            AdvancedMillFootprint::default(),
        ));

        claims.claims.insert(mill_entity, *mill_anchor);
    }
}

fn advanced_mill_build_tick_system(mut mills: Query<&mut AdvancedMillBuildState, With<AdvancedMill>>) {
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

fn advanced_mill_output_system(
    mut mills: Query<(&AdvancedMillBuildState, &mut AdvancedMillOutput), With<AdvancedMill>>,
) {
    for (state, mut output) in &mut mills {
        output.penergy = if state.completed {
            ADVANCED_MILL_PENERGY
        } else {
            0
        };
    }
}

fn advanced_mill_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    mills: Query<
        (
            Entity,
            &BuildingAnchor,
            &AdvancedMillCoreStats,
            &AdvancedMillBuildState,
            &AdvancedMillEconomy,
            &AdvancedMillOutput,
            &AdvancedMillFootprint,
        ),
        With<AdvancedMill>,
    >,
    claims: Res<MillPlacementClaims>,
    occupancy: Res<TileOccupancy>,
) {
    for (entity, anchor, core, build, eco, output, footprint) in &mills {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);

        checksum.accumulate(core.defenses_life.to_bits() as u64);
        checksum.accumulate(core.watch_range.to_bits() as u64);
        checksum.accumulate(core.health.current.to_bits() as u64);
        checksum.accumulate(core.health.max.to_bits() as u64);

        checksum.accumulate(build.build_ticks_remaining as u64);
        checksum.accumulate(u64::from(build.upgrading_from_mill));
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

        checksum.accumulate(output.penergy as u64);

        checksum.accumulate(footprint.size_tiles as u64);
        checksum.accumulate(footprint.watch_range.to_bits() as u64);
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

pub struct AdvancedMillPlugin;

impl Plugin for AdvancedMillPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileOccupancy>()
            .init_resource::<MillPlacementClaims>()
            .add_event::<SetTileBlockedEvent>()
            .add_event::<PlaceAdvancedMillEvent>()
            .add_event::<UpgradeMillToAdvancedMillEvent>()
            .add_event::<RegisterMillStructureEvent>()
            .add_event::<UnregisterMillStructureEvent>()
            .add_event::<AdvancedMillPlacementRejectedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    apply_tile_block_events_system,
                    register_mill_structure_claims_system,
                    place_advanced_mill_system,
                    upgrade_mill_to_advanced_mill_system,
                    advanced_mill_build_tick_system,
                    advanced_mill_output_system,
                    advanced_mill_checksum_system,
                )
                    .chain(),
            );
    }
}
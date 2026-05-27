// Sources: vault/buildings/tesla_tower.md

use std::collections::BTreeMap;

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, SimChecksumState};

const TESLA_TOWER_HP: I32F32 = I32F32::lit("140");
const TESLA_TOWER_DEFENSES_LIFE: I32F32 = I32F32::lit("35");
const TESLA_TOWER_WATCH_RANGE: I32F32 = I32F32::lit("8");
const TESLA_TOWER_WOOD_COST: i32 = 10;
const TESLA_TOWER_GOLD_COST: i32 = 200;
const TESLA_TOWER_WORKERS: i32 = 1;
const TESLA_TOWER_GOLD_MAINTENANCE: i32 = 4;
const TESLA_TOWER_BUILD_TIME_SECONDS: i32 = 30;
const TESLA_TOWER_SIZE_TILES: i32 = 1;
const TESLA_TOWER_ENERGY_RANGE_CARDINAL: i32 = 8;
const TESLA_TOWER_ENERGY_RANGE_DIAGONAL: i32 = 6;
const TESLA_TOWER_EDGE_NEW_COVERAGE_TILES: i32 = 136;
const TESLA_TOWER_CORNER_NEW_COVERAGE_TILES: i32 = 152;
const SIM_HZ: i32 = 25;

#[derive(Component, Default)]
pub struct TeslaTower;

#[derive(Component, Clone, Copy, Default)]
pub struct BuildingAnchor {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Copy)]
pub struct TeslaTowerCore {
    pub defenses_life: I32F32,
    pub watch_range: I32F32,
    pub health: Health,
}

impl Default for TeslaTowerCore {
    fn default() -> Self {
        Self {
            defenses_life: TESLA_TOWER_DEFENSES_LIFE,
            watch_range: TESLA_TOWER_WATCH_RANGE,
            health: Health::full(TESLA_TOWER_HP),
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct TeslaTowerEconomy {
    pub wood_cost: i32,
    pub gold_cost: i32,
    pub workers: i32,
    pub gold_maintenance: i32,
}

impl Default for TeslaTowerEconomy {
    fn default() -> Self {
        Self {
            wood_cost: TESLA_TOWER_WOOD_COST,
            gold_cost: TESLA_TOWER_GOLD_COST,
            workers: TESLA_TOWER_WORKERS,
            gold_maintenance: TESLA_TOWER_GOLD_MAINTENANCE,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct TeslaTowerFootprint {
    pub size_tiles: i32,
}

impl Default for TeslaTowerFootprint {
    fn default() -> Self {
        Self {
            size_tiles: TESLA_TOWER_SIZE_TILES,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct TeslaTowerBuildState {
    pub build_ticks_remaining: i32,
    pub completed: bool,
}

#[derive(Component, Clone, Copy)]
pub struct TeslaTowerEnergyProfile {
    pub cardinal_range_tiles: i32,
    pub diagonal_range_tiles: i32,
    pub edge_new_coverage_tiles: i32,
    pub corner_new_coverage_tiles: i32,
}

impl Default for TeslaTowerEnergyProfile {
    fn default() -> Self {
        Self {
            cardinal_range_tiles: TESLA_TOWER_ENERGY_RANGE_CARDINAL,
            diagonal_range_tiles: TESLA_TOWER_ENERGY_RANGE_DIAGONAL,
            edge_new_coverage_tiles: TESLA_TOWER_EDGE_NEW_COVERAGE_TILES,
            corner_new_coverage_tiles: TESLA_TOWER_CORNER_NEW_COVERAGE_TILES,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct TeslaTowerPowerState {
    pub destroyed: bool,
    pub infected: bool,
    pub depowered: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TeslaDependentBuildingKind {
    GreatBallista,
    ShockingTower,
    Executor,
}

#[derive(Component, Clone, Copy)]
pub struct TeslaDependentBuilding {
    pub kind: TeslaDependentBuildingKind,
    pub connected_to_grid: bool,
    pub depowered: bool,
    pub can_repair: bool,
}

impl Default for TeslaDependentBuilding {
    fn default() -> Self {
        Self {
            kind: TeslaDependentBuildingKind::GreatBallista,
            connected_to_grid: true,
            depowered: false,
            can_repair: true,
        }
    }
}

#[derive(Resource, Default, Clone)]
pub struct TeslaTowerPlacementClaims {
    pub claims: BTreeMap<Entity, BuildingAnchor>,
}

#[derive(Resource, Clone, Copy)]
pub struct TeslaCoverageLedger {
    pub edge_new_tiles: i32,
    pub corner_new_tiles: i32,
}

impl Default for TeslaCoverageLedger {
    fn default() -> Self {
        Self {
            edge_new_tiles: TESLA_TOWER_EDGE_NEW_COVERAGE_TILES,
            corner_new_tiles: TESLA_TOWER_CORNER_NEW_COVERAGE_TILES,
        }
    }
}

#[derive(Event, Clone, Copy)]
pub struct PlaceTeslaTowerEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct TeslaTowerPlacementRejectedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct SetTeslaTowerStateEvent {
    pub entity: Entity,
    pub destroyed: bool,
    pub infected: bool,
}

#[derive(Event, Clone, Copy)]
pub struct RegisterTeslaDependentBuildingEvent {
    pub entity: Entity,
    pub kind: TeslaDependentBuildingKind,
    pub connected_to_grid: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetTeslaDependentBuildingConnectionEvent {
    pub entity: Entity,
    pub connected_to_grid: bool,
}

fn seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn place_tesla_tower_system(
    mut commands: Commands,
    mut events: EventReader<PlaceTeslaTowerEvent>,
    mut rejected: EventWriter<TeslaTowerPlacementRejectedEvent>,
    mut claims: ResMut<TeslaTowerPlacementClaims>,
) {
    for ev in events.read() {
        let anchor = BuildingAnchor {
            x: ev.tile_x,
            y: ev.tile_y,
        };

        let occupied = claims
            .claims
            .values()
            .any(|existing| existing.x == anchor.x && existing.y == anchor.y);
        if occupied {
            rejected.send(TeslaTowerPlacementRejectedEvent {
                tile_x: ev.tile_x,
                tile_y: ev.tile_y,
            });
            continue;
        }

        let entity = commands
            .spawn((
                TeslaTower,
                anchor,
                TeslaTowerCore::default(),
                TeslaTowerEconomy::default(),
                TeslaTowerFootprint::default(),
                TeslaTowerBuildState {
                    build_ticks_remaining: seconds_to_ticks(TESLA_TOWER_BUILD_TIME_SECONDS),
                    completed: false,
                },
                TeslaTowerEnergyProfile::default(),
                TeslaTowerPowerState::default(),
            ))
            .id();

        claims.claims.insert(entity, anchor);
    }
}

fn set_tesla_tower_state_system(
    mut events: EventReader<SetTeslaTowerStateEvent>,
    mut towers: Query<&mut TeslaTowerPowerState, With<TeslaTower>>,
) {
    for ev in events.read() {
        let Ok(mut state) = towers.get_mut(ev.entity) else {
            continue;
        };

        state.destroyed = ev.destroyed;
        state.infected = ev.infected;
    }
}

fn register_dependent_building_system(
    mut commands: Commands,
    mut events: EventReader<RegisterTeslaDependentBuildingEvent>,
) {
    for ev in events.read() {
        commands.entity(ev.entity).insert(TeslaDependentBuilding {
            kind: ev.kind,
            connected_to_grid: ev.connected_to_grid,
            depowered: false,
            can_repair: true,
        });
    }
}

fn set_dependent_building_connection_system(
    mut events: EventReader<SetTeslaDependentBuildingConnectionEvent>,
    mut buildings: Query<&mut TeslaDependentBuilding>,
) {
    for ev in events.read() {
        let Ok(mut building) = buildings.get_mut(ev.entity) else {
            continue;
        };

        building.connected_to_grid = ev.connected_to_grid;
    }
}

fn tesla_tower_build_tick_system(
    mut towers: Query<&mut TeslaTowerBuildState, With<TeslaTower>>,
) {
    for mut build in &mut towers {
        if build.completed {
            continue;
        }

        if build.build_ticks_remaining > 0 {
            build.build_ticks_remaining -= 1;
        }

        if build.build_ticks_remaining <= 0 {
            build.build_ticks_remaining = 0;
            build.completed = true;
        }
    }
}

fn tesla_power_distribution_system(
    mut towers: Query<(&TeslaTowerBuildState, &mut TeslaTowerPowerState), With<TeslaTower>>,
    mut dependents: Query<&mut TeslaDependentBuilding>,
) {
    let mut active_tower_count = 0_i32;
    for (build, state) in &towers {
        if build.completed && !state.destroyed && !state.infected {
            active_tower_count += 1;
        }
    }

    let has_active_tower = active_tower_count > 0;

    for (build, mut state) in &mut towers {
        let functioning = build.completed && !state.destroyed && !state.infected;
        state.depowered = !functioning || !has_active_tower;
    }

    for mut building in &mut dependents {
        let should_be_depowered = !building.connected_to_grid || !has_active_tower;
        building.depowered = should_be_depowered;
        building.can_repair = !should_be_depowered;
    }
}

fn tesla_tower_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    towers: Query<
        (
            Entity,
            &BuildingAnchor,
            &TeslaTowerCore,
            &TeslaTowerEconomy,
            &TeslaTowerFootprint,
            &TeslaTowerBuildState,
            &TeslaTowerEnergyProfile,
            &TeslaTowerPowerState,
        ),
        With<TeslaTower>,
    >,
    dependents: Query<(Entity, &TeslaDependentBuilding), Without<TeslaTower>>,
    claims: Res<TeslaTowerPlacementClaims>,
    coverage: Res<TeslaCoverageLedger>,
) {
    for (entity, anchor, core, economy, footprint, build, profile, power) in &towers {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);

        checksum.accumulate(core.defenses_life.to_bits() as u64);
        checksum.accumulate(core.watch_range.to_bits() as u64);
        checksum.accumulate(core.health.current.to_bits() as u64);
        checksum.accumulate(core.health.max.to_bits() as u64);

        checksum.accumulate(economy.wood_cost as u64);
        checksum.accumulate(economy.gold_cost as u64);
        checksum.accumulate(economy.workers as u64);
        checksum.accumulate(economy.gold_maintenance as u64);

        checksum.accumulate(footprint.size_tiles as u64);

        checksum.accumulate(build.build_ticks_remaining as u64);
        checksum.accumulate(u64::from(build.completed));

        checksum.accumulate(profile.cardinal_range_tiles as u64);
        checksum.accumulate(profile.diagonal_range_tiles as u64);
        checksum.accumulate(profile.edge_new_coverage_tiles as u64);
        checksum.accumulate(profile.corner_new_coverage_tiles as u64);

        checksum.accumulate(u64::from(power.destroyed));
        checksum.accumulate(u64::from(power.infected));
        checksum.accumulate(u64::from(power.depowered));
    }

    for (entity, building) in &dependents {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(match building.kind {
            TeslaDependentBuildingKind::GreatBallista => 1,
            TeslaDependentBuildingKind::ShockingTower => 2,
            TeslaDependentBuildingKind::Executor => 3,
        });
        checksum.accumulate(u64::from(building.connected_to_grid));
        checksum.accumulate(u64::from(building.depowered));
        checksum.accumulate(u64::from(building.can_repair));
    }

    checksum.accumulate(claims.claims.len() as u64);
    for (entity, anchor) in &claims.claims {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);
    }

    checksum.accumulate(coverage.edge_new_tiles as u64);
    checksum.accumulate(coverage.corner_new_tiles as u64);
}

pub struct TeslaTowerPlugin;

impl Plugin for TeslaTowerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TeslaTowerPlacementClaims>()
            .init_resource::<TeslaCoverageLedger>()
            .add_event::<PlaceTeslaTowerEvent>()
            .add_event::<TeslaTowerPlacementRejectedEvent>()
            .add_event::<SetTeslaTowerStateEvent>()
            .add_event::<RegisterTeslaDependentBuildingEvent>()
            .add_event::<SetTeslaDependentBuildingConnectionEvent>()
            .add_systems(
                FixedUpdate,
                (
                    place_tesla_tower_system,
                    set_tesla_tower_state_system,
                    register_dependent_building_system,
                    set_dependent_building_connection_system,
                    tesla_tower_build_tick_system,
                    tesla_power_distribution_system,
                    tesla_tower_checksum_system,
                )
                    .chain(),
            );
    }
}
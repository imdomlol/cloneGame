// Sources: vault/wonders/atlas_transmutator.md

use std::collections::BTreeMap;

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::game_mechanics::resources::ColonyResourcesState;
use crate::sim::{Health, SimChecksumState};

const ATLAS_HP: I32F32 = I32F32::lit("2000");
const ATLAS_DEFENSES_LIFE: I32F32 = I32F32::lit("500");
const ATLAS_WATCH_RANGE: I32F32 = I32F32::lit("16");
const ATLAS_BUILD_TIME_SECONDS: i32 = 0;
const ATLAS_SIZE_TILES: i32 = 5;
const ATLAS_VICTORY_POINTS_REWARD: i32 = 2000;

const ATLAS_ENERGY_COST: i32 = 100;
const ATLAS_WORKERS_COST: i32 = 50;
const ATLAS_GOLD_MAINTENANCE: i32 = 120;

const ATLAS_WOOD_COST: i32 = 0;
const ATLAS_STONE_COST: i32 = 0;
const ATLAS_IRON_COST: i32 = 0;
const ATLAS_OIL_COST: i32 = 0;
const ATLAS_GOLD_COST: i32 = 0;

const ATLAS_CYCLE_SECONDS: i32 = 28_800;
const ATLAS_CYCLE_OIL_PRODUCTION: I32F32 = I32F32::lit("40");
const ATLAS_CYCLE_WOOD_CONSUMPTION: I32F32 = I32F32::lit("20");
const ATLAS_CYCLE_STONE_CONSUMPTION: I32F32 = I32F32::lit("20");
const ATLAS_CYCLE_IRON_CONSUMPTION: I32F32 = I32F32::lit("10");

const SIM_HZ: i32 = 25;

#[derive(Component, Default)]
pub struct AtlasTransmutator;

#[derive(Component, Clone, Copy, Default)]
pub struct AtlasBuildingAnchor {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Copy)]
pub struct AtlasCoreStats {
    pub defenses_life: I32F32,
    pub watch_range: I32F32,
    pub health: Health,
}

impl Default for AtlasCoreStats {
    fn default() -> Self {
        Self {
            defenses_life: ATLAS_DEFENSES_LIFE,
            watch_range: ATLAS_WATCH_RANGE,
            health: Health::full(ATLAS_HP),
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct AtlasBuildState {
    pub build_ticks_remaining: i32,
    pub completed: bool,
}

#[derive(Component, Clone, Copy)]
pub struct AtlasEconomy {
    pub energy_cost: i32,
    pub workers_cost: i32,
    pub gold_maintenance: i32,
    pub wood_cost: i32,
    pub stone_cost: i32,
    pub iron_cost: i32,
    pub oil_cost: i32,
    pub gold_cost: i32,
}

impl Default for AtlasEconomy {
    fn default() -> Self {
        Self {
            energy_cost: ATLAS_ENERGY_COST,
            workers_cost: ATLAS_WORKERS_COST,
            gold_maintenance: ATLAS_GOLD_MAINTENANCE,
            wood_cost: ATLAS_WOOD_COST,
            stone_cost: ATLAS_STONE_COST,
            iron_cost: ATLAS_IRON_COST,
            oil_cost: ATLAS_OIL_COST,
            gold_cost: ATLAS_GOLD_COST,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct AtlasFootprint {
    pub size_tiles: i32,
    pub watch_range: I32F32,
}

impl Default for AtlasFootprint {
    fn default() -> Self {
        Self {
            size_tiles: ATLAS_SIZE_TILES,
            watch_range: ATLAS_WATCH_RANGE,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct AtlasState {
    pub connected_to_grid: bool,
    pub supplied_energy: bool,
    pub supplied_workers: bool,
    pub supplied_gold_maintenance: bool,
    pub destroyed: bool,
    pub demolished: bool,
    pub research_tier_foundry_met: bool,
}

impl Default for AtlasState {
    fn default() -> Self {
        Self {
            connected_to_grid: true,
            supplied_energy: true,
            supplied_workers: true,
            supplied_gold_maintenance: true,
            destroyed: false,
            demolished: false,
            research_tier_foundry_met: true,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct AtlasEffects {
    pub cycle_oil_production: I32F32,
    pub cycle_wood_consumption: I32F32,
    pub cycle_stone_consumption: I32F32,
    pub cycle_iron_consumption: I32F32,
    pub victory_points_reward: i32,
}

impl Default for AtlasEffects {
    fn default() -> Self {
        Self {
            cycle_oil_production: ATLAS_CYCLE_OIL_PRODUCTION,
            cycle_wood_consumption: ATLAS_CYCLE_WOOD_CONSUMPTION,
            cycle_stone_consumption: ATLAS_CYCLE_STONE_CONSUMPTION,
            cycle_iron_consumption: ATLAS_CYCLE_IRON_CONSUMPTION,
            victory_points_reward: ATLAS_VICTORY_POINTS_REWARD,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct AtlasCycleState {
    pub ticks_until_cycle: i32,
    pub cycles_completed: u32,
}

impl Default for AtlasCycleState {
    fn default() -> Self {
        Self {
            ticks_until_cycle: seconds_to_ticks(ATLAS_CYCLE_SECONDS),
            cycles_completed: 0,
        }
    }
}

#[derive(Resource, Default, Clone)]
pub struct AtlasPlacementClaims {
    pub claims: BTreeMap<Entity, AtlasBuildingAnchor>,
}

#[derive(Event, Clone, Copy)]
pub struct PlaceAtlasTransmutatorEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct SetAtlasGridConnectionEvent {
    pub entity: Entity,
    pub connected: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetAtlasSupplyStateEvent {
    pub entity: Entity,
    pub supplied_energy: bool,
    pub supplied_workers: bool,
    pub supplied_gold_maintenance: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetAtlasResearchTierEvent {
    pub entity: Entity,
    pub foundry_met: bool,
}

#[derive(Event, Clone, Copy)]
pub struct DemolishAtlasTransmutatorEvent {
    pub entity: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct AttackDestroyAtlasTransmutatorEvent {
    pub entity: Entity,
}

fn seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn atlas_is_active(build: &AtlasBuildState, state: &AtlasState) -> bool {
    build.completed
        && state.connected_to_grid
        && state.supplied_energy
        && state.supplied_workers
        && state.supplied_gold_maintenance
        && state.research_tier_foundry_met
        && !state.destroyed
        && !state.demolished
}

fn place_atlas_system(
    mut commands: Commands,
    mut events: EventReader<PlaceAtlasTransmutatorEvent>,
    mut claims: ResMut<AtlasPlacementClaims>,
) {
    for ev in events.read() {
        let anchor = AtlasBuildingAnchor {
            x: ev.tile_x,
            y: ev.tile_y,
        };

        let entity = commands
            .spawn((
                AtlasTransmutator,
                anchor,
                AtlasCoreStats::default(),
                AtlasBuildState {
                    build_ticks_remaining: seconds_to_ticks(ATLAS_BUILD_TIME_SECONDS),
                    completed: false,
                },
                AtlasEconomy::default(),
                AtlasFootprint::default(),
                AtlasState::default(),
                AtlasEffects::default(),
                AtlasCycleState::default(),
            ))
            .id();

        claims.claims.insert(entity, anchor);
    }
}

fn set_atlas_grid_connection_system(
    mut events: EventReader<SetAtlasGridConnectionEvent>,
    mut atlases: Query<&mut AtlasState, With<AtlasTransmutator>>,
) {
    for ev in events.read() {
        let Ok(mut state) = atlases.get_mut(ev.entity) else {
            continue;
        };
        state.connected_to_grid = ev.connected;
    }
}

fn set_atlas_supply_state_system(
    mut events: EventReader<SetAtlasSupplyStateEvent>,
    mut atlases: Query<&mut AtlasState, With<AtlasTransmutator>>,
) {
    for ev in events.read() {
        let Ok(mut state) = atlases.get_mut(ev.entity) else {
            continue;
        };
        state.supplied_energy = ev.supplied_energy;
        state.supplied_workers = ev.supplied_workers;
        state.supplied_gold_maintenance = ev.supplied_gold_maintenance;
    }
}

fn set_atlas_research_tier_system(
    mut events: EventReader<SetAtlasResearchTierEvent>,
    mut atlases: Query<&mut AtlasState, With<AtlasTransmutator>>,
) {
    for ev in events.read() {
        let Ok(mut state) = atlases.get_mut(ev.entity) else {
            continue;
        };
        state.research_tier_foundry_met = ev.foundry_met;
    }
}

fn atlas_build_tick_system(mut atlases: Query<&mut AtlasBuildState, With<AtlasTransmutator>>) {
    for mut build in &mut atlases {
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

fn attack_destroy_atlas_system(
    mut events: EventReader<AttackDestroyAtlasTransmutatorEvent>,
    mut atlases: Query<&mut AtlasState, With<AtlasTransmutator>>,
) {
    for ev in events.read() {
        let Ok(mut state) = atlases.get_mut(ev.entity) else {
            continue;
        };
        state.destroyed = true;
    }
}

fn destroy_atlas_at_zero_health_system(
    mut atlases: Query<(&AtlasCoreStats, &mut AtlasState), With<AtlasTransmutator>>,
) {
    for (core, mut state) in &mut atlases {
        if core.health.current <= I32F32::ZERO {
            state.destroyed = true;
        }
    }
}

fn demolish_atlas_system(
    mut events: EventReader<DemolishAtlasTransmutatorEvent>,
    mut atlases: Query<&mut AtlasState, With<AtlasTransmutator>>,
) {
    for ev in events.read() {
        let Ok(mut state) = atlases.get_mut(ev.entity) else {
            continue;
        };
        state.demolished = true;
    }
}

fn atlas_transmutation_tick_system(
    mut colony: ResMut<ColonyResourcesState>,
    mut atlases: Query<
        (
            &AtlasBuildState,
            &AtlasState,
            &AtlasEffects,
            &mut AtlasCycleState,
        ),
        With<AtlasTransmutator>,
    >,
) {
    for (build, state, effects, mut cycle) in &mut atlases {
        if !atlas_is_active(build, state) {
            continue;
        }

        if cycle.ticks_until_cycle > 0 {
            cycle.ticks_until_cycle -= 1;
        }

        if cycle.ticks_until_cycle <= 0 {
            colony.oil = colony.oil + effects.cycle_oil_production;
            colony.wood = colony.wood - effects.cycle_wood_consumption;
            colony.stone = colony.stone - effects.cycle_stone_consumption;
            colony.iron = colony.iron - effects.cycle_iron_consumption;

            cycle.ticks_until_cycle = seconds_to_ticks(ATLAS_CYCLE_SECONDS);
            cycle.cycles_completed = cycle.cycles_completed.saturating_add(1);
        }
    }
}

fn atlas_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    atlases: Query<
        (
            Entity,
            &AtlasBuildingAnchor,
            &AtlasCoreStats,
            &AtlasBuildState,
            &AtlasEconomy,
            &AtlasFootprint,
            &AtlasState,
            &AtlasEffects,
            &AtlasCycleState,
        ),
        With<AtlasTransmutator>,
    >,
    claims: Res<AtlasPlacementClaims>,
) {
    checksum.accumulate(ATLAS_CYCLE_SECONDS as u64);

    for (entity, anchor, core, build, eco, footprint, state, effects, cycle) in &atlases {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);

        checksum.accumulate(core.defenses_life.to_bits() as u64);
        checksum.accumulate(core.watch_range.to_bits() as u64);
        checksum.accumulate(core.health.current.to_bits() as u64);
        checksum.accumulate(core.health.max.to_bits() as u64);

        checksum.accumulate(build.build_ticks_remaining as u64);
        checksum.accumulate(u64::from(build.completed));

        checksum.accumulate(eco.energy_cost as u64);
        checksum.accumulate(eco.workers_cost as u64);
        checksum.accumulate(eco.gold_maintenance as u64);
        checksum.accumulate(eco.wood_cost as u64);
        checksum.accumulate(eco.stone_cost as u64);
        checksum.accumulate(eco.iron_cost as u64);
        checksum.accumulate(eco.oil_cost as u64);
        checksum.accumulate(eco.gold_cost as u64);

        checksum.accumulate(footprint.size_tiles as u64);
        checksum.accumulate(footprint.watch_range.to_bits() as u64);

        checksum.accumulate(u64::from(state.connected_to_grid));
        checksum.accumulate(u64::from(state.supplied_energy));
        checksum.accumulate(u64::from(state.supplied_workers));
        checksum.accumulate(u64::from(state.supplied_gold_maintenance));
        checksum.accumulate(u64::from(state.destroyed));
        checksum.accumulate(u64::from(state.demolished));
        checksum.accumulate(u64::from(state.research_tier_foundry_met));

        checksum.accumulate(effects.cycle_oil_production.to_bits() as u64);
        checksum.accumulate(effects.cycle_wood_consumption.to_bits() as u64);
        checksum.accumulate(effects.cycle_stone_consumption.to_bits() as u64);
        checksum.accumulate(effects.cycle_iron_consumption.to_bits() as u64);
        checksum.accumulate(effects.victory_points_reward as u64);

        checksum.accumulate(cycle.ticks_until_cycle as u64);
        checksum.accumulate(cycle.cycles_completed as u64);
    }

    checksum.accumulate(claims.claims.len() as u64);
    for (entity, anchor) in &claims.claims {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);
    }
}

pub struct AtlasTransmutatorPlugin;

impl Plugin for AtlasTransmutatorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AtlasPlacementClaims>()
            .add_event::<PlaceAtlasTransmutatorEvent>()
            .add_event::<SetAtlasGridConnectionEvent>()
            .add_event::<SetAtlasSupplyStateEvent>()
            .add_event::<SetAtlasResearchTierEvent>()
            .add_event::<DemolishAtlasTransmutatorEvent>()
            .add_event::<AttackDestroyAtlasTransmutatorEvent>()
            .add_systems(
                FixedUpdate,
                (
                    place_atlas_system,
                    set_atlas_grid_connection_system,
                    set_atlas_supply_state_system,
                    set_atlas_research_tier_system,
                    atlas_build_tick_system,
                    attack_destroy_atlas_system,
                    destroy_atlas_at_zero_health_system,
                    demolish_atlas_system,
                    atlas_transmutation_tick_system,
                    atlas_checksum_system,
                )
                    .chain(),
            );
    }
}
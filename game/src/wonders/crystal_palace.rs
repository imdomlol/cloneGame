// Sources: vault/wonders/crystal_palace.md

use std::collections::BTreeMap;

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, SimChecksumState};

const CRYSTAL_PALACE_HP: I32F32 = I32F32::lit("2000");
const CRYSTAL_PALACE_DEFENSES_LIFE: I32F32 = I32F32::lit("500");
const CRYSTAL_PALACE_WATCH_RANGE: I32F32 = I32F32::lit("12");
const CRYSTAL_PALACE_BUILD_TIME_SECONDS: i32 = 0;
const CRYSTAL_PALACE_SIZE_TILES: i32 = 6;
const CRYSTAL_PALACE_VICTORY_POINTS_REWARD: i32 = 1500;
const CRYSTAL_PALACE_FOOD_PROVIDED: i32 = 800;
const CRYSTAL_PALACE_COLONISTS_REQUIRED: i32 = 60;

const CRYSTAL_PALACE_ENERGY_COST: i32 = 0;
const CRYSTAL_PALACE_WORKERS_COST: i32 = 0;
const CRYSTAL_PALACE_GOLD_MAINTENANCE: i32 = 100;

const CRYSTAL_PALACE_WOOD_COST: i32 = 150;
const CRYSTAL_PALACE_STONE_COST: i32 = 200;
const CRYSTAL_PALACE_IRON_COST: i32 = 150;
const CRYSTAL_PALACE_OIL_COST: i32 = 0;
const CRYSTAL_PALACE_GOLD_COST: i32 = 6000;

const SIM_HZ: i32 = 25;

#[derive(Component, Default)]
pub struct CrystalPalace;

#[derive(Component, Clone, Copy, Default)]
pub struct BuildingAnchor {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Copy)]
pub struct CrystalPalaceCoreStats {
    pub defenses_life: I32F32,
    pub watch_range: I32F32,
    pub health: Health,
}

impl Default for CrystalPalaceCoreStats {
    fn default() -> Self {
        Self {
            defenses_life: CRYSTAL_PALACE_DEFENSES_LIFE,
            watch_range: CRYSTAL_PALACE_WATCH_RANGE,
            health: Health::full(CRYSTAL_PALACE_HP),
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct CrystalPalaceBuildState {
    pub build_ticks_remaining: i32,
    pub completed: bool,
}

#[derive(Component, Clone, Copy)]
pub struct CrystalPalaceEconomy {
    pub energy_cost: i32,
    pub workers_cost: i32,
    pub gold_maintenance: i32,
    pub wood_cost: i32,
    pub stone_cost: i32,
    pub iron_cost: i32,
    pub oil_cost: i32,
    pub gold_cost: i32,
}

impl Default for CrystalPalaceEconomy {
    fn default() -> Self {
        Self {
            energy_cost: CRYSTAL_PALACE_ENERGY_COST,
            workers_cost: CRYSTAL_PALACE_WORKERS_COST,
            gold_maintenance: CRYSTAL_PALACE_GOLD_MAINTENANCE,
            wood_cost: CRYSTAL_PALACE_WOOD_COST,
            stone_cost: CRYSTAL_PALACE_STONE_COST,
            iron_cost: CRYSTAL_PALACE_IRON_COST,
            oil_cost: CRYSTAL_PALACE_OIL_COST,
            gold_cost: CRYSTAL_PALACE_GOLD_COST,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct CrystalPalaceFootprint {
    pub size_tiles: i32,
    pub watch_range: I32F32,
}

impl Default for CrystalPalaceFootprint {
    fn default() -> Self {
        Self {
            size_tiles: CRYSTAL_PALACE_SIZE_TILES,
            watch_range: CRYSTAL_PALACE_WATCH_RANGE,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct CrystalPalaceState {
    pub connected_to_grid: bool,
    pub supplied_energy: bool,
    pub supplied_workers: bool,
    pub supplied_gold_maintenance: bool,
    pub destroyed: bool,
    pub demolished: bool,
    pub research_tier_wood_workshop_met: bool,
}

impl Default for CrystalPalaceState {
    fn default() -> Self {
        Self {
            connected_to_grid: true,
            supplied_energy: true,
            supplied_workers: true,
            supplied_gold_maintenance: true,
            destroyed: false,
            demolished: false,
            research_tier_wood_workshop_met: true,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct CrystalPalaceEffects {
    pub food_provided: i32,
    pub victory_points_reward: i32,
    pub colonists_required: i32,
}

impl Default for CrystalPalaceEffects {
    fn default() -> Self {
        Self {
            food_provided: CRYSTAL_PALACE_FOOD_PROVIDED,
            victory_points_reward: CRYSTAL_PALACE_VICTORY_POINTS_REWARD,
            colonists_required: CRYSTAL_PALACE_COLONISTS_REQUIRED,
        }
    }
}

#[derive(Resource, Default, Clone)]
pub struct CrystalPalacePlacementClaims {
    pub claims: BTreeMap<Entity, BuildingAnchor>,
}

#[derive(Event, Clone, Copy)]
pub struct PlaceCrystalPalaceEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct SetCrystalPalaceGridConnectionEvent {
    pub entity: Entity,
    pub connected: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetCrystalPalaceSupplyStateEvent {
    pub entity: Entity,
    pub supplied_energy: bool,
    pub supplied_workers: bool,
    pub supplied_gold_maintenance: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetCrystalPalaceResearchTierEvent {
    pub entity: Entity,
    pub wood_workshop_met: bool,
}

#[derive(Event, Clone, Copy)]
pub struct DemolishCrystalPalaceEvent {
    pub entity: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct AttackDestroyCrystalPalaceEvent {
    pub entity: Entity,
}

fn seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn place_crystal_palace_system(
    mut commands: Commands,
    mut events: EventReader<PlaceCrystalPalaceEvent>,
    mut claims: ResMut<CrystalPalacePlacementClaims>,
) {
    for ev in events.read() {
        let anchor = BuildingAnchor {
            x: ev.tile_x,
            y: ev.tile_y,
        };

        let entity = commands
            .spawn((
                CrystalPalace,
                anchor,
                CrystalPalaceCoreStats::default(),
                CrystalPalaceBuildState {
                    build_ticks_remaining: seconds_to_ticks(CRYSTAL_PALACE_BUILD_TIME_SECONDS),
                    completed: false,
                },
                CrystalPalaceEconomy::default(),
                CrystalPalaceFootprint::default(),
                CrystalPalaceState::default(),
                CrystalPalaceEffects::default(),
            ))
            .id();

        claims.claims.insert(entity, anchor);
    }
}

fn set_crystal_palace_grid_connection_system(
    mut events: EventReader<SetCrystalPalaceGridConnectionEvent>,
    mut palaces: Query<&mut CrystalPalaceState, With<CrystalPalace>>,
) {
    for ev in events.read() {
        let Ok(mut state) = palaces.get_mut(ev.entity) else {
            continue;
        };
        state.connected_to_grid = ev.connected;
    }
}

fn set_crystal_palace_supply_state_system(
    mut events: EventReader<SetCrystalPalaceSupplyStateEvent>,
    mut palaces: Query<&mut CrystalPalaceState, With<CrystalPalace>>,
) {
    for ev in events.read() {
        let Ok(mut state) = palaces.get_mut(ev.entity) else {
            continue;
        };
        state.supplied_energy = ev.supplied_energy;
        state.supplied_workers = ev.supplied_workers;
        state.supplied_gold_maintenance = ev.supplied_gold_maintenance;
    }
}

fn set_crystal_palace_research_tier_system(
    mut events: EventReader<SetCrystalPalaceResearchTierEvent>,
    mut palaces: Query<&mut CrystalPalaceState, With<CrystalPalace>>,
) {
    for ev in events.read() {
        let Ok(mut state) = palaces.get_mut(ev.entity) else {
            continue;
        };
        state.research_tier_wood_workshop_met = ev.wood_workshop_met;
    }
}

fn crystal_palace_build_tick_system(mut palaces: Query<&mut CrystalPalaceBuildState, With<CrystalPalace>>) {
    for mut build in &mut palaces {
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

fn attack_destroy_crystal_palace_system(
    mut events: EventReader<AttackDestroyCrystalPalaceEvent>,
    mut palaces: Query<&mut CrystalPalaceState, With<CrystalPalace>>,
) {
    for ev in events.read() {
        let Ok(mut state) = palaces.get_mut(ev.entity) else {
            continue;
        };
        state.destroyed = true;
    }
}

fn destroy_crystal_palace_at_zero_health_system(
    mut palaces: Query<(&CrystalPalaceCoreStats, &mut CrystalPalaceState), With<CrystalPalace>>,
) {
    for (core, mut state) in &mut palaces {
        if core.health.current <= I32F32::ZERO {
            state.destroyed = true;
        }
    }
}

fn demolish_crystal_palace_system(
    mut events: EventReader<DemolishCrystalPalaceEvent>,
    mut palaces: Query<&mut CrystalPalaceState, With<CrystalPalace>>,
) {
    for ev in events.read() {
        let Ok(mut state) = palaces.get_mut(ev.entity) else {
            continue;
        };
        state.demolished = true;
    }
}

fn crystal_palace_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    palaces: Query<
        (
            Entity,
            &BuildingAnchor,
            &CrystalPalaceCoreStats,
            &CrystalPalaceBuildState,
            &CrystalPalaceEconomy,
            &CrystalPalaceFootprint,
            &CrystalPalaceState,
            &CrystalPalaceEffects,
        ),
        With<CrystalPalace>,
    >,
    claims: Res<CrystalPalacePlacementClaims>,
) {
    for (entity, anchor, core, build, eco, footprint, state, effects) in &palaces {
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
        checksum.accumulate(u64::from(state.research_tier_wood_workshop_met));

        checksum.accumulate(effects.food_provided as u64);
        checksum.accumulate(effects.victory_points_reward as u64);
        checksum.accumulate(effects.colonists_required as u64);
    }

    checksum.accumulate(claims.claims.len() as u64);
    for (entity, anchor) in &claims.claims {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);
    }
}

pub struct CrystalPalacePlugin;

impl Plugin for CrystalPalacePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CrystalPalacePlacementClaims>()
            .add_event::<PlaceCrystalPalaceEvent>()
            .add_event::<SetCrystalPalaceGridConnectionEvent>()
            .add_event::<SetCrystalPalaceSupplyStateEvent>()
            .add_event::<SetCrystalPalaceResearchTierEvent>()
            .add_event::<DemolishCrystalPalaceEvent>()
            .add_event::<AttackDestroyCrystalPalaceEvent>()
            .add_systems(
                FixedUpdate,
                (
                    place_crystal_palace_system,
                    set_crystal_palace_grid_connection_system,
                    set_crystal_palace_supply_state_system,
                    set_crystal_palace_research_tier_system,
                    crystal_palace_build_tick_system,
                    attack_destroy_crystal_palace_system,
                    destroy_crystal_palace_at_zero_health_system,
                    demolish_crystal_palace_system,
                    crystal_palace_checksum_system,
                )
                    .chain(),
            );
    }
}
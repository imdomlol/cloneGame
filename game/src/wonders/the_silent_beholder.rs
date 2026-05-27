// Sources: vault/wonders/the_silent_beholder.md

use std::collections::BTreeMap;

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, SimChecksumState};

const THE_SILENT_BEHOLDER_HP: I32F32 = I32F32::lit("0");
const THE_SILENT_BEHOLDER_DEFENSES_LIFE: I32F32 = I32F32::lit("0");
const THE_SILENT_BEHOLDER_WATCH_RANGE: I32F32 = I32F32::lit("0");
const THE_SILENT_BEHOLDER_BUILD_TIME_SECONDS: i32 = 25;
const THE_SILENT_BEHOLDER_SIZE_TILES: i32 = 0;
const THE_SILENT_BEHOLDER_VICTORY_POINTS_REWARD: i32 = 1000;

const THE_SILENT_BEHOLDER_ENERGY_COST: i32 = 0;
const THE_SILENT_BEHOLDER_WORKERS_COST: i32 = 0;
const THE_SILENT_BEHOLDER_GOLD_MAINTENANCE: i32 = 0;

const THE_SILENT_BEHOLDER_WOOD_COST: i32 = 0;
const THE_SILENT_BEHOLDER_STONE_COST: i32 = 0;
const THE_SILENT_BEHOLDER_IRON_COST: i32 = 0;
const THE_SILENT_BEHOLDER_OIL_COST: i32 = 0;
const THE_SILENT_BEHOLDER_GOLD_COST: i32 = 0;

const SIM_HZ: i32 = 25;

#[derive(Component, Default)]
pub struct TheSilentBeholder;

#[derive(Component, Clone, Copy, Default)]
pub struct BuildingAnchor {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Copy)]
pub struct TheSilentBeholderCoreStats {
    pub defenses_life: I32F32,
    pub watch_range: I32F32,
    pub health: Health,
}

impl Default for TheSilentBeholderCoreStats {
    fn default() -> Self {
        Self {
            defenses_life: THE_SILENT_BEHOLDER_DEFENSES_LIFE,
            watch_range: THE_SILENT_BEHOLDER_WATCH_RANGE,
            health: Health::full(THE_SILENT_BEHOLDER_HP),
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct TheSilentBeholderBuildState {
    pub build_ticks_remaining: i32,
    pub completed: bool,
}

#[derive(Component, Clone, Copy)]
pub struct TheSilentBeholderEconomy {
    pub energy_cost: i32,
    pub workers_cost: i32,
    pub gold_maintenance: i32,
    pub wood_cost: i32,
    pub stone_cost: i32,
    pub iron_cost: i32,
    pub oil_cost: i32,
    pub gold_cost: i32,
}

impl Default for TheSilentBeholderEconomy {
    fn default() -> Self {
        Self {
            energy_cost: THE_SILENT_BEHOLDER_ENERGY_COST,
            workers_cost: THE_SILENT_BEHOLDER_WORKERS_COST,
            gold_maintenance: THE_SILENT_BEHOLDER_GOLD_MAINTENANCE,
            wood_cost: THE_SILENT_BEHOLDER_WOOD_COST,
            stone_cost: THE_SILENT_BEHOLDER_STONE_COST,
            iron_cost: THE_SILENT_BEHOLDER_IRON_COST,
            oil_cost: THE_SILENT_BEHOLDER_OIL_COST,
            gold_cost: THE_SILENT_BEHOLDER_GOLD_COST,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct TheSilentBeholderFootprint {
    pub size_tiles: i32,
    pub watch_range: I32F32,
}

impl Default for TheSilentBeholderFootprint {
    fn default() -> Self {
        Self {
            size_tiles: THE_SILENT_BEHOLDER_SIZE_TILES,
            watch_range: THE_SILENT_BEHOLDER_WATCH_RANGE,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct TheSilentBeholderState {
    pub connected_to_grid: bool,
    pub supplied_energy: bool,
    pub supplied_workers: bool,
    pub supplied_gold_maintenance: bool,
    pub destroyed: bool,
    pub demolished: bool,
    pub research_tier_wood_workshop_met: bool,
    pub command_center_repaired: bool,
    pub map_revealed: bool,
    pub blocks_new_building_construction: bool,
    pub lookout_tower_unnecessary: bool,
    pub radar_tower_unnecessary: bool,
}

impl Default for TheSilentBeholderState {
    fn default() -> Self {
        Self {
            connected_to_grid: true,
            supplied_energy: true,
            supplied_workers: true,
            supplied_gold_maintenance: true,
            destroyed: false,
            demolished: false,
            research_tier_wood_workshop_met: true,
            command_center_repaired: true,
            map_revealed: false,
            blocks_new_building_construction: false,
            lookout_tower_unnecessary: false,
            radar_tower_unnecessary: false,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct TheSilentBeholderEffects {
    pub reveals_entire_map: bool,
    pub no_maintenance_workers_or_power_after_completion: bool,
    pub victory_points_reward: i32,
}

impl Default for TheSilentBeholderEffects {
    fn default() -> Self {
        Self {
            reveals_entire_map: true,
            no_maintenance_workers_or_power_after_completion: true,
            victory_points_reward: THE_SILENT_BEHOLDER_VICTORY_POINTS_REWARD,
        }
    }
}

#[derive(Resource, Default, Clone)]
pub struct TheSilentBeholderPlacementClaims {
    pub claims: BTreeMap<Entity, BuildingAnchor>,
}

#[derive(Event, Clone, Copy)]
pub struct PlaceTheSilentBeholderEvent {
    pub tile_x: i32,
    pub tile_y: i32,
    pub command_center_repaired: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetTheSilentBeholderGridConnectionEvent {
    pub entity: Entity,
    pub connected: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetTheSilentBeholderSupplyStateEvent {
    pub entity: Entity,
    pub supplied_energy: bool,
    pub supplied_workers: bool,
    pub supplied_gold_maintenance: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetTheSilentBeholderResearchTierEvent {
    pub entity: Entity,
    pub wood_workshop_met: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetTheSilentBeholderCommandCenterRepairEvent {
    pub entity: Entity,
    pub repaired: bool,
}

#[derive(Event, Clone, Copy)]
pub struct DemolishTheSilentBeholderEvent {
    pub entity: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct AttackDestroyTheSilentBeholderEvent {
    pub entity: Entity,
}

fn seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn place_the_silent_beholder_system(
    mut commands: Commands,
    mut events: EventReader<PlaceTheSilentBeholderEvent>,
    mut claims: ResMut<TheSilentBeholderPlacementClaims>,
) {
    for ev in events.read() {
        let anchor = BuildingAnchor {
            x: ev.tile_x,
            y: ev.tile_y,
        };

        let entity = commands
            .spawn((
                TheSilentBeholder,
                anchor,
                TheSilentBeholderCoreStats::default(),
                TheSilentBeholderBuildState {
                    build_ticks_remaining: seconds_to_ticks(THE_SILENT_BEHOLDER_BUILD_TIME_SECONDS),
                    completed: false,
                },
                TheSilentBeholderEconomy::default(),
                TheSilentBeholderFootprint::default(),
                TheSilentBeholderState {
                    command_center_repaired: ev.command_center_repaired,
                    ..Default::default()
                },
                TheSilentBeholderEffects::default(),
            ))
            .id();

        claims.claims.insert(entity, anchor);
    }
}

fn set_the_silent_beholder_grid_connection_system(
    mut events: EventReader<SetTheSilentBeholderGridConnectionEvent>,
    mut wonders: Query<&mut TheSilentBeholderState, With<TheSilentBeholder>>,
) {
    for ev in events.read() {
        let Ok(mut state) = wonders.get_mut(ev.entity) else {
            continue;
        };
        state.connected_to_grid = ev.connected;
    }
}

fn set_the_silent_beholder_supply_state_system(
    mut events: EventReader<SetTheSilentBeholderSupplyStateEvent>,
    mut wonders: Query<&mut TheSilentBeholderState, With<TheSilentBeholder>>,
) {
    for ev in events.read() {
        let Ok(mut state) = wonders.get_mut(ev.entity) else {
            continue;
        };
        state.supplied_energy = ev.supplied_energy;
        state.supplied_workers = ev.supplied_workers;
        state.supplied_gold_maintenance = ev.supplied_gold_maintenance;
    }
}

fn set_the_silent_beholder_research_tier_system(
    mut events: EventReader<SetTheSilentBeholderResearchTierEvent>,
    mut wonders: Query<&mut TheSilentBeholderState, With<TheSilentBeholder>>,
) {
    for ev in events.read() {
        let Ok(mut state) = wonders.get_mut(ev.entity) else {
            continue;
        };
        state.research_tier_wood_workshop_met = ev.wood_workshop_met;
    }
}

fn set_the_silent_beholder_command_center_repair_system(
    mut events: EventReader<SetTheSilentBeholderCommandCenterRepairEvent>,
    mut wonders: Query<&mut TheSilentBeholderState, With<TheSilentBeholder>>,
) {
    for ev in events.read() {
        let Ok(mut state) = wonders.get_mut(ev.entity) else {
            continue;
        };
        state.command_center_repaired = ev.repaired;
    }
}

fn the_silent_beholder_build_tick_system(
    mut wonders: Query<(&mut TheSilentBeholderBuildState, &TheSilentBeholderState), With<TheSilentBeholder>>,
) {
    for (mut build, state) in &mut wonders {
        if build.completed {
            continue;
        }

        if !state.command_center_repaired {
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

fn the_silent_beholder_behavior_system(
    mut wonders: Query<
        (
            &TheSilentBeholderBuildState,
            &mut TheSilentBeholderState,
            &TheSilentBeholderEffects,
            &mut TheSilentBeholderEconomy,
        ),
        With<TheSilentBeholder>,
    >,
) {
    for (build, mut state, effects, mut economy) in &mut wonders {
        state.blocks_new_building_construction = !build.completed;

        if build.completed && effects.reveals_entire_map {
            state.map_revealed = true;
            state.lookout_tower_unnecessary = true;
            state.radar_tower_unnecessary = true;
        }

        if build.completed && effects.no_maintenance_workers_or_power_after_completion {
            economy.energy_cost = 0;
            economy.workers_cost = 0;
            economy.gold_maintenance = 0;
        }
    }
}

fn attack_destroy_the_silent_beholder_system(
    mut events: EventReader<AttackDestroyTheSilentBeholderEvent>,
    mut wonders: Query<&mut TheSilentBeholderState, With<TheSilentBeholder>>,
) {
    for ev in events.read() {
        let Ok(mut state) = wonders.get_mut(ev.entity) else {
            continue;
        };
        state.destroyed = true;
    }
}

fn destroy_the_silent_beholder_at_zero_health_system(
    mut wonders: Query<(&TheSilentBeholderCoreStats, &mut TheSilentBeholderState), With<TheSilentBeholder>>,
) {
    for (core, mut state) in &mut wonders {
        if core.health.current <= I32F32::ZERO {
            state.destroyed = true;
        }
    }
}

fn demolish_the_silent_beholder_system(
    mut events: EventReader<DemolishTheSilentBeholderEvent>,
    mut wonders: Query<&mut TheSilentBeholderState, With<TheSilentBeholder>>,
) {
    for ev in events.read() {
        let Ok(mut state) = wonders.get_mut(ev.entity) else {
            continue;
        };
        state.demolished = true;
    }
}

fn the_silent_beholder_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    wonders: Query<
        (
            Entity,
            &BuildingAnchor,
            &TheSilentBeholderCoreStats,
            &TheSilentBeholderBuildState,
            &TheSilentBeholderEconomy,
            &TheSilentBeholderFootprint,
            &TheSilentBeholderState,
            &TheSilentBeholderEffects,
        ),
        With<TheSilentBeholder>,
    >,
    claims: Res<TheSilentBeholderPlacementClaims>,
) {
    for (entity, anchor, core, build, eco, footprint, state, effects) in &wonders {
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
        checksum.accumulate(u64::from(state.command_center_repaired));
        checksum.accumulate(u64::from(state.map_revealed));
        checksum.accumulate(u64::from(state.blocks_new_building_construction));
        checksum.accumulate(u64::from(state.lookout_tower_unnecessary));
        checksum.accumulate(u64::from(state.radar_tower_unnecessary));

        checksum.accumulate(u64::from(effects.reveals_entire_map));
        checksum.accumulate(u64::from(
            effects.no_maintenance_workers_or_power_after_completion,
        ));
        checksum.accumulate(effects.victory_points_reward as u64);
    }

    checksum.accumulate(claims.claims.len() as u64);
    for (entity, anchor) in &claims.claims {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);
    }
}

pub struct TheSilentBeholderPlugin;

impl Plugin for TheSilentBeholderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TheSilentBeholderPlacementClaims>()
            .add_event::<PlaceTheSilentBeholderEvent>()
            .add_event::<SetTheSilentBeholderGridConnectionEvent>()
            .add_event::<SetTheSilentBeholderSupplyStateEvent>()
            .add_event::<SetTheSilentBeholderResearchTierEvent>()
            .add_event::<SetTheSilentBeholderCommandCenterRepairEvent>()
            .add_event::<DemolishTheSilentBeholderEvent>()
            .add_event::<AttackDestroyTheSilentBeholderEvent>()
            .add_systems(
                FixedUpdate,
                (
                    place_the_silent_beholder_system,
                    set_the_silent_beholder_grid_connection_system,
                    set_the_silent_beholder_supply_state_system,
                    set_the_silent_beholder_research_tier_system,
                    set_the_silent_beholder_command_center_repair_system,
                    the_silent_beholder_build_tick_system,
                    the_silent_beholder_behavior_system,
                    attack_destroy_the_silent_beholder_system,
                    destroy_the_silent_beholder_at_zero_health_system,
                    demolish_the_silent_beholder_system,
                    the_silent_beholder_checksum_system,
                )
                    .chain(),
            );
    }
}
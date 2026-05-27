// Sources: vault/wonders/the_victorious.md

use std::collections::BTreeMap;

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, SimChecksumState};

const VICTORIOUS_HP: I32F32 = I32F32::lit("10000");
const VICTORIOUS_DEFENSES_LIFE: I32F32 = I32F32::lit("8000");
const VICTORIOUS_DEFENSES_LIFE_BARRIER_UPGRADED: I32F32 = I32F32::lit("10400");
const VICTORIOUS_WATCH_RANGE: I32F32 = I32F32::lit("16");
const VICTORIOUS_BUILD_TIME_SECONDS: i32 = 0;
const VICTORIOUS_SIZE_TILES: i32 = 4;
const VICTORIOUS_VICTORY_POINTS_REWARD: i32 = 2000;

const VICTORIOUS_ENERGY_COST: i32 = 50;
const VICTORIOUS_WORKERS_COST: i32 = 50;
const VICTORIOUS_OIL_MAINTENANCE: i32 = 10;
const VICTORIOUS_GOLD_MAINTENANCE: i32 = 120;

const VICTORIOUS_WOOD_COST: i32 = 0;
const VICTORIOUS_STONE_COST: i32 = 100;
const VICTORIOUS_IRON_COST: i32 = 200;
const VICTORIOUS_OIL_COST: i32 = 100;
const VICTORIOUS_GOLD_COST: i32 = 7000;

const VICTORIOUS_GOLD_BONUS_PERCENT: i32 = 20;
const VICTORIOUS_GOLD_BONUS_RADIUS_BLOCKS: I32F32 = I32F32::lit("24");
const VICTORIOUS_COVERAGE_TILES: i32 = 52;
const VICTORIOUS_MAX_SUPPORTED_DWELLINGS: i32 = 514;

const SIM_HZ: i32 = 25;

#[derive(Component, Default)]
pub struct TheVictorious;

#[derive(Component, Clone, Copy, Default)]
pub struct BuildingAnchor {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Copy)]
pub struct VictoriousCoreStats {
    pub defenses_life: I32F32,
    pub watch_range: I32F32,
    pub health: Health,
}

impl Default for VictoriousCoreStats {
    fn default() -> Self {
        Self {
            defenses_life: VICTORIOUS_DEFENSES_LIFE,
            watch_range: VICTORIOUS_WATCH_RANGE,
            health: Health::full(VICTORIOUS_HP),
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct VictoriousBuildState {
    pub build_ticks_remaining: i32,
    pub completed: bool,
}

#[derive(Component, Clone, Copy)]
pub struct VictoriousEconomy {
    pub energy_cost: i32,
    pub workers_cost: i32,
    pub oil_maintenance: i32,
    pub gold_maintenance: i32,
    pub wood_cost: i32,
    pub stone_cost: i32,
    pub iron_cost: i32,
    pub oil_cost: i32,
    pub gold_cost: i32,
}

impl Default for VictoriousEconomy {
    fn default() -> Self {
        Self {
            energy_cost: VICTORIOUS_ENERGY_COST,
            workers_cost: VICTORIOUS_WORKERS_COST,
            oil_maintenance: VICTORIOUS_OIL_MAINTENANCE,
            gold_maintenance: VICTORIOUS_GOLD_MAINTENANCE,
            wood_cost: VICTORIOUS_WOOD_COST,
            stone_cost: VICTORIOUS_STONE_COST,
            iron_cost: VICTORIOUS_IRON_COST,
            oil_cost: VICTORIOUS_OIL_COST,
            gold_cost: VICTORIOUS_GOLD_COST,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct VictoriousFootprint {
    pub size_tiles: i32,
    pub watch_range: I32F32,
    pub coverage_tiles: i32,
}

impl Default for VictoriousFootprint {
    fn default() -> Self {
        Self {
            size_tiles: VICTORIOUS_SIZE_TILES,
            watch_range: VICTORIOUS_WATCH_RANGE,
            coverage_tiles: VICTORIOUS_COVERAGE_TILES,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct VictoriousState {
    pub connected_to_grid: bool,
    pub supplied_energy: bool,
    pub supplied_workers: bool,
    pub supplied_oil_maintenance: bool,
    pub supplied_gold_maintenance: bool,
    pub destroyed: bool,
    pub demolished: bool,
    pub research_tier_stone_workshop_met: bool,
    pub barrier_upgrades_applied: bool,
    pub bank_bonus_present: bool,
    pub inn_bonus_present: bool,
}

impl Default for VictoriousState {
    fn default() -> Self {
        Self {
            connected_to_grid: true,
            supplied_energy: true,
            supplied_workers: true,
            supplied_oil_maintenance: true,
            supplied_gold_maintenance: true,
            destroyed: false,
            demolished: false,
            research_tier_stone_workshop_met: true,
            barrier_upgrades_applied: false,
            bank_bonus_present: false,
            inn_bonus_present: false,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct VictoriousEffects {
    pub gold_bonus_percent: i32,
    pub gold_bonus_radius_blocks: I32F32,
    pub stacks_with_bank: bool,
    pub stacks_with_inn: bool,
    pub victory_points_reward: i32,
    pub max_supported_dwellings: i32,
}

impl Default for VictoriousEffects {
    fn default() -> Self {
        Self {
            gold_bonus_percent: VICTORIOUS_GOLD_BONUS_PERCENT,
            gold_bonus_radius_blocks: VICTORIOUS_GOLD_BONUS_RADIUS_BLOCKS,
            stacks_with_bank: true,
            stacks_with_inn: true,
            victory_points_reward: VICTORIOUS_VICTORY_POINTS_REWARD,
            max_supported_dwellings: VICTORIOUS_MAX_SUPPORTED_DWELLINGS,
        }
    }
}

#[derive(Resource, Default, Clone)]
pub struct VictoriousPlacementClaims {
    pub claims: BTreeMap<Entity, BuildingAnchor>,
}

#[derive(Event, Clone, Copy)]
pub struct PlaceTheVictoriousEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct SetVictoriousGridConnectionEvent {
    pub entity: Entity,
    pub connected: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetVictoriousSupplyStateEvent {
    pub entity: Entity,
    pub supplied_energy: bool,
    pub supplied_workers: bool,
    pub supplied_oil_maintenance: bool,
    pub supplied_gold_maintenance: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetVictoriousResearchTierEvent {
    pub entity: Entity,
    pub stone_workshop_met: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetVictoriousBarrierUpgradeEvent {
    pub entity: Entity,
    pub applied: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetVictoriousStackingStateEvent {
    pub entity: Entity,
    pub bank_bonus_present: bool,
    pub inn_bonus_present: bool,
}

#[derive(Event, Clone, Copy)]
pub struct DemolishVictoriousEvent {
    pub entity: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct AttackDestroyVictoriousEvent {
    pub entity: Entity,
}

fn seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn victorious_is_active(build: &VictoriousBuildState, state: &VictoriousState) -> bool {
    build.completed
        && state.connected_to_grid
        && state.supplied_energy
        && state.supplied_workers
        && state.supplied_oil_maintenance
        && state.supplied_gold_maintenance
        && state.research_tier_stone_workshop_met
        && !state.destroyed
        && !state.demolished
}

fn place_victorious_system(
    mut commands: Commands,
    mut events: EventReader<PlaceTheVictoriousEvent>,
    mut claims: ResMut<VictoriousPlacementClaims>,
) {
    for ev in events.read() {
        let anchor = BuildingAnchor {
            x: ev.tile_x,
            y: ev.tile_y,
        };

        let entity = commands
            .spawn((
                TheVictorious,
                anchor,
                VictoriousCoreStats::default(),
                VictoriousBuildState {
                    build_ticks_remaining: seconds_to_ticks(VICTORIOUS_BUILD_TIME_SECONDS),
                    completed: false,
                },
                VictoriousEconomy::default(),
                VictoriousFootprint::default(),
                VictoriousState::default(),
                VictoriousEffects::default(),
            ))
            .id();

        claims.claims.insert(entity, anchor);
    }
}

fn set_victorious_grid_connection_system(
    mut events: EventReader<SetVictoriousGridConnectionEvent>,
    mut wonders: Query<&mut VictoriousState, With<TheVictorious>>,
) {
    for ev in events.read() {
        let Ok(mut state) = wonders.get_mut(ev.entity) else {
            continue;
        };
        state.connected_to_grid = ev.connected;
    }
}

fn set_victorious_supply_state_system(
    mut events: EventReader<SetVictoriousSupplyStateEvent>,
    mut wonders: Query<&mut VictoriousState, With<TheVictorious>>,
) {
    for ev in events.read() {
        let Ok(mut state) = wonders.get_mut(ev.entity) else {
            continue;
        };
        state.supplied_energy = ev.supplied_energy;
        state.supplied_workers = ev.supplied_workers;
        state.supplied_oil_maintenance = ev.supplied_oil_maintenance;
        state.supplied_gold_maintenance = ev.supplied_gold_maintenance;
    }
}

fn set_victorious_research_tier_system(
    mut events: EventReader<SetVictoriousResearchTierEvent>,
    mut wonders: Query<&mut VictoriousState, With<TheVictorious>>,
) {
    for ev in events.read() {
        let Ok(mut state) = wonders.get_mut(ev.entity) else {
            continue;
        };
        state.research_tier_stone_workshop_met = ev.stone_workshop_met;
    }
}

fn set_victorious_barrier_upgrade_system(
    mut events: EventReader<SetVictoriousBarrierUpgradeEvent>,
    mut wonders: Query<(&mut VictoriousCoreStats, &mut VictoriousState), With<TheVictorious>>,
) {
    for ev in events.read() {
        let Ok((mut core, mut state)) = wonders.get_mut(ev.entity) else {
            continue;
        };

        state.barrier_upgrades_applied = ev.applied;
        core.defenses_life = if ev.applied {
            VICTORIOUS_DEFENSES_LIFE_BARRIER_UPGRADED
        } else {
            VICTORIOUS_DEFENSES_LIFE
        };
    }
}

fn set_victorious_stacking_state_system(
    mut events: EventReader<SetVictoriousStackingStateEvent>,
    mut wonders: Query<&mut VictoriousState, With<TheVictorious>>,
) {
    for ev in events.read() {
        let Ok(mut state) = wonders.get_mut(ev.entity) else {
            continue;
        };
        state.bank_bonus_present = ev.bank_bonus_present;
        state.inn_bonus_present = ev.inn_bonus_present;
    }
}

fn victorious_build_tick_system(mut wonders: Query<&mut VictoriousBuildState, With<TheVictorious>>) {
    for mut build in &mut wonders {
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

fn attack_destroy_victorious_system(
    mut events: EventReader<AttackDestroyVictoriousEvent>,
    mut wonders: Query<&mut VictoriousState, With<TheVictorious>>,
) {
    for ev in events.read() {
        let Ok(mut state) = wonders.get_mut(ev.entity) else {
            continue;
        };
        state.destroyed = true;
    }
}

fn destroy_victorious_at_zero_health_system(
    mut wonders: Query<(&VictoriousCoreStats, &mut VictoriousState), With<TheVictorious>>,
) {
    for (core, mut state) in &mut wonders {
        if core.health.current <= I32F32::ZERO {
            state.destroyed = true;
        }
    }
}

fn demolish_victorious_system(
    mut events: EventReader<DemolishVictoriousEvent>,
    mut wonders: Query<&mut VictoriousState, With<TheVictorious>>,
) {
    for ev in events.read() {
        let Ok(mut state) = wonders.get_mut(ev.entity) else {
            continue;
        };
        state.demolished = true;
    }
}

fn victorious_effect_state_system(
    wonders: Query<(&VictoriousBuildState, &VictoriousState, &VictoriousEffects), With<TheVictorious>>,
    mut checksum: ResMut<SimChecksumState>,
) {
    for (build, state, effects) in &wonders {
        let active = victorious_is_active(build, state);
        checksum.accumulate(u64::from(active));
        if !active {
            continue;
        }

        let mut bonus_percent = effects.gold_bonus_percent;
        if state.bank_bonus_present && effects.stacks_with_bank {
            bonus_percent += effects.gold_bonus_percent;
        }
        if state.inn_bonus_present && effects.stacks_with_inn {
            bonus_percent += effects.gold_bonus_percent;
        }

        checksum.accumulate(bonus_percent as u64);
        checksum.accumulate(effects.gold_bonus_radius_blocks.to_bits() as u64);
    }
}

fn victorious_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    wonders: Query<
        (
            Entity,
            &BuildingAnchor,
            &VictoriousCoreStats,
            &VictoriousBuildState,
            &VictoriousEconomy,
            &VictoriousFootprint,
            &VictoriousState,
            &VictoriousEffects,
        ),
        With<TheVictorious>,
    >,
    claims: Res<VictoriousPlacementClaims>,
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
        checksum.accumulate(eco.oil_maintenance as u64);
        checksum.accumulate(eco.gold_maintenance as u64);
        checksum.accumulate(eco.wood_cost as u64);
        checksum.accumulate(eco.stone_cost as u64);
        checksum.accumulate(eco.iron_cost as u64);
        checksum.accumulate(eco.oil_cost as u64);
        checksum.accumulate(eco.gold_cost as u64);

        checksum.accumulate(footprint.size_tiles as u64);
        checksum.accumulate(footprint.watch_range.to_bits() as u64);
        checksum.accumulate(footprint.coverage_tiles as u64);

        checksum.accumulate(u64::from(state.connected_to_grid));
        checksum.accumulate(u64::from(state.supplied_energy));
        checksum.accumulate(u64::from(state.supplied_workers));
        checksum.accumulate(u64::from(state.supplied_oil_maintenance));
        checksum.accumulate(u64::from(state.supplied_gold_maintenance));
        checksum.accumulate(u64::from(state.destroyed));
        checksum.accumulate(u64::from(state.demolished));
        checksum.accumulate(u64::from(state.research_tier_stone_workshop_met));
        checksum.accumulate(u64::from(state.barrier_upgrades_applied));
        checksum.accumulate(u64::from(state.bank_bonus_present));
        checksum.accumulate(u64::from(state.inn_bonus_present));

        checksum.accumulate(effects.gold_bonus_percent as u64);
        checksum.accumulate(effects.gold_bonus_radius_blocks.to_bits() as u64);
        checksum.accumulate(u64::from(effects.stacks_with_bank));
        checksum.accumulate(u64::from(effects.stacks_with_inn));
        checksum.accumulate(effects.victory_points_reward as u64);
        checksum.accumulate(effects.max_supported_dwellings as u64);
    }

    checksum.accumulate(claims.claims.len() as u64);
    for (entity, anchor) in &claims.claims {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);
    }
}

pub struct TheVictoriousPlugin;

impl Plugin for TheVictoriousPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VictoriousPlacementClaims>()
            .add_event::<PlaceTheVictoriousEvent>()
            .add_event::<SetVictoriousGridConnectionEvent>()
            .add_event::<SetVictoriousSupplyStateEvent>()
            .add_event::<SetVictoriousResearchTierEvent>()
            .add_event::<SetVictoriousBarrierUpgradeEvent>()
            .add_event::<SetVictoriousStackingStateEvent>()
            .add_event::<DemolishVictoriousEvent>()
            .add_event::<AttackDestroyVictoriousEvent>()
            .add_systems(
                FixedUpdate,
                (
                    place_victorious_system,
                    set_victorious_grid_connection_system,
                    set_victorious_supply_state_system,
                    set_victorious_research_tier_system,
                    set_victorious_barrier_upgrade_system,
                    set_victorious_stacking_state_system,
                    victorious_build_tick_system,
                    attack_destroy_victorious_system,
                    destroy_victorious_at_zero_health_system,
                    demolish_victorious_system,
                    victorious_effect_state_system,
                    victorious_checksum_system,
                )
                    .chain(),
            );
    }
}
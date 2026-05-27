// Sources: vault/wonders/the_lightning_spire.md

use std::collections::BTreeMap;

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, SimChecksumState};

const LIGHTNING_SPIRE_HP: I32F32 = I32F32::lit("3000");
const LIGHTNING_SPIRE_DEFENSES_LIFE: I32F32 = I32F32::lit("750");
const LIGHTNING_SPIRE_WATCH_RANGE: I32F32 = I32F32::lit("36");
const LIGHTNING_SPIRE_BUILD_TIME_SECONDS: i32 = 0;
const LIGHTNING_SPIRE_SIZE_TILES: i32 = 6;
const LIGHTNING_SPIRE_VICTORY_POINTS_REWARD: i32 = 1500;

const LIGHTNING_SPIRE_ENERGY_OUTPUT: i32 = 800;
const LIGHTNING_SPIRE_ENERGY_TRANSFER_RADIUS: I32F32 = I32F32::lit("30");
const LIGHTNING_SPIRE_WORKERS_COST: i32 = 60;
const LIGHTNING_SPIRE_GOLD_MAINTENANCE: i32 = 100;

const LIGHTNING_SPIRE_WOOD_COST: i32 = 0;
const LIGHTNING_SPIRE_STONE_COST: i32 = 0;
const LIGHTNING_SPIRE_IRON_COST: i32 = 0;
const LIGHTNING_SPIRE_OIL_COST: i32 = 0;
const LIGHTNING_SPIRE_GOLD_COST: i32 = 0;

const SIM_HZ: i32 = 25;

#[derive(Component, Default)]
pub struct LightningSpire;

#[derive(Component, Clone, Copy, Default)]
pub struct BuildingAnchor {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Copy)]
pub struct LightningSpireCoreStats {
    pub defenses_life: I32F32,
    pub watch_range: I32F32,
    pub health: Health,
}

impl Default for LightningSpireCoreStats {
    fn default() -> Self {
        Self {
            defenses_life: LIGHTNING_SPIRE_DEFENSES_LIFE,
            watch_range: LIGHTNING_SPIRE_WATCH_RANGE,
            health: Health::full(LIGHTNING_SPIRE_HP),
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct LightningSpireBuildState {
    pub build_ticks_remaining: i32,
    pub completed: bool,
}

#[derive(Component, Clone, Copy)]
pub struct LightningSpireEconomy {
    pub workers_cost: i32,
    pub gold_maintenance: i32,
    pub wood_cost: i32,
    pub stone_cost: i32,
    pub iron_cost: i32,
    pub oil_cost: i32,
    pub gold_cost: i32,
}

impl Default for LightningSpireEconomy {
    fn default() -> Self {
        Self {
            workers_cost: LIGHTNING_SPIRE_WORKERS_COST,
            gold_maintenance: LIGHTNING_SPIRE_GOLD_MAINTENANCE,
            wood_cost: LIGHTNING_SPIRE_WOOD_COST,
            stone_cost: LIGHTNING_SPIRE_STONE_COST,
            iron_cost: LIGHTNING_SPIRE_IRON_COST,
            oil_cost: LIGHTNING_SPIRE_OIL_COST,
            gold_cost: LIGHTNING_SPIRE_GOLD_COST,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct LightningSpireFootprint {
    pub size_tiles: i32,
    pub watch_range: I32F32,
}

impl Default for LightningSpireFootprint {
    fn default() -> Self {
        Self {
            size_tiles: LIGHTNING_SPIRE_SIZE_TILES,
            watch_range: LIGHTNING_SPIRE_WATCH_RANGE,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct LightningSpireState {
    pub connected_to_grid: bool,
    pub supplied_workers: bool,
    pub supplied_gold_maintenance: bool,
    pub destroyed: bool,
    pub demolished: bool,
    pub research_tier_foundry_met: bool,
}

impl Default for LightningSpireState {
    fn default() -> Self {
        Self {
            connected_to_grid: true,
            supplied_workers: true,
            supplied_gold_maintenance: true,
            destroyed: false,
            demolished: false,
            research_tier_foundry_met: true,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct LightningSpireEffects {
    pub energy_output: i32,
    pub energy_transfer_radius: I32F32,
    pub functions_as_large_tesla_tower: bool,
    pub victory_points_reward: i32,
}

impl Default for LightningSpireEffects {
    fn default() -> Self {
        Self {
            energy_output: LIGHTNING_SPIRE_ENERGY_OUTPUT,
            energy_transfer_radius: LIGHTNING_SPIRE_ENERGY_TRANSFER_RADIUS,
            functions_as_large_tesla_tower: true,
            victory_points_reward: LIGHTNING_SPIRE_VICTORY_POINTS_REWARD,
        }
    }
}

#[derive(Resource, Default, Clone)]
pub struct LightningSpirePlacementClaims {
    pub claims: BTreeMap<Entity, BuildingAnchor>,
}

#[derive(Event, Clone, Copy)]
pub struct PlaceLightningSpireEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct SetLightningSpireGridConnectionEvent {
    pub entity: Entity,
    pub connected: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetLightningSpireSupplyStateEvent {
    pub entity: Entity,
    pub supplied_workers: bool,
    pub supplied_gold_maintenance: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetLightningSpireResearchTierEvent {
    pub entity: Entity,
    pub foundry_met: bool,
}

#[derive(Event, Clone, Copy)]
pub struct DemolishLightningSpireEvent {
    pub entity: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct AttackDestroyLightningSpireEvent {
    pub entity: Entity,
}

fn seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn place_lightning_spire_system(
    mut commands: Commands,
    mut events: EventReader<PlaceLightningSpireEvent>,
    mut claims: ResMut<LightningSpirePlacementClaims>,
) {
    for ev in events.read() {
        let anchor = BuildingAnchor {
            x: ev.tile_x,
            y: ev.tile_y,
        };

        let entity = commands
            .spawn((
                LightningSpire,
                anchor,
                LightningSpireCoreStats::default(),
                LightningSpireBuildState {
                    build_ticks_remaining: seconds_to_ticks(LIGHTNING_SPIRE_BUILD_TIME_SECONDS),
                    completed: false,
                },
                LightningSpireEconomy::default(),
                LightningSpireFootprint::default(),
                LightningSpireState::default(),
                LightningSpireEffects::default(),
            ))
            .id();

        claims.claims.insert(entity, anchor);
    }
}

fn set_lightning_spire_grid_connection_system(
    mut events: EventReader<SetLightningSpireGridConnectionEvent>,
    mut spires: Query<&mut LightningSpireState, With<LightningSpire>>,
) {
    for ev in events.read() {
        let Ok(mut state) = spires.get_mut(ev.entity) else {
            continue;
        };
        state.connected_to_grid = ev.connected;
    }
}

fn set_lightning_spire_supply_state_system(
    mut events: EventReader<SetLightningSpireSupplyStateEvent>,
    mut spires: Query<&mut LightningSpireState, With<LightningSpire>>,
) {
    for ev in events.read() {
        let Ok(mut state) = spires.get_mut(ev.entity) else {
            continue;
        };
        state.supplied_workers = ev.supplied_workers;
        state.supplied_gold_maintenance = ev.supplied_gold_maintenance;
    }
}

fn set_lightning_spire_research_tier_system(
    mut events: EventReader<SetLightningSpireResearchTierEvent>,
    mut spires: Query<&mut LightningSpireState, With<LightningSpire>>,
) {
    for ev in events.read() {
        let Ok(mut state) = spires.get_mut(ev.entity) else {
            continue;
        };
        state.research_tier_foundry_met = ev.foundry_met;
    }
}

fn lightning_spire_build_tick_system(mut spires: Query<&mut LightningSpireBuildState, With<LightningSpire>>) {
    for mut build in &mut spires {
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

fn attack_destroy_lightning_spire_system(
    mut events: EventReader<AttackDestroyLightningSpireEvent>,
    mut spires: Query<&mut LightningSpireState, With<LightningSpire>>,
) {
    for ev in events.read() {
        let Ok(mut state) = spires.get_mut(ev.entity) else {
            continue;
        };
        state.destroyed = true;
    }
}

fn destroy_lightning_spire_at_zero_health_system(
    mut spires: Query<(&LightningSpireCoreStats, &mut LightningSpireState), With<LightningSpire>>,
) {
    for (core, mut state) in &mut spires {
        if core.health.current <= I32F32::ZERO {
            state.destroyed = true;
        }
    }
}

fn demolish_lightning_spire_system(
    mut events: EventReader<DemolishLightningSpireEvent>,
    mut spires: Query<&mut LightningSpireState, With<LightningSpire>>,
) {
    for ev in events.read() {
        let Ok(mut state) = spires.get_mut(ev.entity) else {
            continue;
        };
        state.demolished = true;
    }
}

fn lightning_spire_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    spires: Query<
        (
            Entity,
            &BuildingAnchor,
            &LightningSpireCoreStats,
            &LightningSpireBuildState,
            &LightningSpireEconomy,
            &LightningSpireFootprint,
            &LightningSpireState,
            &LightningSpireEffects,
        ),
        With<LightningSpire>,
    >,
    claims: Res<LightningSpirePlacementClaims>,
) {
    for (entity, anchor, core, build, eco, footprint, state, effects) in &spires {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);

        checksum.accumulate(core.defenses_life.to_bits() as u64);
        checksum.accumulate(core.watch_range.to_bits() as u64);
        checksum.accumulate(core.health.current.to_bits() as u64);
        checksum.accumulate(core.health.max.to_bits() as u64);

        checksum.accumulate(build.build_ticks_remaining as u64);
        checksum.accumulate(u64::from(build.completed));

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
        checksum.accumulate(u64::from(state.supplied_workers));
        checksum.accumulate(u64::from(state.supplied_gold_maintenance));
        checksum.accumulate(u64::from(state.destroyed));
        checksum.accumulate(u64::from(state.demolished));
        checksum.accumulate(u64::from(state.research_tier_foundry_met));

        checksum.accumulate(effects.energy_output as u64);
        checksum.accumulate(effects.energy_transfer_radius.to_bits() as u64);
        checksum.accumulate(u64::from(effects.functions_as_large_tesla_tower));
        checksum.accumulate(effects.victory_points_reward as u64);
    }

    checksum.accumulate(claims.claims.len() as u64);
    for (entity, anchor) in &claims.claims {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);
    }
}

pub struct LightningSpirePlugin;

impl Plugin for LightningSpirePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LightningSpirePlacementClaims>()
            .add_event::<PlaceLightningSpireEvent>()
            .add_event::<SetLightningSpireGridConnectionEvent>()
            .add_event::<SetLightningSpireSupplyStateEvent>()
            .add_event::<SetLightningSpireResearchTierEvent>()
            .add_event::<DemolishLightningSpireEvent>()
            .add_event::<AttackDestroyLightningSpireEvent>()
            .add_systems(
                FixedUpdate,
                (
                    place_lightning_spire_system,
                    set_lightning_spire_grid_connection_system,
                    set_lightning_spire_supply_state_system,
                    set_lightning_spire_research_tier_system,
                    lightning_spire_build_tick_system,
                    attack_destroy_lightning_spire_system,
                    destroy_lightning_spire_at_zero_health_system,
                    demolish_lightning_spire_system,
                    lightning_spire_checksum_system,
                )
                    .chain(),
            );
    }
}
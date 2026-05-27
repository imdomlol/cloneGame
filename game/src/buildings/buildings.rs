// Sources: vault/buildings/buildings.md

use std::collections::BTreeMap;

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, SimChecksumState};

const BUILDINGS_HP: I32F32 = I32F32::lit("0");
const BUILDINGS_DEFENSES_LIFE: I32F32 = I32F32::lit("0");
const BUILDINGS_WATCH_RANGE: I32F32 = I32F32::lit("0");
const BUILDINGS_ENERGY_COST: i32 = 0;
const BUILDINGS_WOOD_COST: i32 = 0;
const BUILDINGS_STONE_COST: i32 = 0;
const BUILDINGS_IRON_COST: i32 = 0;
const BUILDINGS_OIL_COST: i32 = 0;
const BUILDINGS_GOLD_COST: i32 = 0;
const BUILDINGS_BUILD_TIME_SECONDS: i32 = 0;
const BUILDINGS_PFOOD: i32 = 0;
const BUILDINGS_PWOOD: i32 = 0;
const BUILDINGS_PSTONE: i32 = 0;
const BUILDINGS_PIRON: i32 = 0;
const BUILDINGS_POIL: i32 = 0;
const BUILDINGS_PGOLD: i32 = 0;
const BUILDINGS_PENERGY: i32 = 0;
const BUILDINGS_PCOLONISTS: i32 = 0;
const DEMOLISH_RESOURCE_REFUND_PERCENT: I32F32 = I32F32::lit("0.5");
const INFECTED_SPAWN_NOISE_PER_UNIT: i32 = 50;
const SUPPORT_AURA_FOOD_CONSUMPTION_MULTIPLIER: I32F32 = I32F32::lit("0.8");
const SUPPORT_AURA_PRODUCTION_MULTIPLIER: I32F32 = I32F32::lit("1.2");
const SUPPORT_AURA_GOLD_SMALL_MULTIPLIER: I32F32 = I32F32::lit("1.3");
const SUPPORT_AURA_GOLD_LARGE_MULTIPLIER: I32F32 = I32F32::lit("1.1");

#[derive(Component, Default)]
pub struct BuildingRulesIndex;

#[derive(Component, Clone, Copy)]
pub struct BuildingCoreStats {
    pub defenses_life: I32F32,
    pub watch_range: I32F32,
    pub health: Health,
}

impl Default for BuildingCoreStats {
    fn default() -> Self {
        Self {
            defenses_life: BUILDINGS_DEFENSES_LIFE,
            watch_range: BUILDINGS_WATCH_RANGE,
            health: Health::full(BUILDINGS_HP),
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct BuildingEconomy {
    pub energy_cost: i32,
    pub wood_cost: i32,
    pub stone_cost: i32,
    pub iron_cost: i32,
    pub oil_cost: i32,
    pub gold_cost: i32,
    pub build_time_seconds: i32,
    pub pfood: i32,
    pub pwood: i32,
    pub pstone: i32,
    pub piron: i32,
    pub poil: i32,
    pub pgold: i32,
    pub penergy: i32,
    pub pcolonists: i32,
}

impl Default for BuildingEconomy {
    fn default() -> Self {
        Self {
            energy_cost: BUILDINGS_ENERGY_COST,
            wood_cost: BUILDINGS_WOOD_COST,
            stone_cost: BUILDINGS_STONE_COST,
            iron_cost: BUILDINGS_IRON_COST,
            oil_cost: BUILDINGS_OIL_COST,
            gold_cost: BUILDINGS_GOLD_COST,
            build_time_seconds: BUILDINGS_BUILD_TIME_SECONDS,
            pfood: BUILDINGS_PFOOD,
            pwood: BUILDINGS_PWOOD,
            pstone: BUILDINGS_PSTONE,
            piron: BUILDINGS_PIRON,
            poil: BUILDINGS_POIL,
            pgold: BUILDINGS_PGOLD,
            penergy: BUILDINGS_PENERGY,
            pcolonists: BUILDINGS_PCOLONISTS,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct BuildingOccupancy {
    pub initial_workers: i32,
    pub initial_colonists: i32,
}

#[derive(Component, Clone, Copy, Default)]
pub struct BuildingState {
    pub destroyed: bool,
    pub infected: bool,
    pub disabled: bool,
    pub connected_to_grid: bool,
    pub paused: bool,
    pub tier_chain_blocked: bool,
}

#[derive(Component, Clone, Copy, Default)]
pub struct BuildingRefundLedger {
    pub refunded_gold: i32,
    pub refunded_wood: i32,
    pub refunded_stone: i32,
    pub refunded_iron: i32,
    pub refunded_oil: i32,
    pub refunded_energy: i32,
    pub refunded_workers: i32,
    pub refunded_food: i32,
}

#[derive(Component, Clone, Copy, Default)]
pub struct BuildingResearchState {
    pub current_research_id: i32,
    pub current_research_ticks: i32,
    pub can_start_new_research: bool,
}

#[derive(Component, Clone, Copy, Default)]
pub struct BuildingSupportAura {
    pub production_area_side: i32,
    pub food_consumption_area_side: i32,
    pub gold_large_area_side: i32,
    pub gold_small_area_side: i32,
    pub production_multiplier_bits: i64,
    pub food_consumption_multiplier_bits: i64,
    pub gold_large_multiplier_bits: i64,
    pub gold_small_multiplier_bits: i64,
}

impl BuildingSupportAura {
    pub fn index_defaults() -> Self {
        Self {
            production_area_side: 28,
            food_consumption_area_side: 27,
            gold_large_area_side: 52,
            gold_small_area_side: 27,
            production_multiplier_bits: SUPPORT_AURA_PRODUCTION_MULTIPLIER.to_bits(),
            food_consumption_multiplier_bits: SUPPORT_AURA_FOOD_CONSUMPTION_MULTIPLIER.to_bits(),
            gold_large_multiplier_bits: SUPPORT_AURA_GOLD_LARGE_MULTIPLIER.to_bits(),
            gold_small_multiplier_bits: SUPPORT_AURA_GOLD_SMALL_MULTIPLIER.to_bits(),
        }
    }
}

#[derive(Resource, Default, Clone)]
pub struct BuildingNoiseState {
    pub accumulated_noise: i64,
}

#[derive(Resource, Default, Clone)]
pub struct BuildingDemolishRefunds {
    pub by_entity: BTreeMap<Entity, BuildingRefundLedger>,
}

#[derive(Event, Clone, Copy)]
pub struct SetBuildingInfectedEvent {
    pub entity: Entity,
    pub infected: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetBuildingGridConnectionEvent {
    pub entity: Entity,
    pub connected: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetBuildingPausedEvent {
    pub entity: Entity,
    pub paused: bool,
}

#[derive(Event, Clone, Copy)]
pub struct DemolishBuildingEvent {
    pub entity: Entity,
}

fn apply_set_building_infected_system(
    mut events: EventReader<SetBuildingInfectedEvent>,
    mut buildings: Query<(&BuildingOccupancy, &mut BuildingState), With<BuildingRulesIndex>>,
    mut noise: ResMut<BuildingNoiseState>,
) {
    for ev in events.read() {
        let Ok((occupancy, mut state)) = buildings.get_mut(ev.entity) else {
            continue;
        };

        if ev.infected && !state.infected {
            let spawn_count = occupancy.initial_workers + occupancy.initial_colonists;
            noise.accumulated_noise += (spawn_count * INFECTED_SPAWN_NOISE_PER_UNIT) as i64;
        }

        state.infected = ev.infected;
        if state.infected {
            state.disabled = true;
        }
    }
}

fn apply_set_building_grid_connection_system(
    mut events: EventReader<SetBuildingGridConnectionEvent>,
    mut buildings: Query<(&mut BuildingState, &mut BuildingResearchState), With<BuildingRulesIndex>>,
) {
    for ev in events.read() {
        let Ok((mut state, mut research)) = buildings.get_mut(ev.entity) else {
            continue;
        };

        state.connected_to_grid = ev.connected;
        if state.connected_to_grid {
            research.can_start_new_research = true;
        } else {
            research.can_start_new_research = false;
        }
    }
}

fn apply_set_building_paused_system(
    mut events: EventReader<SetBuildingPausedEvent>,
    mut buildings: Query<(&mut BuildingState, &mut BuildingResearchState), With<BuildingRulesIndex>>,
) {
    for ev in events.read() {
        let Ok((mut state, mut research)) = buildings.get_mut(ev.entity) else {
            continue;
        };

        state.paused = ev.paused;
        if state.paused {
            research.current_research_ticks = 0;
        }
    }
}

fn destroy_building_at_zero_health_system(
    mut buildings: Query<
        (&BuildingCoreStats, &mut BuildingState, &mut BuildingResearchState),
        With<BuildingRulesIndex>,
    >,
) {
    for (core, mut state, mut research) in &mut buildings {
        if core.health.current <= I32F32::ZERO {
            state.destroyed = true;
            state.disabled = true;
            research.current_research_id = 0;
            research.current_research_ticks = 0;
            research.can_start_new_research = false;
        }
    }
}

fn demolish_building_system(
    mut events: EventReader<DemolishBuildingEvent>,
    buildings: Query<(&BuildingEconomy, &BuildingState), With<BuildingRulesIndex>>,
    mut refunds: ResMut<BuildingDemolishRefunds>,
) {
    for ev in events.read() {
        let Ok((eco, state)) = buildings.get(ev.entity) else {
            continue;
        };

        if state.destroyed {
            continue;
        }

        let refunded_gold =
            (I32F32::from_num(eco.gold_cost) * DEMOLISH_RESOURCE_REFUND_PERCENT).to_num::<i32>();
        let refunded_wood =
            (I32F32::from_num(eco.wood_cost) * DEMOLISH_RESOURCE_REFUND_PERCENT).to_num::<i32>();
        let refunded_stone =
            (I32F32::from_num(eco.stone_cost) * DEMOLISH_RESOURCE_REFUND_PERCENT).to_num::<i32>();
        let refunded_iron =
            (I32F32::from_num(eco.iron_cost) * DEMOLISH_RESOURCE_REFUND_PERCENT).to_num::<i32>();
        let refunded_oil =
            (I32F32::from_num(eco.oil_cost) * DEMOLISH_RESOURCE_REFUND_PERCENT).to_num::<i32>();

        refunds.by_entity.insert(
            ev.entity,
            BuildingRefundLedger {
                refunded_gold,
                refunded_wood,
                refunded_stone,
                refunded_iron,
                refunded_oil,
                refunded_energy: eco.penergy,
                refunded_workers: 0,
                refunded_food: eco.pfood,
            },
        );
    }
}

fn buildings_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    buildings: Query<
        (
            Entity,
            &BuildingCoreStats,
            &BuildingEconomy,
            &BuildingOccupancy,
            &BuildingState,
            &BuildingRefundLedger,
            &BuildingResearchState,
            &BuildingSupportAura,
        ),
        With<BuildingRulesIndex>,
    >,
    noise: Res<BuildingNoiseState>,
    refunds: Res<BuildingDemolishRefunds>,
) {
    for (entity, core, eco, occupancy, state, ledger, research, aura) in &buildings {
        checksum.accumulate(entity.to_bits() as u64);

        checksum.accumulate(core.defenses_life.to_bits() as u64);
        checksum.accumulate(core.watch_range.to_bits() as u64);
        checksum.accumulate(core.health.current.to_bits() as u64);
        checksum.accumulate(core.health.max.to_bits() as u64);

        checksum.accumulate(eco.energy_cost as u64);
        checksum.accumulate(eco.wood_cost as u64);
        checksum.accumulate(eco.stone_cost as u64);
        checksum.accumulate(eco.iron_cost as u64);
        checksum.accumulate(eco.oil_cost as u64);
        checksum.accumulate(eco.gold_cost as u64);
        checksum.accumulate(eco.build_time_seconds as u64);
        checksum.accumulate(eco.pfood as u64);
        checksum.accumulate(eco.pwood as u64);
        checksum.accumulate(eco.pstone as u64);
        checksum.accumulate(eco.piron as u64);
        checksum.accumulate(eco.poil as u64);
        checksum.accumulate(eco.pgold as u64);
        checksum.accumulate(eco.penergy as u64);
        checksum.accumulate(eco.pcolonists as u64);

        checksum.accumulate(occupancy.initial_workers as u64);
        checksum.accumulate(occupancy.initial_colonists as u64);

        checksum.accumulate(u64::from(state.destroyed));
        checksum.accumulate(u64::from(state.infected));
        checksum.accumulate(u64::from(state.disabled));
        checksum.accumulate(u64::from(state.connected_to_grid));
        checksum.accumulate(u64::from(state.paused));
        checksum.accumulate(u64::from(state.tier_chain_blocked));

        checksum.accumulate(ledger.refunded_gold as u64);
        checksum.accumulate(ledger.refunded_wood as u64);
        checksum.accumulate(ledger.refunded_stone as u64);
        checksum.accumulate(ledger.refunded_iron as u64);
        checksum.accumulate(ledger.refunded_oil as u64);
        checksum.accumulate(ledger.refunded_energy as u64);
        checksum.accumulate(ledger.refunded_workers as u64);
        checksum.accumulate(ledger.refunded_food as u64);

        checksum.accumulate(research.current_research_id as u64);
        checksum.accumulate(research.current_research_ticks as u64);
        checksum.accumulate(u64::from(research.can_start_new_research));

        checksum.accumulate(aura.production_area_side as u64);
        checksum.accumulate(aura.food_consumption_area_side as u64);
        checksum.accumulate(aura.gold_large_area_side as u64);
        checksum.accumulate(aura.gold_small_area_side as u64);
        checksum.accumulate(aura.production_multiplier_bits as u64);
        checksum.accumulate(aura.food_consumption_multiplier_bits as u64);
        checksum.accumulate(aura.gold_large_multiplier_bits as u64);
        checksum.accumulate(aura.gold_small_multiplier_bits as u64);
    }

    checksum.accumulate(noise.accumulated_noise as u64);

    checksum.accumulate(refunds.by_entity.len() as u64);
    for (entity, ledger) in &refunds.by_entity {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(ledger.refunded_gold as u64);
        checksum.accumulate(ledger.refunded_wood as u64);
        checksum.accumulate(ledger.refunded_stone as u64);
        checksum.accumulate(ledger.refunded_iron as u64);
        checksum.accumulate(ledger.refunded_oil as u64);
        checksum.accumulate(ledger.refunded_energy as u64);
        checksum.accumulate(ledger.refunded_workers as u64);
        checksum.accumulate(ledger.refunded_food as u64);
    }
}

pub struct BuildingsPlugin;

impl Plugin for BuildingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BuildingNoiseState>()
            .init_resource::<BuildingDemolishRefunds>()
            .add_event::<SetBuildingInfectedEvent>()
            .add_event::<SetBuildingGridConnectionEvent>()
            .add_event::<SetBuildingPausedEvent>()
            .add_event::<DemolishBuildingEvent>()
            .add_systems(
                FixedUpdate,
                (
                    apply_set_building_infected_system,
                    apply_set_building_grid_connection_system,
                    apply_set_building_paused_system,
                    destroy_building_at_zero_health_system,
                    demolish_building_system,
                    buildings_checksum_system,
                )
                    .chain(),
            );
    }
}
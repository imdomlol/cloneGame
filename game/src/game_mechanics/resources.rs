// Sources: vault/game_mechanics/resources.md

use bevy::prelude::*;
use fixed::types::I32F32;
use rand_core::RngCore;

use crate::sim::{tick_rng, GameSeed, SimChecksumState, SimTick};

const COMMAND_CENTER_START_GOLD_INCOME: I32F32 = I32F32::lit("200");
const NODE_TILE_INCOME: I32F32 = I32F32::lit("0.5");
const UNSUPPLIED_GOLD_MULTIPLIER: I32F32 = I32F32::lit("2.0");
const WAREHOUSE_RADIUS: I32F32 = I32F32::lit("12");
const WAREHOUSE_BONUS: I32F32 = I32F32::lit("0.2");
const BANK_RADIUS: I32F32 = I32F32::lit("12");
const BANK_BONUS: I32F32 = I32F32::lit("0.3");
const SMALL_CACHE_GOLD: I32F32 = I32F32::lit("500");
const SMALL_CACHE_OTHER: I32F32 = I32F32::lit("10");
const VILLAGE_DROP_GOLD: I32F32 = I32F32::lit("1000");
const VILLAGE_DROP_OTHER: I32F32 = I32F32::lit("20");
const POWER_PLANT_WOOD_PENALTY: I32F32 = I32F32::lit("10");

const SMALL_CACHE_SALT: u64 = 0x8A17_5F03_4E2D_AA11;
const VILLAGE_DROP_SALT: u64 = 0x0C91_D5B3_27A4_6139;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceType {
    Gold,
    Wood,
    Stone,
    Iron,
    Oil,
}

#[derive(Resource, Clone, Copy)]
pub struct ResourcesMechanicData {
    pub id: &'static str,
    pub name: &'static str,
    pub mechanic_type: &'static str,
    pub how_it_works: &'static str,
    pub techniques: &'static str,
    pub applications: &'static str,
    pub depends_on: &'static [&'static str],
}

impl Default for ResourcesMechanicData {
    fn default() -> Self {
        Self {
            id: "resources",
            name: "Resources",
            mechanic_type: "resource_system",
            how_it_works: "The economy is split into stockpileable resources and supply resources. Stockpileables such as gold, wood, stone, iron, and oil are spent permanently when they fund a unit or building. Supply resources such as workers, food, and energy are capacity constraints that are occupied rather than consumed, and they can be freed again when the relevant building is destroyed or switched off. Production comes from buildings such as tent, cottage, mill, quarry, oil_platform, warehouse, and bank, while map stashes and villages_of_doom can provide one-time bursts.",
            techniques: "When using resource collectors, place them so their highlighted ranges cover the highest number of tiles inside the node. With identical collectors, keep the ranges from touching by even 1 tile if a wider spread captures more of an uneven deposit. Prioritize warehouse placement for non-gold production and bank placement for gold production.",
            applications: "The resource system gates expansion, unit production, upkeep, and late-game scaling. Food and oil shortages double the gold maintenance cost of unsupplied units, energy shortages shut down attack towers such as great_ballista, and late-game power can be compared across sources such as lightning_spire, power_plant, mill, and advanced_mill.",
            depends_on: &[
                "advanced_farm",
                "advanced_mill",
                "advanced_quarry",
                "bank",
                "command_center",
                "cottage",
                "engineering_center",
                "farm",
                "fisherman_cottage",
                "foundry",
                "great_ballista",
                "hunters_cottage",
                "lightning_spire",
                "lucifer",
                "market",
                "mill",
                "oil_platform",
                "power_plant",
                "quarry",
                "ranger",
                "sawmill",
                "sniper",
                "soldier",
                "soldiers_center",
                "stone_house",
                "stone_wall",
                "stone_workshop",
                "tent",
                "thanatos",
                "titan",
                "villages_of_doom",
                "warehouse",
                "wood_wall",
                "wood_workshop",
            ],
        }
    }
}

#[derive(Resource, Clone, Copy)]
pub struct ColonyResourcesState {
    pub gold: I32F32,
    pub wood: I32F32,
    pub stone: I32F32,
    pub iron: I32F32,
    pub oil: I32F32,
    pub food_capacity: I32F32,
    pub food_occupied: I32F32,
    pub energy_capacity: I32F32,
    pub energy_occupied: I32F32,
    pub worker_capacity: I32F32,
    pub worker_occupied: I32F32,
    pub gold_income_base: I32F32,
    pub unsupplied_gold_multiplier: I32F32,
    pub food_shortage_penalty_active: bool,
    pub oil_shortage_penalty_active: bool,
    pub energy_shortage_shutdown_active: bool,
    pub worker_shortage_blocks_actions: bool,
    pub unit_desertion_active: bool,
}

impl Default for ColonyResourcesState {
    fn default() -> Self {
        Self {
            gold: I32F32::ZERO,
            wood: I32F32::ZERO,
            stone: I32F32::ZERO,
            iron: I32F32::ZERO,
            oil: I32F32::ZERO,
            food_capacity: I32F32::ZERO,
            food_occupied: I32F32::ZERO,
            energy_capacity: I32F32::ZERO,
            energy_occupied: I32F32::ZERO,
            worker_capacity: I32F32::ZERO,
            worker_occupied: I32F32::ZERO,
            gold_income_base: COMMAND_CENTER_START_GOLD_INCOME,
            unsupplied_gold_multiplier: I32F32::ONE,
            food_shortage_penalty_active: false,
            oil_shortage_penalty_active: false,
            energy_shortage_shutdown_active: false,
            worker_shortage_blocks_actions: false,
            unit_desertion_active: false,
        }
    }
}

#[derive(Resource, Clone, Copy, Default)]
pub struct ResourcesMetrics {
    pub small_caches_collected: u64,
    pub villages_of_doom_drops_collected: u64,
    pub total_stockpile_awarded: I32F32,
    pub resource_node_income_total: I32F32,
    pub boosted_non_gold_nodes: u64,
    pub boosted_gold_nodes: u64,
    pub overlap_blocked_collectors: u64,
}

#[derive(Component, Clone, Copy, Default)]
pub struct ResourceCollector {
    pub covered_tiles: u32,
    pub overlaps_same_collector_range: bool,
    pub gathered_income: I32F32,
}

#[derive(Component, Clone, Copy, Default)]
pub struct ResourceNodeProduction {
    pub resource: Option<ResourceType>,
    pub base_income: I32F32,
    pub boosted_income: I32F32,
    pub in_warehouse_radius: bool,
    pub in_bank_radius: bool,
    pub has_power_plant_wood_penalty: bool,
}

#[derive(Component, Clone, Copy, Default)]
pub struct UnsuppliedUnit {
    pub affected_by_food_or_oil_shortage: bool,
    pub effective_gold_maintenance_multiplier: I32F32,
}

#[derive(Component, Clone, Copy, Default)]
pub struct AttackTowerPowerState {
    pub enabled: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SmallCacheCollectedEvent {
    pub collector_stable_id: u64,
}

#[derive(Event, Clone, Copy)]
pub struct VillageOfDoomDropCollectedEvent {
    pub collector_stable_id: u64,
}

#[derive(Event, Clone, Copy)]
pub struct ResourceDropAwardedEvent {
    pub gold: I32F32,
    pub wood: I32F32,
    pub stone: I32F32,
    pub iron: I32F32,
    pub oil: I32F32,
}

fn roll_resource_drop(game_seed: u64, tick: u64, salt: u64, id: u64) -> u32 {
    let mut rng = tick_rng(game_seed, tick, salt ^ id);
    rng.next_u32() % 5
}

fn apply_supply_shortage_rules_system(
    mut state: ResMut<ColonyResourcesState>,
    mut units: Query<&mut UnsuppliedUnit>,
    mut towers: Query<&mut AttackTowerPowerState>,
) {
    let food_supply = state.food_capacity - state.food_occupied;
    let energy_supply = state.energy_capacity - state.energy_occupied;
    let worker_supply = state.worker_capacity - state.worker_occupied;

    state.food_shortage_penalty_active = food_supply < I32F32::ZERO;
    state.oil_shortage_penalty_active = state.oil < I32F32::ZERO;
    state.energy_shortage_shutdown_active = energy_supply < I32F32::ZERO;
    state.worker_shortage_blocks_actions = worker_supply < I32F32::ZERO;
    state.unit_desertion_active = state.gold <= I32F32::ZERO;

    let unsupplied_multiplier = if state.food_shortage_penalty_active || state.oil_shortage_penalty_active {
        UNSUPPLIED_GOLD_MULTIPLIER
    } else {
        I32F32::ONE
    };
    state.unsupplied_gold_multiplier = unsupplied_multiplier;

    for mut unit in &mut units {
        unit.affected_by_food_or_oil_shortage =
            state.food_shortage_penalty_active || state.oil_shortage_penalty_active;
        unit.effective_gold_maintenance_multiplier = if unit.affected_by_food_or_oil_shortage {
            UNSUPPLIED_GOLD_MULTIPLIER
        } else {
            I32F32::ONE
        };
    }

    for mut tower in &mut towers {
        tower.enabled = !state.energy_shortage_shutdown_active;
    }
}

fn apply_resource_collector_rules_system(
    mut metrics: ResMut<ResourcesMetrics>,
    mut collectors: Query<&mut ResourceCollector>,
) {
    let mut income_total = I32F32::ZERO;
    let mut blocked = 0_u64;

    for mut collector in &mut collectors {
        if collector.overlaps_same_collector_range {
            collector.gathered_income = I32F32::ZERO;
            blocked = blocked.saturating_add(1);
            continue;
        }

        let gathered_integer = (collector.covered_tiles.saturating_add(1)) / 2;
        collector.gathered_income = I32F32::from_num(gathered_integer);
        income_total = income_total + collector.gathered_income;
    }

    metrics.resource_node_income_total = income_total;
    metrics.overlap_blocked_collectors = blocked;
}

fn apply_production_aura_rules_system(
    mut metrics: ResMut<ResourcesMetrics>,
    mut nodes: Query<&mut ResourceNodeProduction>,
) {
    let mut boosted_non_gold = 0_u64;
    let mut boosted_gold = 0_u64;

    for mut node in &mut nodes {
        let mut income = node.base_income;

        if node.resource == Some(ResourceType::Wood) && node.has_power_plant_wood_penalty {
            income = income - POWER_PLANT_WOOD_PENALTY;
        }

        if node.resource == Some(ResourceType::Gold) && node.in_bank_radius {
            income = income + (income * BANK_BONUS);
            boosted_gold = boosted_gold.saturating_add(1);
        } else if node.resource != Some(ResourceType::Gold) && node.in_warehouse_radius {
            income = income + (income * WAREHOUSE_BONUS);
            boosted_non_gold = boosted_non_gold.saturating_add(1);
        }

        node.boosted_income = income;
    }

    metrics.boosted_non_gold_nodes = boosted_non_gold;
    metrics.boosted_gold_nodes = boosted_gold;
}

fn apply_small_cache_rewards_system(
    game_seed: Res<GameSeed>,
    tick: Res<SimTick>,
    mut state: ResMut<ColonyResourcesState>,
    mut metrics: ResMut<ResourcesMetrics>,
    mut events: EventReader<SmallCacheCollectedEvent>,
    mut awarded: EventWriter<ResourceDropAwardedEvent>,
) {
    for ev in events.read() {
        let reward_kind = roll_resource_drop(game_seed.0, tick.0, SMALL_CACHE_SALT, ev.collector_stable_id);

        let mut drop = ResourceDropAwardedEvent {
            gold: I32F32::ZERO,
            wood: I32F32::ZERO,
            stone: I32F32::ZERO,
            iron: I32F32::ZERO,
            oil: I32F32::ZERO,
        };

        if reward_kind == 0 {
            drop.gold = SMALL_CACHE_GOLD;
            state.gold = state.gold + SMALL_CACHE_GOLD;
            metrics.total_stockpile_awarded = metrics.total_stockpile_awarded + SMALL_CACHE_GOLD;
        } else if reward_kind == 1 {
            drop.wood = SMALL_CACHE_OTHER;
            state.wood = state.wood + SMALL_CACHE_OTHER;
            metrics.total_stockpile_awarded = metrics.total_stockpile_awarded + SMALL_CACHE_OTHER;
        } else if reward_kind == 2 {
            drop.stone = SMALL_CACHE_OTHER;
            state.stone = state.stone + SMALL_CACHE_OTHER;
            metrics.total_stockpile_awarded = metrics.total_stockpile_awarded + SMALL_CACHE_OTHER;
        } else if reward_kind == 3 {
            drop.iron = SMALL_CACHE_OTHER;
            state.iron = state.iron + SMALL_CACHE_OTHER;
            metrics.total_stockpile_awarded = metrics.total_stockpile_awarded + SMALL_CACHE_OTHER;
        } else {
            drop.oil = SMALL_CACHE_OTHER;
            state.oil = state.oil + SMALL_CACHE_OTHER;
            metrics.total_stockpile_awarded = metrics.total_stockpile_awarded + SMALL_CACHE_OTHER;
        }

        metrics.small_caches_collected = metrics.small_caches_collected.saturating_add(1);
        awarded.send(drop);
    }
}

fn apply_village_of_doom_drop_rewards_system(
    game_seed: Res<GameSeed>,
    tick: Res<SimTick>,
    mut state: ResMut<ColonyResourcesState>,
    mut metrics: ResMut<ResourcesMetrics>,
    mut events: EventReader<VillageOfDoomDropCollectedEvent>,
    mut awarded: EventWriter<ResourceDropAwardedEvent>,
) {
    for ev in events.read() {
        let reward_kind =
            roll_resource_drop(game_seed.0, tick.0, VILLAGE_DROP_SALT, ev.collector_stable_id);

        let mut drop = ResourceDropAwardedEvent {
            gold: I32F32::ZERO,
            wood: I32F32::ZERO,
            stone: I32F32::ZERO,
            iron: I32F32::ZERO,
            oil: I32F32::ZERO,
        };

        if reward_kind == 0 {
            drop.gold = VILLAGE_DROP_GOLD;
            state.gold = state.gold + VILLAGE_DROP_GOLD;
            metrics.total_stockpile_awarded = metrics.total_stockpile_awarded + VILLAGE_DROP_GOLD;
        } else if reward_kind == 1 {
            drop.wood = VILLAGE_DROP_OTHER;
            state.wood = state.wood + VILLAGE_DROP_OTHER;
            metrics.total_stockpile_awarded = metrics.total_stockpile_awarded + VILLAGE_DROP_OTHER;
        } else if reward_kind == 2 {
            drop.stone = VILLAGE_DROP_OTHER;
            state.stone = state.stone + VILLAGE_DROP_OTHER;
            metrics.total_stockpile_awarded = metrics.total_stockpile_awarded + VILLAGE_DROP_OTHER;
        } else if reward_kind == 3 {
            drop.iron = VILLAGE_DROP_OTHER;
            state.iron = state.iron + VILLAGE_DROP_OTHER;
            metrics.total_stockpile_awarded = metrics.total_stockpile_awarded + VILLAGE_DROP_OTHER;
        } else {
            drop.oil = VILLAGE_DROP_OTHER;
            state.oil = state.oil + VILLAGE_DROP_OTHER;
            metrics.total_stockpile_awarded = metrics.total_stockpile_awarded + VILLAGE_DROP_OTHER;
        }

        metrics.villages_of_doom_drops_collected =
            metrics.villages_of_doom_drops_collected.saturating_add(1);
        awarded.send(drop);
    }
}

fn resources_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    state: Res<ColonyResourcesState>,
    metrics: Res<ResourcesMetrics>,
    collectors: Query<&ResourceCollector>,
    nodes: Query<&ResourceNodeProduction>,
    units: Query<&UnsuppliedUnit>,
    towers: Query<&AttackTowerPowerState>,
) {
    checksum.accumulate(COMMAND_CENTER_START_GOLD_INCOME.to_bits() as u64);
    checksum.accumulate(NODE_TILE_INCOME.to_bits() as u64);
    checksum.accumulate(UNSUPPLIED_GOLD_MULTIPLIER.to_bits() as u64);
    checksum.accumulate(WAREHOUSE_RADIUS.to_bits() as u64);
    checksum.accumulate(WAREHOUSE_BONUS.to_bits() as u64);
    checksum.accumulate(BANK_RADIUS.to_bits() as u64);
    checksum.accumulate(BANK_BONUS.to_bits() as u64);
    checksum.accumulate(SMALL_CACHE_GOLD.to_bits() as u64);
    checksum.accumulate(SMALL_CACHE_OTHER.to_bits() as u64);
    checksum.accumulate(VILLAGE_DROP_GOLD.to_bits() as u64);
    checksum.accumulate(VILLAGE_DROP_OTHER.to_bits() as u64);
    checksum.accumulate(POWER_PLANT_WOOD_PENALTY.to_bits() as u64);

    checksum.accumulate(state.gold.to_bits() as u64);
    checksum.accumulate(state.wood.to_bits() as u64);
    checksum.accumulate(state.stone.to_bits() as u64);
    checksum.accumulate(state.iron.to_bits() as u64);
    checksum.accumulate(state.oil.to_bits() as u64);
    checksum.accumulate(state.food_capacity.to_bits() as u64);
    checksum.accumulate(state.food_occupied.to_bits() as u64);
    checksum.accumulate(state.energy_capacity.to_bits() as u64);
    checksum.accumulate(state.energy_occupied.to_bits() as u64);
    checksum.accumulate(state.worker_capacity.to_bits() as u64);
    checksum.accumulate(state.worker_occupied.to_bits() as u64);
    checksum.accumulate(state.gold_income_base.to_bits() as u64);
    checksum.accumulate(state.unsupplied_gold_multiplier.to_bits() as u64);
    checksum.accumulate(u64::from(state.food_shortage_penalty_active));
    checksum.accumulate(u64::from(state.oil_shortage_penalty_active));
    checksum.accumulate(u64::from(state.energy_shortage_shutdown_active));
    checksum.accumulate(u64::from(state.worker_shortage_blocks_actions));
    checksum.accumulate(u64::from(state.unit_desertion_active));

    checksum.accumulate(metrics.small_caches_collected);
    checksum.accumulate(metrics.villages_of_doom_drops_collected);
    checksum.accumulate(metrics.total_stockpile_awarded.to_bits() as u64);
    checksum.accumulate(metrics.resource_node_income_total.to_bits() as u64);
    checksum.accumulate(metrics.boosted_non_gold_nodes);
    checksum.accumulate(metrics.boosted_gold_nodes);
    checksum.accumulate(metrics.overlap_blocked_collectors);

    for collector in &collectors {
        checksum.accumulate(collector.covered_tiles as u64);
        checksum.accumulate(u64::from(collector.overlaps_same_collector_range));
        checksum.accumulate(collector.gathered_income.to_bits() as u64);
    }

    for node in &nodes {
        let resource_bits = match node.resource {
            Some(ResourceType::Gold) => 1_u64,
            Some(ResourceType::Wood) => 2_u64,
            Some(ResourceType::Stone) => 3_u64,
            Some(ResourceType::Iron) => 4_u64,
            Some(ResourceType::Oil) => 5_u64,
            None => 0_u64,
        };
        checksum.accumulate(resource_bits);
        checksum.accumulate(node.base_income.to_bits() as u64);
        checksum.accumulate(node.boosted_income.to_bits() as u64);
        checksum.accumulate(u64::from(node.in_warehouse_radius));
        checksum.accumulate(u64::from(node.in_bank_radius));
        checksum.accumulate(u64::from(node.has_power_plant_wood_penalty));
    }

    for unit in &units {
        checksum.accumulate(u64::from(unit.affected_by_food_or_oil_shortage));
        checksum.accumulate(unit.effective_gold_maintenance_multiplier.to_bits() as u64);
    }

    for tower in &towers {
        checksum.accumulate(u64::from(tower.enabled));
    }
}

pub struct ResourcesPlugin;

impl Plugin for ResourcesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ResourcesMechanicData>()
            .init_resource::<ColonyResourcesState>()
            .init_resource::<ResourcesMetrics>()
            .add_event::<SmallCacheCollectedEvent>()
            .add_event::<VillageOfDoomDropCollectedEvent>()
            .add_event::<ResourceDropAwardedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    apply_supply_shortage_rules_system,
                    apply_resource_collector_rules_system,
                    apply_production_aura_rules_system,
                    apply_small_cache_rewards_system,
                    apply_village_of_doom_drop_rewards_system,
                    resources_checksum_system,
                )
                    .chain(),
            );
    }
}
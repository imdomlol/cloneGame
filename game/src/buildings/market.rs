// Sources: vault/buildings/market.md, vault/buildings/bank.md

use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, SimChecksumState};

const MARKET_HP: I32F32 = I32F32::lit("500");
const MARKET_DEFENSES_LIFE: I32F32 = I32F32::lit("125");
const MARKET_WATCH_RANGE: I32F32 = I32F32::lit("7");
const MARKET_ENERGY_COST: i32 = 8;
const MARKET_WOOD_COST: i32 = 30;
const MARKET_STONE_COST: i32 = 40;
const MARKET_IRON_COST: i32 = 0;
const MARKET_OIL_COST: i32 = 0;
const MARKET_GOLD_COST: i32 = 400;
const MARKET_BUILD_TIME_SECONDS: i32 = 60;
const MARKET_SIZE_TILES: i32 = 3;
const MARKET_WORKERS: i32 = 8;
const MARKET_AURA_RADIUS_TILES: i32 = 12;
const MARKET_AOE_WIDTH_TILES: i32 = 27;
const MARKET_AOE_HEIGHT_TILES: i32 = 27;
const MARKET_MAX_COUNT: usize = 3;
const MARKET_POPULATION_UNLOCK_1: i32 = 200;
const MARKET_POPULATION_UNLOCK_2: i32 = 400;
const MARKET_POPULATION_UNLOCK_3: i32 = 600;
const FOOD_REDUCTION_DEFAULT_PERCENT: I32F32 = I32F32::lit("0.20");
const FOOD_REDUCTION_TENT_PERCENT: I32F32 = I32F32::lit("0.25");
const FOOD_REDUCTION_COTTAGE_PERCENT: I32F32 = I32F32::lit("0.25");
const FOOD_REDUCTION_STONE_HOUSE_PERCENT: I32F32 = I32F32::lit("0.20");
const TRADE_BASE_BATCH: i32 = 5;
const SIM_HZ: i32 = 25;

#[derive(Component, Default)]
pub struct Market;

#[derive(Component, Clone, Copy, Default)]
pub struct BuildingAnchor {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Copy)]
pub struct MarketCoreStats {
    pub defenses_life: I32F32,
    pub watch_range: I32F32,
    pub health: Health,
}

impl Default for MarketCoreStats {
    fn default() -> Self {
        Self {
            defenses_life: MARKET_DEFENSES_LIFE,
            watch_range: MARKET_WATCH_RANGE,
            health: Health::full(MARKET_HP),
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct MarketBuildState {
    pub build_ticks_remaining: i32,
    pub completed: bool,
}

#[derive(Component, Clone, Copy)]
pub struct MarketEconomy {
    pub energy_cost: i32,
    pub wood_cost: i32,
    pub stone_cost: i32,
    pub iron_cost: i32,
    pub oil_cost: i32,
    pub gold_cost: i32,
    pub workers: i32,
}

impl Default for MarketEconomy {
    fn default() -> Self {
        Self {
            energy_cost: MARKET_ENERGY_COST,
            wood_cost: MARKET_WOOD_COST,
            stone_cost: MARKET_STONE_COST,
            iron_cost: MARKET_IRON_COST,
            oil_cost: MARKET_OIL_COST,
            gold_cost: MARKET_GOLD_COST,
            workers: MARKET_WORKERS,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct MarketFootprint {
    pub size_tiles: i32,
    pub watch_range: I32F32,
    pub aoe_width_tiles: i32,
    pub aoe_height_tiles: i32,
}

impl Default for MarketFootprint {
    fn default() -> Self {
        Self {
            size_tiles: MARKET_SIZE_TILES,
            watch_range: MARKET_WATCH_RANGE,
            aoe_width_tiles: MARKET_AOE_WIDTH_TILES,
            aoe_height_tiles: MARKET_AOE_HEIGHT_TILES,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct MarketAura {
    pub radius_tiles: i32,
}

impl Default for MarketAura {
    fn default() -> Self {
        Self {
            radius_tiles: MARKET_AURA_RADIUS_TILES,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct MarketPowerState {
    pub connected_to_grid: bool,
}

#[derive(Component, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum DwellingKind {
    Tent,
    Cottage,
    StoneHouse,
    #[default]
    Other,
}

#[derive(Component, Clone, Copy, Default)]
pub struct DwellingBuilding {
    pub base_food_cost: I32F32,
    pub kind: DwellingKind,
}

#[derive(Component, Clone, Copy, Default)]
pub struct DwellingFoodBonus {
    pub market_entity: Option<Entity>,
    pub reduction_percent: I32F32,
    pub effective_food_cost: I32F32,
}

#[derive(Resource, Default, Clone, Copy)]
pub struct ColonyPopulation {
    pub total: i32,
}

#[derive(Resource, Default, Clone)]
pub struct TileOccupancy {
    pub blocked_tiles: BTreeSet<(i32, i32)>,
}

#[derive(Resource, Default, Clone)]
pub struct MarketPlacementClaims {
    pub claims: BTreeMap<Entity, BuildingAnchor>,
}

#[derive(Resource, Clone, Copy)]
pub struct MarketTradeTier {
    pub tier_count: i32,
    pub buy_wood: i32,
    pub buy_stone: i32,
    pub buy_iron: i32,
    pub buy_oil: i32,
    pub sell_wood: i32,
    pub sell_stone: i32,
    pub sell_iron: i32,
    pub sell_oil: i32,
    pub click_trade_amount: i32,
}

impl Default for MarketTradeTier {
    fn default() -> Self {
        Self {
            tier_count: 1,
            buy_wood: 100,
            buy_stone: 200,
            buy_iron: 400,
            buy_oil: 800,
            sell_wood: 5,
            sell_stone: 10,
            sell_iron: 20,
            sell_oil: 40,
            click_trade_amount: TRADE_BASE_BATCH,
        }
    }
}

#[derive(Resource, Default, Clone)]
pub struct MarketSelectionState {
    pub selected_markets: BTreeSet<Entity>,
}

#[derive(Event, Clone, Copy)]
pub struct SetTileBlockedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
    pub blocked: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetColonyPopulationEvent {
    pub total: i32,
}

#[derive(Event, Clone, Copy)]
pub struct SetMarketPowerEvent {
    pub market_entity: Entity,
    pub connected_to_grid: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetMarketSelectedEvent {
    pub market_entity: Entity,
    pub selected: bool,
}

#[derive(Event, Clone, Copy)]
pub struct PlaceMarketEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct MarketPlacementRejectedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

fn build_seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn footprint_tiles(anchor: BuildingAnchor) -> impl Iterator<Item = (i32, i32)> {
    let min_x = anchor.x;
    let min_y = anchor.y;
    let max_x = anchor.x + MARKET_SIZE_TILES - 1;
    let max_y = anchor.y + MARKET_SIZE_TILES - 1;
    (min_x..=max_x).flat_map(move |x| (min_y..=max_y).map(move |y| (x, y)))
}

fn market_center(anchor: BuildingAnchor) -> (i32, i32) {
    (anchor.x + 1, anchor.y + 1)
}

fn max_axis_delta(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0).abs().max((a.1 - b.1).abs())
}

fn dwelling_fully_inside_market_aura(
    dwelling_anchor: BuildingAnchor,
    market_anchor: BuildingAnchor,
    radius_tiles: i32,
) -> bool {
    let (cx, cy) = market_center(market_anchor);
    let min_x = dwelling_anchor.x;
    let min_y = dwelling_anchor.y;
    let max_x = dwelling_anchor.x + MARKET_SIZE_TILES - 1;
    let max_y = dwelling_anchor.y + MARKET_SIZE_TILES - 1;

    let corners = [(min_x, min_y), (min_x, max_y), (max_x, min_y), (max_x, max_y)];

    for corner in corners {
        if max_axis_delta(corner, (cx, cy)) > radius_tiles {
            return false;
        }
    }

    true
}

fn max_markets_for_population(population: i32) -> usize {
    let mut max_count = 0usize;

    if population >= MARKET_POPULATION_UNLOCK_1 {
        max_count += 1;
    }
    if population >= MARKET_POPULATION_UNLOCK_2 {
        max_count += 1;
    }
    if population >= MARKET_POPULATION_UNLOCK_3 {
        max_count += 1;
    }

    max_count.min(MARKET_MAX_COUNT)
}

fn footprint_is_unblocked(anchor: BuildingAnchor, occupancy: &TileOccupancy) -> bool {
    for tile in footprint_tiles(anchor) {
        if occupancy.blocked_tiles.contains(&tile) {
            return false;
        }
    }
    true
}

fn market_tier_from_count(count: i32) -> (i32, i32, i32, i32, i32, i32, i32, i32, i32) {
    if count >= 3 {
        (3, 90, 180, 360, 720, 9, 18, 36, 72)
    } else if count == 2 {
        (2, 95, 190, 380, 760, 7, 14, 28, 56)
    } else {
        (1, 100, 200, 400, 800, 5, 10, 20, 40)
    }
}

fn dwelling_reduction_percent(kind: DwellingKind) -> I32F32 {
    match kind {
        DwellingKind::Tent => FOOD_REDUCTION_TENT_PERCENT,
        DwellingKind::Cottage => FOOD_REDUCTION_COTTAGE_PERCENT,
        DwellingKind::StoneHouse => FOOD_REDUCTION_STONE_HOUSE_PERCENT,
        DwellingKind::Other => FOOD_REDUCTION_DEFAULT_PERCENT,
    }
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

fn apply_population_events_system(
    mut events: EventReader<SetColonyPopulationEvent>,
    mut population: ResMut<ColonyPopulation>,
) {
    for ev in events.read() {
        population.total = ev.total;
    }
}

fn place_market_system(
    mut commands: Commands,
    mut events: EventReader<PlaceMarketEvent>,
    mut rejected: EventWriter<MarketPlacementRejectedEvent>,
    mut claims: ResMut<MarketPlacementClaims>,
    occupancy: Res<TileOccupancy>,
    population: Res<ColonyPopulation>,
) {
    let allowed = max_markets_for_population(population.total);

    for ev in events.read() {
        let anchor = BuildingAnchor {
            x: ev.tile_x,
            y: ev.tile_y,
        };

        if claims.claims.len() >= allowed || !footprint_is_unblocked(anchor, &occupancy) {
            rejected.send(MarketPlacementRejectedEvent {
                tile_x: ev.tile_x,
                tile_y: ev.tile_y,
            });
            continue;
        }

        let entity = commands
            .spawn((
                Market,
                anchor,
                MarketCoreStats::default(),
                MarketBuildState {
                    build_ticks_remaining: build_seconds_to_ticks(MARKET_BUILD_TIME_SECONDS),
                    completed: false,
                },
                MarketEconomy::default(),
                MarketFootprint::default(),
                MarketAura::default(),
                MarketPowerState {
                    connected_to_grid: true,
                },
            ))
            .id();

        claims.claims.insert(entity, anchor);
    }
}

fn market_build_tick_system(mut markets: Query<&mut MarketBuildState, With<Market>>) {
    for mut state in &mut markets {
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

fn apply_market_power_events_system(
    mut events: EventReader<SetMarketPowerEvent>,
    mut markets: Query<&mut MarketPowerState, With<Market>>,
) {
    for ev in events.read() {
        let Ok(mut power) = markets.get_mut(ev.market_entity) else {
            continue;
        };
        power.connected_to_grid = ev.connected_to_grid;
    }
}

fn apply_market_selection_events_system(
    mut events: EventReader<SetMarketSelectedEvent>,
    mut selection: ResMut<MarketSelectionState>,
) {
    for ev in events.read() {
        if ev.selected {
            selection.selected_markets.insert(ev.market_entity);
        } else {
            selection.selected_markets.remove(&ev.market_entity);
        }
    }
}

fn market_trade_tier_system(
    markets: Query<&MarketBuildState, With<Market>>,
    selection: Res<MarketSelectionState>,
    mut trade_tier: ResMut<MarketTradeTier>,
) {
    let mut completed_count = 0i32;
    for state in &markets {
        if state.completed {
            completed_count += 1;
        }
    }

    let (tier_count, buy_wood, buy_stone, buy_iron, buy_oil, sell_wood, sell_stone, sell_iron, sell_oil) =
        market_tier_from_count(completed_count);

    trade_tier.tier_count = tier_count;
    trade_tier.buy_wood = buy_wood;
    trade_tier.buy_stone = buy_stone;
    trade_tier.buy_iron = buy_iron;
    trade_tier.buy_oil = buy_oil;
    trade_tier.sell_wood = sell_wood;
    trade_tier.sell_stone = sell_stone;
    trade_tier.sell_iron = sell_iron;
    trade_tier.sell_oil = sell_oil;

    let selected_count = selection.selected_markets.len();
    trade_tier.click_trade_amount = if selected_count >= 3 {
        15
    } else if selected_count == 2 {
        10
    } else {
        TRADE_BASE_BATCH
    };
}

fn market_dwelling_food_bonus_system(
    markets: Query<(Entity, &BuildingAnchor, &MarketBuildState, &MarketAura, &MarketPowerState), With<Market>>,
    mut dwellings: Query<(&BuildingAnchor, &DwellingBuilding, &mut DwellingFoodBonus)>,
) {
    for (dwelling_anchor, dwelling, mut bonus) in &mut dwellings {
        let mut selected_market: Option<Entity> = None;

        for (market_entity, market_anchor, market_state, aura, power) in &markets {
            if !market_state.completed || !power.connected_to_grid {
                continue;
            }

            if !dwelling_fully_inside_market_aura(*dwelling_anchor, *market_anchor, aura.radius_tiles) {
                continue;
            }

            match selected_market {
                None => selected_market = Some(market_entity),
                Some(existing) => {
                    if market_entity.to_bits() < existing.to_bits() {
                        selected_market = Some(market_entity);
                    }
                }
            }
        }

        if selected_market.is_some() {
            bonus.market_entity = selected_market;
            bonus.reduction_percent = dwelling_reduction_percent(dwelling.kind);
            bonus.effective_food_cost =
                dwelling.base_food_cost * (I32F32::ONE - bonus.reduction_percent);
        } else {
            bonus.market_entity = None;
            bonus.reduction_percent = I32F32::ZERO;
            bonus.effective_food_cost = dwelling.base_food_cost;
        }
    }
}

fn market_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    markets: Query<
        (
            Entity,
            &BuildingAnchor,
            &MarketCoreStats,
            &MarketBuildState,
            &MarketEconomy,
            &MarketFootprint,
            &MarketAura,
            &MarketPowerState,
        ),
        With<Market>,
    >,
    claims: Res<MarketPlacementClaims>,
    population: Res<ColonyPopulation>,
    trade_tier: Res<MarketTradeTier>,
    selection: Res<MarketSelectionState>,
    dwellings: Query<(&BuildingAnchor, &DwellingBuilding, &DwellingFoodBonus), With<DwellingBuilding>>,
) {
    for (entity, anchor, core, build, eco, footprint, aura, power) in &markets {
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
        checksum.accumulate(eco.wood_cost as u64);
        checksum.accumulate(eco.stone_cost as u64);
        checksum.accumulate(eco.iron_cost as u64);
        checksum.accumulate(eco.oil_cost as u64);
        checksum.accumulate(eco.gold_cost as u64);
        checksum.accumulate(eco.workers as u64);

        checksum.accumulate(footprint.size_tiles as u64);
        checksum.accumulate(footprint.watch_range.to_bits() as u64);
        checksum.accumulate(footprint.aoe_width_tiles as u64);
        checksum.accumulate(footprint.aoe_height_tiles as u64);

        checksum.accumulate(aura.radius_tiles as u64);
        checksum.accumulate(u64::from(power.connected_to_grid));
    }

    checksum.accumulate(claims.claims.len() as u64);
    for (entity, anchor) in &claims.claims {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);
    }

    checksum.accumulate(population.total as u64);

    checksum.accumulate(trade_tier.tier_count as u64);
    checksum.accumulate(trade_tier.buy_wood as u64);
    checksum.accumulate(trade_tier.buy_stone as u64);
    checksum.accumulate(trade_tier.buy_iron as u64);
    checksum.accumulate(trade_tier.buy_oil as u64);
    checksum.accumulate(trade_tier.sell_wood as u64);
    checksum.accumulate(trade_tier.sell_stone as u64);
    checksum.accumulate(trade_tier.sell_iron as u64);
    checksum.accumulate(trade_tier.sell_oil as u64);
    checksum.accumulate(trade_tier.click_trade_amount as u64);

    checksum.accumulate(selection.selected_markets.len() as u64);
    for entity in &selection.selected_markets {
        checksum.accumulate(entity.to_bits() as u64);
    }

    for (anchor, dwelling, bonus) in &dwellings {
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);
        checksum.accumulate(dwelling.base_food_cost.to_bits() as u64);
        checksum.accumulate(dwelling.kind as u64);
        checksum.accumulate(bonus.reduction_percent.to_bits() as u64);
        checksum.accumulate(bonus.effective_food_cost.to_bits() as u64);
        checksum.accumulate(bonus.market_entity.map(Entity::to_bits).unwrap_or(0));
    }
}

pub struct MarketPlugin;

impl Plugin for MarketPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ColonyPopulation>()
            .init_resource::<TileOccupancy>()
            .init_resource::<MarketPlacementClaims>()
            .init_resource::<MarketTradeTier>()
            .init_resource::<MarketSelectionState>()
            .add_event::<SetTileBlockedEvent>()
            .add_event::<SetColonyPopulationEvent>()
            .add_event::<SetMarketPowerEvent>()
            .add_event::<SetMarketSelectedEvent>()
            .add_event::<PlaceMarketEvent>()
            .add_event::<MarketPlacementRejectedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    apply_tile_block_events_system,
                    apply_population_events_system,
                    place_market_system,
                    market_build_tick_system,
                    apply_market_power_events_system,
                    apply_market_selection_events_system,
                    market_trade_tier_system,
                    market_dwelling_food_bonus_system,
                    market_checksum_system,
                )
                    .chain(),
            );
    }
}